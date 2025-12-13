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
* `CitadelProofs.Spiral` - SPIRAL slot enumeration and self-assembly
* `CitadelProofs.Convergence` - Topology-first convergent self-assembly (NO FWW)
* `CitadelProofs.Broadcast` - Broadcast protocol with toroidal wrapping and turn-left algorithm

## Main Results

* Every node in the Citadel mesh has exactly 20 connections (6 planar + 2 vertical + 12 extended)
* The hexagonal distance function forms a metric space
* Connection invariants are preserved under all operations
* **Slot occupancy is unique** - at most one node per slot (pigeonhole)
* **Convergent assembly** - nodes self-organize into SPIRAL topology
* **No FWW needed** - deterministic hash selection replaces timestamps
* **Byzantine tolerant** - survives 6/20 malicious neighbors
* **Toroidal correctness** - wrapped coordinates always within bounds
* **No duplicate delivery** - each node receives broadcast exactly once
* **Broadcast termination** - reaches all reachable nodes in finite time
* **Turn-left optimality** - reduces redundant traffic by avoiding backflow
-/

import CitadelProofs.Topology
import CitadelProofs.Spiral
import CitadelProofs.Convergence
import CitadelProofs.Broadcast
