//! Bounded Feed Product reader workflow.

use crate::error::{RuntimeError, RuntimeResult};
use crate::feed_reports::{
    FeedCompletionCandidate, FeedCompletionHint, FeedCorrespondence, FeedEntry, FeedEntryCell,
    FeedEventCell, FeedMissingType, FeedObstruction, FeedObstructionHint, FeedObstructionSeverity,
    FeedObstructionType, FeedReaderInputDocument, FeedReaderReport, FeedReaderResult,
    FeedReaderScenario, FeedReaderStatus, FeedReportSourceFeed, FeedReportSpace,
    FeedSuggestedStructure, FeedTopicCell,
};
use crate::reports::{ReportEnvelope, ReportMetadata};
use crate::workflows::feed_reader_projection::report_projection;
use crate::workflows::feed_reader_validation::validate_input;
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, Severity, SourceRef};
use higher_graphen_reasoning::completion::{CompletionCandidate, MissingType, SuggestedStructure};
use higher_graphen_reasoning::obstruction::{Obstruction, ObstructionExplanation, ObstructionType};
use higher_graphen_structure::space::{
    Cell, ComplexType, InMemorySpaceStore, Incidence, IncidenceOrientation, Space,
};
use std::collections::BTreeMap;

const WORKFLOW_NAME: &str = "feed_reader";
const REPORT_SCHEMA: &str = "highergraphen.feed.reader.report.v1";
const REPORT_TYPE: &str = "feed_reader";
const REPORT_VERSION: u32 = 1;
const EXTRACTION_METHOD: &str = "feed_reader_fixture.v1";

/// Runs the bounded Feed Product reader workflow.
pub fn run_feed_reader(input: FeedReaderInputDocument) -> RuntimeResult<FeedReaderReport> {
    validate_input(&input)?;

    let lifted = lift_feed_input(&input)?;
    let scenario = report_scenario(&input, &lifted);
    let result = report_result(&input, &lifted);
    let projection = report_projection(&input, &scenario, &result)?;

    Ok(ReportEnvelope {
        schema: REPORT_SCHEMA.to_owned(),
        report_type: REPORT_TYPE.to_owned(),
        report_version: REPORT_VERSION,
        metadata: ReportMetadata::feed_reader(),
        scenario,
        result,
        projection,
    })
}

struct LiftedFeed {
    space: Space,
    entry_cells: Vec<Cell>,
    topic_cells: Vec<Cell>,
    event_cells: Vec<Cell>,
    completion_candidates: Vec<CompletionCandidate>,
    obstructions: Vec<Obstruction>,
}

fn lift_feed_input(input: &FeedReaderInputDocument) -> RuntimeResult<LiftedFeed> {
    let mut store = InMemorySpaceStore::new();
    store.insert_space(input_space(input))?;

    let entry_cells = insert_entry_cells(&mut store, input)?;
    let topic_cells = insert_group_cells(&mut store, input, GroupKind::Topic)?;
    let event_cells = insert_group_cells(&mut store, input, GroupKind::Event)?;
    let incidences = insert_correspondence_incidences(&mut store, input)?;
    construct_complex(
        &mut store,
        input,
        &entry_cells,
        &topic_cells,
        &event_cells,
        &incidences,
    )?;
    let completion_candidates = completion_candidates(input)?;
    let obstructions = obstructions(input)?;

    let space = store
        .space(&input.space.id)
        .ok_or_else(|| validation_error("space was not available after insertion"))?
        .clone();

    Ok(LiftedFeed {
        space,
        entry_cells,
        topic_cells,
        event_cells,
        completion_candidates,
        obstructions,
    })
}

fn input_space(input: &FeedReaderInputDocument) -> Space {
    let space = Space::new(input.space.id.clone(), input.space.name.clone());
    match &input.space.description {
        Some(description) => space.with_description(description.clone()),
        None => space,
    }
}

fn insert_entry_cells(
    store: &mut InMemorySpaceStore,
    input: &FeedReaderInputDocument,
) -> RuntimeResult<Vec<Cell>> {
    let mut cells = Vec::with_capacity(input.entries.len());
    for entry in &input.entries {
        let cell = entry_cell(input, entry)?;
        cells.push(store.insert_cell(cell)?);
    }
    Ok(cells)
}

