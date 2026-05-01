use super::{
    confidence, finalize_validation, generated_id, generated_provenance, metadata_extensions,
    payload_ref, source_uri, CaseGraphenCoreExtensions,
};
use crate::{
    native_model::{CaseMorphism, CaseMorphismType, CaseSpace},
    native_review::NativeCloseCheck,
};
use higher_graphen_core::{
    Capability, CapabilityOperation, CapabilityStatus, CriterionDirection, CriterionValue,
    Derivation, DerivationFailureMode, Description, EquivalenceClaim, EquivalenceCriterion,
    EquivalenceKind, EquivalenceScope, Id, InferenceRule, LifecycleStatus, ObjectRef, OrderType,
    Policy, PolicyApplicability, PolicyKind, PolicyRule, PolicyStatus, Provenance, Reachability,
    ReviewStatus, Scenario, ScenarioChanges, ScenarioKind, ScenarioStatus, SchemaCompatibility,
    SchemaMapping, SchemaMappingKind, SchemaMorphism, SchemaVerification, Valuation,
    ValuationCriterion, ValuationValue, VerificationStatus, Verifier, VerifierKind, Witness,
    WitnessStatus, WitnessType,
};
use serde_json::{json, Value};

pub fn native_close_check_extensions(
    case_space: &CaseSpace,
    check: &NativeCloseCheck,
) -> CaseGraphenCoreExtensions {
    let provenance = close_check_provenance(case_space, check);
    let witnesses = close_invariant_witnesses(case_space, check, &provenance);
    let derivation = close_derivation(case_space, check, &witnesses, &provenance);

    let generated = CaseGraphenCoreExtensions {
        witnesses,
        derivations: vec![derivation],
        policies: close_policy(case_space, check, &provenance)
            .into_iter()
            .collect(),
        capabilities: vec![close_capability(case_space, check, &provenance)],
        ..CaseGraphenCoreExtensions::default()
    };
    let mut extensions = metadata_extensions(&case_space.metadata);
    extensions.append(generated);
    finalize_validation(&mut extensions);
    extensions
}

pub fn native_morphism_check_extensions(
    case_space: &CaseSpace,
    morphism: &CaseMorphism,
) -> CaseGraphenCoreExtensions {
    let provenance = morphism_check_provenance(case_space, morphism);
    let validation_witness = morphism_validation_witness(case_space, morphism, &provenance);
    let validation_witness_id = validation_witness.id.clone();

    let generated = CaseGraphenCoreExtensions {
        witnesses: vec![validation_witness],
        scenarios: vec![morphism_scenario(
            case_space,
            morphism,
            &validation_witness_id,
            &provenance,
        )],
        schema_morphisms: vec![schema_morphism_for_native(
            case_space,
            morphism,
            &validation_witness_id,
            provenance.clone(),
        )],
        equivalence_claims: morphism_equivalence_claims(
            case_space,
            morphism,
            &validation_witness_id,
            &provenance,
        ),
        valuations: vec![morphism_applicability_valuation(
            case_space,
            morphism,
            &validation_witness_id,
            provenance,
        )],
        ..CaseGraphenCoreExtensions::default()
    };
    let mut extensions = metadata_extensions(&case_space.metadata);
    extensions.append(metadata_extensions(&morphism.metadata));
    extensions.append(generated);
    finalize_validation(&mut extensions);
    extensions
}

pub fn native_close_check_result(
    mut check: NativeCloseCheck,
    core_extensions: CaseGraphenCoreExtensions,
) -> Value {
    let core_extension_blocked = core_extensions.is_blocked();
    if core_extension_blocked {
        check.closeable = false;
    }
    json!({
        "close_check": check,
        "core_extension_blocked": core_extension_blocked,
        "core_extensions": core_extensions
    })
}

pub fn native_morphism_check_result(
    morphism: CaseMorphism,
    core_extensions: CaseGraphenCoreExtensions,
) -> Value {
    json!({
        "valid": true,
        "applicable": !core_extensions.is_blocked(),
        "morphism": morphism,
        "core_extensions": core_extensions
    })
}

fn close_check_provenance(case_space: &CaseSpace, check: &NativeCloseCheck) -> Provenance {
    generated_provenance(
        source_uri(
            "native",
            &case_space.case_space_id,
            "close-check",
            &check.check_id,
        ),
        "CaseGraphen native close-check",
        ReviewStatus::Reviewed,
        0.92,
    )
}

