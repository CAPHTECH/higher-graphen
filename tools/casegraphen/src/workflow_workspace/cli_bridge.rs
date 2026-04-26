use super::{
    WorkflowCompletionPatchRequest, WorkflowCompletionReviewRequest, WorkflowPatchReviewAction,
    WorkflowPatchReviewRequest,
};
use crate::{
    store::StoreError, workflow_eval::WorkflowValidationError,
    workflow_model::CompletionReviewAction,
};
use higher_graphen_core::Id;
use reports::{
    completion_patch_json, completion_review_json, history_json, import_json, inspect_json,
    list_json, patch_check_json, patch_review_json, readiness_json, replay_json, validate_json,
};
use std::{ffi::OsString, fmt, path::PathBuf};

mod reports;

const BRIDGE_USAGE: &str = "usage:
  casegraphen cg workflow import --store <dir> --input <workflow.graph.json> --revision-id <id> --format json [--output <path>]
  casegraphen cg workflow list --store <dir> --format json [--output <path>]
  casegraphen cg workflow inspect --store <dir> --workflow-graph-id <id> --format json [--output <path>]
  casegraphen cg workflow history --store <dir> --workflow-graph-id <id> --format json [--output <path>]
  casegraphen cg workflow replay --store <dir> --workflow-graph-id <id> --format json [--output <path>]
  casegraphen cg workflow validate --store <dir> --workflow-graph-id <id> --format json [--output <path>]
  casegraphen cg workflow readiness (--input <workflow.graph.json> | --store <dir> --workflow-graph-id <id>) --format json [--projection <projection.json>] [--output <path>]
  casegraphen cg workflow completion accept|reject|reopen --store <dir> --workflow-graph-id <id> --candidate-id <id> --reviewer-id <id> --reason <text> --revision-id <id> --format json [--reviewed-at <text>] [--evidence-id <id> ...] [--decision-id <id> ...] [--output <path>]
  casegraphen cg workflow completion patch --store <dir> --workflow-graph-id <id> --candidate-id <id> --reviewer-id <id> --reason <text> --revision-id <id> --format json [--transition-id <id>] [--reviewed-at <text>] [--output <path>]
  casegraphen cg workflow patch check --store <dir> --workflow-graph-id <id> --transition-id <id> --format json [--output <path>]
  casegraphen cg workflow patch apply|reject --store <dir> --workflow-graph-id <id> --transition-id <id> --reviewer-id <id> --reason <text> --revision-id <id> --format json [--reviewed-at <text>] [--output <path>]";

