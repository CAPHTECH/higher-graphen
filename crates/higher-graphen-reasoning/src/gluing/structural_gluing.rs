//! Structural gluing through finite pushout construction.

use super::abstract_gluing::safe_id_segment;
use higher_graphen_core::{
    CoreError, GluingResult, Id, PreservationReport, ReviewRequirement, ReviewStatus,
};
use higher_graphen_structure::{
    morphism::{Morphism, PushoutConstruction, PushoutObstructionType, PushoutOutcome},
    space::{ComplexType, InMemorySpaceStore},
};
use std::collections::BTreeSet;

/// Outcome of gluing two concrete structures over a cospan.
#[derive(Clone, Debug, PartialEq)]
pub struct StructuralGluing {
    /// Gluing classification in the gluing vocabulary.
    pub result: GluingResult,
    /// Materialized merged structure, present only when pushout construction succeeded.
    pub construction: Option<PushoutConstruction>,
}

/// Attempts to glue concrete structures by constructing a finite pushout candidate.
///
/// The store is used read-only: the pushout candidate is returned in the result
/// and is never inserted back into the store.
pub fn attempt_structural_gluing(
    left: &Morphism,
    right: &Morphism,
    store: &InMemorySpaceStore,
    candidate_space_id: Id,
    candidate_space_name: String,
    complex_type: ComplexType,
) -> std::result::Result<StructuralGluing, CoreError> {
    let candidate_segment = safe_id_segment(&candidate_space_id);
    let outcome = store.construct_pushout(
        left,
        right,
        candidate_space_id,
        candidate_space_name,
        complex_type,
    )?;

    match outcome {
        PushoutOutcome::Constructed {
            construction,
            report: _,
        } => {
            let construction = *construction;
            match construction.review_status {
                ReviewStatus::Unreviewed | ReviewStatus::Accepted => {
                    let merged_complex = construction.complex.id.clone();
                    let result = GluingResult::Success {
                        merged_complex: Some(merged_complex),
                        preservation_report: preservation_report_from_pushout(&construction),
                    };
                    let structural_gluing = StructuralGluing {
                        result,
                        construction: Some(construction),
                    };
                    debug_assert!(success_honesty_invariant(&structural_gluing));
                    Ok(structural_gluing)
                }
                ReviewStatus::Candidate | ReviewStatus::Reviewed | ReviewStatus::Rejected => {
                    let reason = structural_review_reason(construction.review_status);
                    Ok(StructuralGluing {
                        result: GluingResult::Candidate {
                            completion_candidate: Id::new(format!(
                                "completion:structural-gluing:{candidate_segment}"
                            ))?,
                            required_review: ReviewRequirement::new(true)
                                .with_decision_reason(reason)?,
                        },
                        construction: Some(construction),
                    })
                }
            }
        }
        PushoutOutcome::Blocked { report } => Ok(StructuralGluing {
            result: GluingResult::Failure {
                obstruction: Id::new(format!(
                    "obstruction:pushout:{}:{}",
                    candidate_segment,
                    pushout_obstruction_segment(
                        report
                            .obstructions
                            .first()
                            .map(|obstruction| &obstruction.obstruction_type)
                    )
                ))?,
            },
            construction: None,
        }),
    }
}

fn preservation_report_from_pushout(construction: &PushoutConstruction) -> PreservationReport {
    let preserved_structures = construction
        .complex
        .cell_ids
        .iter()
        .chain(construction.complex.incidence_ids.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    PreservationReport {
        preserved_invariants: Vec::new(),
        preserved_structures,
        summary: Some(format!(
            "constructed pushout complex {} with {} cell(s) and {} incidence(s)",
            construction.complex.id,
            construction.cells.len(),
            construction.incidences.len()
        )),
    }
}

fn structural_review_reason(review_status: ReviewStatus) -> &'static str {
    match review_status {
        ReviewStatus::Candidate => "ambiguous identification requires review",
        ReviewStatus::Reviewed => "reviewed pushout candidate requires explicit acceptance",
        ReviewStatus::Rejected => "rejected pushout candidate cannot be silently glued",
        ReviewStatus::Unreviewed | ReviewStatus::Accepted => {
            "pushout construction does not require review"
        }
    }
}

fn pushout_obstruction_segment(obstruction_type: Option<&PushoutObstructionType>) -> &'static str {
    match obstruction_type {
        Some(PushoutObstructionType::IncompatibleSourceSpace) => "incompatible-source-space",
        Some(PushoutObstructionType::PushoutIncomplete) => "pushout-incomplete",
        Some(PushoutObstructionType::IncompatibleIdentification) => "incompatible-identification",
        Some(PushoutObstructionType::RelationEndpointConflict) => "relation-endpoint-conflict",
        Some(PushoutObstructionType::AmbiguousIdentification) => "ambiguous-identification",
        None => "blocked",
    }
}

pub(super) fn success_honesty_invariant(gluing: &StructuralGluing) -> bool {
    match (&gluing.result, &gluing.construction) {
        (
            GluingResult::Success {
                merged_complex: Some(merged_complex),
                ..
            },
            Some(construction),
        ) => construction.complex.id == *merged_complex,
        (GluingResult::Success { .. }, _) => false,
        _ => true,
    }
}
