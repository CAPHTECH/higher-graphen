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
    let diff_analysis = diff_analysis(&repo_path, &range, &changes)?;

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
    let ownership_scope_ids = representative_reviewable_file_ids_by_owner(changes)?
        .unwrap_or_else(|| vec![diff_evidence_id.clone()]);
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
            source_ids: ownership_scope_ids,
            severity: Severity::Medium,
            confidence: confidence(0.76)?,
        });
    }

    if !dependency_edges.is_empty() {
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

    if !diff_analysis.public_api_ids.is_empty() {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:public-api-surface-change")?,
            signal_type: PrReviewTargetRiskSignalType::Custom,
            summary: "Diff changes public Rust API-like declarations.".to_owned(),
            source_ids: diff_analysis.public_api_ids.clone(),
            severity: Severity::High,
            confidence: confidence(0.74)?,
        });
    }

    if !diff_analysis.serde_contract_ids.is_empty() {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:serde-contract-change")?,
            signal_type: PrReviewTargetRiskSignalType::DependencyChange,
            summary: "Diff changes serde or schema-visible contract annotations.".to_owned(),
            source_ids: diff_analysis.serde_contract_ids.clone(),
            severity: Severity::High,
            confidence: confidence(0.78)?,
        });
    }

    if !diff_analysis.panic_or_placeholder_ids.is_empty() {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:panic-placeholder-added")?,
            signal_type: PrReviewTargetRiskSignalType::Custom,
            summary: "Diff adds panic, unwrap/expect, or placeholder control-flow paths."
                .to_owned(),
            source_ids: diff_analysis.panic_or_placeholder_ids.clone(),
            severity: Severity::Medium,
            confidence: confidence(0.7)?,
        });
    }

    if !diff_analysis.external_effect_ids.is_empty() {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:external-effect-surface-change")?,
            signal_type: PrReviewTargetRiskSignalType::Custom,
            summary: "Diff adds unsafe, subprocess, filesystem, or network effect surfaces."
                .to_owned(),
            source_ids: diff_analysis.external_effect_ids.clone(),
            severity: Severity::Medium,
            confidence: confidence(0.76)?,
        });
    }

    if !diff_analysis.weakened_test_ids.is_empty() {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:test-assertion-weakened")?,
            signal_type: PrReviewTargetRiskSignalType::TestGap,
            summary: "Diff removes test assertions or test declarations.".to_owned(),
            source_ids: diff_analysis.weakened_test_ids.clone(),
            severity: Severity::Medium,
            confidence: confidence(0.68)?,
        });
    }

    if !diff_analysis.review_boundary_ids.is_empty() {
        signals.push(PrReviewTargetInputRiskSignal {
            id: id("signal:ai-review-boundary-change")?,
            signal_type: PrReviewTargetRiskSignalType::OwnershipBoundary,
            summary: "Diff changes AI proposal, human review, or review-status boundary text."
                .to_owned(),
            source_ids: diff_analysis.review_boundary_ids.clone(),
            severity: Severity::Medium,
            confidence: confidence(0.72)?,
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

fn diff_analysis(
    repo: &Path,
    range: &str,
    changes: &[GitChange],
) -> Result<GitDiffAnalysis, String> {
    let reviewable_paths = changes
        .iter()
        .filter(|change| is_review_signal_path(&change.path))
        .map(|change| change.path.as_str())
        .collect::<BTreeSet<_>>();
    let diff = git(repo, &["diff", "--unified=0", "--no-ext-diff", range])?;
    let files = parse_diff_files(&diff);
    let mut analysis = GitDiffAnalysis::default();

    for file in files
        .iter()
        .filter(|file| reviewable_paths.contains(file.path.as_str()))
    {
        let file_id = file_id(&file.path)?;
        if file
            .added_lines
            .iter()
            .any(|line| is_public_api_line(&file.path, line))
            || file
                .removed_lines
                .iter()
                .any(|line| is_public_api_line(&file.path, line))
        {
            push_unique(&mut analysis.public_api_ids, file_id.clone());
        }
        if file
            .added_lines
            .iter()
            .chain(file.removed_lines.iter())
            .any(|line| is_serde_contract_line(&file.path, line))
        {
            push_unique(&mut analysis.serde_contract_ids, file_id.clone());
        }
        if file
            .added_lines
            .iter()
            .any(|line| is_panic_or_placeholder_line(&file.path, line))
        {
            push_unique(&mut analysis.panic_or_placeholder_ids, file_id.clone());
        }
        if file
            .added_lines
            .iter()
            .any(|line| is_external_effect_line(&file.path, line))
        {
            push_unique(&mut analysis.external_effect_ids, file_id.clone());
        }
        if is_test_path(&file.path)
            && file
                .removed_lines
                .iter()
                .any(|line| is_test_assertion_line(line))
        {
            push_unique(&mut analysis.weakened_test_ids, file_id.clone());
        }
        if file
            .added_lines
            .iter()
            .chain(file.removed_lines.iter())
            .any(|line| is_review_boundary_line(&file.path, line))
        {
            push_unique(&mut analysis.review_boundary_ids, file_id);
        }
    }

    Ok(analysis)
}

fn parse_diff_files(diff: &str) -> Vec<GitDiffFile> {
    let mut files = Vec::<GitDiffFile>::new();
    let mut current = None::<GitDiffFile>;

    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("diff --git ") {
            if let Some(file) = current.take() {
                files.push(file);
            }
            let new_path = path
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.strip_prefix("b/"))
                .unwrap_or_default()
                .to_owned();
            current = Some(GitDiffFile {
                path: new_path,
                added_lines: Vec::new(),
                removed_lines: Vec::new(),
            });
            continue;
        }

        let Some(file) = current.as_mut() else {
            continue;
        };
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if let Some(added) = line.strip_prefix('+') {
            file.added_lines.push(added.to_owned());
        } else if let Some(removed) = line.strip_prefix('-') {
            file.removed_lines.push(removed.to_owned());
        }
    }

    if let Some(file) = current {
        files.push(file);
    }

    files
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

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
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

fn is_contract_coupling_path(path: &str) -> bool {
    is_runtime_path(path) || is_cli_path(path) || is_schema_path(path) || is_test_path(path)
}

fn is_review_signal_path(path: &str) -> bool {
    !is_evidence_only_path(path)
}

fn is_evidence_only_path(path: &str) -> bool {
    is_top_level_dot_path(path) && !is_reviewable_dot_path(path)
}

fn is_top_level_dot_path(path: &str) -> bool {
    path.starts_with('.') && !path.starts_with("../")
}

fn is_reviewable_dot_path(path: &str) -> bool {
    path.starts_with(".github/workflows/")
        || path.starts_with(".github/actions/")
        || path.starts_with(".cargo/")
        || path.starts_with(".config/")
        || path.starts_with(".vscode/extensions.json")
        || path == ".gitignore"
        || path == ".gitattributes"
        || path == ".editorconfig"
        || path == ".env.example"
        || path == ".rustfmt.toml"
        || path == ".clippy.toml"
        || path == ".prettierrc"
        || path == ".prettierrc.json"
        || path == ".prettierrc.yaml"
        || path == ".prettierrc.yml"
        || path == ".eslintrc"
        || path == ".eslintrc.js"
        || path == ".eslintrc.cjs"
        || path == ".eslintrc.json"
        || path == ".markdownlint.json"
}

fn is_public_api_line(path: &str, line: &str) -> bool {
    if !path.ends_with(".rs") {
        return false;
    }
    let trimmed = line.trim_start();
    trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("pub type ")
        || trimmed.starts_with("pub const ")
        || trimmed.starts_with("pub mod ")
        || trimmed.starts_with("pub use ")
        || trimmed.starts_with("pub(crate) fn ")
        || trimmed.starts_with("pub(crate) struct ")
        || trimmed.starts_with("pub(crate) enum ")
}

fn is_serde_contract_line(path: &str, line: &str) -> bool {
    let trimmed = line.trim();
    is_schema_contract_path(path)
        || trimmed.contains("#[serde")
        || trimmed.contains("serde(")
        || trimmed.contains("deny_unknown_fields")
        || trimmed.contains("rename_all")
        || trimmed.contains("skip_serializing_if")
}

fn is_schema_contract_path(path: &str) -> bool {
    path.ends_with(".schema.json") || path.ends_with(".example.json")
}

fn is_panic_or_placeholder_line(path: &str, line: &str) -> bool {
    if is_test_path(path) && looks_like_fixture_string(line) {
        return false;
    }
    line.contains(".unwrap(")
        || line.contains(".expect(")
        || line.contains("panic!(")
        || line.contains("todo!(")
        || line.contains("unimplemented!(")
}

fn is_external_effect_line(path: &str, line: &str) -> bool {
    if is_test_path(path) && looks_like_fixture_string(line) {
        return false;
    }
    let trimmed = line.trim();
    trimmed.contains("unsafe ")
        || trimmed.starts_with("unsafe {")
        || trimmed.contains("Command::new")
        || trimmed.contains("ProcessCommand::new")
        || trimmed.contains("fs::read")
        || trimmed.contains("fs::write")
        || trimmed.contains("fs::remove")
        || trimmed.contains("fs::create")
        || trimmed.contains("File::open")
        || trimmed.contains("File::create")
        || trimmed.contains("TcpStream")
        || trimmed.contains("UdpSocket")
        || trimmed.contains("reqwest")
        || trimmed.contains("curl")
}

fn looks_like_fixture_string(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('"')
        || trimmed.starts_with("r#\"")
        || trimmed.starts_with("r\"")
        || trimmed.contains("\\n")
}

fn is_test_assertion_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("#[test]")
        || trimmed.contains("assert!")
        || trimmed.contains("assert_eq!")
        || trimmed.contains("assert_ne!")
        || trimmed.contains(".expect(")
        || trimmed.contains("should_panic")
}

fn is_review_boundary_line(path: &str, line: &str) -> bool {
    if !(is_docs_or_skill_path(path)
        || path.contains("pr_review")
        || path.contains("review_target")
        || path.ends_with("_reports.rs"))
    {
        return false;
    }
    let lower = line.to_ascii_lowercase();
    lower.contains("review_status")
        || lower.contains("unreviewed")
        || lower.contains("accepted fact")
        || lower.contains("accepted facts")
        || lower.contains("ai proposal")
        || lower.contains("human review")
        || lower.contains("recommendation")
        || lower.contains("proposal")
}

fn representative_reviewable_file_ids_by_owner(
    changes: &[GitChange],
) -> Result<Option<Vec<Id>>, String> {
    let mut representatives = BTreeMap::<Id, Id>::new();
    for change in changes
        .iter()
        .filter(|change| is_review_signal_path(&change.path))
    {
        representatives
            .entry(owner_id_for_path(&change.path)?)
            .or_insert(file_id(&change.path)?);
    }

    if representatives.is_empty() {
        Ok(None)
    } else {
        Ok(Some(representatives.into_values().collect()))
    }
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
