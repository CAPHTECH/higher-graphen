{
    let mut model = StructuralModel::default();
    let has_test_gap_surface = changes
        .iter()
        .any(|change| is_test_gap_surface_path(&change.path));
    let has_semantic_proof_surface = changes
        .iter()
        .any(|change| is_semantic_proof_surface_path(&change.path));
    let has_test_semantics_surface = changes
        .iter()
        .any(|change| is_test_semantics_surface_path(&change.path));
    let has_pr_review_surface = changes
        .iter()
        .any(|change| is_pr_review_surface_path(&change.path));
    if !has_test_gap_surface
        && !has_semantic_proof_surface
        && !has_test_semantics_surface
        && !has_pr_review_surface
    {
        return Ok(model);
    }

    if has_test_gap_surface {
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/main.rs",
            "command:highergraphen:test-gap:detect",
            "highergraphen test-gap detect command cell",
            "highergraphen test-gap detect",
            TestGapSymbolKind::PublicApi,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/main.rs",
            "command:highergraphen:test-gap:input-from-git",
            "highergraphen test-gap input from-git command cell",
            "highergraphen test-gap input from-git",
            TestGapSymbolKind::PublicApi,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/main.rs",
            "command:highergraphen:test-gap:input-from-path",
            "highergraphen test-gap input from-path command cell",
            "highergraphen test-gap input from-path",
            TestGapSymbolKind::PublicApi,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/test_gap_git.rs",
            "adapter:test-gap:git-input",
            "test-gap git input adapter cell",
            "test_gap_git::input_from_git",
            TestGapSymbolKind::Module,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/test_gap_git.rs",
            "adapter:test-gap:path-input",
            "test-gap current-tree path input adapter cell",
            "test_gap_git::input_from_path",
            TestGapSymbolKind::Module,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/workflows/test_gap.rs",
            "runner:test-gap:detect",
            "run_test_gap_detect workflow runner cell",
            "run_test_gap_detect",
            TestGapSymbolKind::Function,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/lib.rs",
            "export:test-gap:runtime-api",
            "test-gap runtime public export cell",
            "higher_graphen_runtime test-gap exports",
            TestGapSymbolKind::PublicApi,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/workflows/mod.rs",
            "registry:test-gap:workflow-module",
            "test-gap workflow registry cell",
            "workflows::test_gap module registry",
            TestGapSymbolKind::Module,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/test_gap_reports.rs",
            "contract:test-gap:runtime-report-shapes",
            "test-gap runtime report shape contract cell",
            "TestGap input and report runtime shapes",
            TestGapSymbolKind::Type,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/reports.rs",
            "projection:test-gap:report-envelope",
            "test-gap report envelope projection cell",
            "ReportEnvelope projection boundary for test-gap",
            TestGapSymbolKind::Type,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "schemas/inputs/test-gap.input.schema.json",
            "schema:test-gap:input",
            "test-gap input schema contract cell",
            "highergraphen.test_gap.input.v1 schema",
            TestGapSymbolKind::Unknown,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "schemas/reports/test-gap.report.schema.json",
            "schema:test-gap:report",
            "test-gap report schema contract cell",
            "highergraphen.test_gap.report.v1 schema",
            TestGapSymbolKind::Unknown,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "schemas/inputs/test-gap.input.example.json",
            "fixture:test-gap:input-example",
            "test-gap input example fixture cell",
            "test-gap input example fixture",
            TestGapSymbolKind::Unknown,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "schemas/reports/test-gap.report.example.json",
            "fixture:test-gap:report-example",
            "test-gap report example fixture cell",
            "test-gap report example fixture",
            TestGapSymbolKind::Unknown,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "scripts/validate-json-contracts.py",
            "validator:test-gap:json-contracts",
            "JSON contract validation command cell",
            "scripts/validate-json-contracts.py",
            TestGapSymbolKind::Unknown,
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/main.rs"],
            "law:test-gap:command-routes-to-runner",
            "CLI command routes to the intended test-gap runner",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/main.rs"],
            "law:test-gap:json-format-required",
            "test-gap CLI commands require --format json",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/main.rs"],
            "law:test-gap:output-file-suppresses-stdout",
            "test-gap CLI --output writes the target file without JSON stdout",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/test_gap_git.rs"],
            "law:test-gap:input-from-git-is-deterministic",
            "test-gap input from-git derives a deterministic bounded snapshot",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/test_gap_git.rs"],
            "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
            "test-gap input from-git declares that git structure does not prove semantic coverage",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/test_gap_git.rs"],
            "law:test-gap:input-from-path-is-deterministic",
            "test-gap input from-path derives a deterministic bounded snapshot from selected current-tree files",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/test_gap_git.rs"],
            "law:test-gap:input-from-path-declares-snapshot-boundary",
            "test-gap input from-path keeps current-tree scope and semantic coverage limits visible",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["crates/higher-graphen-runtime/src/workflows/test_gap.rs"],
            "law:test-gap:test-gap-is-bounded",
            "no_gaps_in_snapshot is bounded to the supplied snapshot and detector policy",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["crates/higher-graphen-runtime/src/workflows/test_gap.rs"],
            "law:test-gap:verification-policy-controls-test-kind",
            "detector_context.test_kinds controls which test kinds close obligations",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["crates/higher-graphen-runtime/src/workflows/test_gap.rs"],
            "law:test-gap:requirements-map-to-implementation-and-test",
            "in-scope requirements map to implementation cells and accepted verification cells",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "crates/higher-graphen-runtime/src/workflows/test_gap.rs",
                "crates/higher-graphen-runtime/src/test_gap_reports.rs",
            ],
            "law:test-gap:candidates-remain-unreviewed",
            "detector completion candidates remain unreviewed until explicit review",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "crates/higher-graphen-runtime/src/workflows/test_gap.rs",
                "crates/higher-graphen-runtime/src/reports.rs",
            ],
            "law:test-gap:projection-declares-information-loss",
            "test-gap projections declare information loss for human, AI, and audit views",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "crates/higher-graphen-runtime/src/test_gap_reports.rs",
                "schemas/inputs/test-gap.input.schema.json",
                "schemas/reports/test-gap.report.schema.json",
            ],
            "law:test-gap:schema-id-preserved",
            "test-gap input and report schema IDs are preserved",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["crates/higher-graphen-runtime/src/test_gap_reports.rs"],
            "law:test-gap:enum-casing-round-trips",
            "test-gap enum casing serializes as lower snake case and round-trips",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "crates/higher-graphen-runtime/src/test_gap_reports.rs",
                "schemas/inputs/test-gap.input.schema.json",
                "schemas/reports/test-gap.report.schema.json",
            ],
            "law:test-gap:runtime-shapes-preserve-schema",
            "runtime TestGap shapes preserve the checked-in schema boundary",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "scripts/validate-json-contracts.py",
                "schemas/inputs/test-gap.input.example.json",
                "schemas/reports/test-gap.report.example.json",
                "schemas/inputs/test-gap.input.schema.json",
                "schemas/reports/test-gap.report.schema.json",
            ],
            "law:test-gap:fixtures-validate-against-schema",
            "test-gap fixtures validate against their declared JSON schemas",
        )?;

        push_structural_edge(
            &mut model,
            "edge:test-gap:command-detect-to-runner",
            "command:highergraphen:test-gap:detect",
            "runner:test-gap:detect",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:input-from-git-to-adapter",
            "command:highergraphen:test-gap:input-from-git",
            "adapter:test-gap:git-input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:input-from-path-to-adapter",
            "command:highergraphen:test-gap:input-from-path",
            "adapter:test-gap:path-input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:git-adapter-to-input-schema",
            "adapter:test-gap:git-input",
            "schema:test-gap:input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:path-adapter-to-input-schema",
            "adapter:test-gap:path-input",
            "schema:test-gap:input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:runtime-export-to-runner",
            "export:test-gap:runtime-api",
            "runner:test-gap:detect",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:workflow-registry-to-runner",
            "registry:test-gap:workflow-module",
            "runner:test-gap:detect",
            TestGapDependencyRelationType::Contains,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:runtime-shapes-to-input-schema",
            "contract:test-gap:runtime-report-shapes",
            "schema:test-gap:input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:runtime-shapes-to-report-schema",
            "contract:test-gap:runtime-report-shapes",
            "schema:test-gap:report",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:input-fixture-to-input-schema",
            "fixture:test-gap:input-example",
            "schema:test-gap:input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:report-fixture-to-report-schema",
            "fixture:test-gap:report-example",
            "schema:test-gap:report",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:report-envelope-to-runtime-shapes",
            "projection:test-gap:report-envelope",
            "contract:test-gap:runtime-report-shapes",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:validator-to-input-fixture",
            "validator:test-gap:json-contracts",
            "fixture:test-gap:input-example",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:validator-to-report-fixture",
            "validator:test-gap:json-contracts",
            "fixture:test-gap:report-example",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;

        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:command-detect-to-runner",
            "command_to_runner",
            &["command:highergraphen:test-gap:detect"],
            &["runner:test-gap:detect"],
            &["law:test-gap:command-routes-to-runner"],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:input-from-git-to-input-schema",
            "adapter_to_input_schema",
            &[
                "command:highergraphen:test-gap:input-from-git",
                "adapter:test-gap:git-input",
            ],
            &["schema:test-gap:input"],
            &[
                "law:test-gap:input-from-git-is-deterministic",
                "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
            ],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:input-from-path-to-input-schema",
            "adapter_to_input_schema",
            &[
                "command:highergraphen:test-gap:input-from-path",
                "adapter:test-gap:path-input",
            ],
            &["schema:test-gap:input"],
            &[
                "law:test-gap:input-from-path-is-deterministic",
                "law:test-gap:input-from-path-declares-snapshot-boundary",
            ],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:runtime-shapes-to-schemas",
            "runtime_shape_to_schema",
            &["contract:test-gap:runtime-report-shapes"],
            &["schema:test-gap:input", "schema:test-gap:report"],
            &[
                "law:test-gap:schema-id-preserved",
                "law:test-gap:enum-casing-round-trips",
                "law:test-gap:runtime-shapes-preserve-schema",
            ],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:fixtures-to-schemas",
            "fixture_to_schema",
            &[
                "fixture:test-gap:input-example",
                "fixture:test-gap:report-example",
            ],
            &["schema:test-gap:input", "schema:test-gap:report"],
            &["law:test-gap:fixtures-validate-against-schema"],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:report-envelope-to-runtime-shapes",
            "projection_to_runtime_shape",
            &["projection:test-gap:report-envelope"],
            &["contract:test-gap:runtime-report-shapes"],
            &["law:test-gap:projection-declares-information-loss"],
            diff_evidence_id,
        )?;
    }

    if has_semantic_proof_surface {
        push_semantic_proof_structural_model(&mut model, changes, diff_evidence_id)?;
    }
    if has_test_semantics_surface {
        push_test_semantics_structural_model(&mut model, changes, diff_evidence_id)?;
    }
    if has_pr_review_surface {
        push_pr_review_structural_model(&mut model, changes, diff_evidence_id)?;
    }

    Ok(model)
}
