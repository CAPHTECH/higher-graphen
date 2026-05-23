//! Projection definitions, selectors, results, and renderers for HigherGraphen.

use crate::{ProjectionAudience, ProjectionPurpose, RendererKind};
use higher_graphen_core::{
    Confidence, CorrespondenceCell, CorrespondenceKind, CorrespondenceParticipant,
    CorrespondencePolarity, DifferenceWitness, GluingAttempt, GluingResult, Id, OverlapWitness,
    ProjectionLoss, ReviewStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// JSON schema id for correspondence explanation output.
pub const CORRESPONDENCE_EXPLANATION_SCHEMA: &str = "highergraphen.correspondence.explanation.v1";
/// JSON schema id for audience-specific correspondence projection output.
pub const CORRESPONDENCE_PROJECTION_SCHEMA: &str = "highergraphen.correspondence.projection.v1";

/// Structured explanation of one correspondence cell.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrespondenceExplanation {
    /// Output schema identifier.
    pub schema: String,
    /// Explained correspondence identifier.
    pub correspondence_id: Id,
    /// Correspondence participants.
    pub participants: Vec<CorrespondenceParticipant>,
    /// Context in which the correspondence is valid.
    pub context: Id,
    /// Overlap witnesses included in the explanation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlap_witnesses: Vec<OverlapWitness>,
    /// Difference witnesses included in the explanation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub difference_witnesses: Vec<DifferenceWitness>,
    /// Evidence identifiers included in the explanation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Id>,
    /// Confidence in the correspondence.
    pub confidence: Confidence,
    /// Review status of the correspondence.
    pub review_status: ReviewStatus,
    /// Optional gluing explanation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gluing: Option<GluingExplanation>,
    /// Declared information loss for this projection.
    pub projection_loss: ProjectionLoss,
}

/// Structured gluing subset shown in a correspondence explanation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GluingExplanation {
    /// Gluing attempt identifier.
    pub id: Id,
    /// Context in which gluing was checked.
    pub context: Id,
    /// Gluing result.
    pub result: GluingResult,
    /// Obstruction identifier when gluing failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obstruction: Option<Id>,
    /// Invariant checks included in the explanation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariant_checks: Vec<higher_graphen_core::InvariantCheckResult>,
    /// Evidence identifiers included for the gluing attempt.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Id>,
    /// Confidence in the gluing attempt.
    pub confidence: Confidence,
    /// Review status of the gluing attempt.
    pub status: ReviewStatus,
}

/// Audience-specific projection of one correspondence cell.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrespondenceProjection {
    /// Output schema identifier.
    pub schema: String,
    /// Explained correspondence identifier.
    pub correspondence_id: Id,
    /// Target audience for the projection.
    pub audience: ProjectionAudience,
    /// Purpose of the projection.
    pub purpose: ProjectionPurpose,
    /// Renderer intended for this projection.
    pub renderer: RendererKind,
    /// Correspondence category.
    pub correspondence_kind: CorrespondenceKind,
    /// Correspondence polarity.
    pub polarity: CorrespondencePolarity,
    /// Correspondence participants.
    pub participants: Vec<CorrespondenceParticipant>,
    /// Context in which the correspondence is valid.
    pub context: Id,
    /// Provenance reference retained for auditability.
    pub provenance: Id,
    /// Overlap witnesses included in the projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlap_witnesses: Vec<OverlapWitness>,
    /// Difference witnesses included in the projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub difference_witnesses: Vec<DifferenceWitness>,
    /// Evidence identifiers included in the projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Id>,
    /// Confidence in the correspondence.
    pub confidence: Confidence,
    /// Review status of the correspondence.
    pub review_status: ReviewStatus,
    /// Optional gluing explanation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gluing: Option<GluingExplanation>,
    /// Declared information loss for this projection.
    pub projection_loss: ProjectionLoss,
}

