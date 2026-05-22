//! Projection definitions, selectors, results, and renderers for HigherGraphen.

use higher_graphen_core::{
    Confidence, CoreError, CorrespondenceCell, CorrespondenceKind, CorrespondenceParticipant,
    CorrespondencePolarity, DifferenceWitness, GluingAttempt, GluingResult, Id, OverlapWitness,
    ProjectionLoss, Result, ReviewStatus, Severity,
};
use serde::{Deserialize, Deserializer, Serialize};
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
    push_line(
        &mut text,
        &format!("# Correspondence {}", projection.correspondence_id),
    );
    push_line(
        &mut text,
        &format!("- Audience: {}", audience_label(projection.audience)),
    );
    push_line(
        &mut text,
        &format!("- Purpose: {}", purpose_label(projection.purpose)),
    );
    push_line(
        &mut text,
        &format!("- Kind: {:?}", projection.correspondence_kind),
    );
    push_line(&mut text, &format!("- Polarity: {:?}", projection.polarity));
    push_line(&mut text, &format!("- Context: {}", projection.context));
    push_line(
        &mut text,
        &format!("- Provenance: {}", projection.provenance),
    );
    push_line(
        &mut text,
        &format!("- Review status: {}", projection.review_status),
    );
    push_line(
        &mut text,
        &format!("- Confidence: {}", projection.confidence),
    );

    push_line(&mut text, "");
    push_line(&mut text, "## Participants");
    for participant in &projection.participants {
        push_line(
            &mut text,
            &format!("- {}: {}", participant.role, participant.participant.id()),
        );
    }

    push_line(&mut text, "");
    push_line(&mut text, "## Shared Structure");
    if projection.overlap_witnesses.is_empty() {
        push_line(&mut text, "- none");
    } else {
        for witness in &projection.overlap_witnesses {
            push_line(
                &mut text,
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
                &mut text,
                &format!("  - structure: {}", compact_json(&witness.shared_structure)),
            );
            push_line(
                &mut text,
                &format!("  - evidence: {}", id_list(&witness.evidence)),
            );
        }
    }

    push_line(&mut text, "");
    push_line(&mut text, "## Difference");
    if projection.difference_witnesses.is_empty() {
        push_line(&mut text, "- none");
    } else {
        for witness in &projection.difference_witnesses {
            push_line(
                &mut text,
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
                &mut text,
                &format!(
                    "  - structure: {}",
                    compact_json(&witness.differing_structure)
                ),
            );
            push_line(
                &mut text,
                &format!("  - evidence: {}", id_list(&witness.evidence)),
            );
        }
    }

    push_line(&mut text, "");
    push_line(&mut text, "## Evidence");
    push_line(&mut text, &format!("- {}", id_list(&projection.evidence)));

    push_line(&mut text, "");
    push_line(&mut text, "## Gluing");
    if let Some(gluing) = &projection.gluing {
        push_line(&mut text, &format!("- Attempt: {}", gluing.id));
        push_line(
            &mut text,
            &format!("- Result: {}", gluing_result_label(&gluing.result)),
        );
        if let Some(obstruction) = &gluing.obstruction {
            push_line(&mut text, &format!("- Obstruction: {obstruction}"));
        }
        push_line(&mut text, &format!("- Context: {}", gluing.context));
        push_line(&mut text, &format!("- Status: {}", gluing.status));
        push_line(&mut text, &format!("- Confidence: {}", gluing.confidence));
        if !gluing.invariant_checks.is_empty() {
            push_line(&mut text, "- Invariant checks:");
            for check in &gluing.invariant_checks {
                let detail = check
                    .detail
                    .as_deref()
                    .map(|value| format!(" ({value})"))
                    .unwrap_or_default();
                push_line(
                    &mut text,
                    &format!("  - {}: {}{}", check.invariant, check.result, detail),
                );
            }
        }
    } else {
        push_line(&mut text, "- not attempted");
    }

    push_line(&mut text, "");
    push_line(&mut text, "## Projection Loss");
    push_projection_loss(&mut text, &projection.projection_loss);

    text
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
    let included_overlap_witnesses = explanation
        .overlap_witnesses
        .iter()
        .map(|witness| witness.id.clone())
        .collect::<BTreeSet<_>>();
    let omitted_overlap_witnesses = correspondence
        .overlap_witnesses
        .iter()
        .filter(|witness| !included_overlap_witnesses.contains(&witness.id))
        .map(|witness| witness.id.clone())
        .collect();

    let included_difference_witnesses = explanation
        .difference_witnesses
        .iter()
        .map(|witness| witness.id.clone())
        .collect::<BTreeSet<_>>();
    let omitted_difference_witnesses = correspondence
        .difference_witnesses
        .iter()
        .filter(|witness| !included_difference_witnesses.contains(&witness.id))
        .map(|witness| witness.id.clone())
        .collect();

    let included_evidence = explanation
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
        .collect::<BTreeSet<_>>();
    let omitted_evidence = correspondence
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
        .filter(|evidence| !included_evidence.contains(*evidence))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let included_contexts = std::iter::once(&explanation.context)
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
        .collect::<BTreeSet<_>>();
    let omitted_contexts = std::iter::once(&correspondence.context)
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
        .filter(|context| !included_contexts.contains(*context))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    ProjectionLoss {
        omitted_overlap_witnesses,
        omitted_difference_witnesses,
        omitted_evidence,
        omitted_contexts,
        collapsed_statuses: Vec::new(),
    }
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

