use higher_graphen_core::{Confidence, Id, Severity, SourceKind};
use higher_graphen_runtime::{
    PrReviewTargetChangeType, PrReviewTargetContextType, PrReviewTargetDependencyRelationType,
    PrReviewTargetEvidenceType, PrReviewTargetInputChangedFile, PrReviewTargetInputContext,
    PrReviewTargetInputDependencyEdge, PrReviewTargetInputDocument, PrReviewTargetInputEvidence,
    PrReviewTargetInputOwner, PrReviewTargetInputRiskSignal, PrReviewTargetInputTest,
    PrReviewTargetOwnerType, PrReviewTargetPullRequest, PrReviewTargetRepository,
    PrReviewTargetReviewerContext, PrReviewTargetRiskSignalType, PrReviewTargetSource,
    PrReviewTargetTestType,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

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
    let range = format!("{}..{}", request.base, request.head);

    let commits = commit_summaries(&repo_path, &range)?;
    let changes = changed_files(&repo_path, &range)?;
    if changes.is_empty() {
        return Err(format!("git range {range} has no changed files"));
    }

    let repository_id = id(format!("repo:{}", slug(&repo_name)))?;
    let pull_request_id = id(format!(
        "pr:git:{}:{}..{}",
        slug(&repo_name),
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
    let signals = signals_for_changes(&changes, &dependency_edges)?;

    Ok(PrReviewTargetInputDocument {
        schema: INPUT_SCHEMA.to_owned(),
        source: PrReviewTargetSource {
            kind: SourceKind::Code,
            uri: Some(format!("git:{repo_root}:{range}")),
            title: Some(format!("Git range {range} in {repo_name}")),
            captured_at: None,
            confidence: confidence(1.0)?,
        },
        repository: PrReviewTargetRepository {
            id: repository_id,
            name: repo_name.clone(),
            uri: remote_url
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            default_branch: default_branch.or_else(|| Some(request.base.clone())),
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
        reviewer_context: Some(PrReviewTargetReviewerContext {
            required_expertise: vec![
                "git diff review".to_owned(),
                "schema compatibility".to_owned(),
                "test coverage".to_owned(),
            ],
            declared_focus: vec!["deterministic git-derived review targets".to_owned()],
            excluded_paths: vec!["target/".to_owned()],
        }),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GitChange {
    path: String,
    old_path: Option<String>,
    change_type: PrReviewTargetChangeType,
    additions: u32,
    deletions: u32,
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
) -> Result<Vec<PrReviewTargetInputRiskSignal>, String> {
    let mut signals = Vec::new();
    let file_ids = changes
        .iter()
        .map(|change| file_id(&change.path))
        .collect::<Result<Vec<_>, _>>()?;
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
            source_ids: file_ids.clone(),
            severity: if total_lines >= 1200 {
                Severity::High
            } else {
                Severity::Medium
            },
            confidence: confidence(0.82)?,
        });
    }

    let touched_owners = changes
        .iter()
        .map(|change| owner_id_for_path(&change.path))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if touched_owners.len() > 1 {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:ownership-boundary")?,
            signal_type: PrReviewTargetRiskSignalType::OwnershipBoundary,
            summary: format!(
                "Git range crosses {} ownership areas.",
                touched_owners.len()
            ),
            source_ids: file_ids.clone(),
            severity: Severity::Medium,
            confidence: confidence(0.76)?,
        });
    }

    if !dependency_edges.is_empty() {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:contract-coupling")?,
            signal_type: PrReviewTargetRiskSignalType::DependencyChange,
            summary: "Changed files span dependent CLI, runtime, and schema surfaces.".to_owned(),
            source_ids: dependency_edges
                .iter()
                .map(|edge| edge.id.clone())
                .chain(file_ids.iter().cloned())
                .collect(),
            severity: Severity::High,
            confidence: confidence(0.8)?,
        });
    }

    let schema_ids = changes
        .iter()
        .filter(|change| is_schema_path(&change.path))
        .map(|change| file_id(&change.path))
        .collect::<Result<Vec<_>, _>>()?;
    if !schema_ids.is_empty() {
        let test_ids = changes
            .iter()
            .filter(|change| is_test_path(&change.path))
            .map(|change| id(format!("test:{}", slug(&change.path))))
            .collect::<Result<Vec<_>, _>>()?;
        let mut source_ids = schema_ids;
        source_ids.extend(test_ids);
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:schema-validation-coverage")?,
            signal_type: PrReviewTargetRiskSignalType::TestGap,
            summary: "Schema changes should stay aligned with generated fixtures and validation coverage."
                .to_owned(),
            source_ids,
            severity: Severity::Medium,
            confidence: confidence(0.78)?,
        });
    }

    let docs_ids = changes
        .iter()
        .filter(|change| is_docs_or_skill_path(&change.path))
        .map(|change| file_id(&change.path))
        .collect::<Result<Vec<_>, _>>()?;
    if !docs_ids.is_empty() {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:agent-guidance-boundary")?,
            signal_type: PrReviewTargetRiskSignalType::OwnershipBoundary,
            summary: "Documentation and agent skills must preserve the review-suggestion boundary."
                .to_owned(),
            source_ids: docs_ids,
            severity: Severity::Medium,
            confidence: confidence(0.72)?,
        });
    }

    let security_ids = changes
        .iter()
        .filter(|change| is_security_path(&change.path))
        .map(|change| file_id(&change.path))
        .collect::<Result<Vec<_>, _>>()?;
    if !security_ids.is_empty() {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:security-sensitive-path")?,
            signal_type: PrReviewTargetRiskSignalType::SecuritySensitive,
            summary: "Git range touches security-sensitive paths.".to_owned(),
            source_ids: security_ids,
            severity: Severity::High,
            confidence: confidence(0.82)?,
        });
    }

    Ok(signals)
}