/// Projects a correspondence cell into a complete structured explanation.
#[must_use]
pub fn explain_correspondence(correspondence: &CorrespondenceCell) -> CorrespondenceExplanation {
    let gluing = correspondence.gluing.as_ref().map(gluing_explanation);
    let explanation = CorrespondenceExplanation {
        schema: CORRESPONDENCE_EXPLANATION_SCHEMA.to_owned(),
        correspondence_id: correspondence.id.clone(),
        participants: correspondence.participants.clone(),
        context: correspondence.context.clone(),
        overlap_witnesses: correspondence.overlap_witnesses.clone(),
        difference_witnesses: correspondence.difference_witnesses.clone(),
        evidence: correspondence.evidence.clone(),
        confidence: correspondence.confidence,
        review_status: correspondence.review_status,
        gluing,
        projection_loss: ProjectionLoss::default(),
    };

    CorrespondenceExplanation {
        projection_loss: projection_loss(correspondence, &explanation),
        ..explanation
    }
}

/// Projects a correspondence for a specific audience and purpose.
#[must_use]
pub fn project_correspondence(
    correspondence: &CorrespondenceCell,
    audience: ProjectionAudience,
    purpose: ProjectionPurpose,
) -> CorrespondenceProjection {
    let renderer = renderer_for_audience(audience);
    let gluing = correspondence.gluing.as_ref().map(gluing_explanation);
    let explanation = CorrespondenceExplanation {
        schema: CORRESPONDENCE_EXPLANATION_SCHEMA.to_owned(),
        correspondence_id: correspondence.id.clone(),
        participants: correspondence.participants.clone(),
        context: correspondence.context.clone(),
        overlap_witnesses: correspondence.overlap_witnesses.clone(),
        difference_witnesses: correspondence.difference_witnesses.clone(),
        evidence: correspondence.evidence.clone(),
        confidence: correspondence.confidence,
        review_status: correspondence.review_status,
        gluing: gluing.clone(),
        projection_loss: ProjectionLoss::default(),
    };
    let projection_loss = projection_loss(correspondence, &explanation);

    CorrespondenceProjection {
        schema: CORRESPONDENCE_PROJECTION_SCHEMA.to_owned(),
        correspondence_id: correspondence.id.clone(),
        audience,
        purpose,
        renderer,
        correspondence_kind: correspondence.correspondence_kind,
        polarity: correspondence.polarity,
        participants: correspondence.participants.clone(),
        context: correspondence.context.clone(),
        provenance: correspondence.provenance.clone(),
        overlap_witnesses: correspondence.overlap_witnesses.clone(),
        difference_witnesses: correspondence.difference_witnesses.clone(),
        evidence: correspondence.evidence.clone(),
        confidence: correspondence.confidence,
        review_status: correspondence.review_status,
        gluing,
        projection_loss,
    }
}

/// Renders a correspondence projection as Markdown for CLI and human review.
#[must_use]
pub fn render_correspondence_projection_markdown(projection: &CorrespondenceProjection) -> String {
    let mut text = String::new();
    push_projection_summary(&mut text, projection);
    push_participants(&mut text, projection);
    push_overlap_witnesses(&mut text, projection);
    push_difference_witnesses(&mut text, projection);
    push_evidence(&mut text, projection);
    push_gluing(&mut text, projection);
    push_loss(&mut text, projection);

    text
}

fn push_projection_summary(text: &mut String, projection: &CorrespondenceProjection) {
    push_line(
        text,
        &format!("# Correspondence {}", projection.correspondence_id),
    );
    push_line(
        text,
        &format!("- Audience: {}", audience_label(projection.audience)),
    );
    push_line(
        text,
        &format!("- Purpose: {}", purpose_label(projection.purpose)),
    );
    push_line(
        text,
        &format!("- Kind: {:?}", projection.correspondence_kind),
    );
    push_line(text, &format!("- Polarity: {:?}", projection.polarity));
    push_line(text, &format!("- Context: {}", projection.context));
    push_line(text, &format!("- Provenance: {}", projection.provenance));
    push_line(
        text,
        &format!("- Review status: {}", projection.review_status),
    );
    push_line(text, &format!("- Confidence: {}", projection.confidence));
}

