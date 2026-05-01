# AI Agent Integration

HigherGraphen should provide packages and tools for software projects, and it
should also provide agent-facing integrations so AI systems can use those tools
correctly.

This document defines the agent integration layer at a product architecture
level. Provider-specific marketplace and plugin manifest formats should be
verified when implementation begins because those formats can change.

## Delivery Layers

HigherGraphen has three delivery layers:

```text
Core packages
  Reusable libraries embedded by other projects.

Tools
  CLI, MCP server, SDK commands, workflows, and reports built on the packages.

Agent integrations
  Skills, plugin bundles, marketplace entries, prompts, and workflow guides
  that teach AI agents when and how to use the tools.
```

The agent integration layer is not a replacement for the core packages or
tools. It is the operational surface that makes those capabilities usable by AI
agents.

## Why Agent Integrations Are Required

AI agents need more than executable commands. They need:

- When to use each tool.
- Which input format to prepare.
- Which command or API endpoint to call.
- How to interpret structured output.
- How to preserve provenance and review status.
- When a generated completion must remain a candidate.
- Which projections are safe for a human, an AI agent, or an external system.

Without this layer, HigherGraphen would expose powerful primitives but leave
agent behavior underspecified.

## Integration Surfaces

| Surface | Purpose |
| --- | --- |
| CLI | Human and agent-friendly command execution. |
| MCP server | Tool discovery and structured calls from AI agents. |
| SDK | Programmatic use from Rust, Python, TypeScript, and other environments. |
| Skills | Procedural guidance for agents: when to use a tool, what files to inspect, and what output to produce. |
| Plugin bundle | Packaged tools, skills, metadata, scripts, and optional MCP configuration. |
| Marketplace entry | Discoverability, installation metadata, category, authentication policy, and product availability. |
| Prompt templates | Task-specific prompts for common workflows and projections. |
| Schemas | Stable JSON schemas for inputs, outputs, reports, candidates, and obstructions. |

## Current CLI Skill Path

The first executable agent integration path is intentionally smaller than the
full strategy above:

```text
highergraphen CLI -> JSON report schema -> repository-owned skill
```

The current repository skill is
[`skills/highergraphen/SKILL.md`](../../skills/highergraphen/SKILL.md). Agents
should use it with `scripts/validate-cli-report-contract.py` and the
Architecture Product smoke report schema. MCP servers, provider plugin bundles,
marketplace metadata, and provider-specific manifests remain future optional
work that should consume the same CLI and schema contract.

## Recommended Repository Layout

The repository should keep agent integrations separate from core crates:

```text
higher-graphen/
  crates/
    higher-graphen-core/
    higher-graphen-structure/
    ...

  tools/
    highergraphen-cli/
    mcp-server/
    casegraphen/
    morphographen/
    ...

  integrations/
    claude/
      skills/
        highergraphen/
        casegraphen/
        morphographen/
        architecture-review/
      plugins/
        highergraphen/
      marketplace/

    codex/
      skills/
        highergraphen/
        casegraphen/
        morphographen/
        architecture-review/
      plugins/
        highergraphen/
      marketplace/

    generic-mcp/
      schemas/
      examples/
```

The exact folder names can change when provider packaging rules are confirmed.
The important boundary is that provider-specific manifests should not leak into
core crates.

## Agent-Facing Naming Contract

Agent-facing names must follow the repository naming contract in
[`package-boundaries.md`](package-boundaries.md):

| Surface | Name rule | Example |
| --- | --- | --- |
| Umbrella skill | Lowercase product name without punctuation. | `highergraphen` |
| Intermediate tool skill | Bare `*graphen` tool name. | `casegraphen` |
| Architecture workflow skill | Domain workflow name, not a package name. | `architecture-review` |
| Plugin bundle directory | Provider directory plus umbrella bundle name. | `integrations/claude/plugins/highergraphen/` |
| Marketplace display name | Product casing for humans. | `HigherGraphen` |
| CLI command family | Tool command names mirror intermediate tool names. | `casegraphen`, `morphographen` |
| MCP server name | Product-scoped service name. | `highergraphen-mcp` |

Provider-specific identifiers may require a namespace or reverse-DNS prefix at
implementation time. Such identifiers should derive from the names above
instead of introducing a second naming family.

## Minimum Agent Skill Set

The first agent skill set should cover the primary HigherGraphen workflows.

