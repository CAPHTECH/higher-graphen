use super::common::LifecycleStatus;
use crate::text::normalize_required_text;
use crate::{CoreError, Id, Provenance, Result, ReviewStatus};
use serde::{Deserialize, Serialize};

/// Inference verifier category for a derivation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierKind {
    /// Human review validated the inference.
    HumanReview,
    /// Schema validation checked the inference.
    SchemaValidator,
    /// A proof checker validated the inference.
    ProofChecker,
    /// A test run checked the inference.
    TestRun,
    /// Static analysis checked the inference.
    StaticAnalysis,
    /// A custom engine checked the inference.
    CustomEngine,
}

/// Verifier used by a derivation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Verifier {
    /// Verifier kind.
    pub kind: VerifierKind,
    /// Verifier reference, command, URL, or engine id.
    pub reference: String,
}

impl Verifier {
    /// Creates a verifier with a validated reference.
    pub fn new(kind: VerifierKind, reference: impl Into<String>) -> Result<Self> {
        Ok(Self {
            kind,
            reference: normalize_required_text("verifier.reference", reference)?,
        })
    }
}

/// Status of derivation verification.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// No verifier or review has checked the inference.
    Unverified,
    /// Machine verification succeeded.
    MachineChecked,
    /// Human review succeeded.
    HumanReviewed,
    /// Verification failed.
    Failed,
    /// Replaced by a later derivation.
    Superseded,
}

/// Derivation failure mode.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivationFailureMode {
    /// A required premise is missing.
    MissingPremise,
    /// The inference rule is invalid.
    InvalidRule,
    /// The rule is applied outside its scope.
    OutOfScopeRule,
    /// A witness contradicts the derivation.
    ContradictedByWitness,
    /// The derivation is circular.
    CircularDerivation,
    /// The conclusion jumps beyond the premises and rule.
    UnsupportedJump,
    /// The verifier could not be run.
    VerifierUnavailable,
    /// No failure is known.
    None,
}

/// Inference rule applied by a derivation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceRule {
    /// Rule identifier.
    pub id: Id,
    /// Rule name.
    pub name: String,
    /// Contexts where the rule may be applied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rule_scope_contexts: Vec<Id>,
    /// Interpretation package that owns the rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation_package: Option<Id>,
}

impl InferenceRule {
    /// Creates an inference rule with a validated name.
    pub fn new(id: Id, name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id,
            name: normalize_required_text("inference_rule.name", name)?,
            rule_scope_contexts: Vec::new(),
            interpretation_package: None,
        })
    }

    fn has_scope(&self) -> bool {
        !self.rule_scope_contexts.is_empty() || self.interpretation_package.is_some()
    }
}

/// Structured inference from premises to a conclusion.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Derivation {
    /// Derivation identifier.
    pub id: Id,
    /// Derived conclusion cell.
    pub conclusion: Id,
    /// Premise cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub premises: Vec<Id>,
    /// Inference rule applied.
    pub inference_rule: InferenceRule,
    /// Supporting warrants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warrants: Vec<Id>,
    /// Premises explicitly excluded from the derivation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_premises: Vec<Id>,
    /// Counterexample witnesses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counterexamples: Vec<Id>,
    /// Verifier used to check the derivation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<Verifier>,
    /// Verification state.
    pub verification_status: VerificationStatus,
    /// Known failure mode.
    pub failure_mode: DerivationFailureMode,
    /// Derivation provenance.
    pub provenance: Provenance,
    /// Review lifecycle state.
    pub review_status: LifecycleStatus,
}

impl Derivation {
    /// Creates an unverified derivation candidate.
    pub fn candidate(
        id: Id,
        conclusion: Id,
        premises: Vec<Id>,
        inference_rule: InferenceRule,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            conclusion,
            premises,
            inference_rule,
            warrants: Vec::new(),
            excluded_premises: Vec::new(),
            counterexamples: Vec::new(),
            verifier: None,
            verification_status: VerificationStatus::Unverified,
            failure_mode: DerivationFailureMode::None,
            provenance,
            review_status: LifecycleStatus::Candidate,
        }
    }

    /// Validates conditions required before treating the derivation as accepted.
    pub fn validate_acceptance(&self) -> Result<()> {
        if self
            .premises
            .iter()
            .all(|premise| premise == &self.conclusion)
        {
            return Err(CoreError::malformed_field(
                "premises",
                "derivation cannot be circular over only the conclusion",
            ));
        }
        if !self.inference_rule.has_scope() {
            return Err(CoreError::malformed_field(
                "inference_rule.rule_scope",
                "accepted derivation requires a scoped inference rule",
            ));
        }
        if self.failure_mode != DerivationFailureMode::None {
            return Err(CoreError::malformed_field(
                "failure_mode",
                "accepted derivation requires no unresolved failure mode",
            ));
        }
        if !self.counterexamples.is_empty() {
            return Err(CoreError::malformed_field(
                "counterexamples",
                "accepted derivation cannot have unresolved counterexamples",
            ));
        }
        match self.verification_status {
            VerificationStatus::MachineChecked | VerificationStatus::HumanReviewed => {}
            _ => {
                return Err(CoreError::malformed_field(
                    "verification_status",
                    "accepted derivation requires machine_checked or human_reviewed status",
                ));
            }
        }
        if self.verifier.is_none() && self.provenance.review_status != ReviewStatus::Accepted {
            return Err(CoreError::malformed_field(
                "verifier",
                "accepted derivation requires verifier or explicit accepted review",
            ));
        }
        Ok(())
    }
}