fn push_participants(text: &mut String, projection: &CorrespondenceProjection) {
    push_line(text, "");
    push_line(text, "## Participants");
    for participant in &projection.participants {
        push_line(
            text,
            &format!("- {}: {}", participant.role, participant.participant.id()),
        );
    }
}

fn push_overlap_witnesses(text: &mut String, projection: &CorrespondenceProjection) {
    push_line(text, "");
    push_line(text, "## Shared Structure");
    if projection.overlap_witnesses.is_empty() {
        push_line(text, "- none");
    } else {
        for witness in &projection.overlap_witnesses {
            push_line(
                text,
                &format!(
                    "- {} ({:?}, status {}, confidence {}, context {})",
                    witness.id,
                    witness.witness_kind,
                    witness.status,
                    witness.confidence,
                    witness.context
                ),
            );
            push_line(
                text,
                &format!("  - structure: {}", compact_json(&witness.shared_structure)),
            );
            push_line(
                text,
                &format!("  - evidence: {}", id_list(&witness.evidence)),
            );
        }
    }
}

fn push_difference_witnesses(text: &mut String, projection: &CorrespondenceProjection) {
    push_line(text, "");
    push_line(text, "## Difference");
    if projection.difference_witnesses.is_empty() {
        push_line(text, "- none");
    } else {
        for witness in &projection.difference_witnesses {
            push_line(
                text,
                &format!(
                    "- {} ({:?}, severity {:?}, status {}, confidence {}, context {})",
                    witness.id,
                    witness.difference_kind,
                    witness.severity,
                    witness.status,
                    witness.confidence,
                    witness.context
                ),
            );
            push_line(
                text,
                &format!(
                    "  - structure: {}",
                    compact_json(&witness.differing_structure)
                ),
            );
            push_line(
                text,
                &format!("  - evidence: {}", id_list(&witness.evidence)),
            );
        }
    }
}

fn push_evidence(text: &mut String, projection: &CorrespondenceProjection) {
    push_line(text, "");
    push_line(text, "## Evidence");
    push_line(text, &format!("- {}", id_list(&projection.evidence)));
}

fn push_gluing(text: &mut String, projection: &CorrespondenceProjection) {
    push_line(text, "");
    push_line(text, "## Gluing");
    if let Some(gluing) = &projection.gluing {
        push_line(text, &format!("- Attempt: {}", gluing.id));
        push_line(
            text,
            &format!("- Result: {}", gluing_result_label(&gluing.result)),
        );
        if let Some(obstruction) = &gluing.obstruction {
            push_line(text, &format!("- Obstruction: {obstruction}"));
        }
        push_line(text, &format!("- Context: {}", gluing.context));
        push_line(text, &format!("- Status: {}", gluing.status));
        push_line(text, &format!("- Confidence: {}", gluing.confidence));
        if !gluing.invariant_checks.is_empty() {
            push_invariant_checks(text, gluing);
        }
    } else {
        push_line(text, "- not attempted");
    }
}

fn push_invariant_checks(text: &mut String, gluing: &GluingExplanation) {
    push_line(text, "- Invariant checks:");
    for check in &gluing.invariant_checks {
        let detail = check
            .detail
            .as_deref()
            .map(|value| format!(" ({value})"))
            .unwrap_or_default();
        push_line(
            text,
            &format!("  - {}: {}{}", check.invariant, check.result, detail),
        );
    }
}

fn push_loss(text: &mut String, projection: &CorrespondenceProjection) {
    push_line(text, "");
    push_line(text, "## Projection Loss");
    push_projection_loss(text, &projection.projection_loss);
}

fn gluing_explanation(gluing: &GluingAttempt) -> GluingExplanation {
    GluingExplanation {
        id: gluing.id.clone(),
        context: gluing.context.clone(),
        result: gluing.result.clone(),
        obstruction: match &gluing.result {
            GluingResult::Failure { obstruction } => Some(obstruction.clone()),
            GluingResult::Success { .. } | GluingResult::Candidate { .. } => None,
        },
        invariant_checks: gluing.invariant_checks.clone(),
        evidence: gluing.evidence.clone(),
        confidence: gluing.confidence,
        status: gluing.status,
    }
}

