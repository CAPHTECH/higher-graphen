use super::{parse_projection_audience, NativeCliError};
use crate::native_model::ProjectionAudience;
use crate::topology::TopologyReportOptions;
use higher_graphen_core::Id;
use higher_graphen_structure::space::Dimension;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub(super) struct NativeOptions {
    pub(super) store: Option<PathBuf>,
    pub(super) left_store: Option<PathBuf>,
    pub(super) right_store: Option<PathBuf>,
    pub(super) input: Option<PathBuf>,
    pub(super) output: Option<PathBuf>,
    pub(super) case_space_id: Option<Id>,
    pub(super) left_case_space_id: Option<Id>,
    pub(super) right_case_space_id: Option<Id>,
    pub(super) space_id: Option<Id>,
    pub(super) revision_id: Option<Id>,
    pub(super) base_revision_id: Option<Id>,
    pub(super) morphism_id: Option<Id>,
    pub(super) reviewer_id: Option<Id>,
    pub(super) close_policy_id: Option<Id>,
    pub(super) actor_id: Option<Id>,
    pub(super) capability_ids: Vec<Id>,
    pub(super) operation_scope_id: Option<Id>,
    pub(super) audience: Option<ProjectionAudience>,
    pub(super) source_boundary_id: Option<Id>,
    pub(super) title: Option<String>,
    pub(super) reason: Option<String>,
    pub(super) validation_evidence_ids: Vec<Id>,
    pub(super) higher_order: bool,
    pub(super) max_dimension: Option<Dimension>,
    pub(super) min_persistence_stages: usize,
}

impl NativeOptions {
    pub(super) fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, NativeCliError> {
        let mut options = Self::default();
        let mut format_seen = false;
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            options.consume_arg(&arg, &mut args, &mut format_seen)?;
        }
        if !format_seen {
            return Err(NativeCliError::usage("--format json is required"));
        }
        Ok(options)
    }

    fn consume_arg(
        &mut self,
        arg: &OsString,
        args: &mut impl Iterator<Item = OsString>,
        format_seen: &mut bool,
    ) -> Result<(), NativeCliError> {
        match arg.to_str() {
            Some("--format") => {
                require_json_format(args)?;
                *format_seen = true;
            }
            Some("--store") => self.store = Some(require_path(args, "--store")?),
            Some("--left-store") => self.left_store = Some(require_path(args, "--left-store")?),
            Some("--right-store") => self.right_store = Some(require_path(args, "--right-store")?),
            Some("--input") => self.input = Some(require_path(args, "--input")?),
            Some("--output") => self.output = Some(require_path(args, "--output")?),
            Some("--case-space-id") => {
                self.case_space_id = Some(require_id(args, "--case-space-id")?)
            }
            Some("--left-case-space-id") => {
                self.left_case_space_id = Some(require_id(args, "--left-case-space-id")?)
            }
            Some("--right-case-space-id") => {
                self.right_case_space_id = Some(require_id(args, "--right-case-space-id")?)
            }
            Some("--space-id") => self.space_id = Some(require_id(args, "--space-id")?),
            Some("--revision-id") => self.revision_id = Some(require_id(args, "--revision-id")?),
            Some("--base-revision") | Some("--base-revision-id") => {
                self.base_revision_id = Some(require_id(args, "--base-revision-id")?)
            }
            Some("--morphism-id") => self.morphism_id = Some(require_id(args, "--morphism-id")?),
            Some("--reviewer-id") => self.reviewer_id = Some(require_id(args, "--reviewer-id")?),
            Some("--close-policy-id") => {
                self.close_policy_id = Some(require_id(args, "--close-policy-id")?)
            }
            Some("--actor-id") => self.actor_id = Some(require_id(args, "--actor-id")?),
            Some("--capability-id") => self
                .capability_ids
                .push(require_id(args, "--capability-id")?),
            Some("--operation-scope-id") => {
                self.operation_scope_id = Some(require_id(args, "--operation-scope-id")?)
            }
            Some("--audience") => {
                self.audience = Some(parse_projection_audience(&require_string(
                    args,
                    "--audience",
                )?)?)
            }
            Some("--source-boundary-id") => {
                self.source_boundary_id = Some(require_id(args, "--source-boundary-id")?)
            }
            Some("--title") => self.title = Some(require_string(args, "--title")?),
            Some("--reason") => self.reason = Some(require_string(args, "--reason")?),
            Some("--validation-evidence-id") => self
                .validation_evidence_ids
                .push(require_id(args, "--validation-evidence-id")?),
            Some("--higher-order") => self.higher_order = true,
            Some("--max-dimension") => {
                self.max_dimension = Some(require_dimension(args, "--max-dimension")?)
            }
            Some("--min-persistence") | Some("--min-persistence-stages") => {
                self.min_persistence_stages = require_usize(args, "--min-persistence")?;
            }
            Some(_) | None => {
                return Err(NativeCliError::usage(format!(
                    "unsupported native argument {arg:?}"
                )))
            }
        }
        Ok(())
    }

    pub(super) fn topology_options(&self) -> TopologyReportOptions {
        if self.higher_order {
            TopologyReportOptions::higher_order(self.max_dimension, self.min_persistence_stages)
        } else {
            TopologyReportOptions::baseline()
        }
    }

    pub(super) fn require_store(&self) -> Result<PathBuf, NativeCliError> {
        self.store
            .clone()
            .ok_or_else(|| NativeCliError::usage("--store <dir> is required"))
    }

    pub(super) fn require_path(&self, flag: &str) -> Result<PathBuf, NativeCliError> {
        match flag {
            "--input" => self.input.clone(),
            "--left-store" => self.left_store.clone(),
            "--right-store" => self.right_store.clone(),
            _ => None,
        }
        .ok_or_else(|| NativeCliError::usage(format!("{flag} <path> is required")))
    }

    pub(super) fn require_id(&self, flag: &str) -> Result<Id, NativeCliError> {
        match flag {
            "--case-space-id" => self.case_space_id.clone(),
            "--left-case-space-id" => self.left_case_space_id.clone(),
            "--right-case-space-id" => self.right_case_space_id.clone(),
            "--space-id" => self.space_id.clone(),
            "--revision-id" => self.revision_id.clone(),
            "--reviewer-id" => self.reviewer_id.clone(),
            "--morphism-id" => self.morphism_id.clone(),
            _ => None,
        }
        .ok_or_else(|| NativeCliError::usage(format!("{flag} <id> is required")))
    }

    pub(super) fn require_string(&self, flag: &str) -> Result<String, NativeCliError> {
        match flag {
            "--title" => self.title.clone(),
            "--reason" => self.reason.clone(),
            _ => None,
        }
        .ok_or_else(|| NativeCliError::usage(format!("{flag} <text> is required")))
    }
}

