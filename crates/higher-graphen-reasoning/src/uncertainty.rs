//! Deterministic uncertainty and value-of-information scoring.
//!
//! This kernel recommends which observation action should be reviewed next. It
//! never accepts a claim and never executes the observation.

use higher_graphen_core::{Confidence, CoreError, Id, Provenance, Result, ReviewStatus};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// Supported finite uncertainty measures for a binary claim confidence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyMeasure {
    /// Normalized binary entropy, where 0.5 is maximally uncertain.
    BinaryEntropy,
    /// Distance from a decision threshold, where the threshold is maximally uncertain.
    DecisionThresholdDistance,
}

/// Stable calculation strategy used by an information-gain report.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InformationGainCalculationKind {
    /// Current uncertainty minus expected posterior uncertainty, then cost adjusted.
    CostAdjustedExpectedReduction,
}

/// Structured reason a value-of-information calculation could not recommend an action.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyObstructionType {
    /// A prior or posterior confidence was unavailable or invalid.
    InsufficientPrior,
    /// An action did not declare an expected posterior confidence.
    MissingLikelihoodModel,
    /// A policy blocks the observation action.
    ObservationBlockedByPolicy,
    /// The action cost exceeds the configured budget.
    CostExceedsBudget,
    /// The action is expected not to reduce uncertainty.
    NonReducingObservation,
}

/// Machine-readable obstruction for uncertainty scoring.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UncertaintyObstruction {
    /// Obstruction category.
    pub obstruction_type: UncertaintyObstructionType,
    /// Action affected by the obstruction, when action-specific.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<Id>,
    /// Human-readable diagnostic.
    pub reason: String,
}

impl UncertaintyObstruction {
    fn new(
        obstruction_type: UncertaintyObstructionType,
        action_id: Option<Id>,
        reason: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            obstruction_type,
            action_id,
            reason: required_text("uncertainty_obstruction.reason", reason)?,
        })
    }
}

/// Reviewable uncertainty state for one claim.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UncertaintyState {
    /// Claim being scored.
    pub claim_id: Id,
    /// Prior confidence before current evidence.
    pub prior_confidence: Confidence,
    /// Current confidence after known evidence.
    pub posterior_confidence: Confidence,
    /// Measure used to compute uncertainty.
    pub uncertainty_measure: UncertaintyMeasure,
    /// Evidence supporting the claim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_evidence_ids: Vec<Id>,
    /// Evidence contradicting the claim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contradicting_evidence_ids: Vec<Id>,
    /// Open obstructions affecting claim confidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_obstruction_ids: Vec<Id>,
    /// Human or workflow review status for this uncertainty record.
    #[serde(default)]
    pub review_status: ReviewStatus,
    /// Provenance for the uncertainty record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl UncertaintyState {
    /// Creates an uncertainty state with explicit prior, posterior, and measure.
    #[must_use]
    pub fn new(
        claim_id: Id,
        prior_confidence: Confidence,
        posterior_confidence: Confidence,
        uncertainty_measure: UncertaintyMeasure,
    ) -> Self {
        Self {
            claim_id,
            prior_confidence,
            posterior_confidence,
            uncertainty_measure,
            supporting_evidence_ids: Vec::new(),
            contradicting_evidence_ids: Vec::new(),
            unresolved_obstruction_ids: Vec::new(),
            review_status: ReviewStatus::Unreviewed,
            provenance: None,
        }
    }

    /// Returns this state with supporting evidence identifiers.
    #[must_use]
    pub fn with_supporting_evidence_ids(mut self, ids: impl IntoIterator<Item = Id>) -> Self {
        self.supporting_evidence_ids = unique_ids(ids);
        self
    }

    /// Returns this state with contradicting evidence identifiers.
    #[must_use]
    pub fn with_contradicting_evidence_ids(mut self, ids: impl IntoIterator<Item = Id>) -> Self {
        self.contradicting_evidence_ids = unique_ids(ids);
        self
    }

    /// Returns this state with unresolved obstruction identifiers.
    #[must_use]
    pub fn with_unresolved_obstruction_ids(mut self, ids: impl IntoIterator<Item = Id>) -> Self {
        self.unresolved_obstruction_ids = unique_ids(ids);
        self
    }

    /// Returns this state with review status.
    #[must_use]
    pub fn with_review_status(mut self, review_status: ReviewStatus) -> Self {
        self.review_status = review_status;
        self
    }

    /// Returns this state with provenance.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