/// Target consumer for a projected view.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionAudience {
    /// A human reader.
    Human,
    /// A general AI model.
    Ai,
    /// An AI agent consuming source-stable records.
    AiAgent,
    /// An audit or traceability consumer.
    Audit,
    /// A software developer.
    Developer,
    /// An architecture or design reviewer.
    Architect,
    /// An executive or strategy stakeholder.
    Executive,
    /// An operations or reliability stakeholder.
    Operator,
    /// Another system consuming projection data.
    ExternalSystem,
}

/// Intended use of a projected view.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPurpose {
    /// Explain selected structure.
    Explanation,
    /// Produce a report.
    Report,
    /// Produce dashboard-ready data without UI concerns.
    Dashboard,
    /// Produce an action plan.
    ActionPlan,
    /// Support a review workflow.
    Review,
    /// Return selected query results.
    QueryResult,
    /// Produce data for an API layer without encoding transport details.
    ApiResponse,
}

/// Transport-neutral output shape for a projection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutputSchema {
    /// Unstructured text output.
    Text,
    /// Sectioned output.
    Sections {
        /// Stable section names expected in the output.
        section_names: Vec<String>,
    },
    /// Tabular output.
    Table {
        /// Stable column names expected in each row.
        columns: Vec<String>,
    },
    /// Key-value output.
    KeyValue {
        /// Stable keys expected in the output.
        keys: Vec<String>,
    },
    /// Downstream-owned schema name and fields.
    Custom {
        /// Stable schema name.
        name: String,
        /// Stable field names in the custom schema.
        fields: Vec<String>,
    },
}

impl OutputSchema {
    /// Creates a text output schema.
    pub fn text() -> Self {
        Self::Text
    }

    /// Creates a sectioned output schema with non-empty section names.
    pub fn sections<I, S>(section_names: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Ok(Self::Sections {
            section_names: collect_non_empty_texts(section_names, "section_names")?,
        })
    }

    /// Creates a tabular output schema with non-empty column names.
    pub fn table<I, S>(columns: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Ok(Self::Table {
            columns: collect_non_empty_texts(columns, "columns")?,
        })
    }

    /// Creates a key-value output schema with non-empty keys.
    pub fn key_value<I, S>(keys: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Ok(Self::KeyValue {
            keys: collect_non_empty_texts(keys, "keys")?,
        })
    }