fn entry_cell(input: &FeedReaderInputDocument, entry: &FeedEntry) -> RuntimeResult<Cell> {
    let confidence = entry.confidence.unwrap_or(input.source.confidence);
    let provenance = fact_provenance(input, confidence, entry.source_local_id.as_deref());
    Ok(
        Cell::new(entry.id.clone(), input.space.id.clone(), 0, "feed_entry")
            .with_label(entry.title.clone())
            .with_context(source_context(input, &entry.source_id)?)
            .with_provenance(provenance),
    )
}

#[derive(Clone, Copy)]
enum GroupKind {
    Topic,
    Event,
}

fn insert_group_cells(
    store: &mut InMemorySpaceStore,
    input: &FeedReaderInputDocument,
    group_kind: GroupKind,
) -> RuntimeResult<Vec<Cell>> {
    let grouped = grouped_entries(input, group_kind);
    let mut cells = Vec::with_capacity(grouped.len());
    for (group_id, entry_ids) in grouped {
        let cell = group_cell(input, group_kind, group_id, entry_ids)?;
        cells.push(store.insert_cell(cell)?);
    }
    Ok(cells)
}

fn group_cell(
    input: &FeedReaderInputDocument,
    group_kind: GroupKind,
    group_id: Id,
    entry_ids: Vec<Id>,
) -> RuntimeResult<Cell> {
    let cell_type = match group_kind {
        GroupKind::Topic => "feed_topic",
        GroupKind::Event => "feed_event",
    };
    let mut cell = Cell::new(group_id.clone(), input.space.id.clone(), 1, cell_type)
        .with_label(label_from_id(&group_id))
        .with_provenance(inferred_provenance(
            input,
            confidence_average(input, &entry_ids),
        ));
    for entry_id in entry_ids {
        cell = cell.with_boundary_cell(entry_id);
    }
    Ok(cell)
}

fn insert_correspondence_incidences(
    store: &mut InMemorySpaceStore,
    input: &FeedReaderInputDocument,
) -> RuntimeResult<Vec<Incidence>> {
    let mut incidences = Vec::new();
    for hint in &input.correspondence_hints {
        for incidence in correspondence_incidences(input, hint)? {
            incidences.push(store.insert_incidence(incidence)?);
        }
    }
    Ok(incidences)
}

fn correspondence_incidences(
    input: &FeedReaderInputDocument,
    hint: &crate::feed_reports::FeedCorrespondenceHint,
) -> RuntimeResult<Vec<Incidence>> {
    let Some((first, rest)) = hint.entry_ids.split_first() else {
        return Ok(Vec::new());
    };
    let mut incidences = Vec::with_capacity(rest.len());
    for (index, target) in rest.iter().enumerate() {
        let id = correspondence_incidence_id(hint, index)?;
        let incidence = Incidence::new(
            id,
            input.space.id.clone(),
            first.clone(),
            target.clone(),
            correspondence_type_name(hint.hint_type),
            IncidenceOrientation::Undirected,
        )
        .with_weight(hint.confidence.value())
        .with_provenance(inferred_provenance(input, hint.confidence));
        incidences.push(incidence);
    }
    Ok(incidences)
}

fn construct_complex(
    store: &mut InMemorySpaceStore,
    input: &FeedReaderInputDocument,
    entries: &[Cell],
    topics: &[Cell],
    events: &[Cell],
    incidences: &[Incidence],
) -> RuntimeResult<()> {
    let cells = entries.iter().chain(topics).chain(events);
    store.construct_complex(
        id(format!("complex:feed-reader:{}", input.space.id))?,
        input.space.id.clone(),
        format!("{} feed observation graph", input.space.name),
        ComplexType::TypedGraph,
        cells.map(|cell| cell.id.clone()),
        incidences.iter().map(|incidence| incidence.id.clone()),
    )?;
    Ok(())
}

fn completion_candidates(
    input: &FeedReaderInputDocument,
) -> RuntimeResult<Vec<CompletionCandidate>> {
    input
        .completion_hints
        .iter()
        .map(|hint| completion_candidate(input, hint))
        .collect()
}

fn completion_candidate(
    input: &FeedReaderInputDocument,
    hint: &FeedCompletionHint,
) -> RuntimeResult<CompletionCandidate> {
    let suggested = SuggestedStructure::new(suggested_structure_type(hint), hint.summary.clone())?
        .with_related_ids(completion_related_ids(hint));
    Ok(CompletionCandidate::new(
        hint.id.clone(),
        input.space.id.clone(),
        core_missing_type(hint.missing_type),
        suggested,
        hint.inferred_from.clone(),
        hint.rationale.clone(),
        hint.confidence,
    )?)
}

