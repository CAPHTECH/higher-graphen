use crate::{
    store::{read_workflow_graph, StoreError, StoreResult},
    workflow_eval::validate_workflow_graph,
    workflow_model::{
        ChangeSet, WorkflowCaseGraph, WORKFLOW_GRAPH_SCHEMA, WORKFLOW_GRAPH_SCHEMA_VERSION,
    },
};
use higher_graphen_core::Id;
use serde::Serialize;
use serde_json::Map;
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

pub mod cli_bridge;
mod review;
mod types;
pub use types::*;

const WORKFLOW_DIRECTORY: &str = "workflow_graphs";

impl WorkflowHistoryEntry {
    pub fn imported(graph: &WorkflowCaseGraph, revision_id: Id) -> Self {
        Self::snapshot(
            graph,
            revision_id,
            None,
            WorkflowHistoryEventType::Imported,
            ChangeSet {
                added_ids: workflow_record_ids(graph),
                removed_ids: Vec::new(),
                updated_ids: Vec::new(),
            },
        )
    }

    pub fn snapshot(
        graph: &WorkflowCaseGraph,
        revision_id: Id,
        previous_revision_id: Option<Id>,
        event_type: WorkflowHistoryEventType,
        changed_ids: ChangeSet,
    ) -> Self {
        Self {
            schema: WORKFLOW_HISTORY_ENTRY_SCHEMA.to_owned(),
            schema_version: WORKFLOW_WORKSPACE_SCHEMA_VERSION,
            id: history_entry_id(&graph.workflow_graph_id, &revision_id, event_type),
            workflow_graph_id: graph.workflow_graph_id.clone(),
            case_graph_id: graph.case_graph_id.clone(),
            space_id: graph.space_id.clone(),
            revision_id,
            previous_revision_id,
            event_type,
            graph_path: String::new(),
            changed_ids,
            source_ids: graph
                .work_items
                .iter()
                .flat_map(|item| item.source_ids.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            recorded_at: None,
            actor_id: None,
            metadata: Map::new(),
        }
    }

    pub fn with_recorded_at(mut self, recorded_at: impl Into<String>) -> Self {
        self.recorded_at = Some(recorded_at.into());
        self
    }

    pub fn with_actor_id(mut self, actor_id: Id) -> Self {
        self.actor_id = Some(actor_id);
        self
    }
}

pub struct WorkflowWorkspaceStore {
    root: PathBuf,
}

impl WorkflowWorkspaceStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn import_graph(
        &self,
        graph: &WorkflowCaseGraph,
        revision_id: Id,
    ) -> StoreResult<WorkflowWorkspaceRecord> {
        self.append_history_entry(graph, WorkflowHistoryEntry::imported(graph, revision_id))
    }

    pub fn append_history_entry(
        &self,
        graph: &WorkflowCaseGraph,
        mut entry: WorkflowHistoryEntry,
    ) -> StoreResult<WorkflowWorkspaceRecord> {
        require_graph_contract(&self.root, graph)?;
        entry.graph_path =
            self.relative_revision_path(&entry.workflow_graph_id, &entry.revision_id);

        let graph_dir = self.graph_dir(&entry.workflow_graph_id);
        let revisions_dir = graph_dir.join("revisions");
        fs::create_dir_all(&revisions_dir).map_err(|source| StoreError::Io {
            path: revisions_dir.clone(),
            source,
        })?;

        let history_path = self.history_path(&entry.workflow_graph_id);
        let existing_entries = self.history_entries_allow_missing(&entry.workflow_graph_id)?;
        validate_append(&history_path, graph, &entry, &existing_entries)?;

        let graph_path = self.resolve_graph_path(&entry.graph_path, &history_path)?;
        write_json(&graph_path, graph)?;
        append_json_line(&history_path, &entry)?;

        let workflow_graph_id = entry.workflow_graph_id.clone();
        let mut entries = existing_entries;
        entries.push(entry);
        workspace_record(&self.root, &workflow_graph_id, &entries)
    }

    pub fn list_graphs(&self) -> StoreResult<Vec<WorkflowWorkspaceRecord>> {
        let root = self.workflow_root();
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => return Err(StoreError::Io { path: root, source }),
        };

