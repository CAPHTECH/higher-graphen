# Static Analysis Policy

This policy defines the minimum checks that must exist before implementation
work is treated as complete. It applies to every Rust crate, tool, binding, and
app in the HigherGraphen repository.

## Baseline Gates

Every implementation change must pass these gates for each package it touches:

| Gate | Required command or check | Passing condition |
| --- | --- | --- |
| Formatting | `cargo fmt --all --check` or package-scoped equivalent | No rustfmt changes are pending. |
| Rust compiler | `cargo check -p <package>` | The touched package compiles without errors. |
| Clippy | `cargo clippy -p <package> --all-targets -- -D warnings` | No clippy warnings remain. |
| Tests | Package-specific unit, integration, or snapshot tests | The package's relevant test set passes. |
| Dependency direction | Workspace dependency audit or manifest check | Package dependencies follow [`package-boundaries.md`](package-boundaries.md). |

Implementation tasks must not be marked complete until the package-specific
checks for every touched package pass. If a check is unavailable, the task must
record the blocker and cannot claim the missing check as passed.

The workspace gate is:

```sh
sh scripts/static-analysis.sh
```

The gate runs `cargo fmt --all --check`, `cargo check --workspace`,
locked workspace compile/test/clippy/doc checks, the local
`scripts/check-static-limits.py` checker, generated CLI report contract
validation, JSON schema fixture conformance and report coverage validation, and
the CLI skill bundle smoke check.

## Size Limits

Keep units small enough for review, AI-assisted maintenance, and focused tests.

| Unit | Soft limit | Hard limit |
| --- | --- | --- |
| Rust function or method body | 50 logical lines | 80 logical lines |
| Rust module file | 400 logical lines | 700 logical lines |
| Test helper function | 70 logical lines | 100 logical lines |
| Markdown spec file | 300 logical lines | 600 logical lines |

Soft-limit violations require a short justification in review notes. Hard-limit
violations require splitting the unit before the task is complete, unless the
team explicitly accepts a temporary exception.

## Complexity Policy

Prefer a local approximation until a dedicated complexity tool is selected.
A Rust function is considered too complex when any two of these conditions are
true:

- It has more than 5 nested control-flow levels.
- It has more than 8 decision points, counting `if`, `else if`, `match` arms
  with guards, loop conditions, early-return branches, and boolean `&&` or `||`
  chains.
- It requires more than 3 independent domain concepts to understand.
- Its tests need more than 5 setup facts before the behavior under test is
  visible.

When the approximation flags a function, split by domain concept, introduce a
small helper, or move package-specific policy into an explicit type. Do not hide
complexity behind generic utilities unless the utility has a stable local use
case in at least two packages.

## Dependency Direction

Static checks must enforce the package direction described in
[`package-boundaries.md`](package-boundaries.md):

- Low-level model crates must not depend on product packages, bindings, apps,
  tools, or UI code.
- Core packages may be used by tools and integrations, but tools and
  integrations must not leak provider-specific types back into core crates.
- Package manifests must not introduce cycles across crate families.
- Optional bindings must depend inward on core packages, not laterally on apps.

Until an automated dependency checker exists, reviewers must inspect changed
`Cargo.toml` files and any generated dependency graph for direction violations.

## Package-Specific Verification

Each package must declare its local verification command set before substantial
implementation begins. The command set should include:

- Format, compile, clippy, and test commands.
- Any feature flags or target triples required for the package.
- Snapshot, schema, fixture, or golden-file checks owned by the package.
- Known external services or generated artifacts needed for verification.
- A clear fallback when an expensive check cannot run locally.

Task evidence should name the exact package commands that passed. A workspace
check can supplement package checks, but it does not replace them when a task
touches a specific package.

## Completion Rule

An implementation task is complete only when:

1. All changed Rust code is formatted by rustfmt.
2. Clippy runs with warnings denied for every touched package.
3. Function, file, and local complexity limits are satisfied or explicitly
   waived.
4. Dependency direction remains valid.
5. Package-specific verification passes for every touched package.
6. Any unavailable check is recorded as a blocker rather than treated as a pass.