fn close_invariant_witnesses(
    case_space: &CaseSpace,
    check: &NativeCloseCheck,
    provenance: &Provenance,
) -> Vec<Witness> {
    check
        .invariant_results
        .iter()
        .map(|result| {
            let mut witness = Witness::candidate(
                generated_id(
                    "witness",
                    &[check.check_id.as_str(), result.invariant_id.as_str()],
                ),
                WitnessType::MachineCheckResult,
                payload_ref(
                    "casegraphen_native_close_invariant",
                    source_uri(
                        "native",
                        &case_space.case_space_id,
                        "close-invariant",
                        &result.invariant_id,
                    ),
                ),
                &case_space.revision.created_at,
                provenance.clone(),
                confidence(if result.passed { 0.94 } else { 0.88 }),
            )
            .expect("generated witness is valid");
            witness.supports = vec![ObjectRef::new(result.invariant_id.clone())];
            witness.validity_contexts =
                vec![case_space.case_space_id.clone(), check.check_id.clone()];
            witness.review_status = WitnessStatus::Candidate;
            witness
        })
        .collect()
}

fn close_derivation(
    case_space: &CaseSpace,
    check: &NativeCloseCheck,
    witnesses: &[Witness],
    provenance: &Provenance,
) -> Derivation {
    let mut rule = InferenceRule::new(
        generated_id("rule", &["casegraphen", "native-close-check"]),
        "Native close-check invariant conjunction",
    )
    .expect("generated rule is valid");
    rule.rule_scope_contexts = vec![case_space.case_space_id.clone()];

    let mut derivation = Derivation::candidate(
        generated_id("derivation", &[check.check_id.as_str(), "closeability"]),
        check.check_id.clone(),
        check
            .invariant_results
            .iter()
            .map(|result| result.invariant_id.clone())
            .collect(),
        rule,
        provenance.clone(),
    );
    derivation.warrants = witnesses.iter().map(|witness| witness.id.clone()).collect();
    derivation.verifier = Some(
        Verifier::new(VerifierKind::CustomEngine, "casegraphen case close-check")
            .expect("generated verifier is valid"),
    );
    derivation.verification_status = VerificationStatus::MachineChecked;
    derivation.failure_mode = if check.closeable {
        DerivationFailureMode::None
    } else {
        DerivationFailureMode::MissingPremise
    };
    derivation
}

fn close_policy(
    case_space: &CaseSpace,
    check: &NativeCloseCheck,
    provenance: &Provenance,
) -> Option<Policy> {
    check.close_policy_id.as_ref().map(|policy_id| Policy {
        id: policy_id.clone(),
        policy_kind: PolicyKind::CandidateAcceptance,
        applies_to: PolicyApplicability {
            target_types: vec!["native_case_space".to_owned(), "native_close_check".to_owned()],
            contexts: vec![case_space.case_space_id.clone()],
            operations: vec!["close-check".to_owned()],
        },
        rule: PolicyRule::new(
            "Native closure requires passing close invariants, accepted evidence, reviewed completion and morphism state, and declared projection loss.",
        )
        .expect("generated policy rule is valid"),
        required_witnesses: check
            .invariant_results
            .iter()
            .map(|result| generated_id("witness", &[check.check_id.as_str(), result.invariant_id.as_str()]))
            .collect(),
        required_derivations: vec![generated_id("derivation", &[check.check_id.as_str(), "closeability"])],
        escalation_path: Vec::new(),
        violation_obstruction_template: None,
        status: PolicyStatus::Draft,
        provenance: provenance.clone(),
        review_status: ReviewStatus::Reviewed,
    })
}

fn close_capability(
    case_space: &CaseSpace,
    check: &NativeCloseCheck,
    provenance: &Provenance,
) -> Capability {
    Capability {
        id: generated_id("capability", &[check.check_id.as_str(), "close-check"]),
        actor: generated_id("actor", &["casegraphen-cli"]),
        operation: CapabilityOperation::Read,
        target_type: "native_close_check".to_owned(),
        target_refs: vec![ObjectRef::new(check.check_id.clone())],
        contexts: vec![case_space.case_space_id.clone()],
        preconditions: check
            .invariant_results
            .iter()
            .map(|result| result.invariant_id.clone())
            .collect(),
        postconditions: vec![check.check_id.clone()],
        forbidden_effects: vec![Description::new(
            "Read-only close-check must not append a morphism log entry.",
        )
        .expect("generated description is valid")],
        required_review: None,
        validity_interval: None,
        provenance: provenance.clone(),
        status: CapabilityStatus::Candidate,
    }
}

