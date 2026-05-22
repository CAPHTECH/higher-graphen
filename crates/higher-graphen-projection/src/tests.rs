use super::*;
use higher_graphen_core::{
    Confidence, CorrespondenceCell, CorrespondenceKind, CorrespondenceParticipant,
    CorrespondencePolarity, DifferenceKind, DifferenceSeverity, DifferenceWitness,
    DifferingStructure, GluingAttempt, GluingResult, InvariantCheckResult, NormalizedClaim,
    OverlapWitness, OverlapWitnessKind, ParticipantRef, ReviewStatus, Scope, SharedStructure,
};
use std::collections::BTreeMap;

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}

fn loss(source_id: &str) -> InformationLoss {
    InformationLoss::declared("summarized detail", [id(source_id)])
        .expect("test loss should be valid")
}

#[test]
fn projection_requires_declared_information_loss() {
    let projection = Projection::new(
        id("projection:architecture-summary"),
        id("space:architecture"),
        "Architecture summary",
        ProjectionAudience::Architect,
        ProjectionPurpose::Report,
        ProjectionSelector::all(),
        OutputSchema::sections(["summary", "risks"]).expect("schema should be valid"),
        Vec::<InformationLoss>::new(),
    );

    assert_eq!(
        projection
            .expect_err("empty information loss should fail")
            .code(),
        "malformed_field"
    );
}

#[test]
fn result_requires_explicit_source_ids() {
    let result = ProjectionResult::new(
        id("projection:architecture-summary"),
        ProjectionAudience::Architect,
        ProjectionPurpose::Report,
        OutputSchema::text(),
        RendererKind::PlainText,
        ProjectionOutput::text("summary").expect("output should be valid"),
        Vec::<Id>::new(),
        [loss("cell:service-a")],
    );

    assert_eq!(
        result.expect_err("empty source ids should fail").code(),
        "malformed_field"
    );
}

#[test]
fn output_sections_keep_section_source_ids() {
    let source_id = id("cell:service-a");
    let section = ProjectionSection::new("Risk", "Dependency is unstable", [source_id.clone()])
        .expect("section should be valid");

    let output = ProjectionOutput::sections([section.clone()]).expect("output should be valid");

    assert_eq!(section.source_ids(), [source_id]);
    assert!(matches!(output, ProjectionOutput::Sections { .. }));
}

#[test]
fn projection_result_can_be_created_from_projection_with_explicit_trace_data() {
    let source_id = id("cell:service-a");
    let declared_loss = loss(source_id.as_str());
    let projection = Projection::new(
        id("projection:architecture-summary"),
        id("space:architecture"),
        " Architecture summary ",
        ProjectionAudience::Architect,
        ProjectionPurpose::Report,
        ProjectionSelector::all().with_cell_ids([source_id.clone()]),
        OutputSchema::key_value(["risk"]).expect("schema should be valid"),
        [declared_loss.clone()],
    )
    .expect("projection should be valid")
    .with_renderer(RendererKind::Structured);

    let result = ProjectionResult::from_projection(
        &projection,
        RendererKind::Structured,
        ProjectionOutput::key_value([ProjectionEntry::new(
            "risk",
            "Dependency is unstable",
            [source_id.clone()],
        )
        .expect("entry should be valid")])
        .expect("output should be valid"),
        [source_id.clone()],
        [declared_loss],
    )
    .expect("result should be valid");

    assert_eq!(projection.name, "Architecture summary");
    assert_eq!(result.source_ids(), [source_id]);
    assert_eq!(result.information_loss().len(), 1);
}

#[test]
fn table_output_requires_rows_to_match_columns() {
    let output = ProjectionOutput::table(["cell", "risk"], vec![vec!["cell:1".to_owned()]]);

    assert_eq!(
        output.expect_err("mismatched row should fail").code(),
        "malformed_field"
    );
}

#[test]
fn projection_result_rejects_output_kind_that_does_not_match_schema() {
    let source_id = id("cell:service-a");
    let result = ProjectionResult::new(
        id("projection:architecture-summary"),
        ProjectionAudience::Architect,
        ProjectionPurpose::Report,
        OutputSchema::sections(["summary"]).expect("schema should be valid"),
        RendererKind::PlainText,
        ProjectionOutput::text("summary").expect("output should be valid"),
        [source_id.clone()],
        [loss(source_id.as_str())],
    );

    assert_eq!(
        result.expect_err("schema mismatch should fail").code(),
        "malformed_field"
    );
}