#[derive(Debug, Eq, PartialEq)]
pub enum CgWorkflowBridgeCommand {
    Import {
        store: PathBuf,
        input: PathBuf,
        revision_id: String,
        output: Option<PathBuf>,
    },
    List {
        store: PathBuf,
        output: Option<PathBuf>,
    },
    Inspect {
        store: PathBuf,
        workflow_graph_id: String,
        output: Option<PathBuf>,
    },
    History {
        store: PathBuf,
        workflow_graph_id: String,
        output: Option<PathBuf>,
    },
    Replay {
        store: PathBuf,
        workflow_graph_id: String,
        output: Option<PathBuf>,
    },
    Validate {
        store: PathBuf,
        workflow_graph_id: String,
        output: Option<PathBuf>,
    },
    Readiness {
        source: BridgeWorkflowSource,
        projection: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    CompletionReview {
        action: CompletionReviewAction,
        store: PathBuf,
        workflow_graph_id: String,
        request: WorkflowCompletionReviewRequest,
        output: Option<PathBuf>,
    },
    CompletionPatch {
        store: PathBuf,
        workflow_graph_id: String,
        request: WorkflowCompletionPatchRequest,
        output: Option<PathBuf>,
    },
    PatchCheck {
        store: PathBuf,
        workflow_graph_id: String,
        transition_id: String,
        output: Option<PathBuf>,
    },
    PatchReview {
        action: WorkflowPatchReviewAction,
        store: PathBuf,
        workflow_graph_id: String,
        request: WorkflowPatchReviewRequest,
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum BridgeWorkflowSource {
    File(PathBuf),
    Store {
        store: PathBuf,
        workflow_graph_id: String,
    },
}

impl CgWorkflowBridgeCommand {
    pub fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let mut args = args;
        match required_segment(&mut args, "bridge group")?.to_str() {
            Some("workflow") => Self::parse_workflow(args),
            Some(_) | None => Err(usage("unsupported cg bridge command segment")),
        }
    }

    pub fn output(&self) -> Option<&PathBuf> {
        match self {
            Self::Import { output, .. }
            | Self::List { output, .. }
            | Self::Inspect { output, .. }
            | Self::History { output, .. }
            | Self::Replay { output, .. }
            | Self::Validate { output, .. }
            | Self::Readiness { output, .. }
            | Self::CompletionReview { output, .. }
            | Self::CompletionPatch { output, .. }
            | Self::PatchCheck { output, .. }
            | Self::PatchReview { output, .. } => output.as_ref(),
        }
    }

    pub fn run_json(&self) -> Result<String, WorkflowBridgeError> {
        match self {
            Self::Import {
                store,
                input,
                revision_id,
                ..
            } => import_json(store, input, revision_id),
            Self::List { store, .. } => list_json(store),
            Self::Inspect {
                store,
                workflow_graph_id,
                ..
            } => inspect_json(store, workflow_graph_id),
            Self::History {
                store,
                workflow_graph_id,
                ..
            } => history_json(store, workflow_graph_id),
            Self::Replay {
                store,
                workflow_graph_id,
                ..
            } => replay_json(store, workflow_graph_id),
            Self::Validate {
                store,
                workflow_graph_id,
                ..
            } => validate_json(store, workflow_graph_id),
            Self::Readiness {
                source, projection, ..
            } => readiness_json(source, projection.as_deref()),
            Self::CompletionReview {
                action,
                store,
                workflow_graph_id,
                request,
                ..
            } => completion_review_json(*action, store, workflow_graph_id, request),
            Self::CompletionPatch {
                store,
                workflow_graph_id,
                request,
                ..
            } => completion_patch_json(store, workflow_graph_id, request),
            Self::PatchCheck {
                store,
                workflow_graph_id,
                transition_id,
                ..
            } => patch_check_json(store, workflow_graph_id, transition_id),
            Self::PatchReview {
                action,
                store,
                workflow_graph_id,
                request,
                ..
            } => patch_review_json(*action, store, workflow_graph_id, request),
        }
    }

    fn parse_workflow(args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let mut args = args;
        match required_segment(&mut args, "workflow bridge operation")?.to_str() {
            Some("import") => Self::parse_import(args),
            Some("list") => Self::parse_list(args),
            Some("inspect") => {
                Self::parse_store_id(args, |store, workflow_graph_id, output| Self::Inspect {
                    store,
                    workflow_graph_id,
                    output,
                })
            }
            Some("history") => {
                Self::parse_store_id(args, |store, workflow_graph_id, output| Self::History {
                    store,
                    workflow_graph_id,
                    output,
                })
            }
            Some("replay") => {
                Self::parse_store_id(args, |store, workflow_graph_id, output| Self::Replay {
                    store,
                    workflow_graph_id,
                    output,
                })
            }
            Some("validate") => {
                Self::parse_store_id(args, |store, workflow_graph_id, output| Self::Validate {
                    store,
                    workflow_graph_id,
                    output,
                })
            }
            Some("readiness") => Self::parse_readiness(args),
            Some("completion") => Self::parse_completion(args),
            Some("patch") => Self::parse_patch(args),
            Some(_) | None => Err(usage("unsupported cg workflow bridge operation")),
        }
    }

    fn parse_import(args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let options = BridgeOptions::parse(args)?;
        Ok(Self::Import {
            store: options.require_store()?,
            input: options.require_input()?,
            revision_id: options.require_revision_id()?,
            output: options.output,
        })
    }

    fn parse_list(args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let options = BridgeOptions::parse(args)?;
        Ok(Self::List {
            store: options.require_store()?,
            output: options.output,
        })
    }

    fn parse_store_id(
        args: impl Iterator<Item = OsString>,
        constructor: impl FnOnce(PathBuf, String, Option<PathBuf>) -> Self,
    ) -> Result<Self, String> {
        let options = BridgeOptions::parse(args)?;
        Ok(constructor(
            options.require_store()?,
            options.require_workflow_graph_id()?,
            options.output,
        ))
    }

    fn parse_readiness(args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let options = BridgeOptions::parse(args)?;
        Ok(Self::Readiness {
            source: options.require_source()?,
            projection: options.projection,
            output: options.output,
        })
    }

    fn parse_completion(args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let mut args = args;
        match required_segment(&mut args, "completion operation")?.to_str() {
            Some("accept") => Self::parse_completion_review(args, CompletionReviewAction::Accept),
            Some("reject") => Self::parse_completion_review(args, CompletionReviewAction::Reject),
            Some("reopen") => Self::parse_completion_review(args, CompletionReviewAction::Reopen),
            Some("patch") => Self::parse_completion_patch(args),
            Some(_) | None => Err(usage("unsupported cg workflow completion operation")),
        }
    }

    fn parse_completion_review(
        args: impl Iterator<Item = OsString>,
        action: CompletionReviewAction,
    ) -> Result<Self, String> {
        let options = BridgeOptions::parse(args)?;
        Ok(Self::CompletionReview {
            action,
            store: options.require_store()?,
            workflow_graph_id: options.require_workflow_graph_id()?,
            request: WorkflowCompletionReviewRequest {
                candidate_id: id_from_string(options.require_candidate_id()?)?,
                reviewer_id: id_from_string(options.require_reviewer_id()?)?,
                reason: options.require_reason()?,
                revision_id: id_from_string(options.require_revision_id()?)?,
                reviewed_at: options.reviewed_at,
                evidence_ids: ids_from_strings(options.evidence_ids)?,
                decision_ids: ids_from_strings(options.decision_ids)?,
            },
            output: options.output,
        })
    }

    fn parse_completion_patch(args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let options = BridgeOptions::parse(args)?;
        Ok(Self::CompletionPatch {
            store: options.require_store()?,
            workflow_graph_id: options.require_workflow_graph_id()?,
            request: WorkflowCompletionPatchRequest {
                candidate_id: id_from_string(options.require_candidate_id()?)?,
                reviewer_id: id_from_string(options.require_reviewer_id()?)?,
                reason: options.require_reason()?,
                revision_id: id_from_string(options.require_revision_id()?)?,
                reviewed_at: options.reviewed_at,
                transition_id: options.transition_id.map(id_from_string).transpose()?,
            },
            output: options.output,
        })
    }

    fn parse_patch(args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let mut args = args;
        match required_segment(&mut args, "patch operation")?.to_str() {
            Some("check") => {
                let options = BridgeOptions::parse(args)?;
                Ok(Self::PatchCheck {
                    store: options.require_store()?,
                    workflow_graph_id: options.require_workflow_graph_id()?,
                    transition_id: options.require_transition_id()?,
                    output: options.output,
                })
            }
            Some("apply") => Self::parse_patch_review(args, WorkflowPatchReviewAction::Apply),
            Some("reject") => Self::parse_patch_review(args, WorkflowPatchReviewAction::Reject),
            Some(_) | None => Err(usage("unsupported cg workflow patch operation")),
        }
    }

    fn parse_patch_review(
        args: impl Iterator<Item = OsString>,
        action: WorkflowPatchReviewAction,
    ) -> Result<Self, String> {
        let options = BridgeOptions::parse(args)?;
        Ok(Self::PatchReview {
            action,
            store: options.require_store()?,
            workflow_graph_id: options.require_workflow_graph_id()?,
            request: WorkflowPatchReviewRequest {
                transition_id: id_from_string(options.require_transition_id()?)?,
                reviewer_id: id_from_string(options.require_reviewer_id()?)?,
                reason: options.require_reason()?,
                revision_id: id_from_string(options.require_revision_id()?)?,
                reviewed_at: options.reviewed_at,
            },
            output: options.output,
        })
    }
}

#[derive(Default)]
struct BridgeOptions {
    input: Option<PathBuf>,
    projection: Option<PathBuf>,
    store: Option<PathBuf>,
    output: Option<PathBuf>,
    workflow_graph_id: Option<String>,
    revision_id: Option<String>,
    candidate_id: Option<String>,
    reviewer_id: Option<String>,
    reason: Option<String>,
    reviewed_at: Option<String>,
    transition_id: Option<String>,
    evidence_ids: Vec<String>,
    decision_ids: Vec<String>,
}

impl BridgeOptions {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let mut options = Self::default();
        let mut format_seen = false;
        let mut args = args;
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--format") => {
                    require_json_format(&mut args)?;
                    format_seen = true;
                }
                Some("--input") => options.input = Some(require_path(&mut args, "--input")?),
                Some("--projection") => {
                    options.projection = Some(require_path(&mut args, "--projection")?);
                }
                Some("--store") => options.store = Some(require_path(&mut args, "--store")?),
                Some("--output") => options.output = Some(require_path(&mut args, "--output")?),
                Some("--workflow-graph-id") => {
                    options.workflow_graph_id =
                        Some(require_string(&mut args, "--workflow-graph-id")?);
                }
                Some("--revision-id") => {
                    options.revision_id = Some(require_string(&mut args, "--revision-id")?);
                }
                Some("--candidate-id") => {
                    options.candidate_id = Some(require_string(&mut args, "--candidate-id")?);
                }
                Some("--reviewer-id") => {
                    options.reviewer_id = Some(require_string(&mut args, "--reviewer-id")?);
                }
                Some("--reason") => {
                    options.reason = Some(require_string(&mut args, "--reason")?);
                }
                Some("--reviewed-at") => {
                    options.reviewed_at = Some(require_string(&mut args, "--reviewed-at")?);
                }
                Some("--transition-id") => {
                    options.transition_id = Some(require_string(&mut args, "--transition-id")?);
                }
                Some("--evidence-id") => {
                    options
                        .evidence_ids
                        .push(require_string(&mut args, "--evidence-id")?);
                }
                Some("--decision-id") => {
                    options
                        .decision_ids
                        .push(require_string(&mut args, "--decision-id")?);
                }
                Some(_) | None => return Err(usage(format!("unsupported argument {arg:?}"))),
            }
        }
        require_format_seen(format_seen)?;
        Ok(options)
    }

    fn require_input(&self) -> Result<PathBuf, String> {
        self.input
            .clone()
            .ok_or_else(|| usage("--input <workflow.graph.json> is required"))
    }

    fn require_store(&self) -> Result<PathBuf, String> {
        self.store
            .clone()
            .ok_or_else(|| usage("--store <dir> is required"))
    }

    fn require_workflow_graph_id(&self) -> Result<String, String> {
        self.workflow_graph_id
            .clone()
            .ok_or_else(|| usage("--workflow-graph-id <id> is required"))
    }

    fn require_revision_id(&self) -> Result<String, String> {
        self.revision_id
            .clone()
            .ok_or_else(|| usage("--revision-id <id> is required"))
    }

    fn require_candidate_id(&self) -> Result<String, String> {
        self.candidate_id
            .clone()
            .ok_or_else(|| usage("--candidate-id <id> is required"))
    }

    fn require_reviewer_id(&self) -> Result<String, String> {
        self.reviewer_id
            .clone()
            .ok_or_else(|| usage("--reviewer-id <id> is required"))
    }

    fn require_reason(&self) -> Result<String, String> {
        self.reason
            .clone()
            .ok_or_else(|| usage("--reason <text> is required"))
    }

    fn require_transition_id(&self) -> Result<String, String> {
        self.transition_id
            .clone()
            .ok_or_else(|| usage("--transition-id <id> is required"))
    }

    fn require_source(&self) -> Result<BridgeWorkflowSource, String> {
        match (&self.input, &self.store, &self.workflow_graph_id) {
            (Some(input), None, None) => Ok(BridgeWorkflowSource::File(input.clone())),
            (None, Some(store), Some(workflow_graph_id)) => Ok(BridgeWorkflowSource::Store {
                store: store.clone(),
                workflow_graph_id: workflow_graph_id.clone(),
            }),
            (Some(_), Some(_), _) | (Some(_), _, Some(_)) => Err(usage(
                "use either --input or --store with --workflow-graph-id, not both",
            )),
            (None, Some(_), None) => Err(usage("--workflow-graph-id <id> is required")),
            (None, None, Some(_)) => Err(usage("--store <dir> is required")),
            (None, None, None) => Err(usage(
                "--input or --store with --workflow-graph-id is required",
            )),
        }
    }
}

