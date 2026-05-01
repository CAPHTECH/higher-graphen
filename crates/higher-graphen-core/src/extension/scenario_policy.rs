use super::common::{Description, LifecycleStatus, ObjectRef};
use super::validation::require_some;
use crate::text::{normalize_required_text, normalize_required_text_vec};
use crate::{CoreError, Id, Provenance, Result, ReviewStatus};
use serde::{Deserialize, Serialize};

/// Scenario kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    /// Hypothetical world.
    Hypothetical,
    /// Reachable world.
    Reachable,
    /// Blocked world.
    Blocked,
    /// Counterfactual world.
    Counterfactual,
    /// Planned world.
    Planned,
    /// Refuted world.
    Refuted,
    /// Accepted operational plan.
    AcceptedOperationalPlan,
}

/// Structures changed by a scenario.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioChanges {
    /// Added cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added: Vec<Id>,
    /// Removed cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<Id>,
    /// Modified morphisms.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modified: Vec<Id>,
}

/// Reachability path from a base space.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reachability {
    /// Source space.
    #[serde(rename = "ref")]
    pub reference: Id,
    /// Morphisms used to reach the scenario.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub via_morphisms: Vec<Id>,
}

/// Reviewable hypothetical, reachable, or counterfactual world.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    /// Scenario identifier.
    pub id: Id,
    /// Base space.
    pub base_space: Id,
    /// Scenario kind.
    pub scenario_kind: ScenarioKind,
    /// Assumption cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<Id>,
    /// Changed structures.
    pub changed_structures: ScenarioChanges,
    /// Reachability relation from another space.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reachable_from: Option<Reachability>,
    /// Affected invariants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_invariants: Vec<Id>,
    /// Expected obstructions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_obstructions: Vec<Id>,
    /// Required witnesses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_witnesses: Vec<Id>,
    /// Valuations attached to the scenario.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub valuations: Vec<Id>,
    /// Scenario status.
    pub status: ScenarioStatus,
    /// Scenario provenance.
    pub provenance: Provenance,
    /// Review status.
    pub review_status: LifecycleStatus,
}

/// Scenario-specific status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioStatus {
    /// Draft scenario.
    Draft,
    /// Candidate scenario.
    Candidate,
    /// Under review.
    UnderReview,
    /// Reachable from the base.
    Reachable,
    /// Blocked by obstructions.
    Blocked,
    /// Refuted.
    Refuted,
    /// Accepted for its declared use.
    Accepted,
}

impl Scenario {
    /// Validates conditions required before treating the scenario as accepted.
    pub fn validate_acceptance(&self) -> Result<()> {
        match self.scenario_kind {
            ScenarioKind::Hypothetical if self.status == ScenarioStatus::Accepted => {
                return Err(CoreError::malformed_field(
                    "scenario_kind",
                    "hypothetical scenario cannot be treated as accepted fact",
                ));
            }
            ScenarioKind::Reachable | ScenarioKind::AcceptedOperationalPlan => {
                require_some("reachable_from", self.reachable_from.as_ref())?;
            }
            _ => {}
        }
        if self.affected_invariants.is_empty() {
            return Err(CoreError::malformed_field(
                "affected_invariants",
                "accepted scenario requires invariant checks or affected invariant records",
            ));
        }
        if self.scenario_kind == ScenarioKind::AcceptedOperationalPlan
            && self.provenance.review_status != ReviewStatus::Accepted
        {
            return Err(CoreError::malformed_field(
                "provenance.review_status",
                "accepted operational plan requires policy/capability review",
            ));
        }
        Ok(())
    }
}

/// Operation governed by a capability.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityOperation {
    /// Read a target.
    Read,
    /// Propose a target.
    Propose,
    /// Modify a target.
    Modify,
    /// Accept a target.
    Accept,
    /// Reject a target.
    Reject,
    /// Project a target.
    Project,
    /// Execute a morphism.
    ExecuteMorphism,
    /// Merge an equivalence.
    MergeEquivalence,
    /// Create a scenario.
    CreateScenario,
    /// Approve a policy exception.
    ApprovePolicyException,
}

/// Review required by a capability.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredReview {
    /// Policy requiring review.
    pub policy: Id,
    /// Reviewer actor.
    pub reviewer: Id,
}

/// Validity interval for a capability.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidityInterval {
    /// Start timestamp.
    pub starts_at: String,
    /// End timestamp.
    pub ends_at: String,
}

impl ValidityInterval {
    /// Validates the portable timestamp payloads.
    pub fn validate(&self) -> Result<()> {
        normalize_required_text("validity_interval.starts_at", &self.starts_at)?;
        normalize_required_text("validity_interval.ends_at", &self.ends_at)?;
        Ok(())
    }
}

