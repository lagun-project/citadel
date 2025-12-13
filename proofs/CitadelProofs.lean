/-
Copyright (c) 2025 Lagun Project. All rights reserved.
Released under AGPL-3.0-or-later license.
Authors: Lagun Project Contributors
-/

/-!
# Citadel Proofs

This is the root module for all Citadel formal proofs.

## Modules

* `CitadelProofs.Topology` - Hexagonal mesh topology with 20-connection invariant
* `CitadelProofs.Spore` - Succinct Proof Of Range Exclusion
* `CitadelProofs.Dht` - Distributed Hash Table properties
* `CitadelProofs.Crdt` - Conflict-free Replicated Data Types
* `CitadelProofs.Consensus` - Consensus algorithm correctness
* `CitadelProofs.Protocols` - Protocol properties and safety

## Main Results

* Every node in the Citadel mesh has exactly 20 connections (6 planar + 2 vertical + 12 extended)
* The hexagonal distance function forms a metric space
* Connection invariants are preserved under all operations
-/

import CitadelProofs.Topology
import CitadelProofs.Spiral
