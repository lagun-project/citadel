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
* `CitadelProofs.TwoHopKnowledge` - Three knowledge modes (Mix/Smart/Full) with greedy routing

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

### Section 3: Core Protocol Theorems
* **XOR Cancellation (Identical)** - XOR of identical SPOREs is empty
* **XOR Boundary Cancellation** (Thm 3.1) - |A ⊕ B| ≤ k_A + k_B - 2m (matching ranges cancel)
* **Fundamental Sync Equation** - sync_cost(A, B) = O(|A ⊕ B|) ≠ O(|A| + |B|)
* **Convergence Dominates** (Cor 3.2) - XOR → 0 regardless of absolute boundary count
* **Two-Bucket Axiom** (Sec 3.7) - Universe partitions into HAVE/WANT/EXCLUDED (binary predicates)
* **Binary Sync Decision** - Send = MyHave ∩ TheirWant, Receive = TheirHave ∩ MyWant

### Section 4: Optimality Theorems
* **Information-Theoretic Lower Bound** (Thm 4.2) - Interval-union needs ≥ k×256 bits
* **SPORE Achieves Bound** (Thm 4.3) - SPORE uses exactly 256 bits per boundary
* **Global Optimality** - SPORE achieves Θ(|A ⊕ B|) sync cost (information-theoretic optimum)

### Section 6: Convergence Theorems
* **Coverage Monotonicity** (Lemma 6.1) - Coverage never decreases in cooperative network
* **Self-Optimization** (Thm 6.2) - Each successful sync reduces future overhead
* **Convergence to Zero** (Thm 6.3) - Total WantList size converges to zero at steady state

### Section 6.6: Why Boundary Explosion Doesn't Matter
* **XOR Cancellation Property** - Matching coverage produces empty XOR
* **Boundary Explosion is a Mirage** - Differential cost converges to zero
* **Self-Healing Defragmentation** - Every sync reduces fragmentation
* **Summary** - At equilibrium, |A ⊕ B| = 0 for all pairs

### Section 7-8: Integration and Practical Theorems
* **Sync Bilateral Construction** (Thm 7.1) - Both nodes verify sync completion independently
* **Expected Boundaries** (Thm 8.1) - O(n) worst, O(1) best, O(√n) average
* **Byzantine Safety** (Thm 8.2) - 3f+1 nodes tolerate f Byzantine faults
* **Dynamic Convergence** (Thm 8.3) - Stable state within bounded time after modifications
* **Hierarchical SPORE** - Regional aggregation for networks >10,000 nodes (data structures defined)

## Three Knowledge Modes (TwoHopKnowledge)

The profound insight: **No node needs complete knowledge of the world for all nodes to have complete reachability.**

### Mix Mode (Local Only)
* Storage: O(k) = 20 neighbors
* Routing: Greedy forward to closest neighbor
* Guarantee: Always makes progress toward target (SPIRAL property)
* No global knowledge required!

### Smart Mode (2-Hop + On-Demand)
* Storage: O(k²) = ~400 peers
* Routing: 2-hop neighborhood + direct queries
* Extra benefit: Mesh health verification

### Full Mode (Complete Knowledge via SPORE)
* Storage: O(n) eventually (via SPORE convergence)
* Routing: O(1) direct addressing
* Benefit: Optimal routing when available

The hierarchy: Mix ⊂ Smart ⊂ Full. Lower modes always work as fallback.
-/

import CitadelProofs.Topology
import CitadelProofs.Spiral
import CitadelProofs.Convergence
import CitadelProofs.Broadcast
import CitadelProofs.Spore
import CitadelProofs.TwoHopKnowledge
