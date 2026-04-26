use super::*;
use higher_graphen_obstruction::{Obstruction, ObstructionType};
use std::collections::BTreeSet;

/// Input for detecting completion candidates from structured obstructions.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObstructionCompletionInput {
    /// Space in which obstruction-driven completion is being detected.
    pub space_id: Id,
    /// Context identifiers available to the detection workflow.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Structured obstructions to translate into reviewable completion candidates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<Obstruction>,
}

impl ObstructionCompletionInput {
    /// Creates obstruction-driven completion input for a space.
    pub fn new(space_id: Id, obstructions: Vec<Obstruction>) -> Self {
        Self {
            space_id,
            context_ids: Vec::new(),
            obstructions,
        }
    }

    /// Returns this input with context IDs copied into the detection result.
    #[must_use]
    pub fn with_context_ids(mut self, context_ids: Vec<Id>) -> Self {
        self.context_ids = context_ids;
        self
    }
}

/// Detects reviewable completion candidates from structured obstructions.
///
/// Unsupported obstruction kinds are ignored so downstream products can keep
/// domain-specific completion semantics outside this reusable kernel.
pub fn detect_obstruction_completion_candidates(
    input: ObstructionCompletionInput,
) -> Result<CompletionDetectionResult> {
    let ObstructionCompletionInput {
        space_id,
        context_ids,
        obstructions,
    } = input;

    let candidates = obstructions
        .iter()
        .filter_map(
            |obstruction| match obstruction_to_candidate(obstruction, &space_id) {
                Ok(Some(candidate)) => Some(Ok(candidate)),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            },
        )
        .collect::<Result<Vec<_>>>()?;

    CompletionDetectionResult::new(space_id, context_ids, candidates)
}

#[derive(Clone, Copy, Debug)]
struct ObstructionCompletionContract {
    missing_type: MissingType,
    structure_type: &'static str,
    action: &'static str,
    label: &'static str,
}

fn obstruction_to_candidate(
    obstruction: &Obstruction,
    input_space_id: &Id,
) -> Result<Option<CompletionCandidate>> {
    let Some(contract) = obstruction_completion_contract(&obstruction.obstruction_type) else {
        return Ok(None);
    };

    if &obstruction.space_id != input_space_id {
        return Err(malformed_field(
            "obstructions.space_id",
            format!(
                "obstruction {} belongs to {}, but input space is {}",
                obstruction.id, obstruction.space_id, input_space_id
            ),
        ));
    }

    let suggested_structure = SuggestedStructure::new(
        contract.structure_type,
        obstruction_suggestion_summary(obstruction, contract),
    )?
    .with_structure_id(Id::new(format!(
        "{OBSTRUCTION_STRUCTURE_PREFIX}{}",
        obstruction.id
    ))?)
    .with_related_ids(obstruction_related_ids(obstruction));

    CompletionCandidate::new(
        Id::new(format!("{OBSTRUCTION_CANDIDATE_PREFIX}{}", obstruction.id))?,
        input_space_id.clone(),
        contract.missing_type,
        suggested_structure,
        obstruction_inferred_from(obstruction),
        obstruction_rationale(obstruction, contract),
        obstruction.provenance.confidence,
    )
    .map(Some)
}

fn obstruction_completion_contract(
    obstruction_type: &ObstructionType,
) -> Option<ObstructionCompletionContract> {
    match obstruction_type {
        ObstructionType::MissingMorphism => Some(ObstructionCompletionContract {
            missing_type: MissingType::Morphism,
            structure_type: "morphism",
            action: "Add a morphism",
            label: "missing morphism",
        }),
        ObstructionType::FailedGluing => Some(ObstructionCompletionContract {
            missing_type: MissingType::Section,
            structure_type: "gluing_section",
            action: "Add a gluing section",
            label: "failed gluing",
        }),
        ObstructionType::UncoveredRegion => Some(ObstructionCompletionContract {
            missing_type: MissingType::Cell,
            structure_type: "covering_cell",
            action: "Add coverage",
            label: "uncovered region",
        }),
        ObstructionType::ProjectionLoss => Some(ObstructionCompletionContract {
            missing_type: MissingType::Projection,
            structure_type: "lossless_projection",
            action: "Add a lossless projection",
            label: "projection loss",
        }),
        ObstructionType::ContextMismatch => Some(ObstructionCompletionContract {
            missing_type: MissingType::Context,
            structure_type: "context_alignment",
            action: "Add a context alignment",
            label: "context mismatch",
        }),
        ObstructionType::ConstraintUnsatisfied
        | ObstructionType::InvariantViolation
        | ObstructionType::FailedComposition
        | ObstructionType::Custom(_) => None,
    }
}

fn obstruction_suggestion_summary(
    obstruction: &Obstruction,
    contract: ObstructionCompletionContract,
) -> String {
    let resolution = obstruction
        .required_resolution
        .as_ref()
        .map(|resolution| resolution.summary.as_str())
        .unwrap_or(obstruction.explanation.summary.as_str());

    format!(
        "{} to resolve {}: {}",
        contract.action, obstruction.id, resolution
    )
}

fn obstruction_rationale(
    obstruction: &Obstruction,
    contract: ObstructionCompletionContract,
) -> String {
    let mut rationale = format!(
        "{} obstruction {} was recorded in space {}: {}",
        contract.label, obstruction.id, obstruction.space_id, obstruction.explanation.summary
    );

    if let Some(details) = &obstruction.explanation.details {
        rationale.push_str(" Details: ");
        rationale.push_str(details);
    }

    if let Some(required_resolution) = &obstruction.required_resolution {
        rationale.push_str(" Required resolution: ");
        rationale.push_str(&required_resolution.summary);
    }

    if obstruction.has_counterexample() {
        rationale.push_str(" Counterexample available.");
    }

    rationale
}

fn obstruction_related_ids(obstruction: &Obstruction) -> Vec<Id> {
    let mut ids = BTreeSet::new();

    ids.extend(obstruction.location_cell_ids.iter().cloned());
    ids.extend(obstruction.location_context_ids.iter().cloned());
    ids.extend(
        obstruction
            .related_morphisms
            .iter()
            .map(|related_morphism| related_morphism.morphism_id.clone()),
    );

    if let Some(required_resolution) = &obstruction.required_resolution {
        ids.extend(required_resolution.target_cell_ids.iter().cloned());
        ids.extend(required_resolution.target_context_ids.iter().cloned());
        ids.extend(required_resolution.target_morphism_ids.iter().cloned());
    }

    if let Some(counterexample) = &obstruction.counterexample {
        ids.extend(counterexample.path_cell_ids.iter().cloned());
        ids.extend(counterexample.context_ids.iter().cloned());
    }

    ids.into_iter().collect()
}

fn obstruction_inferred_from(obstruction: &Obstruction) -> Vec<Id> {
    let mut ids = BTreeSet::new();

    ids.insert(obstruction.id.clone());
    ids.extend(obstruction_related_ids(obstruction));

    ids.into_iter().collect()
}
