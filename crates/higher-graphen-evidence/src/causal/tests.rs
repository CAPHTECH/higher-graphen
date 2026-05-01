use super::*;
use higher_graphen_core::{Confidence, ReviewStatus, SourceKind, SourceRef};
use serde::{Deserialize, Serialize};
use serde_json::json;

fn assert_serde_contract<T>()
where
    T: Serialize + for<'de> Deserialize<'de>,
{
}

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

fn provenance() -> Provenance {
    Provenance::new(
        SourceRef::new(SourceKind::custom("causal-test").expect("custom source kind")),
        Confidence::new(1.0).expect("valid confidence"),
    )
    .with_review_status(ReviewStatus::Accepted)
}

fn graph_with_variables(variable_ids: &[&str]) -> CausalGraph {
    variable_ids
        .iter()
        .fold(CausalGraph::new(), |graph, variable_id| {
            graph.with_variable(CausalVariable::new(id(variable_id), *variable_id))
        })
}

#[test]
fn observed_correlation_does_not_become_causality() {
    let graph = graph_with_variables(&["ice-cream", "drowning"]).with_observed_correlation(
        ObservedCorrelation::new(
            id("corr/summer"),
            id("ice-cream"),
            id("drowning"),
            AssociationDirection::Positive,
            provenance(),
        )
        .with_magnitude(0.7)
        .expect("finite magnitude"),
    );

    let assessment = graph
        .assess_pair(&id("ice-cream"), &id("drowning"))
        .expect("assessment");

    assert_eq!(
        assessment.status,
        CausalAssessmentStatus::ObservedCorrelationOnly
    );
    assert!(!assessment.supports_causality());
    assert_eq!(assessment.observed_correlation_ids, vec![id("corr/summer")]);
    assert_eq!(
        assessment.obstructions[0].kind,
        CausalObstructionKind::CorrelationOnly
    );
    assert_eq!(
        assessment.obstructions[0].obstruction_type,
        CORRELATION_ONLY_OBSTRUCTION
    );
}

#[test]
fn unsupported_causal_claim_is_explicit_even_with_correlation() {
    let graph = graph_with_variables(&["ice-cream", "drowning"])
        .with_observed_correlation(ObservedCorrelation::new(
            id("corr/summer"),
            id("ice-cream"),
            id("drowning"),
            AssociationDirection::Positive,
            provenance(),
        ))
        .with_causal_claim(CausalClaim::new(
            id("claim/ice-cream-drowning"),
            id("ice-cream"),
            id("drowning"),
            provenance(),
        ));

    let assessment = graph
        .assess_claim(&id("claim/ice-cream-drowning"))
        .expect("assessment");

    assert_eq!(
        assessment.status,
        CausalAssessmentStatus::UnsupportedCausalClaim
    );
    assert!(!assessment.supports_causality());
    assert_eq!(
        assessment.obstructions[0].obstruction_type,
        UNSUPPORTED_CAUSAL_CLAIM_OBSTRUCTION
    );
    assert_eq!(
        assessment.obstructions[0].related_correlation_ids,
        vec![id("corr/summer")]
    );
}

#[test]
fn intervention_supports_unconfounded_claim() {
    let graph = graph_with_variables(&["exercise", "blood-pressure"])
        .with_causal_claim(CausalClaim::new(
            id("claim/exercise-bp"),
            id("exercise"),
            id("blood-pressure"),
            provenance(),
        ))
        .with_intervention(
            Intervention::new(
                id("intervention/randomized-exercise"),
                id("exercise"),
                InterventionKind::RandomizedAssignment,
                provenance(),
            )
            .with_outcome(id("blood-pressure")),
        );

    let assessment = graph
        .assess_claim(&id("claim/exercise-bp"))
        .expect("assessment");

    assert_eq!(
        assessment.status,
        CausalAssessmentStatus::SupportedCausalClaim
    );
    assert!(assessment.supports_causality());
    assert_eq!(
        assessment.supporting_intervention_ids,
        vec![id("intervention/randomized-exercise")]
    );
    assert!(assessment.obstructions.is_empty());
}

