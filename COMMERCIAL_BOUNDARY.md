# Commercial Boundary

HigherGraphen is intended to be published with an open-source core and a clear
commercial boundary.

The public repository should help the ecosystem understand, inspect, and build
against the HigherGraphen model. Commercial value should come from production
interpretation packages, hosted execution, enterprise integration, support, and
customer-specific assets that are maintained outside this repository unless
they are intentionally open-sourced later.

## Public Repository

The public `CAPHTECH/higher-graphen` repository may include:

- Higher-structure core crates.
- Baseline intermediate tools, including `casegraphen`.
- Stable schemas, report envelopes, and public examples.
- Documentation for the AI operator paradigm and higher-structure product
  model.
- Repository-owned skills and provider-neutral CLI skill bundles.
- Public `.casegraphen` development traces that document roadmap, decisions,
  evidence, and verification for this repository.
- Synthetic fixtures and reference scenarios that contain no customer data.

Public materials in this repository are licensed under the Apache License 2.0
unless a file or subdirectory explicitly states otherwise.

## Commercial Or Private Boundary

The following should not be placed in this public repository unless a later
explicit decision changes the boundary:

- Customer-specific data, documents, cases, traces, reports, or evaluations.
- Private engagement records, support notes, or deployment details.
- Proprietary production interpretation packages.
- Hosted service infrastructure and private operations runbooks.
- Private evaluation datasets, benchmark data, and tuning corpora.
- Commercial pricing, sales pipeline, negotiation notes, or partner terms.
- Secrets, credentials, tokens, API keys, private keys, and local runtime state.
- Any CaseGraphen case that depends on private context rather than a public
  repository roadmap.

Private work should live in separate private repositories, private CaseGraphen
workspaces, hosted infrastructure, or customer-specific systems.

## Interpretation Packages

HigherGraphen's product principle is:

```text
Product = Interpretation Package over Higher Structure
```

This means the public repository can define the shared structural foundation
without giving away every production product. A domain package may be published
when it is meant to become a public reference implementation. A domain package
should remain private or commercial when it contains proprietary rules,
customer-specific workflows, private evaluation data, or operational advantage.

## CaseGraphen Publication Rule

`.casegraphen/` in this repository is public by default only for this
repository's own public development. It is not a place for private customer
work.

Before committing a CaseGraphen case, treat it as a public artifact and verify
that it contains no private context, secrets, customer material, or commercial
strategy that belongs outside the open repository.

Runtime artifacts such as locks, caches, local state, private cases, and SQLite
files are excluded by `.gitignore`.
