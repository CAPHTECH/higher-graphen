# Report Schemas

This directory contains stable JSON report contracts and golden examples for
HigherGraphen runtime and CLI reports.

`architecture-direct-db-access-smoke.report.schema.json` defines the v1 contract
for `highergraphen.architecture.direct_db_access_smoke.report.v1`. The matching
example fixture is generated from:

```sh
highergraphen architecture smoke direct-db-access --format json
```

Schemas are intended to lock the public report envelope, deterministic scenario
IDs, machine-checkable result shape, obstruction and completion candidate fields,
projection audience and purpose, and unreviewed review status.