/// Candidate observation action for one or more claims.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationAction {
    /// Stable action identifier.
    pub id: Id,
    /// Claims this action is expected to inform.
    pub target_claim_ids: Vec<Id>,
    /// Expected evidence kind, using product-neutral text.
    pub expected_evidence_kind: String,
    /// Non-negative normalized or domain-local cost estimate.
    pub estimated_cost: f64,
    /// Expected posterior confidence after the observation, if modeled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_posterior_confidence: Option<Confidence>,
    /// Policy identifiers that block this action.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by_policy_ids: Vec<Id>,
    /// Human or workflow review status for this action.
    #[serde(default)]
    pub review_status: ReviewStatus,
    /// Provenance for this proposed action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl ObservationAction {
    /// Creates an observation action with no expected posterior model yet.
    pub fn new(
        id: Id,
        target_claim_ids: impl IntoIterator<Item = Id>,
        expected_evidence_kind: impl Into<String>,
        estimated_cost: f64,
    ) -> Result<Self> {
        let action = Self {
            id,
            target_claim_ids: unique_ids(target_claim_ids),
            expected_evidence_kind: required_text(
                "observation_action.expected_evidence_kind",
                expected_evidence_kind,
            )?,
            estimated_cost,
            expected_posterior_confidence: None,
            blocked_by_policy_ids: Vec::new(),
            review_status: ReviewStatus::Unreviewed,
            provenance: None,
        };
        action.validate()?;
        Ok(action)
    }

    /// Returns this action with an expected posterior confidence model.
    #[must_use]
    pub fn with_expected_posterior_confidence(
        mut self,
        expected_posterior_confidence: Confidence,
    ) -> Self {
        self.expected_posterior_confidence = Some(expected_posterior_confidence);
        self
    }

    /// Returns this action with policy blockers.
    #[must_use]
    pub fn with_blocked_by_policy_ids(mut self, ids: impl IntoIterator<Item = Id>) -> Self {
        self.blocked_by_policy_ids = unique_ids(ids);
        self
    }

    /// Returns this action with review status.
    #[must_use]
    pub fn with_review_status(mut self, review_status: ReviewStatus) -> Self {
        self.review_status = review_status;
        self
    }

    /// Returns this action with provenance.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    fn validate(&self) -> Result<()> {
        if self.target_claim_ids.is_empty() {
            return Err(CoreError::MalformedField {
                field: "observation_action.target_claim_ids".to_owned(),
                reason: "expected at least one target claim".to_owned(),
            });
        }
        validate_non_negative_f64("observation_action.estimated_cost", self.estimated_cost)?;
        required_text(
            "observation_action.expected_evidence_kind",
            &self.expected_evidence_kind,
        )?;
        Ok(())
    }
}

/// Options for deterministic value-of-information scoring.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InformationGainOptions {
    /// Divisor used to normalize action cost before subtracting it from gain.
    pub cost_normalizer: f64,
    /// Optional maximum allowed raw action cost.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_budget: Option<f64>,
    /// Decision threshold used by threshold-distance uncertainty.
    pub decision_threshold: Confidence,
    /// Optional maximum number of recommended actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_recommended_actions: Option<usize>,
}

impl Default for InformationGainOptions {
    fn default() -> Self {
        Self {
            cost_normalizer: 1.0,
            cost_budget: None,
            decision_threshold: Confidence::new(0.5).expect("default threshold is valid"),
            max_recommended_actions: None,
        }
    }
}

impl InformationGainOptions {
    /// Creates default scoring options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns options with an explicit positive cost normalizer.
    pub fn with_cost_normalizer(mut self, cost_normalizer: f64) -> Result<Self> {
        validate_positive_f64("information_gain_options.cost_normalizer", cost_normalizer)?;
        self.cost_normalizer = cost_normalizer;
        Ok(self)
    }

    /// Returns options with a maximum raw action cost.
    pub fn with_cost_budget(mut self, cost_budget: f64) -> Result<Self> {
        validate_non_negative_f64("information_gain_options.cost_budget", cost_budget)?;
        self.cost_budget = Some(cost_budget);
        Ok(self)
    }

    /// Returns options with a decision threshold.
    #[must_use]
    pub fn with_decision_threshold(mut self, decision_threshold: Confidence) -> Self {
        self.decision_threshold = decision_threshold;
        self
    }

    /// Returns options with a recommendation limit.
    #[must_use]
    pub fn with_max_recommended_actions(mut self, max_recommended_actions: usize) -> Self {
        self.max_recommended_actions = Some(max_recommended_actions);
        self
    }