        let mut directories = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| StoreError::Io {
                path: root.clone(),
                source,
            })?;
            if entry
                .file_type()
                .map_err(|source| StoreError::Io {
                    path: entry.path(),
                    source,
                })?
                .is_dir()
            {
                directories.push(entry.path());
            }
        }
        directories.sort();

        let mut records = Vec::new();
        for directory in directories {
            records.push(self.inspect_directory(&directory)?);
        }
        records.sort_by_key(|record| record.workflow_graph_id.as_str().to_owned());
        Ok(records)
    }

    pub fn inspect_graph(&self, workflow_graph_id: &Id) -> StoreResult<WorkflowWorkspaceRecord> {
        let entries = self.history_entries(workflow_graph_id)?;
        workspace_record(&self.root, workflow_graph_id, &entries)
    }

    pub fn history_entries(
        &self,
        workflow_graph_id: &Id,
    ) -> StoreResult<Vec<WorkflowHistoryEntry>> {
        self.read_history_entries(workflow_graph_id, false)
    }

    pub fn replay_current_graph(&self, workflow_graph_id: &Id) -> StoreResult<WorkflowReplay> {
        let entries = self.history_entries(workflow_graph_id)?;
        let latest = latest_entry(&entries, &self.history_path(workflow_graph_id))?;
        let graph_path = self.resolve_graph_path(
            &latest.graph_path,
            &self.history_path(&latest.workflow_graph_id),
        )?;
        let graph = read_workflow_graph(&graph_path)?;
        require_graph_matches_history(&graph_path, &graph, latest)?;

        Ok(WorkflowReplay {
            schema: WORKFLOW_WORKSPACE_RECORD_SCHEMA.to_owned(),
            schema_version: WORKFLOW_WORKSPACE_SCHEMA_VERSION,
            workflow_graph_id: latest.workflow_graph_id.clone(),
            case_graph_id: latest.case_graph_id.clone(),
            space_id: latest.space_id.clone(),
            current_revision_id: latest.revision_id.clone(),
            graph,
            history: entries,
        })
    }

    pub fn validate_graph(
        &self,
        workflow_graph_id: &Id,
    ) -> StoreResult<WorkflowWorkspaceValidation> {
        let replay = self.replay_current_graph(workflow_graph_id)?;
        Ok(WorkflowWorkspaceValidation {
            schema: WORKFLOW_WORKSPACE_RECORD_SCHEMA.to_owned(),
            schema_version: WORKFLOW_WORKSPACE_SCHEMA_VERSION,
            workflow_graph_id: replay.workflow_graph_id,
            current_revision_id: replay.current_revision_id,
            history_entry_count: replay.history.len() as u32,
            valid: true,
        })
    }

    fn history_entries_allow_missing(
        &self,
        workflow_graph_id: &Id,
    ) -> StoreResult<Vec<WorkflowHistoryEntry>> {
        self.read_history_entries(workflow_graph_id, true)
    }

    fn read_history_entries(
        &self,
        workflow_graph_id: &Id,
        allow_missing: bool,
    ) -> StoreResult<Vec<WorkflowHistoryEntry>> {
        let history_path = self.history_path(workflow_graph_id);
        let text = match fs::read_to_string(&history_path) {
            Ok(text) => text,
            Err(source) if allow_missing && source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            }
            Err(source) => {
                return Err(StoreError::Io {
                    path: history_path,
                    source,
                });
            }
        };
        let entries = parse_history_entries(&history_path, &text)?;
        validate_history_entries(self, Some(workflow_graph_id), &history_path, &entries)?;
        Ok(entries)
    }

    fn inspect_directory(&self, directory: &Path) -> StoreResult<WorkflowWorkspaceRecord> {
        let history_path = directory.join("history.jsonl");
        let text = fs::read_to_string(&history_path).map_err(|source| StoreError::Io {
            path: history_path.clone(),
            source,
        })?;
        let entries = parse_history_entries(&history_path, &text)?;
        validate_history_entries(self, None, &history_path, &entries)?;
        let latest = latest_entry(&entries, &history_path)?;
        workspace_record(&self.root, &latest.workflow_graph_id, &entries)
    }

    fn workflow_root(&self) -> PathBuf {
        self.root.join(WORKFLOW_DIRECTORY)
    }

    fn graph_dir(&self, workflow_graph_id: &Id) -> PathBuf {
        self.workflow_root().join(path_segment(workflow_graph_id))
    }

    fn history_path(&self, workflow_graph_id: &Id) -> PathBuf {
        self.graph_dir(workflow_graph_id).join("history.jsonl")
    }

    fn relative_revision_path(&self, workflow_graph_id: &Id, revision_id: &Id) -> String {
        format!(
            "{}/{}/revisions/{}.workflow.graph.json",
            WORKFLOW_DIRECTORY,
            path_segment(workflow_graph_id),
            path_segment(revision_id)
        )
    }

    fn resolve_graph_path(&self, relative_path: &str, history_path: &Path) -> StoreResult<PathBuf> {
        require_relative_store_path(history_path, relative_path)?;
        Ok(self.root.join(relative_path))
    }
}

