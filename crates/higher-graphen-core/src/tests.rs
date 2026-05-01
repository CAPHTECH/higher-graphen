use crate::{
    Capability, CapabilityOperation, CapabilityStatus, Confidence, CoreError, CriterionDirection,
    CriterionValue, Derivation, DerivationFailureMode, Description, EquivalenceClaim,
    EquivalenceCriterion, EquivalenceKind, EquivalenceScope, Id, InferenceRule, LifecycleStatus,
    ObjectRef, OrderType, PayloadRef, Policy, PolicyApplicability, PolicyKind, PolicyRule,
    PolicyStatus, Provenance, QuotientEffect, Reachability, RequiredReview, ReviewRequirement,
    ReviewStatus, Scenario, ScenarioChanges, ScenarioKind, ScenarioStatus, SchemaCompatibility,
    SchemaMapping, SchemaMappingKind, SchemaMorphism, SchemaVerification, Severity, SourceKind,
    SourceRef, Tradeoff, Valuation, ValuationCriterion, ValuationValue, VerificationStatus,
    Verifier, VerifierKind, Witness, WitnessStatus, WitnessType,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn id_roundtrips_as_string_and_remains_orderable() {
    let id = Id::new("  structure-001  ").expect("valid id");

    assert_eq!(id.as_str(), "structure-001");
    assert_eq!(
        serde_json::to_string(&id).expect("serialize id"),
        "\"structure-001\""
    );

    let roundtrip: Id = serde_json::from_str("\"structure-001\"").expect("deserialize id");
    assert_eq!(roundtrip, id);
    assert!(Id::new("a").expect("valid id") < Id::new("b").expect("valid id"));
}

#[test]
fn id_can_be_used_as_a_downstream_json_map_key() {
    let mut keyed = BTreeMap::new();
    keyed.insert(Id::new("structure/a").expect("valid id"), 1_u8);

    let json = serde_json::to_string(&keyed).expect("serialize keyed map");
    assert_eq!(json, r#"{"structure/a":1}"#);

    let roundtrip: BTreeMap<Id, u8> = serde_json::from_str(&json).expect("deserialize keyed map");
    assert_eq!(roundtrip[&Id::new("structure/a").expect("valid id")], 1);
}

#[test]
fn validation_failures_return_structured_core_errors() {
    assert_eq!(Id::new("   ").expect_err("empty id").code(), "invalid_id");
    assert_eq!(
        Id::new("structure\n001")
            .expect_err("control character id")
            .code(),
        "invalid_id"
    );
    assert!(!Id::is_valid_value("structure\t001"));

    for value in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
        assert_eq!(
            Confidence::new(value)
                .expect_err("invalid confidence")
                .code(),
            "invalid_confidence"
        );
    }

    let error = serde_json::from_str::<Confidence>("1.01").expect_err("invalid confidence json");
    assert!(error.to_string().contains("invalid_confidence"));
}

#[test]
fn confidence_roundtrips_and_rejects_invalid_deserialized_values() {
    let confidence = Confidence::new(0.42).expect("valid confidence");

    let json = serde_json::to_string(&confidence).expect("serialize confidence");
    assert_eq!(json, "0.42");

    let roundtrip: Confidence = serde_json::from_str(&json).expect("deserialize confidence");
    assert!((roundtrip.value() - 0.42).abs() < f64::EPSILON);
    assert!(serde_json::from_str::<Confidence>("-0.1").is_err());

    let zero = Confidence::new(-0.0).expect("negative zero is in range");
    assert_eq!(zero.value().to_bits(), 0.0_f64.to_bits());
    assert_eq!(serde_json::to_string(&zero).expect("serialize zero"), "0.0");
    assert!(Confidence::ZERO.is_zero());
    assert!(Confidence::ONE.is_certain());
    assert!(Confidence::is_valid_value(1.0));
    assert!(!Confidence::is_valid_value(f64::NAN));
}

#[test]
fn source_kind_serializes_canonical_values_and_custom_extensions() {
    let cases = [
        (SourceKind::Document, "\"document\""),
        (SourceKind::Log, "\"log\""),
        (SourceKind::Api, "\"api\""),
        (SourceKind::Human, "\"human\""),
        (SourceKind::Ai, "\"ai\""),
        (SourceKind::Code, "\"code\""),
        (SourceKind::External, "\"external\""),
    ];

    for (kind, expected_json) in cases {
        assert_eq!(
            serde_json::to_string(&kind).expect("serialize kind"),
            expected_json
        );
        assert_eq!(
            serde_json::from_str::<SourceKind>(expected_json).expect("deserialize kind"),
            kind
        );
    }

    let custom = SourceKind::custom("dataset").expect("custom kind");
    assert!(custom.is_custom());
    assert_eq!(
        serde_json::to_string(&custom).expect("serialize custom"),
        "\"custom:dataset\""
    );

    let direct_custom = SourceKind::Custom("  dataset  ".to_owned());
    assert_eq!(
        serde_json::to_string(&direct_custom).expect("serialize direct custom"),
        "\"custom:dataset\""
    );
}

#[test]
fn source_kind_rejects_unknown_values_with_core_code() {
    let custom_error = SourceKind::custom("   ").expect_err("empty custom kind");
    assert_eq!(custom_error.code(), "invalid_source_kind");

    let error = serde_json::from_str::<SourceKind>("\"repository\"").expect_err("unknown kind");
    assert!(error.to_string().contains("invalid_source_kind"));

    let invalid_direct_custom = SourceKind::Custom("   ".to_owned());
    let error =
        serde_json::to_string(&invalid_direct_custom).expect_err("invalid custom serialization");
    assert!(error.to_string().contains("invalid_source_kind"));
}

#[test]
fn source_ref_roundtrips_portable_fields() {
    let source = SourceRef::new(SourceKind::Document)
        .with_uri("  urn:higher-graphen:source:1  ")
        .expect("valid uri")
        .with_title("Abstract source")
        .expect("valid title")
        .with_captured_at("2026-04-25T00:00:00Z")
        .expect("valid captured_at")
        .with_source_local_id("section-1")
        .expect("valid source local id");

    let json = serde_json::to_string(&source).expect("serialize source");
    assert_eq!(source.uri.as_deref(), Some("urn:higher-graphen:source:1"));
    let roundtrip: SourceRef = serde_json::from_str(&json).expect("deserialize source");
    assert_eq!(roundtrip, source);
}

#[test]
fn source_ref_rejects_blank_payloads_at_serde_boundaries() {
    assert_eq!(
        SourceRef::new(SourceKind::Document)
            .with_uri("   ")
            .expect_err("empty source uri")
            .code(),
        "malformed_field"
    );

    let direct_invalid = SourceRef {
        kind: SourceKind::Document,
        uri: Some("   ".to_owned()),
        title: None,
        captured_at: None,
        source_local_id: None,
    };
    let error = serde_json::to_string(&direct_invalid).expect_err("invalid source serialization");
    assert!(error.to_string().contains("malformed_field"));

    let malformed = r#"{"kind":"document","title":"   "}"#;
    let error = serde_json::from_str::<SourceRef>(malformed).expect_err("invalid source input");
    assert!(error.to_string().contains("malformed_field"));
}

#[test]
fn severity_and_review_status_have_stable_values_and_order() {
    assert!(Severity::Low < Severity::Medium);
    assert!(Severity::Medium < Severity::High);
    assert!(Severity::High < Severity::Critical);
    assert!(Severity::Critical.is_at_least(Severity::High));
    assert_eq!(Severity::try_from("critical").unwrap(), Severity::Critical);
    assert_eq!(Severity::Critical.as_str(), "critical");
    assert_eq!(Severity::Critical.to_string(), "critical");
    assert_eq!(
        serde_json::to_string(&Severity::Critical).unwrap(),
        "\"critical\""
    );

    assert_eq!(ReviewStatus::default(), ReviewStatus::Unreviewed);
    assert_eq!(
        ReviewStatus::try_from("accepted").unwrap(),
        ReviewStatus::Accepted
    );
    assert_eq!(ReviewStatus::Accepted.as_str(), "accepted");
    assert_eq!(ReviewStatus::Accepted.to_string(), "accepted");
    assert!(ReviewStatus::Accepted.is_accepted());
    assert!(ReviewStatus::Rejected.is_rejected());
    assert!(ReviewStatus::Reviewed.has_review_action());
    assert!(serde_json::from_str::<Severity>("\"urgent\"").is_err());
    assert!(serde_json::from_str::<ReviewStatus>("\"approved\"").is_err());
    assert_eq!(
        Severity::try_from("urgent")
            .expect_err("invalid severity")
            .code(),
        "parse_failure"
    );
}

#[test]
fn provenance_roundtrips_and_requires_review_status_on_input() {
    let source = SourceRef::new(SourceKind::custom("fixture").expect("custom source"));
    let provenance = Provenance::new(source, Confidence::new(0.8).expect("confidence"))
        .with_review_status(ReviewStatus::Unreviewed)
        .with_extraction_method("  manual_fixture  ")
        .expect("valid extraction method")
        .with_extractor_id("extractor-1")
        .expect("valid extractor id")
        .with_notes("keeps review status separate from confidence")
        .expect("valid notes");

    let value = serde_json::to_value(&provenance).expect("serialize provenance");
    assert_eq!(value["review_status"], json!("unreviewed"));
    assert_eq!(value["extraction_method"], json!("manual_fixture"));

    let roundtrip: Provenance = serde_json::from_value(value).expect("deserialize provenance");
    assert_eq!(roundtrip, provenance);

    let malformed = r#"{"source":{"kind":"document"},"confidence":0.8}"#;
    assert!(serde_json::from_str::<Provenance>(malformed).is_err());
}

#[test]
fn provenance_rejects_blank_optional_payloads_at_serde_boundaries() {
    assert_eq!(
        Provenance::new(
            SourceRef::new(SourceKind::Document),
            Confidence::new(0.8).expect("confidence"),
        )
        .with_reviewer_id("   ")
        .expect_err("empty reviewer id")
        .code(),
        "malformed_field"
    );

    let mut direct_invalid = Provenance::new(
        SourceRef::new(SourceKind::Document),
        Confidence::new(0.8).expect("confidence"),
    )
    .with_review_status(ReviewStatus::Reviewed);
    direct_invalid.reviewed_at = Some("   ".to_owned());
    let error =
        serde_json::to_string(&direct_invalid).expect_err("invalid provenance serialization");
    assert!(error.to_string().contains("malformed_field"));

    let malformed = r#"{"source":{"kind":"document"},"confidence":0.8,"review_status":"reviewed","notes":"   "}"#;
    let error =
        serde_json::from_str::<Provenance>(malformed).expect_err("invalid provenance input");
    assert!(error.to_string().contains("malformed_field"));
}

#[test]
fn core_error_exposes_stable_codes_and_roundtrips() {
    let error = Id::new("").expect_err("invalid id");
    assert_eq!(error.code(), "invalid_id");

    let value = serde_json::to_value(&error).expect("serialize error");
    assert_eq!(value["code"], json!("invalid_id"));

    let roundtrip: CoreError = serde_json::from_value(value).expect("deserialize error");
    assert_eq!(roundtrip, error);
}

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("valid confidence")
}

