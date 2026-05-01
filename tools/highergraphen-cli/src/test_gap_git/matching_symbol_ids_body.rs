{
    let test_key = comparable_path_key(test_path);
    let mut ids = symbols
        .iter()
        .filter(|symbol| {
            symbol
                .path
                .as_deref()
                .map(comparable_path_key)
                .is_some_and(|symbol_key| {
                    test_key.contains(&symbol_key) || symbol_key.contains(&test_key)
                })
        })
        .map(|symbol| symbol.id.clone())
        .collect::<Vec<_>>();

    if test_path == "tools/highergraphen-cli/tests/command.rs" {
        push_matching_symbols(
            &mut ids,
            symbols,
            &[
                "command:highergraphen:test-gap:detect",
                "command:highergraphen:test-gap:input-from-git",
                "command:highergraphen:test-gap:input-from-path",
                "adapter:test-gap:git-input",
                "adapter:test-gap:path-input",
                "law:test-gap:command-routes-to-runner",
                "law:test-gap:json-format-required",
                "law:test-gap:output-file-suppresses-stdout",
                "law:test-gap:input-from-git-is-deterministic",
                "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
                "law:test-gap:input-from-path-is-deterministic",
                "law:test-gap:input-from-path-declares-snapshot-boundary",
                "symbol:tools-highergraphen-cli-src-main-rs:changed-behavior",
                "command:highergraphen:semantic-proof:backend-run",
                "command:highergraphen:semantic-proof:input-from-artifact",
                "command:highergraphen:semantic-proof:input-from-report",
                "command:highergraphen:semantic-proof:verify",
                "runner:semantic-proof:backend-run",
                "adapter:semantic-proof:artifact-input",
                "adapter:semantic-proof:reinput-from-report",
                "test:semantic-proof:artifact-roundtrip",
                "test:semantic-proof:backend-and-reinput",
                "theorem:semantic-proof:artifact-adapter-correctness",
                "theorem:semantic-proof:backend-run-trust-boundary",
                "theorem:semantic-proof:obligation-reinput-correctness",
                "law:semantic-proof:backend-run-records-trust-boundary",
                "law:semantic-proof:artifact-status-totality",
                "law:semantic-proof:certificate-policy-preservation",
                "law:semantic-proof:counterexample-refutation-preservation",
                "law:semantic-proof:counterexample-review-policy",
                "law:semantic-proof:insufficient-proof-reinputs-open-obligations",
                "law:semantic-proof:backend-boundary-is-explicit",
                "law:semantic-proof:roundtrip-tests-cover-proof-and-counterexample",
                "symbol:tools-highergraphen-cli-src-semantic-proof-backend-rs:changed-behavior",
                "symbol:tools-highergraphen-cli-src-semantic-proof-artifact-rs:changed-behavior",
                "symbol:tools-highergraphen-cli-src-semantic-proof-reinput-rs:changed-behavior",
            ],
        );
    }
    if test_path == "crates/higher-graphen-runtime/tests/test_gap.rs" {
        push_matching_symbols(
            &mut ids,
            symbols,
            &[
                "runner:test-gap:detect",
                "export:test-gap:runtime-api",
                "registry:test-gap:workflow-module",
                "contract:test-gap:runtime-report-shapes",
                "projection:test-gap:report-envelope",
                "schema:test-gap:input",
                "schema:test-gap:report",
                "fixture:test-gap:input-example",
                "fixture:test-gap:report-example",
                "law:test-gap:test-gap-is-bounded",
                "law:test-gap:verification-policy-controls-test-kind",
                "law:test-gap:requirements-map-to-implementation-and-test",
                "law:test-gap:candidates-remain-unreviewed",
                "law:test-gap:projection-declares-information-loss",
                "law:test-gap:schema-id-preserved",
                "law:test-gap:enum-casing-round-trips",
                "law:test-gap:runtime-shapes-preserve-schema",
                "symbol:crates-higher-graphen-runtime-src-test-gap-reports-rs:changed-behavior",
                "symbol:crates-higher-graphen-runtime-src-workflows-test-gap-rs:changed-behavior",
            ],
        );
    }

    ids
}
