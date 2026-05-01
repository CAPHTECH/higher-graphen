# HigherGraphen Core Contracts

This document fixes the implementation contract for `higher-graphen-core`
before downstream crates depend on it. It refines the shared-type section of
[`rust-core-model.md`](rust-core-model.md) and the core crate responsibility in
[`package-boundaries.md`](package-boundaries.md).

## Scope

`higher-graphen-core` owns primitives that every model and engine crate can use
without depending on a product package, tool, binding, app, or UI layer.

The crate owns:

- `Id`
- `SourceRef`
- `Provenance`
- `Confidence`
- `Severity`
- `ReviewStatus`
- Shared error types and result aliases
- Serialization contracts for the shared primitives

The crate does not own higher-structure entities such as `Space`, `Cell`,
`Morphism`, `Invariant`, `Obstruction`, `CompletionCandidate`, or `Projection`.
Those belong to the downstream model crates named in
[`package-boundaries.md`](package-boundaries.md).

## Id

`Id` is an opaque, stable identifier for HigherGraphen structures.

Contract:

- It must be non-empty after normalization.
- It must be safe to clone, compare, order, hash, and serialize.
- It must remain stable across process boundaries, snapshots, bindings, and
  projection outputs.
- Its public API must not require consumers to parse semantic meaning from the
  identifier string.
- It may use a string-backed newtype or another portable representation, but
  the serialized form must be a string.

Product-specific prefixes are allowed only as display or generation policy.
Core code must not branch on prefixes such as architecture, contract, evidence,
agent, repository, issue, or ticket identifiers.

## SourceRef

`SourceRef` records where an observed or inferred structure came from.

Contract:

- It must support the source categories listed by
  [`rust-core-model.md`](rust-core-model.md): document, log, API, human, AI,
  code, external source, and custom extensions.
- It must be portable across Rust, Python, TypeScript, WebAssembly, and JSON
  boundaries.
- It may carry optional URI, title, capture time, and source-local identifier
  fields.
- It must not expose provider SDK types, file handles, database handles, HTTP
  clients, or UI objects.
- It must be descriptive, not authoritative. Trust and acceptance are recorded
  by `Provenance` and `ReviewStatus`.

Capture times, when present, must serialize to a stable text representation
such as RFC 3339 rather than a platform-specific debug format.

## Confidence

`Confidence` is a numeric score for extracted or inferred structure.

Contract:

- Its valid range is inclusive `0.0` through `1.0`.
- It must reject NaN, infinity, and values outside the valid range at public
  construction and deserialization boundaries.
- It represents confidence in extraction or inference, not review acceptance,
  impact, severity, or probability of runtime failure.
- It must not be used as a substitute for `ReviewStatus`.

The implementation may wrap `f32` or `f64`, but the public type must prevent
invalid floating-point states from leaking into downstream crates.

## Severity

`Severity` classifies impact.

Canonical values:

- `Low`
- `Medium`
- `High`
- `Critical`

Contract:

- The ordering is `Low < Medium < High < Critical`.
- Serialization must use stable names that bindings can map without inspecting
  Rust enum formatting.
- Severity must describe impact only. Confidence, review acceptance, and source
  trust must stay separate.
- Domain-specific scales such as business priority, CVSS, SLA tier, risk
  matrix labels, or product roadmap priority must be mapped outside core.

## ReviewStatus

`ReviewStatus` records human or workflow review state.

Canonical values:

- `Unreviewed`
- `Reviewed`
- `Rejected`
- `Accepted`

Contract:

- New AI-inferred or machine-extracted structures should default to
  `Unreviewed` unless an explicit caller supplies a stronger state.
- `Accepted` means the structure may be treated as accepted fact by downstream
  workflows.
- `Rejected` means the structure must not be silently promoted or used as an
  accepted fact.
- `Reviewed` means a review occurred but did not produce accepted or rejected
  state.
- Core may provide helper predicates, but product-specific review workflows
  belong outside core.

Completion and inference engines must not silently promote inferred structure
to accepted fact. Promotion requires an explicit review action in the relevant
downstream crate or runtime layer.

## Provenance

`Provenance` ties a structure to source, extraction, confidence, reviewer, and
review status.

Contract:

- It must include a `SourceRef`.
- It must include `Confidence`.
- It must include `ReviewStatus`.
- It may include extraction method, extractor identity, reviewer identity,
  review time, and notes.
- It must remain usable for observed facts, AI inferences, completion
  candidates, projections, and imported external structures.
- It must not encode product-specific evidence graphs, audit trails, issue
  workflows, or domain review states.

Evidence graphs, contradiction support, audit trails, and temporal causality
belong to `higher-graphen-evidence`, `higher-graphen-evolution`, tools, or
product packages. Core only provides the primitive provenance record they can
reference.

## Error Policy

Core errors must be structured and machine-readable.

Contract:

