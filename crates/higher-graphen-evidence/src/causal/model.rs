use super::*;

/// Direction of an observed association.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssociationDirection {
    /// Larger values of one variable tend to coincide with larger values of the other.
    Positive,
    /// Larger values of one variable tend to coincide with smaller values of the other.
    Negative,
    /// The observed association is not monotonic.
    NonMonotonic,
    /// The direction was not captured or is not known.
    Unknown,
}

/// Directional claim polarity for a variable pair.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalClaimPolarity {
    /// The source variable is claimed to cause the target variable.
    Causes,
    /// The source variable is claimed not to cause the target variable.
    DoesNotCause,
}

/// Kind of intervention recorded in a causal graph.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionKind {
    /// A do-operator style intervention fixes the target variable externally.
    DoOperator,
    /// Units were assigned to intervention conditions by randomization.
    RandomizedAssignment,
    /// The target variable was set to a specified condition.
    SetValue,
    /// The target variable was held fixed while outcomes were observed.
    HoldConstant,
    /// A natural experiment supplied plausibly external variation.
    NaturalExperiment,
}

/// Review state for a possible confounder.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfounderStatus {
    /// The variable may confound the claim and should block causal support.
    Suspected,
    /// The variable plausibly confounds the claim and should block causal support.
    Plausible,
    /// The variable is confirmed as a confounder and blocks causal support unless adjusted.
    Confirmed,
    /// The variable was considered and ruled out; it does not block the claim.
    RuledOut,
}

impl ConfounderStatus {
    /// Returns true when the confounder should block a claim unless adjusted.
    #[must_use]
    pub fn is_active(self) -> bool {
        !matches!(self, Self::RuledOut)
    }
}

/// Conservative assessment status for a causal query.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalAssessmentStatus {
    /// A causal claim has explicit causal support and no active unadjusted blocker.
    SupportedCausalClaim,
    /// Association evidence exists, but no causal claim is present.
    ObservedCorrelationOnly,
    /// A causal claim exists but has no explicit causal support.
    UnsupportedCausalClaim,
    /// Active confounders block support for the claim.
    Confounded,
    /// The claim conflicts with another recorded claim about the same directed pair.
    Contradicted,
    /// Structural graph checks block the claim.
    Obstructed,
    /// No association or causal evidence was found for the queried pair.
    NoEvidence,
}

/// Assessment status for an intervention record.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionAssessmentStatus {
    /// The intervention records at least one outcome variable.
    OutcomeObserved,
    /// The intervention is present but no outcome supports a causal conclusion.
    UnsupportedInterventionConclusion,
}

/// Machine-readable causal obstruction kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalObstructionKind {
    /// Correlation was present without a causal claim.
    CorrelationOnly,
    /// A causal claim exists without explicit support.
    UnsupportedCausalClaim,
    /// One or more active confounders are not covered by an adjustment set.
    Confounded,
    /// The graph references a variable that is not declared.
    MissingVariable,
    /// A causal claim points from a variable to itself.
    SelfCausation,
    /// A directed causal cycle is present while feedback cycles are disallowed.
    CausalCycle,
    /// A claim conflicts with an opposite-polarity claim for the same pair.
    ContradictedCausalClaim,
    /// An intervention record has no observed outcome variable.
    UnsupportedInterventionConclusion,
}

impl CausalObstructionKind {
    /// Returns a stable lower snake case string for compact diagnostics.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CorrelationOnly => CORRELATION_ONLY_OBSTRUCTION,
            Self::UnsupportedCausalClaim => UNSUPPORTED_CAUSAL_CLAIM_OBSTRUCTION,
            Self::Confounded => CONFOUNDED_OBSTRUCTION,
            Self::MissingVariable => "missing_variable",
            Self::SelfCausation => "self_causation",
            Self::CausalCycle => "causal_cycle",
            Self::ContradictedCausalClaim => "contradicted_causal_claim",
            Self::UnsupportedInterventionConclusion => {
                UNSUPPORTED_INTERVENTION_CONCLUSION_OBSTRUCTION
            }
        }
    }
}

/// Variable that may participate in association, intervention, or causal-claim records.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CausalVariable {
    /// Stable variable identifier.
    pub id: Id,
    /// Human-readable variable name.
    pub name: String,
    /// Optional explanation of the variable's meaning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional provenance for the variable definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl CausalVariable {
    /// Creates a variable definition.
    #[must_use]
    pub fn new(id: Id, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            provenance: None,
        }
    }

    /// Returns this variable with a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Result<Self> {
        self.description = Some(required_text("description", description)?);
        Ok(self)
    }

    /// Returns this variable with provenance.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

/// Observed association between two variables.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedCorrelation {
    /// Stable correlation identifier.
    pub id: Id,
    /// First associated variable.
    pub variable_a_id: Id,
    /// Second associated variable.
    pub variable_b_id: Id,
    /// Observed association direction.
    pub direction: AssociationDirection,
    /// Optional association magnitude, such as a correlation coefficient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magnitude: Option<f64>,
    /// Source and review metadata for the observation.
    pub provenance: Provenance,
}