    /// Creates a custom output schema with a non-empty name and field list.
    pub fn custom<I, S>(name: impl Into<String>, fields: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Ok(Self::Custom {
            name: normalized_text(name, "name")?,
            fields: collect_non_empty_texts(fields, "fields")?,
        })
    }
}

/// Declared information loss caused by a projection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InformationLoss {
    /// Human-readable description of what was omitted, summarized, or simplified.
    pub description: String,
    /// Source structure identifiers affected by this loss declaration.
    pub source_ids: Vec<Id>,
}

impl InformationLoss {
    /// Creates a loss declaration with explicit source identifiers.
    pub fn declared<I>(description: impl Into<String>, source_ids: I) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
    {
        Ok(Self {
            description: normalized_text(description, "description")?,
            source_ids: collect_non_empty_ids(source_ids, "source_ids")?,
        })
    }

    /// Returns the source identifiers affected by this loss declaration.
    pub fn source_ids(&self) -> &[Id] {
        &self.source_ids
    }
}

/// Transport-neutral renderer category used to produce projection output.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RendererKind {
    /// Plain text renderer.
    PlainText,
    /// Markdown renderer.
    Markdown,
    /// Table renderer.
    Table,
    /// Structured data renderer.
    Structured,
    /// Downstream-owned renderer.
    Custom(String),
}

impl RendererKind {
    /// Creates a custom renderer kind with a non-empty name.
    pub fn custom(name: impl Into<String>) -> Result<Self> {
        Ok(Self::Custom(normalized_text(name, "name")?))
    }
}

/// Selector describing which source structures a projection reads.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionSelector {
    /// Selected cell identifiers. Empty means no cell-id filter.
    pub cell_ids: Vec<Id>,
    /// Selected cell type identifiers. Empty means no cell-type filter.
    pub cell_type_ids: Vec<Id>,
    /// Selected obstruction identifiers. Empty means no obstruction-id filter.
    pub obstruction_ids: Vec<Id>,
    /// Selected obstruction type identifiers. Empty means no obstruction-type filter.
    pub obstruction_type_ids: Vec<Id>,
    /// Selected context identifiers. Empty means no context filter.
    pub context_ids: Vec<Id>,
    /// Minimum severity for selected structures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_severity: Option<Severity>,
}

impl ProjectionSelector {
    /// Creates a selector with no filters.
    pub fn all() -> Self {
        Self::default()
    }

    /// Returns this selector with explicit cell identifiers.
    pub fn with_cell_ids<I>(mut self, cell_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.cell_ids = cell_ids.into_iter().collect();
        self
    }

    /// Returns this selector with explicit cell type identifiers.
    pub fn with_cell_type_ids<I>(mut self, cell_type_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.cell_type_ids = cell_type_ids.into_iter().collect();
        self
    }

    /// Returns this selector with explicit obstruction identifiers.
    pub fn with_obstruction_ids<I>(mut self, obstruction_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.obstruction_ids = obstruction_ids.into_iter().collect();
        self
    }

    /// Returns this selector with explicit obstruction type identifiers.
    pub fn with_obstruction_type_ids<I>(mut self, obstruction_type_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.obstruction_type_ids = obstruction_type_ids.into_iter().collect();
        self
    }

    /// Returns this selector with explicit context identifiers.
    pub fn with_context_ids<I>(mut self, context_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.context_ids = context_ids.into_iter().collect();
        self
    }

    /// Returns this selector with a minimum severity filter.
    pub fn with_min_severity(mut self, min_severity: Severity) -> Self {
        self.min_severity = Some(min_severity);
        self
    }
}

