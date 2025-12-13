import Lake
open Lake DSL

package «citadel-proofs» where
  version := v!"0.1.0"
  leanOptions := #[
    ⟨`pp.unicode.fun, true⟩,
    ⟨`autoImplicit, false⟩
  ]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git"

@[default_target]
lean_lib «CitadelProofs» where
  roots := #[`CitadelProofs]
  globs := #[Glob.submodules `CitadelProofs]
