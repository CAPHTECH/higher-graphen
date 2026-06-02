use super::*;

#[test]
fn projection_loss_key_value_reports_untraced_item_only() {
    let result = ProjectionResult::new(
        id("projection:loss-untraced-entry"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::key_value(["traced", "untraced"]).expect("schema should be valid"),
        RendererKind::Structured,
        ProjectionOutput::key_value([
            ProjectionEntry::new("traced", "A", [id("cell:a")]).expect("entry should be valid"),
            ProjectionEntry::new("untraced", "No source", Vec::<Id>::new())
                .expect("empty trace is valid"),
        ])
        .expect("output should be valid"),
        [id("cell:a")],
        [loss("cell:b")],
    )
    .expect("result should be valid");

    let report = measure_projection_loss(&result, &ids(&["cell:a"]));

    assert!(report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::SourceTraceMissing));
    assert!(!report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UnsupportedLossMetric));
}

#[test]
fn projection_loss_empty_key_value_reports_untraced_item_only() {
    let result = ProjectionResult::new(
        id("projection:loss-empty-kv"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::key_value(["risk"]).expect("schema should be valid"),
        RendererKind::Structured,
        ProjectionOutput::key_value(Vec::<ProjectionEntry>::new()).expect("empty output is valid"),
        [id("cell:a")],
        [loss("cell:b")],
    )
    .expect("empty attributable result should be valid");

    let report = measure_projection_loss(&result, &ids(&["cell:a"]));

    assert_eq!(report.metric.projected_cardinality, 0);
    assert!(report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::SourceTraceMissing));
    assert!(!report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UnsupportedLossMetric));
}

#[test]
fn projection_loss_fully_traced_key_value_has_no_trace_obstructions() {
    let result = ProjectionResult::new(
        id("projection:loss-clean-kv"),
        ProjectionAudience::Human,
        ProjectionPurpose::Report,
        OutputSchema::key_value(["risk"]).expect("schema should be valid"),
        RendererKind::Structured,
        ProjectionOutput::key_value([
            ProjectionEntry::new("risk", "A", [id("cell:a")]).expect("entry should be valid")
        ])
        .expect("output should be valid"),
        [id("cell:a")],
        [loss("cell:b")],
    )
    .expect("result should be valid");

    let report = measure_projection_loss(&result, &ids(&["cell:a"]));

    assert!(!report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::SourceTraceMissing));
    assert!(!report
        .ambiguity
        .obstructions
        .contains(&ProjectionLossObstruction::UnsupportedLossMetric));
}

#[test]
fn projection_entry_and_output_round_trip_empty_traces() {
    let output =
        ProjectionOutput::key_value([ProjectionEntry::new("empty", "No trace", Vec::<Id>::new())
            .expect("empty trace is valid")])
        .expect("output should be valid");
    let empty_output =
        ProjectionOutput::key_value(Vec::<ProjectionEntry>::new()).expect("empty output is valid");

    let output_json = serde_json::to_string(&output).expect("output should serialize");
    let empty_json = serde_json::to_string(&empty_output).expect("empty output should serialize");
    let decoded: ProjectionOutput =
        serde_json::from_str(&output_json).expect("output should deserialize");
    let decoded_empty: ProjectionOutput =
        serde_json::from_str(&empty_json).expect("empty output should deserialize");

    assert_eq!(decoded, output);
    assert_eq!(decoded_empty, empty_output);
}
