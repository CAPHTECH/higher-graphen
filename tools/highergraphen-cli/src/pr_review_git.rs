#[path = "pr_review_git_support.rs"]
mod pr_review_git_support;

use self::pr_review_git_support::*;
use higher_graphen_core::{Id, Severity, SourceKind};
use higher_graphen_runtime::{
    PrReviewTargetChangeType, PrReviewTargetContextType, PrReviewTargetDependencyRelationType,
    PrReviewTargetEvidenceType, PrReviewTargetInputChangedFile, PrReviewTargetInputContext,
    PrReviewTargetInputDependencyEdge, PrReviewTargetInputDocument, PrReviewTargetInputEvidence,
    PrReviewTargetInputOwner, PrReviewTargetInputRiskSignal, PrReviewTargetInputTest,
    PrReviewTargetOwnerType, PrReviewTargetPullRequest, PrReviewTargetRepository,
    PrReviewTargetReviewerContext, PrReviewTargetRiskSignalType, PrReviewTargetSource,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const INPUT_SCHEMA: &str = "highergraphen.pr_review_target.input.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GitInputRequest {
    pub(crate) repo: PathBuf,
    pub(crate) base: String,
    pub(crate) head: String,
}

pub(crate) fn input_from_git(
    request: GitInputRequest,
) -> Result<PrReviewTargetInputDocument, String> {
    let metadata = GitInputMetadata::read(&request)?;
    let range = format!("{}..{}", request.base, request.head);

    let commits = commit_summaries(&metadata.repo_path, &range)?;
    let changes = changed_files(&metadata.repo_path, &range)?;
    if changes.is_empty() {
        return Err(format!("git range {range} has no changed files"));
    }
    let diff_analysis = diff_analysis(&metadata.repo_path, &range, &changes)?;

    let repository_id = id(format!("repo:{}", slug(&metadata.repo_name)))?;
    let pull_request_id = id(format!(
        "pr:git:{}:{}..{}",
        slug(&metadata.repo_name),
        slug(&request.base),
        slug(&request.head)
    ))?;
    let diff_evidence_id = id("evidence:git-diff")?;
    let commit_evidence_id = id("evidence:git-commits")?;

    let changed_files = changes
        .iter()
        .map(|change| changed_file(change, &diff_evidence_id))
        .collect::<Result<Vec<_>, _>>()?;
    let owners = owners_for_changes(&changes)?;
    let contexts = contexts_for_changes(&changes, &pull_request_id)?;
    let tests = tests_for_changes(&changes)?;
    let dependency_edges = dependency_edges_for_changes(&changes, &diff_evidence_id)?;
    let evidence =
        evidence_for_changes(&changes, &commits, &diff_evidence_id, &commit_evidence_id)?;
    let signals = signals_for_changes(
        &changes,
        &dependency_edges,
        &diff_evidence_id,
        &diff_analysis,
    )?;

    Ok(PrReviewTargetInputDocument {
        schema: INPUT_SCHEMA.to_owned(),
        source: PrReviewTargetSource {
            kind: SourceKind::Code,
            uri: Some(format!("git:{}:{range}", metadata.repo_root)),
            title: Some(format!("Git range {range} in {}", metadata.repo_name)),
            captured_at: None,
            confidence: confidence(1.0)?,
        },
        repository: PrReviewTargetRepository {
            id: repository_id,
            name: metadata.repo_name.clone(),
            uri: metadata
                .remote_url
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            default_branch: metadata
                .default_branch
                .or_else(|| Some(request.base.clone())),
        },
        pull_request: PrReviewTargetPullRequest {
            id: pull_request_id,
            number: 1,
            title: format!("Git range {range}"),
            source_branch: request.head,
            target_branch: request.base,
            author_id: None,
            uri: None,
        },
        changed_files,
        symbols: Vec::new(),
        owners,
        contexts,
        tests,
        dependency_edges,
        evidence,
        signals,
        reviewer_context: Some(reviewer_context()),
    })
}

