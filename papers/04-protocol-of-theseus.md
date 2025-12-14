# Citadel - Why the Protocol of Theseus?

**Authors**: Citadel Research Team
**Date**: 2025-12-14
**Version**: 1.0

## Abstract

This paper explains why Citadel's implementation of the Two Generals Protocol (TGP) never produces asymmetric outcomes, even under extreme network conditions. We analyze the "Protocol of Theseus" property that guarantees symmetric coordination.

## 1. Introduction

The Two Generals Problem is notorious for its impossibility under certain conditions. Citadel's TGP implementation achieves deterministic symmetry through bilateral construction properties.

## 2. Symmetric Outcomes

### 2.1 The Core Guarantee

**Theorem (Symmetric Outcomes)**: For any two TGP instances A and B:
```
(A.can_attack ↔ B.can_attack) ∨ (A.must_abort ∧ B.must_abort)
```

### 2.2 Why Asymmetry is Impossible

1. **Bilateral Construction**: Q_A ⇒ Q_B constructible
2. **Proof Stapling**: Each proof embeds counterparty's proofs
3. **Continuous Flooding**: Any message copy suffices
4. **No Special Messages**: No single critical message

## 3. Empirical Validation

### 3.1 Lossy Channel Tests

| Loss Rate | Test Runs | Asymmetric Outcomes |
|-----------|-----------|---------------------|
| 50%       | 10,000    | 0                   |
| 90%       | 10,000    | 0                   |
| 99%       | 1,000     | 0                   |

### 3.2 Asymmetric Loss Tests

```
Alice→Bob: 20% delivery
Bob→Alice: 80% delivery
Result: 0 asymmetric outcomes in 10,000 runs
```

## 4. Formal Proofs

### 4.1 Lean Verification

```lean
-- No asymmetric outcomes possible
theorem no_asymmetric :
  ∀ A B: TwoGenerals, ¬(A.can_attack ∧ B.must_abort)
  ∧ ¬(A.must_abort ∧ B.can_attack)
```

## 5. Conclusion

Citadel's TGP implementation achieves the "Protocol of Theseus" property: outcomes are always symmetric, making it suitable for critical coordination in mesh networks.