fn id_from_string(value: String) -> Result<Id, String> {
    Id::new(value).map_err(|error| usage(error.to_string()))
}

fn ids_from_strings(values: Vec<String>) -> Result<Vec<Id>, String> {
    values.into_iter().map(id_from_string).collect()
}

fn required_segment(
    args: &mut impl Iterator<Item = OsString>,
    expected: &'static str,
) -> Result<OsString, String> {
    args.next()
        .ok_or_else(|| usage(format!("missing command segment {expected:?}")))
}

fn require_json_format(args: &mut impl Iterator<Item = OsString>) -> Result<(), String> {
    match args.next() {
        Some(arg) if arg == "json" => Ok(()),
        Some(arg) => Err(usage(format!(
            "unsupported format {arg:?}; only json is supported"
        ))),
        None => Err(usage("missing value for --format")),
    }
}

fn require_format_seen(format_seen: bool) -> Result<(), String> {
    if format_seen {
        Ok(())
    } else {
        Err(usage("--format json is required"))
    }
}

fn require_path(
    args: &mut impl Iterator<Item = OsString>,
    option: &'static str,
) -> Result<PathBuf, String> {
    match args.next() {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        Some(_) => Err(usage(format!("empty path for {option}"))),
        None => Err(usage(format!("missing value for {option}"))),
    }
}

