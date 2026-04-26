use super::*;

/// In-memory causal graph record set and deterministic conservative checks.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CausalGraph {
    /// Declared causal variables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<CausalVariable>,
    /// Observed association records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_correlations: Vec<ObservedCorrelation>,
    /// Directional causal and non-causal claims.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causal_claims: Vec<CausalClaim>,
    /// Intervention records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interventions: Vec<Intervention>,
    /// Possible confounder records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub confounders: Vec<Confounder>,
    /// Claim-scoped adjustment sets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adjustment_sets: Vec<AdjustmentSet>,
    /// Whether directed cycles among positive causal claims are allowed.
    #[serde(default, skip_serializing_if = "is_false")]
    pub allow_feedback_cycles: bool,
}

#[derive(Clone, Debug, Default)]
struct ClaimSignals {
    observed_correlation_ids: Vec<Id>,
    supporting_intervention_ids: Vec<Id>,
    adjustment_set_ids: Vec<Id>,
    unadjusted_confounder_ids: Vec<Id>,
    contradicting_claim_ids: Vec<Id>,
}

impl ClaimSignals {
    fn has_explicit_support(&self) -> bool {
        !self.supporting_intervention_ids.is_empty() || !self.adjustment_set_ids.is_empty()
    }
}

impl CausalGraph {
    /// Creates an empty causal graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this graph with feedback cycles either allowed or disallowed.
    #[must_use]
    pub fn with_feedback_cycles_allowed(mut self, allow_feedback_cycles: bool) -> Self {
        self.allow_feedback_cycles = allow_feedback_cycles;
        self
    }

    /// Returns this graph with a variable appended.
    #[must_use]
    pub fn with_variable(mut self, variable: CausalVariable) -> Self {
        self.variables.push(variable);
        self
    }

    /// Returns this graph with an observed correlation appended.
    #[must_use]
    pub fn with_observed_correlation(mut self, correlation: ObservedCorrelation) -> Self {
        self.observed_correlations.push(correlation);
        self
    }

    /// Returns this graph with a causal claim appended.
    #[must_use]
    pub fn with_causal_claim(mut self, claim: CausalClaim) -> Self {
        self.causal_claims.push(claim);
        self
    }

    /// Returns this graph with an intervention appended.
    #[must_use]
    pub fn with_intervention(mut self, intervention: Intervention) -> Self {
        self.interventions.push(intervention);
        self
    }

    /// Returns this graph with a confounder appended.
    #[must_use]
    pub fn with_confounder(mut self, confounder: Confounder) -> Self {
        self.confounders.push(confounder);
        self
    }

    /// Returns this graph with an adjustment set appended.
    #[must_use]
    pub fn with_adjustment_set(mut self, adjustment_set: AdjustmentSet) -> Self {
        self.adjustment_sets.push(adjustment_set);
        self
    }

    /// Validates identifier uniqueness, required text, finite numbers, and provenance records.
    pub fn validate(&self) -> Result<()> {
        ensure_unique_ids(
            "variables",
            self.variables.iter().map(|variable| &variable.id),
        )?;
        ensure_unique_ids(
            "observed_correlations",
            self.observed_correlations
                .iter()
                .map(|correlation| &correlation.id),
        )?;
        ensure_unique_ids(
            "causal_claims",
            self.causal_claims.iter().map(|claim| &claim.id),
        )?;
        ensure_unique_ids(
            "interventions",
            self.interventions
                .iter()
                .map(|intervention| &intervention.id),
        )?;
        ensure_unique_ids(
            "confounders",
            self.confounders.iter().map(|confounder| &confounder.id),
        )?;
        ensure_unique_ids(
            "adjustment_sets",
            self.adjustment_sets
                .iter()
                .map(|adjustment_set| &adjustment_set.id),
        )?;

        for variable in &self.variables {
            required_text_ref("variables.name", &variable.name)?;
            if let Some(description) = &variable.description {
                required_text_ref("variables.description", description)?;
            }
            if let Some(provenance) = &variable.provenance {
                provenance.validate()?;
            }
        }
        for correlation in &self.observed_correlations {
            if let Some(magnitude) = correlation.magnitude {
                ensure_finite("observed_correlations.magnitude", magnitude)?;
            }
            correlation.provenance.validate()?;
        }
        for claim in &self.causal_claims {
            claim.provenance.validate()?;
        }
        for intervention in &self.interventions {
            intervention.provenance.validate()?;
        }
        for confounder in &self.confounders {
            confounder.provenance.validate()?;
        }
        for adjustment_set in &self.adjustment_sets {
            required_text_ref("adjustment_sets.rationale", &adjustment_set.rationale)?;
            adjustment_set.provenance.validate()?;
        }

        Ok(())
    }