#[test]
fn active_unadjusted_confounder_blocks_supported_claim() {
    let graph = graph_with_variables(&["exercise", "blood-pressure", "age"])
        .with_causal_claim(CausalClaim::new(
            id("claim/exercise-bp"),
            id("exercise"),
            id("blood-pressure"),
            provenance(),
        ))
        .with_intervention(
            Intervention::new(
                id("intervention/exercise"),
                id("exercise"),
                InterventionKind::SetValue,
                provenance(),
            )
            .with_outcome(id("blood-pressure")),
        )
        .with_confounder(Confounder::new(
            id("confounder/age"),
            id("age"),
            id("exercise"),
            id("blood-pressure"),
            ConfounderStatus::Plausible,
            provenance(),
        ));

    let assessment = graph
        .assess_claim(&id("claim/exercise-bp"))
        .expect("assessment");

    assert_eq!(assessment.status, CausalAssessmentStatus::Confounded);
    assert!(!assessment.supports_causality());
    assert_eq!(
        assessment.unadjusted_confounder_ids,
        vec![id("confounder/age")]
    );
    assert_eq!(
        assessment.obstructions[0].obstruction_type,
        CONFOUNDED_OBSTRUCTION
    );
}

#[test]
fn adjustment_set_clears_matching_confounder() {
    let graph = graph_with_variables(&["exercise", "blood-pressure", "age"])
        .with_causal_claim(CausalClaim::new(
            id("claim/exercise-bp"),
            id("exercise"),
            id("blood-pressure"),
            provenance(),
        ))
        .with_confounder(Confounder::new(
            id("confounder/age"),
            id("age"),
            id("exercise"),
            id("blood-pressure"),
            ConfounderStatus::Confirmed,
            provenance(),
        ))
        .with_adjustment_set(
            AdjustmentSet::new(
                id("adjustment/age"),
                id("claim/exercise-bp"),
                vec![id("age")],
                "adjust age before assessing the exercise effect",
                provenance(),
            )
            .expect("adjustment set"),
        );

    let assessment = graph
        .assess_claim(&id("claim/exercise-bp"))
        .expect("assessment");

    assert_eq!(
        assessment.status,
        CausalAssessmentStatus::SupportedCausalClaim
    );
    assert!(assessment.unadjusted_confounder_ids.is_empty());
    assert_eq!(assessment.adjustment_set_ids, vec![id("adjustment/age")]);
}

#[test]
fn opposite_polarity_claim_contradicts_causation() {
    let graph = graph_with_variables(&["exercise", "blood-pressure"])
        .with_causal_claim(CausalClaim::new(
            id("claim/causes"),
            id("exercise"),
            id("blood-pressure"),
            provenance(),
        ))
        .with_causal_claim(CausalClaim::non_causal(
            id("claim/does-not-cause"),
            id("exercise"),
            id("blood-pressure"),
            provenance(),
        ))
        .with_intervention(
            Intervention::new(
                id("intervention/exercise"),
                id("exercise"),
                InterventionKind::SetValue,
                provenance(),
            )
            .with_outcome(id("blood-pressure")),
        );

    let assessment = graph.assess_claim(&id("claim/causes")).expect("assessment");

    assert_eq!(assessment.status, CausalAssessmentStatus::Contradicted);
    assert_eq!(
        assessment.contradicting_claim_ids,
        vec![id("claim/does-not-cause")]
    );
    assert_eq!(
        assessment.obstructions[0].kind,
        CausalObstructionKind::ContradictedCausalClaim
    );
}