fn workspace_record(
    root: &Path,
    workflow_graph_id: &Id,
    entries: &[WorkflowHistoryEntry],
) -> StoreResult<WorkflowWorkspaceRecord> {
    let latest = latest_entry(entries, &root.join(WORKFLOW_DIRECTORY))?;
    if &latest.workflow_graph_id != workflow_graph_id {
        return Err(contract_error(
            root,
            format!(
                "history for {workflow_graph_id} ended with workflow graph {}",
                latest.workflow_graph_id
            ),
        ));
    }
    let revisions = entries
        .iter()
        .map(|entry| WorkflowRevisionRecord {
            revision_id: entry.revision_id.clone(),
            previous_revision_id: entry.previous_revision_id.clone(),
            event_type: entry.event_type,
            graph_path: entry.graph_path.clone(),
            changed_ids: entry.changed_ids.clone(),
            source_ids: entry.source_ids.clone(),
        })
        .collect::<Vec<_>>();

    Ok(WorkflowWorkspaceRecord {
        schema: WORKFLOW_WORKSPACE_RECORD_SCHEMA.to_owned(),
        schema_version: WORKFLOW_WORKSPACE_SCHEMA_VERSION,
        workflow_graph_id: latest.workflow_graph_id.clone(),
        case_graph_id: latest.case_graph_id.clone(),
        space_id: latest.space_id.clone(),
        current_revision_id: latest.revision_id.clone(),
        workflow_directory: format!("{}/{}", WORKFLOW_DIRECTORY, path_segment(workflow_graph_id)),
        history_path: format!(
            "{}/{}/history.jsonl",
            WORKFLOW_DIRECTORY,
            path_segment(workflow_graph_id)
        ),
        current_graph_path: latest.graph_path.clone(),
        revision_count: revisions.len() as u32,
        history_entry_count: entries.len() as u32,
        revisions,
    })
}

fn validate_append(
    history_path: &Path,
    graph: &WorkflowCaseGraph,
    entry: &WorkflowHistoryEntry,
    existing_entries: &[WorkflowHistoryEntry],
) -> StoreResult<()> {
    require_history_entry_contract(history_path, entry)?;
    require_graph_matches_history(history_path, graph, entry)?;

    if existing_entries.is_empty() {
        if entry.event_type != WorkflowHistoryEventType::Imported {
            return Err(contract_error(
                history_path,
                "first workflow history entry must be imported",
            ));
        }
        if entry.previous_revision_id.is_some() {
            return Err(contract_error(
                history_path,
                "first workflow history entry must not set previous_revision_id",
            ));
        }
    } else {
        let previous = latest_entry(existing_entries, history_path)?;
        if entry.previous_revision_id.as_ref() != Some(&previous.revision_id) {
            return Err(contract_error(
                history_path,
                format!(
                    "history previous_revision_id must be {}; got {:?}",
                    previous.revision_id, entry.previous_revision_id
                ),
            ));
        }
    }

    if existing_entries
        .iter()
        .any(|existing| existing.revision_id == entry.revision_id)
    {
        return Err(contract_error(
            history_path,
            format!("duplicate workflow revision {}", entry.revision_id),
        ));
    }

    Ok(())
}