    /// Returns structural obstructions that are independent of a single query.
    pub fn structural_obstructions(&self) -> Result<Vec<CausalObstruction>> {
        self.validate()?;

        let variable_ids = self.variable_ids();
        let mut obstructions = Vec::new();

        for correlation in &self.observed_correlations {
            push_missing_variable_obstruction(
                &mut obstructions,
                None,
                &variable_ids,
                [&correlation.variable_a_id, &correlation.variable_b_id],
                "observed correlation references an undeclared variable",
            )?;
        }
        for claim in &self.causal_claims {
            push_missing_variable_obstruction(
                &mut obstructions,
                Some(claim.id.clone()),
                &variable_ids,
                [&claim.cause_id, &claim.effect_id],
                "causal claim references an undeclared variable",
            )?;
            if claim.cause_id == claim.effect_id {
                obstructions.push(CausalObstruction::new(
                    CausalObstructionKind::SelfCausation,
                    Severity::High,
                    Some(claim.id.clone()),
                    vec![claim.cause_id.clone()],
                    format!("claim {} points a variable to itself", claim.id),
                )?);
            }
        }
        for intervention in &self.interventions {
            let variable_refs = std::iter::once(&intervention.target_variable_id)
                .chain(intervention.outcome_variable_ids.iter());
            push_missing_variable_obstruction(
                &mut obstructions,
                None,
                &variable_ids,
                variable_refs,
                "intervention references an undeclared variable",
            )?;
        }
        for confounder in &self.confounders {
            push_missing_variable_obstruction(
                &mut obstructions,
                None,
                &variable_ids,
                [
                    &confounder.variable_id,
                    &confounder.cause_id,
                    &confounder.effect_id,
                ],
                "confounder references an undeclared variable",
            )?;
        }
        for adjustment_set in &self.adjustment_sets {
            push_missing_variable_obstruction(
                &mut obstructions,
                Some(adjustment_set.claim_id.clone()),
                &variable_ids,
                adjustment_set.variable_ids.iter(),
                "adjustment set references an undeclared variable",
            )?;
        }

        if !self.allow_feedback_cycles {
            if let Some(cycle) = self.first_causal_cycle() {
                obstructions.push(CausalObstruction::new(
                    CausalObstructionKind::CausalCycle,
                    Severity::High,
                    None,
                    cycle.clone(),
                    format!("directed causal cycle detected: {}", join_ids(&cycle)),
                )?);
            }
        }

        Ok(obstructions)
    }

    /// Assesses a directed pair, using the first matching claim if one exists.
    pub fn assess_pair(&self, cause_id: &Id, effect_id: &Id) -> Result<CausalAssessment> {
        self.validate()?;
        if let Some(claim) = self
            .causal_claims
            .iter()
            .filter(|claim| claim.cause_id == *cause_id && claim.effect_id == *effect_id)
            .min_by(|left, right| left.id.cmp(&right.id))
        {
            return self.assess_claim_record(claim);
        }

        let observed_correlation_ids = self.observed_correlation_ids(cause_id, effect_id);
        let mut obstructions = self.query_structural_obstructions(None, cause_id, effect_id)?;
        let status = if !obstructions.is_empty() {
            CausalAssessmentStatus::Obstructed
        } else if observed_correlation_ids.is_empty() {
            CausalAssessmentStatus::NoEvidence
        } else {
            obstructions.push(
                CausalObstruction::new(
                    CausalObstructionKind::CorrelationOnly,
                    Severity::Low,
                    None,
                    vec![cause_id.clone(), effect_id.clone()],
                    "observed correlation is association evidence, not a causal claim",
                )?
                .with_related_correlation_ids(observed_correlation_ids.clone()),
            );
            CausalAssessmentStatus::ObservedCorrelationOnly
        };

        Ok(CausalAssessment {
            claim_id: None,
            cause_id: cause_id.clone(),
            effect_id: effect_id.clone(),
            status,
            observed_correlation_ids,
            supporting_intervention_ids: Vec::new(),
            adjustment_set_ids: Vec::new(),
            unadjusted_confounder_ids: Vec::new(),
            contradicting_claim_ids: Vec::new(),
            obstructions,
        })
    }