#[test]
fn projection_result_rejects_section_titles_that_do_not_match_schema() {
    let source_id = id("cell:service-a");
    let result = ProjectionResult::new(
        id("projection:architecture-summary"),
        ProjectionAudience::Architect,
        ProjectionPurpose::Report,
        OutputSchema::sections(["summary", "risks"]).expect("schema should be valid"),
        RendererKind::Markdown,
        ProjectionOutput::sections([
            ProjectionSection::new("summary", "Service overview", [source_id.clone()])
                .expect("section should be valid"),
            ProjectionSection::new("constraints", "Dependency is unstable", [source_id.clone()])
                .expect("section should be valid"),
        ])
        .expect("output should be valid"),
        [source_id.clone()],
        [loss(source_id.as_str())],
    );

    assert_eq!(
        result
            .expect_err("section title mismatch should fail")
            .code(),
        "malformed_field"
    );
}

#[test]
fn projection_result_rejects_table_columns_that_do_not_match_schema() {
    let source_id = id("cell:service-a");
    let result = ProjectionResult::new(
        id("projection:architecture-summary"),
        ProjectionAudience::Architect,
        ProjectionPurpose::Report,
        OutputSchema::table(["cell", "risk"]).expect("schema should be valid"),
        RendererKind::Table,
        ProjectionOutput::table(["risk"], Vec::new()).expect("output should be valid"),
        [source_id.clone()],
        [loss(source_id.as_str())],
    );

    assert_eq!(
        result.expect_err("column mismatch should fail").code(),
        "malformed_field"
    );
}

#[test]
fn custom_schema_accepts_key_value_output_with_matching_fields() {
    let source_id = id("cell:service-a");
    let result = ProjectionResult::new(
        id("projection:architecture-summary"),
        ProjectionAudience::ExternalSystem,
        ProjectionPurpose::ApiResponse,
        OutputSchema::custom("architecture_summary", ["risk"]).expect("schema should be valid"),
        RendererKind::Structured,
        ProjectionOutput::key_value([ProjectionEntry::new(
            "risk",
            "Dependency is unstable",
            [source_id.clone()],
        )
        .expect("entry should be valid")])
        .expect("output should be valid"),
        [source_id.clone()],
        [loss(source_id.as_str())],
    )
    .expect("matching custom output should be valid");

    assert_eq!(result.source_ids(), [source_id]);
}

#[test]
fn deserialization_rejects_projection_result_schema_mismatch() {
    let value = serde_json::json!({
        "projection_id": "projection:architecture-summary",
        "audience": "architect",
        "purpose": "report",
        "output_schema": {
            "kind": "sections",
            "section_names": ["summary"]
        },
        "renderer": "plain_text",
        "output": {
            "kind": "text",
            "text": "summary"
        },
        "source_ids": ["cell:service-a"],
        "information_loss": [{
            "description": "summarized detail",
            "source_ids": ["cell:service-a"]
        }]
    });

    assert!(serde_json::from_value::<ProjectionResult>(value).is_err());
}

#[test]
fn correspondence_explanation_includes_witnesses_gluing_and_empty_loss() {
    let correspondence = correspondence_fixture();
    let explanation = explain_correspondence(&correspondence);

    assert_eq!(explanation.schema, CORRESPONDENCE_EXPLANATION_SCHEMA);
    assert_eq!(
        explanation.correspondence_id,
        id("corr:order-service-billing-db-access")
    );
    assert_eq!(explanation.overlap_witnesses.len(), 1);
    assert_eq!(explanation.difference_witnesses.len(), 1);
    assert_eq!(
        explanation
            .gluing
            .as_ref()
            .and_then(|gluing| gluing.obstruction.as_ref()),
        Some(&id("obstruction:direct-db-access-violates-boundary"))
    );
    assert!(explanation
        .projection_loss
        .omitted_overlap_witnesses
        .is_empty());
    assert!(explanation
        .projection_loss
        .omitted_difference_witnesses
        .is_empty());
    assert!(explanation.projection_loss.omitted_evidence.is_empty());
    assert!(explanation.projection_loss.omitted_contexts.is_empty());
}

#[test]
fn correspondence_projection_keeps_candidate_status_and_declares_no_loss_when_complete() {
    let correspondence = correspondence_fixture();
    let projection = project_correspondence(
        &correspondence,
        ProjectionAudience::Human,
        ProjectionPurpose::Review,
    );

    assert_eq!(projection.schema, CORRESPONDENCE_PROJECTION_SCHEMA);
    assert_eq!(projection.audience, ProjectionAudience::Human);
    assert_eq!(projection.purpose, ProjectionPurpose::Review);
    assert_eq!(projection.renderer, RendererKind::Markdown);
    assert_eq!(projection.review_status, ReviewStatus::Candidate);
    assert_eq!(projection.provenance, id("provenance:architecture-review"));
    assert!(projection.projection_loss.omitted_evidence.is_empty());
    assert!(projection.projection_loss.collapsed_statuses.is_empty());
}