    fn validate(&self) -> Result<()> {
        validate_positive_f64(
            "information_gain_options.cost_normalizer",
            self.cost_normalizer,
        )?;
        if let Some(cost_budget) = self.cost_budget {
            validate_non_negative_f64("information_gain_options.cost_budget", cost_budget)?;
        }
        Ok(())
    }
}

/// Score for a single observation action.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScoredObservationAction {
    /// Action scored.
    pub action_id: Id,
    /// Claims this action targets.
    pub target_claim_ids: Vec<Id>,
    /// Uncertainty before the action.
    pub current_uncertainty: f64,
    /// Expected uncertainty after the action.
    pub expected_posterior_uncertainty: f64,
    /// Current uncertainty minus expected posterior uncertainty.
    pub expected_information_gain: f64,
    /// Action cost divided by the configured cost normalizer.
    pub normalized_observation_cost: f64,
    /// Expected gain minus normalized cost.
    pub net_value: f64,
    /// Policy blockers copied from the action.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by_policy_ids: Vec<Id>,
    /// Human or workflow review status copied from the action.
    #[serde(default)]
    pub review_status: ReviewStatus,
}

/// Deterministic value-of-information report for one claim.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InformationGainReport {
    /// Claim being scored.
    pub claim_id: Id,
    /// Measure used to compute uncertainty.
    pub uncertainty_measure: UncertaintyMeasure,
    /// Calculation strategy used by this report.
    pub calculation_kind: InformationGainCalculationKind,
    /// Current uncertainty under the selected measure.
    pub current_uncertainty: f64,
    /// Candidate actions with complete likelihood models.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidate_actions: Vec<ScoredObservationAction>,
    /// Recommended unblocked action identifiers, ordered by net value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recommended_action_ids: Vec<Id>,
    /// Obstructions preventing recommendation or scoring.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<UncertaintyObstruction>,
    /// Explicit calculation limitations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub information_loss: Vec<String>,
}

/// Aggregate score for an action across multiple claim reports.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MultiClaimObservationScore {
    /// Action scored.
    pub action_id: Id,
    /// Claims contributing positive or negative modeled value.
    pub claim_ids: Vec<Id>,
    /// Sum of per-claim expected information gain.
    pub total_expected_information_gain: f64,
    /// Maximum normalized observation cost charged once for the shared action.
    pub normalized_observation_cost: f64,
    /// Total gain minus shared normalized cost.
    pub net_value: f64,
}

/// Aggregate value-of-information report across multiple claims.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MultiClaimInformationGainReport {
    /// Per-claim reports.
    pub claim_reports: Vec<InformationGainReport>,
    /// Action scores aggregated across claims.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aggregate_action_scores: Vec<MultiClaimObservationScore>,
    /// Recommended action identifiers ordered by aggregate net value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recommended_action_ids: Vec<Id>,
}

