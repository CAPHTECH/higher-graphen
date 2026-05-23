---
name: release-runner
description: Prepare, validate, and ship higher-graphen releases from release-readiness triage through final publication. Use when asked to prepare a release, run release quality gates, fix release blockers, create release tags, draft release notes, publish a GitHub release, or verify post-release health for this repository.
---

# Release Runner

## Overview

Use this skill to take higher-graphen from release intent to a verified release. Treat release work as a gate-driven workflow: identify the target, audit readiness, fix blockers, rerun the full gate, then publish only with explicit approval.

For release target inventory, read `references/release-surface.md` while establishing release intent. For exact command lists and current release invariants, read `references/release-gates.md` before final verification or publishing.

## Start

- Check the worktree first with `git status --short --branch`. Do not rewrite, revert, or discard unrelated user changes.
- Use subagents only for independent release audits or parallel checks that do not block the immediate next local step. Keep ownership explicit and avoid duplicate work.
- If the release target, version, or publishing destination is missing and cannot be inferred safely, ask one concise question before tagging or publishing.

## Workflow

1. Establish release intent: version, target commit or branch, release type, expected artifacts, and whether publishing means a Git tag, GitHub Release, CLI binaries, schema or skill bundles, crates, or another channel. Check `references/release-surface.md` before deciding scope.
2. Audit readiness: run or inspect the mandatory release gate in `scripts/static-analysis.sh`, then add dependency, security, schema, docs, and skill-bundle checks from `references/release-gates.md` when relevant.
3. Fix blockers with the repository's existing patterns. Add or update tests and schemas when behavior, CLI contracts, or generated reports change.
4. Preserve release invariants: native morphism proposal must not silently promote review status; report schema IDs need a direct schema or explicit alias; shell code blocks marked `sh` must be copy-pastable.
5. Final verification: rerun `scripts/static-analysis.sh`, run touched-area checks, then run `git diff --check` and `git status --short --branch`.
6. Package the release: update version and release notes for each included release surface. Create annotated tags, GitHub Releases, binaries, bundles, or package publications only after the user explicitly approves publishing.
7. Post-release: verify the tag or release exists and capture artifact URLs, release notes, and any follow-up risks in the final report.

## Publishing Guardrails

- Never call the release complete if `scripts/static-analysis.sh` fails.
- Do not push tags, create GitHub Releases, or publish packages without explicit user approval for that publishing action.
- Do not ignore a dirty worktree. Separate intentional release changes from unrelated existing changes in the final report.
- Do not replace reviewed human intent with AI inference; proposal/check commands must preserve review state unless an explicit apply/reject workflow changes it.

## Final Response

Report the exact verification commands run, blockers fixed, release tag or URL if published, and residual risks. If publishing was not performed, state that plainly.