impl ObservedCorrelation {
    /// Creates an observed correlation with no numeric magnitude.
    #[must_use]
    pub fn new(
        id: Id,
        variable_a_id: Id,
        variable_b_id: Id,
        direction: AssociationDirection,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            variable_a_id,
            variable_b_id,
            direction,
            magnitude: None,
            provenance,
        }
    }

    /// Returns this correlation with a finite magnitude.
    pub fn with_magnitude(mut self, magnitude: f64) -> Result<Self> {
        ensure_finite("magnitude", magnitude)?;
        self.magnitude = Some(magnitude);
        Ok(self)
    }

    pub(crate) fn matches_pair(&self, left: &Id, right: &Id) -> bool {
        (&self.variable_a_id == left && &self.variable_b_id == right)
            || (&self.variable_a_id == right && &self.variable_b_id == left)
    }
}

/// Directional claim about whether one variable causes another.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CausalClaim {
    /// Stable claim identifier.
    pub id: Id,
    /// Claimed cause variable.
    pub cause_id: Id,
    /// Claimed effect variable.
    pub effect_id: Id,
    /// Whether this is a positive or negative causal claim.
    pub polarity: CausalClaimPolarity,
    /// Source and review metadata for the claim.
    pub provenance: Provenance,
}

impl CausalClaim {
    /// Creates a positive causal claim.
    #[must_use]
    pub fn new(id: Id, cause_id: Id, effect_id: Id, provenance: Provenance) -> Self {
        Self {
            id,
            cause_id,
            effect_id,
            polarity: CausalClaimPolarity::Causes,
            provenance,
        }
    }

    /// Creates a claim that the source does not cause the target.
    #[must_use]
    pub fn non_causal(id: Id, cause_id: Id, effect_id: Id, provenance: Provenance) -> Self {
        Self {
            id,
            cause_id,
            effect_id,
            polarity: CausalClaimPolarity::DoesNotCause,
            provenance,
        }
    }
}

/// Intervention record over one target variable and zero or more observed outcomes.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Intervention {
    /// Stable intervention identifier.
    pub id: Id,
    /// Variable directly manipulated or fixed by the intervention.
    pub target_variable_id: Id,
    /// Intervention kind.
    pub kind: InterventionKind,
    /// Variables observed as outcomes after the intervention.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcome_variable_ids: Vec<Id>,
    /// Source and review metadata for the intervention record.
    pub provenance: Provenance,
}

impl Intervention {
    /// Creates an intervention record with no observed outcome yet.
    #[must_use]
    pub fn new(
        id: Id,
        target_variable_id: Id,
        kind: InterventionKind,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            target_variable_id,
            kind,
            outcome_variable_ids: Vec::new(),
            provenance,
        }
    }

    /// Returns this intervention with one observed outcome variable.
    #[must_use]
    pub fn with_outcome(mut self, outcome_variable_id: Id) -> Self {
        push_unique(&mut self.outcome_variable_ids, outcome_variable_id);
        self
    }

    pub(crate) fn supports_claim(&self, claim: &CausalClaim) -> bool {
        claim.polarity == CausalClaimPolarity::Causes
            && self.target_variable_id == claim.cause_id
            && self.outcome_variable_ids.contains(&claim.effect_id)
    }
}

/// Possible confounder for a directed causal claim.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Confounder {
    /// Stable confounder record identifier.
    pub id: Id,
    /// Variable that may influence both cause and effect.
    pub variable_id: Id,
    /// Cause side of the claim this confounder may affect.
    pub cause_id: Id,
    /// Effect side of the claim this confounder may affect.
    pub effect_id: Id,
    /// Review state for this possible confounder.
    pub status: ConfounderStatus,
    /// Source and review metadata for the confounder record.
    pub provenance: Provenance,
}

impl Confounder {
    /// Creates a possible confounder record for a directed pair.
    #[must_use]
    pub fn new(
        id: Id,
        variable_id: Id,
        cause_id: Id,
        effect_id: Id,
        status: ConfounderStatus,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            variable_id,
            cause_id,
            effect_id,
            status,
            provenance,
        }
    }

    pub(crate) fn blocks_pair(&self, cause_id: &Id, effect_id: &Id) -> bool {
        self.status.is_active() && &self.cause_id == cause_id && &self.effect_id == effect_id
    }
}

/// Variables selected for adjustment before assessing a causal claim.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdjustmentSet {
    /// Stable adjustment-set identifier.
    pub id: Id,
    /// Claim this adjustment set is intended to support.
    pub claim_id: Id,
    /// Variables included in the adjustment set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variable_ids: Vec<Id>,
    /// Human-readable reason for selecting the adjustment variables.
    pub rationale: String,
    /// Source and review metadata for the adjustment set.
    pub provenance: Provenance,
}