fn obstructions(input: &FeedReaderInputDocument) -> RuntimeResult<Vec<Obstruction>> {
    input
        .obstruction_hints
        .iter()
        .map(|hint| obstruction(input, hint))
        .collect()
}

fn obstruction(
    input: &FeedReaderInputDocument,
    hint: &FeedObstructionHint,
) -> RuntimeResult<Obstruction> {
    let explanation =
        ObstructionExplanation::new(hint.summary.clone())?.with_details(hint.rationale.clone())?;
    let mut obstruction = Obstruction::new(
        hint.id.clone(),
        input.space.id.clone(),
        ObstructionType::custom(obstruction_type_name(hint.obstruction_type))?,
        explanation,
        core_severity(hint.severity),
        inferred_provenance(input, hint.confidence),
    );
    for entry_id in &hint.entry_ids {
        obstruction = obstruction.with_location_cell(entry_id.clone());
    }
    Ok(obstruction)
}

fn report_scenario(input: &FeedReaderInputDocument, lifted: &LiftedFeed) -> FeedReaderScenario {
    let space = report_space(input, lifted);
    FeedReaderScenario {
        input_schema: input.schema.clone(),
        source: input.source.clone(),
        space,
        source_feeds: report_source_feeds(input),
        entries: input
            .entries
            .iter()
            .map(|entry| entry_report(input, entry))
            .collect(),
        topics: topic_reports(lifted),
        events: event_reports(lifted),
    }
}

fn report_result(input: &FeedReaderInputDocument, lifted: &LiftedFeed) -> FeedReaderResult {
    let status = match lifted.obstructions.is_empty() {
        true => FeedReaderStatus::Analyzed,
        false => FeedReaderStatus::ObstructionsDetected,
    };
    FeedReaderResult {
        status,
        observed_entry_ids: input.entries.iter().map(|entry| entry.id.clone()).collect(),
        inferred_topic_ids: lifted
            .topic_cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect(),
        inferred_event_ids: lifted
            .event_cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect(),
        correspondences: input
            .correspondence_hints
            .iter()
            .map(correspondence_report)
            .collect(),
        completion_candidates: completion_reports(input, lifted),
        obstructions: input
            .obstruction_hints
            .iter()
            .map(obstruction_report)
            .collect(),
    }
}

fn report_space(input: &FeedReaderInputDocument, lifted: &LiftedFeed) -> FeedReportSpace {
    FeedReportSpace {
        id: lifted.space.id.clone(),
        name: lifted.space.name.clone(),
        description: lifted.space.description.clone(),
        source_feed_ids: input
            .source_feeds
            .iter()
            .map(|feed| feed.id.clone())
            .collect(),
        entry_ids: lifted
            .entry_cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect(),
        topic_ids: lifted
            .topic_cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect(),
        event_ids: lifted
            .event_cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect(),
        context_ids: input
            .source_feeds
            .iter()
            .map(|feed| feed.context_id.clone())
            .collect(),
    }
}

fn report_source_feeds(input: &FeedReaderInputDocument) -> Vec<FeedReportSourceFeed> {
    input
        .source_feeds
        .iter()
        .map(|feed| FeedReportSourceFeed {
            id: feed.id.clone(),
            context_id: feed.context_id.clone(),
            title: feed.title.clone(),
            kind: feed.kind,
            url: feed.url.clone(),
            review_status: ReviewStatus::Accepted,
            confidence: feed.confidence,
        })
        .collect()
}

fn entry_report(input: &FeedReaderInputDocument, entry: &FeedEntry) -> FeedEntryCell {
    FeedEntryCell {
        id: entry.id.clone(),
        space_id: input.space.id.clone(),
        source_id: entry.source_id.clone(),
        dimension: 0,
        cell_type: "feed_entry".to_owned(),
        label: entry.title.clone(),
        published_at: entry.published_at.clone(),
        updated_at: entry.updated_at.clone(),
        summary: Some(entry.summary.clone()),
        topic_ids: entry.topic_ids.clone(),
        event_id: entry.event_id.clone(),
        review_status: ReviewStatus::Accepted,
        confidence: Some(entry.confidence.unwrap_or(input.source.confidence)),
    }
}

fn topic_reports(lifted: &LiftedFeed) -> Vec<FeedTopicCell> {
    lifted
        .topic_cells
        .iter()
        .map(|cell| FeedTopicCell {
            id: cell.id.clone(),
            space_id: cell.space_id.clone(),
            cell_type: cell.cell_type.clone(),
            label: cell
                .label
                .clone()
                .unwrap_or_else(|| label_from_id(&cell.id)),
            entry_ids: cell.boundary.clone(),
            review_status: review_status(cell),
            confidence: cell
                .provenance
                .as_ref()
                .map(|provenance| provenance.confidence),
        })
        .collect()
}

