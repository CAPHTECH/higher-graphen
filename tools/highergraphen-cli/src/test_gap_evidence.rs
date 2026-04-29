use higher_graphen_core::{Confidence, Id, Severity};
use higher_graphen_runtime::{
    TestGapContextType, TestGapEvidenceType, TestGapHigherOrderCell, TestGapHigherOrderIncidence,
    TestGapInputContext, TestGapInputDocument, TestGapInputEvidence, TestGapInputRiskSignal,
    TestGapRiskSignalType, TestGapVerificationCell,
};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

pub(crate) struct TestRunEvidenceRequest {
    pub(crate) input: TestGapInputDocument,
    pub(crate) test_run: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TestRunStatus {
    Passed,
    Failed,
    Ignored,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestRunCase {
    name: String,
    status: TestRunStatus,
}

pub(crate) fn input_from_test_run(
    request: TestRunEvidenceRequest,
) -> Result<TestGapInputDocument, String> {
    let text = fs::read_to_string(&request.test_run).map_err(|error| {
        format!(
            "failed to read test run {}: {error}",
            request.test_run.display()
        )
    })?;
    let cases = parse_test_run_cases(&text)?;
    if cases.is_empty() {
        return Err("test run evidence contains no test cases".to_owned());
    }
    augment_with_test_run(request.input, &text, &cases)
}

fn augment_with_test_run(
    mut input: TestGapInputDocument,
    text: &str,
    cases: &[TestRunCase],
) -> Result<TestGapInputDocument, String> {
    push_unique_string(&mut input.source.adapters, "test-run-evidence.v1");

    let hash = stable_hex_hash(text);
    let evidence_id = id(format!("evidence:test-run:{hash}"))?;
    let context_id = id("context:test-run")?;
    push_context(&mut input, context_id.clone(), &evidence_id);
    input.evidence.push(TestGapInputEvidence {
        id: evidence_id.clone(),
        evidence_type: TestGapEvidenceType::TestResult,
        summary: format!(
            "Test run artifact {} contains {} passed, {} failed, and {} ignored tests.",
            hash,
            cases
                .iter()
                .filter(|case| case.status == TestRunStatus::Passed)
                .count(),
            cases
                .iter()
                .filter(|case| case.status == TestRunStatus::Failed)
                .count(),
            cases
                .iter()
                .filter(|case| case.status == TestRunStatus::Ignored)
                .count()
        ),
        source_ids: Vec::new(),
        confidence: Some(confidence(0.95)?),
    });

    let run_cell_id = id(format!("semantic:test-run:artifact:{hash}"))?;
    push_cell(
        &mut input,
        run_cell_id.clone(),
        "test_run_artifact",
        format!("Test run artifact {hash}"),
        vec![context_id.clone()],
        vec![evidence_id.clone()],
        0.95,
    )?;

    for case in cases {
        let case_slug = slug(&case.name);
        let case_id = id(format!("semantic:test-run:case:{case_slug}"))?;
        push_cell(
            &mut input,
            case_id.clone(),
            match case.status {
                TestRunStatus::Passed => "test_execution_passed",
                TestRunStatus::Failed => "test_execution_failed",
                TestRunStatus::Ignored => "test_execution_ignored",
            },
            format!(
                "Executed test {} ({})",
                case.name,
                test_status_label(&case.status)
            ),
            vec![context_id.clone()],
            vec![evidence_id.clone()],
            0.9,
        )?;
        push_incidence(
            &mut input,
            format!("incidence:test-run:artifact-contains-case:{hash}:{case_slug}"),
            run_cell_id.clone(),
            case_id.clone(),
            "contains_test_case",
            vec![evidence_id.clone()],
            0.9,
        )?;

        let function_ids = matching_test_function_ids(&input, &case.name);
        for function_id in function_ids {
            push_incidence(
                &mut input,
                format!(
                    "incidence:test-run:case-executes-function:{case_slug}:{}",
                    slug(function_id.as_str())
                ),
                case_id.clone(),
                function_id.clone(),
                "executes_test_function",
                vec![evidence_id.clone()],
                0.88,
            )?;
            if case.status == TestRunStatus::Passed {
                push_executed_verification_cells(
                    &mut input,
                    &case_slug,
                    &case_id,
                    &function_id,
                    &evidence_id,
                )?;
            }
        }

        if case.status == TestRunStatus::Failed {
            input.signals.push(TestGapInputRiskSignal {
                id: id(format!("signal:test-run:failed:{case_slug}"))?,
                signal_type: TestGapRiskSignalType::TestGap,
                summary: format!("Test run reports failed test {}.", case.name),
                source_ids: vec![evidence_id.clone(), case_id],
                severity: Severity::High,
                confidence: confidence(0.92)?,
            });
        }
    }

    Ok(input)
}

fn parse_test_run_cases(text: &str) -> Result<Vec<TestRunCase>, String> {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        let cases = parse_json_test_cases(&value);
        if !cases.is_empty() {
            return Ok(cases);
        }
    }

    let jsonl_cases = text
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .flat_map(|value| parse_json_test_cases(&value))
        .collect::<Vec<_>>();
    if !jsonl_cases.is_empty() {
        return Ok(jsonl_cases);
    }

    Ok(parse_plain_test_cases(text))
}

fn parse_json_test_cases(value: &Value) -> Vec<TestRunCase> {
    if let Some(cases) = value.get("tests").and_then(Value::as_array) {
        return cases.iter().filter_map(test_case_from_json).collect();
    }
    if let Some(cases) = value.as_array() {
        return cases.iter().filter_map(test_case_from_json).collect();
    }
    test_case_from_json(value).into_iter().collect()
}

fn test_case_from_json(value: &Value) -> Option<TestRunCase> {
    let name = value
        .get("name")
        .or_else(|| value.get("test"))
        .and_then(Value::as_str)?
        .trim();
    if name.is_empty() {
        return None;
    }
    let status = value
        .get("status")
        .or_else(|| value.get("event"))
        .or_else(|| value.get("result"))
        .and_then(Value::as_str)
        .and_then(parse_status)?;
    Some(TestRunCase {
        name: name.to_owned(),
        status,
    })
}

fn parse_plain_test_cases(text: &str) -> Vec<TestRunCase> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let body = trimmed.strip_prefix("test ")?;
            let (name, status_text) = body.rsplit_once(" ... ")?;
            Some(TestRunCase {
                name: name.trim().to_owned(),
                status: parse_status(status_text.trim())?,
            })
        })
        .collect()
}

