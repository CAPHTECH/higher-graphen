# HigherGraphen CLI Skill Bundle

This provider-neutral bundle packages the current HigherGraphen CLI skill
surface for agents. It is intentionally smaller than a provider plugin: it
contains contract references, metadata, and a local smoke check. Distributed
skill source files live under the repository root `skills/` directory.

MCP servers, provider marketplace publication, provider SDK integrations, and
provider-specific manifests are out of scope for this bundle.

## Layout

```text
integrations/cli-skill-bundle/
  bundle.json
  check-bundle.py
  references/
    cli-contract.md
```

The distributed `highergraphen` skill is `skills/highergraphen/SKILL.md`. Run
the bundle smoke check after changing the source skill so the metadata and key
operator terms stay valid.

The distributed `highergraphen-ddd` skill is
`skills/highergraphen-ddd/SKILL.md`. It guides agents through the bounded
`highergraphen ddd` product workflow, including boundary semantic loss,
missing evidence, completion candidates, projection loss, review gaps, and
closeability interpretation.

The distributed `architecture-review` skill is
`skills/architecture-review/SKILL.md`. It is a thin workflow guide for the
current Architecture Product smoke report and points agents back to the
`highergraphen` CLI, schema, fixture, and validator instead of reimplementing
workflow logic.

## Contract References

The stable CLI command is:

```sh
highergraphen architecture smoke direct-db-access --format json
```

The bounded test-gap detector command is:

```sh
highergraphen test-gap input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output test-gap.input.json

highergraphen test-gap detect \
  --input test-gap.input.json \
  --format json
```

`highergraphen test-gap input from-git` creates a deterministic bounded
`highergraphen.test_gap.input.v1` snapshot from a local git range. It does not
execute tests, crawl the full repository, or prove typed semantic equivalence.
Its `detector_context.test_kinds` field is the verification policy; changed
integration tests may be accepted as verification without rewriting their
observed test type.
For HigherGraphen-owned test-gap surfaces, the adapter also emits higher-order
command, runner, export, registry, schema, fixture, projection, base/head
Rust AST and JSON Schema semantic cells, semantic delta morphisms, incidence,
and `requirement:morphism:*` records so tests verify structure instead of
isolated files.

The bounded semantic proof certificate command is:

```sh
highergraphen semantic-proof verify \
  --input schemas/inputs/semantic-proof.input.example.json \
  --format json
```

`semantic-proof verify` validates supplied proof certificates and
counterexamples against theorem, law, morphism, backend, hash, and review
policy. External proof backend execution happens before this command.

The repository-owned validation path is:

```sh
python3 scripts/validate-cli-report-contract.py
```

The machine-readable report contract lives at
`schemas/reports/architecture-direct-db-access-smoke.report.schema.json`, with
the example fixture at
`schemas/reports/architecture-direct-db-access-smoke.report.example.json`.
The test-gap detector consumes
`schemas/inputs/test-gap.input.schema.json` and emits
`schemas/reports/test-gap.report.schema.json`; the fixture pair is
`schemas/inputs/test-gap.input.example.json` and
`schemas/reports/test-gap.report.example.json`.
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
