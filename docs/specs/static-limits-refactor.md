# Static-Limits Compliance Refactor

## Purpose

This session's math-core additions made the repository release gate
(`scripts/static-analysis.sh`) FAIL at `check-static-limits.py`. Bring the code
back into compliance with the repo's structural policy WITHOUT changing any
behavior. This is a pure, behavior-preserving refactor; the extensive existing
tests (including the persistence-distance brute-force oracle and all
exact-assertion tests) are the safety net.

## Policy (hard limits)

- ≤ 700 logical lines per file.
- ≤ 80 logical lines per function.
- ≤ 12 decision points per function.

## Violations to clear

**Structure crate (Task 1):**
- `morphism/mod.rs` — 1547 lines; fns at 898 (146 lines), 1066 (265 lines, **22 decision points**), 1483 (115), 1620 (120).
- `morphism/tests/pushout.rs:5` — test fn (117 lines).
- `topology/distance.rs` — 1172 lines; fns at 199 (81), 458 (**15 decision points**), 559 (**15 decision points**).
- `space/store.rs` — 979 lines.

**Reasoning crate (Task 2):**
- `abstract_interpretation/fixpoint.rs` — 995 lines; fn at 632 (129 lines).
- `gluing.rs` — 767 lines.

## How to refactor (behavior-preserving)

1. **Split over-long files into submodules**, following existing conventions
   (e.g. `morphism/` already has `mod.rs`+`helpers.rs`+`tests/`; `topology/` has
   `analysis.rs`+`analysis/`; `space/` uses sibling files like `order.rs`).
   Suggested (Codex may choose differently as long as constraints hold):
   - `morphism/mod.rs` → keep `mod.rs` as a thin facade; move pushout
     construction to `morphism/pushout.rs`, pullback to `morphism/pullback.rs`,
     pull/pushout candidate + diagram code to existing/new siblings; shared
     helpers stay in `helpers.rs`.
   - `topology/distance.rs` → `topology/distance.rs` facade + `topology/distance/`
     dir (e.g. `bottleneck.rs`, `wasserstein.rs`/`hungarian.rs`, `types.rs`), or
     sibling files.
   - `space/store.rs` → facade + submodule(s) for the pushout/pullback adapters
     and/or complex/coverage method groups.
   - `abstract_interpretation/fixpoint.rs` → facade + submodule(s).
   - `gluing.rs` → `gluing.rs` facade + `gluing/` dir or siblings.
2. **Extract long functions** into smaller, well-named private helpers so each
   function is ≤ 80 logical lines.
3. **Reduce decision points** (the 22- and 15-branch functions —
   `construct_explicit_pullback`, the distance bottleneck/Wasserstein matchers)
   by extracting branch-heavy blocks (obstruction collection, per-class /
   per-pair handling, feasibility inner loops) into helpers. This is pure
   control-flow restructuring; the values computed must be identical.
4. **Split the 117-line test** (`morphism/tests/pushout.rs:5`) into smaller
   focused tests OR extract its fixture setup into a helper — KEEP every
   assertion unchanged.

## Hard constraints (non-negotiable)

- **PUBLIC API PRESERVED.** Every existing public path must still resolve:
  `higher_graphen_structure::morphism::{construct_explicit_pushout, construct_explicit_pullback, PushoutInputs, PushoutOutcome, PushoutConstruction, PullbackInputs, PullbackOutcome, PullbackConstruction, explicit_pushout_candidate, explicit_pullback_candidate, check_diagram_commutativity, Morphism, MorphismType, ...}`,
  `::topology::{persistence_distance, PersistenceDistanceRequest, PersistenceDistanceReport, PersistenceInterval, ...}`,
  `::space::{InMemorySpaceStore, ...}`,
  `higher_graphen_reasoning::abstract_interpretation::{run_fixpoint, AbstractGraph, AbstractDomain, ...}`,
  `::gluing::{attempt_gluing, attempt_structural_gluing, StructuralGluing, ...}`.
  Re-export moved items from the original module path (`pub use`). Downstream
  crates (reasoning, runtime, CLI, tools) and the bundle must compile unchanged.
- **BEHAVIOR-PRESERVING.** All existing tests pass with ASSERTIONS UNCHANGED
  (the pushout test may be split, but no assertion value changes). No logic
  changes — only moves and extractions. Serialized output stays byte-identical.
- No new crate dependencies. No `#[allow(...)]` to suppress the limits. No
  `unwrap`/`expect`/`panic!` on non-test paths.

## Acceptance (per task)

- `python3 scripts/check-static-limits.py` reports **zero violations for the
  files this task owns** (Task 1: the structure files above; Task 2: the
  reasoning files). After BOTH tasks, the full `scripts/static-analysis.sh`
  passes end to end (exit 0).
- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`;
  `cargo test --workspace` (all pass, assertions unchanged);
  `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`.
- Report: the submodule structure chosen, that the public API is unchanged
  (which `pub use` re-exports were added), and verbatim `check-static-limits.py`
  output showing the owned files cleared.
