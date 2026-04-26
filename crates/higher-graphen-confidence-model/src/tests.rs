use super::*;

fn id(value: impl Into<String>) -> Id {
    Id::new(value).expect("valid id")
}

fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("valid confidence")
}

fn likelihood(given_claim: f64, given_not_claim: f64) -> EvidenceLikelihood {
    EvidenceLikelihood::new(confidence(given_claim), confidence(given_not_claim))
        .expect("valid likelihood")
}

fn evidence(id_suffix: &str, given_claim: f64, given_not_claim: f64) -> ConfidenceEvidence {
    ConfidenceEvidence::new(
        id(format!("evidence.{id_suffix}")),
        format!("Evidence {id_suffix}"),
        likelihood(given_claim, given_not_claim),
    )
    .expect("valid evidence")
}

fn assert_approx_eq(left: f64, right: f64) {
    assert!(
        (left - right).abs() < 1.0e-12,
        "expected {left} to approximately equal {right}"
    );
}

#[test]
fn supporting_evidence_increases_posterior() {
    let input = ConfidenceUpdateInput::new(id("claim.supported"), confidence(0.4))
        .with_supporting_evidence(vec![evidence("support", 0.8, 0.2)]);

    let record = update_confidence(input).expect("update succeeds");

    assert!(record.posterior.value() > record.prior.value());
    assert_approx_eq(record.posterior.value(), 8.0 / 11.0);
    assert_eq!(record.review_status, ReviewStatus::Unreviewed);
}

#[test]
fn contradicting_evidence_decreases_posterior() {
    let input = ConfidenceUpdateInput::new(id("claim.contradicted"), confidence(0.6))
        .with_contradicting_evidence(vec![evidence("contradict", 0.2, 0.8)]);

    let record = BayesianConfidenceEngine
        .update(input)
        .expect("update succeeds");

    assert!(record.posterior.value() < record.prior.value());
    assert_approx_eq(record.posterior.value(), 3.0 / 11.0);
}

#[test]
fn mixed_evidence_uses_supporting_and_contradicting_ratios() {
    let input = ConfidenceUpdateInput::new(id("claim.mixed"), confidence(0.5))
        .with_supporting_evidence(vec![evidence("support", 0.75, 0.25)])
        .with_contradicting_evidence(vec![evidence("contradict", 0.25, 0.5)]);

    let record = update_confidence(input).expect("update succeeds");

    assert_approx_eq(record.posterior.value(), 0.6);
}

#[test]
fn rejects_prior_endpoints_because_odds_would_be_infinite() {
    let input = ConfidenceUpdateInput::new(id("claim.endpoint"), Confidence::ZERO)
        .with_supporting_evidence(vec![evidence("support", 0.8, 0.2)]);

    let error = update_confidence(input).expect_err("zero prior is rejected");

    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn rejects_likelihood_endpoints_because_ratios_would_be_infinite() {
    let error = EvidenceLikelihood::new(Confidence::ONE, confidence(0.2))
        .expect_err("endpoint likelihood is rejected");

    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn rejects_evidence_in_the_wrong_direction() {
    let input = ConfidenceUpdateInput::new(id("claim.direction"), confidence(0.5))
        .with_supporting_evidence(vec![evidence("actually_contradicts", 0.2, 0.8)]);

    let error = update_confidence(input).expect_err("wrong direction is rejected");

    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn rejects_duplicate_evidence_ids_across_sets() {
    let duplicate_id = id("evidence.duplicate");
    let input = ConfidenceUpdateInput::new(id("claim.duplicates"), confidence(0.5))
        .with_supporting_evidence(vec![ConfidenceEvidence::new(
            duplicate_id.clone(),
            "Supporting evidence",
            likelihood(0.8, 0.2),
        )
        .expect("valid evidence")])
        .with_contradicting_evidence(vec![ConfidenceEvidence::new(
            duplicate_id,
            "Contradicting evidence",
            likelihood(0.2, 0.8),
        )
        .expect("valid evidence")]);

    let error = update_confidence(input).expect_err("duplicate evidence ids are rejected");

    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn serde_round_trip_preserves_validated_record() {
    let input = ConfidenceUpdateInput::new(id("claim.round_trip"), confidence(0.5))
        .with_supporting_evidence(vec![evidence("support", 0.7, 0.3).with_source_ids(vec![
            id("source.beta"),
            id("source.alpha"),
            id("source.alpha"),
        ])]);
    let record = update_confidence(input).expect("update succeeds");

    let json = serde_json::to_string(&record).expect("record serializes");
    let round_trip: ConfidenceUpdateRecord =
        serde_json::from_str(&json).expect("record deserializes");

    assert_eq!(round_trip, record);
    assert_eq!(
        round_trip.supporting_evidence[0].source_ids,
        vec![id("source.alpha"), id("source.beta")]
    );
}

#[test]
fn deserialization_rejects_tampered_posterior() {
    let input = ConfidenceUpdateInput::new(id("claim.tampered"), confidence(0.5))
        .with_supporting_evidence(vec![evidence("support", 0.8, 0.2)]);
    let record = update_confidence(input).expect("update succeeds");
    let mut json = serde_json::to_value(record).expect("record serializes");
    json["posterior"] = serde_json::json!(0.1);

    let error =
        serde_json::from_value::<ConfidenceUpdateRecord>(json).expect_err("posterior is checked");

    assert!(error.to_string().contains("posterior"));
}

#[test]
fn high_posterior_does_not_silently_accept_review() {
    let input = ConfidenceUpdateInput::new(id("claim.high"), confidence(0.95))
        .with_supporting_evidence(vec![evidence("strong_support", 0.99, 0.01)]);

    let record = update_confidence(input).expect("update succeeds");

    assert!(record.posterior.value() > 0.99);
    assert_eq!(record.review_status, ReviewStatus::Unreviewed);
    assert!(!record.is_review_accepted());

    let accepted = record.with_review_status(ReviewStatus::Accepted);
    assert!(accepted.is_review_accepted());
}
