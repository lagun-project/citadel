# Citadel
Peer-to-peer, scalable, self-organizing consensus.

## Organization
Citadel is a modular *monorepo* - a single repository containing multiple crates and modules.

citadel/
  ├── Cargo.toml          # Workspace
  ├── proofs/             # Lean4 proofs for EVERYTHING
  ├── CitadelProofs.lean  # Parent lean proof that calls the other proofs
  │   ├── topology/
  │   │   └── Topology.lean
  │   ├── spore/
  │   │   └── Spore.lean
  │   ├── dht/
  │   │   └── Dht.lean
  │   ├── crdt/
  │   │   └── Crdt.lean
  │   ├── consensus/
  │   │   └── Consensus.lean
  │   └── protocols/
  │       └── Protocols.lean
  └── crates/
      ├── citadel-topology - Citadel mesh topology crate. Handles all assembly and topology concerns.
      ├── citadel-spiral - SPIRAL mesh assembly. Allows nodes to join and leave the network, without breaking the mesh.
      ├── citadel-spore - Succinct Proof Of Range Exclusion. Efficiently proves that a range of values is not present in a set.
      ├── citadel-dht - Distributed Hash Table. Efficiently stores and retrieves data across a network.
      ├── citadel-vis - Citadel visualization crate. Provides a browser-based visualization of the Citadel network.
      ├── citadel-protocols - Citadel protocols crate. Implements the Citadel protocol for rich synchronization and 
      ├── citadel-crdt - Conflict-free Replicated Data Type. Provides a way to store and synchronize data across a network.
      ├── citadel-consensus - Core consensus crate. Implements the Citadel consensus algorithm for reliable and resilient network operation.
      ├── citadel-utils - Utility crate. Provides various utility functions and types used throughout Citadel.
      ├── citadel-cli - Command-line interface for Citadel. Provides a command-line interface for interacting with the Citadel network.
      ├── citadel-integration-tests - Helpers for integration testing. Provides utilities for testing Citadel's integration with other systems.
      ├── citadel-logging - Citadel logging crate. Provides a logging framework for Citadel, optionally forwarding logs through Citadel to a chosen service.
      ├── citadel-metrics - Citadel metrics crate. Provides a metrics framework for Citadel.
      └── citadel-wasm - Citadel WebAssembly crate.