fn parse_status(value: &str) -> Option<TestRunStatus> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ok" | "passed" | "pass" | "success" | "succeeded" => Some(TestRunStatus::Passed),
        "failed" | "fail" | "failure" | "error" => Some(TestRunStatus::Failed),
        "ignored" | "ignore" | "skipped" | "skip" => Some(TestRunStatus::Ignored),
        _ => None,
    }
}

fn matching_test_function_ids(input: &TestGapInputDocument, test_name: &str) -> Vec<Id> {
    let candidates = test_name_candidates(test_name);
    input
        .higher_order_cells
        .iter()
        .filter(|cell| cell.cell_type == "rust_test_function")
        .filter(|cell| {
            let cell_id = cell.id.as_str();
            candidates
                .iter()
                .any(|candidate| cell_id.ends_with(&format!(":{candidate}")))
        })
        .map(|cell| cell.id.clone())
        .collect()
}

fn test_name_candidates(test_name: &str) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    candidates.insert(slug(test_name));
    for segment in test_name.split("::") {
        candidates.insert(slug(segment));
    }
    if let Some(last) = test_name.split("::").last() {
        candidates.insert(slug(last));
    }
    candidates
}

fn push_executed_verification_cells(
    input: &mut TestGapInputDocument,
    case_slug: &str,
    case_id: &Id,
    function_id: &Id,
    evidence_id: &Id,
) -> Result<(), String> {
    let Some(file_slug) = rust_test_function_file_slug(function_id) else {
        return Ok(());
    };
    let test_ids = input
        .tests
        .iter()
        .filter(|test| {
            test.file_id
                .as_ref()
                .is_some_and(|file_id| file_id.as_str() == format!("file:{file_slug}"))
        })
        .map(|test| test.id.clone())
        .collect::<Vec<_>>();
    if test_ids.is_empty() {
        return Ok(());
    }

    let templates = input
        .verification_cells
        .iter()
        .filter(|verification| {
            test_ids
                .iter()
                .any(|test_id| verification.source_ids.contains(test_id))
        })
        .cloned()
        .collect::<Vec<_>>();
    for template in templates {
        let verification_id = id(format!(
            "verification:test-run:{case_slug}:{}",
            slug(template.id.as_str())
        ))?;
        if input
            .verification_cells
            .iter()
            .any(|verification| verification.id == verification_id)
        {
            continue;
        }
        let mut source_ids = template.source_ids.clone();
        push_unique_id(&mut source_ids, evidence_id.clone());
        push_unique_id(&mut source_ids, case_id.clone());
        push_unique_id(&mut source_ids, function_id.clone());
        input.verification_cells.push(TestGapVerificationCell {
            id: verification_id,
            name: format!("Executed {}", template.name),
            verification_type: "executed_automated_test".to_owned(),
            test_type: template.test_type,
            target_ids: template.target_ids,
            requirement_ids: template.requirement_ids,
            law_ids: template.law_ids,
            morphism_ids: template.morphism_ids,
            source_ids,
            confidence: Some(confidence(0.9)?),
        });
    }
    Ok(())
}

