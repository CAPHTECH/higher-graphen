fn tests_for_changes(
    changes: &[GitChange],
    symbols: &[TestGapInputSymbol],
    content_test_targets: &BTreeMap<String, Vec<Id>>,
    rust_test_files: &BTreeSet<String>,
    diff_evidence_id: &Id,
) -> Result<Vec<TestGapInputTest>, String> {
    let mut tests = changes
        .iter()
        .filter(|change| is_test_path(&change.path) || rust_test_files.contains(&change.path))
        .map(|change| {
            let file_id = file_id(&change.path)?;
            let mut target_ids = matching_symbol_ids(&change.path, symbols);
            if let Some(content_target_ids) = content_test_targets.get(&change.path) {
                for target_id in content_target_ids {
                    push_unique_id(&mut target_ids, target_id.clone());
                }
            }
            Ok::<TestGapInputTest, String>(TestGapInputTest {
                id: id(format!("test:{}", slug(&change.path)))?,
                name: format!("Changed test file {}", change.path),
                test_type: test_gap_test_type_for_observed_rust_test(&change.path),
                file_id: Some(file_id),
                target_ids,
                branch_ids: Vec::new(),
                requirement_ids: Vec::new(),
                is_regression: false,
                context_ids: vec![id("context:test-scope")?],
                source_ids: vec![diff_evidence_id.clone()],
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if changes.iter().any(|change| {
        change.path == "scripts/validate-json-contracts.py" || change.path.ends_with(".schema.json")
    }) && symbols
        .iter()
        .any(|symbol| symbol.id.as_str() == "validator:test-gap:json-contracts")
    {
        let target_ids = [
            "validator:test-gap:json-contracts",
            "law:test-gap:fixtures-validate-against-schema",
        ]
        .into_iter()
        .filter(|symbol_id| {
            symbols
                .iter()
                .any(|symbol| symbol.id.as_str() == *symbol_id)
        })
        .map(id)
        .collect::<Result<Vec<_>, _>>()?;
        tests.push(TestGapInputTest {
            id: id("test:validator:test-gap-json-contracts")?,
            name: "JSON contract validator command".to_owned(),
            test_type: TestGapTestType::Smoke,
            file_id: Some(file_id(
                changes
                    .iter()
                    .find(|change| change.path == "scripts/validate-json-contracts.py")
                    .or_else(|| {
                        changes
                            .iter()
                            .find(|change| change.path.ends_with(".schema.json"))
                    })
                    .map(|change| change.path.as_str())
                    .unwrap_or("scripts/validate-json-contracts.py"),
            )?),
            target_ids,
            branch_ids: Vec::new(),
            requirement_ids: Vec::new(),
            is_regression: false,
            context_ids: vec![id("context:test-scope")?],
            source_ids: vec![diff_evidence_id.clone()],
        });
    }

    Ok(tests)
}

fn verification_cells_for_tests(
    tests: &[TestGapInputTest],
    structural: &StructuralModel,
    diff_evidence_id: &Id,
) -> Result<Vec<TestGapVerificationCell>, String> {
    tests
        .iter()
        .map(|test| {
            let law_ids = structural
                .laws
                .iter()
                .filter(|law| {
                    test.target_ids.contains(&law.id)
                        || test.requirement_ids.iter().any(|requirement_id| {
                            requirement_id.as_str()
                                == format!(
                                    "requirement:law:{}",
                                    law.id.as_str().trim_start_matches("law:")
                                )
                        })
                })
                .map(|law| law.id.clone())
                .collect::<Vec<_>>();
            let morphism_ids = structural
                .morphisms
                .iter()
                .filter(|morphism| {
                    test.target_ids.contains(&morphism.id)
                        || test.target_ids.iter().any(|target_id| {
                            morphism.source_ids.contains(target_id)
                                || morphism.target_ids.contains(target_id)
                                || morphism.law_ids.contains(target_id)
                        })
                        || test_semantically_covers_morphism(test, morphism)
                        || test.requirement_ids.iter().any(|requirement_id| {
                            requirement_id.as_str()
                                == format!(
                                    "requirement:morphism:{}",
                                    morphism.id.as_str().trim_start_matches("morphism:")
                                )
                        })
                })
                .map(|morphism| morphism.id.clone())
                .collect::<Vec<_>>();
            let mut law_ids = law_ids;
            for morphism in structural
                .morphisms
                .iter()
                .filter(|morphism| morphism_ids.contains(&morphism.id))
            {
                for law_id in &morphism.law_ids {
                    push_unique_id(&mut law_ids, law_id.clone());
                }
            }
            Ok(TestGapVerificationCell {
                id: id(format!("verification:{}", slug(test.id.as_str())))?,
                name: format!("Verification cell for {}", test.name),
                verification_type: if test.test_type == TestGapTestType::Smoke {
                    "validator".to_owned()
                } else {
                    "automated_test".to_owned()
                },
                test_type: test.test_type,
                target_ids: test.target_ids.clone(),
                requirement_ids: test.requirement_ids.clone(),
                law_ids,
                morphism_ids,
                source_ids: vec![diff_evidence_id.clone(), test.id.clone()],
                confidence: Some(confidence(0.72)?),
            })
        })
        .collect()
}

fn test_semantically_covers_morphism(
    test: &TestGapInputTest,
    morphism: &TestGapInputMorphism,
) -> bool {
    if !morphism.morphism_type.starts_with("semantic_") {
        return false;
    }
    if morphism
        .source_ids
        .iter()
        .chain(morphism.target_ids.iter())
        .filter_map(semantic_endpoint_path_slug)
        .any(|path_slug| {
            matches!(
                path_slug,
                "tools-highergraphen-cli-src-semantic-proof-artifact-rs"
                    | "tools-highergraphen-cli-src-semantic-proof-backend-rs"
                    | "tools-highergraphen-cli-src-semantic-proof-reinput-rs"
            )
        })
    {
        return false;
    }
    let target_path_slugs = test
        .target_ids
        .iter()
        .filter_map(test_target_path_slug)
        .collect::<BTreeSet<_>>();
    if !target_path_slugs.is_empty()
        && morphism
            .source_ids
            .iter()
            .chain(morphism.target_ids.iter())
            .filter_map(semantic_endpoint_path_slug)
            .any(|path_slug| target_path_slugs.contains(path_slug))
    {
        return true;
    }
    let targets_json_contracts = test
        .target_ids
        .iter()
        .any(|target_id| target_id.as_str() == "validator:test-gap:json-contracts");
    targets_json_contracts
        && morphism
            .source_ids
            .iter()
            .chain(morphism.target_ids.iter())
            .any(|endpoint_id| endpoint_id.as_str().starts_with("semantic:json-schema:"))
}

fn test_target_path_slug(target_id: &Id) -> Option<&str> {
    let value = target_id.as_str();
    value
        .strip_prefix("symbol:")
        .and_then(|value| value.strip_suffix(":changed-behavior"))
        .or_else(|| structural_cell_path_slug(value))
}

fn structural_cell_path_slug(target_id: &str) -> Option<&'static str> {
    match target_id {
        "command:highergraphen:test-gap:detect"
        | "command:highergraphen:test-gap:input-from-git"
        | "command:highergraphen:test-gap:input-from-path" => {
            Some("tools-highergraphen-cli-src-main-rs")
        }
        "adapter:test-gap:git-input" | "adapter:test-gap:path-input" => {
            Some("tools-highergraphen-cli-src-test-gap-git-rs")
        }
        "runner:test-gap:detect" => Some("crates-higher-graphen-runtime-src-workflows-test-gap-rs"),
        "export:test-gap:runtime-api" => Some("crates-higher-graphen-runtime-src-lib-rs"),
        "registry:test-gap:workflow-module" => {
            Some("crates-higher-graphen-runtime-src-workflows-mod-rs")
        }
        "contract:test-gap:runtime-report-shapes" => {
            Some("crates-higher-graphen-runtime-src-test-gap-reports-rs")
        }
        "projection:test-gap:report-envelope" => {
            Some("crates-higher-graphen-runtime-src-reports-rs")
        }
        "schema:test-gap:input" => Some("schemas-inputs-test-gap-input-schema-json"),
        "schema:test-gap:report" => Some("schemas-reports-test-gap-report-schema-json"),
        "command:highergraphen:semantic-proof:input-from-artifact"
        | "command:highergraphen:semantic-proof:backend-run"
        | "command:highergraphen:semantic-proof:input-from-report"
        | "command:highergraphen:semantic-proof:verify" => {
            Some("tools-highergraphen-cli-src-main-rs")
        }
        "runner:semantic-proof:backend-run"
        | "theorem:semantic-proof:backend-run-trust-boundary" => {
            Some("tools-highergraphen-cli-src-semantic-proof-backend-rs")
        }
        "adapter:semantic-proof:artifact-input"
        | "theorem:semantic-proof:artifact-adapter-correctness" => {
            Some("tools-highergraphen-cli-src-semantic-proof-artifact-rs")
        }
        "adapter:semantic-proof:reinput-from-report"
        | "theorem:semantic-proof:obligation-reinput-correctness" => {
            Some("tools-highergraphen-cli-src-semantic-proof-reinput-rs")
        }
        "test:semantic-proof:artifact-roundtrip" => {
            Some("tools-highergraphen-cli-tests-command-rs")
        }
        "test:semantic-proof:backend-and-reinput" => {
            Some("tools-highergraphen-cli-tests-command-rs")
        }
        _ => None,
    }
}

fn semantic_endpoint_path_slug(endpoint_id: &Id) -> Option<&str> {
    let mut parts = endpoint_id.as_str().split(':');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("semantic"), Some("rust"), Some(_), Some(path_slug))
        | (Some("semantic"), Some("json-schema"), Some(_), Some(path_slug)) => Some(path_slug),
        _ => None,
    }
}

fn link_tests_to_requirements(
    tests: Vec<TestGapInputTest>,
    requirements: &[TestGapInputRequirement],
) -> Vec<TestGapInputTest> {
    tests
        .into_iter()
        .map(|mut test| {
            test.requirement_ids = matching_requirement_ids(&test.target_ids, requirements);
            test
        })
        .collect()
}

fn coverage_for_tests(
    tests: &[TestGapInputTest],
    accepted_test_kinds: &[TestGapTestType],
) -> Result<Vec<TestGapInputCoverage>, String> {
    let mut coverage = Vec::new();
    for test in tests
        .iter()
        .filter(|test| accepted_test_kinds.contains(&test.test_type))
    {
        for target_id in &test.target_ids {
            coverage.push(TestGapInputCoverage {
                id: id(format!(
                    "coverage:{}:{}",
                    slug(test.id.as_str()),
                    slug(target_id.as_str())
                ))?,
                coverage_type: TestGapCoverageType::Function,
                target_id: target_id.clone(),
                status: TestGapCoverageStatus::Covered,
                covered_by_test_ids: vec![test.id.clone()],
                source_ids: test.source_ids.clone(),
                summary: Some(format!(
                    "Git adapter matched {} to {}",
                    test.name, target_id
                )),
                confidence: Some(confidence(0.62)?),
            });
        }
    }
    Ok(coverage)
}

fn contexts_for_changes(
    changes: &[GitChange],
    change_set_id: &Id,
    review_focus_name: &str,
) -> Result<Vec<TestGapInputContext>, String> {
    let mut contexts = BTreeMap::<Id, (String, TestGapContextType)>::new();
    contexts.insert(
        id("context:repository")?,
        ("Repository".to_owned(), TestGapContextType::Repository),
    );
    contexts.insert(
        id(format!("context:{}", slug(change_set_id.as_str())))?,
        (
            review_focus_name.to_owned(),
            TestGapContextType::ReviewFocus,
        ),
    );

    for change in changes {
        for (context_id, name, context_type) in test_gap_context_descriptors_for_path(&change.path)?
        {
            contexts.insert(context_id, (name, context_type));
        }
    }

    Ok(contexts
        .into_iter()
        .map(|(id, (name, context_type))| TestGapInputContext {
            id,
            name,
            context_type,
            source_ids: Vec::new(),
        })
        .collect())
}

fn evidence_for_changes(
    changes: &[GitChange],
    commits: &[String],
    diff_evidence_id: &Id,
    commit_evidence_id: &Id,
) -> Result<Vec<TestGapInputEvidence>, String> {
    let mut evidence = vec![TestGapInputEvidence {
        id: diff_evidence_id.clone(),
        evidence_type: TestGapEvidenceType::DiffHunk,
        summary: format!(
            "Git diff contains {} changed files with {} additions and {} deletions.",
            changes.len(),
            changes.iter().map(|change| change.additions).sum::<u32>(),
            changes.iter().map(|change| change.deletions).sum::<u32>()
        ),
        source_ids: changes
            .iter()
            .map(|change| file_id(&change.path))
            .collect::<Result<Vec<_>, _>>()?,
        confidence: Some(confidence(1.0)?),
    }];

    if !commits.is_empty() {
        evidence.push(TestGapInputEvidence {
            id: commit_evidence_id.clone(),
            evidence_type: TestGapEvidenceType::Custom,
            summary: format!(
                "Git range contains {} commits: {}",
                commits.len(),
                commits.join("; ")
            ),
            source_ids: changes
                .iter()
                .map(|change| file_id(&change.path))
                .collect::<Result<Vec<_>, _>>()?,
            confidence: Some(confidence(0.95)?),
        });
    }

    Ok(evidence)
}