/// Projection definition connecting source structure to an audience-specific view.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    /// Projection identifier.
    pub id: Id,
    /// Source space identifier.
    pub source_space_id: Id,
    /// Human-readable projection name.
    pub name: String,
    /// Target audience.
    pub audience: ProjectionAudience,
    /// Projection purpose.
    pub purpose: ProjectionPurpose,
    /// Source structure selector.
    pub input_selector: ProjectionSelector,
    /// Expected output shape.
    pub output_schema: OutputSchema,
    /// Declared information loss for this projection.
    pub information_loss: Vec<InformationLoss>,
    /// Optional transport-neutral renderer choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renderer: Option<RendererKind>,
}

impl Projection {
    /// Creates a projection with explicit information-loss declarations.
    #[allow(clippy::too_many_arguments)]
    pub fn new<I>(
        id: Id,
        source_space_id: Id,
        name: impl Into<String>,
        audience: ProjectionAudience,
        purpose: ProjectionPurpose,
        input_selector: ProjectionSelector,
        output_schema: OutputSchema,
        information_loss: I,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = InformationLoss>,
    {
        Ok(Self {
            id,
            source_space_id,
            name: normalized_text(name, "name")?,
            audience,
            purpose,
            input_selector,
            output_schema,
            information_loss: collect_non_empty_information_loss(information_loss)?,
            renderer: None,
        })
    }

    /// Returns this projection with a renderer choice.
    pub fn with_renderer(mut self, renderer: RendererKind) -> Self {
        self.renderer = Some(renderer);
        self
    }

    /// Returns the declared information-loss entries.
    pub fn information_loss(&self) -> &[InformationLoss] {
        &self.information_loss
    }
}

/// Transport-neutral projected output.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectionOutput {
    /// Plain text output.
    Text {
        /// Rendered text.
        text: String,
    },
    /// Sectioned output.
    Sections {
        /// Rendered sections.
        sections: Vec<ProjectionSection>,
    },
    /// Tabular output.
    Table {
        /// Stable column names.
        columns: Vec<String>,
        /// Rendered rows aligned to `columns`.
        rows: Vec<Vec<String>>,
    },
    /// Key-value output.
    KeyValue {
        /// Rendered entries.
        entries: Vec<ProjectionEntry>,
    },
}

impl ProjectionOutput {
    /// Creates plain text output.
    pub fn text(text: impl Into<String>) -> Result<Self> {
        Ok(Self::Text {
            text: normalized_text(text, "text")?,
        })
    }

    /// Creates sectioned output.
    pub fn sections<I>(sections: I) -> Result<Self>
    where
        I: IntoIterator<Item = ProjectionSection>,
    {
        Ok(Self::Sections {
            sections: collect_non_empty_items(sections, "sections")?,
        })
    }

    /// Creates table output.
    pub fn table<I, S>(columns: I, rows: Vec<Vec<String>>) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let columns = collect_non_empty_texts(columns, "columns")?;
        for row in &rows {
            if row.len() != columns.len() {
                return Err(malformed_field(
                    "rows",
                    "each row must have the same number of values as columns",
                ));
            }
        }

        Ok(Self::Table { columns, rows })
    }

    /// Creates key-value output.
    pub fn key_value<I>(entries: I) -> Result<Self>
    where
        I: IntoIterator<Item = ProjectionEntry>,
    {
        Ok(Self::KeyValue {
            entries: collect_non_empty_items(entries, "entries")?,
        })
    }
}

/// A section in a projected output.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionSection {
    /// Section title.
    pub title: String,
    /// Section body.
    pub body: String,
    /// Source identifiers represented by this section.
    pub source_ids: Vec<Id>,
}

impl ProjectionSection {
    /// Creates a section with explicit source identifiers.
    pub fn new<I>(title: impl Into<String>, body: impl Into<String>, source_ids: I) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
    {
        Ok(Self {
            title: normalized_text(title, "title")?,
            body: normalized_text(body, "body")?,
            source_ids: collect_non_empty_ids(source_ids, "source_ids")?,
        })
    }

    /// Returns the source identifiers represented by this section.
    pub fn source_ids(&self) -> &[Id] {
        &self.source_ids
    }
}