#[test]
fn correspondence_projection_markdown_shows_review_gluing_and_loss() {
    let correspondence = correspondence_fixture();
    let projection = project_correspondence(
        &correspondence,
        ProjectionAudience::Human,
        ProjectionPurpose::Review,
    );
    let markdown = render_correspondence_projection_markdown(&projection);

    assert!(markdown.contains("# Correspondence corr:order-service-billing-db-access"));
    assert!(markdown.contains("Review status: candidate"));
    assert!(markdown.contains("witness:shared-order-billing-access"));
    assert!(markdown.contains("diff:observed-vs-forbidden"));
    assert!(markdown.contains("Result: failure"));
    assert!(markdown.contains("Obstruction: obstruction:direct-db-access-violates-boundary"));
    assert!(markdown.contains("## Projection Loss"));
    assert!(markdown.contains("- none"));
}

#[test]
fn correspondence_projection_supports_audit_profile_with_provenance() {
    let correspondence = correspondence_fixture();
    let projection = project_correspondence(
        &correspondence,
        ProjectionAudience::Audit,
        ProjectionPurpose::Report,
    );

    assert_eq!(projection.audience, ProjectionAudience::Audit);
    assert_eq!(projection.renderer, RendererKind::Structured);
    assert_eq!(projection.provenance, id("provenance:architecture-review"));
    assert_eq!(projection.review_status, ReviewStatus::Candidate);
    assert_eq!(
        projection
            .gluing
            .as_ref()
            .and_then(|gluing| gluing.obstruction.as_ref()),
        Some(&id("obstruction:direct-db-access-violates-boundary"))
    );
}

fn correspondence_fixture() -> CorrespondenceCell {
    CorrespondenceCell {
        id: id("corr:order-service-billing-db-access"),
        participants: vec![
            CorrespondenceParticipant::new(
                "observed_claim",
                ParticipantRef::Claim(id("claim:architecture-doc-order-billing-access")),
            )
            .expect("participant"),
            CorrespondenceParticipant::new(
                "constraint",
                ParticipantRef::Invariant(id("invariant:no-cross-context-db-access")),
            )
            .expect("participant"),
        ],
        correspondence_kind: CorrespondenceKind::ConstraintOverlap,
        polarity: CorrespondencePolarity::Conflicting,
        overlap_witnesses: vec![OverlapWitness {
            id: id("witness:shared-order-billing-access"),
            witness_kind: OverlapWitnessKind::NormalizedClaim,
            shared_structure: SharedStructure::NormalizedClaim(NormalizedClaim {
                subject: "OrderService".to_owned(),
                relation: "accesses".to_owned(),
                object: "BillingDB".to_owned(),
                modality: None,
                temporal_scope: None,
            }),
            participant_mappings: Vec::new(),
            scope: Scope::default(),
            context: id("ctx:architecture-review"),
            evidence: vec![id("evidence:architecture-doc")],
            confidence: Confidence::new(0.91).expect("confidence"),
            status: ReviewStatus::Candidate,
        }],
        difference_witnesses: vec![DifferenceWitness {
            id: id("diff:observed-vs-forbidden"),
            difference_kind: DifferenceKind::ModalityMismatch,
            differing_structure: DifferingStructure::ModalityMismatch(BTreeMap::from([
                ("observed_claim".to_owned(), "observed".to_owned()),
                ("constraint".to_owned(), "forbidden".to_owned()),
            ])),
            participant_mappings: Vec::new(),
            severity: DifferenceSeverity::Blocking,
            context: id("ctx:architecture-review"),
            evidence: vec![id("evidence:architecture-doc")],
            confidence: Confidence::new(0.93).expect("confidence"),
            status: ReviewStatus::Candidate,
        }],
        context: id("ctx:architecture-review"),
        evidence: vec![id("evidence:architecture-doc")],
        provenance: id("provenance:architecture-review"),
        confidence: Confidence::new(0.9).expect("confidence"),
        review_status: ReviewStatus::Candidate,
        gluing: Some(GluingAttempt {
            id: id("glue:order-service-billing-db-access"),
            participants: vec![
                ParticipantRef::Claim(id("claim:architecture-doc-order-billing-access")),
                ParticipantRef::Invariant(id("invariant:no-cross-context-db-access")),
            ],
            overlap_witnesses: vec![id("witness:shared-order-billing-access")],
            difference_witnesses: vec![id("diff:observed-vs-forbidden")],
            context: id("ctx:architecture-review"),
            invariant_checks: vec![InvariantCheckResult {
                invariant: id("invariant:no-cross-context-db-access"),
                result: "failed".to_owned(),
                detail: None,
            }],
            preservation_report: higher_graphen_core::PreservationReport::default(),
            result: GluingResult::Failure {
                obstruction: id("obstruction:direct-db-access-violates-boundary"),
            },
            evidence: vec![id("evidence:architecture-doc")],
            confidence: Confidence::new(0.9).expect("confidence"),
            status: ReviewStatus::Candidate,
            override_review: None,
        }),
    }
}