    /// Assesses one recorded claim by identifier.
    pub fn assess_claim(&self, claim_id: &Id) -> Result<CausalAssessment> {
        self.validate()?;
        let claim = self
            .causal_claims
            .iter()
            .find(|claim| &claim.id == claim_id)
            .ok_or_else(|| malformed("claim_id", format!("identifier {claim_id} is absent")))?;
        self.assess_claim_record(claim)
    }

    /// Assesses whether an intervention can support any causal conclusion.
    pub fn assess_intervention(&self, intervention_id: &Id) -> Result<InterventionAssessment> {
        self.validate()?;
        let intervention = self
            .interventions
            .iter()
            .find(|intervention| &intervention.id == intervention_id)
            .ok_or_else(|| {
                malformed(
                    "intervention_id",
                    format!("identifier {intervention_id} is absent"),
                )
            })?;

        if intervention.outcome_variable_ids.is_empty() {
            let obstruction = CausalObstruction::new(
                CausalObstructionKind::UnsupportedInterventionConclusion,
                Severity::Medium,
                None,
                vec![intervention.target_variable_id.clone()],
                "intervention has no observed outcome variable",
            )?
            .with_related_intervention_ids(vec![intervention.id.clone()]);
            return Ok(InterventionAssessment {
                intervention_id: intervention.id.clone(),
                status: InterventionAssessmentStatus::UnsupportedInterventionConclusion,
                target_variable_id: intervention.target_variable_id.clone(),
                outcome_variable_ids: Vec::new(),
                obstructions: vec![obstruction],
            });
        }

        Ok(InterventionAssessment {
            intervention_id: intervention.id.clone(),
            status: InterventionAssessmentStatus::OutcomeObserved,
            target_variable_id: intervention.target_variable_id.clone(),
            outcome_variable_ids: normalized_ids(&intervention.outcome_variable_ids),
            obstructions: Vec::new(),
        })
    }

    fn assess_claim_record(&self, claim: &CausalClaim) -> Result<CausalAssessment> {
        let signals = self.claim_signals(claim);
        let mut obstructions =
            self.query_structural_obstructions(Some(&claim.id), &claim.cause_id, &claim.effect_id)?;
        self.add_claim_obstructions(claim, &signals, &mut obstructions)?;
        let status = claim_status(claim, &signals, &obstructions);

        Ok(CausalAssessment {
            claim_id: Some(claim.id.clone()),
            cause_id: claim.cause_id.clone(),
            effect_id: claim.effect_id.clone(),
            status,
            observed_correlation_ids: unique_ids(signals.observed_correlation_ids),
            supporting_intervention_ids: unique_ids(signals.supporting_intervention_ids),
            adjustment_set_ids: unique_ids(signals.adjustment_set_ids),
            unadjusted_confounder_ids: unique_ids(signals.unadjusted_confounder_ids),
            contradicting_claim_ids: unique_ids(signals.contradicting_claim_ids),
            obstructions,
        })
    }

    fn claim_signals(&self, claim: &CausalClaim) -> ClaimSignals {
        let observed_correlation_ids =
            self.observed_correlation_ids(&claim.cause_id, &claim.effect_id);
        let supporting_intervention_ids = self
            .interventions
            .iter()
            .filter(|intervention| intervention.supports_claim(claim))
            .map(|intervention| intervention.id.clone())
            .collect::<Vec<_>>();
        let adjustment_set_ids = self
            .adjustment_sets
            .iter()
            .filter(|adjustment_set| adjustment_set.claim_id == claim.id)
            .map(|adjustment_set| adjustment_set.id.clone())
            .collect::<Vec<_>>();
        let adjusted_variable_ids = self.adjusted_variable_ids(&claim.id);
        let unadjusted_confounder_ids = self
            .confounders
            .iter()
            .filter(|confounder| confounder.blocks_pair(&claim.cause_id, &claim.effect_id))
            .filter(|confounder| !adjusted_variable_ids.contains(&confounder.variable_id))
            .map(|confounder| confounder.id.clone())
            .collect::<Vec<_>>();
        let contradicting_claim_ids = self.contradicting_claim_ids(claim);

        ClaimSignals {
            observed_correlation_ids,
            supporting_intervention_ids,
            adjustment_set_ids,
            unadjusted_confounder_ids,
            contradicting_claim_ids,
        }
    }

