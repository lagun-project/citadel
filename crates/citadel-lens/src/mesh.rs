//! Citadel Mesh Service
//!
//! Implements emergent consensus mesh formation using SPIRAL slot claiming
//! with SPORE (Self-Propagating Object Replication Environment) for all data transfer.
//!
//! # SPORE Principles
//!
//! ALL data transfer uses continuous flooding - no request/response patterns:
//! - Slot claims flood immediately on claim
//! - Peer discovery floods on connection
//! - Admin lists flood on change
//! - Every node propagates everything it learns
//!
//! # Emergent Consensus
//!
//! Nodes join by claiming SPIRAL slots with adaptive consensus:
//! - 2 nodes: bilateral TGP (2/2)
//! - 3 nodes: triad coordination (2/3)
//! - 4 nodes: BFT (3/4)
//! - 5+ nodes: fault-tolerant BFT (scales)
//! - Full mesh: 11/20 neighbor validation
//!
//! # 20-Neighbor Topology
//!
//! Each node connects to exactly 20 neighbors:
//! - 6 planar (hexagonal grid)
//! - 2 vertical (above/below)
//! - 12 extended (diagonal across layers)

use crate::error::Result;
use crate::storage::Storage;
use citadel_protocols::PeerCoordinator;
use citadel_topology::{HexCoord, Neighbors, Spiral3DIndex, spiral3d_to_coord};
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

/// Compute PeerID from ed25519 public key using double-BLAKE3 (Archivist/IPFS style)
/// hash₁ = BLAKE3(pubkey), hash₂ = BLAKE3(hash₁), PeerID = "b3b3/{hash₂}"
pub fn compute_peer_id(pubkey: &VerifyingKey) -> String {
    let hash1 = blake3::hash(pubkey.as_bytes());
    let hash2 = blake3::hash(hash1.as_bytes());
    format!("b3b3/{}", hex::encode(hash2.as_bytes()))
}

/// Compute PeerID from raw public key bytes
pub fn compute_peer_id_from_bytes(pubkey_bytes: &[u8]) -> String {
    let hash1 = blake3::hash(pubkey_bytes);
    let hash2 = blake3::hash(hash1.as_bytes());
    format!("b3b3/{}", hex::encode(hash2.as_bytes()))
}

/// Verify that a claimed PeerID matches the given public key
pub fn verify_peer_id(claimed_id: &str, pubkey: &VerifyingKey) -> bool {
    compute_peer_id(pubkey) == claimed_id
}

/// Mesh node identity and state
#[derive(Debug, Clone)]
pub struct MeshPeer {
    pub id: String,
    pub addr: SocketAddr,
    pub public_key: Option<Vec<u8>>,
    pub last_seen: std::time::Instant,
    pub coordinated: bool,
    /// The SPIRAL slot this peer has claimed (if known)
    pub slot: Option<SlotClaim>,
}

/// A claimed SPIRAL slot in the mesh
#[derive(Debug, Clone)]
pub struct SlotClaim {
    /// SPIRAL index (deterministic ordering)
    pub index: u64,
    /// 3D hex coordinate
    pub coord: HexCoord,
    /// PeerID that claimed this slot
    pub peer_id: String,
    /// Number of validators who confirmed this claim
    pub confirmations: u32,
}

impl SlotClaim {
    /// Create a new slot claim
    pub fn new(index: u64, peer_id: String) -> Self {
        let coord = spiral3d_to_coord(Spiral3DIndex::new(index));
        Self {
            index,
            coord,
            peer_id,
            confirmations: 0,
        }
    }

    /// Get the 20 neighbor coordinates of this slot
    pub fn neighbor_coords(&self) -> [HexCoord; 20] {
        Neighbors::of(self.coord)
    }
}