impl AdjustmentSet {
    /// Creates an adjustment set for a claim.
    pub fn new(
        id: Id,
        claim_id: Id,
        variable_ids: Vec<Id>,
        rationale: impl Into<String>,
        provenance: Provenance,
    ) -> Result<Self> {
        Ok(Self {
            id,
            claim_id,
            variable_ids: unique_ids(variable_ids),
            rationale: required_text("rationale", rationale)?,
            provenance,
        })
    }
}

/// Structured reason a causal assessment is blocked or downgraded.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CausalObstruction {
    /// Machine-readable obstruction kind.
    pub kind: CausalObstructionKind,
    /// Stable lower snake case obstruction type.
    pub obstruction_type: String,
    /// Severity for downstream triage.
    pub severity: Severity,
    /// Claim related to the obstruction, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<Id>,
    /// Variables that locate the obstruction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variable_ids: Vec<Id>,
    /// Related claim records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_claim_ids: Vec<Id>,
    /// Related correlation records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_correlation_ids: Vec<Id>,
    /// Related intervention records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_intervention_ids: Vec<Id>,
    /// Related confounder records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_confounder_ids: Vec<Id>,
    /// Related adjustment-set records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_adjustment_set_ids: Vec<Id>,
    /// Human-readable diagnostic summary.
    pub message: String,
}

impl CausalObstruction {
    /// Creates an obstruction with no related source records.
    pub fn new(
        kind: CausalObstructionKind,
        severity: Severity,
        claim_id: Option<Id>,
        variable_ids: Vec<Id>,
        message: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            kind,
            obstruction_type: kind.as_str().to_owned(),
            severity,
            claim_id,
            variable_ids: unique_ids(variable_ids),
            related_claim_ids: Vec::new(),
            related_correlation_ids: Vec::new(),
            related_intervention_ids: Vec::new(),
            related_confounder_ids: Vec::new(),
            related_adjustment_set_ids: Vec::new(),
            message: required_text("message", message)?,
        })
    }

    /// Returns this obstruction with related claim IDs.
    #[must_use]
    pub fn with_related_claim_ids(mut self, claim_ids: Vec<Id>) -> Self {
        self.related_claim_ids = unique_ids(claim_ids);
        self
    }

    /// Returns this obstruction with related correlation IDs.
    #[must_use]
    pub fn with_related_correlation_ids(mut self, correlation_ids: Vec<Id>) -> Self {
        self.related_correlation_ids = unique_ids(correlation_ids);
        self
    }

    /// Returns this obstruction with related intervention IDs.
    #[must_use]
    pub fn with_related_intervention_ids(mut self, intervention_ids: Vec<Id>) -> Self {
        self.related_intervention_ids = unique_ids(intervention_ids);
        self
    }

    /// Returns this obstruction with related confounder IDs.
    #[must_use]
    pub fn with_related_confounder_ids(mut self, confounder_ids: Vec<Id>) -> Self {
        self.related_confounder_ids = unique_ids(confounder_ids);
        self
    }

    /// Returns this obstruction with related adjustment-set IDs.
    #[must_use]
    pub fn with_related_adjustment_set_ids(mut self, adjustment_set_ids: Vec<Id>) -> Self {
        self.related_adjustment_set_ids = unique_ids(adjustment_set_ids);
        self
    }
}

/// Conservative assessment output for a causal pair or claim.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CausalAssessment {
    /// Claim being assessed, when the query matched a claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<Id>,
    /// Queried cause-side variable.
    pub cause_id: Id,
    /// Queried effect-side variable.
    pub effect_id: Id,
    /// Conservative assessment status.
    pub status: CausalAssessmentStatus,
    /// Observed correlations between the queried variables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_correlation_ids: Vec<Id>,
    /// Interventions that manipulate the cause and report the effect as an outcome.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_intervention_ids: Vec<Id>,
    /// Adjustment sets attached to the claim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adjustment_set_ids: Vec<Id>,
    /// Active confounders not covered by any claim adjustment set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unadjusted_confounder_ids: Vec<Id>,
    /// Opposite-polarity claims for the same directed pair.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contradicting_claim_ids: Vec<Id>,
    /// Structured obstructions explaining non-supported statuses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<CausalObstruction>,
}

impl CausalAssessment {
    /// Returns true only for explicitly supported, unobstructed causal claims.
    #[must_use]
    pub fn supports_causality(&self) -> bool {
        matches!(self.status, CausalAssessmentStatus::SupportedCausalClaim)
    }
}

/// Conservative assessment output for one intervention.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterventionAssessment {
    /// Intervention being assessed.
    pub intervention_id: Id,
    /// Conservative intervention status.
    pub status: InterventionAssessmentStatus,
    /// Manipulated target variable.
    pub target_variable_id: Id,
    /// Observed outcome variables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcome_variable_ids: Vec<Id>,
    /// Structured obstructions explaining unsupported conclusions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<CausalObstruction>,
}