fn provenance(kind: SourceKind, review_status: ReviewStatus) -> Provenance {
    Provenance::new(SourceRef::new(kind), confidence(0.9)).with_review_status(review_status)
}

#[test]
fn equivalence_claim_roundtrips_and_enforces_acceptance_boundaries() {
    let mut claim = EquivalenceClaim::candidate(
        id("equivalence_claim:sales_customer_billing_customer"),
        vec![
            ObjectRef::new(id("cell:sales_customer")),
            ObjectRef::new(id("cell:billing_customer")),
        ],
        EquivalenceKind::ContextualEquivalence,
        confidence(0.72),
        provenance(SourceKind::Human, ReviewStatus::Accepted),
    );
    assert_eq!(
        claim
            .validate_acceptance()
            .expect_err("missing acceptance requirements")
            .code(),
        "malformed_field"
    );

    claim.scope = Some(EquivalenceScope {
        contexts: vec![id("context:sales"), id("context:billing")],
        valid_under_morphisms: vec![id("morphism:customer_identity_projection")],
    });
    claim.criterion = Some(
        EquivalenceCriterion::new("same legal counterparty in this projection")
            .expect("valid criterion"),
    );
    claim.witnesses = vec![id("witness:registry_match")];
    claim.quotient_effect = Some(QuotientEffect {
        lost_distinctions: vec![Description::new("billing role is collapsed").unwrap()],
        merged_cells: vec![id("cell:sales_customer"), id("cell:billing_customer")],
        affected_invariants: vec![id("invariant:billing_responsibility")],
        affected_projections: vec![id("projection:identity_risk")],
        unresolved_obstructions: Vec::new(),
    });
    claim.review = Some(
        ReviewRequirement::new(true)
            .with_decision_reason("reviewed with billing owner")
            .expect("valid decision reason"),
    );
    claim.status = LifecycleStatus::Accepted;

    claim
        .validate_acceptance()
        .expect("accepted claim is valid");
    assert!(claim.can_merge_equivalence());

    let value = serde_json::to_value(&claim).expect("serialize equivalence claim");
    assert_eq!(value["equivalence_kind"], json!("contextual_equivalence"));
    let roundtrip: EquivalenceClaim =
        serde_json::from_value(value).expect("deserialize equivalence claim");
    assert_eq!(roundtrip, claim);

    let mut strict = claim;
    strict.equivalence_kind = EquivalenceKind::StrictIdentity;
    strict.counter_witnesses = vec![id("witness:distinct_billing_role")];
    assert_eq!(
        strict
            .validate_acceptance()
            .expect_err("strict identity is blocked by counter witness")
            .code(),
        "malformed_field"
    );
}

