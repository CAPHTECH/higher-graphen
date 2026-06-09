# Dogfooding Log

HigherGraphen's flagship use case is managing AI-performed development work as
CaseGraphen cases. This log records whether driving our own work through the
`cg` workspace CLI (and the deterministic detectors) actually catches anything
we would otherwise miss. If it never does, that is the highest-quality product
defect data we can collect.

Cadence: one entry per development push of real work. Keep it honest — record
"caught nothing" when nothing was caught.

## 2026-06-09 — Loop resumed after the 2026-05-01 stall

**Case:** `hg_external_value_sharpening` (Sharpen HigherGraphen external value:
storefront + wedge + distribution). 13 events appended on 2026-06-09, the first
`.casegraphen/` activity since 2026-05-01.

**Work driven:** `task_storefront_refresh` (README storefront refresh) started →
evidence attached → done. Evidence: commit `296fd6e` pushed to `origin/main`;
both own-repo quick-start pipelines verified end-to-end on this repo.

**Frontier after:** 3 ready next-steps, no false blockers —
`task_wedge_trust_report`, `task_mcp_server`, `task_pull_driven_core`
(`is_ready: true`, `dependency_satisfied_ratio: 1.0`).

**Did the tools catch anything?**

- **Case/frontier mechanics:** worked, low surprise. Expected for a freshly
  created case — the value is durable state + an evolving frontier, which only
  pays off over multiple weeks. Not yet evidence for or against the thesis.
- **`test-gap` on this repo (real signal):** `test-gap detect` over the last
  diff flagged **549 missing-test candidates / 535 counterexamples**. High
  recall, low precision *without* `--binding-rules` — it flags essentially every
  changed code file lacking a co-located test. Product finding: test-gap needs a
  binding-rules config to be signal rather than noise; the README quick start
  should probably mention this. Filed mentally as a precision follow-up.
- **Verification discipline caught a false hypothesis (the loop working):**
  going in, I assumed the README's `casegraphen case import` / `case reason` /
  `case close-check` examples had drifted from the real CLI surface
  (`lift native` / `space reason` / `invariant close-check`). Running the actual
  command proved they still work as aliases — exit 0, full report. No bug. The
  "verify against actual behavior before asserting" loop stopped a fake-bug case
  from being created.

**Next measurement (next entry):** drive `task_wedge_trust_report`. Check (a)
whether `invariant close-check` blocks closure when accepted evidence is
missing, and (b) whether `test-gap` with a binding-rules file tightens precision
enough to be trustworthy.
