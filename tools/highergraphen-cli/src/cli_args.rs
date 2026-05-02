use crate::cli_error::CliError;
use std::{ffi::OsString, path::PathBuf};

pub(crate) fn require_token(
    args: &mut impl Iterator<Item = OsString>,
    expected: &'static str,
) -> Result<(), CliError> {
    match required_segment(args, expected)? {
        arg if arg == expected => Ok(()),
        arg => Err(CliError::usage(format!(
            "unsupported command segment {arg:?}; expected {expected:?}"
        ))),
    }
}

pub(crate) fn required_segment(
    args: &mut impl Iterator<Item = OsString>,
    expected: &'static str,
) -> Result<OsString, CliError> {
    match args.next() {
        Some(arg) => Ok(arg),
        None => Err(CliError::usage(format!(
            "missing command segment {expected:?}"
        ))),
    }
}

fn require_json_format(args: &mut impl Iterator<Item = OsString>) -> Result<(), CliError> {
    match args.next() {
        Some(arg) if arg == "json" => Ok(()),
        Some(arg) => Err(CliError::usage(format!(
            "unsupported format {arg:?}; only json is supported"
        ))),
        None => Err(CliError::usage("missing value for --format")),
    }
}

fn require_path(
    args: &mut impl Iterator<Item = OsString>,
    option: &'static str,
) -> Result<PathBuf, CliError> {
    match args.next() {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        Some(_) => Err(CliError::usage(format!("empty path for {option}"))),
        None => Err(CliError::usage(format!("missing value for {option}"))),
    }
}

fn require_string(
    args: &mut impl Iterator<Item = OsString>,
    option: &'static str,
) -> Result<String, CliError> {
    match args.next() {
        Some(value) if !value.is_empty() => value
            .into_string()
            .map_err(|value| CliError::usage(format!("non-utf8 value for {option}: {value:?}"))),
        Some(_) => Err(CliError::usage(format!("empty value for {option}"))),
        None => Err(CliError::usage(format!("missing value for {option}"))),
    }
}

