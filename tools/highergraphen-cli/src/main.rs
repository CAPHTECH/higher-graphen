//! Command-line entry point for HigherGraphen workflows.

mod cli_args;
mod cli_error;
mod command;
mod command_parse;
mod command_run;
mod pr_review_git;
mod pr_review_structural;
mod rust_test_semantics;
mod semantic_proof_artifact;
mod semantic_proof_attach_artifact;
mod semantic_proof_backend;
mod semantic_proof_reinput;
mod test_gap_evidence;
mod test_gap_git;
mod test_semantics_gap;
mod test_semantics_interpretation;
mod test_semantics_review;
mod test_semantics_verification;

use cli_error::CliError;
use command::Command;
use std::{env, ffi::OsString, fs, process::ExitCode};

pub(crate) const USAGE: &str = "usage:
  highergraphen version
  highergraphen --version
  highergraphen architecture smoke direct-db-access --format json [--output <path>]
  highergraphen architecture input lift --input <path> --format json [--output <path>]
  highergraphen feed reader run --input <path> --format json [--output <path>]
  highergraphen ddd input from-case-space --case-space <path> --format json [--output <path>]
  highergraphen ddd review --input <path> --format json [--output <path>]
  highergraphen pr-review input from-git --base <ref> --head <ref> --format json [--repo <path>] [--output <path>]
  highergraphen pr-review targets recommend --input <path> --format json [--output <path>]
  highergraphen test-gap input from-git --base <ref> --head <ref> --format json [--repo <path>] [--binding-rules <path>] [--output <path>]
  highergraphen test-gap input from-path --path <path> [--path <path> ...] [--include-tests] --format json [--repo <path>] [--binding-rules <path>] [--output <path>]
  highergraphen test-gap evidence from-test-run --input <path> --test-run <path> --format json [--output <path>]
  highergraphen test-gap detect --input <path> --format json [--output <path>]
  highergraphen test-semantics interpret --input <path> --format json [--interpreter <id>] [--output <path>]
  highergraphen test-semantics review accept --input <path> --candidate <id> --reviewer <id> --reason <text> --format json [--output <path>]
  highergraphen test-semantics review reject --input <path> --candidate <id> --reviewer <id> --reason <text> --format json [--output <path>]
  highergraphen test-semantics verify --interpretation <path> --review <path> --format json [--test-run <path>] [--output <path>]
  highergraphen test-semantics gap --expected <path> --verified <path> [--verified <path> ...] --format json [--output <path>]
  highergraphen rust-test semantics from-path --path <path> [--path <path> ...] --format json [--repo <path>] [--test-run <path>] [--output <path>]
  highergraphen semantic-proof backend run --backend <name> --backend-version <version> --command <path> [--arg <text> ...] [--input <path>] --format json [--output <path>]
  highergraphen semantic-proof input from-artifact --artifact <path> --backend <name> --backend-version <version> --theorem-id <id> --theorem-summary <text> --law-id <id> --law-summary <text> --morphism-id <id> --morphism-type <text> --base-cell <id> --base-label <text> --head-cell <id> --head-label <text> --format json [--output <path>]
  highergraphen semantic-proof input from-report --report <path> --format json [--output <path>]
  highergraphen semantic-proof input attach-artifact --input <path> --artifact <path> --backend <name> --backend-version <version> --format json [--output <path>]
  highergraphen semantic-proof verify --input <path> --format json [--output <path>]
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
    if matches!(&command, Command::Version) {
        println!("highergraphen {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

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
