use super::{CaseGraphenCoreExtensions, CoreExtensionValidation, CoreExtensionValidationFinding};
use higher_graphen_core::{Id, LifecycleStatus, PolicyStatus, ScenarioStatus, WitnessStatus};

pub(crate) fn finalize_validation(extensions: &mut CaseGraphenCoreExtensions) {
    extensions.validation = CoreExtensionValidation::default();
    extensions.validation.generated_count = extensions.witnesses.len()
        + extensions.derivations.len()
        + extensions.policies.len()
        + extensions.capabilities.len()
        + extensions.scenarios.len()
        + extensions.schema_morphisms.len()
        + extensions.equivalence_claims.len()
        + extensions.valuations.len();

    for witness in &extensions.witnesses {
        if witness.review_status == WitnessStatus::Accepted {
            push_validation_result(
                &mut extensions.validation,
                &witness.id,
                "witness",
                witness.validate_acceptance(),
            );
        }
    }
    for derivation in &extensions.derivations {
        if derivation.review_status == LifecycleStatus::Accepted {
            push_validation_result(
                &mut extensions.validation,
                &derivation.id,
                "derivation",
                derivation.validate_acceptance(),
            );
        }
    }
    for policy in &extensions.policies {
        if policy.status == PolicyStatus::Active {
            push_validation_result(
                &mut extensions.validation,
                &policy.id,
                "policy",
                policy.validate_active(),
            );
        }
    }
    for scenario in &extensions.scenarios {
        if scenario.status == ScenarioStatus::Accepted {
            push_validation_result(
                &mut extensions.validation,
                &scenario.id,
                "scenario",
                scenario.validate_acceptance(),
            );
        }
    }
    for schema_morphism in &extensions.schema_morphisms {
        push_validation_result(
            &mut extensions.validation,
            &schema_morphism.id,
            "schema_morphism",
            schema_morphism.validate_application(),
        );
    }
    for claim in &extensions.equivalence_claims {
        if claim.status == LifecycleStatus::Accepted {
            push_validation_result(
                &mut extensions.validation,
                &claim.id,
                "equivalence_claim",
                claim.validate_acceptance(),
            );
        }
    }
    for valuation in &extensions.valuations {
        push_validation_result(
            &mut extensions.validation,
            &valuation.id,
            "valuation",
            valuation.validate_for_decision(),
        );
    }
}

fn push_validation_result(
    validation: &mut CoreExtensionValidation,
    object_id: &Id,
    object_type: &str,
    result: higher_graphen_core::Result<()>,
) {
    match result {
        Ok(()) => validation.accepted_ready_count += 1,
        Err(error) => {
            validation.blocked_count += 1;
            validation.findings.push(CoreExtensionValidationFinding {
                object_id: object_id.clone(),
                object_type: object_type.to_owned(),
                status: "blocked".to_owned(),
                message: error.to_string(),
            });
        }
    }
}
