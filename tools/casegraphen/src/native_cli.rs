use crate::native_store::{NativeCaseStore, NativeStoreError};
use higher_graphen_core::Id;
use serde_json::{json, Value};
use std::{
    ffi::OsString,
    fmt,
    path::{Path, PathBuf},
};

mod ops;
use ops::{
    case_close_check, case_import, case_new, case_reason, case_topology, morphism_apply,
    morphism_check, morphism_propose, morphism_reject, report,
};

#[derive(Debug, Eq, PartialEq)]
pub enum NativeCliCommand {
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
    CaseReason {
        store: PathBuf,
        case_space_id: Id,
        section: NativeReasonSection,
        output: Option<PathBuf>,
    },
    CaseCloseCheck {
        store: PathBuf,
        case_space_id: Id,
        base_revision_id: Id,
        validation_evidence_ids: Vec<Id>,
        output: Option<PathBuf>,
    },
    CaseTopology {
        store: PathBuf,
        case_space_id: Id,
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
            | Self::CaseList { output, .. }
            | Self::CaseInspect { output, .. }
            | Self::CaseHistory { output, .. }
            | Self::CaseReplay { output, .. }
            | Self::CaseValidate { output, .. }
            | Self::CaseReason { output, .. }
            | Self::CaseCloseCheck { output, .. }
            | Self::CaseTopology { output, .. }
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
            | Self::CaseList { .. }
            | Self::CaseInspect { .. }
            | Self::CaseHistory { .. }
            | Self::CaseReplay { .. }
            | Self::CaseValidate { .. }
            | Self::CaseReason { .. }
            | Self::CaseCloseCheck { .. }
            | Self::CaseTopology { .. } => self.run_case_value(),
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
            Self::CaseList { store, .. } => {
                let records = NativeCaseStore::new(store.clone()).list_case_spaces()?;
                report("casegraphen case list", json!({ "case_spaces": records }))
            }
            Self::CaseInspect {
                store,
                case_space_id,
                ..
            } => {
                let record =
                    NativeCaseStore::new(store.clone()).inspect_case_space(case_space_id)?;
                report("casegraphen case inspect", json!({ "record": record }))
            }
            Self::CaseHistory {
                store,
                case_space_id,
                ..
            } => {
                let entries = NativeCaseStore::new(store.clone()).history_entries(case_space_id)?;
                report("casegraphen case history", json!({ "entries": entries }))
            }
            Self::CaseReplay {
                store,
                case_space_id,
                ..
            } => {
                let replay =
                    NativeCaseStore::new(store.clone()).replay_current_case_space(case_space_id)?;
                report("casegraphen case replay", json!({ "replay": replay }))
            }
            Self::CaseValidate {
                store,
                case_space_id,
                ..
            } => {
                let validation =
                    NativeCaseStore::new(store.clone()).validate_case_space(case_space_id)?;
                report(
                    "casegraphen case validate",
                    json!({ "validation": validation }),
                )
            }
            Self::CaseReason {
                store,
                case_space_id,
                section,
                ..
            } => case_reason(store, case_space_id, *section)?,
            Self::CaseCloseCheck {
                store,
                case_space_id,
                base_revision_id,
                validation_evidence_ids,
                ..
            } => case_close_check(
                store,
                case_space_id,
                base_revision_id,
                validation_evidence_ids,
            )?,
            Self::CaseTopology {
                store,
                case_space_id,
                ..
            } => case_topology(store, case_space_id)?,
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
        let history_topology = operation == "history"
            && args
                .first()
                .and_then(|argument| argument.to_str())
                .is_some_and(|argument| argument == "topology");
        if history_topology {
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
            "history" if history_topology => Ok(Self::CaseTopology {
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
            "obstructions" => Self::parse_reason(options, NativeReasonSection::Obstructions),
            "completions" => Self::parse_reason(options, NativeReasonSection::Completions),
            "evidence" => Self::parse_reason(options, NativeReasonSection::Evidence),
            "project" => Self::parse_reason(options, NativeReasonSection::Project),
            "close-check" => Ok(Self::CaseCloseCheck {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                base_revision_id: options
                    .base_revision_id
                    .clone()
                    .or(options.revision_id.clone())
                    .ok_or_else(|| NativeCliError::usage("--base-revision-id <id> is required"))?,
                validation_evidence_ids: options.validation_evidence_ids,
                output: options.output,
            }),
            _ => Err(NativeCliError::usage("unsupported native case command")),
        }
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

#[derive(Default)]
struct NativeOptions {
    store: Option<PathBuf>,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    case_space_id: Option<Id>,
    space_id: Option<Id>,
    revision_id: Option<Id>,
    base_revision_id: Option<Id>,
    morphism_id: Option<Id>,
    reviewer_id: Option<Id>,
    title: Option<String>,
    reason: Option<String>,
    validation_evidence_ids: Vec<Id>,
}

impl NativeOptions {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, NativeCliError> {
        let mut options = Self::default();
        let mut format_seen = false;
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--format") => {
                    require_json_format(&mut args)?;
                    format_seen = true;
                }
                Some("--store") => options.store = Some(require_path(&mut args, "--store")?),
                Some("--input") => options.input = Some(require_path(&mut args, "--input")?),
                Some("--output") => options.output = Some(require_path(&mut args, "--output")?),
                Some("--case-space-id") => {
                    options.case_space_id = Some(require_id(&mut args, "--case-space-id")?)
                }
                Some("--space-id") => options.space_id = Some(require_id(&mut args, "--space-id")?),
                Some("--revision-id") => {
                    options.revision_id = Some(require_id(&mut args, "--revision-id")?)
                }
                Some("--base-revision") | Some("--base-revision-id") => {
                    options.base_revision_id = Some(require_id(&mut args, "--base-revision-id")?)
                }
                Some("--morphism-id") => {
                    options.morphism_id = Some(require_id(&mut args, "--morphism-id")?)
                }
                Some("--reviewer-id") => {
                    options.reviewer_id = Some(require_id(&mut args, "--reviewer-id")?)
                }
                Some("--title") => options.title = Some(require_string(&mut args, "--title")?),
                Some("--reason") => options.reason = Some(require_string(&mut args, "--reason")?),
                Some("--validation-evidence-id") => options
                    .validation_evidence_ids
                    .push(require_id(&mut args, "--validation-evidence-id")?),
                Some(_) | None => {
                    return Err(NativeCliError::usage(format!(
                        "unsupported native argument {arg:?}"
                    )))
                }
            }
        }
        if !format_seen {
            return Err(NativeCliError::usage("--format json is required"));
        }
        Ok(options)
    }

    fn require_store(&self) -> Result<PathBuf, NativeCliError> {
        self.store
            .clone()
            .ok_or_else(|| NativeCliError::usage("--store <dir> is required"))
    }

    fn require_path(&self, flag: &str) -> Result<PathBuf, NativeCliError> {
        match flag {
            "--input" => self.input.clone(),
            _ => None,
        }
        .ok_or_else(|| NativeCliError::usage(format!("{flag} <path> is required")))
    }

    fn require_id(&self, flag: &str) -> Result<Id, NativeCliError> {
        match flag {
            "--case-space-id" => self.case_space_id.clone(),
            "--space-id" => self.space_id.clone(),
            "--revision-id" => self.revision_id.clone(),
            "--reviewer-id" => self.reviewer_id.clone(),
            "--morphism-id" => self.morphism_id.clone(),
            _ => None,
        }
        .ok_or_else(|| NativeCliError::usage(format!("{flag} <id> is required")))
    }

    fn require_string(&self, flag: &str) -> Result<String, NativeCliError> {
        match flag {
            "--title" => self.title.clone(),
            "--reason" => self.reason.clone(),
            _ => None,
        }
        .ok_or_else(|| NativeCliError::usage(format!("{flag} <text> is required")))
    }
}

fn required_segment(
    args: &mut impl Iterator<Item = OsString>,
    label: &str,
) -> Result<OsString, NativeCliError> {
    args.next()
        .ok_or_else(|| NativeCliError::usage(format!("{label} is required")))
}

fn require_json_format(args: &mut impl Iterator<Item = OsString>) -> Result<(), NativeCliError> {
    match required_segment(args, "--format value")?.to_str() {
        Some("json") => Ok(()),
        Some(_) | None => Err(NativeCliError::usage("--format json is required")),
    }
}

fn require_path(
    args: &mut impl Iterator<Item = OsString>,
    flag: &str,
) -> Result<PathBuf, NativeCliError> {
    let value = required_segment(args, flag)?;
    let path = PathBuf::from(value);
    reject_unsafe_path(flag, &path)?;
    Ok(path)
}

fn require_string(
    args: &mut impl Iterator<Item = OsString>,
    flag: &str,
) -> Result<String, NativeCliError> {
    required_segment(args, flag)?
        .into_string()
        .map_err(|_| NativeCliError::usage(format!("{flag} must be UTF-8")))
}

fn require_id(args: &mut impl Iterator<Item = OsString>, flag: &str) -> Result<Id, NativeCliError> {
    Ok(Id::new(require_string(args, flag)?)?)
}

fn reject_unsafe_path(flag: &str, path: &Path) -> Result<(), NativeCliError> {
    if path.as_os_str().is_empty() {
        return Err(NativeCliError::usage(format!("{flag} must not be empty")));
    }
    Ok(())
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
