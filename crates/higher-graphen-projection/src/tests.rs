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

fn loss_many(source_ids: &[&str]) -> InformationLoss {
    InformationLoss::declared(
        "summarized detail",
        source_ids.iter().map(|source_id| id(source_id)),
    )
    .expect("test loss should be valid")
}

fn ids(source_ids: &[&str]) -> Vec<Id> {
    source_ids.iter().map(|source_id| id(source_id)).collect()
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
fn projection_loss_declared_vs_undeclared_collapse() {
    let declared_result = ProjectionResult::new(
        id("projection:loss-collapse"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::sections(["merged", "solo"]).expect("schema should be valid"),
        RendererKind::Markdown,
        ProjectionOutput::sections([
            ProjectionSection::new(
                "merged",
                "A and B summarized together",
                ids(&["cell:b", "cell:a"]),
            )
            .expect("section should be valid"),
            ProjectionSection::new("solo", "C preserved separately", [id("cell:c")])
                .expect("section should be valid"),
        ])
        .expect("output should be valid"),
        ids(&["cell:c", "cell:a", "cell:b"]),
        [loss_many(&["cell:b", "cell:a"])],
    )
    .expect("result should be valid");

    let declared_report =
        measure_projection_loss(&declared_result, &ids(&["cell:c", "cell:b", "cell:a"]));

    assert_eq!(declared_report.metric.source_cardinality, 3);
    assert_eq!(declared_report.metric.projected_cardinality, 2);
    assert_eq!(declared_report.metric.collapsed_pair_count, 1);
    assert_eq!(declared_report.metric.distinguished_pair_count, 2);
    assert_eq!(
        declared_report.metric.declared_loss_source_ids,
        ids(&["cell:a", "cell:b"])
    );
    assert_eq!(
        declared_report.ambiguity.collapsed_source_groups,
        [ProjectionCollapsedSourceGroup {
            item_id: "section:0:merged".to_owned(),
            source_ids: ids(&["cell:a", "cell:b"]),
        }]
    );
    assert!(!declared_report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UndeclaredProjectionLoss));
    assert!(declared_report
        .ambiguity
        .missing_loss_declarations
        .is_empty());

    let undeclared_result = ProjectionResult::new(
        id("projection:loss-collapse"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::sections(["merged", "solo"]).expect("schema should be valid"),
        RendererKind::Markdown,
        ProjectionOutput::sections([
            ProjectionSection::new(
                "merged",
                "A and B summarized together",
                ids(&["cell:b", "cell:a"]),
            )
            .expect("section should be valid"),
            ProjectionSection::new("solo", "C preserved separately", [id("cell:c")])
                .expect("section should be valid"),
        ])
        .expect("output should be valid"),
        ids(&["cell:c", "cell:a", "cell:b"]),
        [loss("cell:c")],
    )
    .expect("result should be valid");

    let undeclared_report =
        measure_projection_loss(&undeclared_result, &ids(&["cell:c", "cell:b", "cell:a"]));

    assert!(undeclared_report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UndeclaredProjectionLoss));
    assert_eq!(
        undeclared_report.ambiguity.missing_loss_declarations,
        ids(&["cell:a", "cell:b"])
    );
    assert_eq!(undeclared_report.ambiguity.risk_severity, Severity::High);
}

#[test]
fn projection_loss_ambiguous_declared_vs_undeclared() {
    let declared_result = ProjectionResult::new(
        id("projection:loss-ambiguous"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::sections(["first", "second"]).expect("schema should be valid"),
        RendererKind::Markdown,
        ProjectionOutput::sections([
            ProjectionSection::new("first", "A appears here", [id("cell:a")])
                .expect("section should be valid"),
            ProjectionSection::new("second", "A also appears here", [id("cell:a")])
                .expect("section should be valid"),
        ])
        .expect("output should be valid"),
        [id("cell:a")],
        [loss("cell:a")],
    )
    .expect("result should be valid");

    let declared_report = measure_projection_loss(&declared_result, &ids(&["cell:a"]));

    assert_eq!(declared_report.metric.ambiguity_score, 1.0);
    assert_eq!(
        declared_report.ambiguity.ambiguous_output_ids,
        ["section:0:first".to_owned(), "section:1:second".to_owned()]
    );
    assert!(!declared_report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::AmbiguousProjectionOutput));

    let undeclared_result = ProjectionResult::new(
        id("projection:loss-ambiguous"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::sections(["first", "second"]).expect("schema should be valid"),
        RendererKind::Markdown,
        ProjectionOutput::sections([
            ProjectionSection::new("first", "A appears here", [id("cell:a")])
                .expect("section should be valid"),
            ProjectionSection::new("second", "A also appears here", [id("cell:a")])
                .expect("section should be valid"),
        ])
        .expect("output should be valid"),
        [id("cell:a")],
        [loss("cell:unrelated")],
    )
    .expect("result should be valid");

    let undeclared_report = measure_projection_loss(&undeclared_result, &ids(&["cell:a"]));

    assert!(undeclared_report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::AmbiguousProjectionOutput));
    assert_eq!(
        undeclared_report.ambiguity.missing_loss_declarations,
        ids(&["cell:a"])
    );
    assert_eq!(undeclared_report.ambiguity.risk_severity, Severity::High);
}

#[test]
fn projection_loss_reports_undeclared_omission() {
    let result = ProjectionResult::new(
        id("projection:loss-omission"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::sections(["kept"]).expect("schema should be valid"),
        RendererKind::Markdown,
        ProjectionOutput::sections([ProjectionSection::new(
            "kept",
            "A is represented",
            [id("cell:a")],
        )
        .expect("section should be valid")])
        .expect("output should be valid"),
        [id("cell:a")],
        [loss("cell:unrelated")],
    )
    .expect("result should be valid");

    let report = measure_projection_loss(&result, &ids(&["cell:a", "cell:b"]));

    assert_eq!(report.metric.source_cardinality, 2);
    assert_eq!(
        report.metric.source_cardinality_basis,
        ProjectionSourceCardinalityBasis::EligibleSourceUniverse
    );
    assert_eq!(report.metric.omitted_source_ids, ids(&["cell:b"]));
    assert!(report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UndeclaredProjectionLoss));
    assert_eq!(report.ambiguity.missing_loss_declarations, ids(&["cell:b"]));
}

#[test]
fn projection_loss_text_reports_unsupported_and_untraced_metrics() {
    let result = ProjectionResult::new(
        id("projection:loss-text"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::text(),
        RendererKind::PlainText,
        ProjectionOutput::text("A summary").expect("output should be valid"),
        [id("cell:a")],
        [loss("cell:b")],
    )
    .expect("result should be valid");

    let report = measure_projection_loss(&result, &ids(&["cell:a", "cell:b"]));

    assert_eq!(report.metric.projected_cardinality, 1);
    assert_eq!(report.metric.collapsed_pair_count, 0);
    assert_eq!(report.metric.distinguished_pair_count, 0);
    assert_eq!(report.metric.omitted_source_ids, ids(&["cell:b"]));
    assert_eq!(report.metric.ambiguity_score, 0.0);
    assert!(report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::SourceTraceMissing));
    assert!(report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UnsupportedLossMetric));
    assert!(!report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UndeclaredProjectionLoss));
    assert_eq!(report.ambiguity.risk_severity, Severity::Medium);
}

