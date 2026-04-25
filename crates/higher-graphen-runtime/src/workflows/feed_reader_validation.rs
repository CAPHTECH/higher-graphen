//! Input validation helpers for the bounded Feed Product reader workflow.

use crate::error::{RuntimeError, RuntimeResult};
use crate::feed_reports::{FeedCompletionHint, FeedReaderInputDocument};
use higher_graphen_core::Id;

const WORKFLOW_NAME: &str = "feed_reader";
const INPUT_SCHEMA: &str = "highergraphen.feed.input.v1";

pub(super) fn validate_input(input: &FeedReaderInputDocument) -> RuntimeResult<()> {
    validate_input_schema(input)?;
    validate_references(input)
}

fn validate_input_schema(input: &FeedReaderInputDocument) -> RuntimeResult<()> {
    if input.schema == INPUT_SCHEMA {
        return Ok(());
    }
    Err(RuntimeError::unsupported_input_schema(
        input.schema.clone(),
        INPUT_SCHEMA,
    ))
}

fn validate_references(input: &FeedReaderInputDocument) -> RuntimeResult<()> {
    let source_ids = input
        .source_feeds
        .iter()
        .map(|feed| feed.id.clone())
        .collect::<Vec<_>>();
    let known_ids = known_source_ids(input);
    for entry in &input.entries {
        ensure_known(&source_ids, &entry.source_id, "entry source")?;
    }
    for hint in &input.correspondence_hints {
        ensure_all_known(&known_ids, &hint.entry_ids, "correspondence entry")?;
    }
    for hint in &input.completion_hints {
        ensure_completion_known(&known_ids, hint)?;
    }
    for hint in &input.obstruction_hints {
        ensure_all_known(&known_ids, &hint.entry_ids, "obstruction entry")?;
    }
    for request in &input.projection_requests {
        ensure_all_known(&known_ids, &request.source_ids, "projection source")?;
    }
    Ok(())
}

fn known_source_ids(input: &FeedReaderInputDocument) -> Vec<Id> {
    let mut ids = input
        .source_feeds
        .iter()
        .map(|feed| feed.id.clone())
        .collect::<Vec<_>>();
    extend_unique(
        &mut ids,
        input
            .source_feeds
            .iter()
            .map(|feed| feed.context_id.clone()),
    );
    extend_unique(&mut ids, input.entries.iter().map(|entry| entry.id.clone()));
    extend_unique(
        &mut ids,
        input
            .entries
            .iter()
            .flat_map(|entry| entry.topic_ids.clone()),
    );
    extend_unique(
        &mut ids,
        input
            .entries
            .iter()
            .filter_map(|entry| entry.event_id.clone()),
    );
    extend_unique(
        &mut ids,
        input
            .correspondence_hints
            .iter()
            .map(|hint| hint.id.clone()),
    );
    extend_unique(
        &mut ids,
        input.completion_hints.iter().map(|hint| hint.id.clone()),
    );
    extend_unique(
        &mut ids,
        input.obstruction_hints.iter().map(|hint| hint.id.clone()),
    );
    ids
}

fn ensure_completion_known(known_ids: &[Id], hint: &FeedCompletionHint) -> RuntimeResult<()> {
    if let Some(subject_id) = &hint.subject_id {
        ensure_known(known_ids, subject_id, "completion subject")?;
    }
    if let Some(topic_id) = &hint.topic_id {
        ensure_known(known_ids, topic_id, "completion topic")?;
    }
    ensure_all_known(known_ids, &hint.inferred_from, "completion inferred_from")
}

fn ensure_all_known(known_ids: &[Id], ids: &[Id], role: &str) -> RuntimeResult<()> {
    for id in ids {
        ensure_known(known_ids, id, role)?;
    }
    Ok(())
}

fn ensure_known(known_ids: &[Id], id: &Id, role: &str) -> RuntimeResult<()> {
    if known_ids.contains(id) {
        return Ok(());
    }
    Err(validation_error(format!(
        "{role} references unknown id {id}"
    )))
}

fn extend_unique(ids: &mut Vec<Id>, new_ids: impl IntoIterator<Item = Id>) {
    for id in new_ids {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
}

fn validation_error(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::workflow_construction(WORKFLOW_NAME, reason)
}
