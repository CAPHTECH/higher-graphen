# Public CaseGraphen Workspace

This directory is intentionally part of the public HigherGraphen repository.
It records the development of HigherGraphen as a CaseGraphen case graph: goals,
tasks, decisions, evidence, blockers, completion candidates, and verification
events.

The purpose is not only project management. This workspace is a public
reasoning trace for HigherGraphen itself. External readers should be able to
inspect how the product was decomposed, which constraints were used, what was
verified, and which future cases remain open.

## Role

HigherGraphen treats reports, dashboards, and summaries as projections over a
deeper structure. This workspace applies the same idea to the repository's own
development process.

The case graph is useful for:

- Understanding why the repository was built in this order.
- Inspecting the boundary between completed foundation work and future work.
- Seeing how agent-facing CLI, schema, skill, and product-package decisions
  were made.
- Studying concrete examples of tasks, evidence, blockers, completions, and
  projections as public product artifacts.

Official product documentation still lives under `docs/`. The CaseGraphen
workspace is a structured trace of work and decisions, not a replacement for
the curated documentation set.

## Tracked Content

These files are intended to be public and versioned:

- `workspace.yaml`: CaseGraphen workspace metadata.
- `config.yaml`: repository-local CaseGraphen configuration.
- `cases/<case-id>/case.yaml`: current case projection.
- `cases/<case-id>/events.jsonl`: append-only event log for the case.
- `cases/<case-id>/attachments/`: public evidence attachments when the
  attachment is safe to publish.

Open cases are allowed in this directory. They represent the public roadmap and
future work for this repository, not private commercial planning.

## Ignored Content

Runtime and private-local content must stay out of Git:

- `.casegraphen/.lock`
- `.casegraphen/cache/`
- `.casegraphen/tmp/`
- `.casegraphen/local/`
- `.casegraphen/private/`
- SQLite cache files and SQLite sidecar files

These paths are ignored from the repository root `.gitignore`.

## Publication Rules

Commit content here only when it is intended to be public.

Allowed:

- Public HigherGraphen development cases.
- Public roadmap cases for this repository.
- Public design decisions and scope boundaries.
- Public verification evidence.
- Synthetic examples and reference scenarios.
- Public attachments that help readers understand the work.

Not allowed:

- API keys, tokens, credentials, private keys, or secrets.
- Customer-specific data or private engagement records.
- Private commercial strategy, pricing, sales pipeline, or negotiation detail.
- Personal information that is not already intended for publication.
- Proprietary interpretation packages meant for a private or commercial repo.
- Local runtime state, caches, locks, or database files.

If a case needs private context, keep that case outside this repository or place
only a public summary here.

## Authoring Rules

Use the `cg` CLI for CaseGraphen mutations. Do not hand-edit
`cases/<case-id>/events.jsonl`; it is the append-only source of truth.

`case.yaml` is a projection derived from events. If it drifts from the event
log, rebuild or validate the workspace with CaseGraphen tooling instead of
manually patching the projection.

Useful inspection commands:

```sh
cg case list
cg case show --case <case-id>
cg frontier --case <case-id> --format json
cg blockers --case <case-id> --format json
cg validate --case <case-id> --format json
cg validate storage
```

Before publishing new CaseGraphen content, check that no private-local files
are staged and run a targeted secret scan over `.casegraphen/`.

## Reading Order

For repository history, start with closed foundation cases:

- `hg_impl_foundation`
- `hg_runtime_cli_foundation`
- `hg_casegraphen_tool_foundation`
- `hg_architecture_end_to_end_reference`

For current direction, inspect open cases. Open cases are public work items and
may change as the project evolves.
