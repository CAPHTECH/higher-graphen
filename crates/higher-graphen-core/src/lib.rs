//! Shared primitive types and contracts for HigherGraphen.

mod confidence;
mod error;
mod extension;
mod id;
mod provenance;
mod review;
mod source;
mod text;

pub use confidence::Confidence;
pub use error::{CoreError, Result};
pub use extension::{
    Capability, CapabilityOperation, CapabilityStatus, CriterionDirection, CriterionValue,
    Derivation, DerivationFailureMode, Description, EquivalenceClaim, EquivalenceCriterion,
    EquivalenceKind, EquivalenceScope, InferenceRule, LifecycleStatus, ObjectRef, OrderType,
    PayloadRef, Policy, PolicyApplicability, PolicyKind, PolicyRule, PolicyStatus, QuotientEffect,
    Reachability, RequiredReview, ReviewRequirement, Scenario, ScenarioChanges, ScenarioKind,
    ScenarioStatus, SchemaCompatibility, SchemaMapping, SchemaMappingKind, SchemaMorphism,
    SchemaVerification, Tradeoff, ValidityInterval, Valuation, ValuationCriterion, ValuationValue,
    VerificationStatus, Verifier, VerifierKind, Witness, WitnessStatus, WitnessType,
};
pub use id::Id;
pub use provenance::Provenance;
pub use review::{ReviewStatus, Severity};
pub use source::{SourceKind, SourceRef};

#[cfg(test)]
mod tests;