| Skill | Agent should use it when | Main tools used |
| --- | --- | --- |
| `highergraphen` | The task is to run or interpret the first HigherGraphen Architecture Product smoke workflow, or later to model a target world as higher structure. | Current `highergraphen` CLI, report schema, and contract validator; future MCP structural operations. |
| `casegraphen` | The task is to create, compare, inspect, or complete structured cases. | Case tooling, space tooling, projection tooling. |
| `morphographen` | The task is to check transformations, mappings, migrations, or preservation. | Morphism tooling, invariant tooling. |
| `contextgraphen` | The task involves context boundaries, semantic mismatch, local/global consistency, or gluing. | Context tooling, obstruction tooling. |
| `invariantgraphen` | The task asks what must be preserved or whether a change is safe. | Invariant checks, morphism preservation checks. |
| `obstructiongraphen` | The task asks why a plan, structure, or transformation cannot hold. | Obstruction engine, counterexample projection. |
| `completiongraphen` | The task asks what is missing or which candidate structure should be proposed. | Completion engine, review workflow. |
| `evidencegraphen` | The task involves claims, evidence, contradiction, provenance, or confidence. | Evidence tooling, projection tooling. |
| `projectiongraphen` | The task asks for a report, AI view, human view, or audit view. | Projection engine and schema renderers. |
| `architecture-review` | The task is an architecture analysis reference workflow. | Case, context, invariant, obstruction, completion, evidence, projection tools. |

Skills should be concise. Detailed schemas and examples should live in
references that the agent loads only when needed.

## Plugin Bundle Responsibilities

A HigherGraphen agent plugin bundle should include:

- Tool metadata.
- Skill metadata.
- Tool command definitions.
- Optional MCP server configuration.
- JSON schemas for stable inputs and outputs.
- Example tasks and expected projections.
- Safety guidance for reviewable completions and evidence handling.
- Marketplace metadata for discoverability.

The plugin should not embed the full theoretical documentation. It should point
agents to the smallest procedural guidance needed for each task.

## MCP Capability Contract

The MCP surface should expose operations at the level of structural intent, not
only low-level storage primitives.

Initial capabilities:

| Capability | Purpose |
| --- | --- |
| `create_space` | Create a structural universe for a target world. |
| `add_cells` | Add typed cells with provenance. |
| `add_incidences` | Add relations between cells. |
| `define_context` | Define local vocabulary and validity scope. |
| `define_morphism` | Define a transformation or mapping. |
| `check_invariants` | Evaluate invariants and return obstructions. |
| `detect_obstructions` | Find structured impossibility or inconsistency. |
| `propose_completions` | Generate reviewable completion candidates. |
| `accept_completion` | Promote an accepted completion into structure. |
| `reject_completion` | Record rejected completion with reason. |
| `project` | Produce a view for a target audience and purpose. |
| `explain_obstruction` | Explain a failure through a selected projection. |

Every operation that creates or changes structure should accept provenance and
review metadata.

## Agent Safety Rules

Agent integrations must preserve these rules:

- Do not treat AI-inferred structure as accepted fact.
- Do not accept a completion candidate without explicit review policy.
- Do not hide information loss in projections.
- Do not collapse context-specific terms into one global meaning without a
  context mapping.
- Do not report an invariant as preserved unless a check has established it.
- Do not present unsupported claims as evidence-backed conclusions.

These rules should be repeated in skills because they directly affect agent
behavior.

## Reference Workflow: Architecture Review

An architecture review agent workflow should look like this:

```text
1. Lift architecture input into cells, incidences, contexts, and provenance.
2. Build or load an Architecture Product interpretation package.
3. Check boundary, ownership, requirement, and projection invariants.
4. Produce obstructions with counterexamples where possible.
5. Propose completion candidates for missing APIs, tests, constraints, or
   ownership definitions.
6. Project the result into an architecture review report.
7. Keep accepted facts, AI inferences, and reviewable candidates separate.
```

## Implementation Order

Recommended order:

1. Define stable CLI and JSON schemas for the Architecture Product scenario.
2. Create the repository-owned CLI skill and no-network contract validation
   path.
3. Add an MCP server around the same operations only after the CLI plus skill
   contract is stable.
4. Package the CLI, optional MCP metadata, schemas, and skills into a plugin
   bundle.
5. Add marketplace metadata only after the plugin structure is stable.
6. Repeat for the primary intermediate tools.

This order keeps provider-specific packaging behind a stable tool contract.