fn rust_test_function_file_slug(function_id: &Id) -> Option<&str> {
    let mut parts = function_id.as_str().split(':');
    match (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) {
        (Some("semantic"), Some("rust-test"), Some("function"), Some(file_slug), Some(_)) => {
            Some(file_slug)
        }
        _ => None,
    }
}

fn push_context(input: &mut TestGapInputDocument, context_id: Id, evidence_id: &Id) {
    if input
        .contexts
        .iter()
        .any(|context| context.id == context_id)
    {
        return;
    }
    input.contexts.push(TestGapInputContext {
        id: context_id,
        name: "Test Run".to_owned(),
        context_type: TestGapContextType::TestScope,
        source_ids: vec![evidence_id.clone()],
    });
}

fn push_cell(
    input: &mut TestGapInputDocument,
    cell_id: Id,
    cell_type: &str,
    label: String,
    context_ids: Vec<Id>,
    source_ids: Vec<Id>,
    confidence_value: f64,
) -> Result<(), String> {
    if input
        .higher_order_cells
        .iter()
        .any(|cell| cell.id == cell_id)
    {
        return Ok(());
    }
    input.higher_order_cells.push(TestGapHigherOrderCell {
        id: cell_id,
        cell_type: cell_type.to_owned(),
        label,
        dimension: 0,
        context_ids,
        source_ids,
        confidence: Some(confidence(confidence_value)?),
    });
    Ok(())
}

fn push_incidence(
    input: &mut TestGapInputDocument,
    incidence_id: String,
    from_id: Id,
    to_id: Id,
    relation_type: &str,
    source_ids: Vec<Id>,
    confidence_value: f64,
) -> Result<(), String> {
    let incidence_id = id(incidence_id)?;
    if input
        .higher_order_incidences
        .iter()
        .any(|incidence| incidence.id == incidence_id)
    {
        return Ok(());
    }
    input
        .higher_order_incidences
        .push(TestGapHigherOrderIncidence {
            id: incidence_id,
            from_id,
            to_id,
            relation_type: relation_type.to_owned(),
            orientation: None,
            source_ids,
            confidence: Some(confidence(confidence_value)?),
        });
    Ok(())
}

fn test_status_label(status: &TestRunStatus) -> &'static str {
    match status {
        TestRunStatus::Passed => "passed",
        TestRunStatus::Failed => "failed",
        TestRunStatus::Ignored => "ignored",
    }
}

fn push_unique_id(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn push_unique_string(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}

fn id(value: impl Into<String>) -> Result<Id, String> {
    Id::new(value).map_err(|error| error.to_string())
}

fn confidence(value: f64) -> Result<Confidence, String> {
    Confidence::new(value).map_err(|error| error.to_string())
}

fn stable_hex_hash(text: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "item".to_owned()
    } else {
        slug.to_owned()
    }
}