fn reviewer_context() -> PrReviewTargetReviewerContext {
    PrReviewTargetReviewerContext {
        required_expertise: vec![
            "git diff review".to_owned(),
            "schema compatibility".to_owned(),
            "test coverage".to_owned(),
        ],
        declared_focus: vec!["deterministic git-derived review targets".to_owned()],
        excluded_paths: vec!["target/".to_owned()],
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GitInputMetadata {
    repo_root: String,
    repo_path: PathBuf,
    repo_name: String,
    remote_url: Option<String>,
    default_branch: Option<String>,
}

impl GitInputMetadata {
    fn read(request: &GitInputRequest) -> Result<Self, String> {
        let repo_root = git(&request.repo, &["rev-parse", "--show-toplevel"])?
            .trim()
            .to_owned();
        let repo_path = PathBuf::from(&repo_root);
        let repo_name = repo_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("repository")
            .to_owned();
        let remote_url = optional_git(&repo_path, &["config", "--get", "remote.origin.url"]);
        let default_branch = optional_git(
            &repo_path,
            &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
        )
        .and_then(|value| value.trim().strip_prefix("origin/").map(str::to_owned));

        Ok(Self {
            repo_root,
            repo_path,
            repo_name,
            remote_url,
            default_branch,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GitChange {
    path: String,
    old_path: Option<String>,
    change_type: PrReviewTargetChangeType,
    additions: u32,
    deletions: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct GitDiffAnalysis {
    public_api_ids: Vec<Id>,
    serde_contract_ids: Vec<Id>,
    panic_or_placeholder_ids: Vec<Id>,
    external_effect_ids: Vec<Id>,
    weakened_test_ids: Vec<Id>,
    review_boundary_ids: Vec<Id>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct GitDiffFile {
    path: String,
    added_lines: Vec<String>,
    removed_lines: Vec<String>,
}

fn changed_file(
    change: &GitChange,
    diff_evidence_id: &Id,
) -> Result<PrReviewTargetInputChangedFile, String> {
    let file_id = file_id(&change.path)?;
    Ok(PrReviewTargetInputChangedFile {
        id: file_id,
        path: change.path.clone(),
        change_type: change.change_type,
        old_path: change.old_path.clone(),
        language: language_for_path(&change.path),
        additions: change.additions,
        deletions: change.deletions,
        symbol_ids: Vec::new(),
        owner_ids: vec![owner_id_for_path(&change.path)?],
        context_ids: context_ids_for_path(&change.path)?,
        source_ids: vec![diff_evidence_id.clone()],
    })
}

fn owners_for_changes(changes: &[GitChange]) -> Result<Vec<PrReviewTargetInputOwner>, String> {
    let mut owners = BTreeMap::<Id, String>::new();
    for change in changes {
        let (id, name) = owner_for_path(&change.path)?;
        owners.insert(id, name);
    }

    Ok(owners
        .into_iter()
        .map(|(id, name)| PrReviewTargetInputOwner {
            id,
            owner_type: PrReviewTargetOwnerType::Team,
            name: Some(name),
            source_ids: Vec::new(),
        })
        .collect())
}

fn contexts_for_changes(
    changes: &[GitChange],
    pull_request_id: &Id,
) -> Result<Vec<PrReviewTargetInputContext>, String> {
    let mut contexts = BTreeMap::<Id, (String, PrReviewTargetContextType)>::new();
    contexts.insert(
        id("context:repository")?,
        (
            "Repository".to_owned(),
            PrReviewTargetContextType::Repository,
        ),
    );
    contexts.insert(
        id(format!("context:{}", slug(pull_request_id.as_str())))?,
        (
            "Git Range".to_owned(),
            PrReviewTargetContextType::PullRequest,
        ),
    );

    for change in changes {
        for (context_id, name, context_type) in context_descriptors_for_path(&change.path)? {
            contexts.insert(context_id, (name, context_type));
        }
    }

    Ok(contexts
        .into_iter()
        .map(|(id, (name, context_type))| PrReviewTargetInputContext {
            id,
            name,
            context_type,
            source_ids: Vec::new(),
        })
        .collect())
}

fn tests_for_changes(changes: &[GitChange]) -> Result<Vec<PrReviewTargetInputTest>, String> {
    changes
        .iter()
        .filter(|change| is_test_path(&change.path))
        .map(|change| {
            Ok(PrReviewTargetInputTest {
                id: id(format!("test:{}", slug(&change.path)))?,
                name: format!("Changed test file {}", change.path),
                test_type: test_type_for_path(&change.path),
                file_id: Some(file_id(&change.path)?),
                symbol_ids: Vec::new(),
                context_ids: vec![id("context:test-coverage")?],
                source_ids: vec![file_id(&change.path)?],
            })
        })
        .collect()
}

fn dependency_edges_for_changes(
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<Vec<PrReviewTargetInputDependencyEdge>, String> {
    let mut edges = Vec::new();
    let cli = first_path_matching(changes, is_cli_path);
    let runtime = first_path_matching(changes, is_runtime_path);
    let report_schema = first_path_matching(changes, is_report_schema_path);
    let input_schema = first_path_matching(changes, is_input_schema_path);

    if let (Some(cli), Some(runtime)) = (cli, runtime) {
        edges.push(PrReviewTargetInputDependencyEdge {
            id: id("dependency:cli-to-runtime")?,
            from_id: file_id(&cli.path)?,
            to_id: file_id(&runtime.path)?,
            relation_type: PrReviewTargetDependencyRelationType::Calls,
            orientation: None,
            source_ids: vec![diff_evidence_id.clone()],
            confidence: Some(confidence(0.8)?),
        });
    }

    if let (Some(runtime), Some(report_schema)) = (runtime, report_schema) {
        edges.push(PrReviewTargetInputDependencyEdge {
            id: id("dependency:runtime-to-report-schema")?,
            from_id: file_id(&runtime.path)?,
            to_id: file_id(&report_schema.path)?,
            relation_type: PrReviewTargetDependencyRelationType::DependsOn,
            orientation: None,
            source_ids: vec![diff_evidence_id.clone()],
            confidence: Some(confidence(0.75)?),
        });
    }

    if let (Some(input_schema), Some(report_schema)) = (input_schema, report_schema) {
        edges.push(PrReviewTargetInputDependencyEdge {
            id: id("dependency:input-schema-to-report-schema")?,
            from_id: file_id(&input_schema.path)?,
            to_id: file_id(&report_schema.path)?,
            relation_type: PrReviewTargetDependencyRelationType::DependsOn,
            orientation: None,
            source_ids: vec![diff_evidence_id.clone()],
            confidence: Some(confidence(0.7)?),
        });
    }

    Ok(edges)
}

fn evidence_for_changes(
    changes: &[GitChange],
    commits: &[String],
    diff_evidence_id: &Id,
    commit_evidence_id: &Id,
) -> Result<Vec<PrReviewTargetInputEvidence>, String> {
    let mut evidence = vec![PrReviewTargetInputEvidence {
        id: diff_evidence_id.clone(),
        evidence_type: PrReviewTargetEvidenceType::DiffHunk,
        summary: format!(
            "Git diff contains {} changed files with {} additions and {} deletions.",
            changes.len(),
            changes.iter().map(|change| change.additions).sum::<u32>(),
            changes.iter().map(|change| change.deletions).sum::<u32>()
        ),
        source_ids: changes
            .iter()
            .map(|change| file_id(&change.path))
            .collect::<Result<Vec<_>, _>>()?,
        confidence: Some(confidence(1.0)?),
    }];

    if !commits.is_empty() {
        evidence.push(PrReviewTargetInputEvidence {
            id: commit_evidence_id.clone(),
            evidence_type: PrReviewTargetEvidenceType::Custom,
            summary: format!(
                "Git range contains {} commits: {}",
                commits.len(),
                commits.join("; ")
            ),
            source_ids: changes
                .iter()
                .map(|change| file_id(&change.path))
                .collect::<Result<Vec<_>, _>>()?,
            confidence: Some(confidence(0.95)?),
        });
    }

    Ok(evidence)
}

fn signals_for_changes(
    changes: &[GitChange],
    dependency_edges: &[PrReviewTargetInputDependencyEdge],
    diff_evidence_id: &Id,
    diff_analysis: &GitDiffAnalysis,
) -> Result<Vec<PrReviewTargetInputRiskSignal>, String> {
    let mut signals = Vec::new();
    push_size_and_owner_signals(&mut signals, changes, diff_evidence_id)?;
    push_dependency_signal(&mut signals, changes, dependency_edges)?;
    push_path_role_signals(&mut signals, changes)?;
    push_diff_analysis_signals(&mut signals, diff_analysis)?;
    Ok(signals)
}

fn push_size_and_owner_signals(
    signals: &mut Vec<PrReviewTargetInputRiskSignal>,
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<(), String> {
    let total_lines = changes
        .iter()
        .map(|change| change.additions + change.deletions)
        .sum::<u32>();
    if changes.len() >= 6 || total_lines >= 500 {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:large-git-change")?,
            signal_type: PrReviewTargetRiskSignalType::LargeChange,
            summary: format!(
                "Git range changes {} files and {} lines.",
                changes.len(),
                total_lines
            ),
            source_ids: vec![diff_evidence_id.clone()],
            severity: large_change_severity(total_lines),
            confidence: confidence(0.82)?,
        });
    }

    if let Some(signal) = ownership_boundary_signal(changes, diff_evidence_id)? {
        signals.push(signal);
    }
    Ok(())
}

fn large_change_severity(total_lines: u32) -> Severity {
    if total_lines >= 1200 {
        Severity::High
    } else {
        Severity::Medium
    }
}

fn ownership_boundary_signal(
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<Option<PrReviewTargetInputRiskSignal>, String> {
    let touched_owners = changes
        .iter()
        .map(|change| owner_id_for_path(&change.path))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if touched_owners.len() <= 1 {
        return Ok(None);
    }
    let source_ids = representative_reviewable_file_ids_by_owner(changes)?
        .unwrap_or_else(|| vec![diff_evidence_id.clone()]);
    Ok(Some(PrReviewTargetInputRiskSignal {
        id: id("signal:ownership-boundary")?,
        signal_type: PrReviewTargetRiskSignalType::OwnershipBoundary,
        summary: format!(
            "Git range crosses {} ownership areas.",
            touched_owners.len()
        ),
        source_ids,
        severity: Severity::Medium,
        confidence: confidence(0.76)?,
    }))
}

fn push_dependency_signal(
    signals: &mut Vec<PrReviewTargetInputRiskSignal>,
    changes: &[GitChange],
    dependency_edges: &[PrReviewTargetInputDependencyEdge],
) -> Result<(), String> {
    if dependency_edges.is_empty() {
        return Ok(());
    }
    let contract_file_ids = changes
        .iter()
        .filter(|change| is_review_signal_path(&change.path))
        .filter(|change| is_contract_coupling_path(&change.path))
        .map(|change| file_id(&change.path))
        .collect::<Result<Vec<_>, _>>()?;
    signals.push(PrReviewTargetInputRiskSignal {
        id: id("signal:contract-coupling")?,
        signal_type: PrReviewTargetRiskSignalType::DependencyChange,
        summary: "Changed files span dependent CLI, runtime, and schema surfaces.".to_owned(),
        source_ids: dependency_edges
            .iter()
            .map(|edge| edge.id.clone())
            .chain(contract_file_ids)
            .collect(),
        severity: Severity::High,
        confidence: confidence(0.8)?,
    });
    Ok(())
}

fn push_path_role_signals(
    signals: &mut Vec<PrReviewTargetInputRiskSignal>,
    changes: &[GitChange],
) -> Result<(), String> {
    push_optional_signal(signals, schema_coverage_signal(changes)?);
    push_optional_signal(signals, docs_guidance_signal(changes)?);
    push_optional_signal(signals, security_path_signal(changes)?);
    Ok(())
}

fn push_optional_signal(
    signals: &mut Vec<PrReviewTargetInputRiskSignal>,
    signal: Option<PrReviewTargetInputRiskSignal>,
) {
    if let Some(signal) = signal {
        signals.push(signal);
    }
}

fn schema_coverage_signal(
    changes: &[GitChange],
) -> Result<Option<PrReviewTargetInputRiskSignal>, String> {
    let schema_ids = ids_for_changes(changes, is_schema_path)?;
    if schema_ids.is_empty() {
        return Ok(None);
    }
    let test_ids = changes
        .iter()
        .filter(|change| is_test_path(&change.path))
        .map(|change| id(format!("test:{}", slug(&change.path))))
        .collect::<Result<Vec<_>, _>>()?;
    let mut source_ids = schema_ids;
    source_ids.extend(test_ids);
    Ok(Some(PrReviewTargetInputRiskSignal {
        id: id("signal:schema-validation-coverage")?,
        signal_type: PrReviewTargetRiskSignalType::TestGap,
        summary:
            "Schema changes should stay aligned with generated fixtures and validation coverage."
                .to_owned(),
        source_ids,
        severity: Severity::Medium,
        confidence: confidence(0.78)?,
    }))
}

fn docs_guidance_signal(
    changes: &[GitChange],
) -> Result<Option<PrReviewTargetInputRiskSignal>, String> {
    let source_ids = ids_for_changes(changes, is_docs_or_skill_path)?;
    if source_ids.is_empty() {
        return Ok(None);
    }
    Ok(Some(PrReviewTargetInputRiskSignal {
        id: id("signal:agent-guidance-boundary")?,
        signal_type: PrReviewTargetRiskSignalType::OwnershipBoundary,
        summary: "Documentation and agent skills must preserve the review-suggestion boundary."
            .to_owned(),
        source_ids,
        severity: Severity::Medium,
        confidence: confidence(0.72)?,
    }))
}

fn security_path_signal(
    changes: &[GitChange],
) -> Result<Option<PrReviewTargetInputRiskSignal>, String> {
    let source_ids = ids_for_changes(changes, is_security_path)?;
    if source_ids.is_empty() {
        return Ok(None);
    }
    Ok(Some(PrReviewTargetInputRiskSignal {
        id: id("signal:security-sensitive-path")?,
        signal_type: PrReviewTargetRiskSignalType::SecuritySensitive,
        summary: "Git range touches security-sensitive paths.".to_owned(),
        source_ids,
        severity: Severity::High,
        confidence: confidence(0.82)?,
    }))
}

fn ids_for_changes(changes: &[GitChange], predicate: fn(&str) -> bool) -> Result<Vec<Id>, String> {
    changes
        .iter()
        .filter(|change| predicate(&change.path))
        .map(|change| file_id(&change.path))
        .collect()
}

fn push_diff_analysis_signals(
    signals: &mut Vec<PrReviewTargetInputRiskSignal>,
    diff_analysis: &GitDiffAnalysis,
) -> Result<(), String> {
    push_diff_signal(
        signals,
        "signal:public-api-surface-change",
        PrReviewTargetRiskSignalType::Custom,
        "Diff changes public Rust API-like declarations.",
        &diff_analysis.public_api_ids,
        Severity::High,
        0.74,
    )?;
    push_diff_signal(
        signals,
        "signal:serde-contract-change",
        PrReviewTargetRiskSignalType::DependencyChange,
        "Diff changes serde or schema-visible contract annotations.",
        &diff_analysis.serde_contract_ids,
        Severity::High,
        0.78,
    )?;
    push_diff_signal(
        signals,
        "signal:panic-placeholder-added",
        PrReviewTargetRiskSignalType::Custom,
        "Diff adds panic, unwrap/expect, or placeholder control-flow paths.",
        &diff_analysis.panic_or_placeholder_ids,
        Severity::Medium,
        0.7,
    )?;
    push_diff_signal(
        signals,
        "signal:external-effect-surface-change",
        PrReviewTargetRiskSignalType::Custom,
        "Diff adds unsafe, subprocess, filesystem, or network effect surfaces.",
        &diff_analysis.external_effect_ids,
        Severity::Medium,
        0.76,
    )?;
    push_diff_signal(
        signals,
        "signal:test-assertion-weakened",
        PrReviewTargetRiskSignalType::TestGap,
        "Diff removes test assertions or test declarations.",
        &diff_analysis.weakened_test_ids,
        Severity::Medium,
        0.68,
    )?;
    push_diff_signal(
        signals,
        "signal:ai-review-boundary-change",
        PrReviewTargetRiskSignalType::OwnershipBoundary,
        "Diff changes AI proposal, human review, or review-status boundary text.",
        &diff_analysis.review_boundary_ids,
        Severity::Medium,
        0.72,
    )
}

fn push_diff_signal(
    signals: &mut Vec<PrReviewTargetInputRiskSignal>,
    id_value: &str,
    signal_type: PrReviewTargetRiskSignalType,
    summary: &str,
    source_ids: &[Id],
    severity: Severity,
    confidence_value: f64,
) -> Result<(), String> {
    if source_ids.is_empty() {
        return Ok(());
    }
    signals.push(PrReviewTargetInputRiskSignal {
        id: id(id_value)?,
        signal_type,
        summary: summary.to_owned(),
        source_ids: source_ids.to_vec(),
        severity,
        confidence: confidence(confidence_value)?,
    });
    Ok(())
}
