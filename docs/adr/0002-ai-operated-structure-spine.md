# ADR 0002: AI-Operated Structure Spine

## Status

Accepted after AdvisoryGraphen review on 2026-05-09.

## Context

HigherGraphen is intended to be operated primarily by AI agents over structured
concepts, with human-facing reports, dashboards, and review screens produced as
projections. The repository already contains separate core, structure,
projection, evidence, reasoning, interpretation, runtime, and CLI surfaces.

The design question reviewed was whether HigherGraphen's basic design should be
centered on a broad application surface, a library-first abstraction set, or an
AI-operated structural spine that executable tools and agent skills can use.

## Decision

HigherGraphen's basic design will use this spine:

```text
Core primitives and extension lifecycle states
  -> Structure, evidence, and reasoning crates
  -> Interpretation packages
  -> Runtime workflows
  -> CLI commands and JSON report schemas
  -> Agent skills and projections
```

Human UI, MCP servers, SDK bindings, and apps are consumers of this spine. They
must preserve source boundaries, provenance, review status, and projection loss
rather than introducing separate product truth.

## Rationale

The accepted design follows the current repository evidence:

- The README defines HigherGraphen as an AI-native higher-structure framework
  and names `Space`, `Cell`, `Context`, `Morphism`, `Invariant`,
  `Obstruction`, `CompletionCandidate`, `Projection`, and
  `InterpretationPackage` as agent-operable product concepts.
- `package-boundaries.md` defines the reusable crate responsibilities and
  dependency direction.
- `runtime-workflow-contract.md` defines report-producing runtime workflows
  where domain findings are successful structured results.
- `ai-agent-integration.md` defines the current executable path as
  `highergraphen CLI -> JSON report schema -> repository-owned skill`.
- The current runtime and CLI expose concrete workflow/report surfaces before
  broad MCP, binding, or app surfaces.

## Consequences

Positive consequences:

- New capabilities have an executable contract for AI agents, not just Rust
  types or prose.
- Human-facing views remain projections with declared information loss.
- Runtime, CLI, schema, and skill work can be evaluated against one spine.
- MCP servers, bindings, and apps can be added later without changing the
  source-of-truth model.

Tradeoffs:

- Some library abstractions should wait until at least one runtime workflow and
  report contract needs them.
- Core extension objects must carry lifecycle, review, validation, and
  projection semantics, otherwise they become passive abstract records.
- Higher-priority proposals require explicit hypothesis refinement and review
  before they are treated as accepted design facts.

## Design Rules

New workflow capabilities are design-complete only when they provide:

- a bounded input schema or source adapter;
- runtime report data with stable serialization;
- a CLI command that emits JSON;
- schema or report contract validation;
- agent skill guidance for when to use and how to interpret the output;
- explicit provenance, review status, and projection loss.

Core extension work must maintain the operation contract matrix in
[`../specs/core-extension-operation-contract.md`](../specs/core-extension-operation-contract.md).

## Deferred Scope

MCP servers, Python/WASM/Node bindings, and Studio or app surfaces are not part
of the immediate basic-design critical path. They are later consumers that must
preserve the accepted spine and its review/projection rules.
