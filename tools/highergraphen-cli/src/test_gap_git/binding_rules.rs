#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DefaultHgRustTestBindingRule {
    trigger_terms: &'static [&'static str],
    cli_label: Option<&'static str>,
    target_ids: &'static [&'static str],
}

const DEFAULT_HG_RUST_TEST_BINDING_RULES: &[DefaultHgRustTestBindingRule] = &[
    DefaultHgRustTestBindingRule {
        trigger_terms: &["test-gap", "input", "from-git"],
        cli_label: Some("highergraphen test-gap input from-git"),
        target_ids: &[
            "command:highergraphen:test-gap:input-from-git",
            "adapter:test-gap:git-input",
            "morphism:test-gap:input-from-git-to-input-schema",
            "law:test-gap:input-from-git-is-deterministic",
            "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["test-gap", "input", "from-path"],
        cli_label: Some("highergraphen test-gap input from-path"),
        target_ids: &[
            "command:highergraphen:test-gap:input-from-path",
            "adapter:test-gap:path-input",
            "morphism:test-gap:input-from-path-to-input-schema",
            "law:test-gap:input-from-path-is-deterministic",
            "law:test-gap:input-from-path-declares-snapshot-boundary",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["test-gap", "evidence", "from-test-run"],
        cli_label: Some("highergraphen test-gap evidence from-test-run"),
        target_ids: &[],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["test-gap", "detect"],
        cli_label: Some("highergraphen test-gap detect"),
        target_ids: &[
            "command:highergraphen:test-gap:detect",
            "runner:test-gap:detect",
            "morphism:test-gap:command-detect-to-runner",
            "law:test-gap:command-routes-to-runner",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["--format", "json"],
        cli_label: None,
        target_ids: &["law:test-gap:json-format-required"],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["highergraphen.test_gap.input.v1"],
        cli_label: None,
        target_ids: &["schema:test-gap:input", "law:test-gap:schema-id-preserved"],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["schema"],
        cli_label: None,
        target_ids: &["schema:test-gap:input", "law:test-gap:schema-id-preserved"],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["test_semantics_interpret_emits_unreviewed_ai_candidate_structure"],
        cli_label: Some("highergraphen test-semantics interpret"),
        target_ids: &[
            "adapter:test-semantics:interpretation",
            "law:test-semantics:interpretation-candidates-remain-unreviewed",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &[
            "test_semantics_review_accepts_candidate_without_promoting_coverage_or_proof",
        ],
        cli_label: Some("highergraphen test-semantics review accept"),
        target_ids: &[
            "adapter:test-semantics:review",
            "law:test-semantics:review-accept-does-not-promote-coverage",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &[
            "test_semantics_review_rejects_candidate_without_promoting_coverage_or_proof",
        ],
        cli_label: Some("highergraphen test-semantics review reject"),
        target_ids: &[
            "adapter:test-semantics:review",
            "law:test-semantics:review-reject-does-not-promote-coverage",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &[
            "test_semantics_verify_promotes_reviewed_candidate_with_execution_evidence",
        ],
        cli_label: Some("highergraphen test-semantics verify"),
        target_ids: &[
            "adapter:test-semantics:verification",
            "law:test-semantics:verify-positive-gates-promote-coverage",
            "law:test-semantics:verify-does-not-create-proof-objects",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["test_semantics_verify_rejected_review_fails_review_gate"],
        cli_label: Some("highergraphen test-semantics verify"),
        target_ids: &[
            "adapter:test-semantics:verification",
            "law:test-semantics:verify-rejected-review-fails-review-gate",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["test_semantics_verify_missing_evidence_fails_evidence_gate"],
        cli_label: Some("highergraphen test-semantics verify"),
        target_ids: &[
            "adapter:test-semantics:verification",
            "law:test-semantics:verify-missing-evidence-fails-evidence-gate",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["test_semantics_verify_missing_binding_fails_semantic_binding_gate"],
        cli_label: Some("highergraphen test-semantics verify"),
        target_ids: &[
            "adapter:test-semantics:verification",
            "law:test-semantics:verify-missing-binding-fails-semantic-binding-gate",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["test_semantics_gap_detects_missing_expected_obligation"],
        cli_label: Some("highergraphen test-semantics gap"),
        target_ids: &[
            "adapter:test-semantics:gap",
            "law:test-semantics:gap-missing-obligation-emits-candidate",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["pr_review_input_from_git_emits_bounded_snapshot"],
        cli_label: Some("highergraphen pr-review input from-git"),
        target_ids: &[
            "adapter:pr-review:git-input",
            "law:pr-review:input-from-git-emits-bounded-snapshot",
        ],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["pr_review_input_from_git_output_feeds_recommender"],
        cli_label: Some("highergraphen pr-review targets recommend"),
        target_ids: &["law:pr-review:recommendations-remain-unreviewed"],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &[
            "pr_review_targets_recommend_reads_fixture_and_writes_one_json_report_to_stdout",
        ],
        cli_label: Some("highergraphen pr-review targets recommend"),
        target_ids: &["law:pr-review:recommendations-remain-unreviewed"],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["parses_rename_name_status_with_quoted_path_and_binary_numstat"],
        cli_label: None,
        target_ids: &["law:pr-review:git-parser-handles-rename-and-quoted-paths"],
    },
    DefaultHgRustTestBindingRule {
        trigger_terms: &["detects_boundary_incidence_and_composition_roles"],
        cli_label: None,
        target_ids: &["law:pr-review:structural-detects-boundary-incidence-composition"],
    },
];

fn binding_rules_from_path(path: Option<&Path>) -> Result<HgRustTestBindingRules, String> {
    match path {
        Some(path) => read_binding_rules(path),
        None => Ok(default_binding_rules()),
    }
}

fn default_binding_rules() -> HgRustTestBindingRules {
    HgRustTestBindingRules {
        rules: DEFAULT_HG_RUST_TEST_BINDING_RULES
            .iter()
            .map(|rule| HgRustTestBindingRule {
                trigger_terms: rule
                    .trigger_terms
                    .iter()
                    .map(|term| (*term).to_owned())
                    .collect(),
                cli_label: rule.cli_label.map(str::to_owned),
                target_ids: rule
                    .target_ids
                    .iter()
                    .map(|target_id| (*target_id).to_owned())
                    .collect(),
            })
            .collect(),
    }
}

fn read_binding_rules(path: &Path) -> Result<HgRustTestBindingRules, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read binding rules {}: {error}", path.display()))?;
    let value: Value = serde_json::from_str(&contents)
        .map_err(|error| format!("failed to parse binding rules {}: {error}", path.display()))?;
    parse_binding_rules_value(&value)
}

fn parse_binding_rules_value(value: &Value) -> Result<HgRustTestBindingRules, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "binding rules document must be an object".to_owned())?;
    let schema = object
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| "binding rules document needs schema".to_owned())?;
    if schema != BINDING_RULES_SCHEMA {
        return Err(format!(
            "binding rules schema must be {BINDING_RULES_SCHEMA}, got {schema}"
        ));
    }
    let rules = object
        .get("rules")
        .and_then(Value::as_array)
        .ok_or_else(|| "binding rules document needs rules array".to_owned())?;
    rules
        .iter()
        .enumerate()
        .map(|(index, value)| parse_binding_rule(index, value))
        .collect::<Result<Vec<_>, _>>()
        .map(|rules| HgRustTestBindingRules { rules })
}

fn parse_binding_rule(index: usize, value: &Value) -> Result<HgRustTestBindingRule, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("rules[{index}] must be an object"))?;
    let trigger_terms = string_array_field(object, "trigger_terms")
        .map_err(|error| format!("rules[{index}].{error}"))?;
    if trigger_terms.is_empty() {
        return Err(format!("rules[{index}].trigger_terms must not be empty"));
    }
    let target_ids = string_array_field(object, "target_ids")
        .map_err(|error| format!("rules[{index}].{error}"))?;
    let cli_label = object
        .get("cli_label")
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("rules[{index}].cli_label must be a non-empty string"))
        })
        .transpose()?;
    Ok(HgRustTestBindingRule {
        trigger_terms,
        cli_label,
        target_ids,
    })
}

fn string_array_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Vec<String>, String> {
    object
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{field} must be an array"))?
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("{field}[{index}] must be a non-empty string"))
        })
        .collect()
}
