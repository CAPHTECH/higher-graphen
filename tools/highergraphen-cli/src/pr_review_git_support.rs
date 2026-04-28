use super::{GitChange, GitDiffAnalysis, GitDiffFile};
use higher_graphen_core::{Confidence, Id};
use higher_graphen_runtime::{
    PrReviewTargetChangeType, PrReviewTargetContextType, PrReviewTargetTestType,
};
use higher_graphen_space::{StructuralBoundaryAnalyzer, StructuralObservation, StructuralRole};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command as ProcessCommand;

impl GitDiffFile {
    fn new(path: String) -> Self {
        Self {
            path,
            added_lines: Vec::new(),
            removed_lines: Vec::new(),
        }
    }
}

pub(super) fn commit_summaries(repo: &Path, range: &str) -> Result<Vec<String>, String> {
    let output = git(repo, &["log", "--format=%h %s", range])?;
    Ok(output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

pub(super) fn changed_files(repo: &Path, range: &str) -> Result<Vec<GitChange>, String> {
    let name_status = git(repo, &["diff", "--name-status", "-M", range])?;
    let numstat = git(repo, &["diff", "--numstat", range])?;
    let stats = parse_numstat(&numstat);
    name_status
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_name_status(line, &stats))
        .collect()
}

pub(super) fn diff_analysis(
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
        collect_diff_file_signals(file, &mut analysis)?;
    }

    Ok(analysis)
}

fn collect_diff_file_signals(
    file: &GitDiffFile,
    analysis: &mut GitDiffAnalysis,
) -> Result<(), String> {
    let file_id = file_id(&file.path)?;
    if changed_public_api(file) {
        push_unique(&mut analysis.public_api_ids, file_id.clone());
    }
    if changed_serde_contract(file) {
        push_unique(&mut analysis.serde_contract_ids, file_id.clone());
    }
    if added_panic_or_placeholder(file) {
        push_unique(&mut analysis.panic_or_placeholder_ids, file_id.clone());
    }
    if added_external_effect(file) {
        push_unique(&mut analysis.external_effect_ids, file_id.clone());
    }
    if removed_test_assertion(file) {
        push_unique(&mut analysis.weakened_test_ids, file_id.clone());
    }
    if changed_review_boundary(file) {
        push_unique(&mut analysis.review_boundary_ids, file_id.clone());
    }
    if changed_structural_boundary(file, &file_id)? {
        push_unique(&mut analysis.structural_boundary_ids, file_id);
    }
    Ok(())
}

fn changed_public_api(file: &GitDiffFile) -> bool {
    file.added_lines
        .iter()
        .chain(file.removed_lines.iter())
        .any(|line| is_public_api_line(&file.path, line))
}

fn changed_serde_contract(file: &GitDiffFile) -> bool {
    file.added_lines
        .iter()
        .chain(file.removed_lines.iter())
        .any(|line| is_serde_contract_line(&file.path, line))
}

fn added_panic_or_placeholder(file: &GitDiffFile) -> bool {
    file.added_lines
        .iter()
        .any(|line| is_panic_or_placeholder_line(&file.path, line))
}

fn added_external_effect(file: &GitDiffFile) -> bool {
    file.added_lines
        .iter()
        .any(|line| is_external_effect_line(&file.path, line))
}

fn removed_test_assertion(file: &GitDiffFile) -> bool {
    is_test_path(&file.path)
        && file
            .removed_lines
            .iter()
            .any(|line| is_test_assertion_line(line))
}

fn changed_review_boundary(file: &GitDiffFile) -> bool {
    file.added_lines
        .iter()
        .chain(file.removed_lines.iter())
        .any(|line| is_review_boundary_line(&file.path, line))
}

fn changed_structural_boundary(file: &GitDiffFile, subject_id: &Id) -> Result<bool, String> {
    let observations = structural_observations(file, subject_id)?;
    if observations.is_empty() {
        return Ok(false);
    }
    Ok(!StructuralBoundaryAnalyzer::new()
        .with_observations(observations)
        .analyze()
        .signals
        .is_empty())
}

