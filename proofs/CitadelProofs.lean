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
* `CitadelProofs.Spore` - SPORE: Succinct Proof of Range Exclusions (optimal sync)

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
* **SPORE optimality** - encoding size ∝ boundary count (information-theoretic bound)
* **Implicit exclusion** - gaps never sync, zero encoding cost
* **Symmetry** - both empty and full nodes have O(1) SPORE size

## SPORE Extended Theorems (from paper)

* **Sync Bilateral Construction** (Thm 7.1) - Both nodes verify sync completion independently
* **Expected Boundaries** (Thm 8.1) - O(n) worst, O(1) best, O(√n) average
* **Byzantine Safety** (Thm 8.2) - 3f+1 nodes tolerate f Byzantine faults
* **Dynamic Convergence** (Thm 8.3) - Stable state within bounded time after modifications
* **Coverage Monotonicity** (Lemma 6.1) - Coverage never decreases in cooperative network
* **Self-Optimization** (Thm 6.2) - Each successful sync reduces future overhead
* **Convergence to Zero** (Thm 6.3) - Total WantList size converges to zero at steady state
* **Hierarchical SPORE** - Regional aggregation for networks >10,000 nodes (data structures defined)
-/

import CitadelProofs.Topology
import CitadelProofs.Spiral
import CitadelProofs.Convergence
import CitadelProofs.Broadcast
import CitadelProofs.Spore
