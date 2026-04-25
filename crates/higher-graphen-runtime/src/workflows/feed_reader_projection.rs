//! Projection helpers for the bounded Feed Product reader workflow.

use crate::error::RuntimeResult;
use crate::feed_reports::{
    FeedAuditTrace, FeedAuditTraceRecord, FeedCompletionCandidate, FeedCorrespondence,
    FeedEntryCell, FeedObstruction, FeedProjectionAudience, FeedProjectionPurpose,
    FeedProjectionRecord, FeedProjectionRecordType, FeedProjectionRequest, FeedProjectionView,
    FeedReaderInputDocument, FeedReaderProjection, FeedReaderResult, FeedReaderScenario,
    FeedTopicCell,
};
use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_projection::InformationLoss;

pub(super) fn report_projection(
    input: &FeedReaderInputDocument,
    scenario: &FeedReaderScenario,
    result: &FeedReaderResult,
) -> RuntimeResult<FeedReaderProjection> {
    Ok(FeedReaderProjection {
        timeline: timeline_projection(input, scenario, result)?,
        topic_digest: topic_digest_projection(input, scenario, result)?,
        audit_trace: audit_projection(input, scenario, result)?,
    })
}

fn timeline_projection(
    input: &FeedReaderInputDocument,
    scenario: &FeedReaderScenario,
    result: &FeedReaderResult,
) -> RuntimeResult<FeedProjectionView> {
    let request = projection_request(input, FeedProjectionPurpose::Timeline);
    let records = sorted_entry_records(&scenario.entries);
    let source_ids = projection_source_ids(request, || result.observed_entry_ids.clone());
    Ok(FeedProjectionView {
        audience: request.map_or(FeedProjectionAudience::Human, |value| value.audience),
        purpose: FeedProjectionPurpose::Timeline,
        summary: format!(
            "Timeline contains {} entries across {} source feeds.",
            scenario.entries.len(),
            scenario.source_feeds.len()
        ),
        records,
        information_loss: vec![projection_loss(request, source_ids.clone())?],
        source_ids,
    })
}

fn topic_digest_projection(
    input: &FeedReaderInputDocument,
    scenario: &FeedReaderScenario,
    result: &FeedReaderResult,
) -> RuntimeResult<FeedProjectionView> {
    let request = projection_request(input, FeedProjectionPurpose::TopicDigest);
    let source_ids = projection_source_ids(request, || topic_digest_source_ids(result));
    Ok(FeedProjectionView {
        audience: request.map_or(FeedProjectionAudience::AiAgent, |value| value.audience),
        purpose: FeedProjectionPurpose::TopicDigest,
        summary: format!(
            "Topic digest contains {} topics, {} correspondences, {} completion candidates, and {} obstructions.",
            scenario.topics.len(),
            result.correspondences.len(),
            result.completion_candidates.len(),
            result.obstructions.len()
        ),
        records: topic_digest_records(scenario, result),
        information_loss: vec![projection_loss(request, source_ids.clone())?],
        source_ids,
    })
}

fn audit_projection(
    input: &FeedReaderInputDocument,
    scenario: &FeedReaderScenario,
    result: &FeedReaderResult,
) -> RuntimeResult<FeedAuditTrace> {
    let request = projection_request(input, FeedProjectionPurpose::AuditTrace);
    let source_ids = projection_source_ids(request, || audit_source_ids(scenario, result));
    Ok(FeedAuditTrace {
        audience: FeedProjectionAudience::Audit,
        purpose: FeedProjectionPurpose::AuditTrace,
        traces: source_ids.iter().map(audit_trace).collect(),
        information_loss: vec![projection_loss(request, source_ids.clone())?],
        source_ids,
    })
}

fn sorted_entry_records(entries: &[FeedEntryCell]) -> Vec<FeedProjectionRecord> {
    let mut entries = entries.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.published_at.cmp(&right.published_at));
    entries.into_iter().map(entry_record).collect()
}

fn entry_record(entry: &FeedEntryCell) -> FeedProjectionRecord {
    FeedProjectionRecord {
        id: entry.id.clone(),
        record_type: FeedProjectionRecordType::Entry,
        summary: format!("{}: {}", entry.published_at, entry.label),
        source_ids: entry_source_ids(entry),
        confidence: entry.confidence,
        review_status: Some(entry.review_status),
    }
}

fn topic_digest_records(
    scenario: &FeedReaderScenario,
    result: &FeedReaderResult,
) -> Vec<FeedProjectionRecord> {
    let topic_records = scenario.topics.iter().map(topic_record);
    let event_records = scenario.events.iter().map(event_record);
    let correspondence_records = result.correspondences.iter().map(correspondence_record);
    let completion_records = result.completion_candidates.iter().map(completion_record);
    let obstruction_records = result.obstructions.iter().map(obstruction_record);
    topic_records
        .chain(event_records)
        .chain(correspondence_records)
        .chain(completion_records)
        .chain(obstruction_records)
        .collect()
}

fn topic_record(topic: &FeedTopicCell) -> FeedProjectionRecord {
    FeedProjectionRecord {
        id: topic.id.clone(),
        record_type: FeedProjectionRecordType::Topic,
        summary: format!("{} groups {} entries.", topic.label, topic.entry_ids.len()),
        source_ids: topic.entry_ids.clone(),
        confidence: topic.confidence,
        review_status: Some(topic.review_status),
    }
}

