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
    casegraphen-ddd-diagnostics/
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
`casegraphen cg workflow ...` bridge. It also covers native CaseGraphen
`casegraphen case ...` and `casegraphen morphism ...` commands for CaseSpace
plus MorphismLog operation without introducing MCP or provider SDK
integrations. Installed `cg` is the meta `.casegraphen` workflow driver, not
the native CaseGraphen product model.

The bundled `casegraphen-ddd-diagnostics` skill is copied from
`skills/casegraphen-ddd-diagnostics/SKILL.md`. It guides agents through DDD
domain model review using native CaseGraphen `CaseSpace` plus `MorphismLog`
reports, including boundary semantic loss, missing evidence, completion
candidates, projection loss, and close-check interpretation.

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
casegraphen workflow history topology --input workflow.graph.json --format json
casegraphen workflow project --input workflow.graph.json --projection projection.json --format json
casegraphen workflow correspond --left left.workflow.json --right right.workflow.json --format json
casegraphen workflow evolution --input workflow.graph.json --format json
```

The repo-owned bridge for workflow storage, history, completion review, and
patch review is:

```sh
casegraphen cg workflow import --store casegraphen-workflow-store --input workflow.graph.json --revision-id revision:initial --format json
casegraphen cg workflow validate --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow completion accept --store casegraphen-workflow-store --workflow-graph-id <id> --candidate-id <candidate-id> --reviewer-id <reviewer-id> --reason "<reason>" --revision-id <revision-id> --format json
casegraphen cg workflow completion reject --store casegraphen-workflow-store --workflow-graph-id <id> --candidate-id <candidate-id> --reviewer-id <reviewer-id> --reason "<reason>" --revision-id <revision-id> --format json
casegraphen cg workflow completion reopen --store casegraphen-workflow-store --workflow-graph-id <id> --candidate-id <candidate-id> --reviewer-id <reviewer-id> --reason "<reason>" --revision-id <revision-id> --format json
casegraphen cg workflow patch check --store casegraphen-workflow-store --workflow-graph-id <id> --transition-id <transition-id> --format json
casegraphen cg workflow patch apply --store casegraphen-workflow-store --workflow-graph-id <id> --transition-id <transition-id> --reviewer-id <reviewer-id> --reason "<reason>" --revision-id <revision-id> --format json
casegraphen cg workflow patch reject --store casegraphen-workflow-store --workflow-graph-id <id> --transition-id <transition-id> --reviewer-id <reviewer-id> --reason "<reason>" --revision-id <revision-id> --format json
```

Installed `cg` remains the native `.casegraphen` workspace surface for case
creation, evidence, frontier, blockers, topology, and `cg validate --case`.

The native CaseGraphen case store, reasoning, close-check, and morphism
commands are:

```sh
casegraphen case import --store casegraphen-native-store --input native.case.space.json --revision-id revision:initial --format json
casegraphen case reason --store casegraphen-native-store --case-space-id <id> --format json
casegraphen case frontier --store casegraphen-native-store --case-space-id <id> --format json
casegraphen case history topology --store casegraphen-native-store --case-space-id <id> --format json
casegraphen case close-check --store casegraphen-native-store --case-space-id <id> --base-revision-id <revision-id> --validation-evidence-id <evidence-id> --format json
casegraphen morphism propose --store casegraphen-native-store --case-space-id <id> --input case_morphism.json --format json
casegraphen morphism apply --store casegraphen-native-store --case-space-id <id> --morphism-id <morphism-id> --base-revision-id <revision-id> --reviewer-id <reviewer-id> --reason "<reason>" --format json
```

The native reference examples live at `examples/casegraphen/native/README.md`.
They document the current residual limitations: metadata-only morphism
application and close-check without a native close command.

The DDD diagnostic reference example lives at
`examples/casegraphen/ddd/domain-model-design/README.md`. It uses
`sales-billing-customer.case.space.json` to show how `casegraphen case reason`,
`casegraphen case obstructions`, `casegraphen case completions`,
`casegraphen case evidence`, `casegraphen case project`, and
`casegraphen case close-check` can flag a blocked domain model decision without
turning domain findings into CLI failures.

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
The native CaseGraphen CaseSpace contract is
`docs/specs/intermediate-tools/casegraphen-native-case-management.md`, with
schemas and fixtures under `schemas/casegraphen/native.case.*`.

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