fn morphism_check_provenance(case_space: &CaseSpace, morphism: &CaseMorphism) -> Provenance {
    generated_provenance(
        source_uri(
            "native",
            &case_space.case_space_id,
            "morphism",
            &morphism.morphism_id,
        ),
        "CaseGraphen native morphism check",
        ReviewStatus::Reviewed,
        0.9,
    )
}

fn morphism_validation_witness(
    case_space: &CaseSpace,
    morphism: &CaseMorphism,
    provenance: &Provenance,
) -> Witness {
    let mut witness = Witness::candidate(
        generated_id("witness", &[morphism.morphism_id.as_str(), "check"]),
        WitnessType::MachineCheckResult,
        payload_ref(
            "casegraphen_native_morphism_check",
            source_uri(
                "native",
                &case_space.case_space_id,
                "morphism-check",
                &morphism.morphism_id,
            ),
        ),
        &case_space.revision.created_at,
        provenance.clone(),
        confidence(if morphism.violated_invariant_ids.is_empty() {
            0.91
        } else {
            0.72
        }),
    )
    .expect("generated witness is valid");
    witness.supports = vec![ObjectRef::new(morphism.morphism_id.clone())];
    witness.validity_contexts = vec![case_space.case_space_id.clone()];
    witness
}

fn morphism_scenario(
    case_space: &CaseSpace,
    morphism: &CaseMorphism,
    validation_witness_id: &Id,
    provenance: &Provenance,
) -> Scenario {
    Scenario {
        id: generated_id("scenario", &[morphism.morphism_id.as_str(), "target"]),
        base_space: case_space.case_space_id.clone(),
        scenario_kind: ScenarioKind::Planned,
        assumptions: morphism.source_ids.clone(),
        changed_structures: ScenarioChanges {
            added: morphism.added_ids.clone(),
            removed: morphism.retired_ids.clone(),
            modified: morphism.updated_ids.clone(),
        },
        reachable_from: Some(Reachability {
            reference: case_space.case_space_id.clone(),
            via_morphisms: vec![morphism.morphism_id.clone()],
        }),
        affected_invariants: morphism.violated_invariant_ids.clone(),
        expected_obstructions: morphism.violated_invariant_ids.clone(),
        required_witnesses: vec![validation_witness_id.clone()],
        valuations: vec![generated_id(
            "valuation",
            &[morphism.morphism_id.as_str(), "applicability"],
        )],
        status: if morphism.violated_invariant_ids.is_empty() {
            ScenarioStatus::Candidate
        } else {
            ScenarioStatus::Blocked
        },
        provenance: provenance.clone(),
        review_status: LifecycleStatus::Candidate,
    }
}

fn morphism_equivalence_claims(
    case_space: &CaseSpace,
    morphism: &CaseMorphism,
    validation_witness_id: &Id,
    provenance: &Provenance,
) -> Vec<EquivalenceClaim> {
    morphism
        .preserved_ids
        .iter()
        .map(|preserved_id| {
            let mut claim = EquivalenceClaim::candidate(
                generated_id(
                    "equivalence",
                    &[morphism.morphism_id.as_str(), preserved_id.as_str()],
                ),
                vec![
                    ObjectRef::new(preserved_id.clone()),
                    ObjectRef::new(preserved_id.clone()),
                ],
                EquivalenceKind::ObservationalEquivalence,
                confidence(0.86),
                provenance.clone(),
            );
            claim.scope = Some(EquivalenceScope {
                contexts: vec![case_space.case_space_id.clone()],
                valid_under_morphisms: vec![morphism.morphism_id.clone()],
            });
            claim.criterion = Some(EquivalenceCriterion::new(
                "Preserved identifiers keep their observable case role across the candidate morphism.",
            )
            .expect("generated equivalence criterion is valid"));
            claim.witnesses = vec![validation_witness_id.clone()];
            claim
        })
        .collect()
}

fn morphism_applicability_valuation(
    case_space: &CaseSpace,
    morphism: &CaseMorphism,
    validation_witness_id: &Id,
    provenance: Provenance,
) -> Valuation {
    Valuation {
        id: generated_id(
            "valuation",
            &[morphism.morphism_id.as_str(), "applicability"],
        ),
        target: ObjectRef::new(morphism.morphism_id.clone()),
        valuation_context: Some(case_space.case_space_id.clone()),
        criteria: morphism_valuation_criteria(),
        order_type: OrderType::ThresholdAcceptance,
        values: morphism_valuation_values(morphism, validation_witness_id),
        tradeoffs: Vec::new(),
        incomparable_with: Vec::new(),
        confidence: confidence(0.88),
        provenance,
        review_status: LifecycleStatus::Candidate,
    }
}

