use crate::native_eval::evaluate_native_case;
use crate::native_model::ProjectionAudience;
use crate::native_store::{NativeCaseStore, NativeStoreError};
use crate::topology::TopologyReportOptions;
use higher_graphen_core::Id;
use serde_json::{json, Value};
use std::{
    ffi::OsString,
    fmt,
    path::{Path, PathBuf},
};

mod ops;
#[path = "native_cli_options.rs"]
mod options;
#[path = "native_cli_path.rs"]
mod path_helpers;
#[path = "native_cli_reporting.rs"]
mod reporting;
use ops::{
    case_close_check, case_import, case_new, case_reason, case_topology, case_topology_diff,
    lift_structured_source, morphism_apply, morphism_check, morphism_propose, morphism_reject,
    projection_apply, NativeCloseGateOptions,
};
use options::{required_segment, NativeOptions};
use reporting::report;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum NativeCliCommand {
    CaseNew {
        store: PathBuf,
        case_space_id: Id,
        space_id: Id,
        title: String,
        revision_id: Id,
        output: Option<PathBuf>,
    },
    CaseImport {
        store: PathBuf,
        input: PathBuf,
        revision_id: Id,
        output: Option<PathBuf>,
    },
    LiftStructuredSource {
        store: PathBuf,
        input: PathBuf,
        revision_id: Id,
        adapter: String,
        output: Option<PathBuf>,
    },
    CaseList {
        store: PathBuf,
        output: Option<PathBuf>,
    },
    CaseInspect {
        store: PathBuf,
        case_space_id: Id,
        output: Option<PathBuf>,
    },
    CaseHistory {
        store: PathBuf,
        case_space_id: Id,
        output: Option<PathBuf>,
    },
    CaseReplay {
        store: PathBuf,
        case_space_id: Id,
        output: Option<PathBuf>,
    },
    CaseValidate {
        store: PathBuf,
        case_space_id: Id,
        output: Option<PathBuf>,
    },
    InvariantCheck {
        store: PathBuf,
        case_space_id: Id,
        output: Option<PathBuf>,
    },
    CaseReason {
        store: PathBuf,
        case_space_id: Id,
        section: NativeReasonSection,
        output: Option<PathBuf>,
    },
    ProjectionApply {
        store: PathBuf,
        case_space_id: Id,
        projection: PathBuf,
        output: Option<PathBuf>,
    },
    CaseCloseCheck {
        store: PathBuf,
        case_space_id: Id,
        base_revision_id: Id,
        validation_evidence_ids: Vec<Id>,
        gate_options: NativeCloseGateOptions,
        output: Option<PathBuf>,
    },
    CaseTopology {
        store: PathBuf,
        case_space_id: Id,
        topology_options: TopologyReportOptions,
        output: Option<PathBuf>,
    },
    CaseTopologyDiff {
        left_store: PathBuf,
        left_case_space_id: Id,
        right_store: PathBuf,
        right_case_space_id: Id,
        topology_options: TopologyReportOptions,
        output: Option<PathBuf>,
    },
    EquivalenceCheck {
        left_store: PathBuf,
        left_case_space_id: Id,
        right_store: PathBuf,
        right_case_space_id: Id,
        topology_options: TopologyReportOptions,
        output: Option<PathBuf>,
    },
    MorphismPropose {
        store: PathBuf,
        case_space_id: Id,
        input: PathBuf,
        output: Option<PathBuf>,
    },
    MorphismCheck {
        store: PathBuf,
        case_space_id: Id,
        morphism_id: Id,
        output: Option<PathBuf>,
    },
    MorphismApply {
        store: PathBuf,
        case_space_id: Id,
        morphism_id: Id,
        base_revision_id: Id,
        reviewer_id: Option<Id>,
        reason: Option<String>,
        output: Option<PathBuf>,
    },
    MorphismReject {
        store: PathBuf,
        case_space_id: Id,
        morphism_id: Id,
        reviewer_id: Id,
        reason: String,
        revision_id: Id,
        output: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeReasonSection {
    Reason,
    Frontier,
    Obstructions,
    Completions,
    Evidence,
    Project,
}

impl NativeCliCommand {
    pub fn parse(
        namespace: &str,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let mut args = args.into_iter();
        match namespace {
            "case" => Self::parse_case(required_segment(&mut args, "case operation")?, args),
            "space" => Self::parse_space(required_segment(&mut args, "space operation")?, args),
            "lift" => Self::parse_lift(required_segment(&mut args, "lift adapter")?, args),
            "obstruction" => {
                Self::parse_obstruction(required_segment(&mut args, "obstruction operation")?, args)
            }
            "completion" => {
                Self::parse_completion(required_segment(&mut args, "completion operation")?, args)
            }
            "projection" => {
                Self::parse_projection(required_segment(&mut args, "projection operation")?, args)
            }
            "equivalence" => {
                Self::parse_equivalence(required_segment(&mut args, "equivalence operation")?, args)
            }
            "invariant" => {
                Self::parse_invariant(required_segment(&mut args, "invariant operation")?, args)
            }
            "morphism" => {
                Self::parse_morphism(required_segment(&mut args, "morphism operation")?, args)
            }
            _ => Err(NativeCliError::usage("unsupported native namespace")),
        }
    }

    pub fn output(&self) -> Option<&PathBuf> {
        match self {
            Self::CaseNew { output, .. }
            | Self::CaseImport { output, .. }
            | Self::LiftStructuredSource { output, .. }
            | Self::CaseList { output, .. }
            | Self::CaseInspect { output, .. }
            | Self::CaseHistory { output, .. }
            | Self::CaseReplay { output, .. }
            | Self::CaseValidate { output, .. }
            | Self::InvariantCheck { output, .. }
            | Self::CaseReason { output, .. }
            | Self::ProjectionApply { output, .. }
            | Self::CaseCloseCheck { output, .. }
            | Self::CaseTopology { output, .. }
            | Self::CaseTopologyDiff { output, .. }
            | Self::EquivalenceCheck { output, .. }
            | Self::MorphismPropose { output, .. }
            | Self::MorphismCheck { output, .. }
            | Self::MorphismApply { output, .. }
            | Self::MorphismReject { output, .. } => output.as_ref(),
        }
    }

    pub fn run_json(&self) -> Result<String, NativeCliError> {
        serde_json::to_string(&self.run_value()?).map_err(NativeCliError::from)
    }

    fn run_value(&self) -> Result<Value, NativeCliError> {
        match self {
            Self::CaseNew { .. }
            | Self::CaseImport { .. }
            | Self::LiftStructuredSource { .. }
            | Self::CaseList { .. }
            | Self::CaseInspect { .. }
            | Self::CaseHistory { .. }
            | Self::CaseReplay { .. }
            | Self::CaseValidate { .. }
            | Self::InvariantCheck { .. }
            | Self::CaseReason { .. }
            | Self::ProjectionApply { .. }
            | Self::CaseCloseCheck { .. }
            | Self::CaseTopology { .. }
            | Self::CaseTopologyDiff { .. }
            | Self::EquivalenceCheck { .. } => self.run_case_value(),
            Self::MorphismPropose { .. }
            | Self::MorphismCheck { .. }
            | Self::MorphismApply { .. }
            | Self::MorphismReject { .. } => self.run_morphism_value(),
        }
    }

    fn run_case_value(&self) -> Result<Value, NativeCliError> {
        Ok(match self {
            Self::CaseNew {
                store,
                case_space_id,
                space_id,
                title,
                revision_id,
                ..
            } => case_new(store, case_space_id, space_id, title, revision_id)?,
            Self::CaseImport {
                store,
                input,
                revision_id,
                ..
            } => case_import(store, input, revision_id)?,
            Self::LiftStructuredSource {
                store,
                input,
                revision_id,
                adapter,
                ..
            } => lift_structured_source(store, input, revision_id, adapter)?,
            Self::CaseList { store, .. } => case_list(store)?,
            Self::CaseInspect {
                store,
                case_space_id,
                ..
            } => case_inspect(store, case_space_id)?,
            Self::CaseHistory {
                store,
                case_space_id,
                ..
            } => case_history(store, case_space_id)?,
            Self::CaseReplay {
                store,
                case_space_id,
                ..
            } => case_replay(store, case_space_id)?,
            Self::CaseValidate {
                store,
                case_space_id,
                ..
            } => case_validate(store, case_space_id)?,
            Self::InvariantCheck {
                store,
                case_space_id,
                ..
            } => invariant_check(store, case_space_id)?,
            Self::CaseReason {
                store,
                case_space_id,
                section,
                ..
            } => case_reason(store, case_space_id, *section)?,
            Self::ProjectionApply {
                store,
                case_space_id,
                projection,
                ..
            } => projection_apply(store, case_space_id, projection)?,
            Self::CaseCloseCheck {
                store,
                case_space_id,
                base_revision_id,
                validation_evidence_ids,
                gate_options,
                ..
            } => case_close_check(
                store,
                case_space_id,
                base_revision_id,
                validation_evidence_ids,
                gate_options.clone(),
            )?,
            Self::CaseTopology {
                store,
                case_space_id,
                topology_options,
                ..
            } => case_topology(store, case_space_id, *topology_options)?,
            Self::CaseTopologyDiff {
                left_store,
                left_case_space_id,
                right_store,
                right_case_space_id,
                topology_options,
                ..
            } => case_topology_diff(
                left_store,
                left_case_space_id,
                right_store,
                right_case_space_id,
                *topology_options,
            )?,
            Self::EquivalenceCheck {
                left_store,
                left_case_space_id,
                right_store,
                right_case_space_id,
                topology_options,
                ..
            } => equivalence_check(
                left_store,
                left_case_space_id,
                right_store,
                right_case_space_id,
                *topology_options,
            )?,
            _ => unreachable!("run_case_value called for morphism command"),
        })
    }

    fn run_morphism_value(&self) -> Result<Value, NativeCliError> {
        Ok(match self {
            Self::MorphismPropose {
                store,
                case_space_id,
                input,
                ..
            } => morphism_propose(store, case_space_id, input)?,
            Self::MorphismCheck {
                store,
                case_space_id,
                morphism_id,
                ..
            } => morphism_check(store, case_space_id, morphism_id)?,
            Self::MorphismApply {
                store,
                case_space_id,
                morphism_id,
                base_revision_id,
                reviewer_id,
                reason,
                ..
            } => morphism_apply(
                store,
                case_space_id,
                morphism_id,
                base_revision_id,
                reviewer_id.as_ref(),
                reason.as_deref(),
            )?,
            Self::MorphismReject {
                store,
                case_space_id,
                morphism_id,
                reviewer_id,
                reason,
                revision_id,
                ..
            } => morphism_reject(
                store,
                case_space_id,
                morphism_id,
                reviewer_id,
                reason,
                revision_id,
            )?,
            _ => unreachable!("run_morphism_value called for case command"),
        })
    }

    fn parse_case(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let operation = operation
            .to_str()
            .ok_or_else(|| NativeCliError::usage("case operation must be UTF-8"))?;
        let mut args = args.into_iter().collect::<Vec<_>>();
        let history_topology = is_history_topology(operation, &args);
        if history_topology {
            args.remove(0);
        }
        let history_topology_diff = history_topology
            && args
                .first()
                .and_then(|argument| argument.to_str())
                .is_some_and(|argument| argument == "diff");
        if history_topology_diff {
            args.remove(0);
        }
        let options = NativeOptions::parse(args)?;
        match operation {
            "new" | "create" => Ok(Self::CaseNew {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                space_id: options.require_id("--space-id")?,
                title: options.require_string("--title")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "import" => Ok(Self::CaseImport {
                store: options.require_store()?,
                input: options.require_path("--input")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "list" => Ok(Self::CaseList {
                store: options.require_store()?,
                output: options.output,
            }),
            "inspect" => Ok(Self::CaseInspect {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "history" => Self::parse_history_case(options, history_topology, history_topology_diff),
            "replay" => Ok(Self::CaseReplay {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "validate" => Ok(Self::CaseValidate {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "reason" => Self::parse_reason(options, NativeReasonSection::Reason),
            "frontier" => Self::parse_reason(options, NativeReasonSection::Frontier),
            "obstructions" => Self::parse_reason(options, NativeReasonSection::Obstructions),
            "completions" => Self::parse_reason(options, NativeReasonSection::Completions),
            "evidence" => Self::parse_reason(options, NativeReasonSection::Evidence),
            "project" => Self::parse_reason(options, NativeReasonSection::Project),
            "close-check" => Self::parse_close_check(options),
            _ => Err(NativeCliError::usage("unsupported native case command")),
        }
    }

    fn parse_space(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let operation = operation
            .to_str()
            .ok_or_else(|| NativeCliError::usage("space operation must be UTF-8"))?;
        let mut args = args.into_iter().collect::<Vec<_>>();
        let topology = operation == "topology";
        let topology_diff = topology
            && args
                .first()
                .and_then(|argument| argument.to_str())
                .is_some_and(|argument| argument == "diff");
        if topology_diff {
            args.remove(0);
        }
        let options = NativeOptions::parse(args)?;
        match operation {
            "new" | "create" => Ok(Self::CaseNew {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                space_id: options.require_id("--space-id")?,
                title: options.require_string("--title")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "import" => Ok(Self::CaseImport {
                store: options.require_store()?,
                input: options.require_path("--input")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "list" => Ok(Self::CaseList {
                store: options.require_store()?,
                output: options.output,
            }),
            "inspect" => Ok(Self::CaseInspect {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "history" => Ok(Self::CaseHistory {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "replay" => Ok(Self::CaseReplay {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "validate" => Ok(Self::CaseValidate {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "reason" => Self::parse_reason(options, NativeReasonSection::Reason),
            "frontier" => Self::parse_reason(options, NativeReasonSection::Frontier),
            "topology" if topology_diff => Ok(Self::CaseTopologyDiff {
                left_store: options.require_path("--left-store")?,
                left_case_space_id: options.require_id("--left-case-space-id")?,
                right_store: options.require_path("--right-store")?,
                right_case_space_id: options.require_id("--right-case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            }),
            "topology" => Ok(Self::CaseTopology {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            }),
            _ => Err(NativeCliError::usage("unsupported native space command")),
        }
    }

    fn parse_lift(
        adapter: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let adapter = adapter
            .to_str()
            .ok_or_else(|| NativeCliError::usage("lift adapter must be UTF-8"))?;
        let options = NativeOptions::parse(args)?;
        match adapter {
            "native" => Ok(Self::CaseImport {
                store: options.require_store()?,
                input: options.require_path("--input")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "workflow" | "case-graph" => Ok(Self::LiftStructuredSource {
                store: options.require_store()?,
                input: options.require_path("--input")?,
                revision_id: options.require_id("--revision-id")?,
                adapter: adapter.to_owned(),
                output: options.output,
            }),
            _ => Err(NativeCliError::usage("unsupported lift adapter")),
        }
    }

    fn parse_obstruction(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        match operation.to_str() {
            Some("list") => Self::parse_reason(
                NativeOptions::parse(args)?,
                NativeReasonSection::Obstructions,
            ),
            Some(_) | None => Err(NativeCliError::usage("unsupported obstruction command")),
        }
    }

    fn parse_completion(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        match operation.to_str() {
            Some("candidates") => Self::parse_reason(
                NativeOptions::parse(args)?,
                NativeReasonSection::Completions,
            ),
            Some(_) | None => Err(NativeCliError::usage("unsupported completion command")),
        }
    }

    fn parse_projection(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let options = NativeOptions::parse(args)?;
        match operation.to_str() {
            Some("apply") => {
                let projection = options.require_path("--projection")?;
                Ok(Self::ProjectionApply {
                    store: options.require_store()?,
                    case_space_id: options.require_id("--case-space-id")?,
                    projection,
                    output: options.output,
                })
            }
            Some(_) | None => Err(NativeCliError::usage("unsupported projection command")),
        }
    }

    fn parse_equivalence(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let options = NativeOptions::parse(args)?;
        match operation.to_str() {
            Some("check") => Ok(Self::EquivalenceCheck {
                left_store: options.require_path("--left-store")?,
                left_case_space_id: options.require_id("--left-case-space-id")?,
                right_store: options.require_path("--right-store")?,
                right_case_space_id: options.require_id("--right-case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            }),
            Some(_) | None => Err(NativeCliError::usage("unsupported equivalence command")),
        }
    }

    fn parse_invariant(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let options = NativeOptions::parse(args)?;
        match operation.to_str() {
            Some("check") => Ok(Self::InvariantCheck {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            Some("close-check") => Self::parse_close_check(options),
            Some(_) | None => Err(NativeCliError::usage("unsupported invariant command")),
        }
    }

    fn parse_close_check(options: NativeOptions) -> Result<Self, NativeCliError> {
        Ok(Self::CaseCloseCheck {
            store: options.require_store()?,
            case_space_id: options.require_id("--case-space-id")?,
            base_revision_id: options
                .base_revision_id
                .clone()
                .or(options.revision_id.clone())
                .ok_or_else(|| NativeCliError::usage("--base-revision-id <id> is required"))?,
            validation_evidence_ids: options.validation_evidence_ids,
            gate_options: NativeCloseGateOptions {
                close_policy_id: options.close_policy_id,
                actor_id: options.actor_id,
                capability_ids: options.capability_ids,
                operation_scope_id: options.operation_scope_id,
                audience: options.audience,
                source_boundary_id: options.source_boundary_id,
            },
            output: options.output,
        })
    }

    fn parse_history_case(
        options: NativeOptions,
        history_topology: bool,
        history_topology_diff: bool,
    ) -> Result<Self, NativeCliError> {
        if history_topology_diff {
            return Ok(Self::CaseTopologyDiff {
                left_store: options.require_path("--left-store")?,
                left_case_space_id: options.require_id("--left-case-space-id")?,
                right_store: options.require_path("--right-store")?,
                right_case_space_id: options.require_id("--right-case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            });
        }
        if history_topology {
            return Ok(Self::CaseTopology {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            });
        }
        Ok(Self::CaseHistory {
            store: options.require_store()?,
            case_space_id: options.require_id("--case-space-id")?,
            output: options.output,
        })
    }

    fn parse_reason(
        options: NativeOptions,
        section: NativeReasonSection,
    ) -> Result<Self, NativeCliError> {
        Ok(Self::CaseReason {
            store: options.require_store()?,
            case_space_id: options.require_id("--case-space-id")?,
            section,
            output: options.output,
        })
    }

    fn parse_morphism(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let operation = operation
            .to_str()
            .ok_or_else(|| NativeCliError::usage("morphism operation must be UTF-8"))?;
        let options = NativeOptions::parse(args)?;
        match operation {
            "propose" => Ok(Self::MorphismPropose {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                input: options.require_path("--input")?,
                output: options.output,
            }),
            "check" => Ok(Self::MorphismCheck {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                morphism_id: options.require_id("--morphism-id")?,
                output: options.output,
            }),
            "apply" => Ok(Self::MorphismApply {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                morphism_id: options.require_id("--morphism-id")?,
                base_revision_id: options
                    .base_revision_id
                    .clone()
                    .or(options.revision_id.clone())
                    .ok_or_else(|| NativeCliError::usage("--base-revision-id <id> is required"))?,
                reviewer_id: Some(options.require_id("--reviewer-id")?),
                reason: Some(options.require_string("--reason")?),
                output: options.output,
            }),
            "reject" => Ok(Self::MorphismReject {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                morphism_id: options.require_id("--morphism-id")?,
                reviewer_id: options.require_id("--reviewer-id")?,
                reason: options.require_string("--reason")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            _ => Err(NativeCliError::usage("unsupported native morphism command")),
        }
    }
}

pub(super) fn parse_projection_audience(value: &str) -> Result<ProjectionAudience, NativeCliError> {
    match value {
        "human_review" => Ok(ProjectionAudience::HumanReview),
        "ai_agent" => Ok(ProjectionAudience::AiAgent),
        "audit" => Ok(ProjectionAudience::Audit),
        "system" => Ok(ProjectionAudience::System),
        "migration" => Ok(ProjectionAudience::Migration),
        _ => Err(NativeCliError::usage(format!(
            "unsupported projection audience {value:?}"
        ))),
    }
}

fn case_list(store: &Path) -> Result<Value, NativeCliError> {
    let records = NativeCaseStore::new(store.to_path_buf()).list_case_spaces()?;
    Ok(report(
        "casegraphen space list",
        json!({ "case_spaces": records }),
    ))
}

fn case_inspect(store: &Path, case_space_id: &Id) -> Result<Value, NativeCliError> {
    let record = NativeCaseStore::new(store.to_path_buf()).inspect_case_space(case_space_id)?;
    Ok(report(
        "casegraphen space inspect",
        json!({ "record": record }),
    ))
}

fn case_history(store: &Path, case_space_id: &Id) -> Result<Value, NativeCliError> {
    let entries = NativeCaseStore::new(store.to_path_buf()).history_entries(case_space_id)?;
    Ok(report(
        "casegraphen space history",
        json!({ "entries": entries }),
    ))
}

fn case_replay(store: &Path, case_space_id: &Id) -> Result<Value, NativeCliError> {
    let replay =
        NativeCaseStore::new(store.to_path_buf()).replay_current_case_space(case_space_id)?;
    Ok(report(
        "casegraphen space replay",
        json!({ "replay": replay }),
    ))
}

fn case_validate(store: &Path, case_space_id: &Id) -> Result<Value, NativeCliError> {
    let validation =
        NativeCaseStore::new(store.to_path_buf()).validate_case_space(case_space_id)?;
    Ok(report(
        "casegraphen space validate",
        json!({ "validation": validation }),
    ))
}

fn invariant_check(store: &Path, case_space_id: &Id) -> Result<Value, NativeCliError> {
    let store = NativeCaseStore::new(store.to_path_buf());
    let validation = store.validate_case_space(case_space_id)?;
    let replay = store.replay_current_case_space(case_space_id)?;
    let evaluation = evaluate_native_case(&replay.case_space)?;
    let evidence_findings = evaluation.evidence_findings.clone();
    let projection_loss = evaluation.projection_loss.clone();
    let obstructions = evaluation.obstructions.clone();
    let completion_candidates = evaluation.completion_candidates.clone();
    let review_gaps = evaluation.review_gaps.clone();
    Ok(report(
        "casegraphen invariant check",
        json!({
            "validation": validation,
            "evaluation": evaluation,
            "evidence_findings": evidence_findings,
            "projection_loss": projection_loss,
            "obstructions": obstructions,
            "completion_candidates": completion_candidates,
            "review_gaps": review_gaps,
        }),
    ))
}

fn equivalence_check(
    left_store: &Path,
    left_case_space_id: &Id,
    right_store: &Path,
    right_case_space_id: &Id,
    topology_options: TopologyReportOptions,
) -> Result<Value, NativeCliError> {
    let left_replay = NativeCaseStore::new(left_store.to_path_buf())
        .replay_current_case_space(left_case_space_id)?;
    let right_replay = NativeCaseStore::new(right_store.to_path_buf())
        .replay_current_case_space(right_case_space_id)?;
    let left_topology = crate::topology::native_case_topology_with_history(
        &left_replay.case_space,
        &left_replay.history,
        topology_options,
    )?;
    let right_topology = crate::topology::native_case_topology_with_history(
        &right_replay.case_space,
        &right_replay.history,
        topology_options,
    )?;
    let topology_diff = crate::topology::topology_diff(&left_topology, &right_topology);
    Ok(report(
        "casegraphen equivalence check",
        json!({
            "left_case_space_id": left_case_space_id,
            "right_case_space_id": right_case_space_id,
            "topology_diff": topology_diff
        }),
    ))
}

fn is_history_topology(operation: &str, args: &[OsString]) -> bool {
    operation == "history"
        && args
            .first()
            .and_then(|argument| argument.to_str())
            .is_some_and(|argument| argument == "topology")
}

#[derive(Debug)]
pub enum NativeCliError {
    Usage(String),
    Invalid(String),
    Core(higher_graphen_core::CoreError),
    Store(NativeStoreError),
    Review(crate::native_review::NativeReviewError),
    Eval(crate::native_eval::NativeEvalError),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json(serde_json::Error),
}

impl NativeCliError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }
}

impl From<higher_graphen_core::CoreError> for NativeCliError {
    fn from(error: higher_graphen_core::CoreError) -> Self {
        Self::Core(error)
    }
}

impl From<NativeStoreError> for NativeCliError {
    fn from(error: NativeStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<crate::native_review::NativeReviewError> for NativeCliError {
    fn from(error: crate::native_review::NativeReviewError) -> Self {
        Self::Review(error)
    }
}

impl From<crate::native_eval::NativeEvalError> for NativeCliError {
    fn from(error: crate::native_eval::NativeEvalError) -> Self {
        Self::Eval(error)
    }
}

impl From<serde_json::Error> for NativeCliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl fmt::Display for NativeCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) | Self::Invalid(message) => formatter.write_str(message),
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Review(error) => write!(formatter, "{error}"),
            Self::Eval(error) => write!(formatter, "{error:?}"),
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for NativeCliError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_space_commands_as_canonical_native_surface() {
        let command = NativeCliCommand::parse(
            "space",
            args(&[
                "reason",
                "--store",
                "store",
                "--case-space-id",
                "case_space:demo",
                "--format",
                "json",
            ]),
        )
        .expect("space reason command");

        assert!(matches!(
            command,
            NativeCliCommand::CaseReason {
                section: NativeReasonSection::Reason,
                ..
            }
        ));

        let topology = NativeCliCommand::parse(
            "space",
            args(&[
                "topology",
                "diff",
                "--left-store",
                "left",
                "--left-case-space-id",
                "case_space:left",
                "--right-store",
                "right",
                "--right-case-space-id",
                "case_space:right",
                "--format",
                "json",
            ]),
        )
        .expect("space topology diff command");

        assert!(matches!(
            topology,
            NativeCliCommand::CaseTopologyDiff { .. }
        ));
    }

    #[test]
    fn parses_value_namespaces_to_existing_native_operations() {
        let obstruction = NativeCliCommand::parse(
            "obstruction",
            args(&[
                "list",
                "--store",
                "store",
                "--case-space-id",
                "case_space:demo",
                "--format",
                "json",
            ]),
        )
        .expect("obstruction list command");
        assert!(matches!(
            obstruction,
            NativeCliCommand::CaseReason {
                section: NativeReasonSection::Obstructions,
                ..
            }
        ));

        let completion = NativeCliCommand::parse(
            "completion",
            args(&[
                "candidates",
                "--store",
                "store",
                "--case-space-id",
                "case_space:demo",
                "--format",
                "json",
            ]),
        )
        .expect("completion candidates command");
        assert!(matches!(
            completion,
            NativeCliCommand::CaseReason {
                section: NativeReasonSection::Completions,
                ..
            }
        ));

        let projection = NativeCliCommand::parse(
            "projection",
            args(&[
                "apply",
                "--store",
                "store",
                "--case-space-id",
                "case_space:demo",
                "--projection",
                "projection.json",
                "--format",
                "json",
            ]),
        )
        .expect("projection apply command");
        assert!(matches!(
            projection,
            NativeCliCommand::ProjectionApply { .. }
        ));

        let invariant = NativeCliCommand::parse(
            "invariant",
            args(&[
                "check",
                "--store",
                "store",
                "--case-space-id",
                "case_space:demo",
                "--format",
                "json",
            ]),
        )
        .expect("invariant check command");
        assert!(matches!(invariant, NativeCliCommand::InvariantCheck { .. }));

        let equivalence = NativeCliCommand::parse(
            "equivalence",
            args(&[
                "check",
                "--left-store",
                "left",
                "--left-case-space-id",
                "case_space:left",
                "--right-store",
                "right",
                "--right-case-space-id",
                "case_space:right",
                "--format",
                "json",
            ]),
        )
        .expect("equivalence check command");
        assert!(matches!(
            equivalence,
            NativeCliCommand::EquivalenceCheck { .. }
        ));
    }

    #[test]
    fn parses_lift_adapters() {
        let workflow = NativeCliCommand::parse(
            "lift",
            args(&[
                "workflow",
                "--store",
                "store",
                "--input",
                "workflow.graph.json",
                "--revision-id",
                "revision:lifted",
                "--format",
                "json",
            ]),
        )
        .expect("workflow lift command");

        assert!(matches!(
            workflow,
            NativeCliCommand::LiftStructuredSource { adapter, .. } if adapter == "workflow"
        ));

        let native = NativeCliCommand::parse(
            "lift",
            args(&[
                "native",
                "--store",
                "store",
                "--input",
                "native.case.space.json",
                "--revision-id",
                "revision:lifted",
                "--format",
                "json",
            ]),
        )
        .expect("native lift command");
        assert!(matches!(native, NativeCliCommand::CaseImport { .. }));
    }
}
