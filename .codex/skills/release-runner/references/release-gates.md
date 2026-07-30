# higher-graphen Release Gates

Read this reference when preparing final verification or publishing a release.

## Mandatory Gate

Run the repository release gate from the project root:

```sh
scripts/static-analysis.sh
```

The gate is expected to cover formatting, locked dependency metadata, workspace check/test/clippy/doc, static limits, CLI report contract validation, JSON contract validation, and CLI skill bundle validation.

## Supplementary Checks

Use these when the release changes dependencies, security posture, schemas, generated reports, docs, or publishing artifacts:

```sh
cargo tree --workspace --duplicates
cargo audit -q
git diff --check
git status --short --branch
```

For secret scanning, prefer `gitleaks` or `trufflehog` if installed. If neither is available, run a heuristic scan and report that limitation:

```sh
rg -n --hidden --glob '!.git' --glob '!target' 'secret|token|password|api[_-]?key|private[_-]?key'
```

If `cargo audit -q` fails only because the advisory DB lock or update needs access outside the sandbox, rerun it with the required approval instead of treating the first failure as a project defect.

## Release Invariants

- CaseGraphen release invariants (morphism proposal review status, its report schema aliases) moved with the tool to https://github.com/CAPHTECH/casegraphen per ADR 0003.
- Report schema IDs must resolve to a concrete schema file under `schemas/`.
- Documentation code blocks marked `sh` must contain copy-pastable shell, not placeholders or pseudo-shell.
- `integrations/cli-skill-bundle/check-bundle.py` is part of the release gate for bundled skill compatibility.

## Publishing

Publishing means a durable external action: pushing tags, creating GitHub Releases, or publishing packages. Do not do it without explicit user approval for that action.

Typical Git tag flow after approval:

```sh
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

Typical GitHub Release flow after approval:

```sh
gh release create vX.Y.Z --title "vX.Y.Z" --notes-file RELEASE_NOTES.md
gh release view vX.Y.Z
```