/// Calculate consensus threshold based on mesh size
/// Implements emergent consensus: starts bilateral, grows to 11/20
pub fn consensus_threshold(mesh_size: usize) -> usize {
    match mesh_size {
        0 | 1 => 1,           // First node auto-claims
        2 => 2,               // Bilateral TGP: 2/2
        3 => 2,               // Triad: 2/3
        4 => 3,               // BFT: 3/4 (can tolerate 1 fault)
        5..=6 => 4,           // Growing BFT
        7..=9 => 5,           // Approaching 2/3
        10..=14 => 7,         // 2/3 + 1 for larger groups
        15..=19 => 9,         // Approaching full mesh
        _ => 11,              // Full mesh: 11/20
    }
}

/// Mesh service state
pub struct MeshState {
    /// Our node ID (PeerID)
    pub self_id: String,
    /// Our signing key for authentication
    pub signing_key: SigningKey,
    /// Our claimed slot in the mesh
    pub self_slot: Option<SlotClaim>,
    /// Known peers in the mesh (by PeerID)
    pub peers: HashMap<String, MeshPeer>,
    /// Claimed slots (by SPIRAL index)
    pub claimed_slots: HashMap<u64, SlotClaim>,
    /// Coordinates with claimed slots (for neighbor lookup)
    pub slot_coords: HashSet<HexCoord>,
    /// Active coordinators for bilateral connections
    pub coordinators: HashMap<String, PeerCoordinator>,
}

impl MeshState {
    /// Find the next available SPIRAL slot
    pub fn next_available_slot(&self) -> u64 {
        let mut index = 0u64;
        while self.claimed_slots.contains_key(&index) {
            index += 1;
        }
        index
    }

    /// Check if a coordinate has a claimed slot
    pub fn is_slot_claimed(&self, coord: &HexCoord) -> bool {
        self.slot_coords.contains(coord)
    }

    /// Get neighbors of our slot that are present in the mesh
    pub fn present_neighbors(&self) -> Vec<&SlotClaim> {
        let Some(ref self_slot) = self.self_slot else {
            return Vec::new();
        };

        self_slot.neighbor_coords()
            .iter()
            .filter_map(|coord| {
                // Find claimed slot at this coordinate
                self.claimed_slots.values()
                    .find(|s| s.coord == *coord)
            })
            .collect()
    }

    /// Count how many of our 20 neighbors are present
    pub fn neighbor_count(&self) -> usize {
        self.present_neighbors().len()
    }
}

/// Broadcast message for continuous flooding
#[derive(Clone, Debug)]
pub enum FloodMessage {
    /// Peer discovery (id, addr, slot_index)
    Peers(Vec<(String, String, Option<u64>)>),
    /// Admin list sync
    Admins(Vec<String>),
    /// Slot claim announcement (index, peer_id, coord as (q, r, z))
    SlotClaim { index: u64, peer_id: String, coord: (i64, i64, i64) },
    /// Slot claim validation response
    SlotValidation { index: u64, peer_id: String, validator_id: String, accepted: bool },
    /// SPORE HaveList - advertise what slots we know about (for targeted sync)
    SporeHaveList { peer_id: String, slots: Vec<u64> },
}

/// Citadel Mesh Service
pub struct MeshService {
    /// P2P listen address
    listen_addr: SocketAddr,
    /// Bootstrap peers to connect to
    bootstrap_peers: Vec<String>,
    /// Shared storage for replication
    storage: Arc<Storage>,
    /// Mesh state
    state: Arc<RwLock<MeshState>>,
    /// Broadcast channel for continuous flooding
    flood_tx: broadcast::Sender<FloodMessage>,
}

