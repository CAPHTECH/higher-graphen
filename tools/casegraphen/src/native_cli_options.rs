use super::NativeCliError;
use crate::topology::TopologyReportOptions;
use higher_graphen_core::Id;
use higher_graphen_space::Dimension;
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
            match arg.to_str() {
                Some("--format") => {
                    require_json_format(&mut args)?;
                    format_seen = true;
                }
                Some("--store") => options.store = Some(require_path(&mut args, "--store")?),
                Some("--left-store") => {
                    options.left_store = Some(require_path(&mut args, "--left-store")?)
                }
                Some("--right-store") => {
                    options.right_store = Some(require_path(&mut args, "--right-store")?)
                }
                Some("--input") => options.input = Some(require_path(&mut args, "--input")?),
                Some("--output") => options.output = Some(require_path(&mut args, "--output")?),
                Some("--case-space-id") => {
                    options.case_space_id = Some(require_id(&mut args, "--case-space-id")?)
                }
                Some("--left-case-space-id") => {
                    options.left_case_space_id =
                        Some(require_id(&mut args, "--left-case-space-id")?)
                }
                Some("--right-case-space-id") => {
                    options.right_case_space_id =
                        Some(require_id(&mut args, "--right-case-space-id")?)
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
                Some("--higher-order") => {
                    options.higher_order = true;
                }
                Some("--max-dimension") => {
                    options.max_dimension = Some(require_dimension(&mut args, "--max-dimension")?)
                }
                Some("--min-persistence") | Some("--min-persistence-stages") => {
                    options.min_persistence_stages = require_usize(&mut args, "--min-persistence")?;
                }
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
