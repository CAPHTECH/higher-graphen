use crate::common::*;
use serde_json::{json, Value};
use std::fs;

#[test]
fn semantic_proof_verify_reads_fixture_and_writes_one_json_report_to_stdout() {
    let fixture = semantic_proof_fixture();
    let output = run_cli(&[
        "semantic-proof",
        "verify",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let stdout = stdout(&output);
    assert_eq!(stdout.lines().count(), 1);
    let value: Value = serde_json::from_str(stdout.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(SEMANTIC_PROOF_REPORT_SCHEMA));
    assert_eq!(value["result"]["status"], json!("proved"));
    assert!(value["result"]["proof_objects"]
        .as_array()
        .expect("proof objects")
        .iter()
        .any(|proof| proof["certificate_ids"]
            .as_array()
            .is_some_and(
                |certificate_ids| certificate_ids.contains(&json!("certificate:semantic:pricing"))
            )));
}

#[test]
fn semantic_proof_input_from_artifact_emits_proved_input_and_verifies() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let artifact = directory.join("kani-artifact.json");
    fs::write(
        &artifact,
        json!({
            "status": "proved",
            "input_hash": "sha256:input",
            "proof_hash": "sha256:proof",
            "witness_ids": [
                "cell:semantic:pricing:base",
                "cell:semantic:pricing:head"
            ],
            "review_status": "accepted",
            "confidence": 0.91
        })
        .to_string(),
    )
    .expect("write semantic proof artifact");

    let output = run_cli_owned(&semantic_artifact_command(&artifact, None));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());
    let generated_input = stdout(&output);
    assert_eq!(generated_input.lines().count(), 1);
    let value: Value =
        serde_json::from_str(generated_input.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(SEMANTIC_PROOF_INPUT_SCHEMA));
    assert_eq!(value["source"]["kind"], json!("code"));
    assert!(value["source"]["adapters"]
        .as_array()
        .expect("source adapters")
        .contains(&json!("semantic-proof-from-artifact.v1")));
    assert_eq!(
        value["verification_policy"]["accepted_backends"],
        json!(["kani"])
    );
    assert_eq!(value["proof_certificates"][0]["backend"], json!("kani"));

    let input = directory.join("semantic-proof.input.json");
    fs::write(&input, generated_input).expect("write generated semantic proof input");
    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        input
            .to_str()
            .expect("input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("proved"));
    assert!(report["result"]["accepted_certificate_ids"]
        .as_array()
        .expect("accepted certificate ids")
        .iter()
        .any(|id| id == &json!("certificate:semantic:kani:theorem-semantic-pricing")));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_artifact_emits_counterexample_input_and_verifies() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let artifact = directory.join("smt-artifact.json");
    fs::write(
        &artifact,
        json!({
            "status": "counterexample",
            "path_ids": [
                "cell:semantic:pricing:base",
                "cell:semantic:pricing:head"
            ],
            "summary": "symbolic execution found a mismatch",
            "severity": "critical",
            "review_status": "accepted",
            "confidence": 0.93
        })
        .to_string(),
    )
    .expect("write semantic proof artifact");

    let input = directory.join("semantic-proof.input.json");
    let output = run_cli_owned(&semantic_artifact_command(&artifact, Some(&input)));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());
    let value: Value = serde_json::from_str(
        &fs::read_to_string(&input).expect("read generated semantic proof input"),
    )
    .expect("generated input should be JSON");
    assert_eq!(value["schema"], json!(SEMANTIC_PROOF_INPUT_SCHEMA));
    assert_eq!(value["counterexamples"][0]["severity"], json!("critical"));

    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        input
            .to_str()
            .expect("input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("counterexample_found"));
    assert_eq!(
        report["result"]["counterexamples"][0]["id"],
        json!("counterexample:semantic:kani:theorem-semantic-pricing")
    );

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_artifact_rejects_invalid_artifacts() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");

    for (name, artifact, expected_error) in [
        (
            "missing-status",
            json!({
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof"
            }),
            "missing status",
        ),
        (
            "unknown-status",
            json!({
                "status": "unknown",
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof"
            }),
            "unsupported semantic proof artifact status",
        ),
        (
            "bad-confidence",
            json!({
                "status": "proved",
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof",
                "confidence": 1.1
            }),
            "confidence must be between 0.0 and 1.0 inclusive",
        ),
        (
            "bad-witness-ids",
            json!({
                "status": "proved",
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof",
                "witness_ids": "not-an-array"
            }),
            "witness_ids must be an array of strings",
        ),
        (
            "bad-path-ids-entry",
            json!({
                "status": "counterexample",
                "path_ids": ["cell:semantic:pricing:base", 1]
            }),
            "path_ids entries must be strings",
        ),
    ] {
        let artifact_path = directory.join(format!("{name}.json"));
        fs::write(&artifact_path, artifact.to_string()).expect("write invalid artifact");

        let output = run_cli_owned(&semantic_artifact_command(&artifact_path, None));

        assert!(
            !output.status.success(),
            "{name} unexpectedly succeeded with stdout: {}",
            stdout(&output)
        );
        assert!(stdout(&output).is_empty());
        assert!(
            stderr(&output).contains(expected_error),
            "{name} stderr did not contain {expected_error:?}: {}",
            stderr(&output)
        );
    }

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_artifact_rejected_or_unhashed_certificates_are_insufficient() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");

    for (name, artifact) in [
        (
            "rejected-review",
            json!({
                "status": "proved",
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof",
                "review_status": "rejected",
                "confidence": 0.91
            }),
        ),
        (
            "missing-input-hash",
            json!({
                "status": "proved",
                "proof_hash": "sha256:proof",
                "review_status": "accepted",
                "confidence": 0.91
            }),
        ),
        (
            "missing-proof-hash",
            json!({
                "status": "proved",
                "input_hash": "sha256:input",
                "review_status": "accepted",
                "confidence": 0.91
            }),
        ),
    ] {
        let artifact_path = directory.join(format!("{name}.json"));
        let input_path = directory.join(format!("{name}.input.json"));
        fs::write(&artifact_path, artifact.to_string()).expect("write policy artifact");

        let input_output = run_cli_owned(&semantic_artifact_command(
            &artifact_path,
            Some(&input_path),
        ));
        assert!(
            input_output.status.success(),
            "{name} input stderr: {}",
            stderr(&input_output)
        );
        assert!(stdout(&input_output).is_empty());

        let verify = run_cli_owned(&[
            "semantic-proof".to_owned(),
            "verify".to_owned(),
            "--input".to_owned(),
            input_path
                .to_str()
                .expect("input path should be utf-8")
                .to_owned(),
            "--format".to_owned(),
            "json".to_owned(),
        ]);
        assert!(
            verify.status.success(),
            "{name} stderr: {}",
            stderr(&verify)
        );
        let report: Value =
            serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
        assert_eq!(
            report["result"]["status"],
            json!("insufficient_proof"),
            "{name} should not prove the theorem"
        );
        assert!(report["result"]["proof_objects"]
            .as_array()
            .map(|proof_objects| proof_objects.is_empty())
            .unwrap_or(true));
    }

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_artifact_preserves_counterexample_found_defaults() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let artifact = directory.join("counterexample-found-artifact.json");
    fs::write(
        &artifact,
        json!({
            "status": "counterexample_found",
            "confidence": 0.88
        })
        .to_string(),
    )
    .expect("write semantic proof artifact");

    let output = run_cli_owned(&semantic_artifact_command(&artifact, None));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());
    let generated_input = stdout(&output);
    let value: Value =
        serde_json::from_str(generated_input.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["counterexamples"][0]["severity"], json!("high"));
    assert_eq!(
        value["counterexamples"][0]["path_ids"],
        json!(["cell:semantic:pricing:base", "cell:semantic:pricing:head"])
    );
    assert_eq!(
        value["counterexamples"][0]["summary"],
        json!("Backend artifact supplied a counterexample.")
    );
    assert_eq!(
        value["counterexamples"][0]["review_status"],
        json!("accepted")
    );

    let input = directory.join("semantic-proof.input.json");
    fs::write(&input, generated_input).expect("write generated semantic proof input");
    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        input
            .to_str()
            .expect("input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("counterexample_found"));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_backend_run_emits_trusted_proof_artifact_and_verifies() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let backend_input = directory.join("backend-input.txt");
    fs::write(&backend_input, "semantic obligation").expect("write backend input");
    let artifact = directory.join("backend-artifact.json");

    let output = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "backend".to_owned(),
        "run".to_owned(),
        "--backend".to_owned(),
        "kani".to_owned(),
        "--backend-version".to_owned(),
        "1.0.0".to_owned(),
        "--command".to_owned(),
        "/bin/sh".to_owned(),
        "--arg".to_owned(),
        "-c".to_owned(),
        "--arg".to_owned(),
        "printf proof-ok".to_owned(),
        "--input".to_owned(),
        backend_input
            .to_str()
            .expect("backend input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
        "--output".to_owned(),
        artifact
            .to_str()
            .expect("artifact path should be utf-8")
            .to_owned(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    let artifact_value: Value =
        serde_json::from_str(&fs::read_to_string(&artifact).expect("read backend artifact"))
            .expect("backend artifact should be JSON");
    assert_eq!(
        artifact_value["schema"],
        json!("highergraphen.semantic_proof.backend_artifact.v1")
    );
    assert_eq!(artifact_value["status"], json!("proved"));
    assert_eq!(artifact_value["review_status"], json!("accepted"));
    assert_eq!(
        artifact_value["backend_run"]["trust_boundary"],
        json!("local_process_output_untrusted_until_semantic_proof_verify_and_review")
    );
    assert!(artifact_value["input_hash"]
        .as_str()
        .expect("input hash")
        .starts_with("fnv64:"));

    let input = directory.join("semantic-proof.input.json");
    let generated = run_cli_owned(&semantic_artifact_command(&artifact, Some(&input)));
    assert!(generated.status.success(), "stderr: {}", stderr(&generated));

    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        input
            .to_str()
            .expect("input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("proved"));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_backend_counterexample_stays_unreviewed_at_trust_boundary() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let artifact = directory.join("backend-counterexample.json");

    let output = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "backend".to_owned(),
        "run".to_owned(),
        "--backend".to_owned(),
        "kani".to_owned(),
        "--backend-version".to_owned(),
        "1.0.0".to_owned(),
        "--command".to_owned(),
        "/bin/sh".to_owned(),
        "--arg".to_owned(),
        "-c".to_owned(),
        "--arg".to_owned(),
        "printf counterexample; exit 1".to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
        "--output".to_owned(),
        artifact
            .to_str()
            .expect("artifact path should be utf-8")
            .to_owned(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let artifact_value: Value =
        serde_json::from_str(&fs::read_to_string(&artifact).expect("read backend artifact"))
            .expect("backend artifact should be JSON");
    assert_eq!(artifact_value["status"], json!("counterexample_found"));
    assert_eq!(artifact_value["review_status"], json!("unreviewed"));

    let input = directory.join("semantic-proof.input.json");
    let generated = run_cli_owned(&semantic_artifact_command(&artifact, Some(&input)));
    assert!(generated.status.success(), "stderr: {}", stderr(&generated));

    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        input
            .to_str()
            .expect("input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("insufficient_proof"));
    assert!(report["result"]["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .any(|issue| issue["issue_type"] == json!("counterexample_not_accepted_by_policy")));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_report_requeues_unproved_obligations() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let artifact = directory.join("missing-proof-hash.json");
    fs::write(
        &artifact,
        json!({
            "status": "proved",
            "input_hash": "fnv64:input",
            "review_status": "accepted",
            "confidence": 0.91
        })
        .to_string(),
    )
    .expect("write insufficient artifact");

    let input = directory.join("semantic-proof.input.json");
    let generated = run_cli_owned(&semantic_artifact_command(&artifact, Some(&input)));
    assert!(generated.status.success(), "stderr: {}", stderr(&generated));

    let report_path = directory.join("semantic-proof.report.json");
    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        input
            .to_str()
            .expect("input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
        "--output".to_owned(),
        report_path
            .to_str()
            .expect("report path should be utf-8")
            .to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));

    let reinput = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "input".to_owned(),
        "from-report".to_owned(),
        "--report".to_owned(),
        report_path
            .to_str()
            .expect("report path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(reinput.status.success(), "stderr: {}", stderr(&reinput));
    let value: Value =
        serde_json::from_str(stdout(&reinput).trim_end()).expect("reinput stdout should be JSON");
    assert_eq!(value["schema"], json!(SEMANTIC_PROOF_INPUT_SCHEMA));
    assert!(value["source"]["adapters"]
        .as_array()
        .expect("adapters")
        .contains(&json!("semantic-proof-reinput-from-report.v1")));
    assert!(value["proof_certificates"]
        .as_array()
        .expect("proof certs")
        .is_empty());
    assert_eq!(
        value["theorem"]["morphism_ids"],
        json!(["morphism:semantic:pricing-signature"])
    );

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_test_semantics_verification_report_feeds_verify() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let proof_input_path = directory.join("semantic-proof.from-test-semantics.input.json");
    let verification_report = test_semantics_verification_report_fixture();

    let generated = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "input".to_owned(),
        "from-report".to_owned(),
        "--report".to_owned(),
        verification_report
            .to_str()
            .expect("verification report path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
        "--output".to_owned(),
        proof_input_path
            .to_str()
            .expect("proof input path should be utf-8")
            .to_owned(),
    ]);
    assert!(generated.status.success(), "stderr: {}", stderr(&generated));
    assert!(stdout(&generated).is_empty());

    let value: Value = serde_json::from_str(
        &fs::read_to_string(&proof_input_path).expect("read generated proof input"),
    )
    .expect("proof input should be JSON");
    assert_eq!(value["schema"], json!(SEMANTIC_PROOF_INPUT_SCHEMA));
    assert!(value["source"]["adapters"]
        .as_array()
        .expect("adapters")
        .contains(&json!(
            "test-semantics-verification-to-semantic-proof-input.v1"
        )));
    assert_eq!(
        value["theorem"]["law_ids"],
        json!(["candidate-law:schema-identity-preservation:schema-acme-audit-input-v1"])
    );
    assert!(!value["theorem"]["morphism_ids"]
        .as_array()
        .expect("morphism ids")
        .is_empty());
    assert!(value["proof_certificates"]
        .as_array()
        .expect("proof certificates")
        .is_empty());

    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        proof_input_path
            .to_str()
            .expect("proof input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["schema"], json!(SEMANTIC_PROOF_REPORT_SCHEMA));
    assert_eq!(report["result"]["status"], json!("insufficient_proof"));
    assert!(report["result"]["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .any(|issue| issue["issue_type"] == json!("missing_law_proof")));
    assert!(report["result"]["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .any(|issue| issue["issue_type"] == json!("missing_morphism_proof")));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_attach_artifact_proves_test_semantics_obligation() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let base_input_path = directory.join("semantic-proof.from-test-semantics.input.json");
    let proved_input_path = directory.join("semantic-proof.with-artifact.input.json");
    let artifact_path = directory.join("proof-artifact.json");
    let verification_report = test_semantics_verification_report_fixture();

    let generated = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "input".to_owned(),
        "from-report".to_owned(),
        "--report".to_owned(),
        verification_report
            .to_str()
            .expect("verification report path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
        "--output".to_owned(),
        base_input_path
            .to_str()
            .expect("proof input path should be utf-8")
            .to_owned(),
    ]);
    assert!(generated.status.success(), "stderr: {}", stderr(&generated));

    fs::write(
        &artifact_path,
        json!({
            "status": "proved",
            "input_hash": "fnv64:test-semantics-input",
            "proof_hash": "fnv64:test-semantics-proof",
            "review_status": "accepted",
            "confidence": 0.93
        })
        .to_string(),
    )
    .expect("write proof artifact");

    let attached = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "input".to_owned(),
        "attach-artifact".to_owned(),
        "--input".to_owned(),
        base_input_path
            .to_str()
            .expect("base input path should be utf-8")
            .to_owned(),
        "--artifact".to_owned(),
        artifact_path
            .to_str()
            .expect("artifact path should be utf-8")
            .to_owned(),
        "--backend".to_owned(),
        "kani".to_owned(),
        "--backend-version".to_owned(),
        "1.0.0".to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
        "--output".to_owned(),
        proved_input_path
            .to_str()
            .expect("proved input path should be utf-8")
            .to_owned(),
    ]);
    assert!(attached.status.success(), "stderr: {}", stderr(&attached));
    assert!(stdout(&attached).is_empty());

    let value: Value = serde_json::from_str(
        &fs::read_to_string(&proved_input_path).expect("read attached proof input"),
    )
    .expect("attached proof input should be JSON");
    assert!(value["source"]["adapters"]
        .as_array()
        .expect("adapters")
        .contains(&json!("semantic-proof-attach-artifact.v1")));
    assert_eq!(
        value["proof_certificates"][0]["law_ids"],
        value["theorem"]["law_ids"]
    );
    assert_eq!(
        value["proof_certificates"][0]["morphism_ids"],
        value["theorem"]["morphism_ids"]
    );

    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        proved_input_path
            .to_str()
            .expect("proved input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("proved"));
    assert!(!report["result"]["proof_objects"]
        .as_array()
        .expect("proof objects")
        .is_empty());
    assert!(report["result"].get("issues").is_none());

    fs::remove_dir_all(directory).expect("remove temp test directory");
}
