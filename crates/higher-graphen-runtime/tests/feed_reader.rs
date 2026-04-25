//! Contract tests for the bounded Feed Product reader workflow.

use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_runtime::{
    run_feed_reader, FeedMissingType, FeedObstructionSeverity, FeedProjectionAudience,
    FeedProjectionPurpose, FeedReaderInputDocument, FeedReaderStatus,
};
use serde_json::{json, Value};

const INPUT_SCHEMA: &str = "highergraphen.feed.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.feed.reader.report.v1";
const REPORT_TYPE: &str = "feed_reader";
const FEED_SPACE: &str = "space:feed-reader-input";
const OFFICIAL_STATUS_SOURCE: &str = "source:official-status";
const API_LATENCY_TOPIC: &str = "topic:api-latency";
const API_LATENCY_EVENT: &str = "event:api-latency-incident";
const OFFICIAL_STATUS_ENTRY: &str = "entry:official-status-api-latency-resolved";
const COMMUNITY_LATENCY_ENTRY: &str = "entry:community-api-latency-continues";
const OFFICIAL_FOLLOW_UP_CANDIDATE: &str = "completion:api-latency-official-follow-up";
const API_LATENCY_OBSTRUCTION: &str = "obstruction:api-latency-resolution-conflict";

#[test]
fn runner_lifts_bounded_feed_fixture() {
    let report = run_feed_reader(fixture()).expect("workflow should run");

    assert_eq!(report.schema, REPORT_SCHEMA);
    assert_eq!(report.report_type, REPORT_TYPE);
    assert_eq!(report.report_version, 1);
    assert_eq!(report.metadata.command, "highergraphen feed reader run");
    assert_eq!(report.result.status, FeedReaderStatus::ObstructionsDetected);
}

#[test]
fn scenario_contains_source_feeds_entries_topics_and_events() {
    let report = run_feed_reader(fixture()).expect("workflow should run");
    let scenario = report.scenario;

    assert_eq!(scenario.input_schema, INPUT_SCHEMA);
    assert_eq!(scenario.space.id, id(FEED_SPACE));
    assert!(scenario
        .space
        .source_feed_ids
        .contains(&id(OFFICIAL_STATUS_SOURCE)));
    assert!(scenario
        .space
        .entry_ids
        .contains(&id(OFFICIAL_STATUS_ENTRY)));
    assert!(scenario.space.topic_ids.contains(&id(API_LATENCY_TOPIC)));
    assert!(scenario.space.event_ids.contains(&id(API_LATENCY_EVENT)));

    let entry = scenario
        .entries
        .iter()
        .find(|entry| entry.id == id(OFFICIAL_STATUS_ENTRY))
        .expect("official status entry");
    assert_eq!(entry.dimension, 0);
    assert_eq!(entry.cell_type, "feed_entry");
    assert_eq!(entry.source_id, id(OFFICIAL_STATUS_SOURCE));
    assert_eq!(entry.review_status, ReviewStatus::Accepted);
    assert_eq!(entry.confidence.expect("confidence").value(), 1.0);
}

#[test]
fn result_preserves_correspondence_completion_and_obstruction_hints() {
    let report = run_feed_reader(fixture()).expect("workflow should run");
    let result = report.result;

    assert_eq!(result.correspondences.len(), 2);
    assert_eq!(result.completion_candidates.len(), 2);
    assert_eq!(result.obstructions.len(), 1);
    assert!(result
        .observed_entry_ids
        .contains(&id(COMMUNITY_LATENCY_ENTRY)));

    let candidate = result
        .completion_candidates
        .iter()
        .find(|candidate| candidate.id == id(OFFICIAL_FOLLOW_UP_CANDIDATE))
        .expect("official follow-up completion");
    assert_eq!(candidate.missing_type, FeedMissingType::OfficialSource);
    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    assert!(candidate
        .inferred_from
        .contains(&id(COMMUNITY_LATENCY_ENTRY)));

    let obstruction = result
        .obstructions
        .iter()
        .find(|obstruction| obstruction.id == id(API_LATENCY_OBSTRUCTION))
        .expect("api latency obstruction");
    assert_eq!(obstruction.severity, FeedObstructionSeverity::Warning);
    assert!(obstruction.entry_ids.contains(&id(OFFICIAL_STATUS_ENTRY)));
}

#[test]
fn projection_keeps_source_ids_and_declares_information_loss() {
    let report = run_feed_reader(fixture()).expect("workflow should run");
    let projection = report.projection;

    assert_eq!(projection.timeline.audience, FeedProjectionAudience::Human);
    assert_eq!(projection.timeline.purpose, FeedProjectionPurpose::Timeline);
    assert_eq!(projection.timeline.records.len(), 4);
    assert_eq!(projection.timeline.records[0].id, id(OFFICIAL_STATUS_ENTRY));
    assert!(!projection.timeline.information_loss.is_empty());
    assert!(projection
        .timeline
        .source_ids
        .contains(&id(COMMUNITY_LATENCY_ENTRY)));

    assert_eq!(
        projection.topic_digest.audience,
        FeedProjectionAudience::AiAgent
    );
    assert!(projection
        .topic_digest
        .records
        .iter()
        .any(|record| record.id == id(API_LATENCY_OBSTRUCTION)));
    assert!(projection
        .audit_trace
        .traces
        .iter()
        .any(|trace| trace.source_id == id(OFFICIAL_STATUS_SOURCE)));
}

#[test]
fn report_serializes_lower_snake_case_values_and_round_trips() {
    let report = run_feed_reader(fixture()).expect("workflow should run");
    let value = serde_json::to_value(&report).expect("serialize report");

    assert_eq!(value["schema"], json!(REPORT_SCHEMA));
    assert_eq!(value["report_type"], json!(REPORT_TYPE));
    assert_eq!(value["result"]["status"], json!("obstructions_detected"));
    assert_eq!(
        value["result"]["completion_candidates"][0]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["result"]["obstructions"][0]["severity"],
        json!("warning")
    );
    assert_eq!(
        value["projection"]["topic_digest"]["audience"],
        json!("ai_agent")
    );

    let json_text = serde_json::to_string(&report).expect("serialize report text");
    let round_tripped: Value = serde_json::from_str(&json_text).expect("parse report json");
    assert_eq!(round_tripped["scenario"]["space"]["id"], json!(FEED_SPACE));
}

#[test]
fn rejects_unknown_entry_source() {
    let mut input = fixture();
    input.entries[0].source_id = id("source:missing");

    let error = run_feed_reader(input).expect_err("unknown source should fail");

    assert_eq!(error.code(), "workflow_construction");
    assert!(error
        .to_string()
        .contains("entry source references unknown id"));
}

fn fixture() -> FeedReaderInputDocument {
    serde_json::from_str(include_str!(
        "../../../schemas/inputs/feed-lift.input.example.json"
    ))
    .expect("fixture should parse")
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}