impl MeshService {
    /// Create a new mesh service
    pub fn new(
        listen_addr: SocketAddr,
        bootstrap_peers: Vec<String>,
        storage: Arc<Storage>,
    ) -> Self {
        // Generate or load node keypair for peer identity
        let signing_key = storage.get_or_create_node_key()
            .unwrap_or_else(|_| {
                // Fallback: generate ephemeral key
                let mut rng = rand::thread_rng();
                SigningKey::generate(&mut rng)
            });

        // PeerID is double-BLAKE3 hash of ed25519 public key (Archivist/IPFS style)
        let verifying_key = signing_key.verifying_key();
        let self_id = compute_peer_id(&verifying_key);

        info!("Node PeerID: {}", self_id);

        // Create broadcast channel for continuous flooding (capacity for burst)
        let (flood_tx, _) = broadcast::channel(1024);

        Self {
            listen_addr,
            bootstrap_peers,
            storage,
            state: Arc::new(RwLock::new(MeshState {
                self_id,
                signing_key,
                self_slot: None,
                peers: HashMap::new(),
                claimed_slots: HashMap::new(),
                slot_coords: HashSet::new(),
                coordinators: HashMap::new(),
            })),
            flood_tx,
        }
    }

    /// Claim a SPIRAL slot with emergent consensus
    pub async fn claim_slot(&self, index: u64) -> bool {
        let mut state = self.state.write().await;

        // Check if slot is already claimed
        if state.claimed_slots.contains_key(&index) {
            warn!("Slot {} already claimed", index);
            return false;
        }

        let peer_id = state.self_id.clone();
        let claim = SlotClaim::new(index, peer_id.clone());
        let coord = claim.coord;

        // Record our claim
        state.self_slot = Some(claim.clone());
        state.claimed_slots.insert(index, claim.clone());
        state.slot_coords.insert(coord);

        // Calculate required confirmations based on mesh size
        let mesh_size = state.claimed_slots.len();
        let threshold = consensus_threshold(mesh_size);

        info!(
            "Claimed slot {} at ({}, {}, {}) - mesh size {}, threshold {}",
            index, coord.q, coord.r, coord.z, mesh_size, threshold
        );

        drop(state);

        // Flood our claim to the network
        self.flood(FloodMessage::SlotClaim {
            index,
            peer_id,
            coord: (coord.q, coord.r, coord.z),
        });

        true
    }

    /// Process a slot claim from another node
    pub async fn process_slot_claim(&self, index: u64, peer_id: String, coord: (i64, i64, i64)) {
        let mut state = self.state.write().await;
        let hex_coord = HexCoord::new(coord.0, coord.1, coord.2);

        // Verify the coord matches the index
        let expected_coord = spiral3d_to_coord(Spiral3DIndex::new(index));
        if hex_coord != expected_coord {
            warn!("Invalid slot claim: index {} should be at {:?}, not {:?}",
                  index, expected_coord, hex_coord);
            return;
        }

        // Check if already claimed by someone else
        if let Some(existing) = state.claimed_slots.get(&index) {
            if existing.peer_id != peer_id {
                warn!("Slot {} already claimed by {}, rejecting claim from {}",
                      index, existing.peer_id, peer_id);
                return;
            }
        }

        // Accept the claim
        let claim = SlotClaim::new(index, peer_id.clone());
        state.claimed_slots.insert(index, claim);
        state.slot_coords.insert(hex_coord);

        info!("Accepted slot claim {} from {} at ({}, {}, {})",
              index, peer_id, coord.0, coord.1, coord.2);

        // If this peer is connected to us, update their slot info
        if let Some(peer) = state.peers.get_mut(&peer_id) {
            peer.slot = Some(SlotClaim::new(index, peer_id));
        }
    }

    /// Get a receiver for flood messages (for connections to subscribe)
    pub fn subscribe_floods(&self) -> broadcast::Receiver<FloodMessage> {
        self.flood_tx.subscribe()
    }

    /// Broadcast a flood message to all connections
    pub fn flood(&self, msg: FloodMessage) {
        let _ = self.flood_tx.send(msg);
    }

    /// Get current mesh state for API
    pub async fn get_peers(&self) -> Vec<MeshPeer> {
        self.state.read().await.peers.values().cloned().collect()
    }

