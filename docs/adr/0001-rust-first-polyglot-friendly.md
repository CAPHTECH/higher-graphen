# ADR 0001: Rust-First, Polyglot-Friendly Core

## Status

Accepted as initial direction.

## Context

HigherGraphen is intended to be a long-lived structural core for AI-native
higher-structure reasoning. The core must represent complex data models,
perform structural operations, support consistency checks, and remain usable
from AI, web, and research environments.

The main implementation options considered were Rust, TypeScript, Python,
OCaml, Haskell, and Scala.

## Decision

HigherGraphen will use a Rust-first, polyglot-friendly architecture.

The core model and core engines should be implemented in Rust. Python,
TypeScript, WebAssembly, and Node integrations should be provided through
bindings and adapters.

## Rationale

Rust is the best fit for the core because it provides:

- Strong type safety for structural data models.
- Good performance for large structure spaces.
- Ownership and borrowing tools for safe structural operations.
- WebAssembly compatibility.
- Practical binding paths for Python and Node.
- Good long-term maintainability for library distribution.

TypeScript remains preferred for Studio UI, API gateway, and developer
experience. Python remains preferred for AI, ML, notebooks, and exploratory
research.

## Consequences

Positive consequences:

- The core can remain robust as the model grows.
- Web, Python, and Node environments can still use the system.
- Product packages can share a stable structural foundation.

Tradeoffs:

- Initial implementation speed may be slower than a TypeScript-only or
  Python-only prototype.
- Binding and packaging work must be planned as first-class engineering work.
- The core model must avoid overfitting to one frontend or orchestration
  language.

## Non-Goals

This decision does not require every component to be written in Rust.

The intended split is:

```text
Rust:
  Structural data model
  Structural operations
  Consistency checks
  Obstruction, completion, and projection core

TypeScript:
  Studio UI
  Web API gateway
  Developer experience

Python:
  AI, ML, notebooks, and exploratory analysis
```

