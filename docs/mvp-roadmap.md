# MVP Roadmap

## MVP Purpose

The MVP should prove that HigherGraphen can connect abstract higher structure to
a useful product output. It should not attempt to implement the full long-term
operating system at once.

## MVP Scope

The MVP should implement:

- Space
- Cell
- Incidence
- Complex
- Morphism
- Invariant
- Constraint
- Obstruction
- CompletionCandidate
- Projection
- InterpretationPackage

## MVP Package Set

The MVP package set is:

- `higher-graphen-core`
- `higher-graphen-structure`
- `higher-graphen-projection`
- `higher-graphen-evidence`
- `higher-graphen-reasoning`
- `higher-graphen-interpretation`
- `higher-graphen-runtime`

## Reference Product

The first reference product should be the Architecture Product.

Rationale:

- Architecture inputs are relatively structured.
- Components, APIs, databases, dependencies, requirements, and tests map well
  to cells.
- Invariants, obstructions, and completions can produce visible value quickly.
- The scenario connects directly to real development work.

## Roadmap

### Phase 0: Concept Spec

Target duration: 2 to 4 weeks.

Deliverables:

- Core concept document.
- Rust model specification.
- Package boundary design.
- Architecture Product scenario.

### Phase 1: Core Kernel MVP

Target duration: 1 to 2 months.

Implementation:

- `higher-graphen-core`
- `higher-graphen-structure`
- `higher-graphen-projection`
- `higher-graphen-reasoning`

Deliverables:

- Rust workspace.
- Core model.
- Simple in-memory store.
- CLI query.

### Phase 2: Interpretation and Projection MVP

Target duration: 1 to 2 months.

Implementation:

- `higher-graphen-interpretation`
- `higher-graphen-projection`
- Architecture Interpretation Package.

Deliverables:

- Interpretation package loader.
- Projection renderer.
- Architecture review output.

### Phase 3: Completion MVP

Target duration: 1 to 2 months.

Implementation:

- `higher-graphen-reasoning::completion`
- Completion rule engine.
- Accept and reject workflow.

Deliverables:

- Missing API detector.
- Missing test detector.
- Completion candidate review CLI or UI.

### Phase 4: Bindings and Studio

Target duration: 2 to 4 months.

Implementation:

- Python bindings.
- WebAssembly bindings.
- Studio UI.
- Examples.

Deliverables:

- Python notebook usage.
- Web playground.
- Documentation site.

### Phase 5: Agent Integrations

Target duration: 1 to 2 months after the first stable tool contracts.

Implementation:

- CLI schemas for agent use.
- MCP server for structured agent calls.
- First agent skills for HigherGraphen and Architecture Product workflows.
- Plugin bundle containing tools, schemas, skills, and metadata.
- Marketplace metadata after the provider-specific package shape is confirmed.

Deliverables:

- Agent can run the Architecture Product scenario through structured tools.
- Agent can distinguish accepted facts, AI inferences, and completion
  candidates.
- Agent can produce projections with declared information loss.

## Success Criteria

The MVP is successful when:

1. Rust can represent Space, Cell, Complex, and Morphism.
2. An Interpretation Package can give domain meaning to the abstract structure.
3. An invariant violation can be detected as an Obstruction.
4. A CompletionCandidate can be proposed for missing structure.
5. A Projection can generate human-readable output.
6. The Architecture Product reference scenario works end to end.
7. The same core appears reusable for at least one additional product package.

## Recommended Stack

Rust core:

- `serde`
- `serde_json`
- `thiserror`
- `anyhow`
- `async-trait`
- `petgraph` or a custom structure
- `indexmap`
- `uuid`
- `tokio`

Initial storage:

- In-memory store
- JSON or MessagePack snapshot

Future storage candidates:

- SQLite
- PostgreSQL
- SurrealDB
- RocksDB
- Graph database integration
- Vector database integration
- Object storage

Bindings:

- Python: PyO3 and maturin
- WebAssembly: wasm-bindgen
- Node or TypeScript: napi-rs or WebAssembly

Studio:

- Tauri with TypeScript, or
- Web app with WebAssembly
