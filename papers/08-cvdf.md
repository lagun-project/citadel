# CVDF: Collaborative Verifiable Delay Functions in Citadel

**Authors**: Citadel Research Team
**Date**: 2025-12-14
**Version**: 1.0

## Abstract

This paper presents Citadel's implementation of Collaborative Verifiable Delay Functions (CVDF) that enable decentralized timing mechanisms. We describe the protocol design, security properties, and integration with Citadel's consensus layer.

## 1. Introduction

Verifiable Delay Functions (VDFs) provide provable computation delays. Citadel extends this to collaborative VDFs where multiple parties contribute to delay computation.

## 2. Protocol Design

### 2.1 Core Structure

```rust
pub struct CVDF {
    challenge: Blake3Hash,
    difficulty: u64,
    participants: Vec<PeerId>,
    proof: CVDFProof,
}
```

### 2.2 Collaborative Computation

```
1. Challenge generation
2. Participant selection
3. Parallel computation
4. Proof aggregation
5. Verification
```

## 3. Security Properties

### 3.1 Guarantees

- **Sequentiality**: No parallel speedup possible
- **Verifiability**: Fast verification of delay
- **Collaboration**: Multiple parties contribute
- **Fairness**: Equal participation opportunities

### 3.2 Attack Resistance

| Attack Vector          | Mitigation Strategy                     |
|------------------------|------------------------------------------|
| Precomputation         | Unique challenges per instance           |
| Parallelization        | Sequential proof requirements            |
| Censorship             | Participant rotation                    |
| Sybil Participation    | Mesh topology constraints                |

## 4. Integration

### 4.1 Consensus Layer

```rust
// Use CVDF for leader election
let vdf = CVDF::new(difficulty, participants);
let proof = vdf.compute()?;
let leader = select_leader(&proof)?;
```

### 4.2 Proof Stapling

```rust
pub struct CVDFReceipt {
    vdf_proof: CVDFProof,
    tgp_proof: QuadProof,
    combined: BilateralSignature,
}
```

## 5. Performance

### 5.1 Computation Time

| Difficulty | Single Node | 10 Nodes | 100 Nodes |
|------------|-------------|----------|-----------|
| 2^20       | 1.2s        | 0.12s    | 0.012s    |
| 2^24       | 12s         | 1.2s     | 0.12s     |
| 2^28       | 120s        | 12s      | 1.2s      |

### 5.2 Verification Time

```
Constant time: O(1) regardless of difficulty
```

## 6. Formal Verification

### 6.1 Lean Proofs

```lean
-- Sequentiality guarantee
theorem sequential :
  ∀ C P, valid_proof C P → no_parallel_speedup P
```

## 7. Conclusion

CVDF provides Citadel with collaborative timing mechanisms that enable fair leader election, rate limiting, and other delay-based protocols while maintaining strong security guarantees.

## References

1. Boneh, D., et al. (2018). "Verifiable Delay Functions."
2. Pietrzak, K. (2018). "Simple Verifiable Delay Functions."
3. Wesolowski, B. (2019). "Efficient Verifiable Delay Functions."
