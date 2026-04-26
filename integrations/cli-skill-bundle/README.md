# HigherGraphen CLI Skill Bundle

This provider-neutral bundle packages the current HigherGraphen CLI skill
surface for agents. It is intentionally smaller than a provider plugin: it
contains skill files, contract references, metadata, and a local smoke check.

MCP servers, provider marketplace publication, provider SDK integrations, and
provider-specific manifests are out of scope for this bundle.

## Layout

```text
integrations/cli-skill-bundle/
  bundle.json
  check-bundle.py
  references/
    cli-contract.md
  skills/
    highergraphen/
      SKILL.md
    casegraphen/
      SKILL.md
    architecture-review/
      SKILL.md
```

The bundled `highergraphen` skill is copied from
`skills/highergraphen/SKILL.md`. Run the bundle smoke check after changing the
source skill so the packaged copy stays in sync.

The bundled `casegraphen` skill is copied from `skills/casegraphen/SKILL.md`.
It covers installed `cg` workspace operation, the repo-owned
`casegraphen workflow ...` report surface, and the repo-owned
`casegraphen cg workflow ...` bridge without introducing MCP or provider SDK
integrations.

The bundled `architecture-review` skill is a thin workflow guide for the
current Architecture Product smoke report. It points agents back to the
`highergraphen` CLI, schema, fixture, and validator instead of reimplementing
workflow logic.

## Contract References

The stable CLI command is:

```sh
highergraphen architecture smoke direct-db-access --format json
```

The stable CaseGraphen workflow reasoning command is:

```sh
casegraphen workflow reason --input workflow.graph.json --format json
```

Focused CaseGraphen workflow report commands are:

```sh
casegraphen workflow validate --input workflow.graph.json --format json
casegraphen workflow readiness --input workflow.graph.json --format json
casegraphen workflow obstructions --input workflow.graph.json --format json
casegraphen workflow completions --input workflow.graph.json --format json
casegraphen workflow evidence --input workflow.graph.json --format json
casegraphen workflow project --input workflow.graph.json --projection projection.json --format json
casegraphen workflow correspond --left left.workflow.json --right right.workflow.json --format json
casegraphen workflow evolution --input workflow.graph.json --format json
```

The repo-owned bridge for workflow storage, history, completion review, and
patch review is:

```sh
casegraphen cg workflow import --store casegraphen-workflow-store --input workflow.graph.json --revision-id revision:initial --format json
casegraphen cg workflow validate --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow completion accept|reject|reopen --store casegraphen-workflow-store --workflow-graph-id <id> --candidate-id <candidate-id> --reviewer-id <reviewer-id> --reason "<reason>" --revision-id <revision-id> --format json
casegraphen cg workflow patch check --store casegraphen-workflow-store --workflow-graph-id <id> --transition-id <transition-id> --format json
casegraphen cg workflow patch apply|reject --store casegraphen-workflow-store --workflow-graph-id <id> --transition-id <transition-id> --reviewer-id <reviewer-id> --reason "<reason>" --revision-id <revision-id> --format json
```

Installed `cg` remains the native `.casegraphen` workspace surface for case
creation, evidence, frontier, blockers, topology, and `cg validate --case`.

The repository-owned validation path is:

```sh
python3 scripts/validate-cli-report-contract.py
```

The machine-readable report contract lives at
`schemas/reports/architecture-direct-db-access-smoke.report.schema.json`, with
the example fixture at
`schemas/reports/architecture-direct-db-access-smoke.report.example.json`.
CaseGraphen workflow contracts live at
`schemas/casegraphen/workflow.graph.schema.json` and
`schemas/casegraphen/workflow.report.schema.json`. The completed operator
surface is summarized in
`docs/specs/intermediate-tools/casegraphen-feature-completion-contract.md`.

## Checks

Run the bundle smoke check from the repository root:

```sh
python3 integrations/cli-skill-bundle/check-bundle.py
```

Run the CLI report contract validator:

```sh
python3 scripts/validate-cli-report-contract.py
```

If code or scripts changed, also run:

```sh
sh scripts/static-analysis.sh
```
