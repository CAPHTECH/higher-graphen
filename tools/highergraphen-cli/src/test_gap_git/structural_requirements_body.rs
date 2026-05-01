{
    let mut requirements = Vec::new();
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:command-detect-to-runner",
        "CLI command highergraphen test-gap detect preserves its morphism to run_test_gap_detect",
        &[
            "command:highergraphen:test-gap:detect",
            "runner:test-gap:detect",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:input-from-git-to-input-schema",
        "CLI command highergraphen test-gap input from-git routes through the git adapter and emits the bounded test-gap input schema",
        &[
            "command:highergraphen:test-gap:input-from-git",
            "adapter:test-gap:git-input",
            "schema:test-gap:input",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:input-from-path-to-input-schema",
        "CLI command highergraphen test-gap input from-path routes through the current-tree path adapter and emits the bounded test-gap input schema",
        &[
            "command:highergraphen:test-gap:input-from-path",
            "adapter:test-gap:path-input",
            "schema:test-gap:input",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:runtime-export-to-runner",
        "Runtime public exports preserve access to the test-gap detector runner and report types",
        &[
            "export:test-gap:runtime-api",
            "runner:test-gap:detect",
            "contract:test-gap:runtime-report-shapes",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:workflow-registry-to-runner",
        "Workflow module registry preserves the test-gap runner connection",
        &[
            "registry:test-gap:workflow-module",
            "runner:test-gap:detect",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:runtime-shapes-to-schemas",
        "Runtime TestGap shapes preserve the input and report schema contracts",
        &[
            "contract:test-gap:runtime-report-shapes",
            "schema:test-gap:input",
            "schema:test-gap:report",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:fixtures-to-schemas",
        "Checked-in test-gap fixtures preserve their input and report schema contracts",
        &[
            "fixture:test-gap:input-example",
            "fixture:test-gap:report-example",
            "schema:test-gap:input",
            "schema:test-gap:report",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:report-envelope-to-runtime-shapes",
        "Report envelope projection preserves the TestGap runtime report shape boundary",
        &[
            "projection:test-gap:report-envelope",
            "contract:test-gap:runtime-report-shapes",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:backend-run-to-artifact",
        "semantic-proof backend run records local proof command output as bounded artifact material before HG verification",
        &[
            "command:highergraphen:semantic-proof:backend-run",
            "runner:semantic-proof:backend-run",
            "theorem:semantic-proof:backend-run-trust-boundary",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:artifact-to-input-document",
        "semantic-proof input from-artifact preserves backend artifacts as HG theorem, law, morphism, and certificate or counterexample input",
        &[
            "command:highergraphen:semantic-proof:input-from-artifact",
            "adapter:semantic-proof:artifact-input",
            "theorem:semantic-proof:artifact-adapter-correctness",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:certificate-to-proof-object",
        "proved semantic-proof artifacts roundtrip through verify as accepted proof objects",
        &[
            "adapter:semantic-proof:artifact-input",
            "command:highergraphen:semantic-proof:verify",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:counterexample-to-refutation",
        "counterexample semantic-proof artifacts roundtrip through verify as refutations",
        &[
            "adapter:semantic-proof:artifact-input",
            "command:highergraphen:semantic-proof:verify",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:insufficient-report-to-reinput",
        "insufficient semantic-proof reports requeue open law and morphism obligations as a new bounded input",
        &[
            "command:highergraphen:semantic-proof:input-from-report",
            "adapter:semantic-proof:reinput-from-report",
            "theorem:semantic-proof:obligation-reinput-correctness",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:roundtrip-tests-to-adapter-correctness",
        "semantic-proof artifact roundtrip tests verify the adapter correctness theorem at the HG structure boundary",
        &[
            "test:semantic-proof:artifact-roundtrip",
            "test:semantic-proof:backend-and-reinput",
            "theorem:semantic-proof:artifact-adapter-correctness",
            "theorem:semantic-proof:backend-run-trust-boundary",
            "theorem:semantic-proof:obligation-reinput-correctness",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:command-routes-to-runner",
        "requirement:law:test-gap:command-routes-to-runner",
        "CLI command parsing dispatches test-gap commands to the intended runtime runner or adapter",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:json-format-required",
        "requirement:law:test-gap:json-format-required",
        "test-gap CLI commands reject missing or unsupported non-JSON formats",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:output-file-suppresses-stdout",
        "requirement:law:test-gap:output-file-suppresses-stdout",
        "test-gap CLI --output writes to the requested file and suppresses JSON stdout",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:input-from-git-is-deterministic",
        "requirement:law:test-gap:input-from-git-is-deterministic",
        "test-gap input from-git emits deterministic bounded structure from the requested git range",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
        "requirement:law:test-gap:input-from-git-does-not-prove-semantic-coverage",
        "test-gap input from-git keeps semantic coverage limits visible instead of proving full coverage",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:input-from-path-is-deterministic",
        "requirement:law:test-gap:input-from-path-is-deterministic",
        "test-gap input from-path emits deterministic bounded structure from selected current-tree files",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:input-from-path-declares-snapshot-boundary",
        "requirement:law:test-gap:input-from-path-declares-snapshot-boundary",
        "test-gap input from-path keeps current-tree scope and semantic coverage limits visible",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:test-gap-is-bounded",
        "requirement:law:test-gap:test-gap-is-bounded",
        "test-gap detector status is bounded to the supplied snapshot and detector policy",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:verification-policy-controls-test-kind",
        "requirement:law:test-gap:verification-policy-controls-test-kind",
        "detector_context.test_kinds controls which observed test kinds close obligations",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:requirements-map-to-implementation-and-test",
        "requirement:law:test-gap:requirements-map-to-implementation-and-test",
        "in-scope test-gap requirements require implementation targets and accepted verification cells",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:candidates-remain-unreviewed",
        "requirement:law:test-gap:candidates-remain-unreviewed",
        "generated completion candidates stay unreviewed until explicit review",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:projection-declares-information-loss",
        "requirement:law:test-gap:projection-declares-information-loss",
        "test-gap projections declare information loss in human, AI, and audit views",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:schema-id-preserved",
        "requirement:law:test-gap:schema-id-preserved",
        "test-gap input and report schema IDs are preserved through runtime and CLI boundaries",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:enum-casing-round-trips",
        "requirement:law:test-gap:enum-casing-round-trips",
        "test-gap enum values serialize using schema casing and round-trip",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:runtime-shapes-preserve-schema",
        "requirement:law:test-gap:runtime-shapes-preserve-schema",
        "runtime TestGap shapes preserve required schema fields and report boundaries",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:fixtures-validate-against-schema",
        "requirement:law:test-gap:fixtures-validate-against-schema",
        "checked-in test-gap fixtures validate against their declared schemas",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:backend-run-records-trust-boundary",
        "requirement:law:semantic-proof:backend-run-records-trust-boundary",
        "semantic-proof backend run records hashes, exit status, and review state without silently accepting failing outputs",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:artifact-status-totality",
        "requirement:law:semantic-proof:artifact-status-totality",
        "semantic-proof artifact adapter handles the proved and counterexample status partition",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:certificate-policy-preservation",
        "requirement:law:semantic-proof:certificate-policy-preservation",
        "proved semantic-proof artifacts preserve backend policy, hashes, witnesses, and accepted review state",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:counterexample-refutation-preservation",
        "requirement:law:semantic-proof:counterexample-refutation-preservation",
        "counterexample semantic-proof artifacts preserve refutation paths, severity, and review state",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:backend-boundary-is-explicit",
        "requirement:law:semantic-proof:backend-boundary-is-explicit",
        "semantic-proof artifact adapter keeps proof backend execution outside the bounded HG input adapter",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:counterexample-review-policy",
        "requirement:law:semantic-proof:counterexample-review-policy",
        "semantic-proof verification keeps unaccepted counterexamples behind the review boundary when policy requires it",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:insufficient-proof-reinputs-open-obligations",
        "requirement:law:semantic-proof:insufficient-proof-reinputs-open-obligations",
        "semantic-proof input from-report preserves open obligations from insufficient proof reports",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:roundtrip-tests-cover-proof-and-counterexample",
        "requirement:law:semantic-proof:roundtrip-tests-cover-proof-and-counterexample",
        "semantic-proof CLI roundtrip tests cover proved and counterexample artifact paths through verify",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:interpretation-candidates-remain-unreviewed",
        "requirement:law:test-semantics:interpretation-candidates-remain-unreviewed",
        "test-semantics interpretation keeps AI-created cells, morphisms, laws, bindings, and evidence links unreviewed",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:review-accept-does-not-promote-coverage",
        "requirement:law:test-semantics:review-accept-does-not-promote-coverage",
        "test-semantics review accept records explicit review without accepted facts, coverage, or proof objects",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:review-reject-does-not-promote-coverage",
        "requirement:law:test-semantics:review-reject-does-not-promote-coverage",
        "test-semantics review reject records explicit review without accepted facts, coverage, or proof objects",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:verify-positive-gates-promote-coverage",
        "requirement:law:test-semantics:verify-positive-gates-promote-coverage",
        "test-semantics verification promotes reviewed candidates only when review, evidence, and semantic-binding gates pass",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:verify-rejected-review-fails-review-gate",
        "requirement:law:test-semantics:verify-rejected-review-fails-review-gate",
        "test-semantics verification reports not_verified when the review gate is rejected or unaccepted",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:verify-missing-evidence-fails-evidence-gate",
        "requirement:law:test-semantics:verify-missing-evidence-fails-evidence-gate",
        "test-semantics verification reports not_verified when passed execution evidence is missing",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:verify-missing-binding-fails-semantic-binding-gate",
        "requirement:law:test-semantics:verify-missing-binding-fails-semantic-binding-gate",
        "test-semantics verification reports not_verified when the candidate has no semantic target binding",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:verify-does-not-create-proof-objects",
        "requirement:law:test-semantics:verify-does-not-create-proof-objects",
        "test-semantics verification leaves proof objects empty until a separate proof backend verifies obligations",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:gap-missing-obligation-emits-candidate",
        "requirement:law:test-semantics:gap-missing-obligation-emits-candidate",
        "test-semantics gap emits missing-test obstructions and candidates for uncovered expected obligations",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-semantics:gap-no-gaps-when-all-obligations-covered",
        "requirement:law:test-semantics:gap-no-gaps-when-all-obligations-covered",
        "test-semantics gap reports no_gaps_detected when all expected obligations are covered",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:pr-review:input-from-git-emits-bounded-snapshot",
        "requirement:law:pr-review:input-from-git-emits-bounded-snapshot",
        "pr-review input from-git emits bounded changed-file, evidence, context, owner, and risk-signal structure",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:pr-review:git-parser-handles-rename-and-quoted-paths",
        "requirement:law:pr-review:git-parser-handles-rename-and-quoted-paths",
        "pr-review git support handles rename, quoted path, and numstat parser edge cases",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:pr-review:structural-detects-boundary-incidence-composition",
        "requirement:law:pr-review:structural-detects-boundary-incidence-composition",
        "pr-review structural analysis detects boundary, incidence, and composition roles directly",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:pr-review:recommendations-remain-unreviewed",
        "requirement:law:pr-review:recommendations-remain-unreviewed",
        "pr-review generated review targets, obstructions, and completion candidates remain unreviewed suggestions",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    Ok(requirements)
}