/// A key-value entry in a projected output.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionEntry {
    /// Entry key.
    pub key: String,
    /// Entry value.
    pub value: String,
    /// Source identifiers represented by this entry.
    pub source_ids: Vec<Id>,
}

impl ProjectionEntry {
    /// Creates an entry with explicit source identifiers.
    pub fn new<I>(key: impl Into<String>, value: impl Into<String>, source_ids: I) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
    {
        Ok(Self {
            key: normalized_text(key, "key")?,
            value: normalized_text(value, "value")?,
            source_ids: collect_non_empty_ids(source_ids, "source_ids")?,
        })
    }

    /// Returns the source identifiers represented by this entry.
    pub fn source_ids(&self) -> &[Id] {
        &self.source_ids
    }
}

/// Result of applying a projection.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionResult {
    /// Projection identifier used to produce this result.
    projection_id: Id,
    /// Target audience copied from the projection definition.
    audience: ProjectionAudience,
    /// Purpose copied from the projection definition.
    purpose: ProjectionPurpose,
    /// Output schema used for this result.
    output_schema: OutputSchema,
    /// Renderer used to produce the output.
    renderer: RendererKind,
    /// Transport-neutral rendered output.
    output: ProjectionOutput,
    /// Source identifiers actually represented in the output.
    source_ids: Vec<Id>,
    /// Information-loss declarations that apply to the output.
    information_loss: Vec<InformationLoss>,
}