    /// Get self ID
    pub async fn self_id(&self) -> String {
        self.state.read().await.self_id.clone()
    }

    /// Get the shared mesh state (for API access)
    pub fn mesh_state(&self) -> Arc<RwLock<MeshState>> {
        Arc::clone(&self.state)
    }

    /// Get the flood sender (for admin socket to propagate changes)
    pub fn flood_tx(&self) -> broadcast::Sender<FloodMessage> {
        self.flood_tx.clone()
    }

    /// Run the mesh service
    pub async fn run(self: Arc<Self>) -> Result<()> {
        info!("Starting mesh service on {}", self.listen_addr);

        // Start listener for incoming connections
        let listener = TcpListener::bind(self.listen_addr).await?;
        info!("Mesh P2P listening on {}", self.listen_addr);

        // Spawn task to connect to bootstrap peers and claim slot
        let self_clone = Arc::clone(&self);
        tokio::spawn(async move {
            // First connect to bootstrap peers to learn mesh state
            self_clone.connect_to_bootstrap_peers().await;

            // After learning mesh state, claim next available slot
            let next_slot = self_clone.state.read().await.next_available_slot();
            self_clone.claim_slot(next_slot).await;
        });

        // Accept incoming connections
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("Incoming mesh connection from {}", addr);
                    let self_clone = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = self_clone.handle_connection(stream, addr).await {
                            warn!("Connection error from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }

    /// Connect to bootstrap peers (spawns connections as tasks, doesn't block)
    async fn connect_to_bootstrap_peers(self: &Arc<Self>) {
        for peer_addr in &self.bootstrap_peers {
            info!("Connecting to bootstrap peer: {}", peer_addr);

            match TcpStream::connect(peer_addr).await {
                Ok(stream) => {
                    let addr: SocketAddr = peer_addr.parse().unwrap_or_else(|_| {
                        SocketAddr::from(([127, 0, 0, 1], 9000))
                    });
                    info!("Connected to bootstrap peer: {}", peer_addr);

                    // Spawn connection handler as task - don't block!
                    let self_clone = Arc::clone(self);
                    tokio::spawn(async move {
                        if let Err(e) = self_clone.handle_connection(stream, addr).await {
                            warn!("Bootstrap peer {} error: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    warn!("Failed to connect to bootstrap peer {}: {}", peer_addr, e);
                }
            }
        }
    }

    /// Handle a peer connection
    async fn handle_connection(&self, stream: TcpStream, addr: SocketAddr) -> Result<()> {
        let peer_id = format!("peer-{}", addr.port());

        // Register peer (slot unknown until they announce it via SPORE flood)
        {
            let mut state = self.state.write().await;
            state.peers.insert(
                peer_id.clone(),
                MeshPeer {
                    id: peer_id.clone(),
                    addr,
                    public_key: None,
                    last_seen: std::time::Instant::now(),
                    coordinated: false,
                    slot: None,  // Will be learned via SPORE slot_claim flood
                },
            );
        }

        info!("Peer {} registered", peer_id);

        // Simple protocol: exchange node info and sync state
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send our node info
        let self_id = self.state.read().await.self_id.clone();
        let hello = serde_json::json!({
            "type": "hello",
            "node_id": self_id,
            "addr": self.listen_addr.to_string(),
        });
        writer.write_all(hello.to_string().as_bytes()).await?;
        writer.write_all(b"\n").await?;

        // Flood our complete state to this peer (event-driven, no request/response)
        // Admin list
        if let Ok(admins) = self.storage.list_admins() {
            if !admins.is_empty() {
                let flood_admins = serde_json::json!({
                    "type": "flood_admins",
                    "admins": admins,
                });
                writer.write_all(flood_admins.to_string().as_bytes()).await?;
                writer.write_all(b"\n").await?;
                debug!("Flooded {} admins to peer {}", admins.len(), peer_id);
            }
        }

        // Peer list - flood our complete view of the mesh with slot info
        // SPORE: only flood real peer IDs (b3b3/...), never temp IDs
        {
            let state = self.state.read().await;
            let self_slot = state.self_slot.as_ref().map(|s| s.index);
            let mut all_peers = vec![serde_json::json!({
                "id": state.self_id,
                "addr": self.listen_addr.to_string(),
                "slot": self_slot,
            })];
            for peer in state.peers.values() {
                // Only flood peers with real IDs (b3b3/...), skip temp IDs
                if !peer.id.starts_with("b3b3/") {
                    continue;
                }
                all_peers.push(serde_json::json!({
                    "id": peer.id,
                    "addr": peer.addr.to_string(),
                    "slot": peer.slot.as_ref().map(|s| s.index),
                }));
            }

            let flood_peers = serde_json::json!({
                "type": "flood_peers",
                "peers": all_peers,
            });
            writer.write_all(flood_peers.to_string().as_bytes()).await?;
            writer.write_all(b"\n").await?;

            // Also flood all claimed slots
            for claim in state.claimed_slots.values() {
                let slot_msg = serde_json::json!({
                    "type": "slot_claim",
                    "index": claim.index,
                    "peer_id": claim.peer_id,
                    "coord": [claim.coord.q, claim.coord.r, claim.coord.z],
                });
                writer.write_all(slot_msg.to_string().as_bytes()).await?;
                writer.write_all(b"\n").await?;
            }

            // SPORE: Send our HaveList so peer can identify missing slots
            let have_slots: Vec<u64> = state.claimed_slots.keys().copied().collect();
            let have_list = serde_json::json!({
                "type": "spore_have_list",
                "peer_id": state.self_id,
                "slots": have_slots,
            });
            writer.write_all(have_list.to_string().as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }

        // Subscribe to broadcast floods
        let mut flood_rx = self.flood_tx.subscribe();

        // Track current peer key (may change from peer-{port} to real PeerID)
        let mut current_peer_key = peer_id.clone();

        // Read peer messages and forward floods concurrently
        let mut line = String::new();
        loop {
            line.clear();
            tokio::select! {
                // Handle incoming messages from peer
                read_result = reader.read_line(&mut line) => {
                    match read_result {
                        Ok(0) => {
                            info!("Peer {} disconnected", current_peer_key);
                            break;
                        }
                        Ok(_) => {
                            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                                // handle_message returns Some(real_id) when peer is re-keyed
                                if let Ok(Some(real_id)) = self.handle_message(&current_peer_key, msg, &mut writer).await {
                                    current_peer_key = real_id;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Read error from {}: {}", current_peer_key, e);
                            break;
                        }
                    }
                }
                // Forward broadcast floods to this peer
                flood_result = flood_rx.recv() => {
                    match flood_result {
                        Ok(FloodMessage::Peers(peers)) => {
                            let flood_msg = serde_json::json!({
                                "type": "flood_peers",
                                "peers": peers.into_iter().map(|(id, addr, slot)| {
                                    serde_json::json!({"id": id, "addr": addr, "slot": slot})
                                }).collect::<Vec<_>>(),
                            });
                            let _ = writer.write_all(flood_msg.to_string().as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                        Ok(FloodMessage::Admins(admins)) => {
                            let flood_msg = serde_json::json!({
                                "type": "flood_admins",
                                "admins": admins,
                            });
                            let _ = writer.write_all(flood_msg.to_string().as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                        Ok(FloodMessage::SlotClaim { index, peer_id, coord }) => {
                            let flood_msg = serde_json::json!({
                                "type": "slot_claim",
                                "index": index,
                                "peer_id": peer_id,
                                "coord": [coord.0, coord.1, coord.2],
                            });
                            let _ = writer.write_all(flood_msg.to_string().as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                        Ok(FloodMessage::SlotValidation { index, peer_id, validator_id, accepted }) => {
                            let flood_msg = serde_json::json!({
                                "type": "slot_validation",
                                "index": index,
                                "peer_id": peer_id,
                                "validator_id": validator_id,
                                "accepted": accepted,
                            });
                            let _ = writer.write_all(flood_msg.to_string().as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                        Ok(FloodMessage::SporeHaveList { peer_id, slots }) => {
                            let flood_msg = serde_json::json!({
                                "type": "spore_have_list",
                                "peer_id": peer_id,
                                "slots": slots,
                            });
                            let _ = writer.write_all(flood_msg.to_string().as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                        Err(_) => {
                            // Channel closed or lagged, continue
                        }
                    }
                }
            }
        }

        // Remove peer on disconnect using current key
        {
            let mut state = self.state.write().await;
            state.peers.remove(&current_peer_key);
        }

        Ok(())
    }

    /// Handle incoming message from peer
    /// Returns the real PeerID if learned from hello (for re-keying)
    async fn handle_message(
        &self,
        peer_id: &str,
        msg: serde_json::Value,
        _writer: &mut OwnedWriteHalf,
    ) -> Result<Option<String>> {
        let msg_type = msg.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match msg_type {
            "hello" => {
                debug!("Received hello from {}: {:?}", peer_id, msg);
                // Re-key peer entry with real PeerID
                if let Some(node_id) = msg.get("node_id").and_then(|n| n.as_str()) {
                    let mut state = self.state.write().await;
                    // Remove temporary peer-{port} entry and re-add with real ID
                    if let Some(mut peer) = state.peers.remove(peer_id) {
                        // Only add if we don't already have this peer (avoid duplicates)
                        if node_id != state.self_id && !state.peers.contains_key(node_id) {
                            peer.id = node_id.to_string();
                            peer.last_seen = std::time::Instant::now();
                            state.peers.insert(node_id.to_string(), peer);
                            info!("Peer {} identified as {}", peer_id, node_id);
                            return Ok(Some(node_id.to_string()));
                        }
                    }
                }
            }
            "flood_admins" | "sync_admins" => {
                // Merge flooded admin list into our state
                if let Some(admins) = msg.get("admins").and_then(|a| a.as_array()) {
                    for admin in admins {
                        if let Some(key) = admin.as_str() {
                            let _ = self.storage.set_admin(key, true);
                            info!("Merged admin from {}: {}", peer_id, key);
                        }
                    }
                }
            }
            "flood_peers" | "sync_peers" => {
                // Merge flooded peer list - this propagates mesh topology
                // SPORE: only accept real peer IDs, skip those we already know
                let mut new_peers = Vec::new();
                if let Some(peers) = msg.get("peers").and_then(|p| p.as_array()) {
                    let mut state = self.state.write().await;
                    for peer_info in peers {
                        if let (Some(id), Some(addr_str)) = (
                            peer_info.get("id").and_then(|i| i.as_str()),
                            peer_info.get("addr").and_then(|a| a.as_str()),
                        ) {
                            // SPORE: only accept real peer IDs (b3b3/...)
                            if !id.starts_with("b3b3/") {
                                continue;
                            }

                            // Get slot index if present
                            let slot_index = peer_info.get("slot").and_then(|s| s.as_u64());

                            // Don't add ourselves or peers we already know
                            if id != state.self_id && !state.peers.contains_key(id) {
                                if let Ok(addr) = addr_str.parse() {
                                    let slot = slot_index.map(|idx| SlotClaim::new(idx, id.to_string()));

                                    // Record slot claim if present
                                    if let Some(idx) = slot_index {
                                        if !state.claimed_slots.contains_key(&idx) {
                                            let claim = SlotClaim::new(idx, id.to_string());
                                            state.slot_coords.insert(claim.coord);
                                            state.claimed_slots.insert(idx, claim);
                                        }
                                    }

                                    state.peers.insert(
                                        id.to_string(),
                                        MeshPeer {
                                            id: id.to_string(),
                                            addr,
                                            public_key: None,
                                            last_seen: std::time::Instant::now(),
                                            coordinated: false,
                                            slot,
                                        },
                                    );
                                    new_peers.push((id.to_string(), addr_str.to_string(), slot_index));
                                    info!("Discovered peer {} (slot {:?}) via flood from {}", id, slot_index, peer_id);
                                }
                            }
                        }
                    }
                }
                // Re-flood newly discovered peers to propagate through mesh
                if !new_peers.is_empty() {
                    self.flood(FloodMessage::Peers(new_peers));
                }
            }
            "slot_claim" => {
                // Process a slot claim from another node
                // SPORE: re-flood new claims to propagate through mesh
                if let (Some(index), Some(claimer_id)) = (
                    msg.get("index").and_then(|i| i.as_u64()),
                    msg.get("peer_id").and_then(|p| p.as_str()),
                ) {
                    let coord = msg.get("coord")
                        .and_then(|c| c.as_array())
                        .map(|arr| {
                            let q = arr.first().and_then(|v| v.as_i64()).unwrap_or(0);
                            let r = arr.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
                            let z = arr.get(2).and_then(|v| v.as_i64()).unwrap_or(0);
                            (q, r, z)
                        })
                        .unwrap_or((0, 0, 0));

                    // Check if this is a new claim before processing
                    let is_new = !self.state.read().await.claimed_slots.contains_key(&index);

                    self.process_slot_claim(index, claimer_id.to_string(), coord).await;

                    // Re-flood new claims to propagate through mesh
                    if is_new {
                        self.flood(FloodMessage::SlotClaim {
                            index,
                            peer_id: claimer_id.to_string(),
                            coord,
                        });
                    }
                }
            }
            "slot_validation" => {
                // Process a slot validation response
                if let (Some(index), Some(claimer_id), Some(_validator_id), Some(accepted)) = (
                    msg.get("index").and_then(|i| i.as_u64()),
                    msg.get("peer_id").and_then(|p| p.as_str()),
                    msg.get("validator_id").and_then(|v| v.as_str()),
                    msg.get("accepted").and_then(|a| a.as_bool()),
                ) {
                    if accepted {
                        let mut state = self.state.write().await;
                        if let Some(claim) = state.claimed_slots.get_mut(&index) {
                            if claim.peer_id == claimer_id {
                                claim.confirmations += 1;
                                debug!("Slot {} now has {} confirmations",
                                       index, claim.confirmations);
                            }
                        }
                    }
                }
            }
            "spore_have_list" => {
                // SPORE: Compare their HaveList with ours and send missing slots
                if let Some(their_slots) = msg.get("slots").and_then(|s| s.as_array()) {
                    let their_slots: std::collections::HashSet<u64> = their_slots
                        .iter()
                        .filter_map(|v| v.as_u64())
                        .collect();

                    let state = self.state.read().await;

                    // Find slots we have that they don't
                    let mut missing_slots = Vec::new();
                    for (index, claim) in &state.claimed_slots {
                        if !their_slots.contains(index) {
                            missing_slots.push(claim.clone());
                        }
                    }
                    drop(state);

                    // Send missing slots to this peer
                    if !missing_slots.is_empty() {
                        info!("SPORE: Sending {} missing slots to {}", missing_slots.len(), peer_id);
                        for claim in missing_slots {
                            self.flood(FloodMessage::SlotClaim {
                                index: claim.index,
                                peer_id: claim.peer_id,
                                coord: (claim.coord.q, claim.coord.r, claim.coord.z),
                            });
                        }
                    }
                }
            }
            _ => {
                debug!("Unknown message type from {}: {}", peer_id, msg_type);
            }
        }

        Ok(None)
    }
}