- Public fallible APIs must return a core-owned `Result<T>` alias or
  `Result<T, CoreError>`.
- Public APIs must not expose `anyhow::Error`, boxed dynamic errors, or untyped
  string errors.
- Error variants must carry stable error codes or variant names suitable for
  bindings and tests.
- Display text is diagnostic and must not be the only contract consumers can
  inspect.
- Invalid primitive construction, parse failures, range violations,
  unsupported serialized versions, and malformed required fields are core
  errors.
- Invariant violations, consistency failures, obstructions, and completion
  rejections are domain results owned by downstream crates, not generic core
  errors.
- Core must not panic for invalid external input. Panics are reserved for
  internal invariant violations that indicate a programming bug.

`thiserror` may be used to implement ergonomic Rust errors, but downstream
bindings must rely on structured error data rather than formatted strings.

## Serialization And Serde Boundary

Serialization is part of the core contract because the repository is
Rust-first and polyglot-friendly.

Contract:

- Core shared primitives must implement `serde::Serialize` and
  `serde::Deserialize`.
- Serialized field names must be stable and use lower snake case.
- Serialized enum values must be stable and language-neutral.
- Deserialization must enforce the same validation as public constructors.
- Round-tripping through JSON must preserve valid primitive values.
- Core may define primitive serialization helpers, but it must not select the
  storage engine, API transport, or binding packaging format.
- Core should depend on `serde` for traits. Format-specific dependencies such
  as `serde_json` belong in storage, runtime, tests, examples, or adapters
  unless a narrowly scoped primitive boundary explicitly requires them.

Do not use `serde_json::Value` or another catch-all payload as a way to move
domain semantics into core. Opaque metadata and product payload schemas must be
owned by the downstream crate or product package that interprets them.

## Feature Flags

The initial `higher-graphen-core` contract defines no feature flags.

Rules for future flags:

- Do not add domain, product, provider, app, or UI feature flags to core.
- A feature flag must not change the meaning, validation rules, or serialized
  schema of existing core primitives.
- Optional dependencies must only add capabilities such as convenience
  conversions or target-specific integration.
- Any new feature flag must be documented here before downstream crates rely on
  it.

Because serialization primitives are a core responsibility, `serde` support is
baseline behavior rather than a domain feature.

## Downstream Dependency Rules

`higher-graphen-core` is the inward dependency anchor for the workspace.

Contract:

- Every HigherGraphen model or engine crate may depend on
  `higher-graphen-core`.
- `higher-graphen-core` must not depend on `higher-graphen-structure::space`,
  `higher-graphen-structure::morphism`, `higher-graphen-structure::context`,
  `higher-graphen-reasoning::invariant`, `higher-graphen-reasoning::obstruction`,
  `higher-graphen-reasoning::completion`, `higher-graphen-projection`,
  `higher-graphen-interpretation`, or `higher-graphen-runtime`.
- `higher-graphen-core` must not depend on bindings, tools, apps, product
  packages, UI frameworks, provider SDKs, or AI-agent runtimes.
- Downstream crates must reuse core primitives instead of defining competing
  `Id`, `Confidence`, `Severity`, `ReviewStatus`, or provenance types.
- Downstream crates may wrap core primitives for local ergonomics, but wrappers
  must preserve the core validation and serialization contract.
- Dependency cycles involving core are forbidden.

This keeps low-level model crates reusable and prevents operational tools or
product packages from leaking provider-specific concepts into the shared model.

## Domain Concept Boundary

Core must not absorb domain-specific concepts.

Forbidden examples in `higher-graphen-core` include:

- Architecture concepts such as service, bounded context, repository, module,
  API endpoint, deployment, or incident.
- Contract concepts such as clause, party, obligation, jurisdiction, or term.
- Project-management concepts such as ticket, milestone, sprint, owner, or
  priority.
- AI-agent concepts such as prompt, tool call, memory, run, thread, or model.
- CaseGraphen or intermediate-tool concepts such as case, task, evidence node,
  blocker, frontier, or event log.

Such concepts belong in interpretation packages, product packages, runtime
adapters, or intermediate tools. Core may provide `Id`, `SourceRef`,
`Provenance`, `Confidence`, `Severity`, `ReviewStatus`, and error/serialization
primitives that those packages use.

## Implementation Checklist

Before downstream crates rely on `higher-graphen-core`, implementation should
provide:

- Validated constructors or parsers for `Id` and `Confidence`.
- Stable serde round-trip tests for `Id`, `SourceRef`, `Provenance`,
  `Severity`, and `ReviewStatus`.
- Error tests showing invalid IDs, invalid confidence values, unknown enum
  values, and malformed provenance fail with structured core errors.
- Dependency checks showing core has no dependency on downstream HigherGraphen
  crates, product packages, tools, apps, or bindings.
- Documentation examples that use abstract HigherGraphen structures rather than
  product-specific domains.