    fn add_claim_obstructions(
        &self,
        claim: &CausalClaim,
        signals: &ClaimSignals,
        obstructions: &mut Vec<CausalObstruction>,
    ) -> Result<()> {
        if !signals.contradicting_claim_ids.is_empty() {
            obstructions.push(
                CausalObstruction::new(
                    CausalObstructionKind::ContradictedCausalClaim,
                    Severity::High,
                    Some(claim.id.clone()),
                    vec![claim.cause_id.clone(), claim.effect_id.clone()],
                    format!(
                        "claim {} conflicts with an opposite-polarity claim",
                        claim.id
                    ),
                )?
                .with_related_claim_ids(signals.contradicting_claim_ids.clone()),
            );
        }

        if !signals.unadjusted_confounder_ids.is_empty() {
            obstructions.push(
                CausalObstruction::new(
                    CausalObstructionKind::Confounded,
                    Severity::High,
                    Some(claim.id.clone()),
                    vec![claim.cause_id.clone(), claim.effect_id.clone()],
                    "active confounders are not covered by a claim adjustment set",
                )?
                .with_related_confounder_ids(signals.unadjusted_confounder_ids.clone())
                .with_related_adjustment_set_ids(signals.adjustment_set_ids.clone()),
            );
        }

        if claim.polarity == CausalClaimPolarity::Causes && !signals.has_explicit_support() {
            obstructions.push(
                CausalObstruction::new(
                    CausalObstructionKind::UnsupportedCausalClaim,
                    Severity::Medium,
                    Some(claim.id.clone()),
                    vec![claim.cause_id.clone(), claim.effect_id.clone()],
                    "causal claim has no supporting intervention or adjustment set",
                )?
                .with_related_correlation_ids(signals.observed_correlation_ids.clone()),
            );
        }

        Ok(())
    }

    fn observed_correlation_ids(&self, cause_id: &Id, effect_id: &Id) -> Vec<Id> {
        self.observed_correlations
            .iter()
            .filter(|correlation| correlation.matches_pair(cause_id, effect_id))
            .map(|correlation| correlation.id.clone())
            .collect()
    }

    fn adjusted_variable_ids(&self, claim_id: &Id) -> BTreeSet<Id> {
        self.adjustment_sets
            .iter()
            .filter(|adjustment_set| &adjustment_set.claim_id == claim_id)
            .flat_map(|adjustment_set| adjustment_set.variable_ids.iter().cloned())
            .collect()
    }

    fn contradicting_claim_ids(&self, claim: &CausalClaim) -> Vec<Id> {
        self.causal_claims
            .iter()
            .filter(|other| other.id != claim.id)
            .filter(|other| other.cause_id == claim.cause_id && other.effect_id == claim.effect_id)
            .filter(|other| other.polarity != claim.polarity)
            .map(|other| other.id.clone())
            .collect()
    }

    fn query_structural_obstructions(
        &self,
        claim_id: Option<&Id>,
        cause_id: &Id,
        effect_id: &Id,
    ) -> Result<Vec<CausalObstruction>> {
        let variable_ids = self.variable_ids();
        let mut obstructions = Vec::new();
        push_missing_variable_obstruction(
            &mut obstructions,
            claim_id.cloned(),
            &variable_ids,
            [cause_id, effect_id],
            "causal query references an undeclared variable",
        )?;

        if cause_id == effect_id {
            obstructions.push(CausalObstruction::new(
                CausalObstructionKind::SelfCausation,
                Severity::High,
                claim_id.cloned(),
                vec![cause_id.clone()],
                "causal query points a variable to itself",
            )?);
        }

        if !self.allow_feedback_cycles {
            if let Some(cycle) = self
                .first_causal_cycle()
                .filter(|cycle| cycle.contains(cause_id) && cycle.contains(effect_id))
            {
                obstructions.push(CausalObstruction::new(
                    CausalObstructionKind::CausalCycle,
                    Severity::High,
                    claim_id.cloned(),
                    cycle.clone(),
                    format!("directed causal cycle detected: {}", join_ids(&cycle)),
                )?);
            }
        }

        Ok(obstructions)
    }