#[test]
fn ai_generated_equivalence_requires_explicit_accepted_review() {
    let mut claim = EquivalenceClaim::candidate(
        id("equivalence_claim:ai_candidate"),
        vec![ObjectRef::new(id("cell:a")), ObjectRef::new(id("cell:b"))],
        EquivalenceKind::ObservationalEquivalence,
        confidence(0.5),
        provenance(SourceKind::Ai, ReviewStatus::Unreviewed),
    );
    claim.scope = Some(EquivalenceScope {
        contexts: vec![id("context:local")],
        valid_under_morphisms: Vec::new(),
    });
    claim.criterion = Some(EquivalenceCriterion::new("same observed label").unwrap());
    claim.witnesses = vec![id("witness:label")];
    claim.quotient_effect = Some(QuotientEffect {
        lost_distinctions: Vec::new(),
        merged_cells: vec![id("cell:a"), id("cell:b")],
        affected_invariants: Vec::new(),
        affected_projections: Vec::new(),
        unresolved_obstructions: Vec::new(),
    });
    claim.review = Some(ReviewRequirement::new(false));

    let error = claim
        .validate_acceptance()
        .expect_err("AI-generated equivalence must not auto-accept");
    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn derivation_acceptance_distinguishes_evidence_from_verified_inference() {
    let mut rule =
        InferenceRule::new(id("inference_rule:domain_rule"), "Domain rule").expect("valid rule");
    rule.rule_scope_contexts = vec![id("context:billing")];

    let mut derivation = Derivation::candidate(
        id("derivation:billing_obligation"),
        id("cell:billing_obligation"),
        vec![id("cell:contract"), id("cell:invoice")],
        rule,
        provenance(SourceKind::Ai, ReviewStatus::Unreviewed),
    );
    derivation.warrants = vec![id("witness:contract_clause")];
    derivation.verifier = Some(
        Verifier::new(VerifierKind::ProofChecker, "proof-checker://billing/rule")
            .expect("valid verifier"),
    );
    derivation.verification_status = VerificationStatus::MachineChecked;
    derivation.review_status = LifecycleStatus::Accepted;

    derivation
        .validate_acceptance()
        .expect("machine-checked derivation is valid");

    let mut circular = derivation.clone();
    circular.premises = vec![circular.conclusion.clone()];
    assert_eq!(
        circular
            .validate_acceptance()
            .expect_err("circular derivation is rejected")
            .code(),
        "malformed_field"
    );

    let mut unsupported = derivation;
    unsupported.failure_mode = DerivationFailureMode::UnsupportedJump;
    assert_eq!(
        unsupported
            .validate_acceptance()
            .expect_err("unsupported jump is rejected")
            .code(),
        "malformed_field"
    );
}

#[test]
fn witness_payload_context_and_status_are_required_for_accepted_support() {
    let mut witness = Witness::candidate(
        id("witness:test_result"),
        WitnessType::TestResult,
        PayloadRef::new("file", "urn:higher-graphen:test-result:1").unwrap(),
        "2026-05-01T00:00:00Z",
        provenance(SourceKind::Code, ReviewStatus::Accepted),
        confidence(1.0),
    )
    .expect("valid witness");

    assert_eq!(
        witness
            .validate_acceptance()
            .expect_err("context is required")
            .code(),
        "malformed_field"
    );

    witness.validity_contexts = vec![id("context:test")];
    witness.review_status = WitnessStatus::Accepted;
    witness
        .validate_acceptance()
        .expect("accepted witness with context is valid");

    witness.review_status = WitnessStatus::Rejected;
    assert_eq!(
        witness
            .validate_acceptance()
            .expect_err("rejected witness cannot support acceptance")
            .code(),
        "malformed_field"
    );
}

#[test]
fn scenario_acceptance_requires_invariant_and_policy_checks() {
    let mut scenario = Scenario {
        id: id("scenario:rollout"),
        base_space: id("space:current"),
        scenario_kind: ScenarioKind::AcceptedOperationalPlan,
        assumptions: vec![id("cell:feature_enabled")],
        changed_structures: ScenarioChanges {
            added: vec![id("cell:new_api")],
            removed: Vec::new(),
            modified: vec![id("morphism:route_change")],
        },
        reachable_from: Some(Reachability {
            reference: id("space:current"),
            via_morphisms: vec![id("morphism:route_change")],
        }),
        affected_invariants: Vec::new(),
        expected_obstructions: Vec::new(),
        required_witnesses: vec![id("witness:policy_approval")],
        valuations: vec![id("valuation:rollout")],
        status: ScenarioStatus::Accepted,
        provenance: provenance(SourceKind::Human, ReviewStatus::Accepted),
        review_status: LifecycleStatus::Accepted,
    };
    assert_eq!(
        scenario
            .validate_acceptance()
            .expect_err("accepted scenario requires invariant checks")
            .code(),
        "malformed_field"
    );
    scenario.affected_invariants = vec![id("invariant:route_safety")];
    scenario
        .validate_acceptance()
        .expect("reviewed operational plan is valid");
}

#[test]
fn capability_accept_requires_policy_review() {
    let mut capability = Capability {
        id: id("capability:agent_accept"),
        actor: id("cell:agent"),
        operation: CapabilityOperation::Accept,
        target_type: "equivalence_claim".to_owned(),
        target_refs: vec![ObjectRef::new(id("equivalence_claim:target"))],
        contexts: vec![id("context:review")],
        preconditions: Vec::new(),
        postconditions: Vec::new(),
        forbidden_effects: vec![Description::new("unreviewed promotion").unwrap()],
        required_review: None,
        validity_interval: None,
        provenance: provenance(SourceKind::Ai, ReviewStatus::Unreviewed),
        status: CapabilityStatus::Active,
    };
    assert_eq!(
        capability
            .validate_active_use()
            .expect_err("accept operation requires policy review")
            .code(),
        "malformed_field"
    );
    capability.required_review = Some(RequiredReview {
        policy: id("policy:candidate_acceptance"),
        reviewer: id("cell:human_reviewer"),
    });
    capability
        .validate_active_use()
        .expect("policy-reviewed accept capability is valid");
}

#[test]
fn active_policy_requires_accepted_review_and_provenance() {
    let mut policy = Policy {
        id: id("policy:candidate_acceptance"),
        policy_kind: PolicyKind::CandidateAcceptance,
        applies_to: PolicyApplicability {
            target_types: vec!["equivalence_claim".to_owned(), "scenario".to_owned()],
            contexts: vec![id("context:review")],
            operations: vec!["accept".to_owned()],
        },
        rule: PolicyRule::new("candidate acceptance requires human review").unwrap(),
        required_witnesses: vec![id("witness:review_record")],
        required_derivations: Vec::new(),
        escalation_path: vec![id("cell:owner")],
        violation_obstruction_template: Some(id("obstruction_template:policy_violation")),
        status: PolicyStatus::Active,
        provenance: provenance(SourceKind::Human, ReviewStatus::Unreviewed),
        review_status: ReviewStatus::Accepted,
    };
    assert_eq!(
        policy
            .validate_active()
            .expect_err("active policy provenance requires review")
            .code(),
        "malformed_field"
    );
    policy.provenance = provenance(SourceKind::Human, ReviewStatus::Accepted);
    policy.validate_active().expect("active policy is valid");
}

#[test]
fn valuation_and_schema_morphism_preserve_loss_and_incomparability() {
    let valuation = Valuation {
        id: id("valuation:decision"),
        target: ObjectRef::new(id("scenario:a")),
        valuation_context: Some(id("context:decision")),
        criteria: vec![ValuationCriterion {
            criterion_id: "risk".to_owned(),
            name: "Risk".to_owned(),
            direction: CriterionDirection::Minimize,
            weight: Some(1.0),
            required: true,
        }],
        order_type: OrderType::ScalarScore,
        values: vec![CriterionValue {
            criterion_id: "risk".to_owned(),
            value: ValuationValue::Number(0.2),
            evidence: id("witness:risk_assessment"),
        }],
        tradeoffs: vec![Tradeoff {
            gains: "lower lead time".to_owned(),
            losses: "higher rollout uncertainty".to_owned(),
            affected_invariants: vec![id("invariant:operability")],
        }],
        incomparable_with: vec![id("valuation:ethics_review")],
        confidence: confidence(0.8),
        provenance: provenance(SourceKind::Human, ReviewStatus::Accepted),
        review_status: LifecycleStatus::UnderReview,
    };
    assert_eq!(
        valuation
            .validate_for_decision()
            .expect_err("incomparable scalar ranking is rejected")
            .code(),
        "malformed_field"
    );

    let mut schema_morphism = SchemaMorphism {
        id: id("schema_morphism:v1_to_v2"),
        source_schema: id("schema:v1"),
        target_schema: id("schema:v2"),
        source_interpretation_package: id("interpretation_package:v1"),
        target_interpretation_package: id("interpretation_package:v2"),
        mapping_kind: SchemaMappingKind::Merge,
        mappings: vec![SchemaMapping {
            source_ref: "Customer".to_owned(),
            target_ref: "Party".to_owned(),
            mapping_rule: "merge customer-like actors".to_owned(),
            preservation_claims: vec![id("invariant:identity_trace")],
            loss_claims: Vec::new(),
            required_reviews: vec![id("policy:schema_review")],
        }],
        affected_objects: vec![ObjectRef::new(id("cell:customer"))],
        compatibility: SchemaCompatibility::Lossy,
        verification: SchemaVerification {
            checks: vec![id("derivation:compatibility")],
            witnesses: vec![id("witness:migration_fixture")],
        },
        provenance: provenance(SourceKind::Human, ReviewStatus::Accepted),
        review_status: LifecycleStatus::Accepted,
    };
    assert_eq!(
        schema_morphism
            .validate_application()
            .expect_err("lossy merge requires loss claims")
            .code(),
        "malformed_field"
    );
    schema_morphism.mappings[0].loss_claims =
        vec![Description::new("customer role granularity is collapsed").unwrap()];
    schema_morphism
        .validate_application()
        .expect("lossy merge with explicit loss is valid");
}