fn projection_loss(
    correspondence: &CorrespondenceCell,
    explanation: &CorrespondenceExplanation,
) -> ProjectionLoss {
    ProjectionLoss {
        omitted_overlap_witnesses: omitted_overlap_witnesses(correspondence, explanation),
        omitted_difference_witnesses: omitted_difference_witnesses(correspondence, explanation),
        omitted_evidence: omitted_evidence(correspondence, explanation),
        omitted_contexts: omitted_contexts(correspondence, explanation),
        collapsed_statuses: Vec::new(),
    }
}

fn omitted_overlap_witnesses(
    correspondence: &CorrespondenceCell,
    explanation: &CorrespondenceExplanation,
) -> Vec<Id> {
    let included = explanation
        .overlap_witnesses
        .iter()
        .map(|witness| witness.id.clone())
        .collect::<BTreeSet<_>>();
    correspondence
        .overlap_witnesses
        .iter()
        .filter(|witness| !included.contains(&witness.id))
        .map(|witness| witness.id.clone())
        .collect()
}

fn omitted_difference_witnesses(
    correspondence: &CorrespondenceCell,
    explanation: &CorrespondenceExplanation,
) -> Vec<Id> {
    let included = explanation
        .difference_witnesses
        .iter()
        .map(|witness| witness.id.clone())
        .collect::<BTreeSet<_>>();
    correspondence
        .difference_witnesses
        .iter()
        .filter(|witness| !included.contains(&witness.id))
        .map(|witness| witness.id.clone())
        .collect()
}

fn omitted_evidence(
    correspondence: &CorrespondenceCell,
    explanation: &CorrespondenceExplanation,
) -> Vec<Id> {
    let included = explanation_evidence(explanation);
    correspondence_evidence(correspondence)
        .into_iter()
        .filter(|evidence| !included.contains(evidence))
        .collect()
}

fn explanation_evidence(explanation: &CorrespondenceExplanation) -> BTreeSet<Id> {
    explanation
        .evidence
        .iter()
        .chain(
            explanation
                .overlap_witnesses
                .iter()
                .flat_map(|witness| witness.evidence.iter()),
        )
        .chain(
            explanation
                .difference_witnesses
                .iter()
                .flat_map(|witness| witness.evidence.iter()),
        )
        .chain(
            explanation
                .gluing
                .iter()
                .flat_map(|gluing| gluing.evidence.iter()),
        )
        .cloned()
        .collect()
}