/// Capability status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    /// Active capability.
    Active,
    /// Temporarily suspended capability.
    Suspended,
    /// Expired capability.
    Expired,
    /// Revoked capability.
    Revoked,
    /// Candidate capability.
    Candidate,
}

/// Actor-specific ability to operate on a target in a context.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Capability {
    /// Capability identifier.
    pub id: Id,
    /// Actor cell.
    pub actor: Id,
    /// Operation granted.
    pub operation: CapabilityOperation,
    /// Target type.
    pub target_type: String,
    /// Target references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_refs: Vec<ObjectRef>,
    /// Valid contexts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<Id>,
    /// Preconditions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<Id>,
    /// Postconditions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub postconditions: Vec<Id>,
    /// Forbidden effects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_effects: Vec<Description>,
    /// Required review.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_review: Option<RequiredReview>,
    /// Validity interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_interval: Option<ValidityInterval>,
    /// Capability provenance.
    pub provenance: Provenance,
    /// Capability status.
    pub status: CapabilityStatus,
}

impl Capability {
    /// Validates whether this capability may be used for a mutating operation.
    pub fn validate_active_use(&self) -> Result<()> {
        if self.status != CapabilityStatus::Active {
            return Err(CoreError::malformed_field(
                "status",
                "capability must be active before use",
            ));
        }
        normalize_required_text("target_type", &self.target_type)?;
        if let Some(validity_interval) = &self.validity_interval {
            validity_interval.validate()?;
        }
        if self.operation == CapabilityOperation::Accept
            && self.required_review.is_none()
            && self.provenance.review_status != ReviewStatus::Accepted
        {
            return Err(CoreError::malformed_field(
                "required_review",
                "accept operation requires explicit policy review",
            ));
        }
        Ok(())
    }
}

/// Policy kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyKind {
    /// Permission policy.
    Permission,
    /// Prohibition policy.
    Prohibition,
    /// Obligation policy.
    Obligation,
    /// Review requirement policy.
    ReviewRequirement,
    /// Projection safety policy.
    ProjectionSafety,
    /// Candidate acceptance policy.
    CandidateAcceptance,
    /// Data boundary policy.
    DataBoundary,
    /// Escalation policy.
    Escalation,
}

/// Targets and operations a policy applies to.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyApplicability {
    /// Target type names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_types: Vec<String>,
    /// Context identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<Id>,
    /// Operation names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<String>,
}

impl PolicyApplicability {
    fn validate(&self) -> Result<()> {
        normalize_required_text_vec("target_types", &self.target_types)?;
        normalize_required_text_vec("operations", &self.operations)?;
        Ok(())
    }
}

/// Policy rule payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRule {
    /// Rule description.
    pub description: String,
    /// Constraint identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Id>,
}

impl PolicyRule {
    /// Creates a policy rule.
    pub fn new(description: impl Into<String>) -> Result<Self> {
        Ok(Self {
            description: normalize_required_text("policy.rule.description", description)?,
            constraints: Vec::new(),
        })
    }
}

/// Policy status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    /// Draft policy.
    Draft,
    /// Active policy.
    Active,
    /// Deprecated policy.
    Deprecated,
    /// Revoked policy.
    Revoked,
}

/// System-wide or context-bound rule.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    /// Policy identifier.
    pub id: Id,
    /// Policy kind.
    pub policy_kind: PolicyKind,
    /// Applicability.
    pub applies_to: PolicyApplicability,
    /// Policy rule.
    pub rule: PolicyRule,
    /// Required witnesses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_witnesses: Vec<Id>,
    /// Required derivations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_derivations: Vec<Id>,
    /// Escalation path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub escalation_path: Vec<Id>,
    /// Violation obstruction template id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violation_obstruction_template: Option<Id>,
    /// Policy status.
    pub status: PolicyStatus,
    /// Policy provenance.
    pub provenance: Provenance,
    /// Policy review status.
    pub review_status: ReviewStatus,
}

impl Policy {
    /// Validates conditions required for an active policy.
    pub fn validate_active(&self) -> Result<()> {
        self.applies_to.validate()?;
        normalize_required_text("policy.rule.description", &self.rule.description)?;
        if self.status == PolicyStatus::Active && self.review_status != ReviewStatus::Accepted {
            return Err(CoreError::malformed_field(
                "review_status",
                "active policy requires accepted review status",
            ));
        }
        if self.status == PolicyStatus::Active
            && self.provenance.review_status != ReviewStatus::Accepted
        {
            return Err(CoreError::malformed_field(
                "provenance.review_status",
                "active policy provenance must be accepted",
            ));
        }
        Ok(())
    }
}