    fn variable_ids(&self) -> BTreeSet<Id> {
        self.variables
            .iter()
            .map(|variable| variable.id.clone())
            .collect()
    }

    fn first_causal_cycle(&self) -> Option<Vec<Id>> {
        let mut adjacency: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
        for claim in &self.causal_claims {
            if claim.polarity != CausalClaimPolarity::Causes {
                continue;
            }
            adjacency
                .entry(claim.cause_id.clone())
                .or_default()
                .push(claim.effect_id.clone());
            adjacency.entry(claim.effect_id.clone()).or_default();
        }
        for neighbors in adjacency.values_mut() {
            normalize_ids_in_place(neighbors);
        }

        let mut states = BTreeMap::new();
        let mut stack = Vec::new();
        for variable_id in adjacency.keys().cloned().collect::<Vec<_>>() {
            if states.contains_key(&variable_id) {
                continue;
            }
            if let Some(cycle) =
                directed_cycle_from(&variable_id, &adjacency, &mut states, &mut stack)
            {
                return Some(cycle);
            }
        }
        None
    }
}

fn claim_status(
    claim: &CausalClaim,
    signals: &ClaimSignals,
    obstructions: &[CausalObstruction],
) -> CausalAssessmentStatus {
    if obstructions.iter().any(is_structural_obstruction) {
        CausalAssessmentStatus::Obstructed
    } else if !signals.contradicting_claim_ids.is_empty() {
        CausalAssessmentStatus::Contradicted
    } else if !signals.unadjusted_confounder_ids.is_empty() {
        CausalAssessmentStatus::Confounded
    } else if !signals.has_explicit_support() || claim.polarity == CausalClaimPolarity::DoesNotCause
    {
        CausalAssessmentStatus::UnsupportedCausalClaim
    } else {
        CausalAssessmentStatus::SupportedCausalClaim
    }
}

fn is_structural_obstruction(obstruction: &CausalObstruction) -> bool {
    matches!(
        obstruction.kind,
        CausalObstructionKind::MissingVariable
            | CausalObstructionKind::SelfCausation
            | CausalObstructionKind::CausalCycle
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisitState {
    Visiting,
    Visited,
}

fn directed_cycle_from(
    variable_id: &Id,
    adjacency: &BTreeMap<Id, Vec<Id>>,
    states: &mut BTreeMap<Id, VisitState>,
    stack: &mut Vec<Id>,
) -> Option<Vec<Id>> {
    states.insert(variable_id.clone(), VisitState::Visiting);
    stack.push(variable_id.clone());

    for neighbor_id in adjacency.get(variable_id).into_iter().flatten() {
        match states.get(neighbor_id).copied() {
            Some(VisitState::Visiting) => {
                let start = stack
                    .iter()
                    .position(|candidate| candidate == neighbor_id)
                    .unwrap_or(0);
                let mut cycle = stack[start..].to_vec();
                cycle.push(neighbor_id.clone());
                return Some(cycle);
            }
            Some(VisitState::Visited) => {}
            None => {
                if let Some(cycle) = directed_cycle_from(neighbor_id, adjacency, states, stack) {
                    return Some(cycle);
                }
            }
        }
    }

    stack.pop();
    states.insert(variable_id.clone(), VisitState::Visited);
    None
}

fn push_missing_variable_obstruction<'a, I>(
    obstructions: &mut Vec<CausalObstruction>,
    claim_id: Option<Id>,
    declared_variable_ids: &BTreeSet<Id>,
    variable_ids: I,
    message: &'static str,
) -> Result<()>
where
    I: IntoIterator<Item = &'a Id>,
{
    let missing = variable_ids
        .into_iter()
        .filter(|variable_id| !declared_variable_ids.contains(*variable_id))
        .cloned()
        .collect::<BTreeSet<_>>();
    if missing.is_empty() {
        return Ok(());
    }

    obstructions.push(CausalObstruction::new(
        CausalObstructionKind::MissingVariable,
        Severity::High,
        claim_id,
        missing.into_iter().collect(),
        message,
    )?);
    Ok(())
}