pub(super) fn required_segment(
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

pub(super) fn require_path(
    args: &mut impl Iterator<Item = OsString>,
    flag: &str,
) -> Result<PathBuf, NativeCliError> {
    let value = required_segment(args, flag)?;
    let path = PathBuf::from(value);
    reject_unsafe_path(flag, &path)?;
    Ok(path)
}

pub(super) fn require_string(
    args: &mut impl Iterator<Item = OsString>,
    flag: &str,
) -> Result<String, NativeCliError> {
    required_segment(args, flag)?
        .into_string()
        .map_err(|_| NativeCliError::usage(format!("{flag} must be UTF-8")))
}

pub(super) fn require_id(
    args: &mut impl Iterator<Item = OsString>,
    flag: &str,
) -> Result<Id, NativeCliError> {
    Ok(Id::new(require_string(args, flag)?)?)
}

fn require_dimension(
    args: &mut impl Iterator<Item = OsString>,
    flag: &str,
) -> Result<Dimension, NativeCliError> {
    require_string(args, flag)?
        .parse::<Dimension>()
        .map_err(|_| NativeCliError::usage(format!("invalid integer for {flag}")))
}

fn require_usize(
    args: &mut impl Iterator<Item = OsString>,
    flag: &str,
) -> Result<usize, NativeCliError> {
    require_string(args, flag)?
        .parse::<usize>()
        .map_err(|_| NativeCliError::usage(format!("invalid integer for {flag}")))
}

fn reject_unsafe_path(flag: &str, path: &Path) -> Result<(), NativeCliError> {
    if path.as_os_str().is_empty() {
        return Err(NativeCliError::usage(format!("{flag} must not be empty")));
    }
    Ok(())
}
