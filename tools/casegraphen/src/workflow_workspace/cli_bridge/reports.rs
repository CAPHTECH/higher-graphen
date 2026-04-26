use super::{BridgeResult, BridgeWorkflowSource, WorkflowBridgeError};
use crate::{
    store::{read_projection, read_workflow_graph},
    workflow_eval::evaluate_readiness,
    workflow_model::CompletionReviewAction,
    workflow_report,
    workflow_workspace::{
        WorkflowCompletionPatchRequest, WorkflowCompletionReviewRequest, WorkflowPatchReviewAction,
        WorkflowPatchReviewRequest, WorkflowWorkspaceStore,
    },
};
use higher_graphen_core::Id;
use serde_json::{json, Value};
use std::{fmt, path::Path};

pub(super) fn import_json(
    store_root: &Path,
    input: &Path,
    revision_id: &str,
) -> BridgeResult<String> {
    let graph = read_workflow_graph(input)?;
    let record = WorkflowWorkspaceStore::new(store_root.to_owned())
        .import_graph(&graph, id(revision_id)?)?;
    report_json(
        "casegraphen cg workflow import",
        "workspace_import",
        json!({
            "store": store_root.display().to_string(),
            "path": input.display().to_string(),
            "revision_id": revision_id
        }),
        serde_json::to_value(record)?,
    )
}

pub(super) fn list_json(store_root: &Path) -> BridgeResult<String> {
    let records = WorkflowWorkspaceStore::new(store_root.to_owned()).list_graphs()?;
    let graph_count = records.len();
    report_json(
        "casegraphen cg workflow list",
        "workspace_list",
        json!({ "store": store_root.display().to_string() }),
        json!({ "workflow_graph_count": graph_count, "workflow_graphs": records }),
    )
}

pub(super) fn inspect_json(store_root: &Path, workflow_graph_id: &str) -> BridgeResult<String> {
    let record = WorkflowWorkspaceStore::new(store_root.to_owned())
        .inspect_graph(&id(workflow_graph_id)?)?;
    report_json(
        "casegraphen cg workflow inspect",
        "workspace_inspect",
        store_id_input(store_root, workflow_graph_id),
        serde_json::to_value(record)?,
    )
}

pub(super) fn history_json(store_root: &Path, workflow_graph_id: &str) -> BridgeResult<String> {
    let entries = WorkflowWorkspaceStore::new(store_root.to_owned())
        .history_entries(&id(workflow_graph_id)?)?;
    let entry_count = entries.len();
    report_json(
        "casegraphen cg workflow history",
        "workspace_history",
        store_id_input(store_root, workflow_graph_id),
        json!({
            "workflow_graph_id": workflow_graph_id,
            "history_entry_count": entry_count,
            "entries": entries
        }),
    )
}

pub(super) fn replay_json(store_root: &Path, workflow_graph_id: &str) -> BridgeResult<String> {
    let replay = WorkflowWorkspaceStore::new(store_root.to_owned())
        .replay_current_graph(&id(workflow_graph_id)?)?;
    report_json(
        "casegraphen cg workflow replay",
        "workspace_replay",
        store_id_input(store_root, workflow_graph_id),
        serde_json::to_value(replay)?,
    )
}

pub(super) fn validate_json(store_root: &Path, workflow_graph_id: &str) -> BridgeResult<String> {
    let validation = WorkflowWorkspaceStore::new(store_root.to_owned())
        .validate_graph(&id(workflow_graph_id)?)?;
    report_json(
        "casegraphen cg workflow validate",
        "workspace_validate",
        store_id_input(store_root, workflow_graph_id),
        serde_json::to_value(validation)?,
    )
}

pub(super) fn readiness_json(
    source: &BridgeWorkflowSource,
    projection: Option<&Path>,
) -> BridgeResult<String> {
    let graph = match source {
        BridgeWorkflowSource::File(input) => read_workflow_graph(input)?,
        BridgeWorkflowSource::Store {
            store,
            workflow_graph_id,
        } => {
            WorkflowWorkspaceStore::new(store.clone())
                .replay_current_graph(&id(workflow_graph_id)?)?
                .graph
        }
    };
    validate_optional_projection(projection)?;
    report_json_with_projection(
        "casegraphen cg workflow readiness",
        "readiness",
        readiness_input(source, projection),
        serde_json::to_value(evaluate_readiness(&graph)?)?,
        workflow_report::focused_projection(&graph, "readiness"),
    )
}

pub(super) fn completion_review_json(
    action: CompletionReviewAction,
    store_root: &Path,
    workflow_graph_id: &str,
    request: &WorkflowCompletionReviewRequest,
) -> BridgeResult<String> {
    let result = WorkflowWorkspaceStore::new(store_root.to_owned()).review_completion_candidate(
        &id(workflow_graph_id)?,
        action,
        request.clone(),
    )?;
    let operation = match action {
        CompletionReviewAction::Accept => "completion_accept",
        CompletionReviewAction::Reject => "completion_reject",
        CompletionReviewAction::Reopen => "completion_reopen",
    };
    report_json(
        &format!(
            "casegraphen cg workflow completion {}",
            completion_action_command(action)
        ),
        operation,
        store_candidate_input(store_root, workflow_graph_id, &request.candidate_id),
        serde_json::to_value(result)?,
    )
}