fn require_string(
    args: &mut impl Iterator<Item = OsString>,
    option: &'static str,
) -> Result<String, String> {
    match args.next() {
        Some(value) if !value.is_empty() => value
            .into_string()
            .map_err(|value| usage(format!("non-utf8 value for {option}: {value:?}"))),
        Some(_) => Err(usage(format!("empty value for {option}"))),
        None => Err(usage(format!("missing value for {option}"))),
    }
}

fn usage(message: impl Into<String>) -> String {
    format!("{}\n{BRIDGE_USAGE}", message.into())
}

pub(super) type BridgeResult<T> = Result<T, WorkflowBridgeError>;

#[derive(Debug)]
pub enum WorkflowBridgeError {
    Core(higher_graphen_core::CoreError),
    Store(StoreError),
    Validation(WorkflowValidationError),
    Json(serde_json::Error),
}

impl From<higher_graphen_core::CoreError> for WorkflowBridgeError {
    fn from(error: higher_graphen_core::CoreError) -> Self {
        Self::Core(error)
    }
}

impl From<StoreError> for WorkflowBridgeError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<WorkflowValidationError> for WorkflowBridgeError {
    fn from(error: WorkflowValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<serde_json::Error> for WorkflowBridgeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl fmt::Display for WorkflowBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Validation(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for WorkflowBridgeError {}