/// Scores candidate observation actions for one claim.
pub fn score_information_gain(
    state: &UncertaintyState,
    actions: &[ObservationAction],
    options: &InformationGainOptions,
) -> Result<InformationGainReport> {
    options.validate()?;
    let current_uncertainty = uncertainty(
        state.posterior_confidence,
        state.uncertainty_measure,
        options.decision_threshold,
    );
    let mut candidate_actions = Vec::new();
    let mut obstructions = Vec::new();

    for action in actions {
        action.validate()?;
        if !action.target_claim_ids.contains(&state.claim_id) {
            continue;
        }
        if !action.blocked_by_policy_ids.is_empty() {
            obstructions.push(UncertaintyObstruction::new(
                UncertaintyObstructionType::ObservationBlockedByPolicy,
                Some(action.id.clone()),
                "observation action is blocked by policy",
            )?);
        }

        let Some(expected_posterior_confidence) = action.expected_posterior_confidence else {
            obstructions.push(UncertaintyObstruction::new(
                UncertaintyObstructionType::MissingLikelihoodModel,
                Some(action.id.clone()),
                "observation action does not declare an expected posterior confidence",
            )?);
            continue;
        };

        if options
            .cost_budget
            .is_some_and(|cost_budget| action.estimated_cost > cost_budget)
        {
            obstructions.push(UncertaintyObstruction::new(
                UncertaintyObstructionType::CostExceedsBudget,
                Some(action.id.clone()),
                "observation action exceeds the configured cost budget",
            )?);
        }

        let expected_posterior_uncertainty = uncertainty(
            expected_posterior_confidence,
            state.uncertainty_measure,
            options.decision_threshold,
        );
        let expected_information_gain = current_uncertainty - expected_posterior_uncertainty;
        if expected_information_gain <= 0.0 {
            obstructions.push(UncertaintyObstruction::new(
                UncertaintyObstructionType::NonReducingObservation,
                Some(action.id.clone()),
                "observation action is not expected to reduce uncertainty",
            )?);
        }
        let normalized_observation_cost = action.estimated_cost / options.cost_normalizer;
        candidate_actions.push(ScoredObservationAction {
            action_id: action.id.clone(),
            target_claim_ids: action.target_claim_ids.clone(),
            current_uncertainty,
            expected_posterior_uncertainty,
            expected_information_gain,
            normalized_observation_cost,
            net_value: expected_information_gain - normalized_observation_cost,
            blocked_by_policy_ids: action.blocked_by_policy_ids.clone(),
            review_status: action.review_status,
        });
    }

    candidate_actions.sort_by(compare_scored_actions);
    let mut recommended_action_ids = candidate_actions
        .iter()
        .filter(|action| {
            action.net_value > 0.0
                && action.blocked_by_policy_ids.is_empty()
                && options.cost_budget.is_none_or(|budget| {
                    action.normalized_observation_cost * options.cost_normalizer <= budget
                })
        })
        .map(|action| action.action_id.clone())
        .collect::<Vec<_>>();
    if let Some(max_recommended_actions) = options.max_recommended_actions {
        recommended_action_ids.truncate(max_recommended_actions);
    }

    Ok(InformationGainReport {
        claim_id: state.claim_id.clone(),
        uncertainty_measure: state.uncertainty_measure,
        calculation_kind: InformationGainCalculationKind::CostAdjustedExpectedReduction,
        current_uncertainty,
        candidate_actions,
        recommended_action_ids,
        obstructions,
        information_loss: vec![
            "expected posterior confidence is supplied by the caller, not inferred".to_owned(),
            "cost is normalized by a caller-provided scalar".to_owned(),
        ],
    })
}

/// Scores observation actions across multiple claims, charging shared action cost once.
pub fn score_multi_claim_information_gain(
    states: &[UncertaintyState],
    actions: &[ObservationAction],
    options: &InformationGainOptions,
) -> Result<MultiClaimInformationGainReport> {
    let claim_reports = states
        .iter()
        .map(|state| score_information_gain(state, actions, options))
        .collect::<Result<Vec<_>>>()?;
    let mut aggregate = BTreeMap::<Id, MultiClaimAccumulator>::new();

    for report in &claim_reports {
        for action in &report.candidate_actions {
            let entry = aggregate.entry(action.action_id.clone()).or_default();
            entry.claim_ids.insert(report.claim_id.clone());
            entry.total_expected_information_gain += action.expected_information_gain;
            entry.normalized_observation_cost = entry
                .normalized_observation_cost
                .max(action.normalized_observation_cost);
            entry.blocked |= !action.blocked_by_policy_ids.is_empty()
                || options.cost_budget.is_some_and(|budget| {
                    action.normalized_observation_cost * options.cost_normalizer > budget
                });
        }
    }

    let mut aggregate_action_scores = aggregate
        .into_iter()
        .map(|(action_id, accumulator)| MultiClaimObservationScore {
            action_id,
            claim_ids: accumulator.claim_ids.into_iter().collect(),
            total_expected_information_gain: accumulator.total_expected_information_gain,
            normalized_observation_cost: accumulator.normalized_observation_cost,
            net_value: if accumulator.blocked {
                f64::NEG_INFINITY
            } else {
                accumulator.total_expected_information_gain
                    - accumulator.normalized_observation_cost
            },
        })
        .collect::<Vec<_>>();
    aggregate_action_scores.sort_by(compare_multi_claim_scores);
    let mut recommended_action_ids = aggregate_action_scores
        .iter()
        .filter(|score| score.net_value > 0.0)
        .map(|score| score.action_id.clone())
        .collect::<Vec<_>>();
    if let Some(max_recommended_actions) = options.max_recommended_actions {
        recommended_action_ids.truncate(max_recommended_actions);
    }

    Ok(MultiClaimInformationGainReport {
        claim_reports,
        aggregate_action_scores,
        recommended_action_ids,
    })
}

#[derive(Debug, Default)]
struct MultiClaimAccumulator {
    claim_ids: BTreeSet<Id>,
    total_expected_information_gain: f64,
    normalized_observation_cost: f64,
    blocked: bool,
}