pub(super) fn completion_patch_json(
    store_root: &Path,
    workflow_graph_id: &str,
    request: &WorkflowCompletionPatchRequest,
) -> BridgeResult<String> {
    let result = WorkflowWorkspaceStore::new(store_root.to_owned())
        .convert_completion_to_patch(&id(workflow_graph_id)?, request.clone())?;
    report_json(
        "casegraphen cg workflow completion patch",
        "completion_patch",
        store_candidate_input(store_root, workflow_graph_id, &request.candidate_id),
        serde_json::to_value(result)?,
    )
}

pub(super) fn patch_check_json(
    store_root: &Path,
    workflow_graph_id: &str,
    transition_id: &str,
) -> BridgeResult<String> {
    let result = WorkflowWorkspaceStore::new(store_root.to_owned())
        .check_patch_transition(&id(workflow_graph_id)?, &id(transition_id)?)?;
    report_json(
        "casegraphen cg workflow patch check",
        "patch_check",
        store_transition_input(store_root, workflow_graph_id, transition_id),
        serde_json::to_value(result)?,
    )
}

pub(super) fn patch_review_json(
    action: WorkflowPatchReviewAction,
    store_root: &Path,
    workflow_graph_id: &str,
    request: &WorkflowPatchReviewRequest,
) -> BridgeResult<String> {
    let result = WorkflowWorkspaceStore::new(store_root.to_owned()).review_patch_transition(
        &id(workflow_graph_id)?,
        action,
        request.clone(),
    )?;
    let operation = match action {
        WorkflowPatchReviewAction::Apply => "patch_apply",
        WorkflowPatchReviewAction::Reject => "patch_reject",
    };
    report_json(
        &format!(
            "casegraphen cg workflow patch {}",
            patch_action_command(action)
        ),
        operation,
        store_transition_input(store_root, workflow_graph_id, &request.transition_id),
        serde_json::to_value(result)?,
    )
}

fn readiness_input(source: &BridgeWorkflowSource, projection: Option<&Path>) -> Value {
    let mut value = match source {
        BridgeWorkflowSource::File(input) => json!({
            "source": "file",
            "path": input.display().to_string()
        }),
        BridgeWorkflowSource::Store {
            store,
            workflow_graph_id,
        } => json!({
            "source": "workspace_store",
            "store": store.display().to_string(),
            "workflow_graph_id": workflow_graph_id
        }),
    };
    if let (Value::Object(object), Some(path)) = (&mut value, projection) {
        object.insert("projection".to_owned(), json!(path.display().to_string()));
    }
    value
}

fn validate_optional_projection(projection: Option<&Path>) -> BridgeResult<()> {
    if let Some(path) = projection {
        let _definition = read_projection(path)?;
    }
    Ok(())
}

fn report_json(
    command: &str,
    operation: &str,
    input: Value,
    result: Value,
) -> BridgeResult<String> {
    report_json_with_projection(command, operation, input, result, bridge_projection())
}

fn report_json_with_projection(
    command: &str,
    operation: &str,
    input: Value,
    result: Value,
    projection: Value,
) -> BridgeResult<String> {
    serde_json::to_string(&workflow_report::workflow_operation_report(
        command, operation, input, result, projection,
    ))
    .map_err(WorkflowBridgeError::from)
}

fn bridge_projection() -> Value {
    json!({
        "human_review": {
            "summary": "Repo-owned CaseGraphen bridge command completed."
        },
        "ai_view": {
            "bridge": "casegraphen cg workflow",
            "installed_cg_boundary": "native cg workspace commands remain the durable task backbone"
        },
        "audit_trace": {
            "source_ids": [],
            "information_loss": [
                "Bridge workspace commands use WorkflowWorkspaceStore JSON history and do not append native .casegraphen events."
            ]
        }
    })
}

fn store_id_input(store_root: &Path, workflow_graph_id: &str) -> Value {
    json!({
        "store": store_root.display().to_string(),
        "workflow_graph_id": workflow_graph_id
    })
}

fn store_candidate_input(store_root: &Path, workflow_graph_id: &str, candidate_id: &Id) -> Value {
    json!({
        "store": store_root.display().to_string(),
        "workflow_graph_id": workflow_graph_id,
        "candidate_id": candidate_id
    })
}

fn store_transition_input(
    store_root: &Path,
    workflow_graph_id: &str,
    transition_id: impl fmt::Display,
) -> Value {
    json!({
        "store": store_root.display().to_string(),
        "workflow_graph_id": workflow_graph_id,
        "transition_id": transition_id.to_string()
    })
}

fn completion_action_command(action: CompletionReviewAction) -> &'static str {
    match action {
        CompletionReviewAction::Accept => "accept",
        CompletionReviewAction::Reject => "reject",
        CompletionReviewAction::Reopen => "reopen",
    }
}

fn patch_action_command(action: WorkflowPatchReviewAction) -> &'static str {
    match action {
        WorkflowPatchReviewAction::Apply => "apply",
        WorkflowPatchReviewAction::Reject => "reject",
    }
}

fn id(value: &str) -> Result<Id, higher_graphen_core::CoreError> {
    Id::new(value.to_owned())
}
