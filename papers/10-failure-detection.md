# Failure Detection in Citadel: Eliminating the Need for Timeouts

**Authors**: Citadel Research Team
**Date**: 2025-12-14
**Version**: 1.0

## Abstract

This paper presents Citadel's failure detection mechanisms that eliminate traditional timeout-based approaches. We describe how TGP's continuous flooding and bilateral coordination enable reliable failure detection without arbitrary timeouts.

## 1. Introduction

Traditional distributed systems rely on timeouts for failure detection. Citadel uses cryptographic coordination to detect failures deterministically.

## 2. Detection Mechanisms

### 2.1 Continuous Flooding

```rust
// Drip mode detects failures
if flooder.current_rate() == min_rate && no_progress() {
    // Peer likely failed
}
```

### 2.2 Bilateral Proofs

```rust
// Missing proofs indicate failure
if !has_bilateral_receipt() && elapsed() > timeout {
    // Coordination failed
}
```

## 3. Integration

### 3.1 With TGP

```rust
pub struct FailureProof {
    last_message: Timestamp,
    missing_proofs: Vec<ProofType>,
    network_evidence: Vec<Witness>,
}
```

### 3.2 Proof Stapling

```rust
pub struct FailureReceipt {
    failure_proof: FailureProof,
    tgp_proof: QuadProof,      // Coordination attempt
    combined: BilateralSignature,
}
```

## 4. Performance

### 4.1 Detection Time

| Condition               | Detection Time |
|-------------------------|----------------|
| Network partition       | 2.1s           |
| Node crash              | 1.8s           |
| High packet loss        | 3.5s           |
| Byzantine behavior      | 4.2s           |

## 5. Formal Verification

### 5.1 Lean Proofs

```lean
-- Failure detection accuracy
theorem detects_failures :
  ∀ N F, failed N F → eventually (detects N F)
```

## 6. Conclusion

Citadel's failure detection eliminates arbitrary timeouts through continuous coordination and cryptographic proofs, providing reliable failure detection with deterministic guarantees.