fn validate_history_entries(
    store: &WorkflowWorkspaceStore,
    expected_workflow_graph_id: Option<&Id>,
    history_path: &Path,
    entries: &[WorkflowHistoryEntry],
) -> StoreResult<()> {
    if entries.is_empty() {
        return Err(contract_error(history_path, "workflow history is empty"));
    }

    let mut seen_revisions = BTreeSet::new();
    let mut previous_revision_id: Option<Id> = None;
    for (index, entry) in entries.iter().enumerate() {
        require_history_entry_contract(history_path, entry)?;
        if let Some(expected_id) = expected_workflow_graph_id {
            if &entry.workflow_graph_id != expected_id {
                return Err(contract_error(
                    history_path,
                    format!(
                        "history entry {} belongs to {}, expected {}",
                        entry.id, entry.workflow_graph_id, expected_id
                    ),
                ));
            }
        }
        validate_history_chain(history_path, index, entry, previous_revision_id.as_ref())?;
        if !seen_revisions.insert(entry.revision_id.clone()) {
            return Err(contract_error(
                history_path,
                format!("duplicate workflow revision {}", entry.revision_id),
            ));
        }
        let graph_path = store.resolve_graph_path(&entry.graph_path, history_path)?;
        let graph = read_workflow_graph(&graph_path)?;
        require_graph_matches_history(&graph_path, &graph, entry)?;
        previous_revision_id = Some(entry.revision_id.clone());
    }
    Ok(())
}

fn validate_history_chain(
    history_path: &Path,
    index: usize,
    entry: &WorkflowHistoryEntry,
    previous_revision_id: Option<&Id>,
) -> StoreResult<()> {
    if index == 0 {
        if entry.event_type != WorkflowHistoryEventType::Imported {
            return Err(contract_error(
                history_path,
                "first workflow history entry must be imported",
            ));
        }
        if entry.previous_revision_id.is_some() {
            return Err(contract_error(
                history_path,
                "first workflow history entry must not set previous_revision_id",
            ));
        }
        return Ok(());
    }

    if entry.previous_revision_id.as_ref() != previous_revision_id {
        return Err(contract_error(
            history_path,
            format!(
                "history entry {} has previous_revision_id {:?}, expected {:?}",
                entry.id, entry.previous_revision_id, previous_revision_id
            ),
        ));
    }
    Ok(())
}

fn require_graph_contract(path: &Path, graph: &WorkflowCaseGraph) -> StoreResult<()> {
    if graph.schema != WORKFLOW_GRAPH_SCHEMA {
        return Err(contract_error(
            path,
            format!(
                "unsupported workflow schema {:?}; expected {:?}",
                graph.schema, WORKFLOW_GRAPH_SCHEMA
            ),
        ));
    }
    if graph.schema_version != WORKFLOW_GRAPH_SCHEMA_VERSION {
        return Err(contract_error(
            path,
            format!(
                "unsupported workflow schema version {}; expected {}",
                graph.schema_version, WORKFLOW_GRAPH_SCHEMA_VERSION
            ),
        ));
    }
    validate_workflow_graph(graph).map_err(|source| StoreError::Validation {
        path: path.to_owned(),
        source,
    })
}

fn require_history_entry_contract(path: &Path, entry: &WorkflowHistoryEntry) -> StoreResult<()> {
    if entry.schema != WORKFLOW_HISTORY_ENTRY_SCHEMA {
        return Err(contract_error(
            path,
            format!(
                "unsupported workflow history schema {:?}; expected {:?}",
                entry.schema, WORKFLOW_HISTORY_ENTRY_SCHEMA
            ),
        ));
    }
    if entry.schema_version != WORKFLOW_WORKSPACE_SCHEMA_VERSION {
        return Err(contract_error(
            path,
            format!(
                "unsupported workflow history schema version {}; expected {}",
                entry.schema_version, WORKFLOW_WORKSPACE_SCHEMA_VERSION
            ),
        ));
    }
    require_relative_store_path(path, &entry.graph_path)
}

fn require_graph_matches_history(
    path: &Path,
    graph: &WorkflowCaseGraph,
    entry: &WorkflowHistoryEntry,
) -> StoreResult<()> {
    if graph.workflow_graph_id != entry.workflow_graph_id {
        return Err(contract_error(
            path,
            format!(
                "workflow graph id {} does not match history entry {}",
                graph.workflow_graph_id, entry.workflow_graph_id
            ),
        ));
    }
    if graph.case_graph_id != entry.case_graph_id {
        return Err(contract_error(
            path,
            format!(
                "case graph id {} does not match history entry {}",
                graph.case_graph_id, entry.case_graph_id
            ),
        ));
    }
    if graph.space_id != entry.space_id {
        return Err(contract_error(
            path,
            format!(
                "space id {} does not match history entry {}",
                graph.space_id, entry.space_id
            ),
        ));
    }
    Ok(())
}

