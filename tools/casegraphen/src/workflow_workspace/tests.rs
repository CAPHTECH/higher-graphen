use super::*;
use crate::workflow_model::{WorkItemState, WorkflowCaseGraph};
use std::time::{SystemTime, UNIX_EPOCH};

const WORKFLOW_EXAMPLE: &str =
    include_str!("../../../../schemas/casegraphen/workflow.graph.example.json");

#[test]
fn workspace_store_imports_lists_inspects_replays_and_validates() {
    let root = unique_temp_dir("import-list-inspect-replay");
    let store = WorkflowWorkspaceStore::new(root.clone());
    let graph = workflow_graph();
    let revision_id = id("revision:workflow-import");

    let record = store
        .import_graph(&graph, revision_id.clone())
        .expect("import workflow graph");

    assert_eq!(record.workflow_graph_id, graph.workflow_graph_id);
    assert_eq!(record.current_revision_id, revision_id);
    assert_eq!(record.revision_count, 1);
    assert_eq!(record.history_entry_count, 1);
    assert!(root.join(&record.current_graph_path).exists());

    let listed = store.list_graphs().expect("list workflow graphs");
    assert_eq!(listed, vec![record.clone()]);
    let inspected = store
        .inspect_graph(&graph.workflow_graph_id)
        .expect("inspect workflow graph");
    assert_eq!(inspected, record);

    let replay = store
        .replay_current_graph(&graph.workflow_graph_id)
        .expect("replay current graph");
    assert_eq!(replay.graph, graph);
    assert_eq!(replay.history.len(), 1);

    let validation = store
        .validate_graph(&replay.workflow_graph_id)
        .expect("validate workspace graph");
    assert!(validation.valid);
    assert_eq!(validation.history_entry_count, 1);

    remove_temp_dir(&root);
}

#[test]
fn workspace_store_appends_and_reads_history_entries() {
    let root = unique_temp_dir("append-history");
    let store = WorkflowWorkspaceStore::new(root.clone());
    let mut graph = workflow_graph();
    let first_revision_id = id("revision:workflow-import");
    store
        .import_graph(&graph, first_revision_id.clone())
        .expect("import workflow graph");

    graph.work_items[1].state = WorkItemState::Done;
    let second_revision_id = id("revision:workflow-proof-done");
    let entry = WorkflowHistoryEntry::snapshot(
        &graph,
        second_revision_id.clone(),
        Some(first_revision_id.clone()),
        WorkflowHistoryEventType::Snapshot,
        ChangeSet {
            added_ids: Vec::new(),
            removed_ids: Vec::new(),
            updated_ids: vec![id("proof:workflow-schema-parse-check")],
        },
    );
    let record = store
        .append_history_entry(&graph, entry)
        .expect("append workflow history entry");

    assert_eq!(record.current_revision_id, second_revision_id);
    assert_eq!(record.history_entry_count, 2);

    let history = store
        .history_entries(&graph.workflow_graph_id)
        .expect("read workflow history entries");
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].previous_revision_id, Some(first_revision_id));

    let replay = store
        .replay_current_graph(&graph.workflow_graph_id)
        .expect("replay appended graph");
    assert_eq!(replay.graph.work_items[1].state, WorkItemState::Done);

    remove_temp_dir(&root);
}

#[test]
fn workspace_store_rejects_unsupported_workflow_schema_version() {
    let root = unique_temp_dir("unsupported-schema");
    let store = WorkflowWorkspaceStore::new(root.clone());
    let mut graph = workflow_graph();
    graph.schema_version = WORKFLOW_GRAPH_SCHEMA_VERSION + 1;

    let error = store
        .import_graph(&graph, id("revision:bad-schema"))
        .expect_err("unsupported workflow schema");

    assert!(error
        .to_string()
        .contains("unsupported workflow schema version"));

    remove_temp_dir(&root);
}

#[test]
fn workspace_store_rejects_malformed_history_json() {
    let root = unique_temp_dir("malformed-history");
    let store = WorkflowWorkspaceStore::new(root.clone());
    let graph = workflow_graph();
    let record = store
        .import_graph(&graph, id("revision:workflow-import"))
        .expect("import workflow graph");
    fs::write(root.join(&record.history_path), "{not json}\n").expect("corrupt history");

    let error = store
        .history_entries(&graph.workflow_graph_id)
        .expect_err("malformed history");

    assert!(matches!(error, StoreError::Json { .. }));

    remove_temp_dir(&root);
}

#[test]
fn workspace_store_rejects_unsupported_history_schema() {
    let root = unique_temp_dir("bad-history-schema");
    let store = WorkflowWorkspaceStore::new(root.clone());
    let graph = workflow_graph();
    let record = store
        .import_graph(&graph, id("revision:workflow-import"))
        .expect("import workflow graph");
    let mut history = store
        .history_entries(&graph.workflow_graph_id)
        .expect("read history entries");
    history[0].schema = "highergraphen.case.workflow.history_entry.v0".to_owned();
    let text = format!(
        "{}\n",
        serde_json::to_string(&history[0]).expect("serialize corrupted history")
    );
    fs::write(root.join(&record.history_path), text).expect("write corrupted history");

    let error = store
        .history_entries(&graph.workflow_graph_id)
        .expect_err("unsupported history schema");

    assert!(error
        .to_string()
        .contains("unsupported workflow history schema"));

    remove_temp_dir(&root);
}

fn workflow_graph() -> WorkflowCaseGraph {
    serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example")
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id")
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time since epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "casegraphen-workflow-workspace-{label}-{}-{nanos}",
        std::process::id()
    ))
}

fn remove_temp_dir(path: &Path) {
    match fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("remove temp store: {error}"),
    }
}
