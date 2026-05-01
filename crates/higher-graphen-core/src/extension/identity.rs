use super::common::{Description, LifecycleStatus, ObjectRef, ReviewRequirement};
use super::validation::{
    require_declared_scope, require_min_len, require_non_empty, require_reviewed, require_some,
};
use crate::text::normalize_required_text;
use crate::{Confidence, CoreError, Id, Provenance, Result, ReviewStatus};
use serde::{Deserialize, Serialize};

/// Kind of equivalence asserted by an [`EquivalenceClaim`].
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EquivalenceKind {
    /// The subjects are asserted to be identical.
    StrictIdentity,
    /// The subjects are equivalent only in declared contexts.
    ContextualEquivalence,
    /// The subjects are indistinguishable under declared observations.
    ObservationalEquivalence,
    /// The subjects behave equivalently under declared operations.
    BehavioralEquivalence,
    /// The subjects are semantically close, but not identical.
    SemanticNearEquivalence,
    /// The subjects may be collapsed in a quotient view.
    QuotientEquivalence,
}

/// Scope in which an equivalence claim may be considered.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EquivalenceScope {
    /// Contexts where the claim may be valid.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<Id>,
    /// Morphisms under which the claim is expected to remain valid.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub valid_under_morphisms: Vec<Id>,
}

impl EquivalenceScope {
    /// Returns true when at least one scope boundary is declared.
    pub fn is_declared(&self) -> bool {
        !self.contexts.is_empty() || !self.valid_under_morphisms.is_empty()
    }
}

/// Criterion used to judge an equivalence claim.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EquivalenceCriterion {
    /// Criterion description.
    pub description: String,
    /// Invariants that must be preserved by the equivalence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_invariants: Vec<Id>,
    /// Distinctions intentionally ignored by the claim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignored_distinctions: Vec<Description>,
}

impl EquivalenceCriterion {
    /// Creates a criterion with a validated description.
    pub fn new(description: impl Into<String>) -> Result<Self> {
        Ok(Self {
            description: normalize_required_text("criterion.description", description)?,
            required_invariants: Vec::new(),
            ignored_distinctions: Vec::new(),
        })
    }
}

/// Preview of the structural effect of accepting an equivalence claim.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QuotientEffect {
    /// Distinctions lost by the quotient.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lost_distinctions: Vec<Description>,
    /// Cells that would be merged in a quotient view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merged_cells: Vec<Id>,
    /// Invariants affected by the quotient.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_invariants: Vec<Id>,
    /// Projections affected by the quotient.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_projections: Vec<Id>,
    /// Unresolved obstructions that block acceptance.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_obstructions: Vec<Id>,
}

/// Reviewable claim that several structures may be treated as equivalent.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EquivalenceClaim {
    /// Claim identifier.
    pub id: Id,
    /// Structures whose equivalence is claimed.
    pub subjects: Vec<ObjectRef>,
    /// Equivalence kind.
    pub equivalence_kind: EquivalenceKind,
    /// Declared validity scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<EquivalenceScope>,
    /// Equivalence criterion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criterion: Option<EquivalenceCriterion>,
    /// Supporting witnesses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witnesses: Vec<Id>,
    /// Contradicting witnesses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counter_witnesses: Vec<Id>,
    /// Precomputed quotient effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotient_effect: Option<QuotientEffect>,
    /// Confidence in the claim.
    pub confidence: Confidence,
    /// Claim lifecycle state.
    pub status: LifecycleStatus,
    /// Claim provenance.
    pub provenance: Provenance,
    /// Review requirement and decision data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<ReviewRequirement>,
}

impl EquivalenceClaim {
    /// Creates a candidate equivalence claim.
    pub fn candidate(
        id: Id,
        subjects: Vec<ObjectRef>,
        equivalence_kind: EquivalenceKind,
        confidence: Confidence,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            subjects,
            equivalence_kind,
            scope: None,
            criterion: None,
            witnesses: Vec::new(),
            counter_witnesses: Vec::new(),
            quotient_effect: None,
            confidence,
            status: LifecycleStatus::Candidate,
            provenance,
            review: None,
        }
    }

    /// Validates conditions required before treating the claim as accepted.
    pub fn validate_acceptance(&self) -> Result<()> {
        require_min_len("subjects", self.subjects.len(), 2)?;
        require_declared_scope(self.scope.as_ref().map(EquivalenceScope::is_declared))?;
        require_some("criterion", self.criterion.as_ref())?;
        require_non_empty("witnesses", &self.witnesses)?;

        let quotient_effect = require_some("quotient_effect", self.quotient_effect.as_ref())?;
        if !quotient_effect.unresolved_obstructions.is_empty() {
            return Err(CoreError::malformed_field(
                "quotient_effect.unresolved_obstructions",
                "accepted equivalence must not have unresolved obstructions",
            ));
        }

        if self.equivalence_kind == EquivalenceKind::StrictIdentity
            && !self.counter_witnesses.is_empty()
        {
            return Err(CoreError::malformed_field(
                "counter_witnesses",
                "strict identity cannot be accepted with counter witnesses",
            ));
        }

        if self.provenance.source.kind == crate::SourceKind::Ai
            && self.provenance.review_status != ReviewStatus::Accepted
        {
            return Err(CoreError::malformed_field(
                "provenance.review_status",
                "AI-generated equivalence requires explicit accepted review",
            ));
        }

        require_reviewed(self.review.as_ref(), "review")?;
        Ok(())
    }

    /// Returns true when a merge operation may proceed.
    pub fn can_merge_equivalence(&self) -> bool {
        self.status.is_accepted() && self.validate_acceptance().is_ok()
    }
}