fn parse_history_entries(path: &Path, text: &str) -> StoreResult<Vec<WorkflowHistoryEntry>> {
    let mut entries = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        entries.push(
            serde_json::from_str(line).map_err(|source| StoreError::Json {
                path: path.to_owned(),
                source,
            })?,
        );
    }
    Ok(entries)
}

fn append_json_line(path: &Path, value: &impl Serialize) -> StoreResult<()> {
    let text = serde_json::to_string(value).map_err(|source| StoreError::Json {
        path: path.to_owned(),
        source,
    })?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| StoreError::Io {
            path: path.to_owned(),
            source,
        })?;
    writeln!(file, "{text}").map_err(|source| StoreError::Io {
        path: path.to_owned(),
        source,
    })
}

fn write_json(path: &Path, value: &impl Serialize) -> StoreResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| StoreError::Io {
            path: parent.to_owned(),
            source,
        })?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|source| StoreError::Json {
        path: path.to_owned(),
        source,
    })?;
    fs::write(path, format!("{text}\n")).map_err(|source| StoreError::Io {
        path: path.to_owned(),
        source,
    })
}

fn latest_entry<'a>(
    entries: &'a [WorkflowHistoryEntry],
    path: &Path,
) -> StoreResult<&'a WorkflowHistoryEntry> {
    entries
        .last()
        .ok_or_else(|| contract_error(path, "workflow history is empty"))
}

fn require_relative_store_path(path: &Path, value: &str) -> StoreResult<()> {
    let candidate = Path::new(value);
    if value.trim().is_empty() {
        return Err(contract_error(path, "history graph_path is empty"));
    }
    for component in candidate.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(contract_error(
                path,
                format!("history graph_path {value:?} must stay inside the workflow store"),
            ));
        }
    }
    Ok(())
}

fn workflow_record_ids(graph: &WorkflowCaseGraph) -> Vec<Id> {
    let mut ids = graph
        .work_items
        .iter()
        .map(|item| item.id.clone())
        .chain(
            graph
                .workflow_relations
                .iter()
                .map(|relation| relation.id.clone()),
        )
        .chain(graph.readiness_rules.iter().map(|rule| rule.id.clone()))
        .chain(
            graph
                .evidence_records
                .iter()
                .map(|record| record.id.clone()),
        )
        .chain(
            graph
                .completion_reviews
                .iter()
                .map(|record| record.id.clone()),
        )
        .chain(
            graph
                .transition_records
                .iter()
                .map(|record| record.id.clone()),
        )
        .chain(
            graph
                .projection_profiles
                .iter()
                .map(|profile| profile.id.clone()),
        )
        .chain(
            graph
                .correspondence_records
                .iter()
                .map(|record| record.id.clone()),
        )
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn history_entry_id(
    workflow_graph_id: &Id,
    revision_id: &Id,
    event_type: WorkflowHistoryEventType,
) -> Id {
    Id::new(format!(
        "workflow_history:{}:{}:{}",
        path_segment(workflow_graph_id),
        path_segment(revision_id),
        event_type_id_segment(event_type)
    ))
    .expect("history entry id is derived from validated non-empty ids")
}

fn event_type_id_segment(event_type: WorkflowHistoryEventType) -> &'static str {
    match event_type {
        WorkflowHistoryEventType::Imported => "imported",
        WorkflowHistoryEventType::Snapshot => "snapshot",
        WorkflowHistoryEventType::Transition => "transition",
        WorkflowHistoryEventType::Patch => "patch",
        WorkflowHistoryEventType::Review => "review",
        WorkflowHistoryEventType::Validation => "validation",
    }
}

fn path_segment(id: &Id) -> String {
    let mut segment = String::new();
    for byte in id.as_str().bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' => {
                segment.push(byte as char);
            }
            _ => segment.push_str(&format!("~{byte:02x}")),
        }
    }
    segment
}

fn contract_error(path: &Path, reason: impl Into<String>) -> StoreError {
    StoreError::Contract {
        path: path.to_owned(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