#[test]
fn disallowed_causal_cycle_is_structural_obstruction() {
    let graph = graph_with_variables(&["a", "b"])
        .with_causal_claim(CausalClaim::new(
            id("claim/a-b"),
            id("a"),
            id("b"),
            provenance(),
        ))
        .with_causal_claim(CausalClaim::new(
            id("claim/b-a"),
            id("b"),
            id("a"),
            provenance(),
        ))
        .with_intervention(
            Intervention::new(
                id("intervention/a"),
                id("a"),
                InterventionKind::DoOperator,
                provenance(),
            )
            .with_outcome(id("b")),
        );

    let assessment = graph.assess_claim(&id("claim/a-b")).expect("assessment");
    assert_eq!(assessment.status, CausalAssessmentStatus::Obstructed);
    assert!(assessment
        .obstructions
        .iter()
        .any(|obstruction| obstruction.kind == CausalObstructionKind::CausalCycle));

    let allowed = graph.with_feedback_cycles_allowed(true);
    let allowed_assessment = allowed.assess_claim(&id("claim/a-b")).expect("assessment");
    assert_eq!(
        allowed_assessment.status,
        CausalAssessmentStatus::SupportedCausalClaim
    );
}

#[test]
fn intervention_without_outcome_has_unsupported_conclusion() {
    let graph = graph_with_variables(&["exercise"]).with_intervention(Intervention::new(
        id("intervention/exercise"),
        id("exercise"),
        InterventionKind::SetValue,
        provenance(),
    ));

    let assessment = graph
        .assess_intervention(&id("intervention/exercise"))
        .expect("assessment");

    assert_eq!(
        assessment.status,
        InterventionAssessmentStatus::UnsupportedInterventionConclusion
    );
    assert_eq!(
        assessment.obstructions[0].obstruction_type,
        UNSUPPORTED_INTERVENTION_CONCLUSION_OBSTRUCTION
    );
}

#[test]
fn validation_rejects_malformed_records_but_reports_missing_references_as_obstructions() {
    let invalid_magnitude = ObservedCorrelation::new(
        id("corr/bad"),
        id("a"),
        id("b"),
        AssociationDirection::Unknown,
        provenance(),
    )
    .with_magnitude(f64::NAN);
    assert_eq!(
        invalid_magnitude
            .expect_err("nan magnitude should fail")
            .code(),
        "malformed_field"
    );

    let graph = CausalGraph::new().with_causal_claim(CausalClaim::new(
        id("claim/missing"),
        id("a"),
        id("b"),
        provenance(),
    ));
    let obstructions = graph
        .structural_obstructions()
        .expect("structural obstructions");
    assert!(obstructions
        .iter()
        .any(|obstruction| obstruction.kind == CausalObstructionKind::MissingVariable));
}

#[test]
fn assessment_serializes_stable_status_and_obstruction_type() {
    let graph =
        graph_with_variables(&["x", "y"]).with_observed_correlation(ObservedCorrelation::new(
            id("corr/x-y"),
            id("x"),
            id("y"),
            AssociationDirection::Positive,
            provenance(),
        ));
    let assessment = graph.assess_pair(&id("x"), &id("y")).expect("assessment");
    let value = serde_json::to_value(assessment).expect("serialize assessment");

    assert_eq!(value["status"], json!("observed_correlation_only"));
    assert_eq!(
        value["obstructions"][0]["obstruction_type"],
        json!(CORRELATION_ONLY_OBSTRUCTION)
    );
}

#[test]
fn public_types_implement_serde_contracts() {
    assert_serde_contract::<AssociationDirection>();
    assert_serde_contract::<CausalClaimPolarity>();
    assert_serde_contract::<InterventionKind>();
    assert_serde_contract::<ConfounderStatus>();
    assert_serde_contract::<CausalAssessmentStatus>();
    assert_serde_contract::<InterventionAssessmentStatus>();
    assert_serde_contract::<CausalObstructionKind>();
    assert_serde_contract::<CausalVariable>();
    assert_serde_contract::<ObservedCorrelation>();
    assert_serde_contract::<CausalClaim>();
    assert_serde_contract::<Intervention>();
    assert_serde_contract::<Confounder>();
    assert_serde_contract::<AdjustmentSet>();
    assert_serde_contract::<CausalObstruction>();
    assert_serde_contract::<CausalAssessment>();
    assert_serde_contract::<InterventionAssessment>();
    assert_serde_contract::<CausalGraph>();
}
