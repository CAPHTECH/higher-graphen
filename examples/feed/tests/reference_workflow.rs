//! End-to-end reference checks for the Feed Product workflow.

use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_runtime::{
    run_feed_reader, FeedMissingType, FeedObstructionSeverity, FeedProjectionAudience,
    FeedProjectionPurpose, FeedProjectionRecordType, FeedReaderInputDocument, FeedReaderReport,
    FeedReaderStatus,
};

const REPORT_SCHEMA: &str = "highergraphen.feed.reader.report.v1";
const REPORT_TYPE: &str = "feed_reader";
const INPUT_SCHEMA: &str = "highergraphen.feed.input.v1";
const FEED_SPACE: &str = "space:feed-reader-input";
const COMMUNITY_ENTRY: &str = "entry:community-api-latency-continues";
const OFFICIAL_FOLLOW_UP: &str = "completion:api-latency-official-follow-up";
const API_LATENCY_OBSTRUCTION: &str = "obstruction:api-latency-resolution-conflict";

#[test]
fn reference_fixture_emits_feed_observation_space() {
    let report = run_feed_reader(reference_input()).expect("reference feed reader");

    assert_eq!(report.schema, REPORT_SCHEMA);
    assert_eq!(report.report_type, REPORT_TYPE);
    assert_eq!(report.scenario.input_schema, INPUT_SCHEMA);
    assert_eq!(report.scenario.space.id, id(FEED_SPACE));
    assert_eq!(report.scenario.source_feeds.len(), 4);
    assert_eq!(report.scenario.entries.len(), 4);
    assert_eq!(report.scenario.topics.len(), 2);
    assert_eq!(report.scenario.events.len(), 2);
    assert_eq!(report.result.status, FeedReaderStatus::ObstructionsDetected);
}

#[test]
fn reference_result_preserves_completion_and_obstruction_signals() {
    let report = run_feed_reader(reference_input()).expect("reference feed reader");

    assert!(report
        .result
        .observed_entry_ids
        .contains(&id(COMMUNITY_ENTRY)));
    assert_eq!(report.result.correspondences.len(), 2);
    assert_eq!(report.result.completion_candidates.len(), 2);
    assert_eq!(report.result.obstructions.len(), 1);

    let candidate = report
        .result
        .completion_candidates
        .iter()
        .find(|candidate| candidate.id == id(OFFICIAL_FOLLOW_UP))
        .expect("official follow-up completion candidate");
    assert_eq!(candidate.missing_type, FeedMissingType::OfficialSource);
    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    assert!(candidate.inferred_from.contains(&id(COMMUNITY_ENTRY)));

    let obstruction = report
        .result
        .obstructions
        .iter()
        .find(|obstruction| obstruction.id == id(API_LATENCY_OBSTRUCTION))
        .expect("api latency obstruction");
    assert_eq!(obstruction.severity, FeedObstructionSeverity::Warning);
    assert!(obstruction.entry_ids.contains(&id(COMMUNITY_ENTRY)));
}

#[test]
fn reference_projection_declares_information_loss() {
    let report = run_feed_reader(reference_input()).expect("reference feed reader");
    let projection = report.projection;

    assert_eq!(projection.timeline.audience, FeedProjectionAudience::Human);
    assert_eq!(projection.timeline.purpose, FeedProjectionPurpose::Timeline);
    assert_eq!(projection.timeline.records.len(), 4);
    assert!(!projection.timeline.information_loss.is_empty());

    assert_eq!(
        projection.topic_digest.audience,
        FeedProjectionAudience::AiAgent
    );
    assert!(projection
        .topic_digest
        .records
        .iter()
        .any(|record| record.record_type == FeedProjectionRecordType::Obstruction));

    assert_eq!(
        projection.audit_trace.audience,
        FeedProjectionAudience::Audit
    );
    assert_eq!(
        projection.audit_trace.purpose,
        FeedProjectionPurpose::AuditTrace
    );
    assert!(!projection.audit_trace.information_loss.is_empty());
}

#[test]
fn checked_in_report_matches_reference_contract() {
    let generated = run_feed_reader(reference_input()).expect("runtime report");
    let checked_in_text = include_str!("../reference/reports/feed-reader.report.json");
    let _checked_in: FeedReaderReport =
        serde_json::from_str(checked_in_text).expect("checked-in report parses");
    let generated_text = serde_json::to_string(&generated).expect("runtime report serializes");

    assert_eq!(
        checked_in_text, generated_text,
        "checked-in feed reader report drifted"
    );
}

fn reference_input() -> FeedReaderInputDocument {
    serde_json::from_str(include_str!("../reference/feed-reader.input.json"))
        .expect("reference input")
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id")
}
