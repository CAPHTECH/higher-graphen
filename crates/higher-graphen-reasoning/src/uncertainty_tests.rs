use super::{
    score_information_gain, score_multi_claim_information_gain, EvidenceLikelihoodModel,
    InformationGainOptions, InformationGainReport, MultiClaimInformationGainReport,
    ObservationAction, UncertaintyMeasure, UncertaintyObstructionType, UncertaintyState,
};
use higher_graphen_core::{Confidence, Id};
use serde::{Deserialize, Serialize};

fn assert_serde_contract<T>()
where
    T: Serialize + for<'de> Deserialize<'de>,
{
}

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("valid confidence")
}

#[test]
fn value_of_information_recommends_best_net_reducing_action() {
    let state = UncertaintyState::new(
        id("claim/api"),
        confidence(0.4),
        confidence(0.5),
        UncertaintyMeasure::BinaryEntropy,
    );
    let cheap_log_check =
        ObservationAction::new(id("observe/logs"), [id("claim/api")], "logs", 0.05)
            .expect("valid action")
            .with_expected_posterior_confidence(confidence(0.85));
    let expensive_audit =
        ObservationAction::new(id("observe/audit"), [id("claim/api")], "audit", 0.4)
            .expect("valid action")
            .with_expected_posterior_confidence(confidence(0.95));

    let report = score_information_gain(
        &state,
        &[cheap_log_check, expensive_audit],
        &InformationGainOptions::new(),
    )
    .expect("score actions");

    assert_eq!(report.recommended_action_ids[0], id("observe/logs"));
    assert_eq!(report.candidate_actions[0].action_id, id("observe/logs"));
    assert!(report.candidate_actions[0].net_value > 0.0);
    assert_eq!(report.obstructions.len(), 0);

    let json = serde_json::to_string(&report).expect("serialize");
    let roundtrip: InformationGainReport = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(roundtrip, report);
}

#[test]
fn blocked_missing_and_over_budget_actions_are_obstructed() {
    let state = UncertaintyState::new(
        id("claim/risk"),
        confidence(0.6),
        confidence(0.52),
        UncertaintyMeasure::DecisionThresholdDistance,
    );
    let blocked = ObservationAction::new(
        id("observe/customer-data"),
        [id("claim/risk")],
        "customer data",
        0.2,
    )
    .expect("valid action")
    .with_expected_posterior_confidence(confidence(0.9))
    .with_blocked_by_policy_ids([id("policy/privacy")]);
    let missing_model = ObservationAction::new(
        id("observe/interview"),
        [id("claim/risk")],
        "interview",
        0.1,
    )
    .expect("valid action");
    let over_budget = ObservationAction::new(id("observe/audit"), [id("claim/risk")], "audit", 0.9)
        .expect("valid action")
        .with_expected_posterior_confidence(confidence(0.95));
    let options = InformationGainOptions::new()
        .with_cost_budget(0.5)
        .expect("budget")
        .with_cost_normalizer(1.0)
        .expect("normalizer");

    let report = score_information_gain(&state, &[blocked, missing_model, over_budget], &options)
        .expect("score actions");

    assert!(report.recommended_action_ids.is_empty());
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == UncertaintyObstructionType::ObservationBlockedByPolicy
    }));
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == UncertaintyObstructionType::MissingLikelihoodModel
    }));
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == UncertaintyObstructionType::CostExceedsBudget
    }));
}

#[test]
fn multi_claim_scoring_charges_shared_action_cost_once() {
    let states = vec![
        UncertaintyState::new(
            id("claim/a"),
            confidence(0.4),
            confidence(0.5),
            UncertaintyMeasure::BinaryEntropy,
        ),
        UncertaintyState::new(
            id("claim/b"),
            confidence(0.4),
            confidence(0.5),
            UncertaintyMeasure::BinaryEntropy,
        ),
    ];
    let shared = ObservationAction::new(
        id("observe/shared"),
        [id("claim/a"), id("claim/b")],
        "shared logs",
        0.1,
    )
    .expect("valid action")
    .with_expected_posterior_confidence(confidence(0.9));

    let report =
        score_multi_claim_information_gain(&states, &[shared], &InformationGainOptions::new())
            .expect("score multiple claims");

    assert_eq!(report.recommended_action_ids, vec![id("observe/shared")]);
    assert_eq!(
        report.aggregate_action_scores[0].claim_ids,
        vec![id("claim/a"), id("claim/b")]
    );
    assert_eq!(report.claim_reports.len(), 2);
    assert!(report.aggregate_action_scores[0].net_value > 0.0);

    let roundtrip: MultiClaimInformationGainReport =
        serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
            .expect("deserialize");
    assert_eq!(
        roundtrip.recommended_action_ids,
        report.recommended_action_ids
    );
    assert_eq!(
        roundtrip.aggregate_action_scores[0].action_id,
        id("observe/shared")
    );
}

#[test]
fn constructors_reject_malformed_costs_and_text() {
    assert!(ObservationAction::new(id("observe/bad"), [id("claim")], " ", 0.1).is_err());
    assert!(ObservationAction::new(id("observe/bad"), [id("claim")], "logs", -0.1).is_err());
    assert!(InformationGainOptions::new()
        .with_cost_normalizer(0.0)
        .is_err());
    assert!(InformationGainOptions::new()
        .with_cost_budget(f64::NAN)
        .is_err());
}

#[test]
fn likelihood_model_computes_posterior_confidence() {
    let model = EvidenceLikelihoodModel::new(confidence(0.8), confidence(0.2));
    let posterior = model.posterior(confidence(0.5)).expect("posterior");

    assert!((posterior.value() - 0.8).abs() < 0.000_000_1);
    assert!(
        super::posterior_from_likelihood(confidence(0.5), confidence(0.0), confidence(0.0))
            .is_err()
    );
}

#[test]
fn public_types_implement_serde_contracts() {
    assert_serde_contract::<UncertaintyMeasure>();
    assert_serde_contract::<super::InformationGainCalculationKind>();
    assert_serde_contract::<UncertaintyObstructionType>();
    assert_serde_contract::<super::UncertaintyObstruction>();
    assert_serde_contract::<UncertaintyState>();
    assert_serde_contract::<EvidenceLikelihoodModel>();
    assert_serde_contract::<ObservationAction>();
    assert_serde_contract::<InformationGainOptions>();
    assert_serde_contract::<super::ScoredObservationAction>();
    assert_serde_contract::<InformationGainReport>();
    assert_serde_contract::<super::MultiClaimObservationScore>();
    assert_serde_contract::<MultiClaimInformationGainReport>();
}