fn morphism_valuation_criteria() -> Vec<ValuationCriterion> {
    vec![
        ValuationCriterion {
            criterion_id: "applicable".to_owned(),
            name: "Candidate morphism is applicable".to_owned(),
            direction: CriterionDirection::Maximize,
            weight: Some(1.0),
            required: true,
        },
        ValuationCriterion {
            criterion_id: "violated_invariants".to_owned(),
            name: "Violated invariants".to_owned(),
            direction: CriterionDirection::Avoid,
            weight: Some(1.0),
            required: true,
        },
    ]
}

fn morphism_valuation_values(
    morphism: &CaseMorphism,
    validation_witness_id: &Id,
) -> Vec<CriterionValue> {
    vec![
        CriterionValue {
            criterion_id: "applicable".to_owned(),
            value: ValuationValue::Boolean(true),
            evidence: validation_witness_id.clone(),
        },
        CriterionValue {
            criterion_id: "violated_invariants".to_owned(),
            value: ValuationValue::Number(morphism.violated_invariant_ids.len() as f64),
            evidence: validation_witness_id.clone(),
        },
    ]
}

fn schema_morphism_for_native(
    case_space: &CaseSpace,
    morphism: &CaseMorphism,
    validation_witness_id: &Id,
    provenance: Provenance,
) -> SchemaMorphism {
    SchemaMorphism {
        id: generated_id("schema_morphism", &[morphism.morphism_id.as_str()]),
        source_schema: generated_id("schema", &["highergraphen.case.space.v1"]),
        target_schema: generated_id("schema", &["highergraphen.case.space.v1"]),
        source_interpretation_package: generated_id("interpretation", &["casegraphen-native-v1"]),
        target_interpretation_package: generated_id("interpretation", &["casegraphen-native-v1"]),
        mapping_kind: schema_mapping_kind(morphism),
        mappings: vec![native_schema_mapping(case_space, morphism)],
        affected_objects: native_schema_morphism_affected_objects(morphism),
        compatibility: schema_compatibility(morphism),
        verification: SchemaVerification {
            checks: vec![morphism.morphism_id.clone()],
            witnesses: vec![validation_witness_id.clone()],
        },
        provenance,
        review_status: LifecycleStatus::Candidate,
    }
}

fn native_schema_mapping(case_space: &CaseSpace, morphism: &CaseMorphism) -> SchemaMapping {
    SchemaMapping {
        source_ref: morphism
            .source_revision_id
            .as_ref()
            .unwrap_or(&case_space.revision.revision_id)
            .to_string(),
        target_ref: morphism.target_revision_id.to_string(),
        mapping_rule: "Validate candidate case morphism against the replayed native case space."
            .to_owned(),
        preservation_claims: morphism.preserved_ids.clone(),
        loss_claims: morphism
            .violated_invariant_ids
            .iter()
            .map(|invariant_id| {
                Description::new(format!(
                    "Candidate morphism reports violated invariant {invariant_id}."
                ))
                .expect("generated loss description is valid")
            })
            .collect(),
        required_reviews: Vec::new(),
    }
}

fn native_schema_morphism_affected_objects(morphism: &CaseMorphism) -> Vec<ObjectRef> {
    morphism
        .added_ids
        .iter()
        .chain(&morphism.updated_ids)
        .chain(&morphism.retired_ids)
        .chain(&morphism.preserved_ids)
        .cloned()
        .map(ObjectRef::new)
        .collect()
}

fn schema_mapping_kind(morphism: &CaseMorphism) -> SchemaMappingKind {
    match morphism.morphism_type {
        CaseMorphismType::Migration => SchemaMappingKind::Refinement,
        CaseMorphismType::Projection => SchemaMappingKind::Abstraction,
        CaseMorphismType::Retire => SchemaMappingKind::Deprecation,
        _ => SchemaMappingKind::Custom,
    }
}

fn schema_compatibility(morphism: &CaseMorphism) -> SchemaCompatibility {
    if !morphism.violated_invariant_ids.is_empty() {
        SchemaCompatibility::Lossy
    } else if matches!(morphism.morphism_type, CaseMorphismType::Migration) {
        SchemaCompatibility::BackwardCompatible
    } else {
        SchemaCompatibility::Unknown
    }
}
