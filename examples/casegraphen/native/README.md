# Native CaseGraphen Reference Case

This directory documents the reference native CaseGraphen flow for the
`casegraphen case ...` and `casegraphen morphism ...` command namespaces.
Native CaseGraphen is a `CaseSpace` replayed from a `MorphismLog`; it is not a
clone of installed `cg` task/event semantics.

The canonical native fixture currently lives at:

- `schemas/casegraphen/native.case.space.example.json`
- `schemas/casegraphen/native.case.report.example.json`
- `schemas/casegraphen/native.case.space.schema.json`
- `schemas/casegraphen/native.case.report.schema.json`

Use a temporary store when exercising the fixture:

With a local binary, the command spellings are `casegraphen case import`,
`casegraphen case reason`, `casegraphen case frontier`,
`casegraphen case history topology`, `casegraphen case close-check`,
`casegraphen morphism propose`, and `casegraphen morphism apply`. The examples
below use Cargo so they work from a fresh repository checkout.

```sh
cargo run -q -p casegraphen -- \
  case import \
  --store /tmp/casegraphen-native-store \
  --input schemas/casegraphen/native.case.space.example.json \
  --revision-id revision:native-reference-imported \
  --format json
```

Inspect and replay the stored native case space:

```sh
cargo run -q -p casegraphen -- \
  case inspect \
  --store /tmp/casegraphen-native-store \
  --case-space-id case_space:native-case-management-contract \
  --format json

cargo run -q -p casegraphen -- \
  case replay \
  --store /tmp/casegraphen-native-store \
  --case-space-id case_space:native-case-management-contract \
  --format json
```

Derive read-only native reasoning views from replayed state:

```sh
cargo run -q -p casegraphen -- case reason --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --format json
cargo run -q -p casegraphen -- case frontier --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --format json
cargo run -q -p casegraphen -- case history topology --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --format json
cargo run -q -p casegraphen -- case obstructions --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --format json
cargo run -q -p casegraphen -- case completions --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --format json
cargo run -q -p casegraphen -- case evidence --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --format json
cargo run -q -p casegraphen -- case project --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --format json
```

Run close checking against an explicit replay revision and validation evidence:

```sh
cargo run -q -p casegraphen -- \
  case close-check \
  --store /tmp/casegraphen-native-store \
  --case-space-id case_space:native-case-management-contract \
  --base-revision-id revision:native-reference-imported \
  --validation-evidence-id evidence:native-schema-json-valid \
  --format json
```

Reviewable native mutations use morphism proposals. The first CLI
implementation intentionally accepts only metadata-only morphisms, so examples
should document proposal/check/apply/reject mechanics without implying that
arbitrary case cell payloads are already materialized:

```sh
cargo run -q -p casegraphen -- \
  morphism propose \
  --store /tmp/casegraphen-native-store \
  --case-space-id case_space:native-case-management-contract \
  --input /tmp/metadata-only.case_morphism.json \
  --format json

cargo run -q -p casegraphen -- morphism check --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --morphism-id morphism:native-example --format json
cargo run -q -p casegraphen -- morphism apply --store /tmp/casegraphen-native-store --case-space-id case_space:native-case-management-contract --morphism-id morphism:native-example --base-revision-id revision:native-reference-imported --reviewer-id reviewer:native-operator --reason "Accept metadata-only example morphism" --format json
```

## Expected Reports

The expected report shapes are documented in
[`reports/README.md`](reports/README.md). The schema fixtures remain the stable
contract; generated reports may include store paths, timestamps, checksums, and
revision IDs from the local run.

## Verification Status

`tools/casegraphen/tests/command.rs` covers the deterministic native CLI flow:
create, import, list, inspect, history, replay, reason, frontier,
obstructions, completions, evidence, project, close-check, and morphism
propose/check/apply/reject. It also validates
`schemas/casegraphen/native.case.space.example.json` and
`schemas/casegraphen/native.case.report.example.json` against their JSON
Schema files.

The 2026-04-26 release gate for `task_native_case_e2e_verification` passed
formatting, package tests, workspace tests, static analysis, bundle smoke,
whitespace diff check, CaseGraphen case validation, storage validation, and
higher-order topology smoke.

## Residual Limitations

- Native `casegraphen case ...` commands currently operate on an explicit
  repo-owned native store under the supplied `--store` path.
- Native morphism proposal/apply is conservative: it validates and appends
  metadata-only morphisms and rejects unmaterialized payload changes.
- There is no native `case close` command yet; operators run
  `case close-check` and record accepted close evidence in the owning workflow.
- Installed `cg` is only the meta workflow driver for `.casegraphen/` cases in
  this repository. Do not treat installed `cg` task states as the native
  CaseGraphen product model.