fn event_reports(lifted: &LiftedFeed) -> Vec<FeedEventCell> {
    lifted
        .event_cells
        .iter()
        .map(|cell| FeedEventCell {
            id: cell.id.clone(),
            space_id: cell.space_id.clone(),
            cell_type: cell.cell_type.clone(),
            label: cell
                .label
                .clone()
                .unwrap_or_else(|| label_from_id(&cell.id)),
            entry_ids: cell.boundary.clone(),
            review_status: review_status(cell),
            confidence: cell
                .provenance
                .as_ref()
                .map(|provenance| provenance.confidence),
        })
        .collect()
}

fn correspondence_report(hint: &crate::feed_reports::FeedCorrespondenceHint) -> FeedCorrespondence {
    FeedCorrespondence {
        id: hint.id.clone(),
        correspondence_type: hint.hint_type,
        entry_ids: hint.entry_ids.clone(),
        topic_id: hint.topic_id.clone(),
        event_id: hint.event_id.clone(),
        summary: hint.summary.clone(),
        confidence: hint.confidence,
    }
}

fn completion_reports(
    input: &FeedReaderInputDocument,
    lifted: &LiftedFeed,
) -> Vec<FeedCompletionCandidate> {
    input
        .completion_hints
        .iter()
        .zip(&lifted.completion_candidates)
        .map(|(hint, candidate)| completion_report(hint, candidate))
        .collect()
}

fn completion_report(
    hint: &FeedCompletionHint,
    candidate: &CompletionCandidate,
) -> FeedCompletionCandidate {
    FeedCompletionCandidate {
        id: candidate.id.clone(),
        space_id: candidate.space_id.clone(),
        missing_type: hint.missing_type,
        suggested_structure: FeedSuggestedStructure {
            structure_id: candidate.suggested_structure.structure_id.clone(),
            structure_type: candidate.suggested_structure.structure_type.clone(),
            summary: candidate.suggested_structure.summary.clone(),
            related_ids: candidate.suggested_structure.related_ids.clone(),
        },
        inferred_from: candidate.inferred_from.clone(),
        rationale: candidate.rationale.clone(),
        confidence: candidate.confidence,
        review_status: candidate.review_status,
    }
}

fn obstruction_report(hint: &FeedObstructionHint) -> FeedObstruction {
    FeedObstruction {
        id: hint.id.clone(),
        obstruction_type: hint.obstruction_type,
        entry_ids: hint.entry_ids.clone(),
        summary: hint.summary.clone(),
        severity: hint.severity,
        source_ids: hint.entry_ids.clone(),
        confidence: hint.confidence,
    }
}

fn grouped_entries(
    input: &FeedReaderInputDocument,
    group_kind: GroupKind,
) -> BTreeMap<Id, Vec<Id>> {
    let mut grouped = BTreeMap::new();
    for entry in &input.entries {
        match group_kind {
            GroupKind::Topic => {
                for topic_id in &entry.topic_ids {
                    grouped
                        .entry(topic_id.clone())
                        .or_insert_with(Vec::new)
                        .push(entry.id.clone());
                }
            }
            GroupKind::Event => {
                if let Some(event_id) = &entry.event_id {
                    grouped
                        .entry(event_id.clone())
                        .or_insert_with(Vec::new)
                        .push(entry.id.clone());
                }
            }
        }
    }
    grouped
}

fn source_context(input: &FeedReaderInputDocument, source_id: &Id) -> RuntimeResult<Id> {
    input
        .source_feeds
        .iter()
        .find(|feed| &feed.id == source_id)
        .map(|feed| feed.context_id.clone())
        .ok_or_else(|| validation_error(format!("unknown source feed {source_id}")))
}

fn fact_provenance(
    input: &FeedReaderInputDocument,
    confidence: Confidence,
    source_local_id: Option<&str>,
) -> Provenance {
    let mut provenance = Provenance::new(source_ref(input, source_local_id), confidence)
        .with_review_status(ReviewStatus::Accepted);
    provenance.extraction_method = Some(EXTRACTION_METHOD.to_owned());
    provenance
}

