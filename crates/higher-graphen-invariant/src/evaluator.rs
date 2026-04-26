use super::*;

/// Reusable deterministic evaluator over invariant and constraint rule records.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatorKernel {
    /// Rule records evaluated by this kernel.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<EvaluatorRule>,
}

impl EvaluatorKernel {
    /// Creates an empty evaluator kernel.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this kernel with an appended evaluator rule.
    #[must_use]
    pub fn with_rule(mut self, rule: EvaluatorRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Evaluates selected rules and returns one result per evaluated rule.
    pub fn evaluate(&self, context: &EvaluatorContext<'_>) -> Result<EvaluationReport> {
        let normalized_input = context.check_input.normalized();
        let normalized_context = EvaluatorContext {
            check_input: &normalized_input,
            space_store: context.space_store,
            morphisms: context.morphisms,
            projections: context.projections,
        };
        let mut results = Vec::new();

        for rule in &self.rules {
            if rule.is_selected_by(&normalized_input) {
                results.push(rule.evaluate(&normalized_context)?);
            }
        }

        Ok(EvaluationReport { results })
    }
}

/// Runtime data supplied to evaluator rules.
#[derive(Clone, Copy, Debug)]
pub struct EvaluatorContext<'a> {
    /// Normal or incremental check selector.
    pub check_input: &'a CheckInput,
    /// Space store used by structural graph checks.
    pub space_store: &'a InMemorySpaceStore,
    /// Morphisms available to preservation checks.
    pub morphisms: &'a [Morphism],
    /// Projections available to information-loss declaration checks.
    pub projections: &'a [Projection],
}

impl<'a> EvaluatorContext<'a> {
    /// Creates a context for checks that only need a space store.
    #[must_use]
    pub fn new(check_input: &'a CheckInput, space_store: &'a InMemorySpaceStore) -> Self {
        Self {
            check_input,
            space_store,
            morphisms: &[],
            projections: &[],
        }
    }

    /// Returns this context with morphisms available to preservation checks.
    #[must_use]
    pub fn with_morphisms(mut self, morphisms: &'a [Morphism]) -> Self {
        self.morphisms = morphisms;
        self
    }

    /// Returns this context with projections available to declaration checks.
    #[must_use]
    pub fn with_projections(mut self, projections: &'a [Projection]) -> Self {
        self.projections = projections;
        self
    }
}

/// Evaluation output for a batch of invariant and constraint rules.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationReport {
    /// Results in rule evaluation order after input selection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<CheckResult>,
}

impl EvaluationReport {
    /// Returns true when every evaluated rule was satisfied.
    #[must_use]
    pub fn all_satisfied(&self) -> bool {
        self.results.iter().all(CheckResult::is_satisfied)
    }

    /// Returns violated results.
    #[must_use]
    pub fn violations(&self) -> Vec<&CheckResult> {
        self.results
            .iter()
            .filter(|result| result.is_violated())
            .collect()
    }

    /// Returns unsupported results.
    #[must_use]
    pub fn unsupported(&self) -> Vec<&CheckResult> {
        self.results
            .iter()
            .filter(|result| result.is_unsupported())
            .collect()
    }
}

/// One evaluator rule bound to an invariant or constraint target.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatorRule {
    /// Kind of definition targeted by the rule.
    pub target_kind: CheckTargetKind,
    /// Identifier of the targeted invariant or constraint.
    pub target_id: Id,
    /// Impact used when the rule produces a violation.
    pub severity: Severity,
    /// Product-neutral deterministic check performed by the rule.
    pub check: EvaluatorCheck,
}

impl EvaluatorRule {
    /// Creates a rule for an invariant or constraint target.
    #[must_use]
    pub fn new(
        target_kind: CheckTargetKind,
        target_id: Id,
        severity: Severity,
        check: EvaluatorCheck,
    ) -> Self {
        Self {
            target_kind,
            target_id,
            severity,
            check,
        }
    }

    /// Creates a rule targeting an invariant.
    #[must_use]
    pub fn invariant(target_id: Id, severity: Severity, check: EvaluatorCheck) -> Self {
        Self::new(CheckTargetKind::Invariant, target_id, severity, check)
    }

    /// Creates a rule targeting a constraint.
    #[must_use]
    pub fn constraint(target_id: Id, severity: Severity, check: EvaluatorCheck) -> Self {
        Self::new(CheckTargetKind::Constraint, target_id, severity, check)
    }

    /// Evaluates this rule against the supplied context.
    pub fn evaluate(&self, context: &EvaluatorContext<'_>) -> Result<CheckResult> {
        match &self.check {
            EvaluatorCheck::Acyclicity(check) => evaluate_acyclicity(self, check, context),
            EvaluatorCheck::RequiredPath(check) => evaluate_required_path(self, check, context),
            EvaluatorCheck::ReachabilitySafety(check) => {
                evaluate_reachability_safety(self, check, context)
            }
            EvaluatorCheck::ContextCompatibility(check) => {
                evaluate_context_compatibility(self, check, context)
            }
            EvaluatorCheck::MorphismPreservation(check) => {
                evaluate_morphism_preservation(self, check, context)
            }
            EvaluatorCheck::ProjectionLossDeclared(check) => {
                evaluate_projection_loss_declared(self, check, context)
            }
        }
    }

    fn is_selected_by(&self, input: &CheckInput) -> bool {
        let has_explicit_selection =
            !input.invariant_ids.is_empty() || !input.constraint_ids.is_empty();
        match self.target_kind {
            CheckTargetKind::Invariant => {
                if input.invariant_ids.is_empty() {
                    !has_explicit_selection
                } else {
                    input.invariant_ids.contains(&self.target_id)
                }
            }
            CheckTargetKind::Constraint => {
                if input.constraint_ids.is_empty() {
                    !has_explicit_selection
                } else {
                    input.constraint_ids.contains(&self.target_id)
                }
            }
        }
    }
}

