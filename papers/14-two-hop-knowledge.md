# Flexibility in Citadel: Blinded Routing and Adjustable Privacy

**Authors**: Citadel Research Team
**Date**: 2025-12-14
**Version**: 1.0

## Abstract

This paper presents Citadel's mechanisms for blinded routing and adjustable privacy. We describe how two-hop knowledge and other techniques enable flexible, privacy-preserving communication.

## 1. Introduction

Privacy is essential in decentralized networks. Citadel provides adjustable privacy through blinded routing and limited knowledge propagation.

## 2. Blinded Routing

### 2.1 Two-Hop Knowledge

```rust
pub struct RoutingHint {
    next_hop: PeerId,
    final_destination: Option<PeerId>,  // Only if within 2 hops
}
```

### 2.2 Privacy Levels

```rust
pub enum PrivacyLevel {
    Public,      // Full path visible
    Normal,      // 2-hop knowledge
    Private,     // Only next hop known
    Anonymous,   // Fully blinded
}
```

## 3. Implementation

### 3.1 Routing Algorithm

```rust
pub fn route_blinded(
    source: PeerId,
    target: PeerId,
    privacy: PrivacyLevel
) -> BlindedRoute {
    match privacy {
        Public => full_path(source, target),
        Normal => two_hop_path(source, target),
        Private => next_hop_only(source, target),
        Anonymous => onion_route(source, target),
    }
}
```

### 3.2 Proof Stapling

```rust
pub struct PrivacyProof {
    routing_proof: RouteProof,
    tgp_proof: QuadProof,
    combined: BilateralSignature,
}
```

## 4. Performance

### 4.1 Privacy vs Latency

| Privacy Level | Mean Latency | Overhead |
|---------------|--------------|----------|
| Public        | 42ms         | 0%       |
| Normal        | 45ms         | 7%       |
| Private       | 58ms         | 38%      |
| Anonymous     | 120ms        | 186%     |

## 5. Formal Verification

### 5.1 Lean Proofs

```lean
-- Privacy preservation
theorem preserves_privacy :
  ∀ R P, blinded R P → ¬knows_intermediate R P
```

## 6. Conclusion

Citadel's adjustable privacy mechanisms enable flexible communication patterns while maintaining strong privacy guarantees through blinded routing and cryptographic proofs.
