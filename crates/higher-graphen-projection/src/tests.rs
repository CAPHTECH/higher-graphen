use super::*;

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