fn structural_observations(
    file: &GitDiffFile,
    subject_id: &Id,
) -> Result<Vec<StructuralObservation>, String> {
    if !file.path.ends_with(".rs") || is_test_path(&file.path) {
        return Ok(Vec::new());
    }

    file.added_lines
        .iter()
        .chain(file.removed_lines.iter())
        .enumerate()
        .filter_map(|(index, line)| structural_role_for_line(line).map(|role| (index, role)))
        .map(|(index, role)| {
            Ok(StructuralObservation::new(
                id(format!(
                    "observation:structural:{}:{}",
                    slug(&file.path),
                    index
                ))?,
                subject_id.clone(),
                role,
            )
            .with_source(subject_id.clone()))
        })
        .collect()
}

fn parse_diff_files(diff: &str) -> Vec<GitDiffFile> {
    let mut files = Vec::<GitDiffFile>::new();
    let mut current = None::<GitDiffFile>;

    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("diff --git ") {
            if let Some(file) = current.take() {
                files.push(file);
            }
            current = Some(GitDiffFile::new(diff_path(path)));
            continue;
        }

        let Some(file) = current.as_mut() else {
            continue;
        };
        if let Some(path) = line.strip_prefix("+++ ") {
            if let Some(path) = diff_file_header_path(path) {
                file.path = path;
            }
            continue;
        }
        if line.starts_with("---") {
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

fn diff_path(path: &str) -> String {
    path.rsplit_once(" b/")
        .map(|(_, value)| value)
        .or_else(|| path.split_whitespace().nth(1)?.strip_prefix("b/"))
        .map(unquote_git_path)
        .unwrap_or_default()
}

fn diff_file_header_path(path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed == "/dev/null" {
        return None;
    }
    trimmed
        .strip_prefix("b/")
        .or_else(|| trimmed.strip_prefix("\"b/"))
        .map(unquote_git_path)
}

fn unquote_git_path(path: &str) -> String {
    path.trim_end_matches('"').to_owned()
}

fn parse_name_status(
    line: &str,
    stats: &BTreeMap<String, (u32, u32)>,
) -> Result<GitChange, String> {
    let parts = line.split('\t').collect::<Vec<_>>();
    let status = parts.first().copied().unwrap_or_default();
    let (change_type, old_path, path) = if status.starts_with('R') {
        rename_change_parts(line, &parts)?
    } else {
        non_rename_change_parts(line, &parts, status)?
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

fn rename_change_parts(
    line: &str,
    parts: &[&str],
) -> Result<(PrReviewTargetChangeType, Option<String>, String), String> {
    let old = parts
        .get(1)
        .ok_or_else(|| format!("invalid rename status line: {line}"))?;
    let new = parts
        .get(2)
        .ok_or_else(|| format!("invalid rename status line: {line}"))?;
    Ok((
        PrReviewTargetChangeType::Renamed,
        Some((*old).to_owned()),
        (*new).to_owned(),
    ))
}

fn non_rename_change_parts(
    line: &str,
    parts: &[&str],
    status: &str,
) -> Result<(PrReviewTargetChangeType, Option<String>, String), String> {
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
    Ok((change_type, None, path))
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

pub(super) fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
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

pub(super) fn optional_git(repo: &Path, args: &[&str]) -> Option<String> {
    git(repo, args).ok()
}

pub(super) fn file_id(path: &str) -> Result<Id, String> {
    id(format!("file:{}", slug(path)))
}

pub(super) fn id(value: impl Into<String>) -> Result<Id, String> {
    Id::new(value).map_err(|error| error.to_string())
}

pub(super) fn confidence(value: f64) -> Result<Confidence, String> {
    Confidence::new(value).map_err(|error| error.to_string())
}

pub(super) fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

pub(super) fn first_path_matching(
    changes: &[GitChange],
    predicate: fn(&str) -> bool,
) -> Option<&GitChange> {
    changes.iter().find(|change| predicate(&change.path))
}

pub(super) fn owner_for_path(path: &str) -> Result<(Id, String), String> {
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

pub(super) fn owner_id_for_path(path: &str) -> Result<Id, String> {
    owner_for_path(path).map(|(id, _)| id)
}

pub(super) fn context_ids_for_path(path: &str) -> Result<Vec<Id>, String> {
    context_descriptors_for_path(path).map(|descriptors| {
        descriptors
            .into_iter()
            .map(|(context_id, _, _)| context_id)
            .collect()
    })
}

pub(super) fn context_descriptors_for_path(
    path: &str,
) -> Result<Vec<(Id, String, PrReviewTargetContextType)>, String> {
    let mut contexts = base_contexts()?;
    append_path_contexts(path, &mut contexts)?;
    Ok(contexts)
}

fn base_contexts() -> Result<Vec<(Id, String, PrReviewTargetContextType)>, String> {
    Ok(vec![(
        id("context:repository")?,
        "Repository".to_owned(),
        PrReviewTargetContextType::Repository,
    )])
}

fn append_path_contexts(
    path: &str,
    contexts: &mut Vec<(Id, String, PrReviewTargetContextType)>,
) -> Result<(), String> {
    push_context_if(contexts, is_runtime_path(path), "runtime", "Runtime")?;
    push_context_if(
        contexts,
        path.contains("/workflows/"),
        "workflow-logic",
        "Workflow Logic",
    )?;
    push_context_if(contexts, is_cli_path(path), "cli", "CLI")?;
    push_context_if(contexts, is_schema_path(path), "schema", "Schema")?;
    push_context_if(
        contexts,
        is_report_contract_path(path),
        "report-contract",
        "Report Contract",
    )?;
    if is_test_path(path) {
        contexts.push((
            id("context:test-coverage")?,
            "Test Coverage".to_owned(),
            PrReviewTargetContextType::TestScope,
        ));
    }
    push_context_if(contexts, path.starts_with("docs/"), "docs", "Documentation")?;
    push_context_if(
        contexts,
        path.starts_with("skills/"),
        "agent-guidance",
        "Agent Guidance",
    )
}

fn push_context_if(
    contexts: &mut Vec<(Id, String, PrReviewTargetContextType)>,
    condition: bool,
    id_suffix: &str,
    name: &str,
) -> Result<(), String> {
    if condition {
        contexts.push((
            id(format!("context:{id_suffix}"))?,
            name.to_owned(),
            PrReviewTargetContextType::ReviewFocus,
        ));
    }
    Ok(())
}

fn is_report_contract_path(path: &str) -> bool {
    is_report_schema_path(path) || path.ends_with("reports.rs") || path.contains("_reports.rs")
}

pub(super) fn language_for_path(path: &str) -> Option<String> {
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

pub(super) fn test_type_for_path(path: &str) -> PrReviewTargetTestType {
    if path.contains("tests/") {
        PrReviewTargetTestType::Integration
    } else if path.contains("smoke") {
        PrReviewTargetTestType::Smoke
    } else {
        PrReviewTargetTestType::Unknown
    }
}

pub(super) fn is_runtime_path(path: &str) -> bool {
    path.starts_with("crates/higher-graphen-runtime/")
}

pub(super) fn is_cli_path(path: &str) -> bool {
    path.starts_with("tools/highergraphen-cli/")
}

pub(super) fn is_schema_path(path: &str) -> bool {
    path.starts_with("schemas/")
}

pub(super) fn is_input_schema_path(path: &str) -> bool {
    path.starts_with("schemas/inputs/") && path.ends_with(".schema.json")
}

pub(super) fn is_report_schema_path(path: &str) -> bool {
    path.starts_with("schemas/reports/") && path.ends_with(".schema.json")
}

pub(super) fn is_docs_or_skill_path(path: &str) -> bool {
    path.starts_with("docs/") || path.starts_with("skills/")
}

pub(super) fn is_test_path(path: &str) -> bool {
    path.contains("/tests/") || path.ends_with("_test.rs") || path.ends_with(".test.rs")
}

pub(super) fn is_security_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("auth")
        || lower.contains("security")
        || lower.contains("permission")
        || lower.contains("secret")
        || lower.contains("crypto")
}

pub(super) fn is_contract_coupling_path(path: &str) -> bool {
    is_runtime_path(path) || is_cli_path(path) || is_schema_path(path) || is_test_path(path)
}

pub(super) fn is_review_signal_path(path: &str) -> bool {
    !is_evidence_only_path(path)
}

fn is_evidence_only_path(path: &str) -> bool {
    is_top_level_dot_path(path) && !is_reviewable_dot_path(path)
}

fn is_top_level_dot_path(path: &str) -> bool {
    path.starts_with('.') && !path.starts_with("../")
}

fn is_reviewable_dot_path(path: &str) -> bool {
    const EXACT: &[&str] = &[
        ".gitignore",
        ".gitattributes",
        ".editorconfig",
        ".env.example",
        ".rustfmt.toml",
        ".clippy.toml",
        ".prettierrc",
        ".prettierrc.json",
        ".prettierrc.yaml",
        ".prettierrc.yml",
        ".eslintrc",
        ".eslintrc.js",
        ".eslintrc.cjs",
        ".eslintrc.json",
        ".markdownlint.json",
    ];
    reviewable_dot_prefix(path) || EXACT.contains(&path)
}

fn reviewable_dot_prefix(path: &str) -> bool {
    path.starts_with(".github/workflows/")
        || path.starts_with(".github/actions/")
        || path.starts_with(".cargo/")
        || path.starts_with(".config/")
        || path.starts_with(".vscode/extensions.json")
}

fn is_public_api_line(path: &str, line: &str) -> bool {
    if !path.ends_with(".rs") {
        return false;
    }
    let trimmed = line.trim_start();
    const PREFIXES: &[&str] = &[
        "pub fn ",
        "pub struct ",
        "pub enum ",
        "pub trait ",
        "pub type ",
        "pub const ",
        "pub mod ",
        "pub use ",
        "pub(crate) fn ",
        "pub(crate) struct ",
        "pub(crate) enum ",
        "pub(super) fn ",
        "pub(super) struct ",
        "pub(super) enum ",
        "pub(super) type ",
        "pub(super) const ",
    ];
    PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix))
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
    external_effect_prefix(trimmed) || external_effect_token(trimmed)
}

fn external_effect_prefix(trimmed: &str) -> bool {
    trimmed.contains("unsafe ") || starts_unsafe_block(trimmed)
}

fn starts_unsafe_block(trimmed: &str) -> bool {
    trimmed.starts_with("unsafe") && trimmed.as_bytes().get(6).is_some_and(|byte| *byte == 123)
}

fn external_effect_token(trimmed: &str) -> bool {
    const TOKENS: &[&str] = &[
        "Command::new",
        "ProcessCommand::new",
        "fs::read",
        "fs::write",
        "fs::remove",
        "fs::create",
        "File::open",
        "File::create",
        "TcpStream",
        "UdpSocket",
        "reqwest",
        "curl",
    ];
    TOKENS.iter().any(|token| trimmed.contains(token))
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
    if !review_boundary_path(path) {
        return false;
    }
    let lower = line.to_ascii_lowercase();
    review_boundary_token(&lower)
}

fn review_boundary_path(path: &str) -> bool {
    is_docs_or_skill_path(path)
        || path.contains("pr_review")
        || path.contains("review_target")
        || path.ends_with("_reports.rs")
}

fn review_boundary_token(lower: &str) -> bool {
    const TOKENS: &[&str] = &[
        "review_status",
        "unreviewed",
        "accepted fact",
        "accepted facts",
        "ai proposal",
        "human review",
        "recommendation",
        "proposal",
    ];
    TOKENS.iter().any(|token| lower.contains(token))
}

fn structural_role_for_line(line: &str) -> Option<StructuralRole> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return None;
    }
    if is_module_boundary_line(trimmed) {
        return Some(StructuralRole::Boundary);
    }
    if is_dispatch_incidence_line(trimmed) {
        return Some(StructuralRole::Incidence);
    }
    if is_composition_line(trimmed) {
        return Some(StructuralRole::Composition);
    }
    None
}

fn is_module_boundary_line(trimmed: &str) -> bool {
    trimmed.starts_with("mod ")
        || trimmed.starts_with("pub mod ")
        || trimmed.starts_with("use ")
        || trimmed.starts_with("pub use ")
}

fn is_dispatch_incidence_line(trimmed: &str) -> bool {
    trimmed.contains("=>")
        && (trimmed.contains("::")
            || trimmed.contains("Some(")
            || trimmed.contains("Ok(")
            || trimmed.contains("Err("))
}

fn is_composition_line(trimmed: &str) -> bool {
    looks_like_variant_or_constructor(trimmed)
        || trimmed.contains("::parse_")
        || trimmed.contains("::run_")
        || trimmed.contains("_json(")
}

fn looks_like_variant_or_constructor(trimmed: &str) -> bool {
    let Some(first) = trimmed.chars().next() else {
        return false;
    };
    first.is_ascii_uppercase()
        && (trimmed.ends_with('{') || trimmed.ends_with(','))
        && !trimmed.starts_with("Self::")
        && !trimmed.contains("=>")
}

pub(super) fn representative_reviewable_file_ids_by_owner(
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

pub(super) fn slug(value: &str) -> String {
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
