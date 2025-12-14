//! Unix socket server for admin commands.
//!
//! Provides a local IPC interface for managing users, uploaders, and admins.

use crate::error::Result;
use crate::mesh::FloodMessage;
use crate::storage::Storage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;

/// Admin command sent over the socket.
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum AdminCommand {
    /// Add an admin
    AddAdmin { public_key: String },
    /// Remove an admin
    RemoveAdmin { public_key: String },
    /// Grant upload permission
    GrantUpload { public_key: String },
    /// Revoke upload permission
    RevokeUpload { public_key: String },
    /// List all admins
    ListAdmins,
    /// Check if a key is admin
    IsAdmin { public_key: String },
    /// Ping (health check)
    Ping,
}

/// Response from admin command.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AdminResponse {
    Ok { message: String },
    Error { error: String },
    List { items: Vec<String> },
    Bool { value: bool },
    Pong,
}

/// Admin socket server.
pub struct AdminSocket {
    storage: Arc<Storage>,
    socket_path: String,
    flood_tx: Option<broadcast::Sender<FloodMessage>>,
}

impl AdminSocket {
    /// Create a new admin socket server.
    pub fn new(storage: Arc<Storage>, socket_path: &str) -> Self {
        Self {
            storage,
            socket_path: socket_path.to_string(),
            flood_tx: None,
        }
    }

    /// Set the flood sender for mesh propagation.
    pub fn with_flood_tx(mut self, tx: broadcast::Sender<FloodMessage>) -> Self {
        self.flood_tx = Some(tx);
        self
    }

    /// Run the admin socket server.
    pub async fn run(&self) -> Result<()> {
        // Remove existing socket file if present
        let _ = std::fs::remove_file(&self.socket_path);

        let listener = UnixListener::bind(&self.socket_path)?;
        tracing::info!("Admin socket listening on {}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let storage = Arc::clone(&self.storage);
                    let flood_tx = self.flood_tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, storage, flood_tx).await {
                            tracing::error!("Admin connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to accept admin connection: {}", e);
                }
            }
        }
    }

    /// Get the socket path.
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }
}

async fn handle_connection(
    stream: UnixStream,
    storage: Arc<Storage>,
    flood_tx: Option<broadcast::Sender<FloodMessage>>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let response = match serde_json::from_str::<AdminCommand>(&line) {
            Ok(cmd) => execute_command(cmd, &storage, &flood_tx),
            Err(e) => AdminResponse::Error {
                error: format!("Invalid command: {}", e),
            },
        };

        let response_json = serde_json::to_string(&response)? + "\n";
        writer.write_all(response_json.as_bytes()).await?;
        line.clear();
    }

    Ok(())
}

/// Flood the current admin list to the mesh
fn flood_admins(storage: &Arc<Storage>, flood_tx: &Option<broadcast::Sender<FloodMessage>>) {
    if let Some(tx) = flood_tx {
        if let Ok(admins) = storage.list_admins() {
            let _ = tx.send(FloodMessage::Admins(admins));
            tracing::info!("Flooded admin list to mesh");
        }
    }
}

fn execute_command(
    cmd: AdminCommand,
    storage: &Arc<Storage>,
    flood_tx: &Option<broadcast::Sender<FloodMessage>>,
) -> AdminResponse {
    match cmd {
        AdminCommand::AddAdmin { public_key } => {
            match storage.set_admin(&public_key, true) {
                Ok(()) => {
                    tracing::info!("Added admin: {}", public_key);
                    // Flood updated admin list to mesh
                    flood_admins(storage, flood_tx);
                    AdminResponse::Ok {
                        message: format!("Added admin: {}", public_key),
                    }
                }
                Err(e) => AdminResponse::Error {
                    error: e.to_string(),
                },
            }
        }

        AdminCommand::RemoveAdmin { public_key } => {
            match storage.set_admin(&public_key, false) {
                Ok(()) => {
                    tracing::info!("Removed admin: {}", public_key);
                    // Flood updated admin list to mesh
                    flood_admins(storage, flood_tx);
                    AdminResponse::Ok {
                        message: format!("Removed admin: {}", public_key),
                    }
                }
                Err(e) => AdminResponse::Error {
                    error: e.to_string(),
                },
            }
        }

        AdminCommand::GrantUpload { public_key } => {
            match storage.grant_permission(&public_key, "upload") {
                Ok(()) => {
                    tracing::info!("Granted upload to: {}", public_key);
                    AdminResponse::Ok {
                        message: format!("Granted upload permission to: {}", public_key),
                    }
                }
                Err(e) => AdminResponse::Error {
                    error: e.to_string(),
                },
            }
        }

        AdminCommand::RevokeUpload { public_key } => {
            match storage.revoke_permission(&public_key, "upload") {
                Ok(()) => {
                    tracing::info!("Revoked upload from: {}", public_key);
                    AdminResponse::Ok {
                        message: format!("Revoked upload permission from: {}", public_key),
                    }
                }
                Err(e) => AdminResponse::Error {
                    error: e.to_string(),
                },
            }
        }

        AdminCommand::ListAdmins => {
            match storage.list_admins() {
                Ok(admins) => AdminResponse::List { items: admins },
                Err(e) => AdminResponse::Error {
                    error: e.to_string(),
                },
            }
        }

        AdminCommand::IsAdmin { public_key } => {
            match storage.is_admin(&public_key) {
                Ok(is_admin) => AdminResponse::Bool { value: is_admin },
                Err(e) => AdminResponse::Error {
                    error: e.to_string(),
                },
            }
        }

        AdminCommand::Ping => AdminResponse::Pong,
    }
}

/// Default socket path.
pub fn default_socket_path() -> String {
    let data_dir = std::env::var("LENS_DATA_DIR").unwrap_or_else(|_| "./lens-data".to_string());
    format!("{}/admin.sock", data_dir)
}