fn inferred_provenance(input: &FeedReaderInputDocument, confidence: Confidence) -> Provenance {
    let mut provenance = Provenance::new(source_ref(input, None), confidence)
        .with_review_status(ReviewStatus::Unreviewed);
    provenance.extraction_method = Some(EXTRACTION_METHOD.to_owned());
    provenance
}

fn source_ref(input: &FeedReaderInputDocument, source_local_id: Option<&str>) -> SourceRef {
    SourceRef {
        kind: input.source.kind.clone(),
        uri: input.source.uri.clone(),
        title: input.source.title.clone(),
        captured_at: input.source.captured_at.clone(),
        source_local_id: source_local_id
            .map(ToOwned::to_owned)
            .or_else(|| input.source.source_local_id.clone()),
    }
}

fn correspondence_incidence_id(
    hint: &crate::feed_reports::FeedCorrespondenceHint,
    index: usize,
) -> RuntimeResult<Id> {
    if hint.entry_ids.len() == 2 {
        return Ok(hint.id.clone());
    }
    id(format!("{}:{index}", hint.id))
}

fn completion_related_ids(hint: &FeedCompletionHint) -> Vec<Id> {
    let mut ids = Vec::new();
    if let Some(subject_id) = &hint.subject_id {
        push_unique(&mut ids, subject_id.clone());
    }
    if let Some(topic_id) = &hint.topic_id {
        push_unique(&mut ids, topic_id.clone());
    }
    extend_unique(&mut ids, hint.inferred_from.clone());
    ids
}

fn suggested_structure_type(hint: &FeedCompletionHint) -> &'static str {
    match hint.missing_type {
        FeedMissingType::OfficialSource => "feed_official_source",
        FeedMissingType::Counterpoint => "feed_counterpoint",
        FeedMissingType::EntryMetadata => "feed_entry_metadata",
        FeedMissingType::TimelineContext => "feed_timeline_context",
        FeedMissingType::SourceFeed => "feed_source_feed",
    }
}

fn core_missing_type(missing_type: FeedMissingType) -> MissingType {
    match missing_type {
        FeedMissingType::OfficialSource | FeedMissingType::SourceFeed => MissingType::Context,
        FeedMissingType::Counterpoint => MissingType::Projection,
        FeedMissingType::EntryMetadata | FeedMissingType::TimelineContext => MissingType::Cell,
    }
}

fn core_severity(severity: FeedObstructionSeverity) -> Severity {
    match severity {
        FeedObstructionSeverity::Info => Severity::Low,
        FeedObstructionSeverity::Warning => Severity::Medium,
        FeedObstructionSeverity::Error => Severity::High,
        FeedObstructionSeverity::Critical => Severity::Critical,
    }
}

fn correspondence_type_name(kind: crate::feed_reports::FeedCorrespondenceType) -> &'static str {
    match kind {
        crate::feed_reports::FeedCorrespondenceType::SameTopic => "same_topic",
        crate::feed_reports::FeedCorrespondenceType::Duplicate => "duplicate",
        crate::feed_reports::FeedCorrespondenceType::FollowUp => "follow_up",
        crate::feed_reports::FeedCorrespondenceType::Counterpoint => "counterpoint",
    }
}

fn obstruction_type_name(kind: FeedObstructionType) -> &'static str {
    match kind {
        FeedObstructionType::FeedContentConflict => "feed_content_conflict",
        FeedObstructionType::TimestampConflict => "timestamp_conflict",
        FeedObstructionType::BrokenSourceReference => "broken_source_reference",
        FeedObstructionType::TopicMismatch => "topic_mismatch",
    }
}

fn confidence_average(input: &FeedReaderInputDocument, entry_ids: &[Id]) -> Confidence {
    let mut total = 0.0;
    let mut count = 0.0;
    for entry in &input.entries {
        if entry_ids.contains(&entry.id) {
            total += entry.confidence.unwrap_or(input.source.confidence).value();
            count += 1.0;
        }
    }
    Confidence::new(total / count).expect("averaged confidence remains between 0 and 1")
}

fn review_status(cell: &Cell) -> ReviewStatus {
    cell.provenance
        .as_ref()
        .map_or(ReviewStatus::Unreviewed, |provenance| {
            provenance.review_status
        })
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

fn label_from_id(id: &Id) -> String {
    id.as_str()
        .rsplit(':')
        .next()
        .unwrap_or(id.as_str())
        .replace('-', " ")
}

fn id(value: impl Into<String>) -> RuntimeResult<Id> {
    Ok(Id::new(value)?)
}

fn validation_error(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::workflow_construction(WORKFLOW_NAME, reason)
}
