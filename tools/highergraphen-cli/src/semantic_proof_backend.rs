//! Local proof backend runner that emits bounded semantic proof artifacts.

use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::Command};

const ARTIFACT_SCHEMA: &str = "highergraphen.semantic_proof.backend_artifact.v1";
const MAX_EXCERPT_CHARS: usize = 512;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BackendRunRequest {
    pub(crate) backend: String,
    pub(crate) backend_version: String,
    pub(crate) command: PathBuf,
    pub(crate) args: Vec<String>,
    pub(crate) input: Option<PathBuf>,
}

pub(crate) fn run_backend(request: BackendRunRequest) -> Result<Value, String> {
    let input_material = input_material(&request)?;
    let input_hash = fingerprint(&[
        command_fingerprint_material(&request).as_bytes(),
        &input_material,
    ]);
    let output = Command::new(&request.command)
        .args(&request.args)
        .output()
        .map_err(|error| {
            format!(
                "failed to run semantic proof backend command {}: {error}",
                request.command.display()
            )
        })?;

    let exit_code = output.status.code();
    let status = if output.status.success() {
        "proved"
    } else {
        "counterexample_found"
    };
    let review_status = if output.status.success() {
        "accepted"
    } else {
        "unreviewed"
    };
    let proof_hash = fingerprint(&[
        exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_owned())
            .as_bytes(),
        &output.stdout,
        &output.stderr,
    ]);

    Ok(json!({
        "schema": ARTIFACT_SCHEMA,
        "status": status,
        "input_hash": input_hash,
        "proof_hash": proof_hash,
        "review_status": review_status,
        "confidence": if output.status.success() { 0.9 } else { 0.7 },
        "summary": if output.status.success() {
            "Local proof backend exited successfully."
        } else {
            "Local proof backend did not prove the obligation; inspect backend_run before accepting any counterexample."
        },
        "backend_run": {
            "trust_boundary": "local_process_output_untrusted_until_semantic_proof_verify_and_review",
            "backend": request.backend,
            "backend_version": request.backend_version,
            "command": request.command.display().to_string(),
            "args": request.args,
            "exit_code": exit_code,
            "stdout_hash": fingerprint(&[&output.stdout]),
            "stderr_hash": fingerprint(&[&output.stderr]),
            "stdout_excerpt": excerpt(&output.stdout),
            "stderr_excerpt": excerpt(&output.stderr)
        }
    }))
}

fn input_material(request: &BackendRunRequest) -> Result<Vec<u8>, String> {
    match &request.input {
        Some(path) => fs::read(path).map_err(|error| {
            format!(
                "failed to read semantic proof backend input {}: {error}",
                path.display()
            )
        }),
        None => Ok(Vec::new()),
    }
}

fn command_fingerprint_material(request: &BackendRunRequest) -> String {
    let mut material = format!(
        "backend={}\nbackend_version={}\ncommand={}\n",
        request.backend,
        request.backend_version,
        request.command.display()
    );
    for arg in &request.args {
        material.push_str("arg=");
        material.push_str(arg);
        material.push('\n');
    }
    material
}

fn fingerprint(parts: &[&[u8]]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for part in parts {
        for byte in *part {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{hash:016x}")
}

fn excerpt(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    text.chars().take(MAX_EXCERPT_CHARS).collect()
}