fn commit_summaries(repo: &Path, range: &str) -> Result<Vec<String>, String> {
    let output = git(repo, &["log", "--format=%h %s", range])?;
    Ok(output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

fn changed_files(repo: &Path, range: &str) -> Result<Vec<GitChange>, String> {
    let name_status = git(repo, &["diff", "--name-status", "-M", range])?;
    let numstat = git(repo, &["diff", "--numstat", range])?;
    let stats = parse_numstat(&numstat);
    name_status
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_name_status(line, &stats))
        .collect()
}

fn parse_name_status(
    line: &str,
    stats: &BTreeMap<String, (u32, u32)>,
) -> Result<GitChange, String> {
    let parts = line.split('\t').collect::<Vec<_>>();
    let status = parts.first().copied().unwrap_or_default();
    let (change_type, old_path, path) = if status.starts_with('R') {
        let old = parts
            .get(1)
            .ok_or_else(|| format!("invalid rename status line: {line}"))?;
        let new = parts
            .get(2)
            .ok_or_else(|| format!("invalid rename status line: {line}"))?;
        (
            PrReviewTargetChangeType::Renamed,
            Some((*old).to_owned()),
            (*new).to_owned(),
        )
    } else {
        let path = parts
            .get(1)
            .ok_or_else(|| format!("invalid name-status line: {line}"))?
            .to_string();
        let change_type = match status.chars().next() {
            Some('A') => PrReviewTargetChangeType::Added,
            Some('D') => PrReviewTargetChangeType::Deleted,
            Some('M') => PrReviewTargetChangeType::Modified,
            _ => PrReviewTargetChangeType::Modified,
        };
        (change_type, None, path)
    };
    let (additions, deletions) = stats.get(&path).copied().unwrap_or((0, 0));
    Ok(GitChange {
        path,
        old_path,
        change_type,
        additions,
        deletions,
    })
}

fn parse_numstat(output: &str) -> BTreeMap<String, (u32, u32)> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let additions = parse_git_count(parts.next()?);
            let deletions = parse_git_count(parts.next()?);
            let path = parts.next()?.to_owned();
            Some((path, (additions, deletions)))
        })
        .collect()
}

fn parse_git_count(value: &str) -> u32 {
    value.parse().unwrap_or(0)
}

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("git {} returned non-utf8 stdout: {error}", args.join(" ")))
}

fn optional_git(repo: &Path, args: &[&str]) -> Option<String> {
    git(repo, args).ok()
}

fn file_id(path: &str) -> Result<Id, String> {
    id(format!("file:{}", slug(path)))
}

fn id(value: impl Into<String>) -> Result<Id, String> {
    Id::new(value).map_err(|error| error.to_string())
}

fn confidence(value: f64) -> Result<Confidence, String> {
    Confidence::new(value).map_err(|error| error.to_string())
}

fn first_path_matching(changes: &[GitChange], predicate: fn(&str) -> bool) -> Option<&GitChange> {
    changes.iter().find(|change| predicate(&change.path))
}

