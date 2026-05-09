# Native Report Expectations

Native CLI reports use the repository-owned `casegraphen` binary and the
`highergraphen.case.native_cli.report.v1` envelope emitted by
`tools/casegraphen/src/native_cli/ops.rs`. The generated CLI envelope is
validated by `schemas/casegraphen/native-cli.report.schema.json`; the separate
`schemas/casegraphen/native.case.report.schema.json` fixture remains the
package-level native CaseSpace report contract.

Expected command metadata values:

| Command | `metadata.command` |
| --- | --- |
| `space new` | `casegraphen space new` |
| `lift native` | `casegraphen lift native` |
| `space list` | `casegraphen space list` |
| `space inspect` | `casegraphen space inspect` |
| `space history` | `casegraphen space history` |
| `space replay` | `casegraphen space replay` |
| `space validate` | `casegraphen space validate` |
| `space reason` | `casegraphen space reason` |
| `space frontier` | `casegraphen space frontier` |
| `obstruction list` | `casegraphen obstruction list` |
| `completion candidates` | `casegraphen completion candidates` |
| `invariant check` | `casegraphen invariant check` |
| `projection apply` | `casegraphen projection apply` |
| `equivalence check` | `casegraphen equivalence check` |
| `invariant close-check` | `casegraphen invariant close-check` |
| `morphism propose` | `casegraphen morphism propose` |
| `morphism check` | `casegraphen morphism check` |
| `morphism apply` | `casegraphen morphism apply` |
| `morphism reject` | `casegraphen morphism reject` |

Expected domain findings for
`schemas/casegraphen/native.case.space.example.json` after import:

- `space reason` reports `result.evaluation.status == "review_required"`.
- `space frontier` includes `goal:native-case-contract`.
- `completion candidates` reports an empty `completion_candidates` array.
- `invariant close-check` returns a structured `close_check` result;
  closability is a domain finding, not a CLI failure.
- `projection apply` reports matched projections and derived projection loss.

Domain statuses such as review-required, blocked, incomplete, projection loss,
missing evidence, and non-closeable are successful JSON report data. Only
malformed input, stale base revisions, invalid morphism IDs, unsafe paths, and
invalid native stores should make the CLI command fail.