#[test]
fn projection_loss_table_reports_unsupported_and_untraced_metrics() {
    let result = ProjectionResult::new(
        id("projection:loss-table"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::table(["cell"]).expect("schema should be valid"),
        RendererKind::Table,
        ProjectionOutput::table(["cell"], vec![vec!["cell:a".to_owned()]])
            .expect("output should be valid"),
        [id("cell:a")],
        [loss("cell:b")],
    )
    .expect("result should be valid");

    let report = measure_projection_loss(&result, &ids(&["cell:a", "cell:b"]));

    assert_eq!(report.metric.projected_cardinality, 1);
    assert_eq!(report.metric.source_cardinality, 2);
    assert_eq!(report.metric.omitted_source_ids, ids(&["cell:b"]));
    assert!(report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::SourceTraceMissing));
    assert!(report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UnsupportedLossMetric));
    assert!(!report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UndeclaredProjectionLoss));
    assert_eq!(report.ambiguity.risk_severity, Severity::Medium);
}

#[test]
fn projection_loss_report_is_deterministic() {
    let result = ProjectionResult::new(
        id("projection:loss-deterministic"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::key_value(["merged", "solo"]).expect("schema should be valid"),
        RendererKind::Structured,
        ProjectionOutput::key_value([
            ProjectionEntry::new("merged", "B and A", ids(&["cell:b", "cell:a"]))
                .expect("entry should be valid"),
            ProjectionEntry::new("solo", "C", [id("cell:c")]).expect("entry should be valid"),
        ])
        .expect("output should be valid"),
        ids(&["cell:c", "cell:a", "cell:b"]),
        [loss_many(&["cell:b", "cell:a"])],
    )
    .expect("result should be valid");

    let first = measure_projection_loss(&result, &ids(&["cell:c", "cell:b", "cell:a"]));
    let second = measure_projection_loss(&result, &ids(&["cell:a", "cell:c", "cell:b"]));
    let first_json = serde_json::to_string(&first).expect("report should serialize");
    let second_json = serde_json::to_string(&second).expect("report should serialize");

    assert_eq!(first_json, second_json);
}

#[test]
fn projection_loss_report_json_round_trips() {
    let result = ProjectionResult::new(
        id("projection:loss-round-trip"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::sections(["merged"]).expect("schema should be valid"),
        RendererKind::Markdown,
        ProjectionOutput::sections([ProjectionSection::new(
            "merged",
            "A and B summarized",
            ids(&["cell:a", "cell:b"]),
        )
        .expect("section should be valid")])
        .expect("output should be valid"),
        ids(&["cell:a", "cell:b"]),
        [loss_many(&["cell:a", "cell:b"])],
    )
    .expect("result should be valid");
    let report = measure_projection_loss(&result, &ids(&["cell:a", "cell:b"]));

    let json = serde_json::to_string(&report).expect("report should serialize");
    let decoded: ProjectionLossReport =
        serde_json::from_str(&json).expect("report should deserialize");

    assert_eq!(decoded, report);
    assert_eq!(
        decoded.metric.ambiguity_score,
        report.metric.ambiguity_score
    );
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
