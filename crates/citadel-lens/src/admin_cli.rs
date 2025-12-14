//! lens-admin CLI tool
//!
//! Manages users, uploaders, and admins for the Lens node.
//!
//! Usage:
//!   lens-admin add-admin <public_key>
//!   lens-admin remove-admin <public_key>
//!   lens-admin grant-upload <public_key>
//!   lens-admin revoke-upload <public_key>
//!   lens-admin list-admins
//!   lens-admin is-admin <public_key>
//!   lens-admin ping

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Admin command sent over the socket.
#[derive(Debug, Serialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum AdminCommand {
    AddAdmin { public_key: String },
    RemoveAdmin { public_key: String },
    GrantUpload { public_key: String },
    RevokeUpload { public_key: String },
    ListAdmins,
    IsAdmin { public_key: String },
    Ping,
}

/// Response from admin command.
#[derive(Debug, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum AdminResponse {
    Ok { message: String },
    Error { error: String },
    List { items: Vec<String> },
    Bool { value: bool },
    Pong,
}

fn print_usage() {
    eprintln!("lens-admin - Manage Lens node users and permissions");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  lens-admin add-admin <public_key>      Add an admin");
    eprintln!("  lens-admin remove-admin <public_key>   Remove an admin");
    eprintln!("  lens-admin grant-upload <public_key>   Grant upload permission");
    eprintln!("  lens-admin revoke-upload <public_key>  Revoke upload permission");
    eprintln!("  lens-admin list-admins                 List all admins");
    eprintln!("  lens-admin is-admin <public_key>       Check if key is admin");
    eprintln!("  lens-admin ping                        Check if daemon is running");
    eprintln!();
    eprintln!("Environment:");
    eprintln!("  LENS_SOCKET  Path to admin socket (default: ./lens-data/admin.sock)");
}

fn get_socket_path() -> PathBuf {
    std::env::var("LENS_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./lens-data/admin.sock"))
}

fn send_command(cmd: AdminCommand) -> Result<AdminResponse, String> {
    let socket_path = get_socket_path();

    let mut stream = UnixStream::connect(&socket_path).map_err(|e| {
        format!(
            "Failed to connect to lens-node at {:?}: {}\n\
             Is the lens-node running?",
            socket_path, e
        )
    })?;

    // Send command
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;
    writeln!(stream, "{}", cmd_json).map_err(|e| e.to_string())?;

    // Read response
    let mut reader = BufReader::new(&stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| e.to_string())?;

    serde_json::from_str(&response_line).map_err(|e| format!("Invalid response: {}", e))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let cmd = match args[1].as_str() {
        "add-admin" => {
            if args.len() < 3 {
                eprintln!("Error: add-admin requires a public_key argument");
                std::process::exit(1);
            }
            AdminCommand::AddAdmin {
                public_key: args[2].clone(),
            }
        }
        "remove-admin" => {
            if args.len() < 3 {
                eprintln!("Error: remove-admin requires a public_key argument");
                std::process::exit(1);
            }
            AdminCommand::RemoveAdmin {
                public_key: args[2].clone(),
            }
        }
        "grant-upload" => {
            if args.len() < 3 {
                eprintln!("Error: grant-upload requires a public_key argument");
                std::process::exit(1);
            }
            AdminCommand::GrantUpload {
                public_key: args[2].clone(),
            }
        }
        "revoke-upload" => {
            if args.len() < 3 {
                eprintln!("Error: revoke-upload requires a public_key argument");
                std::process::exit(1);
            }
            AdminCommand::RevokeUpload {
                public_key: args[2].clone(),
            }
        }
        "list-admins" => AdminCommand::ListAdmins,
        "is-admin" => {
            if args.len() < 3 {
                eprintln!("Error: is-admin requires a public_key argument");
                std::process::exit(1);
            }
            AdminCommand::IsAdmin {
                public_key: args[2].clone(),
            }
        }
        "ping" => AdminCommand::Ping,
        "-h" | "--help" | "help" => {
            print_usage();
            std::process::exit(0);
        }
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
            std::process::exit(1);
        }
    };

    match send_command(cmd) {
        Ok(response) => match response {
            AdminResponse::Ok { message } => {
                println!("{}", message);
            }
            AdminResponse::Error { error } => {
                eprintln!("Error: {}", error);
                std::process::exit(1);
            }
            AdminResponse::List { items } => {
                if items.is_empty() {
                    println!("(none)");
                } else {
                    for item in items {
                        println!("{}", item);
                    }
                }
            }
            AdminResponse::Bool { value } => {
                println!("{}", value);
                if !value {
                    std::process::exit(1);
                }
            }
            AdminResponse::Pong => {
                println!("pong - lens-node is running");
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
