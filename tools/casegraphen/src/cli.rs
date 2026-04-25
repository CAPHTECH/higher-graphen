use crate::{
    eval::{detect_conflicts, detect_missing_cases, evaluate_coverage, validate_case_graph},
    model::{CaseGraph, ProjectionDefinition},
    report,
    store::{read_case_graph, read_coverage_policy, read_projection, write_report, LocalCaseStore},
};
use higher_graphen_core::Id;
use std::{
    env,
    ffi::OsString,
    fmt,
    path::{Path, PathBuf},
    process::ExitCode,
};

const USAGE: &str = "usage:
  casegraphen create --case-graph-id <id> --space-id <id> --store <dir> --format json [--output <path>]
  casegraphen inspect --input <path> --format json [--output <path>]
  casegraphen list --store <dir> --format json [--output <path>]
  casegraphen validate --input <path> --format json [--output <path>]
  casegraphen coverage --input <path> --coverage <path> --format json [--output <path>]
  casegraphen missing --input <path> --coverage <path> --format json [--output <path>]
  casegraphen conflicts --input <path> --format json [--output <path>]
  casegraphen project --input <path> --projection <path> --format json [--output <path>]
  casegraphen compare --left <path> --right <path> --format json [--output <path>]";

pub fn main_entry() -> ExitCode {
    match run(env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), CliError> {
    let command = Command::parse(args)?;
    let json = command.run_json()?;
    match command.output() {
        Some(path) => write_report(path, &serde_json::from_str::<serde_json::Value>(&json)?)
            .map_err(CliError::from),
        None => {
            println!("{json}");
            Ok(())
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Create {
        case_graph_id: String,
        space_id: String,
        store: PathBuf,
        output: Option<PathBuf>,
    },
    Inspect {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    List {
        store: PathBuf,
        output: Option<PathBuf>,
    },
    Validate {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    Coverage {
        input: PathBuf,
        coverage: PathBuf,
        output: Option<PathBuf>,
    },
    Missing {
        input: PathBuf,
        coverage: PathBuf,
        output: Option<PathBuf>,
    },
    Conflicts {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    Project {
        input: PathBuf,
        projection: PathBuf,
        output: Option<PathBuf>,
    },
    Compare {
        left: PathBuf,
        right: PathBuf,
        output: Option<PathBuf>,
    },
}

impl Command {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, CliError> {
        let mut args = args.into_iter();
        match required_segment(&mut args, "command")?.to_str() {
            Some("create") => Self::parse_create(args),
            Some("inspect") => {
                Self::parse_one_input(args, |input, output| Self::Inspect { input, output })
            }
            Some("list") => Self::parse_list(args),
            Some("validate") => {
                Self::parse_one_input(args, |input, output| Self::Validate { input, output })
            }
            Some("coverage") => {
                Self::parse_policy_command(args, |input, coverage, output| Self::Coverage {
                    input,
                    coverage,
                    output,
                })
            }
            Some("missing") => {
                Self::parse_policy_command(args, |input, coverage, output| Self::Missing {
                    input,
                    coverage,
                    output,
                })
            }
            Some("conflicts") => {
                Self::parse_one_input(args, |input, output| Self::Conflicts { input, output })
            }
            Some("project") => Self::parse_project(args),
            Some("compare") => Self::parse_compare(args),
            Some(_) | None => Err(CliError::usage("unsupported command segment")),
        }
    }

    fn parse_create(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let options = Options::parse(args)?;
        Ok(Self::Create {
            case_graph_id: options
                .case_graph_id
                .ok_or_else(|| CliError::usage("--case-graph-id <id> is required"))?,
            space_id: options
                .space_id
                .ok_or_else(|| CliError::usage("--space-id <id> is required"))?,
            store: options
                .store
                .ok_or_else(|| CliError::usage("--store <dir> is required"))?,
            output: options.output,
        })
    }

    fn parse_one_input(
        args: impl Iterator<Item = OsString>,
        constructor: impl FnOnce(PathBuf, Option<PathBuf>) -> Self,
    ) -> Result<Self, CliError> {
        let options = Options::parse(args)?;
        Ok(constructor(
            options
                .input
                .ok_or_else(|| CliError::usage("--input <path> is required"))?,
            options.output,
        ))
    }

    fn parse_list(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let options = Options::parse(args)?;
        Ok(Self::List {
            store: options
                .store
                .ok_or_else(|| CliError::usage("--store <dir> is required"))?,
            output: options.output,
        })
    }

    fn parse_policy_command(
        args: impl Iterator<Item = OsString>,
        constructor: impl FnOnce(PathBuf, PathBuf, Option<PathBuf>) -> Self,
    ) -> Result<Self, CliError> {
        let options = Options::parse(args)?;
        Ok(constructor(
            options
                .input
                .ok_or_else(|| CliError::usage("--input <path> is required"))?,
            options
                .coverage
                .ok_or_else(|| CliError::usage("--coverage <path> is required"))?,
            options.output,
        ))
    }

    fn parse_project(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let options = Options::parse(args)?;
        Ok(Self::Project {
            input: options
                .input
                .ok_or_else(|| CliError::usage("--input <path> is required"))?,
            projection: options
                .projection
                .ok_or_else(|| CliError::usage("--projection <path> is required"))?,
            output: options.output,
        })
    }

    fn parse_compare(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let options = Options::parse(args)?;
        Ok(Self::Compare {
            left: options
                .left
                .ok_or_else(|| CliError::usage("--left <path> is required"))?,
            right: options
                .right
                .ok_or_else(|| CliError::usage("--right <path> is required"))?,
            output: options.output,
        })
    }

    fn output(&self) -> Option<&PathBuf> {
        match self {
            Self::Create { output, .. }
            | Self::Inspect { output, .. }
            | Self::List { output, .. }
            | Self::Validate { output, .. }
            | Self::Coverage { output, .. }
            | Self::Missing { output, .. }
            | Self::Conflicts { output, .. }
            | Self::Project { output, .. }
            | Self::Compare { output, .. } => output.as_ref(),
        }
    }

    fn run_json(&self) -> Result<String, CliError> {
        match self {
            Self::Create {
                case_graph_id,
                space_id,
                store,
                ..
            } => run_create(case_graph_id, space_id, store),
            Self::Inspect { input, .. } => run_inspect(input),
            Self::List { store, .. } => run_list(store),
            Self::Validate { input, .. } => run_validate(input),
            Self::Coverage {
                input, coverage, ..
            } => run_coverage(input, coverage),
            Self::Missing {
                input, coverage, ..
            } => run_missing(input, coverage),
            Self::Conflicts { input, .. } => run_conflicts(input),
            Self::Project {
                input, projection, ..
            } => run_project(input, projection),
            Self::Compare { left, right, .. } => run_compare(left, right),
        }
    }
}

#[derive(Default)]
struct Options {
    input: Option<PathBuf>,
    coverage: Option<PathBuf>,
    projection: Option<PathBuf>,
    left: Option<PathBuf>,
    right: Option<PathBuf>,
    store: Option<PathBuf>,
    output: Option<PathBuf>,
    case_graph_id: Option<String>,
    space_id: Option<String>,
}

impl Options {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
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
                Some("--coverage") => {
                    options.coverage = Some(require_path(&mut args, "--coverage")?)
                }
                Some("--projection") => {
                    options.projection = Some(require_path(&mut args, "--projection")?);
                }
                Some("--left") => options.left = Some(require_path(&mut args, "--left")?),
                Some("--right") => options.right = Some(require_path(&mut args, "--right")?),
                Some("--store") => options.store = Some(require_path(&mut args, "--store")?),
                Some("--output") => options.output = Some(require_path(&mut args, "--output")?),
                Some("--case-graph-id") => {
                    options.case_graph_id = Some(require_string(&mut args, "--case-graph-id")?);
                }
                Some("--space-id") => {
                    options.space_id = Some(require_string(&mut args, "--space-id")?)
                }
                Some(_) | None => {
                    return Err(CliError::usage(format!("unsupported argument {arg:?}")))
                }
            }
        }
        require_format_seen(format_seen)?;
        Ok(options)
    }
}

fn run_create(case_graph_id: &str, space_id: &str, store: &Path) -> Result<String, CliError> {
    let graph = CaseGraph::empty(
        Id::new(case_graph_id.to_owned())?,
        Id::new(space_id.to_owned())?,
    );
    let path = LocalCaseStore::new(store.to_path_buf()).create_graph(&graph)?;
    serialize(&report::create_report("casegraphen create", &path, &graph))
}

fn run_inspect(input: &Path) -> Result<String, CliError> {
    let graph = read_case_graph(input)?;
    serialize(&report::inspect_report(
        "casegraphen inspect",
        input,
        &graph,
    ))
}

fn run_list(store: &Path) -> Result<String, CliError> {
    let entries = LocalCaseStore::new(store.to_path_buf()).list_graphs()?;
    serialize(&report::list_report("casegraphen list", store, entries))
}

fn run_validate(input: &Path) -> Result<String, CliError> {
    let graph = read_case_graph(input)?;
    let result = validate_case_graph(&graph);
    serialize(&report::validate_report(
        "casegraphen validate",
        input,
        &graph,
        result,
    ))
}

fn run_coverage(input: &Path, coverage: &Path) -> Result<String, CliError> {
    let graph = read_case_graph(input)?;
    let policy = read_coverage_policy(coverage)?;
    let result = evaluate_coverage(&graph, &policy);
    serialize(&report::coverage_report(
        "casegraphen coverage",
        input,
        coverage,
        &graph,
        result,
    ))
}

fn run_missing(input: &Path, coverage: &Path) -> Result<String, CliError> {
    let graph = read_case_graph(input)?;
    let policy = read_coverage_policy(coverage)?;
    let result = detect_missing_cases(&graph, &policy);
    serialize(&report::missing_report(
        "casegraphen missing",
        input,
        coverage,
        &graph,
        result,
    ))
}

fn run_conflicts(input: &Path) -> Result<String, CliError> {
    let graph = read_case_graph(input)?;
    let result = detect_conflicts(&graph);
    serialize(&report::conflicts_report(
        "casegraphen conflicts",
        input,
        &graph,
        result,
    ))
}

fn run_project(input: &Path, projection: &Path) -> Result<String, CliError> {
    let graph = read_case_graph(input)?;
    let _definition: ProjectionDefinition = read_projection(projection)?;
    serialize(&report::project_report(
        "casegraphen project",
        input,
        projection,
        &graph,
        report::operation_projection(&graph),
    ))
}

fn run_compare(left: &Path, right: &Path) -> Result<String, CliError> {
    let left_graph = read_case_graph(left)?;
    let right_graph = read_case_graph(right)?;
    serialize(&report::compare_report(
        "casegraphen compare",
        left,
        right,
        &left_graph,
        &right_graph,
    ))
}

fn serialize(report: &impl serde::Serialize) -> Result<String, CliError> {
    serde_json::to_string(report).map_err(CliError::from)
}

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Core(higher_graphen_core::CoreError),
    Store(crate::store::StoreError),
    Json(serde_json::Error),
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }
}

impl From<higher_graphen_core::CoreError> for CliError {
    fn from(error: higher_graphen_core::CoreError) -> Self {
        Self::Core(error)
    }
}

impl From<crate::store::StoreError> for CliError {
    fn from(error: crate::store::StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n{USAGE}"),
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for CliError {}

fn required_segment(
    args: &mut impl Iterator<Item = OsString>,
    expected: &'static str,
) -> Result<OsString, CliError> {
    args.next()
        .ok_or_else(|| CliError::usage(format!("missing command segment {expected:?}")))
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

fn require_format_seen(format_seen: bool) -> Result<(), CliError> {
    if format_seen {
        Ok(())
    } else {
        Err(CliError::usage("--format json is required"))
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
