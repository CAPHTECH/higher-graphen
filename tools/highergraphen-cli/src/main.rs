//! Command-line entry point for HigherGraphen workflows.

use higher_graphen_core::Id;
use higher_graphen_runtime::{
    run_architecture_direct_db_access_smoke, run_architecture_input_lift, run_completion_review,
    run_feed_reader, ArchitectureInputLiftDocument, CompletionReviewDecision,
    CompletionReviewRequest, CompletionReviewSnapshot, CompletionReviewSourceReport,
    FeedReaderInputDocument, RuntimeError,
};
use serde_json::Value;
use std::{
    env,
    ffi::OsString,
    fmt, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const USAGE: &str = "usage:
  highergraphen architecture smoke direct-db-access --format json [--output <path>]
  highergraphen architecture input lift --input <path> --format json [--output <path>]
  highergraphen feed reader run --input <path> --format json [--output <path>]
  highergraphen completion review accept --input <path> --candidate <id> --reviewer <id> --reason <text> --format json [--reviewed-at <text>] [--output <path>]
  highergraphen completion review reject --input <path> --candidate <id> --reviewer <id> --reason <text> --format json [--reviewed-at <text>] [--output <path>]";

fn main() -> ExitCode {
    match run(env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), CliError> {
    let command = Command::parse(args)?;
    let output = command.output().cloned();
    let json = command.run_json()?;

    match output {
        Some(path) => fs::write(path, json).map_err(CliError::write_output),
        None => {
            println!("{json}");
            Ok(())
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    ArchitectureSmokeDirectDbAccess {
        output: Option<PathBuf>,
    },
    ArchitectureInputLift {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    FeedReaderRun {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    CompletionReview {
        decision: CompletionReviewDecision,
        input: PathBuf,
        candidate_id: String,
        reviewer_id: String,
        reason: String,
        reviewed_at: Option<String>,
        output: Option<PathBuf>,
    },
}

impl Command {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, CliError> {
        let mut args = args.into_iter();
        let root = required_segment(&mut args, "command")?;

        match root.to_str() {
            Some("architecture") => Self::parse_architecture(args),
            Some("feed") => Self::parse_feed(args),
            Some("completion") => Self::parse_completion(args),
            Some(_) | None => Err(CliError::usage("unsupported command segment")),
        }
    }

    fn parse_architecture(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "architecture command")?;
        match segment.to_str() {
            Some("smoke") => Self::parse_smoke(args),
            Some("input") => Self::parse_input(args),
            Some(_) | None => Err(CliError::usage("unsupported architecture command segment")),
        }
    }

    fn parse_smoke(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "direct-db-access")?;
        let options = ReportOptions::parse(args, false)?;
        Ok(Self::ArchitectureSmokeDirectDbAccess {
            output: options.output,
        })
    }

    fn parse_input(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "lift")?;
        let options = ReportOptions::parse(args, true)?;
        let input = options
            .input
            .ok_or_else(|| CliError::usage("--input <path> is required"))?;
        Ok(Self::ArchitectureInputLift {
            input,
            output: options.output,
        })
    }

    fn parse_feed(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "reader")?;
        require_token(&mut args, "run")?;
        let options = ReportOptions::parse(args, true)?;
        let input = options
            .input
            .ok_or_else(|| CliError::usage("--input <path> is required"))?;
        Ok(Self::FeedReaderRun {
            input,
            output: options.output,
        })
    }

    fn parse_completion(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "review")?;
        let decision = match required_segment(&mut args, "completion review action")?.to_str() {
            Some("accept") => CompletionReviewDecision::Accepted,
            Some("reject") => CompletionReviewDecision::Rejected,
            Some(_) | None => return Err(CliError::usage("unsupported completion review action")),
        };
        let options = ReviewOptions::parse(args)?;

        Ok(Self::CompletionReview {
            decision,
            input: options
                .input
                .ok_or_else(|| CliError::usage("--input <path> is required"))?,
            candidate_id: options
                .candidate_id
                .ok_or_else(|| CliError::usage("--candidate <id> is required"))?,
            reviewer_id: options
                .reviewer_id
                .ok_or_else(|| CliError::usage("--reviewer <id> is required"))?,
            reason: options
                .reason
                .ok_or_else(|| CliError::usage("--reason <text> is required"))?,
            reviewed_at: options.reviewed_at,
            output: options.output,
        })
    }

    fn output(&self) -> Option<&PathBuf> {
        match self {
            Self::ArchitectureSmokeDirectDbAccess { output }
            | Self::ArchitectureInputLift { output, .. }
            | Self::FeedReaderRun { output, .. }
            | Self::CompletionReview { output, .. } => output.as_ref(),
        }
    }

    fn run_json(&self) -> Result<String, CliError> {
        match self {
            Self::ArchitectureSmokeDirectDbAccess { .. } => {
                let report = run_architecture_direct_db_access_smoke()?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::ArchitectureInputLift { input, .. } => {
                let document = read_input_document(input)?;
                let report = run_architecture_input_lift(document)?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::FeedReaderRun { input, .. } => {
                let document = read_feed_reader_input_document(input)?;
                let report = run_feed_reader(document)?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::CompletionReview {
                decision,
                input,
                candidate_id,
                reviewer_id,
                reason,
                reviewed_at,
                ..
            } => {
                let snapshot = read_completion_review_snapshot(input)?;
                let mut request = CompletionReviewRequest::new(
                    Id::new(candidate_id.clone())?,
                    *decision,
                    Id::new(reviewer_id.clone())?,
                    reason.clone(),
                )?;
                if let Some(reviewed_at) = reviewed_at {
                    request = request.with_reviewed_at(reviewed_at.clone())?;
                }
                let report = run_completion_review(snapshot, request)?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct ReportOptions {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl ReportOptions {
    fn parse(args: impl Iterator<Item = OsString>, allow_input: bool) -> Result<Self, CliError> {
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

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct ReviewOptions {
    input: Option<PathBuf>,
    candidate_id: Option<String>,
    reviewer_id: Option<String>,
    reason: Option<String>,
    reviewed_at: Option<String>,
    output: Option<PathBuf>,
}

impl ReviewOptions {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
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

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Runtime(RuntimeError),
    InputRead {
        path: PathBuf,
        source: std::io::Error,
    },
    InputParse {
        path: PathBuf,
        source: serde_json::Error,
    },
    InputContract {
        path: PathBuf,
        reason: String,
    },
    Output(std::io::Error),
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    fn write_output(error: std::io::Error) -> Self {
        Self::Output(error)
    }
}

impl From<RuntimeError> for CliError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

impl From<higher_graphen_core::CoreError> for CliError {
    fn from(error: higher_graphen_core::CoreError) -> Self {
        Self::Runtime(RuntimeError::from(error))
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n{USAGE}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::InputRead { path, source } => {
                write!(
                    formatter,
                    "failed to read input {}: {source}",
                    path.display()
                )
            }
            Self::InputParse { path, source } => {
                write!(
                    formatter,
                    "failed to parse input {}: {source}",
                    path.display()
                )
            }
            Self::InputContract { path, reason } => {
                write!(formatter, "invalid input {}: {reason}", path.display())
            }
            Self::Output(error) => write!(formatter, "failed to write output: {error}"),
        }
    }
}

impl std::error::Error for CliError {}

fn require_token(
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

fn required_segment(
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

fn read_input_document(path: &Path) -> Result<ArchitectureInputLiftDocument, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn read_feed_reader_input_document(path: &Path) -> Result<FeedReaderInputDocument, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn read_completion_review_snapshot(path: &Path) -> Result<CompletionReviewSnapshot, CliError> {
    let value = read_json_value(path)?;
    if value.get("source_report").is_some() && value.get("completion_candidates").is_some() {
        return serde_json::from_value(value).map_err(|source| CliError::InputParse {
            path: path.to_owned(),
            source,
        });
    }

    snapshot_from_report_value(path, &value)
}

fn read_json_value(path: &Path) -> Result<Value, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn snapshot_from_report_value(
    path: &Path,
    value: &Value,
) -> Result<CompletionReviewSnapshot, CliError> {
    let candidates = dig_json(value, &["result", "completion_candidates"])
        .ok_or_else(|| input_contract(path, "missing result.completion_candidates"))?;
    let completion_candidates =
        serde_json::from_value(candidates.clone()).map_err(|source| CliError::InputParse {
            path: path.to_owned(),
            source,
        })?;

    Ok(CompletionReviewSnapshot {
        source_report: CompletionReviewSourceReport {
            schema: required_json_string(path, value, &["schema"])?,
            report_type: required_json_string(path, value, &["report_type"])?,
            report_version: required_json_u32(path, value, &["report_version"])?,
            command: required_json_string(path, value, &["metadata", "command"])?,
        },
        completion_candidates,
    })
}

fn dig_json<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn required_json_string(
    input_path: &Path,
    value: &Value,
    json_path: &[&str],
) -> Result<String, CliError> {
    match dig_json(value, json_path) {
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(text.clone()),
        Some(_) => Err(input_contract(
            input_path,
            format!("{} must be a non-empty string", json_path.join(".")),
        )),
        None => Err(input_contract(
            input_path,
            format!("missing {}", json_path.join(".")),
        )),
    }
}

fn required_json_u32(
    input_path: &Path,
    value: &Value,
    json_path: &[&str],
) -> Result<u32, CliError> {
    match dig_json(value, json_path).and_then(Value::as_u64) {
        Some(number) => u32::try_from(number).map_err(|_| {
            input_contract(
                input_path,
                format!("{} must fit in u32", json_path.join(".")),
            )
        }),
        None => Err(input_contract(
            input_path,
            format!("{} must be a non-negative integer", json_path.join(".")),
        )),
    }
}

fn input_contract(path: &Path, reason: impl Into<String>) -> CliError {
    CliError::InputContract {
        path: path.to_owned(),
        reason: reason.into(),
    }
}
