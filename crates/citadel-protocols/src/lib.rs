//! Citadel Protocols - Reliable Bilateral Coordination
//!
//! This crate provides reliable coordination protocols for peer connections
//! in the Citadel mesh network, built on the Two Generals Protocol (TGP).
//!
//! # Overview
//!
//! The core abstraction is [`PeerCoordinator`], which wraps TGP to provide
//! bilateral coordination between two peers. This enables:
//!
//! - Reliable peer connection establishment
//! - Coordinated state transitions
//! - Cryptographic proof of mutual agreement
//!
//! # Two Generals Protocol
//!
//! TGP provides deterministically failsafe coordination through epistemic
//! proof escalation (C → D → T → Q phases). The protocol guarantees:
//!
//! - **Symmetric outcomes**: Both parties either ATTACK or ABORT together
//! - **No special messages**: Any copy of a proof suffices (flooding-friendly)
//! - **Bilateral construction**: If one party can construct Q, so can the other
//!
//! # Example
//!
//! ```rust,ignore
//! use citadel_protocols::{PeerCoordinator, CoordinatorConfig};
//!
//! // Create coordinators for two peers
//! let mut alice = PeerCoordinator::new(
//!     alice_keypair,
//!     bob_public_key,
//!     CoordinatorConfig::default(),
//! );
//! let mut bob = PeerCoordinator::new(
//!     bob_keypair,
//!     alice_public_key,
//!     CoordinatorConfig::default(),
//! );
//!
//! // Exchange messages until coordination achieved
//! while !alice.is_coordinated() || !bob.is_coordinated() {
//!     for msg in alice.get_messages() {
//!         bob.receive(&msg)?;
//!     }
//!     for msg in bob.get_messages() {
//!         alice.receive(&msg)?;
//!     }
//! }
//!
//! // Both peers can now proceed with coordinated action
//! assert!(alice.can_proceed());
//! assert!(bob.can_proceed());
//! ```

pub mod coordinator;
pub mod error;

pub use coordinator::{CoordinatorConfig, CoordinatorState, FloodRateConfig, PeerCoordinator};
pub use error::{Error, Result};

// Re-export core TGP types for convenience
pub use two_generals::{
    crypto::{KeyPair, PublicKey, Signature},
    Decision, Message, ProtocolState as TgpState, QuadProof,
};