impl ProjectionResult {
    /// Creates a result with explicit source identifiers and information loss.
    #[allow(clippy::too_many_arguments)]
    pub fn new<I, L>(
        projection_id: Id,
        audience: ProjectionAudience,
        purpose: ProjectionPurpose,
        output_schema: OutputSchema,
        renderer: RendererKind,
        output: ProjectionOutput,
        source_ids: I,
        information_loss: L,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
        L: IntoIterator<Item = InformationLoss>,
    {
        ensure_output_matches_schema(&output_schema, &output)?;

        Ok(Self {
            projection_id,
            audience,
            purpose,
            output_schema,
            renderer,
            output,
            source_ids: collect_non_empty_ids(source_ids, "source_ids")?,
            information_loss: collect_non_empty_information_loss(information_loss)?,
        })
    }

    /// Creates a result from a projection while still requiring explicit trace data.
    pub fn from_projection<I, L>(
        projection: &Projection,
        renderer: RendererKind,
        output: ProjectionOutput,
        source_ids: I,
        information_loss: L,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
        L: IntoIterator<Item = InformationLoss>,
    {
        Self::new(
            projection.id.clone(),
            projection.audience,
            projection.purpose,
            projection.output_schema.clone(),
            renderer,
            output,
            source_ids,
            information_loss,
        )
    }

    /// Returns the source identifiers actually represented in the output.
    pub fn source_ids(&self) -> &[Id] {
        &self.source_ids
    }

    /// Returns the information-loss declarations that apply to the output.
    pub fn information_loss(&self) -> &[InformationLoss] {
        &self.information_loss
    }

    /// Returns the projection identifier used to produce this result.
    #[must_use]
    pub fn projection_id(&self) -> &Id {
        &self.projection_id
    }

    /// Returns the target audience copied from the projection definition.
    #[must_use]
    pub fn audience(&self) -> ProjectionAudience {
        self.audience
    }

    /// Returns the purpose copied from the projection definition.
    #[must_use]
    pub fn purpose(&self) -> ProjectionPurpose {
        self.purpose
    }

    /// Returns the output schema used for this result.
    #[must_use]
    pub fn output_schema(&self) -> &OutputSchema {
        &self.output_schema
    }

    /// Returns the renderer used to produce the output.
    #[must_use]
    pub fn renderer(&self) -> &RendererKind {
        &self.renderer
    }

    /// Returns the rendered output.
    #[must_use]
    pub fn output(&self) -> &ProjectionOutput {
        &self.output
    }
}

impl<'de> Deserialize<'de> for ProjectionResult {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            projection_id: Id,
            audience: ProjectionAudience,
            purpose: ProjectionPurpose,
            output_schema: OutputSchema,
            renderer: RendererKind,
            output: ProjectionOutput,
            source_ids: Vec<Id>,
            information_loss: Vec<InformationLoss>,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(
            wire.projection_id,
            wire.audience,
            wire.purpose,
            wire.output_schema,
            wire.renderer,
            wire.output,
            wire.source_ids,
            wire.information_loss,
        )
        .map_err(serde::de::Error::custom)
    }
}

fn ensure_output_matches_schema(
    output_schema: &OutputSchema,
    output: &ProjectionOutput,
) -> Result<()> {
    match (output_schema, output) {
        (OutputSchema::Text, ProjectionOutput::Text { .. }) => Ok(()),
        (OutputSchema::Sections { section_names }, ProjectionOutput::Sections { sections }) => {
            if sections
                .iter()
                .map(|section| section.title.as_str())
                .eq(section_names.iter().map(String::as_str))
            {
                Ok(())
            } else {
                Err(malformed_field(
                    "output",
                    "section output titles must match output_schema.section_names",
                ))
            }
        }
        (
            OutputSchema::Table { columns },
            ProjectionOutput::Table {
                columns: output_columns,
                ..
            },
        ) => {
            if output_columns == columns {
                Ok(())
            } else {
                Err(malformed_field(
                    "output",
                    "table output columns must match output_schema.columns",
                ))
            }
        }
        (OutputSchema::KeyValue { keys }, ProjectionOutput::KeyValue { entries }) => {
            if entries
                .iter()
                .map(|entry| entry.key.as_str())
                .eq(keys.iter().map(String::as_str))
            {
                Ok(())
            } else {
                Err(malformed_field(
                    "output",
                    "key-value output keys must match output_schema.keys",
                ))
            }
        }
        (OutputSchema::Custom { fields, .. }, ProjectionOutput::KeyValue { entries }) => {
            if entries
                .iter()
                .map(|entry| entry.key.as_str())
                .eq(fields.iter().map(String::as_str))
            {
                Ok(())
            } else {
                Err(malformed_field(
                    "output",
                    "custom output keys must match output_schema.fields",
                ))
            }
        }
        _ => Err(malformed_field(
            "output",
            "projection output kind must match output_schema kind",
        )),
    }
}

fn normalized_text(value: impl Into<String>, field: &'static str) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();

    if normalized.is_empty() {
        return Err(malformed_field(
            field,
            "value must not be empty after trimming",
        ));
    }

    Ok(normalized)
}

fn collect_non_empty_texts<I, S>(values: I, field: &'static str) -> Result<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let texts = values
        .into_iter()
        .map(|value| normalized_text(value, field))
        .collect::<Result<Vec<_>>>()?;

    if texts.is_empty() {
        return Err(malformed_field(field, "at least one value is required"));
    }

    Ok(texts)
}

fn collect_non_empty_ids<I>(ids: I, field: &'static str) -> Result<Vec<Id>>
where
    I: IntoIterator<Item = Id>,
{
    let ids = ids.into_iter().collect::<Vec<_>>();

    if ids.is_empty() {
        return Err(malformed_field(field, "at least one id is required"));
    }

    Ok(ids)
}

fn collect_non_empty_items<I, T>(items: I, field: &'static str) -> Result<Vec<T>>
where
    I: IntoIterator<Item = T>,
{
    let items = items.into_iter().collect::<Vec<_>>();

    if items.is_empty() {
        return Err(malformed_field(field, "at least one value is required"));
    }

    Ok(items)
}

fn collect_non_empty_information_loss<I>(information_loss: I) -> Result<Vec<InformationLoss>>
where
    I: IntoIterator<Item = InformationLoss>,
{
    collect_non_empty_items(information_loss, "information_loss")
}

fn malformed_field(field: impl Into<String>, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
