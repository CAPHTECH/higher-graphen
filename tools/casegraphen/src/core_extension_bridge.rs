mod native;
mod support;
mod validation;
mod workflow;

pub use native::{
    native_close_check_extensions, native_close_check_result, native_morphism_check_extensions,
    native_morphism_check_result,
};
pub use workflow::workflow_reason_extensions;

use higher_graphen_core::{
    Capability, Derivation, EquivalenceClaim, Policy, Scenario, SchemaMorphism, Valuation, Witness,
};
use serde::{Deserialize, Serialize};

pub(crate) use support::*;
pub(crate) use validation::finalize_validation;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseGraphenCoreExtensions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witnesses: Vec<Witness>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derivations: Vec<Derivation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<Policy>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<Capability>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scenarios: Vec<Scenario>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_morphisms: Vec<SchemaMorphism>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equivalence_claims: Vec<EquivalenceClaim>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub valuations: Vec<Valuation>,
    #[serde(default)]
    pub validation: CoreExtensionValidation,
}

impl CaseGraphenCoreExtensions {
    pub fn is_empty(&self) -> bool {
        self.witnesses.is_empty()
            && self.derivations.is_empty()
            && self.policies.is_empty()
            && self.capabilities.is_empty()
            && self.scenarios.is_empty()
            && self.schema_morphisms.is_empty()
            && self.equivalence_claims.is_empty()
            && self.valuations.is_empty()
            && self.validation.findings.is_empty()
    }

    pub fn is_blocked(&self) -> bool {
        self.validation.blocked_count > 0
    }

    pub(crate) fn append(&mut self, mut other: Self) {
        self.witnesses.append(&mut other.witnesses);
        self.derivations.append(&mut other.derivations);
        self.policies.append(&mut other.policies);
        self.capabilities.append(&mut other.capabilities);
        self.scenarios.append(&mut other.scenarios);
        self.schema_morphisms.append(&mut other.schema_morphisms);
        self.equivalence_claims
            .append(&mut other.equivalence_claims);
        self.valuations.append(&mut other.valuations);
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoreExtensionValidation {
    pub generated_count: usize,
    pub accepted_ready_count: usize,
    pub blocked_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<CoreExtensionValidationFinding>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoreExtensionValidationFinding {
    pub object_id: higher_graphen_core::Id,
    pub object_type: String,
    pub status: String,
    pub message: String,
}
