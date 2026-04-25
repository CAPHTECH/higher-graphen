//! Command-line entry point for HigherGraphen workflows.

use higher_graphen_runtime::{run_architecture_direct_db_access_smoke, RuntimeError};
use std::{env, ffi::OsString, fmt, fs, path::PathBuf, process::ExitCode};

const USAGE: &str =
    "usage: highergraphen architecture smoke direct-db-access --format json [--output <path>]";

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
    let report = run_architecture_direct_db_access_smoke()?;
    let json = serde_json::to_string(&report)
        .map_err(|error| RuntimeError::serialization(error.to_string()))?;

    match command.output {
        Some(path) => fs::write(path, json).map_err(CliError::write_output),
        None => {
            println!("{json}");
            Ok(())
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Command {
    output: Option<PathBuf>,
}

impl Command {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, CliError> {
        let mut args = args.into_iter();
        require_token(&mut args, "architecture")?;
        require_token(&mut args, "smoke")?;
        require_token(&mut args, "direct-db-access")?;

        let mut format_seen = false;
        let mut output = None;

        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--output" {
                output = Some(require_output_path(&mut args)?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(Self { output })
    }
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Runtime(RuntimeError),
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

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n{USAGE}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::Output(error) => write!(formatter, "failed to write output: {error}"),
        }
    }
}

impl std::error::Error for CliError {}

fn require_token(
    args: &mut impl Iterator<Item = OsString>,
    expected: &'static str,
) -> Result<(), CliError> {
    match args.next() {
        Some(arg) if arg == expected => Ok(()),
        Some(arg) => Err(CliError::usage(format!(
            "unsupported command segment {arg:?}; expected {expected:?}"
        ))),
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

fn require_output_path(args: &mut impl Iterator<Item = OsString>) -> Result<PathBuf, CliError> {
    match args.next() {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        Some(_) => Err(CliError::usage("empty output path")),
        None => Err(CliError::usage("missing value for --output")),
    }
}