fn uncertainty(
    confidence: Confidence,
    measure: UncertaintyMeasure,
    decision_threshold: Confidence,
) -> f64 {
    match measure {
        UncertaintyMeasure::BinaryEntropy => binary_entropy(confidence.value()),
        UncertaintyMeasure::DecisionThresholdDistance => {
            threshold_distance_uncertainty(confidence.value(), decision_threshold.value())
        }
    }
}

fn binary_entropy(probability: f64) -> f64 {
    if probability <= 0.0 || probability >= 1.0 {
        0.0
    } else {
        let entropy =
            -probability * probability.ln() - (1.0 - probability) * (1.0 - probability).ln();
        entropy / std::f64::consts::LN_2
    }
}

fn threshold_distance_uncertainty(confidence: f64, threshold: f64) -> f64 {
    let maximum_distance = threshold.max(1.0 - threshold);
    if maximum_distance == 0.0 {
        return 0.0;
    }
    (1.0 - ((confidence - threshold).abs() / maximum_distance)).clamp(0.0, 1.0)
}

fn compare_scored_actions(
    left: &ScoredObservationAction,
    right: &ScoredObservationAction,
) -> Ordering {
    right
        .net_value
        .total_cmp(&left.net_value)
        .then_with(|| {
            right
                .expected_information_gain
                .total_cmp(&left.expected_information_gain)
        })
        .then_with(|| {
            left.normalized_observation_cost
                .total_cmp(&right.normalized_observation_cost)
        })
        .then_with(|| left.action_id.cmp(&right.action_id))
}

fn compare_multi_claim_scores(
    left: &MultiClaimObservationScore,
    right: &MultiClaimObservationScore,
) -> Ordering {
    right
        .net_value
        .total_cmp(&left.net_value)
        .then_with(|| {
            right
                .total_expected_information_gain
                .total_cmp(&left.total_expected_information_gain)
        })
        .then_with(|| {
            left.normalized_observation_cost
                .total_cmp(&right.normalized_observation_cost)
        })
        .then_with(|| left.action_id.cmp(&right.action_id))
}

fn required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let normalized = value.into().trim().to_owned();
    if normalized.is_empty() {
        Err(CoreError::MalformedField {
            field: field.to_owned(),
            reason: "value must not be empty after trimming".to_owned(),
        })
    } else {
        Ok(normalized)
    }
}

fn validate_non_negative_f64(field: &'static str, value: f64) -> Result<()> {
    if !value.is_finite() {
        return Err(CoreError::MalformedField {
            field: field.to_owned(),
            reason: "value must be finite".to_owned(),
        });
    }
    if value < 0.0 {
        return Err(CoreError::MalformedField {
            field: field.to_owned(),
            reason: "value must be non-negative".to_owned(),
        });
    }
    Ok(())
}

fn validate_positive_f64(field: &'static str, value: f64) -> Result<()> {
    validate_non_negative_f64(field, value)?;
    if value == 0.0 {
        return Err(CoreError::MalformedField {
            field: field.to_owned(),
            reason: "value must be positive".to_owned(),
        });
    }
    Ok(())
}

fn unique_ids(ids: impl IntoIterator<Item = Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        score_information_gain, score_multi_claim_information_gain, InformationGainOptions,
        InformationGainReport, MultiClaimInformationGainReport, ObservationAction,
        UncertaintyMeasure, UncertaintyObstructionType, UncertaintyState,
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
        let over_budget =
            ObservationAction::new(id("observe/audit"), [id("claim/risk")], "audit", 0.9)
                .expect("valid action")
                .with_expected_posterior_confidence(confidence(0.95));
        let options = InformationGainOptions::new()
            .with_cost_budget(0.5)
            .expect("budget")
            .with_cost_normalizer(1.0)
            .expect("normalizer");

        let report =
            score_information_gain(&state, &[blocked, missing_model, over_budget], &options)
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
    fn public_types_implement_serde_contracts() {
        assert_serde_contract::<UncertaintyMeasure>();
        assert_serde_contract::<super::InformationGainCalculationKind>();
        assert_serde_contract::<UncertaintyObstructionType>();
        assert_serde_contract::<super::UncertaintyObstruction>();
        assert_serde_contract::<UncertaintyState>();
        assert_serde_contract::<ObservationAction>();
        assert_serde_contract::<InformationGainOptions>();
        assert_serde_contract::<super::ScoredObservationAction>();
        assert_serde_contract::<InformationGainReport>();
        assert_serde_contract::<super::MultiClaimObservationScore>();
        assert_serde_contract::<MultiClaimInformationGainReport>();
    }
}