fn correspondence_evidence(correspondence: &CorrespondenceCell) -> Vec<Id> {
    correspondence
        .evidence
        .iter()
        .chain(
            correspondence
                .overlap_witnesses
                .iter()
                .flat_map(|witness| witness.evidence.iter()),
        )
        .chain(
            correspondence
                .difference_witnesses
                .iter()
                .flat_map(|witness| witness.evidence.iter()),
        )
        .chain(
            correspondence
                .gluing
                .iter()
                .flat_map(|gluing| gluing.evidence.iter()),
        )
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn omitted_contexts(
    correspondence: &CorrespondenceCell,
    explanation: &CorrespondenceExplanation,
) -> Vec<Id> {
    let included = explanation_contexts(explanation);
    correspondence_contexts(correspondence)
        .into_iter()
        .filter(|context| !included.contains(context))
        .collect()
}

fn explanation_contexts(explanation: &CorrespondenceExplanation) -> BTreeSet<Id> {
    std::iter::once(&explanation.context)
        .chain(
            explanation
                .overlap_witnesses
                .iter()
                .map(|witness| &witness.context),
        )
        .chain(
            explanation
                .difference_witnesses
                .iter()
                .map(|witness| &witness.context),
        )
        .chain(explanation.gluing.iter().map(|gluing| &gluing.context))
        .cloned()
        .collect()
}

fn correspondence_contexts(correspondence: &CorrespondenceCell) -> Vec<Id> {
    std::iter::once(&correspondence.context)
        .chain(
            correspondence
                .overlap_witnesses
                .iter()
                .map(|witness| &witness.context),
        )
        .chain(
            correspondence
                .difference_witnesses
                .iter()
                .map(|witness| &witness.context),
        )
        .chain(correspondence.gluing.iter().map(|gluing| &gluing.context))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn renderer_for_audience(audience: ProjectionAudience) -> RendererKind {
    match audience {
        ProjectionAudience::Human
        | ProjectionAudience::Developer
        | ProjectionAudience::Architect
        | ProjectionAudience::Executive
        | ProjectionAudience::Operator => RendererKind::Markdown,
        ProjectionAudience::Ai | ProjectionAudience::AiAgent | ProjectionAudience::Audit => {
            RendererKind::Structured
        }
        ProjectionAudience::ExternalSystem => RendererKind::Structured,
    }
}

fn push_line(text: &mut String, line: &str) {
    text.push_str(line);
    text.push('\n');
}

fn compact_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unrenderable>".to_owned())
}

fn id_list(ids: &[Id]) -> String {
    if ids.is_empty() {
        "none".to_owned()
    } else {
        ids.iter().map(Id::as_str).collect::<Vec<_>>().join(", ")
    }
}

fn gluing_result_label(result: &GluingResult) -> &'static str {
    match result {
        GluingResult::Success { .. } => "success",
        GluingResult::Candidate { .. } => "candidate",
        GluingResult::Failure { .. } => "failure",
    }
}

fn push_projection_loss(text: &mut String, loss: &ProjectionLoss) {
    let mut has_loss = false;

    if !loss.omitted_overlap_witnesses.is_empty() {
        has_loss = true;
        push_line(
            text,
            &format!(
                "- omitted overlap witnesses: {}",
                id_list(&loss.omitted_overlap_witnesses)
            ),
        );
    }
    if !loss.omitted_difference_witnesses.is_empty() {
        has_loss = true;
        push_line(
            text,
            &format!(
                "- omitted difference witnesses: {}",
                id_list(&loss.omitted_difference_witnesses)
            ),
        );
    }
    if !loss.omitted_evidence.is_empty() {
        has_loss = true;
        push_line(
            text,
            &format!("- omitted evidence: {}", id_list(&loss.omitted_evidence)),
        );
    }
    if !loss.omitted_contexts.is_empty() {
        has_loss = true;
        push_line(
            text,
            &format!("- omitted contexts: {}", id_list(&loss.omitted_contexts)),
        );
    }
    if !loss.collapsed_statuses.is_empty() {
        has_loss = true;
        push_line(
            text,
            &format!("- collapsed statuses: {}", loss.collapsed_statuses.len()),
        );
    }

    if !has_loss {
        push_line(text, "- none");
    }
}

fn audience_label(audience: ProjectionAudience) -> &'static str {
    match audience {
        ProjectionAudience::Human => "human",
        ProjectionAudience::Ai => "ai",
        ProjectionAudience::AiAgent => "ai-agent",
        ProjectionAudience::Audit => "audit",
        ProjectionAudience::Developer => "developer",
        ProjectionAudience::Architect => "architect",
        ProjectionAudience::Executive => "executive",
        ProjectionAudience::Operator => "operator",
        ProjectionAudience::ExternalSystem => "external-system",
    }
}

fn purpose_label(purpose: ProjectionPurpose) -> &'static str {
    match purpose {
        ProjectionPurpose::Explanation => "explanation",
        ProjectionPurpose::Report => "report",
        ProjectionPurpose::Dashboard => "dashboard",
        ProjectionPurpose::ActionPlan => "action-plan",
        ProjectionPurpose::Review => "review",
        ProjectionPurpose::QueryResult => "query-result",
        ProjectionPurpose::ApiResponse => "api-response",
    }
}