fn require_format_seen(format_seen: bool) -> Result<(), CliError> {
    if format_seen {
        Ok(())
    } else {
        Err(CliError::usage("--format json is required"))
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct GitInputOptions {
    pub(crate) repo: Option<PathBuf>,
    pub(crate) base: Option<String>,
    pub(crate) head: Option<String>,
    pub(crate) binding_rules: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

impl GitInputOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--repo" {
                options.repo = Some(require_path(&mut args, "--repo")?);
            } else if arg == "--base" {
                options.base = Some(require_string(&mut args, "--base")?);
            } else if arg == "--head" {
                options.head = Some(require_string(&mut args, "--head")?);
            } else if arg == "--binding-rules" {
                options.binding_rules = Some(require_path(&mut args, "--binding-rules")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct PathInputOptions {
    pub(crate) repo: Option<PathBuf>,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) include_tests: bool,
    pub(crate) binding_rules: Option<PathBuf>,
    pub(crate) test_run: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

impl PathInputOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--repo" {
                options.repo = Some(require_path(&mut args, "--repo")?);
            } else if arg == "--path" {
                options.paths.push(require_path(&mut args, "--path")?);
            } else if arg == "--include-tests" {
                options.include_tests = true;
            } else if arg == "--binding-rules" {
                options.binding_rules = Some(require_path(&mut args, "--binding-rules")?);
            } else if arg == "--test-run" {
                options.test_run = Some(require_path(&mut args, "--test-run")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct TestSemanticsInterpretOptions {
    pub(crate) input: Option<PathBuf>,
    pub(crate) interpreter: Option<String>,
    pub(crate) output: Option<PathBuf>,
}

impl TestSemanticsInterpretOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--input" {
                options.input = Some(require_path(&mut args, "--input")?);
            } else if arg == "--interpreter" {
                options.interpreter = Some(require_string(&mut args, "--interpreter")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct TestSemanticsReviewOptions {
    pub(crate) input: Option<PathBuf>,
    pub(crate) candidate_id: Option<String>,
    pub(crate) reviewer_id: Option<String>,
    pub(crate) reason: Option<String>,
    pub(crate) output: Option<PathBuf>,
}

impl TestSemanticsReviewOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--input" {
                options.input = Some(require_path(&mut args, "--input")?);
            } else if arg == "--candidate" {
                options.candidate_id = Some(require_string(&mut args, "--candidate")?);
            } else if arg == "--reviewer" {
                options.reviewer_id = Some(require_string(&mut args, "--reviewer")?);
            } else if arg == "--reason" {
                options.reason = Some(require_string(&mut args, "--reason")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct TestSemanticsVerifyOptions {
    pub(crate) interpretation: Option<PathBuf>,
    pub(crate) review: Option<PathBuf>,
    pub(crate) test_run: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

impl TestSemanticsVerifyOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--interpretation" {
                options.interpretation = Some(require_path(&mut args, "--interpretation")?);
            } else if arg == "--review" {
                options.review = Some(require_path(&mut args, "--review")?);
            } else if arg == "--test-run" {
                options.test_run = Some(require_path(&mut args, "--test-run")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct TestSemanticsGapOptions {
    pub(crate) expected: Option<PathBuf>,
    pub(crate) verified: Vec<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

impl TestSemanticsGapOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--expected" {
                options.expected = Some(require_path(&mut args, "--expected")?);
            } else if arg == "--verified" {
                options
                    .verified
                    .push(require_path(&mut args, "--verified")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct TestRunEvidenceOptions {
    pub(crate) input: Option<PathBuf>,
    pub(crate) test_run: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct DddCaseSpaceInputOptions {
    pub(crate) case_space: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

impl DddCaseSpaceInputOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--case-space" {
                options.case_space = Some(require_path(&mut args, "--case-space")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

impl TestRunEvidenceOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--input" {
                options.input = Some(require_path(&mut args, "--input")?);
            } else if arg == "--test-run" {
                options.test_run = Some(require_path(&mut args, "--test-run")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct SemanticProofBackendOptions {
    pub(crate) backend: Option<String>,
    pub(crate) backend_version: Option<String>,
    pub(crate) command: Option<PathBuf>,
    pub(crate) args: Vec<String>,
    pub(crate) input: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

impl SemanticProofBackendOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--backend" {
                options.backend = Some(require_string(&mut args, "--backend")?);
            } else if arg == "--backend-version" {
                options.backend_version = Some(require_string(&mut args, "--backend-version")?);
            } else if arg == "--command" {
                options.command = Some(require_path(&mut args, "--command")?);
            } else if arg == "--arg" {
                options.args.push(require_string(&mut args, "--arg")?);
            } else if arg == "--input" {
                options.input = Some(require_path(&mut args, "--input")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct SemanticProofArtifactOptions {
    pub(crate) artifact: Option<PathBuf>,
    pub(crate) backend: Option<String>,
    pub(crate) backend_version: Option<String>,
    pub(crate) theorem_id: Option<String>,
    pub(crate) theorem_summary: Option<String>,
    pub(crate) law_id: Option<String>,
    pub(crate) law_summary: Option<String>,
    pub(crate) morphism_id: Option<String>,
    pub(crate) morphism_type: Option<String>,
    pub(crate) base_cell: Option<String>,
    pub(crate) base_label: Option<String>,
    pub(crate) head_cell: Option<String>,
    pub(crate) head_label: Option<String>,
    pub(crate) output: Option<PathBuf>,
}

impl SemanticProofArtifactOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();
        let mut args = args;

        while let Some(arg) = args.next() {
            format_seen |= options.parse_arg(&mut args, arg)?;
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }

    fn parse_arg(
        &mut self,
        args: &mut impl Iterator<Item = OsString>,
        arg: OsString,
    ) -> Result<bool, CliError> {
        match arg.to_str() {
            Some("--format") => {
                require_json_format(args)?;
                Ok(true)
            }
            Some("--artifact") => {
                self.artifact = Some(require_path(args, "--artifact")?);
                Ok(false)
            }
            Some("--backend") => {
                self.backend = Some(require_string(args, "--backend")?);
                Ok(false)
            }
            Some("--backend-version") => {
                self.backend_version = Some(require_string(args, "--backend-version")?);
                Ok(false)
            }
            Some("--theorem-id") => {
                self.theorem_id = Some(require_string(args, "--theorem-id")?);
                Ok(false)
            }
            Some("--theorem-summary") => {
                self.theorem_summary = Some(require_string(args, "--theorem-summary")?);
                Ok(false)
            }
            Some("--law-id") => {
                self.law_id = Some(require_string(args, "--law-id")?);
                Ok(false)
            }
            Some("--law-summary") => {
                self.law_summary = Some(require_string(args, "--law-summary")?);
                Ok(false)
            }
            Some("--morphism-id") => {
                self.morphism_id = Some(require_string(args, "--morphism-id")?);
                Ok(false)
            }
            Some("--morphism-type") => {
                self.morphism_type = Some(require_string(args, "--morphism-type")?);
                Ok(false)
            }
            Some("--base-cell") => {
                self.base_cell = Some(require_string(args, "--base-cell")?);
                Ok(false)
            }
            Some("--base-label") => {
                self.base_label = Some(require_string(args, "--base-label")?);
                Ok(false)
            }
            Some("--head-cell") => {
                self.head_cell = Some(require_string(args, "--head-cell")?);
                Ok(false)
            }
            Some("--head-label") => {
                self.head_label = Some(require_string(args, "--head-label")?);
                Ok(false)
            }
            Some("--output") => {
                self.output = Some(require_path(args, "--output")?);
                Ok(false)
            }
            Some(_) | None => Err(CliError::usage(format!("unsupported argument {arg:?}"))),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct SemanticProofReportInputOptions {
    pub(crate) report: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

impl SemanticProofReportInputOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--report" {
                options.report = Some(require_path(&mut args, "--report")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct SemanticProofAttachArtifactOptions {
    pub(crate) input: Option<PathBuf>,
    pub(crate) artifact: Option<PathBuf>,
    pub(crate) backend: Option<String>,
    pub(crate) backend_version: Option<String>,
    pub(crate) output: Option<PathBuf>,
}

impl SemanticProofAttachArtifactOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--input" {
                options.input = Some(require_path(&mut args, "--input")?);
            } else if arg == "--artifact" {
                options.artifact = Some(require_path(&mut args, "--artifact")?);
            } else if arg == "--backend" {
                options.backend = Some(require_string(&mut args, "--backend")?);
            } else if arg == "--backend-version" {
                options.backend_version = Some(require_string(&mut args, "--backend-version")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct ReportOptions {
    pub(crate) input: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

impl ReportOptions {
    pub(crate) fn parse(
        args: impl Iterator<Item = OsString>,
        allow_input: bool,
    ) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else if arg == "--input" && allow_input {
                options.input = Some(require_path(&mut args, "--input")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct ReviewOptions {
    pub(crate) input: Option<PathBuf>,
    pub(crate) candidate_id: Option<String>,
    pub(crate) reviewer_id: Option<String>,
    pub(crate) reason: Option<String>,
    pub(crate) reviewed_at: Option<String>,
    pub(crate) output: Option<PathBuf>,
}

impl ReviewOptions {
    pub(crate) fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--input" {
                options.input = Some(require_path(&mut args, "--input")?);
            } else if arg == "--candidate" {
                options.candidate_id = Some(require_string(&mut args, "--candidate")?);
            } else if arg == "--reviewer" {
                options.reviewer_id = Some(require_string(&mut args, "--reviewer")?);
            } else if arg == "--reason" {
                options.reason = Some(require_string(&mut args, "--reason")?);
            } else if arg == "--reviewed-at" {
                options.reviewed_at = Some(require_string(&mut args, "--reviewed-at")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        require_format_seen(format_seen)?;
        Ok(options)
    }
}
