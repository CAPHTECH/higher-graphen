# Graph Traversal API

This document defines the reusable graph traversal surface for
`higher-graphen-space`.

## Purpose

HigherGraphen packages often need to ask structural questions such as:

- Can cell `A` reach cell `C` through typed incidences?
- Which path witnesses the chain `A -> B -> C`?
- Does a fixed layer pattern such as `Requirement -> Design -> Implementation`
  exist inside a space?
- Where does traversal stop when the target is not reachable?

Before this API, those questions were implemented directly inside
product-specific reasoners. Native `casegraphen` now projects `CaseSpace` into a
`higher-graphen-space` graph view and uses the traversal API for hard relation
targets and frontier exclusion. The same operation is reusable for case, state,
morphism, obstruction, completion, projection, and correspondence tools.

## Package Boundary

The API lives in `higher-graphen-space`.

This is intentional:

- `higher-graphen-core` owns primitive identifiers and errors, but not graph
  storage.
- `higher-graphen-space` owns `Space`, `Cell`, and `Incidence`, so it is the
  correct layer for graph reachability over stored structure.
- Product packages and tools should consume this API rather than duplicating
  bespoke graph scans.

## Public Surface

`InMemorySpaceStore` exposes three traversal operations:

```rust
store.reachable(&query) -> Result<ReachabilityResult>
store.walk_paths(&query) -> Result<Vec<GraphPath>>
store.matches_path_pattern(&pattern) -> Result<Vec<PathPatternMatch>>
```

`reachable` performs breadth-first search and returns the shortest witness path
when the target is reachable.

`walk_paths` returns simple paths between the query endpoints. It respects
`TraversalOptions::max_depth` and `TraversalOptions::max_paths`.

`matches_path_pattern` detects fixed, single-edge-per-layer chains. It is the
API for questions like "does this space contain an `A -> B -> C` chain where
each layer has a required cell type and relation type?"

## Traversal Semantics

`TraversalDirection` controls directed incidence handling:

| Direction | Directed incidence behavior |
| --- | --- |
| `Outgoing` | Follow `from_cell_id -> to_cell_id`. |
| `Incoming` | Follow `to_cell_id -> from_cell_id`. |
| `Both` | Follow either direction. |

Undirected incidences can be traversed from either endpoint.

`TraversalOptions::relation_types` filters incidences by relation type. An empty
list accepts all relation types.

`ReachabilityResult` includes:

- `reachable`
- `shortest_path`
- `visited_cell_ids`
- `frontier_cell_ids`

The path witness is deliberately part of the result because AI operators and
obstruction reports need an explainable chain, not only a boolean answer.

## Layer Pattern Example

```rust
let pattern = PathPattern::new(
    space_id,
    CellPattern::any().of_type("layer.requirement"),
)
.then(
    PathPatternSegment::new(CellPattern::any().of_type("layer.design"))
        .with_relation_type("maps_to"),
)
.then(
    PathPatternSegment::new(CellPattern::any().of_type("layer.implementation"))
        .with_relation_type("implements"),
);

let matches = store.matches_path_pattern(&pattern)?;
```

This detects chains shaped like:

```text
Requirement --maps_to--> Design --implements--> Implementation
```

Each match returns both the `GraphPath` witness and the matched cell identifiers.

## Non-goals

The MVP traversal API does not yet provide:

- weighted shortest paths
- transitive closure materialization
- query planning across multiple stores
- graph database backends
- arbitrary regular path expressions

Those belong in a future query package if richer graph reasoning becomes large
enough to justify a separate crate.
