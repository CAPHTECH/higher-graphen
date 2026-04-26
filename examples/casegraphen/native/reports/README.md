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
| `case new` | `casegraphen case new` |
| `case import` | `casegraphen case import` |
| `case list` | `casegraphen case list` |
| `case inspect` | `casegraphen case inspect` |
| `case history` | `casegraphen case history` |
| `case replay` | `casegraphen case replay` |
| `case validate` | `casegraphen case validate` |
| `case reason` | `casegraphen case reason` |
| `case frontier` | `casegraphen case frontier` |
| `case obstructions` | `casegraphen case obstructions` |
| `case completions` | `casegraphen case completions` |
| `case evidence` | `casegraphen case evidence` |
| `case project` | `casegraphen case project` |
| `case close-check` | `casegraphen case close-check` |
| `morphism propose` | `casegraphen morphism propose` |
| `morphism check` | `casegraphen morphism check` |
| `morphism apply` | `casegraphen morphism apply` |
| `morphism reject` | `casegraphen morphism reject` |

Expected domain findings for
`schemas/casegraphen/native.case.space.example.json` after import:

- `case reason` reports `result.evaluation.status == "review_required"`.
- `case frontier` includes `goal:native-case-contract`.
- `case completions` reports an empty `completion_candidates` array.
- `case close-check` returns a structured `close_check` result; closability is
  a domain finding, not a CLI failure.
- `case project` reports replayed projections and derived projection loss.

Domain statuses such as review-required, blocked, incomplete, projection loss,
missing evidence, and non-closeable are successful JSON report data. Only
malformed input, stale base revisions, invalid morphism IDs, unsafe paths, and
invalid native stores should make the CLI command fail.