fn owner_for_path(path: &str) -> Result<(Id, String), String> {
    if is_runtime_path(path) {
        Ok((
            id("owner:highergraphen-runtime")?,
            "HigherGraphen Runtime".to_owned(),
        ))
    } else if is_cli_path(path) {
        Ok((
            id("owner:highergraphen-cli")?,
            "HigherGraphen CLI".to_owned(),
        ))
    } else if is_schema_path(path) {
        Ok((id("owner:schema-contracts")?, "Schema Contracts".to_owned()))
    } else if is_docs_or_skill_path(path) {
        Ok((id("owner:agent-docs")?, "Agent Documentation".to_owned()))
    } else {
        Ok((id("owner:repository")?, "Repository".to_owned()))
    }
}

fn owner_id_for_path(path: &str) -> Result<Id, String> {
    owner_for_path(path).map(|(id, _)| id)
}

fn context_ids_for_path(path: &str) -> Result<Vec<Id>, String> {
    context_descriptors_for_path(path).map(|descriptors| {
        descriptors
            .into_iter()
            .map(|(context_id, _, _)| context_id)
            .collect()
    })
}

fn context_descriptors_for_path(
    path: &str,
) -> Result<Vec<(Id, String, PrReviewTargetContextType)>, String> {
    let mut contexts = vec![(
        id("context:repository")?,
        "Repository".to_owned(),
        PrReviewTargetContextType::Repository,
    )];
    if is_runtime_path(path) {
        contexts.push((
            id("context:runtime")?,
            "Runtime".to_owned(),
            PrReviewTargetContextType::ReviewFocus,
        ));
    }
    if path.contains("/workflows/") {
        contexts.push((
            id("context:workflow-logic")?,
            "Workflow Logic".to_owned(),
            PrReviewTargetContextType::ReviewFocus,
        ));
    }
    if is_cli_path(path) {
        contexts.push((
            id("context:cli")?,
            "CLI".to_owned(),
            PrReviewTargetContextType::ReviewFocus,
        ));
    }
    if is_schema_path(path) {
        contexts.push((
            id("context:schema")?,
            "Schema".to_owned(),
            PrReviewTargetContextType::ReviewFocus,
        ));
    }
    if is_report_schema_path(path) || path.ends_with("reports.rs") || path.contains("_reports.rs") {
        contexts.push((
            id("context:report-contract")?,
            "Report Contract".to_owned(),
            PrReviewTargetContextType::ReviewFocus,
        ));
    }
    if is_test_path(path) {
        contexts.push((
            id("context:test-coverage")?,
            "Test Coverage".to_owned(),
            PrReviewTargetContextType::TestScope,
        ));
    }
    if path.starts_with("docs/") {
        contexts.push((
            id("context:docs")?,
            "Documentation".to_owned(),
            PrReviewTargetContextType::ReviewFocus,
        ));
    }
    if path.starts_with("skills/") {
        contexts.push((
            id("context:agent-guidance")?,
            "Agent Guidance".to_owned(),
            PrReviewTargetContextType::ReviewFocus,
        ));
    }
    Ok(contexts)
}

fn language_for_path(path: &str) -> Option<String> {
    let extension = Path::new(path).extension()?.to_str()?;
    let language = match extension {
        "rs" => "rust",
        "json" => "json",
        "md" => "markdown",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "py" => "python",
        _ => extension,
    };
    Some(language.to_owned())
}

fn test_type_for_path(path: &str) -> PrReviewTargetTestType {
    if path.contains("tests/") {
        PrReviewTargetTestType::Integration
    } else if path.contains("smoke") {
        PrReviewTargetTestType::Smoke
    } else {
        PrReviewTargetTestType::Unknown
    }
}

fn is_runtime_path(path: &str) -> bool {
    path.starts_with("crates/higher-graphen-runtime/")
}

fn is_cli_path(path: &str) -> bool {
    path.starts_with("tools/highergraphen-cli/")
}

fn is_schema_path(path: &str) -> bool {
    path.starts_with("schemas/")
}

fn is_input_schema_path(path: &str) -> bool {
    path.starts_with("schemas/inputs/") && path.ends_with(".schema.json")
}

fn is_report_schema_path(path: &str) -> bool {
    path.starts_with("schemas/reports/") && path.ends_with(".schema.json")
}

fn is_docs_or_skill_path(path: &str) -> bool {
    path.starts_with("docs/") || path.starts_with("skills/")
}

fn is_test_path(path: &str) -> bool {
    path.contains("/tests/") || path.ends_with("_test.rs") || path.ends_with(".test.rs")
}

fn is_security_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("auth")
        || lower.contains("security")
        || lower.contains("permission")
        || lower.contains("secret")
        || lower.contains("crypto")
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