/// Supported deterministic evaluator check kinds.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "spec", rename_all = "snake_case")]
pub enum EvaluatorCheck {
    /// Detects whether selected incidences form a directed cycle.
    Acyclicity(AcyclicityCheck),
    /// Requires at least one path between two cells.
    RequiredPath(RequiredPathCheck),
    /// Rejects reachability from selected source cells to forbidden cells.
    ReachabilitySafety(ReachabilitySafetyCheck),
    /// Placeholder compatibility check based on declared cell context membership.
    ContextCompatibility(ContextCompatibilityCheck),
    /// Checks selected invariant IDs against a morphism preservation report.
    MorphismPreservation(MorphismPreservationCheck),
    /// Requires projection information loss to be explicitly declared.
    ProjectionLossDeclared(ProjectionLossDeclarationCheck),
}

/// Acyclicity rule parameters.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AcyclicityCheck {
    /// Relation types included in the acyclicity graph. Empty means all relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relation_types: Vec<String>,
}

impl AcyclicityCheck {
    /// Creates an acyclicity check over all relation types.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this check with one included relation type.
    #[must_use]
    pub fn with_relation_type(mut self, relation_type: impl Into<String>) -> Self {
        self.relation_types
            .push(relation_type.into().trim().to_owned());
        self
    }
}

/// Required path rule parameters.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredPathCheck {
    /// Required path start cell.
    pub from_cell_id: Id,
    /// Required path target cell.
    pub to_cell_id: Id,
    /// Traversal controls for the path search.
    pub options: TraversalOptions,
}

impl RequiredPathCheck {
    /// Creates an outgoing required-path check with default traversal options.
    #[must_use]
    pub fn new(from_cell_id: Id, to_cell_id: Id) -> Self {
        Self {
            from_cell_id,
            to_cell_id,
            options: TraversalOptions::default(),
        }
    }

    /// Returns this check with traversal options.
    #[must_use]
    pub fn with_options(mut self, options: TraversalOptions) -> Self {
        self.options = options;
        self
    }
}

/// Reachability safety rule parameters.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReachabilitySafetyCheck {
    /// Cells from which forbidden cells must not be reachable.
    pub from_cell_ids: Vec<Id>,
    /// Cells that must not be reachable from `from_cell_ids`.
    pub forbidden_cell_ids: Vec<Id>,
    /// Traversal controls for reachability queries.
    pub options: TraversalOptions,
}

impl ReachabilitySafetyCheck {
    /// Creates a reachability safety check with default traversal options.
    #[must_use]
    pub fn new<I, F>(from_cell_ids: I, forbidden_cell_ids: F) -> Self
    where
        I: IntoIterator<Item = Id>,
        F: IntoIterator<Item = Id>,
    {
        Self {
            from_cell_ids: from_cell_ids.into_iter().collect(),
            forbidden_cell_ids: forbidden_cell_ids.into_iter().collect(),
            options: TraversalOptions::default(),
        }
    }

    /// Returns this check with traversal options.
    #[must_use]
    pub fn with_options(mut self, options: TraversalOptions) -> Self {
        self.options = options;
        self
    }
}

/// Context compatibility placeholder parameters.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextCompatibilityCheck {
    /// Cells to check. Empty means use `CheckInput.changed_cell_ids`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cell_ids: Vec<Id>,
    /// Contexts every checked cell must declare.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_context_ids: Vec<Id>,
}

impl ContextCompatibilityCheck {
    /// Creates a context compatibility check with explicit cells and contexts.
    #[must_use]
    pub fn new<I, C>(cell_ids: I, required_context_ids: C) -> Self
    where
        I: IntoIterator<Item = Id>,
        C: IntoIterator<Item = Id>,
    {
        Self {
            cell_ids: cell_ids.into_iter().collect(),
            required_context_ids: required_context_ids.into_iter().collect(),
        }
    }
}

/// Morphism preservation placeholder parameters.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MorphismPreservationCheck {
    /// Morphism whose preservation report is checked.
    pub morphism_id: Id,
    /// Invariants that must be preserved. Empty means use the rule target when it is an invariant.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariant_ids: Vec<Id>,
}

impl MorphismPreservationCheck {
    /// Creates a preservation check for one morphism.
    #[must_use]
    pub fn new(morphism_id: Id) -> Self {
        Self {
            morphism_id,
            invariant_ids: Vec::new(),
        }
    }

    /// Returns this check with explicit invariant IDs.
    #[must_use]
    pub fn with_invariants<I>(mut self, invariant_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.invariant_ids = invariant_ids.into_iter().collect();
        self
    }
}

/// Projection loss declaration rule parameters.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionLossDeclarationCheck {
    /// Projection whose information-loss declarations are checked.
    pub projection_id: Id,
    /// Source IDs that must be covered by at least one loss declaration.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_source_ids: Vec<Id>,
}

impl ProjectionLossDeclarationCheck {
    /// Creates a declaration check for one projection.
    #[must_use]
    pub fn new(projection_id: Id) -> Self {
        Self {
            projection_id,
            required_source_ids: Vec::new(),
        }
    }

    /// Returns this check with required source identifiers.
    #[must_use]
    pub fn with_required_sources<I>(mut self, required_source_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.required_source_ids = required_source_ids.into_iter().collect();
        self
    }
}

mod algorithms;
use algorithms::*;