fn event_record(event: &crate::feed_reports::FeedEventCell) -> FeedProjectionRecord {
    FeedProjectionRecord {
        id: event.id.clone(),
        record_type: FeedProjectionRecordType::Event,
        summary: format!("{} groups {} entries.", event.label, event.entry_ids.len()),
        source_ids: event.entry_ids.clone(),
        confidence: event.confidence,
        review_status: Some(event.review_status),
    }
}

fn correspondence_record(correspondence: &FeedCorrespondence) -> FeedProjectionRecord {
    FeedProjectionRecord {
        id: correspondence.id.clone(),
        record_type: FeedProjectionRecordType::Correspondence,
        summary: correspondence.summary.clone(),
        source_ids: correspondence.entry_ids.clone(),
        confidence: Some(correspondence.confidence),
        review_status: Some(ReviewStatus::Accepted),
    }
}

fn completion_record(candidate: &FeedCompletionCandidate) -> FeedProjectionRecord {
    FeedProjectionRecord {
        id: candidate.id.clone(),
        record_type: FeedProjectionRecordType::CompletionCandidate,
        summary: candidate.suggested_structure.summary.clone(),
        source_ids: candidate.inferred_from.clone(),
        confidence: Some(candidate.confidence),
        review_status: Some(candidate.review_status),
    }
}

fn obstruction_record(obstruction: &FeedObstruction) -> FeedProjectionRecord {
    FeedProjectionRecord {
        id: obstruction.id.clone(),
        record_type: FeedProjectionRecordType::Obstruction,
        summary: obstruction.summary.clone(),
        source_ids: obstruction.source_ids.clone(),
        confidence: Some(obstruction.confidence),
        review_status: Some(ReviewStatus::Unreviewed),
    }
}

fn projection_loss(
    request: Option<&FeedProjectionRequest>,
    source_ids: Vec<Id>,
) -> RuntimeResult<InformationLoss> {
    let description = request
        .and_then(|value| value.information_loss_policy.clone())
        .unwrap_or_else(|| {
            "Projection summarizes feed observations and omits full content text.".to_owned()
        });
    Ok(InformationLoss::declared(description, source_ids)?)
}

fn projection_request(
    input: &FeedReaderInputDocument,
    purpose: FeedProjectionPurpose,
) -> Option<&FeedProjectionRequest> {
    input
        .projection_requests
        .iter()
        .find(|request| request.purpose == purpose)
}

fn projection_source_ids(
    request: Option<&FeedProjectionRequest>,
    fallback: impl FnOnce() -> Vec<Id>,
) -> Vec<Id> {
    request.map_or_else(fallback, |value| value.source_ids.clone())
}

fn topic_digest_source_ids(result: &FeedReaderResult) -> Vec<Id> {
    let mut ids = result.inferred_topic_ids.clone();
    extend_unique(&mut ids, result.inferred_event_ids.clone());
    extend_unique(
        &mut ids,
        result.correspondences.iter().map(|item| item.id.clone()),
    );
    extend_unique(
        &mut ids,
        result
            .completion_candidates
            .iter()
            .map(|item| item.id.clone()),
    );
    extend_unique(
        &mut ids,
        result.obstructions.iter().map(|item| item.id.clone()),
    );
    ids
}

fn audit_source_ids(scenario: &FeedReaderScenario, result: &FeedReaderResult) -> Vec<Id> {
    let mut ids = scenario.space.source_feed_ids.clone();
    extend_unique(&mut ids, scenario.space.entry_ids.clone());
    extend_unique(&mut ids, scenario.space.topic_ids.clone());
    extend_unique(&mut ids, scenario.space.event_ids.clone());
    extend_unique(
        &mut ids,
        result.correspondences.iter().map(|item| item.id.clone()),
    );
    extend_unique(
        &mut ids,
        result
            .completion_candidates
            .iter()
            .map(|item| item.id.clone()),
    );
    extend_unique(
        &mut ids,
        result.obstructions.iter().map(|item| item.id.clone()),
    );
    ids
}

fn audit_trace(source_id: &Id) -> FeedAuditTraceRecord {
    FeedAuditTraceRecord {
        source_id: source_id.clone(),
        role: source_role(source_id).to_owned(),
        represented_in: represented_views(source_id),
    }
}

fn represented_views(source_id: &Id) -> Vec<String> {
    let mut views = vec!["audit_trace".to_owned()];
    if source_id.as_str().starts_with("entry:") {
        views.push("timeline".to_owned());
    }
    if !source_id.as_str().starts_with("source:") {
        views.push("topic_digest".to_owned());
    }
    views.sort();
    views
}

fn entry_source_ids(entry: &FeedEntryCell) -> Vec<Id> {
    let mut ids = vec![entry.id.clone(), entry.source_id.clone()];
    extend_unique(&mut ids, entry.topic_ids.clone());
    if let Some(event_id) = &entry.event_id {
        push_unique(&mut ids, event_id.clone());
    }
    ids
}

fn extend_unique(ids: &mut Vec<Id>, new_ids: impl IntoIterator<Item = Id>) {
    for id in new_ids {
        push_unique(ids, id);
    }
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn source_role(source_id: &Id) -> &'static str {
    if source_id.as_str().starts_with("source:") {
        "source_feed"
    } else if source_id.as_str().starts_with("entry:") {
        "entry"
    } else if source_id.as_str().starts_with("topic:") {
        "topic"
    } else if source_id.as_str().starts_with("event:") {
        "event"
    } else if source_id.as_str().starts_with("correspondence:") {
        "correspondence"
    } else if source_id.as_str().starts_with("completion:") {
        "completion_candidate"
    } else if source_id.as_str().starts_with("obstruction:") {
        "obstruction"
    } else {
        "source"
    }
}
