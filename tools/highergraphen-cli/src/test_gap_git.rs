#[path = "pr_review_git_support.rs"]
#[allow(dead_code)]
mod pr_review_git_support;

use self::pr_review_git_support::*;
use crate::rust_test_semantics::{contains_all_strings, extract_rust_test_semantics};
use higher_graphen_core::{Id, Severity, SourceKind};
use higher_graphen_runtime::{
    PrReviewTargetChangeType, TestGapChangeSet, TestGapChangeType, TestGapContextType,
    TestGapCoverageStatus, TestGapCoverageType, TestGapDependencyRelationType,
    TestGapDetectorContext, TestGapEvidenceType, TestGapHigherOrderCell,
    TestGapHigherOrderIncidence, TestGapInputChangedFile, TestGapInputContext,
    TestGapInputCoverage, TestGapInputDependencyEdge, TestGapInputDocument, TestGapInputEvidence,
    TestGapInputLaw, TestGapInputMorphism, TestGapInputRequirement, TestGapInputRiskSignal,
    TestGapInputSymbol, TestGapInputTest, TestGapRepository, TestGapRequirementType,
    TestGapRiskSignalType, TestGapSource, TestGapSymbolKind, TestGapTestType,
    TestGapVerificationCell, TestGapVisibility,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const INPUT_SCHEMA: &str = "highergraphen.test_gap.input.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GitInputRequest {
    pub(crate) repo: PathBuf,
    pub(crate) base: String,
    pub(crate) head: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathInputRequest {
    pub(crate) repo: PathBuf,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) include_tests: bool,
}

pub(crate) fn input_from_git(request: GitInputRequest) -> Result<TestGapInputDocument, String> {
    let metadata = GitInputMetadata::read(&request)?;
    let range = format!("{}..{}", request.base, request.head);

    let commits = commit_summaries(&metadata.repo_path, &range)?;
    let changes = changed_files(&metadata.repo_path, &range)?;
    if changes.is_empty() {
        return Err(format!("git range {range} has no changed files"));
    }
    let diff_analysis = diff_analysis(&metadata.repo_path, &range, &changes)?;

    let repository_id = id(format!("repo:{}", slug(&metadata.repo_name)))?;
    let change_set_id = id(format!(
        "change:test-gap:{}:{}..{}",
        slug(&metadata.repo_name),
        slug(&request.base),
        slug(&request.head)
    ))?;
    let diff_evidence_id = id("evidence:git-diff")?;
    let commit_evidence_id = id("evidence:git-commits")?;

    let mut symbols = symbols_for_changes(&changes, &diff_evidence_id, &diff_analysis)?;
    let mut structural = structural_model_for_changes(&changes, &diff_evidence_id)?;
    structural.extend(semantic_model_for_changes(
        &metadata.repo_path,
        &request.base,
        &request.head,
        &changes,
        &diff_evidence_id,
    )?);
    let test_content = rust_test_content_model_for_changes(
        &metadata.repo_path,
        &request.head,
        &changes,
        &structural,
        &diff_evidence_id,
    )?;
    let rust_test_files = test_content.test_files.clone();
    let content_test_targets = test_content.target_ids_by_file;
    structural.extend(test_content.structural);
    symbols.extend(structural.symbols.clone());
    let tests = tests_for_changes(
        &changes,
        &symbols,
        &content_test_targets,
        &rust_test_files,
        &diff_evidence_id,
    )?;
    let accepted_test_kinds = accepted_test_kinds_for_tests(&tests);
    let mut requirements =
        requirements_for_symbols(&symbols, &diff_evidence_id, &accepted_test_kinds)?;
    requirements.extend(structural_requirements(
        &structural,
        &diff_evidence_id,
        &accepted_test_kinds,
    )?);
    let tests = link_tests_to_requirements(tests, &requirements);
    let verification_cells = verification_cells_for_tests(&tests, &structural, &diff_evidence_id)?;
    let coverage = coverage_for_tests(&tests, &accepted_test_kinds)?;
    let changed_files = changed_files_for_input(&changes, &symbols, &diff_evidence_id)?;
    let contexts = contexts_for_changes(&changes, &change_set_id, "Git Range")?;
    let evidence =
        evidence_for_changes(&changes, &commits, &diff_evidence_id, &commit_evidence_id)?;
    let signals = signals_for_changes(
        &changes,
        &tests,
        &accepted_test_kinds,
        &diff_analysis,
        &diff_evidence_id,
    )?;
    let declared_obligation_ids = requirements
        .iter()
        .map(|requirement| requirement.id.clone())
        .collect::<Vec<_>>();

    Ok(TestGapInputDocument {
        schema: INPUT_SCHEMA.to_owned(),
        source: TestGapSource {
            kind: SourceKind::Code,
            uri: Some(format!("git:{}:{range}", metadata.repo_root)),
            title: Some(format!("Git range {range} in {}", metadata.repo_name)),
            captured_at: None,
            confidence: confidence(1.0)?,
            adapters: vec!["git-diff.v1".to_owned(), "test-gap-from-git.v1".to_owned()],
        },
        repository: TestGapRepository {
            id: repository_id,
            name: metadata.repo_name,
            uri: metadata
                .remote_url
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            default_branch: metadata
                .default_branch
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .or_else(|| Some(request.base.clone())),
        },
        change_set: TestGapChangeSet {
            id: change_set_id,
            base_ref: request.base.clone(),
            head_ref: request.head.clone(),
            base_commit: optional_git(&metadata.repo_path, &["rev-parse", &request.base])
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            head_commit: optional_git(&metadata.repo_path, &["rev-parse", &request.head])
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            boundary: format!("git diff {range}"),
            excluded_paths: vec!["target/".to_owned()],
        },
        changed_files,
        symbols,
        branches: Vec::new(),
        requirements,
        tests,
        coverage,
        dependency_edges: structural.dependency_edges,
        higher_order_cells: structural.higher_order_cells,
        higher_order_incidences: structural.higher_order_incidences,
        morphisms: structural.morphisms,
        laws: structural.laws,
        verification_cells,
        contexts,
        evidence,
        signals,
        detector_context: Some(TestGapDetectorContext {
            required_focus: vec![
                "policy-accepted automated tests for changed source behavior".to_owned(),
                "git-derived deterministic evidence only".to_owned(),
            ],
            excluded_paths: vec!["target/".to_owned()],
            test_kinds: accepted_test_kinds,
            declared_obligation_ids,
        }),
    })
}

pub(crate) fn input_from_path(request: PathInputRequest) -> Result<TestGapInputDocument, String> {
    if request.paths.is_empty() {
        return Err("--path <path> is required".to_owned());
    }

    let metadata = GitInputMetadata::read_repo(&request.repo)?;
    let changes = path_changes(&metadata.repo_path, &request.paths, request.include_tests)?;
    if changes.is_empty() {
        return Err("path scan has no supported files".to_owned());
    }
    let diff_analysis = current_tree_analysis(&metadata.repo_path, &changes)?;

    let repository_id = id(format!("repo:{}", slug(&metadata.repo_name)))?;
    let path_scope = request
        .paths
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>()
        .join(",");
    let change_set_id = id(format!(
        "change:test-gap:{}:path:{}",
        slug(&metadata.repo_name),
        slug(&path_scope)
    ))?;
    let diff_evidence_id = id("evidence:path-scan")?;
    let commit_evidence_id = id("evidence:current-head")?;

    let mut symbols = symbols_for_changes(&changes, &diff_evidence_id, &diff_analysis)?;
    let mut structural = structural_model_for_changes(&changes, &diff_evidence_id)?;
    structural.extend(semantic_model_for_paths(
        &metadata.repo_path,
        &changes,
        &diff_evidence_id,
    )?);
    let test_content = rust_test_content_model_for_paths(
        &metadata.repo_path,
        &changes,
        &structural,
        &diff_evidence_id,
    )?;
    let rust_test_files = test_content.test_files.clone();
    let content_test_targets = test_content.target_ids_by_file;
    structural.extend(test_content.structural);
    symbols.extend(structural.symbols.clone());
    let tests = tests_for_changes(
        &changes,
        &symbols,
        &content_test_targets,
        &rust_test_files,
        &diff_evidence_id,
    )?;
    let accepted_test_kinds = accepted_test_kinds_for_tests(&tests);
    let mut requirements =
        requirements_for_symbols(&symbols, &diff_evidence_id, &accepted_test_kinds)?;
    requirements.extend(structural_requirements(
        &structural,
        &diff_evidence_id,
        &accepted_test_kinds,
    )?);
    let tests = link_tests_to_requirements(tests, &requirements);
    let verification_cells = verification_cells_for_tests(&tests, &structural, &diff_evidence_id)?;
    let coverage = coverage_for_tests(&tests, &accepted_test_kinds)?;
    let changed_files = changed_files_for_input(&changes, &symbols, &diff_evidence_id)?;
    let contexts = contexts_for_changes(&changes, &change_set_id, "Path Scan")?;
    let evidence = evidence_for_changes(&changes, &[], &diff_evidence_id, &commit_evidence_id)?;
    let signals = signals_for_changes(
        &changes,
        &tests,
        &accepted_test_kinds,
        &diff_analysis,
        &diff_evidence_id,
    )?;
    let declared_obligation_ids = requirements
        .iter()
        .map(|requirement| requirement.id.clone())
        .collect::<Vec<_>>();

    Ok(TestGapInputDocument {
        schema: INPUT_SCHEMA.to_owned(),
        source: TestGapSource {
            kind: SourceKind::Code,
            uri: Some(format!("path:{}:{path_scope}", metadata.repo_root)),
            title: Some(format!("Path scan {path_scope} in {}", metadata.repo_name)),
            captured_at: None,
            confidence: confidence(1.0)?,
            adapters: vec![
                "current-tree.v1".to_owned(),
                "test-gap-from-path.v1".to_owned(),
            ],
        },
        repository: TestGapRepository {
            id: repository_id,
            name: metadata.repo_name,
            uri: metadata
                .remote_url
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            default_branch: metadata
                .default_branch
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
        },
        change_set: TestGapChangeSet {
            id: change_set_id,
            base_ref: "current-tree".to_owned(),
            head_ref: "current-tree".to_owned(),
            base_commit: None,
            head_commit: optional_git(&metadata.repo_path, &["rev-parse", "HEAD"])
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            boundary: format!(
                "current tree path scan {path_scope}{}",
                if request.include_tests {
                    " with tests"
                } else {
                    ""
                }
            ),
            excluded_paths: vec![".git/".to_owned(), "target/".to_owned()],
        },
        changed_files,
        symbols,
        branches: Vec::new(),
        requirements,
        tests,
        coverage,
        dependency_edges: structural.dependency_edges,
        higher_order_cells: structural.higher_order_cells,
        higher_order_incidences: structural.higher_order_incidences,
        morphisms: structural.morphisms,
        laws: structural.laws,
        verification_cells,
        contexts,
        evidence,
        signals,
        detector_context: Some(TestGapDetectorContext {
            required_focus: vec![
                "policy-accepted automated tests for selected path behavior".to_owned(),
                "current-tree path-derived deterministic evidence only".to_owned(),
            ],
            excluded_paths: vec![".git/".to_owned(), "target/".to_owned()],
            test_kinds: accepted_test_kinds,
            declared_obligation_ids,
        }),
    })
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
        Self::read_repo(&request.repo)
    }

    fn read_repo(repo: &Path) -> Result<Self, String> {
        let repo_root = git(repo, &["rev-parse", "--show-toplevel"])?
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
    structural_boundary_ids: Vec<Id>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct GitDiffFile {
    path: String,
    added_lines: Vec<String>,
    removed_lines: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct StructuralModel {
    symbols: Vec<TestGapInputSymbol>,
    dependency_edges: Vec<TestGapInputDependencyEdge>,
    higher_order_cells: Vec<TestGapHigherOrderCell>,
    higher_order_incidences: Vec<TestGapHigherOrderIncidence>,
    morphisms: Vec<TestGapInputMorphism>,
    laws: Vec<TestGapInputLaw>,
}

impl StructuralModel {
    fn extend(&mut self, other: StructuralModel) {
        self.symbols.extend(other.symbols);
        self.dependency_edges.extend(other.dependency_edges);
        self.higher_order_cells.extend(other.higher_order_cells);
        self.higher_order_incidences
            .extend(other.higher_order_incidences);
        self.morphisms.extend(other.morphisms);
        self.laws.extend(other.laws);
    }
}

fn path_changes(
    repo: &Path,
    paths: &[PathBuf],
    include_tests: bool,
) -> Result<Vec<GitChange>, String> {
    let mut files = BTreeSet::<String>::new();
    for path in paths {
        let resolved = resolve_input_path(repo, path)?;
        collect_current_tree_files(repo, &resolved, &mut files)?;
    }
    if include_tests {
        collect_tests(repo, &mut files)?;
    }

    files
        .into_iter()
        .map(|path| {
            Ok(GitChange {
                path,
                old_path: None,
                change_type: PrReviewTargetChangeType::Modified,
                additions: 0,
                deletions: 0,
            })
        })
        .collect()
}

fn resolve_input_path(repo: &Path, path: &Path) -> Result<PathBuf, String> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo.join(path)
    };
    let canonical = fs::canonicalize(&candidate)
        .map_err(|error| format!("failed to resolve path {}: {error}", candidate.display()))?;
    if !canonical.starts_with(repo) {
        return Err(format!(
            "path {} is outside repository {}",
            canonical.display(),
            repo.display()
        ));
    }
    Ok(canonical)
}

fn collect_tests(repo: &Path, files: &mut BTreeSet<String>) -> Result<(), String> {
    collect_current_tree_files_with_filter(repo, repo, files, &|relative| is_test_path(relative))
}

fn collect_current_tree_files(
    repo: &Path,
    path: &Path,
    files: &mut BTreeSet<String>,
) -> Result<(), String> {
    collect_current_tree_files_with_filter(repo, path, files, &|_| true)
}

fn collect_current_tree_files_with_filter(
    repo: &Path,
    path: &Path,
    files: &mut BTreeSet<String>,
    include: &dyn Fn(&str) -> bool,
) -> Result<(), String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("failed to read path {}: {error}", path.display()))?;
    if metadata.is_dir() {
        for entry in fs::read_dir(path)
            .map_err(|error| format!("failed to read directory {}: {error}", path.display()))?
        {
            let entry =
                entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
            let child = entry.path();
            if should_skip_path(repo, &child)? {
                continue;
            }
            collect_current_tree_files_with_filter(repo, &child, files, include)?;
        }
    } else if metadata.is_file() {
        let relative = relative_repo_path(repo, path)?;
        if include(&relative) && supported_path_input_file(&relative) {
            files.insert(relative);
        }
    }
    Ok(())
}

fn should_skip_path(repo: &Path, path: &Path) -> Result<bool, String> {
    let relative = relative_repo_path(repo, path)?;
    Ok(relative == ".git"
        || relative.starts_with(".git/")
        || relative == "target"
        || relative.starts_with("target/"))
}

fn relative_repo_path(repo: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(repo).map_err(|_| {
        format!(
            "path {} is outside repository {}",
            path.display(),
            repo.display()
        )
    })?;
    let mut parts = Vec::new();
    for component in relative.components() {
        let value = component.as_os_str().to_str().ok_or_else(|| {
            format!(
                "path {} is not valid UTF-8 relative to {}",
                path.display(),
                repo.display()
            )
        })?;
        if !value.is_empty() {
            parts.push(value.to_owned());
        }
    }
    Ok(parts.join("/"))
}

fn supported_path_input_file(path: &str) -> bool {
    is_source_code_path(path)
        || is_test_path(path)
        || path.ends_with(".schema.json")
        || path.ends_with(".example.json")
        || path.ends_with(".md")
        || path.ends_with(".toml")
        || path.ends_with(".yaml")
        || path.ends_with(".yml")
        || path.ends_with(".json")
}

fn current_tree_analysis(repo: &Path, changes: &[GitChange]) -> Result<GitDiffAnalysis, String> {
    let mut analysis = GitDiffAnalysis::default();
    for change in changes {
        let file_id = file_id(&change.path)?;
        let Ok(contents) = fs::read_to_string(repo.join(&change.path)) else {
            continue;
        };
        if is_source_code_path(&change.path) && has_public_api_like_text(&contents) {
            push_unique_id(&mut analysis.public_api_ids, file_id.clone());
        }
        if change.path.ends_with(".schema.json")
            || contents.contains("#[serde")
            || contents.contains("deny_unknown_fields")
            || contents.contains("rename_all")
        {
            push_unique_id(&mut analysis.serde_contract_ids, file_id.clone());
        }
        if !is_test_path(&change.path) && has_panic_or_placeholder_text(&contents) {
            push_unique_id(&mut analysis.panic_or_placeholder_ids, file_id.clone());
        }
        if !is_test_path(&change.path) && has_external_effect_text(&contents) {
            push_unique_id(&mut analysis.external_effect_ids, file_id.clone());
        }
        if is_highergraphen_structural_path(&change.path)
            || is_test_gap_surface_path(&change.path)
            || is_semantic_proof_surface_path(&change.path)
        {
            push_unique_id(&mut analysis.structural_boundary_ids, file_id);
        }
    }
    Ok(analysis)
}

fn has_public_api_like_text(contents: &str) -> bool {
    contents.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("pub ")
            || trimmed.starts_with("pub(")
            || trimmed.starts_with("pub(crate)")
            || trimmed.starts_with("pub struct")
            || trimmed.starts_with("pub enum")
            || trimmed.starts_with("pub fn")
    })
}

fn has_panic_or_placeholder_text(contents: &str) -> bool {
    contents.lines().any(|line| {
        line.contains(".unwrap(")
            || line.contains(".expect(")
            || line.contains("panic!(")
            || line.contains("todo!(")
            || line.contains("unimplemented!(")
    })
}

fn has_external_effect_text(contents: &str) -> bool {
    contents.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.contains("Command::new")
            || trimmed.contains("ProcessCommand::new")
            || trimmed.contains("fs::read")
            || trimmed.contains("fs::write")
            || trimmed.contains("fs::remove")
            || trimmed.contains("fs::create")
            || trimmed.contains("File::open")
            || trimmed.contains("File::create")
    })
}

fn changed_files_for_input(
    changes: &[GitChange],
    symbols: &[TestGapInputSymbol],
    diff_evidence_id: &Id,
) -> Result<Vec<TestGapInputChangedFile>, String> {
    let symbol_ids_by_file = symbols
        .iter()
        .map(|symbol| (symbol.file_id.clone(), symbol.id.clone()))
        .fold(
            BTreeMap::<Id, Vec<Id>>::new(),
            |mut by_file, (file_id, symbol_id)| {
                by_file.entry(file_id).or_default().push(symbol_id);
                by_file
            },
        );

    changes
        .iter()
        .map(|change| {
            let file_id = file_id(&change.path)?;
            Ok(TestGapInputChangedFile {
                id: file_id.clone(),
                path: change.path.clone(),
                change_type: map_change_type(change.change_type),
                old_path: change.old_path.clone(),
                language: language_for_path(&change.path),
                additions: change.additions,
                deletions: change.deletions,
                symbol_ids: symbol_ids_by_file
                    .get(&file_id)
                    .cloned()
                    .unwrap_or_default(),
                context_ids: test_gap_context_ids_for_path(&change.path)?,
                source_ids: vec![diff_evidence_id.clone()],
            })
        })
        .collect()
}

fn symbols_for_changes(
    changes: &[GitChange],
    diff_evidence_id: &Id,
    diff_analysis: &GitDiffAnalysis,
) -> Result<Vec<TestGapInputSymbol>, String> {
    changes
        .iter()
        .filter(|change| is_source_code_path(&change.path))
        .map(|change| {
            let file_id = file_id(&change.path)?;
            let symbol_id = id(format!("symbol:{}:changed-behavior", slug(&change.path)))?;
            let structural_path = is_highergraphen_structural_path(&change.path);
            let public_api = !structural_path && diff_analysis.public_api_ids.contains(&file_id);
            Ok(TestGapInputSymbol {
                id: symbol_id,
                file_id,
                name: format!("Changed behavior in {}", change.path),
                kind: if public_api {
                    TestGapSymbolKind::PublicApi
                } else {
                    TestGapSymbolKind::Module
                },
                visibility: if public_api {
                    TestGapVisibility::Public
                } else {
                    TestGapVisibility::Unknown
                },
                public_api,
                path: Some(change.path.clone()),
                line_start: None,
                line_end: None,
                branch_ids: Vec::new(),
                requirement_ids: if structural_path {
                    Vec::new()
                } else {
                    vec![id(format!(
                        "requirement:{}:unit-verification",
                        slug(&change.path)
                    ))?]
                },
                context_ids: test_gap_context_ids_for_path(&change.path)?,
                source_ids: vec![diff_evidence_id.clone()],
            })
        })
        .collect()
}

fn structural_model_for_changes(
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<StructuralModel, String> {
    let mut model = StructuralModel::default();
    let has_test_gap_surface = changes
        .iter()
        .any(|change| is_test_gap_surface_path(&change.path));
    let has_semantic_proof_surface = changes
        .iter()
        .any(|change| is_semantic_proof_surface_path(&change.path));
    if !has_test_gap_surface && !has_semantic_proof_surface {
        return Ok(model);
    }

    if has_test_gap_surface {
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/main.rs",
            "command:highergraphen:test-gap:detect",
            "highergraphen test-gap detect command cell",
            "highergraphen test-gap detect",
            TestGapSymbolKind::PublicApi,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/main.rs",
            "command:highergraphen:test-gap:input-from-git",
            "highergraphen test-gap input from-git command cell",
            "highergraphen test-gap input from-git",
            TestGapSymbolKind::PublicApi,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/main.rs",
            "command:highergraphen:test-gap:input-from-path",
            "highergraphen test-gap input from-path command cell",
            "highergraphen test-gap input from-path",
            TestGapSymbolKind::PublicApi,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/test_gap_git.rs",
            "adapter:test-gap:git-input",
            "test-gap git input adapter cell",
            "test_gap_git::input_from_git",
            TestGapSymbolKind::Module,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "tools/highergraphen-cli/src/test_gap_git.rs",
            "adapter:test-gap:path-input",
            "test-gap current-tree path input adapter cell",
            "test_gap_git::input_from_path",
            TestGapSymbolKind::Module,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/workflows/test_gap.rs",
            "runner:test-gap:detect",
            "run_test_gap_detect workflow runner cell",
            "run_test_gap_detect",
            TestGapSymbolKind::Function,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/lib.rs",
            "export:test-gap:runtime-api",
            "test-gap runtime public export cell",
            "higher_graphen_runtime test-gap exports",
            TestGapSymbolKind::PublicApi,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/workflows/mod.rs",
            "registry:test-gap:workflow-module",
            "test-gap workflow registry cell",
            "workflows::test_gap module registry",
            TestGapSymbolKind::Module,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/test_gap_reports.rs",
            "contract:test-gap:runtime-report-shapes",
            "test-gap runtime report shape contract cell",
            "TestGap input and report runtime shapes",
            TestGapSymbolKind::Type,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "crates/higher-graphen-runtime/src/reports.rs",
            "projection:test-gap:report-envelope",
            "test-gap report envelope projection cell",
            "ReportEnvelope projection boundary for test-gap",
            TestGapSymbolKind::Type,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "schemas/inputs/test-gap.input.schema.json",
            "schema:test-gap:input",
            "test-gap input schema contract cell",
            "highergraphen.test_gap.input.v1 schema",
            TestGapSymbolKind::Unknown,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "schemas/reports/test-gap.report.schema.json",
            "schema:test-gap:report",
            "test-gap report schema contract cell",
            "highergraphen.test_gap.report.v1 schema",
            TestGapSymbolKind::Unknown,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "schemas/inputs/test-gap.input.example.json",
            "fixture:test-gap:input-example",
            "test-gap input example fixture cell",
            "test-gap input example fixture",
            TestGapSymbolKind::Unknown,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "schemas/reports/test-gap.report.example.json",
            "fixture:test-gap:report-example",
            "test-gap report example fixture cell",
            "test-gap report example fixture",
            TestGapSymbolKind::Unknown,
        )?;
        push_structural_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            "scripts/validate-json-contracts.py",
            "validator:test-gap:json-contracts",
            "JSON contract validation command cell",
            "scripts/validate-json-contracts.py",
            TestGapSymbolKind::Unknown,
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/main.rs"],
            "law:test-gap:command-routes-to-runner",
            "CLI command routes to the intended test-gap runner",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/main.rs"],
            "law:test-gap:json-format-required",
            "test-gap CLI commands require --format json",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/main.rs"],
            "law:test-gap:output-file-suppresses-stdout",
            "test-gap CLI --output writes the target file without JSON stdout",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/test_gap_git.rs"],
            "law:test-gap:input-from-git-is-deterministic",
            "test-gap input from-git derives a deterministic bounded snapshot",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/test_gap_git.rs"],
            "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
            "test-gap input from-git declares that git structure does not prove semantic coverage",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/test_gap_git.rs"],
            "law:test-gap:input-from-path-is-deterministic",
            "test-gap input from-path derives a deterministic bounded snapshot from selected current-tree files",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["tools/highergraphen-cli/src/test_gap_git.rs"],
            "law:test-gap:input-from-path-declares-snapshot-boundary",
            "test-gap input from-path keeps current-tree scope and semantic coverage limits visible",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["crates/higher-graphen-runtime/src/workflows/test_gap.rs"],
            "law:test-gap:test-gap-is-bounded",
            "no_gaps_in_snapshot is bounded to the supplied snapshot and detector policy",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["crates/higher-graphen-runtime/src/workflows/test_gap.rs"],
            "law:test-gap:verification-policy-controls-test-kind",
            "detector_context.test_kinds controls which test kinds close obligations",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["crates/higher-graphen-runtime/src/workflows/test_gap.rs"],
            "law:test-gap:requirements-map-to-implementation-and-test",
            "in-scope requirements map to implementation cells and accepted verification cells",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "crates/higher-graphen-runtime/src/workflows/test_gap.rs",
                "crates/higher-graphen-runtime/src/test_gap_reports.rs",
            ],
            "law:test-gap:candidates-remain-unreviewed",
            "detector completion candidates remain unreviewed until explicit review",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "crates/higher-graphen-runtime/src/workflows/test_gap.rs",
                "crates/higher-graphen-runtime/src/reports.rs",
            ],
            "law:test-gap:projection-declares-information-loss",
            "test-gap projections declare information loss for human, AI, and audit views",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "crates/higher-graphen-runtime/src/test_gap_reports.rs",
                "schemas/inputs/test-gap.input.schema.json",
                "schemas/reports/test-gap.report.schema.json",
            ],
            "law:test-gap:schema-id-preserved",
            "test-gap input and report schema IDs are preserved",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &["crates/higher-graphen-runtime/src/test_gap_reports.rs"],
            "law:test-gap:enum-casing-round-trips",
            "test-gap enum casing serializes as lower snake case and round-trips",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "crates/higher-graphen-runtime/src/test_gap_reports.rs",
                "schemas/inputs/test-gap.input.schema.json",
                "schemas/reports/test-gap.report.schema.json",
            ],
            "law:test-gap:runtime-shapes-preserve-schema",
            "runtime TestGap shapes preserve the checked-in schema boundary",
        )?;
        push_law_symbol(
            &mut model,
            changes,
            diff_evidence_id,
            &[
                "scripts/validate-json-contracts.py",
                "schemas/inputs/test-gap.input.example.json",
                "schemas/reports/test-gap.report.example.json",
                "schemas/inputs/test-gap.input.schema.json",
                "schemas/reports/test-gap.report.schema.json",
            ],
            "law:test-gap:fixtures-validate-against-schema",
            "test-gap fixtures validate against their declared JSON schemas",
        )?;

        push_structural_edge(
            &mut model,
            "edge:test-gap:command-detect-to-runner",
            "command:highergraphen:test-gap:detect",
            "runner:test-gap:detect",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:input-from-git-to-adapter",
            "command:highergraphen:test-gap:input-from-git",
            "adapter:test-gap:git-input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:input-from-path-to-adapter",
            "command:highergraphen:test-gap:input-from-path",
            "adapter:test-gap:path-input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:git-adapter-to-input-schema",
            "adapter:test-gap:git-input",
            "schema:test-gap:input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:path-adapter-to-input-schema",
            "adapter:test-gap:path-input",
            "schema:test-gap:input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:runtime-export-to-runner",
            "export:test-gap:runtime-api",
            "runner:test-gap:detect",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:workflow-registry-to-runner",
            "registry:test-gap:workflow-module",
            "runner:test-gap:detect",
            TestGapDependencyRelationType::Contains,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:runtime-shapes-to-input-schema",
            "contract:test-gap:runtime-report-shapes",
            "schema:test-gap:input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:runtime-shapes-to-report-schema",
            "contract:test-gap:runtime-report-shapes",
            "schema:test-gap:report",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:input-fixture-to-input-schema",
            "fixture:test-gap:input-example",
            "schema:test-gap:input",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:report-fixture-to-report-schema",
            "fixture:test-gap:report-example",
            "schema:test-gap:report",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:report-envelope-to-runtime-shapes",
            "projection:test-gap:report-envelope",
            "contract:test-gap:runtime-report-shapes",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:validator-to-input-fixture",
            "validator:test-gap:json-contracts",
            "fixture:test-gap:input-example",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;
        push_structural_edge(
            &mut model,
            "edge:test-gap:validator-to-report-fixture",
            "validator:test-gap:json-contracts",
            "fixture:test-gap:report-example",
            TestGapDependencyRelationType::Supports,
            diff_evidence_id,
        )?;

        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:command-detect-to-runner",
            "command_to_runner",
            &["command:highergraphen:test-gap:detect"],
            &["runner:test-gap:detect"],
            &["law:test-gap:command-routes-to-runner"],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:input-from-git-to-input-schema",
            "adapter_to_input_schema",
            &[
                "command:highergraphen:test-gap:input-from-git",
                "adapter:test-gap:git-input",
            ],
            &["schema:test-gap:input"],
            &[
                "law:test-gap:input-from-git-is-deterministic",
                "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
            ],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:input-from-path-to-input-schema",
            "adapter_to_input_schema",
            &[
                "command:highergraphen:test-gap:input-from-path",
                "adapter:test-gap:path-input",
            ],
            &["schema:test-gap:input"],
            &[
                "law:test-gap:input-from-path-is-deterministic",
                "law:test-gap:input-from-path-declares-snapshot-boundary",
            ],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:runtime-shapes-to-schemas",
            "runtime_shape_to_schema",
            &["contract:test-gap:runtime-report-shapes"],
            &["schema:test-gap:input", "schema:test-gap:report"],
            &[
                "law:test-gap:schema-id-preserved",
                "law:test-gap:enum-casing-round-trips",
                "law:test-gap:runtime-shapes-preserve-schema",
            ],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:fixtures-to-schemas",
            "fixture_to_schema",
            &[
                "fixture:test-gap:input-example",
                "fixture:test-gap:report-example",
            ],
            &["schema:test-gap:input", "schema:test-gap:report"],
            &["law:test-gap:fixtures-validate-against-schema"],
            diff_evidence_id,
        )?;
        push_higher_order_morphism(
            &mut model,
            "morphism:test-gap:report-envelope-to-runtime-shapes",
            "projection_to_runtime_shape",
            &["projection:test-gap:report-envelope"],
            &["contract:test-gap:runtime-report-shapes"],
            &["law:test-gap:projection-declares-information-loss"],
            diff_evidence_id,
        )?;
    }

    if has_semantic_proof_surface {
        push_semantic_proof_structural_model(&mut model, changes, diff_evidence_id)?;
    }

    Ok(model)
}

fn push_semantic_proof_structural_model(
    model: &mut StructuralModel,
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<(), String> {
    if let Some(schema_change) = changes
        .iter()
        .find(|change| change.path.ends_with(".schema.json"))
    {
        push_structural_symbol_unconditional(
            model,
            diff_evidence_id,
            &schema_change.path,
            "validator:test-gap:json-contracts",
            "JSON contract validation command cell",
            "scripts/validate-json-contracts.py",
            TestGapSymbolKind::Unknown,
        )?;
    }
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/main.rs",
        "command:highergraphen:semantic-proof:backend-run",
        "highergraphen semantic-proof backend run command cell",
        "highergraphen semantic-proof backend run",
        TestGapSymbolKind::PublicApi,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/main.rs",
        "command:highergraphen:semantic-proof:input-from-artifact",
        "highergraphen semantic-proof input from-artifact command cell",
        "highergraphen semantic-proof input from-artifact",
        TestGapSymbolKind::PublicApi,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/main.rs",
        "command:highergraphen:semantic-proof:verify",
        "highergraphen semantic-proof verify command cell",
        "highergraphen semantic-proof verify",
        TestGapSymbolKind::PublicApi,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/main.rs",
        "command:highergraphen:semantic-proof:input-from-report",
        "highergraphen semantic-proof input from-report command cell",
        "highergraphen semantic-proof input from-report",
        TestGapSymbolKind::PublicApi,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/semantic_proof_backend.rs",
        "runner:semantic-proof:backend-run",
        "semantic-proof local backend runner cell",
        "semantic_proof_backend::run_backend",
        TestGapSymbolKind::Module,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/semantic_proof_artifact.rs",
        "adapter:semantic-proof:artifact-input",
        "semantic-proof artifact input adapter cell",
        "semantic_proof_artifact::input_from_artifact",
        TestGapSymbolKind::Module,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/semantic_proof_reinput.rs",
        "adapter:semantic-proof:reinput-from-report",
        "semantic-proof report reinput adapter cell",
        "semantic_proof_reinput::input_from_report",
        TestGapSymbolKind::Module,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/tests/command.rs",
        "test:semantic-proof:artifact-roundtrip",
        "semantic-proof artifact roundtrip CLI test cell",
        "semantic_proof_input_from_artifact roundtrip tests",
        TestGapSymbolKind::Function,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/tests/command.rs",
        "test:semantic-proof:backend-and-reinput",
        "semantic-proof backend runner and reinput CLI test cell",
        "semantic_proof_backend_run and semantic_proof_input_from_report tests",
        TestGapSymbolKind::Function,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/semantic_proof_artifact.rs",
        "theorem:semantic-proof:artifact-adapter-correctness",
        "semantic-proof artifact adapter correctness theorem",
        "artifact adapter preserves semantic proof obligations",
        TestGapSymbolKind::Unknown,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/semantic_proof_backend.rs",
        "theorem:semantic-proof:backend-run-trust-boundary",
        "semantic-proof backend runner trust boundary theorem",
        "backend runner records command output without accepting it beyond policy",
        TestGapSymbolKind::Unknown,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/semantic_proof_reinput.rs",
        "theorem:semantic-proof:obligation-reinput-correctness",
        "semantic-proof obligation reinput correctness theorem",
        "insufficient proof reports requeue unproved laws and morphisms",
        TestGapSymbolKind::Unknown,
    )?;

    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/semantic_proof_backend.rs"],
        "law:semantic-proof:backend-run-records-trust-boundary",
        "backend runs record command, hashes, exit status, and unreviewed failing outputs at the trust boundary",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/semantic_proof_artifact.rs"],
        "law:semantic-proof:artifact-status-totality",
        "artifact statuses are total over proved, counterexample, and counterexample_found",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/semantic_proof_artifact.rs"],
        "law:semantic-proof:certificate-policy-preservation",
        "proved artifacts preserve backend, hashes, witnesses, and accepted review policy",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/semantic_proof_artifact.rs"],
        "law:semantic-proof:counterexample-refutation-preservation",
        "counterexample artifacts preserve theorem, law, morphism, path, severity, and review state",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &[
            "tools/highergraphen-cli/src/semantic_proof_artifact.rs",
            "docs/cli/highergraphen.md",
            "skills/highergraphen/SKILL.md",
        ],
        "law:semantic-proof:backend-boundary-is-explicit",
        "artifact adapter normalizes already-produced backend artifacts without executing proof backends",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &[
            "crates/higher-graphen-runtime/src/workflows/semantic_proof.rs",
            "crates/higher-graphen-runtime/src/semantic_proof_reports.rs",
            "schemas/inputs/semantic-proof.input.schema.json",
        ],
        "law:semantic-proof:counterexample-review-policy",
        "counterexamples only refute automatically when the policy does not require accepted review or the counterexample is accepted",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/semantic_proof_reinput.rs"],
        "law:semantic-proof:insufficient-proof-reinputs-open-obligations",
        "insufficient proof reports generate new inputs containing the open law and morphism obligations",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/tests/command.rs"],
        "law:semantic-proof:roundtrip-tests-cover-proof-and-counterexample",
        "CLI roundtrip tests cover proved and counterexample artifact paths through verify",
    )?;

    push_structural_edge(
        model,
        "edge:semantic-proof:backend-run-to-artifact-adapter",
        "command:highergraphen:semantic-proof:backend-run",
        "runner:semantic-proof:backend-run",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;
    push_structural_edge(
        model,
        "edge:semantic-proof:backend-run-to-input-from-artifact",
        "runner:semantic-proof:backend-run",
        "adapter:semantic-proof:artifact-input",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;
    push_structural_edge(
        model,
        "edge:semantic-proof:command-input-to-adapter",
        "command:highergraphen:semantic-proof:input-from-artifact",
        "adapter:semantic-proof:artifact-input",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;
    push_structural_edge(
        model,
        "edge:semantic-proof:adapter-to-verify-command",
        "adapter:semantic-proof:artifact-input",
        "command:highergraphen:semantic-proof:verify",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;
    push_structural_edge(
        model,
        "edge:semantic-proof:command-reinput-to-adapter",
        "command:highergraphen:semantic-proof:input-from-report",
        "adapter:semantic-proof:reinput-from-report",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;
    push_structural_edge(
        model,
        "edge:semantic-proof:reinput-to-verify-command",
        "adapter:semantic-proof:reinput-from-report",
        "command:highergraphen:semantic-proof:verify",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;
    push_structural_edge(
        model,
        "edge:semantic-proof:roundtrip-test-to-adapter",
        "test:semantic-proof:artifact-roundtrip",
        "adapter:semantic-proof:artifact-input",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;
    push_structural_edge(
        model,
        "edge:semantic-proof:backend-test-to-runner",
        "test:semantic-proof:backend-and-reinput",
        "runner:semantic-proof:backend-run",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;
    push_structural_edge(
        model,
        "edge:semantic-proof:reinput-test-to-adapter",
        "test:semantic-proof:backend-and-reinput",
        "adapter:semantic-proof:reinput-from-report",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;
    push_structural_edge(
        model,
        "edge:semantic-proof:theorem-to-adapter",
        "theorem:semantic-proof:artifact-adapter-correctness",
        "adapter:semantic-proof:artifact-input",
        TestGapDependencyRelationType::Supports,
        diff_evidence_id,
    )?;

    push_higher_order_morphism(
        model,
        "morphism:semantic-proof:backend-run-to-artifact",
        "backend_execution_to_bounded_artifact",
        &[
            "command:highergraphen:semantic-proof:backend-run",
            "runner:semantic-proof:backend-run",
        ],
        &[
            "adapter:semantic-proof:artifact-input",
            "theorem:semantic-proof:backend-run-trust-boundary",
        ],
        &[
            "law:semantic-proof:backend-run-records-trust-boundary",
            "law:semantic-proof:backend-boundary-is-explicit",
        ],
        diff_evidence_id,
    )?;
    push_higher_order_morphism(
        model,
        "morphism:semantic-proof:artifact-to-input-document",
        "artifact_to_semantic_proof_input",
        &[
            "command:highergraphen:semantic-proof:input-from-artifact",
            "adapter:semantic-proof:artifact-input",
        ],
        &["theorem:semantic-proof:artifact-adapter-correctness"],
        &[
            "law:semantic-proof:artifact-status-totality",
            "law:semantic-proof:backend-boundary-is-explicit",
        ],
        diff_evidence_id,
    )?;
    push_higher_order_morphism(
        model,
        "morphism:semantic-proof:certificate-to-proof-object",
        "certificate_artifact_to_proof_object",
        &["adapter:semantic-proof:artifact-input"],
        &["command:highergraphen:semantic-proof:verify"],
        &["law:semantic-proof:certificate-policy-preservation"],
        diff_evidence_id,
    )?;
    push_higher_order_morphism(
        model,
        "morphism:semantic-proof:counterexample-to-refutation",
        "counterexample_artifact_to_refutation",
        &["adapter:semantic-proof:artifact-input"],
        &["command:highergraphen:semantic-proof:verify"],
        &["law:semantic-proof:counterexample-refutation-preservation"],
        diff_evidence_id,
    )?;
    push_higher_order_morphism(
        model,
        "morphism:semantic-proof:counterexample-review-to-obstruction",
        "counterexample_review_policy_to_obstruction",
        &[
            "adapter:semantic-proof:artifact-input",
            "law:semantic-proof:counterexample-review-policy",
        ],
        &["command:highergraphen:semantic-proof:verify"],
        &["law:semantic-proof:counterexample-review-policy"],
        diff_evidence_id,
    )?;
    push_higher_order_morphism(
        model,
        "morphism:semantic-proof:insufficient-report-to-reinput",
        "insufficient_report_to_open_obligation_input",
        &[
            "command:highergraphen:semantic-proof:input-from-report",
            "adapter:semantic-proof:reinput-from-report",
        ],
        &["theorem:semantic-proof:obligation-reinput-correctness"],
        &["law:semantic-proof:insufficient-proof-reinputs-open-obligations"],
        diff_evidence_id,
    )?;
    push_higher_order_morphism(
        model,
        "morphism:semantic-proof:roundtrip-tests-to-adapter-correctness",
        "roundtrip_test_to_adapter_correctness",
        &[
            "test:semantic-proof:artifact-roundtrip",
            "test:semantic-proof:backend-and-reinput",
        ],
        &[
            "theorem:semantic-proof:artifact-adapter-correctness",
            "theorem:semantic-proof:backend-run-trust-boundary",
            "theorem:semantic-proof:obligation-reinput-correctness",
        ],
        &["law:semantic-proof:roundtrip-tests-cover-proof-and-counterexample"],
        diff_evidence_id,
    )?;

    Ok(())
}

fn semantic_model_for_changes(
    repo: &Path,
    base_ref: &str,
    head_ref: &str,
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<StructuralModel, String> {
    let mut model = StructuralModel::default();
    for change in changes {
        let base_path = change.old_path.as_deref().unwrap_or(&change.path);
        let mut base_cells = Vec::new();
        let mut head_cells = Vec::new();
        if change.path.ends_with(".rs") && is_source_code_path(&change.path) {
            if change.change_type != PrReviewTargetChangeType::Added {
                if let Some(contents) = git_show_file(repo, base_ref, base_path) {
                    base_cells = push_rust_semantic_cells(
                        &mut model,
                        change,
                        SemanticRevision::Base,
                        base_path,
                        &contents,
                        diff_evidence_id,
                    )?;
                }
            }
            if change.change_type != PrReviewTargetChangeType::Deleted {
                if let Some(contents) = git_show_file(repo, head_ref, &change.path) {
                    head_cells = push_rust_semantic_cells(
                        &mut model,
                        change,
                        SemanticRevision::Head,
                        &change.path,
                        &contents,
                        diff_evidence_id,
                    )?;
                }
            }
        } else if change.path.ends_with(".schema.json") {
            if change.change_type != PrReviewTargetChangeType::Added {
                if let Some(contents) = git_show_file(repo, base_ref, base_path) {
                    base_cells = push_json_schema_semantic_cells(
                        &mut model,
                        change,
                        SemanticRevision::Base,
                        base_path,
                        &contents,
                        diff_evidence_id,
                    )?;
                }
            }
            if change.change_type != PrReviewTargetChangeType::Deleted {
                if let Some(contents) = git_show_file(repo, head_ref, &change.path) {
                    head_cells = push_json_schema_semantic_cells(
                        &mut model,
                        change,
                        SemanticRevision::Head,
                        &change.path,
                        &contents,
                        diff_evidence_id,
                    )?;
                }
            }
        }
        push_semantic_delta_structure(
            &mut model,
            change,
            &base_cells,
            &head_cells,
            diff_evidence_id,
        )?;
    }
    Ok(model)
}

fn semantic_model_for_paths(
    repo: &Path,
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<StructuralModel, String> {
    let mut model = StructuralModel::default();
    for change in changes {
        let mut head_cells = Vec::new();
        let path = repo.join(&change.path);
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        if change.path.ends_with(".rs") && is_source_code_path(&change.path) {
            head_cells = push_rust_semantic_cells(
                &mut model,
                change,
                SemanticRevision::Head,
                &change.path,
                &contents,
                diff_evidence_id,
            )?;
        } else if change.path.ends_with(".schema.json") {
            head_cells = push_json_schema_semantic_cells(
                &mut model,
                change,
                SemanticRevision::Head,
                &change.path,
                &contents,
                diff_evidence_id,
            )?;
        }
        push_semantic_delta_structure(&mut model, change, &[], &head_cells, diff_evidence_id)?;
    }
    Ok(model)
}

fn rust_test_content_model_for_changes(
    repo: &Path,
    head_ref: &str,
    changes: &[GitChange],
    target_model: &StructuralModel,
    diff_evidence_id: &Id,
) -> Result<RustTestContentModel, String> {
    let mut content_model = RustTestContentModel::default();
    for change in changes {
        if !is_rust_source_path(&change.path)
            || change.change_type == PrReviewTargetChangeType::Deleted
        {
            continue;
        }
        if let Some(contents) = git_show_file(repo, head_ref, &change.path) {
            push_rust_test_content_cells(
                &mut content_model,
                target_model,
                change,
                SemanticRevision::Head,
                &change.path,
                &contents,
                diff_evidence_id,
            )?;
        }
    }
    Ok(content_model)
}

fn rust_test_content_model_for_paths(
    repo: &Path,
    changes: &[GitChange],
    target_model: &StructuralModel,
    diff_evidence_id: &Id,
) -> Result<RustTestContentModel, String> {
    let mut content_model = RustTestContentModel::default();
    for change in changes {
        if !is_rust_source_path(&change.path) {
            continue;
        }
        let path = repo.join(&change.path);
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        push_rust_test_content_cells(
            &mut content_model,
            target_model,
            change,
            SemanticRevision::Head,
            &change.path,
            &contents,
            diff_evidence_id,
        )?;
    }
    Ok(content_model)
}

fn git_show_file(repo: &Path, rev: &str, path: &str) -> Option<String> {
    let rev_path = format!("{rev}:{path}");
    optional_git(repo, &["show", &rev_path])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SemanticRevision {
    Base,
    Head,
}

impl SemanticRevision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Head => "head",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SemanticCell {
    id: Id,
    key: String,
    cell_type: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RustTestContentModel {
    structural: StructuralModel,
    target_ids_by_file: BTreeMap<String, Vec<Id>>,
    test_files: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HgRustTestBindingRule {
    trigger_terms: &'static [&'static str],
    cli_label: Option<&'static str>,
    target_ids: &'static [&'static str],
}

const HG_RUST_TEST_BINDING_RULES: &[HgRustTestBindingRule] = &[
    HgRustTestBindingRule {
        trigger_terms: &["test-gap", "input", "from-git"],
        cli_label: Some("highergraphen test-gap input from-git"),
        target_ids: &[
            "command:highergraphen:test-gap:input-from-git",
            "adapter:test-gap:git-input",
            "morphism:test-gap:input-from-git-to-input-schema",
            "law:test-gap:input-from-git-is-deterministic",
            "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
        ],
    },
    HgRustTestBindingRule {
        trigger_terms: &["test-gap", "input", "from-path"],
        cli_label: Some("highergraphen test-gap input from-path"),
        target_ids: &[
            "command:highergraphen:test-gap:input-from-path",
            "adapter:test-gap:path-input",
            "morphism:test-gap:input-from-path-to-input-schema",
            "law:test-gap:input-from-path-is-deterministic",
            "law:test-gap:input-from-path-declares-snapshot-boundary",
        ],
    },
    HgRustTestBindingRule {
        trigger_terms: &["test-gap", "evidence", "from-test-run"],
        cli_label: Some("highergraphen test-gap evidence from-test-run"),
        target_ids: &[],
    },
    HgRustTestBindingRule {
        trigger_terms: &["test-gap", "detect"],
        cli_label: Some("highergraphen test-gap detect"),
        target_ids: &[
            "command:highergraphen:test-gap:detect",
            "runner:test-gap:detect",
            "morphism:test-gap:command-detect-to-runner",
            "law:test-gap:command-routes-to-runner",
        ],
    },
    HgRustTestBindingRule {
        trigger_terms: &["--format", "json"],
        cli_label: None,
        target_ids: &["law:test-gap:json-format-required"],
    },
    HgRustTestBindingRule {
        trigger_terms: &["highergraphen.test_gap.input.v1"],
        cli_label: None,
        target_ids: &["schema:test-gap:input", "law:test-gap:schema-id-preserved"],
    },
    HgRustTestBindingRule {
        trigger_terms: &["schema"],
        cli_label: None,
        target_ids: &["schema:test-gap:input", "law:test-gap:schema-id-preserved"],
    },
];

fn push_rust_test_content_cells(
    content_model: &mut RustTestContentModel,
    target_model: &StructuralModel,
    change: &GitChange,
    revision: SemanticRevision,
    source_path: &str,
    contents: &str,
    diff_evidence_id: &Id,
) -> Result<(), String> {
    let Some(document) = extract_rust_test_semantics(contents) else {
        return Ok(());
    };
    if document.functions.is_empty() {
        return Ok(());
    }
    content_model.test_files.insert(change.path.clone());
    let file_cell_id = id(format!(
        "semantic:rust-test:file:{}:{}",
        slug(&change.path),
        revision.as_str()
    ))?;
    push_higher_order_cell(
        &mut content_model.structural,
        file_cell_id.clone(),
        "rust_test_file_revision",
        format!(
            "Rust test file {} at {} from {}",
            change.path,
            revision.as_str(),
            source_path
        ),
        0,
        change,
        diff_evidence_id,
        0.74,
    )?;

    for function in &document.functions {
        let function_slug = slug(&function.name);
        let function_id = id(format!(
            "semantic:rust-test:function:{}:{}:{}",
            slug(&change.path),
            revision.as_str(),
            function_slug
        ))?;
        push_higher_order_cell(
            &mut content_model.structural,
            function_id.clone(),
            "rust_test_function",
            format!("Rust test function {}", function.name),
            0,
            change,
            diff_evidence_id,
            0.76,
        )?;
        push_higher_order_incidence(
            &mut content_model.structural,
            format!(
                "incidence:semantic:rust-test:file-contains-test:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                function_slug
            ),
            file_cell_id.clone(),
            function_id.clone(),
            "contains_test_function",
            diff_evidence_id,
            0.76,
        )?;

        let mut evidence_cell_ids = vec![function_id.clone()];
        for (index, macro_name) in function.assertion_macros.iter().enumerate() {
            let assertion_number = index + 1;
            let assertion_id = id(format!(
                "semantic:rust-test:assertion:{}:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                function_slug,
                assertion_number
            ))?;
            push_higher_order_cell(
                &mut content_model.structural,
                assertion_id.clone(),
                "rust_test_assertion",
                format!("Rust test assertion {macro_name}! in {}", function.name),
                0,
                change,
                diff_evidence_id,
                0.72,
            )?;
            push_higher_order_incidence(
                &mut content_model.structural,
                format!(
                    "incidence:semantic:rust-test:test-contains-assertion:{}:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    function_slug,
                    assertion_number
                ),
                function_id.clone(),
                assertion_id.clone(),
                "contains_assertion",
                diff_evidence_id,
                0.72,
            )?;
            evidence_cell_ids.push(assertion_id);
        }

        for observation in &function.cli_observations {
            let observation_label =
                hg_cli_observation_label(&observation.tokens, &observation.label);
            let cli_id = id(format!(
                "semantic:rust-test:cli-invocation:{}:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                function_slug,
                slug(&observation_label)
            ))?;
            push_higher_order_cell(
                &mut content_model.structural,
                cli_id.clone(),
                "rust_test_cli_invocation",
                format!("Rust test observes CLI invocation {observation_label}"),
                0,
                change,
                diff_evidence_id,
                0.74,
            )?;
            push_higher_order_incidence(
                &mut content_model.structural,
                format!(
                    "incidence:semantic:rust-test:test-observes-cli:{}:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    function_slug,
                    slug(&observation_label)
                ),
                function_id.clone(),
                cli_id.clone(),
                "observes_cli_invocation",
                diff_evidence_id,
                0.74,
            )?;
            evidence_cell_ids.push(cli_id);
        }

        for observation in &function.json_observations {
            let observation_id = id(format!(
                "semantic:rust-test:json-observation:{}:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                function_slug,
                slug(&observation.label)
            ))?;
            push_higher_order_cell(
                &mut content_model.structural,
                observation_id.clone(),
                "rust_test_json_observation",
                format!("Rust test observes JSON {}", observation.label),
                0,
                change,
                diff_evidence_id,
                0.74,
            )?;
            push_higher_order_incidence(
                &mut content_model.structural,
                format!(
                    "incidence:semantic:rust-test:test-observes-json:{}:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    function_slug,
                    slug(&observation.label)
                ),
                function_id.clone(),
                observation_id.clone(),
                "observes_json_contract",
                diff_evidence_id,
                0.74,
            )?;
            evidence_cell_ids.push(observation_id);
        }

        let target_ids = hg_rust_test_content_target_ids(target_model, &function.string_literals)?;
        for target_id in target_ids {
            push_unique_id(
                content_model
                    .target_ids_by_file
                    .entry(change.path.clone())
                    .or_default(),
                target_id.clone(),
            );
            push_rust_test_content_morphism(
                &mut content_model.structural,
                target_model,
                change,
                &function_slug,
                &evidence_cell_ids,
                target_id,
            )?;
        }
    }
    Ok(())
}

fn hg_cli_observation_label(tokens: &[String], fallback: &str) -> String {
    for rule in HG_RUST_TEST_BINDING_RULES {
        if rule
            .cli_label
            .is_some_and(|_| contains_all_tokens(tokens, rule.trigger_terms))
        {
            return rule.cli_label.expect("checked label").to_owned();
        }
    }
    fallback.to_owned()
}

fn contains_all_tokens(tokens: &[String], expected: &[&str]) -> bool {
    expected
        .iter()
        .all(|value| tokens.iter().any(|token| token == value))
}

fn hg_rust_test_content_target_ids(
    target_model: &StructuralModel,
    strings: &BTreeSet<String>,
) -> Result<Vec<Id>, String> {
    let mut target_ids = Vec::new();
    for value in strings {
        push_model_id_if_present(target_model, &mut target_ids, value)?;
    }

    for rule in HG_RUST_TEST_BINDING_RULES {
        if contains_all_strings(strings, rule.trigger_terms) {
            for target_id in rule.target_ids {
                push_model_id_if_present(target_model, &mut target_ids, target_id)?;
            }
        }
    }
    Ok(target_ids)
}

fn push_rust_test_content_morphism(
    model: &mut StructuralModel,
    target_model: &StructuralModel,
    change: &GitChange,
    function_slug: &str,
    evidence_cell_ids: &[Id],
    target_id: Id,
) -> Result<(), String> {
    let morphism_id = id(format!(
        "morphism:test-gap:rust-test-content:{}:{}:{}",
        slug(&change.path),
        function_slug,
        slug(target_id.as_str())
    ))?;
    if model
        .morphisms
        .iter()
        .any(|morphism| morphism.id == morphism_id)
    {
        return Ok(());
    }
    let law_ids = law_ids_for_content_target(target_model, &target_id);
    model.morphisms.push(TestGapInputMorphism {
        id: morphism_id.clone(),
        morphism_type: "rust_test_content_evidence".to_owned(),
        source_ids: evidence_cell_ids.to_vec(),
        target_ids: vec![target_id.clone()],
        law_ids,
        requirement_ids: Vec::new(),
        expected_verification: None,
        confidence: Some(confidence(0.68)?),
    });
    Ok(())
}

fn law_ids_for_content_target(target_model: &StructuralModel, target_id: &Id) -> Vec<Id> {
    if target_model.laws.iter().any(|law| &law.id == target_id) {
        return vec![target_id.clone()];
    }
    target_model
        .morphisms
        .iter()
        .find(|morphism| &morphism.id == target_id)
        .map(|morphism| morphism.law_ids.clone())
        .unwrap_or_default()
}

fn push_model_id_if_present(
    target_model: &StructuralModel,
    target_ids: &mut Vec<Id>,
    candidate: &str,
) -> Result<(), String> {
    let Ok(candidate_id) = id(candidate) else {
        return Ok(());
    };
    if model_contains_id(target_model, &candidate_id) {
        push_unique_id(target_ids, candidate_id);
    }
    Ok(())
}

fn model_contains_id(model: &StructuralModel, candidate_id: &Id) -> bool {
    model
        .symbols
        .iter()
        .any(|symbol| &symbol.id == candidate_id)
        || model.laws.iter().any(|law| &law.id == candidate_id)
        || model
            .morphisms
            .iter()
            .any(|morphism| &morphism.id == candidate_id)
        || model
            .higher_order_cells
            .iter()
            .any(|cell| &cell.id == candidate_id)
}

fn push_rust_semantic_cells(
    model: &mut StructuralModel,
    change: &GitChange,
    revision: SemanticRevision,
    source_path: &str,
    contents: &str,
    diff_evidence_id: &Id,
) -> Result<Vec<SemanticCell>, String> {
    let Ok(file) = syn::parse_file(contents) else {
        return Ok(Vec::new());
    };
    let mut semantic_cells = Vec::new();
    let file_cell_id = id(format!(
        "semantic:rust:file:{}:{}",
        slug(&change.path),
        revision.as_str()
    ))?;
    push_higher_order_cell(
        model,
        file_cell_id.clone(),
        "rust_file_revision",
        format!(
            "Rust semantic file {} at {} from {}",
            change.path,
            revision.as_str(),
            source_path
        ),
        0,
        change,
        diff_evidence_id,
        0.7,
    )?;
    semantic_cells.push(SemanticCell {
        id: file_cell_id.clone(),
        key: format!("rust:file:{}", slug(&change.path)),
        cell_type: "rust_file_revision".to_owned(),
    });

    for item in &file.items {
        match item {
            syn::Item::Fn(item_fn) => {
                let function_id = id(format!(
                    "semantic:rust:function:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    slug(&item_fn.sig.ident.to_string())
                ))?;
                push_higher_order_cell(
                    model,
                    function_id.clone(),
                    "rust_function",
                    format!("Rust function {}", item_fn.sig.ident),
                    0,
                    change,
                    diff_evidence_id,
                    0.72,
                )?;
                semantic_cells.push(SemanticCell {
                    id: function_id.clone(),
                    key: format!(
                        "rust:function:{}:{}",
                        slug(&change.path),
                        slug(&item_fn.sig.ident.to_string())
                    ),
                    cell_type: "rust_function".to_owned(),
                });
                push_higher_order_incidence(
                    model,
                    format!(
                        "incidence:semantic:rust:file-contains-function:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        slug(&item_fn.sig.ident.to_string())
                    ),
                    file_cell_id.clone(),
                    function_id.clone(),
                    "contains_function",
                    diff_evidence_id,
                    0.72,
                )?;
                let mut visitor = RustSemanticVisitor::new(
                    change,
                    revision,
                    &function_id,
                    &slug(&item_fn.sig.ident.to_string()),
                    diff_evidence_id,
                );
                visitor.visit_block(&item_fn.block);
                semantic_cells.extend(visitor.semantic_cells);
                model.extend(visitor.model);
            }
            syn::Item::Struct(item_struct) => {
                let struct_id = id(format!(
                    "semantic:rust:struct:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    slug(&item_struct.ident.to_string())
                ))?;
                push_higher_order_cell(
                    model,
                    struct_id.clone(),
                    "rust_struct",
                    format!("Rust struct {}", item_struct.ident),
                    0,
                    change,
                    diff_evidence_id,
                    0.72,
                )?;
                semantic_cells.push(SemanticCell {
                    id: struct_id.clone(),
                    key: format!(
                        "rust:struct:{}:{}",
                        slug(&change.path),
                        slug(&item_struct.ident.to_string())
                    ),
                    cell_type: "rust_struct".to_owned(),
                });
                push_higher_order_incidence(
                    model,
                    format!(
                        "incidence:semantic:rust:file-contains-struct:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        slug(&item_struct.ident.to_string())
                    ),
                    file_cell_id.clone(),
                    struct_id,
                    "contains_struct",
                    diff_evidence_id,
                    0.72,
                )?;
            }
            syn::Item::Enum(item_enum) => {
                let enum_id = id(format!(
                    "semantic:rust:enum:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    slug(&item_enum.ident.to_string())
                ))?;
                push_higher_order_cell(
                    model,
                    enum_id.clone(),
                    "rust_enum",
                    format!("Rust enum {}", item_enum.ident),
                    0,
                    change,
                    diff_evidence_id,
                    0.72,
                )?;
                semantic_cells.push(SemanticCell {
                    id: enum_id.clone(),
                    key: format!(
                        "rust:enum:{}:{}",
                        slug(&change.path),
                        slug(&item_enum.ident.to_string())
                    ),
                    cell_type: "rust_enum".to_owned(),
                });
                push_higher_order_incidence(
                    model,
                    format!(
                        "incidence:semantic:rust:file-contains-enum:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        slug(&item_enum.ident.to_string())
                    ),
                    file_cell_id.clone(),
                    enum_id.clone(),
                    "contains_enum",
                    diff_evidence_id,
                    0.72,
                )?;
                for variant in &item_enum.variants {
                    let variant_id = id(format!(
                        "semantic:rust:enum-variant:{}:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        slug(&item_enum.ident.to_string()),
                        slug(&variant.ident.to_string())
                    ))?;
                    push_higher_order_cell(
                        model,
                        variant_id.clone(),
                        "rust_enum_variant",
                        format!("Rust enum variant {}::{}", item_enum.ident, variant.ident),
                        0,
                        change,
                        diff_evidence_id,
                        0.7,
                    )?;
                    semantic_cells.push(SemanticCell {
                        id: variant_id.clone(),
                        key: format!(
                            "rust:enum-variant:{}:{}:{}",
                            slug(&change.path),
                            slug(&item_enum.ident.to_string()),
                            slug(&variant.ident.to_string())
                        ),
                        cell_type: "rust_enum_variant".to_owned(),
                    });
                    push_higher_order_incidence(
                        model,
                        format!(
                            "incidence:semantic:rust:enum-contains-variant:{}:{}:{}:{}",
                            slug(&change.path),
                            revision.as_str(),
                            slug(&item_enum.ident.to_string()),
                            slug(&variant.ident.to_string())
                        ),
                        enum_id.clone(),
                        variant_id,
                        "contains_variant",
                        diff_evidence_id,
                        0.7,
                    )?;
                }
            }
            syn::Item::Impl(item_impl) => {
                let impl_index = semantic_cells
                    .iter()
                    .filter(|cell| cell.cell_type == "rust_impl")
                    .count()
                    + 1;
                let impl_id = id(format!(
                    "semantic:rust:impl:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    impl_index
                ))?;
                push_higher_order_cell(
                    model,
                    impl_id.clone(),
                    "rust_impl",
                    format!("Rust impl block {impl_index}"),
                    0,
                    change,
                    diff_evidence_id,
                    0.68,
                )?;
                semantic_cells.push(SemanticCell {
                    id: impl_id.clone(),
                    key: format!("rust:impl:{}:{}", slug(&change.path), impl_index),
                    cell_type: "rust_impl".to_owned(),
                });
                push_higher_order_incidence(
                    model,
                    format!(
                        "incidence:semantic:rust:file-contains-impl:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        impl_index
                    ),
                    file_cell_id.clone(),
                    impl_id.clone(),
                    "contains_impl",
                    diff_evidence_id,
                    0.68,
                )?;
                for impl_item in &item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        let method_id = id(format!(
                            "semantic:rust:method:{}:{}:{}:{}",
                            slug(&change.path),
                            revision.as_str(),
                            impl_index,
                            slug(&method.sig.ident.to_string())
                        ))?;
                        push_higher_order_cell(
                            model,
                            method_id.clone(),
                            "rust_method",
                            format!("Rust method {}", method.sig.ident),
                            0,
                            change,
                            diff_evidence_id,
                            0.7,
                        )?;
                        semantic_cells.push(SemanticCell {
                            id: method_id.clone(),
                            key: format!(
                                "rust:method:{}:{}:{}",
                                slug(&change.path),
                                impl_index,
                                slug(&method.sig.ident.to_string())
                            ),
                            cell_type: "rust_method".to_owned(),
                        });
                        push_higher_order_incidence(
                            model,
                            format!(
                                "incidence:semantic:rust:impl-contains-method:{}:{}:{}:{}",
                                slug(&change.path),
                                revision.as_str(),
                                impl_index,
                                slug(&method.sig.ident.to_string())
                            ),
                            impl_id.clone(),
                            method_id.clone(),
                            "contains_method",
                            diff_evidence_id,
                            0.7,
                        )?;
                        let mut visitor = RustSemanticVisitor::new(
                            change,
                            revision,
                            &method_id,
                            &format!("impl-{impl_index}-{}", slug(&method.sig.ident.to_string())),
                            diff_evidence_id,
                        );
                        visitor.visit_block(&method.block);
                        semantic_cells.extend(visitor.semantic_cells);
                        model.extend(visitor.model);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(semantic_cells)
}

fn push_json_schema_semantic_cells(
    model: &mut StructuralModel,
    change: &GitChange,
    revision: SemanticRevision,
    source_path: &str,
    contents: &str,
    diff_evidence_id: &Id,
) -> Result<Vec<SemanticCell>, String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(contents) else {
        return Ok(Vec::new());
    };
    let schema_id = id(format!(
        "semantic:json-schema:document:{}:{}",
        slug(&change.path),
        revision.as_str()
    ))?;
    push_higher_order_cell(
        model,
        schema_id.clone(),
        "json_schema_revision",
        format!(
            "JSON schema {} at {} from {}",
            change.path,
            revision.as_str(),
            source_path
        ),
        0,
        change,
        diff_evidence_id,
        0.74,
    )?;
    let mut semantic_cells = vec![SemanticCell {
        id: schema_id.clone(),
        key: format!("json-schema:document:{}", slug(&change.path)),
        cell_type: "json_schema_revision".to_owned(),
    }];
    push_json_schema_properties(
        model,
        change,
        revision,
        &schema_id,
        "#".to_owned(),
        &value,
        diff_evidence_id,
        &mut semantic_cells,
    )?;
    Ok(semantic_cells)
}

fn push_json_schema_properties(
    model: &mut StructuralModel,
    change: &GitChange,
    revision: SemanticRevision,
    schema_id: &Id,
    pointer: String,
    value: &serde_json::Value,
    diff_evidence_id: &Id,
    semantic_cells: &mut Vec<SemanticCell>,
) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    if let Some(properties) = object.get("properties").and_then(|value| value.as_object()) {
        for property_name in properties.keys() {
            let property_id = id(format!(
                "semantic:json-schema:property:{}:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                slug(&pointer),
                slug(property_name)
            ))?;
            push_higher_order_cell(
                model,
                property_id.clone(),
                "json_schema_property",
                format!("JSON schema property {pointer}/{property_name}"),
                0,
                change,
                diff_evidence_id,
                0.74,
            )?;
            semantic_cells.push(SemanticCell {
                id: property_id.clone(),
                key: format!(
                    "json-schema:property:{}:{}:{}",
                    slug(&change.path),
                    slug(&pointer),
                    slug(property_name)
                ),
                cell_type: "json_schema_property".to_owned(),
            });
            push_higher_order_incidence(
                model,
                format!(
                    "incidence:semantic:json-schema:declares-property:{}:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    slug(&pointer),
                    slug(property_name)
                ),
                schema_id.clone(),
                property_id,
                "declares_property",
                diff_evidence_id,
                0.74,
            )?;
        }
    }
    if let Some(defs) = object.get("$defs").and_then(|value| value.as_object()) {
        for (def_name, def_value) in defs {
            push_json_schema_properties(
                model,
                change,
                revision,
                schema_id,
                format!("{pointer}/$defs/{def_name}"),
                def_value,
                diff_evidence_id,
                semantic_cells,
            )?;
        }
    }
    Ok(())
}

fn push_semantic_delta_structure(
    model: &mut StructuralModel,
    change: &GitChange,
    base_cells: &[SemanticCell],
    head_cells: &[SemanticCell],
    diff_evidence_id: &Id,
) -> Result<(), String> {
    if base_cells.is_empty() && head_cells.is_empty() {
        return Ok(());
    }
    push_semantic_law(
        model,
        "law:test-gap:semantic-delta-is-explicit",
        "base/head semantic cells expose preservation, addition, and deletion morphisms",
        diff_evidence_id,
    )?;
    push_semantic_law(
        model,
        "law:test-gap:semantic-delta-has-verification",
        "changed semantic delta morphisms require accepted verification cells",
        diff_evidence_id,
    )?;

    let base_by_key = base_cells
        .iter()
        .map(|cell| (cell.key.as_str(), cell))
        .collect::<BTreeMap<_, _>>();
    let head_by_key = head_cells
        .iter()
        .map(|cell| (cell.key.as_str(), cell))
        .collect::<BTreeMap<_, _>>();
    for (key, base_cell) in &base_by_key {
        if let Some(head_cell) = head_by_key.get(key) {
            push_semantic_morphism(
                model,
                change,
                "semantic_preservation",
                base_cell,
                Some(head_cell),
                &[
                    "law:test-gap:semantic-delta-is-explicit",
                    "law:test-gap:semantic-delta-has-verification",
                ],
                diff_evidence_id,
            )?;
        } else {
            push_semantic_morphism(
                model,
                change,
                "semantic_deletion",
                base_cell,
                None,
                &[
                    "law:test-gap:semantic-delta-is-explicit",
                    "law:test-gap:semantic-delta-has-verification",
                ],
                diff_evidence_id,
            )?;
        }
    }
    for (key, head_cell) in &head_by_key {
        if !base_by_key.contains_key(key) {
            push_semantic_morphism(
                model,
                change,
                "semantic_addition",
                head_cell,
                None,
                &[
                    "law:test-gap:semantic-delta-is-explicit",
                    "law:test-gap:semantic-delta-has-verification",
                ],
                diff_evidence_id,
            )?;
        }
    }
    Ok(())
}

fn push_semantic_law(
    model: &mut StructuralModel,
    law_id: &str,
    summary: &str,
    diff_evidence_id: &Id,
) -> Result<(), String> {
    let law_id = id(law_id)?;
    if model.laws.iter().any(|law| law.id == law_id) {
        return Ok(());
    }
    model.laws.push(TestGapInputLaw {
        id: law_id,
        summary: summary.to_owned(),
        applies_to_ids: Vec::new(),
        requirement_ids: Vec::new(),
        source_ids: vec![diff_evidence_id.clone()],
        expected_verification: Some("policy_accepted_verification".to_owned()),
        confidence: Some(confidence(0.76)?),
    });
    Ok(())
}

fn push_semantic_morphism(
    model: &mut StructuralModel,
    change: &GitChange,
    morphism_type: &str,
    source_cell: &SemanticCell,
    target_cell: Option<&SemanticCell>,
    law_ids: &[&str],
    _diff_evidence_id: &Id,
) -> Result<(), String> {
    let morphism_id = id(format!(
        "morphism:test-gap:{}:{}:{}",
        morphism_type,
        slug(&change.path),
        slug(&source_cell.key)
    ))?;
    if model
        .morphisms
        .iter()
        .any(|morphism| morphism.id == morphism_id)
    {
        return Ok(());
    }
    let mut source_ids = vec![source_cell.id.clone()];
    let target_ids = if let Some(target_cell) = target_cell {
        vec![target_cell.id.clone()]
    } else if morphism_type == "semantic_addition" {
        source_ids = vec![file_id(&change.path)?];
        vec![source_cell.id.clone()]
    } else {
        vec![file_id(&change.path)?]
    };
    let law_ids = law_ids
        .iter()
        .filter_map(|law_id| id(*law_id).ok())
        .filter(|law_id| model.laws.iter().any(|law| &law.id == law_id))
        .collect::<Vec<_>>();
    model.morphisms.push(TestGapInputMorphism {
        id: morphism_id.clone(),
        morphism_type: morphism_type.to_owned(),
        source_ids,
        target_ids,
        law_ids: law_ids.clone(),
        requirement_ids: Vec::new(),
        expected_verification: semantic_delta_expected_verification(change),
        confidence: Some(confidence(0.7)?),
    });
    for law_id in law_ids {
        if let Some(law) = model.laws.iter_mut().find(|law| law.id == law_id) {
            push_unique_id(&mut law.applies_to_ids, morphism_id.clone());
        }
    }
    Ok(())
}

fn semantic_delta_expected_verification(change: &GitChange) -> Option<String> {
    if matches!(
        change.path.as_str(),
        "tools/highergraphen-cli/src/semantic_proof_artifact.rs"
            | "tools/highergraphen-cli/src/semantic_proof_backend.rs"
            | "tools/highergraphen-cli/src/semantic_proof_reinput.rs"
    ) {
        None
    } else {
        Some("policy_accepted_verification".to_owned())
    }
}

struct RustSemanticVisitor<'a> {
    change: &'a GitChange,
    revision: SemanticRevision,
    parent_id: &'a Id,
    parent_slug: String,
    diff_evidence_id: &'a Id,
    model: StructuralModel,
    semantic_cells: Vec<SemanticCell>,
    match_index: usize,
    error_index: usize,
}

impl<'a> RustSemanticVisitor<'a> {
    fn new(
        change: &'a GitChange,
        revision: SemanticRevision,
        parent_id: &'a Id,
        parent_slug: &str,
        diff_evidence_id: &'a Id,
    ) -> Self {
        Self {
            change,
            revision,
            parent_id,
            parent_slug: parent_slug.to_owned(),
            diff_evidence_id,
            model: StructuralModel::default(),
            semantic_cells: Vec::new(),
            match_index: 0,
            error_index: 0,
        }
    }
}

impl Visit<'_> for RustSemanticVisitor<'_> {
    fn visit_expr_match(&mut self, node: &syn::ExprMatch) {
        self.match_index += 1;
        let match_id = match id(format!(
            "semantic:rust:match:{}:{}:{}:{}",
            slug(&self.change.path),
            self.revision.as_str(),
            self.parent_slug,
            self.match_index
        )) {
            Ok(value) => value,
            Err(_) => return,
        };
        if push_higher_order_cell(
            &mut self.model,
            match_id.clone(),
            "rust_match",
            format!("Rust match expression {}", self.match_index),
            0,
            self.change,
            self.diff_evidence_id,
            0.68,
        )
        .is_err()
        {
            return;
        }
        self.semantic_cells.push(SemanticCell {
            id: match_id.clone(),
            key: format!(
                "rust:match:{}:{}:{}",
                slug(&self.change.path),
                self.parent_slug,
                self.match_index
            ),
            cell_type: "rust_match".to_owned(),
        });
        let _ = push_higher_order_incidence(
            &mut self.model,
            format!(
                "incidence:semantic:rust:function-contains-match:{}:{}:{}:{}",
                slug(&self.change.path),
                self.revision.as_str(),
                self.parent_slug,
                self.match_index
            ),
            self.parent_id.clone(),
            match_id.clone(),
            "contains_match",
            self.diff_evidence_id,
            0.68,
        );
        for (arm_index, _) in node.arms.iter().enumerate() {
            let arm_number = arm_index + 1;
            let Ok(arm_id) = id(format!(
                "semantic:rust:match-arm:{}:{}:{}:{}:{}",
                slug(&self.change.path),
                self.revision.as_str(),
                self.parent_slug,
                self.match_index,
                arm_number
            )) else {
                continue;
            };
            let _ = push_higher_order_cell(
                &mut self.model,
                arm_id.clone(),
                "rust_match_arm",
                format!("Rust match arm {arm_number}"),
                0,
                self.change,
                self.diff_evidence_id,
                0.66,
            );
            self.semantic_cells.push(SemanticCell {
                id: arm_id.clone(),
                key: format!(
                    "rust:match-arm:{}:{}:{}:{}",
                    slug(&self.change.path),
                    self.parent_slug,
                    self.match_index,
                    arm_number
                ),
                cell_type: "rust_match_arm".to_owned(),
            });
            let _ = push_higher_order_incidence(
                &mut self.model,
                format!(
                    "incidence:semantic:rust:match-contains-arm:{}:{}:{}:{}:{}",
                    slug(&self.change.path),
                    self.revision.as_str(),
                    self.parent_slug,
                    self.match_index,
                    arm_number
                ),
                match_id.clone(),
                arm_id,
                "contains_match_arm",
                self.diff_evidence_id,
                0.66,
            );
        }
        syn::visit::visit_expr_match(self, node);
    }

    fn visit_expr_call(&mut self, node: &syn::ExprCall) {
        if semantic_expr_mentions_error_path(&node.func) {
            self.push_error_path_cell();
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_macro(&mut self, node: &syn::ExprMacro) {
        let macro_name = node
            .mac
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();
        if matches!(macro_name.as_str(), "panic" | "todo" | "unimplemented") {
            self.push_error_path_cell();
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        if matches!(node.method.to_string().as_str(), "unwrap" | "expect") {
            self.push_error_path_cell();
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

impl RustSemanticVisitor<'_> {
    fn push_error_path_cell(&mut self) {
        self.error_index += 1;
        let Ok(error_id) = id(format!(
            "semantic:rust:error-path:{}:{}:{}:{}",
            slug(&self.change.path),
            self.revision.as_str(),
            self.parent_slug,
            self.error_index
        )) else {
            return;
        };
        let _ = push_higher_order_cell(
            &mut self.model,
            error_id.clone(),
            "rust_error_path",
            format!("Rust error path {}", self.error_index),
            0,
            self.change,
            self.diff_evidence_id,
            0.64,
        );
        self.semantic_cells.push(SemanticCell {
            id: error_id.clone(),
            key: format!(
                "rust:error-path:{}:{}:{}",
                slug(&self.change.path),
                self.parent_slug,
                self.error_index
            ),
            cell_type: "rust_error_path".to_owned(),
        });
        let _ = push_higher_order_incidence(
            &mut self.model,
            format!(
                "incidence:semantic:rust:function-contains-error-path:{}:{}:{}:{}",
                slug(&self.change.path),
                self.revision.as_str(),
                self.parent_slug,
                self.error_index
            ),
            self.parent_id.clone(),
            error_id,
            "contains_error_path",
            self.diff_evidence_id,
            0.64,
        );
    }
}

fn semantic_expr_mentions_error_path(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| matches!(segment.ident.to_string().as_str(), "panic"))
            .unwrap_or(false),
        _ => false,
    }
}

fn push_higher_order_cell(
    model: &mut StructuralModel,
    cell_id: Id,
    cell_type: &str,
    label: String,
    dimension: u32,
    change: &GitChange,
    diff_evidence_id: &Id,
    confidence_value: f64,
) -> Result<(), String> {
    if model
        .higher_order_cells
        .iter()
        .any(|cell| cell.id == cell_id)
    {
        return Ok(());
    }
    model.higher_order_cells.push(TestGapHigherOrderCell {
        id: cell_id,
        cell_type: cell_type.to_owned(),
        label,
        dimension,
        context_ids: test_gap_context_ids_for_path(&change.path)?,
        source_ids: vec![diff_evidence_id.clone()],
        confidence: Some(confidence(confidence_value)?),
    });
    Ok(())
}

fn push_higher_order_incidence(
    model: &mut StructuralModel,
    incidence_id: String,
    from_id: Id,
    to_id: Id,
    relation_type: &str,
    diff_evidence_id: &Id,
    confidence_value: f64,
) -> Result<(), String> {
    let incidence_id = id(incidence_id)?;
    if model
        .higher_order_incidences
        .iter()
        .any(|incidence| incidence.id == incidence_id)
    {
        return Ok(());
    }
    model
        .higher_order_incidences
        .push(TestGapHigherOrderIncidence {
            id: incidence_id,
            from_id,
            to_id,
            relation_type: relation_type.to_owned(),
            orientation: None,
            source_ids: vec![diff_evidence_id.clone()],
            confidence: Some(confidence(confidence_value)?),
        });
    Ok(())
}

fn push_structural_symbol(
    model: &mut StructuralModel,
    changes: &[GitChange],
    diff_evidence_id: &Id,
    file_path: &str,
    symbol_id: &str,
    name: &str,
    path: &str,
    kind: TestGapSymbolKind,
) -> Result<(), String> {
    if !changes.iter().any(|change| change.path == file_path) {
        return Ok(());
    }
    model.symbols.push(TestGapInputSymbol {
        id: id(symbol_id)?,
        file_id: file_id(file_path)?,
        name: name.to_owned(),
        kind,
        visibility: TestGapVisibility::Public,
        public_api: true,
        path: Some(path.to_owned()),
        line_start: None,
        line_end: None,
        branch_ids: Vec::new(),
        requirement_ids: Vec::new(),
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
    });
    model.higher_order_cells.push(TestGapHigherOrderCell {
        id: id(symbol_id)?,
        cell_type: symbol_id
            .split(':')
            .next()
            .unwrap_or("structure")
            .to_owned(),
        label: name.to_owned(),
        dimension: 0,
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
        confidence: Some(confidence(0.8)?),
    });
    Ok(())
}

fn push_structural_symbol_unconditional(
    model: &mut StructuralModel,
    diff_evidence_id: &Id,
    file_path: &str,
    symbol_id: &str,
    name: &str,
    path: &str,
    kind: TestGapSymbolKind,
) -> Result<(), String> {
    model.symbols.push(TestGapInputSymbol {
        id: id(symbol_id)?,
        file_id: file_id(file_path)?,
        name: name.to_owned(),
        kind,
        visibility: TestGapVisibility::Public,
        public_api: true,
        path: Some(path.to_owned()),
        line_start: None,
        line_end: None,
        branch_ids: Vec::new(),
        requirement_ids: Vec::new(),
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
    });
    model.higher_order_cells.push(TestGapHigherOrderCell {
        id: id(symbol_id)?,
        cell_type: symbol_id
            .split(':')
            .next()
            .unwrap_or("structure")
            .to_owned(),
        label: name.to_owned(),
        dimension: 0,
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
        confidence: Some(confidence(0.8)?),
    });
    Ok(())
}

fn push_law_symbol(
    model: &mut StructuralModel,
    changes: &[GitChange],
    diff_evidence_id: &Id,
    file_paths: &[&str],
    symbol_id: &str,
    name: &str,
) -> Result<(), String> {
    let Some(file_path) = file_paths
        .iter()
        .find(|file_path| changes.iter().any(|change| change.path == **file_path))
    else {
        return Ok(());
    };
    model.symbols.push(TestGapInputSymbol {
        id: id(symbol_id)?,
        file_id: file_id(file_path)?,
        name: name.to_owned(),
        kind: TestGapSymbolKind::Unknown,
        visibility: TestGapVisibility::Public,
        public_api: true,
        path: Some(symbol_id.to_owned()),
        line_start: None,
        line_end: None,
        branch_ids: Vec::new(),
        requirement_ids: Vec::new(),
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
    });
    model.laws.push(TestGapInputLaw {
        id: id(symbol_id)?,
        summary: name.to_owned(),
        applies_to_ids: Vec::new(),
        requirement_ids: Vec::new(),
        source_ids: vec![diff_evidence_id.clone()],
        expected_verification: Some("policy_accepted_verification".to_owned()),
        confidence: Some(confidence(0.82)?),
    });
    Ok(())
}

fn push_structural_edge(
    model: &mut StructuralModel,
    edge_id: &str,
    from_id: &str,
    to_id: &str,
    relation_type: TestGapDependencyRelationType,
    diff_evidence_id: &Id,
) -> Result<(), String> {
    let from_id = id(from_id)?;
    let to_id = id(to_id)?;
    if !has_structural_symbol(&model.symbols, &from_id)
        || !has_structural_symbol(&model.symbols, &to_id)
    {
        return Ok(());
    }
    model.dependency_edges.push(TestGapInputDependencyEdge {
        id: id(edge_id)?,
        from_id: from_id.clone(),
        to_id: to_id.clone(),
        relation_type,
        orientation: None,
        source_ids: vec![diff_evidence_id.clone()],
        confidence: Some(confidence(0.78)?),
    });
    model
        .higher_order_incidences
        .push(TestGapHigherOrderIncidence {
            id: id(format!("incidence:{edge_id}"))?,
            from_id,
            to_id,
            relation_type: serde_plain_dependency_relation_type(relation_type),
            orientation: None,
            source_ids: vec![diff_evidence_id.clone()],
            confidence: Some(confidence(0.78)?),
        });
    Ok(())
}

fn push_higher_order_morphism(
    model: &mut StructuralModel,
    morphism_id: &str,
    morphism_type: &str,
    source_ids: &[&str],
    target_ids: &[&str],
    law_ids: &[&str],
    _diff_evidence_id: &Id,
) -> Result<(), String> {
    let source_ids = ids_present_in_model(model, source_ids)?;
    let target_ids = ids_present_in_model(model, target_ids)?;
    if source_ids.is_empty() || target_ids.is_empty() {
        return Ok(());
    }
    let law_ids = ids_present_in_model(model, law_ids)?;
    let morphism_id = id(morphism_id)?;
    model.morphisms.push(TestGapInputMorphism {
        id: morphism_id.clone(),
        morphism_type: morphism_type.to_owned(),
        source_ids,
        target_ids,
        law_ids: law_ids.clone(),
        requirement_ids: Vec::new(),
        expected_verification: Some("policy_accepted_verification".to_owned()),
        confidence: Some(confidence(0.8)?),
    });
    for law_id in law_ids {
        if let Some(law) = model.laws.iter_mut().find(|law| law.id == law_id) {
            push_unique_id(&mut law.applies_to_ids, morphism_id.clone());
        }
    }
    Ok(())
}

fn ids_present_in_model(model: &StructuralModel, ids: &[&str]) -> Result<Vec<Id>, String> {
    ids.iter()
        .filter(|value| {
            model
                .higher_order_cells
                .iter()
                .any(|cell| cell.id.as_str() == **value)
                || model.laws.iter().any(|law| law.id.as_str() == **value)
        })
        .map(|value| id(*value))
        .collect()
}

fn serde_plain_dependency_relation_type(relation_type: TestGapDependencyRelationType) -> String {
    match relation_type {
        TestGapDependencyRelationType::Contains => "contains",
        TestGapDependencyRelationType::ImplementsRequirement => "implements_requirement",
        TestGapDependencyRelationType::HasBranch => "has_branch",
        TestGapDependencyRelationType::CoveredByTest => "covered_by_test",
        TestGapDependencyRelationType::ExercisesCondition => "exercises_condition",
        TestGapDependencyRelationType::DependsOn => "depends_on",
        TestGapDependencyRelationType::Supports => "supports",
        TestGapDependencyRelationType::InContext => "in_context",
        TestGapDependencyRelationType::Custom => "custom",
    }
    .to_owned()
}

fn has_structural_symbol(symbols: &[TestGapInputSymbol], symbol_id: &Id) -> bool {
    symbols.iter().any(|symbol| &symbol.id == symbol_id)
}

fn requirements_for_symbols(
    symbols: &[TestGapInputSymbol],
    diff_evidence_id: &Id,
    accepted_test_kinds: &[TestGapTestType],
) -> Result<Vec<TestGapInputRequirement>, String> {
    symbols
        .iter()
        .filter(|symbol| {
            symbol.id.as_str().ends_with(":changed-behavior")
                && !symbol
                    .path
                    .as_deref()
                    .is_some_and(is_highergraphen_structural_path)
        })
        .map(|symbol| {
            let requirement_id = id(format!(
                "requirement:{}:unit-verification",
                slug(symbol.path())
            ))?;
            Ok(TestGapInputRequirement {
                id: requirement_id,
                requirement_type: TestGapRequirementType::Custom,
                summary: format!(
                    "Changed behavior in {} has policy-accepted test verification",
                    symbol.path()
                ),
                in_scope: true,
                bug_fix: false,
                implementation_ids: vec![symbol.id.clone()],
                source_ids: vec![diff_evidence_id.clone(), symbol.file_id.clone()],
                expected_verification: Some(expected_verification_label(accepted_test_kinds)),
            })
        })
        .collect()
}

fn structural_requirements(
    structural: &StructuralModel,
    diff_evidence_id: &Id,
    accepted_test_kinds: &[TestGapTestType],
) -> Result<Vec<TestGapInputRequirement>, String> {
    let mut requirements = Vec::new();
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:command-detect-to-runner",
        "CLI command highergraphen test-gap detect preserves its morphism to run_test_gap_detect",
        &[
            "command:highergraphen:test-gap:detect",
            "runner:test-gap:detect",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:input-from-git-to-input-schema",
        "CLI command highergraphen test-gap input from-git routes through the git adapter and emits the bounded test-gap input schema",
        &[
            "command:highergraphen:test-gap:input-from-git",
            "adapter:test-gap:git-input",
            "schema:test-gap:input",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:input-from-path-to-input-schema",
        "CLI command highergraphen test-gap input from-path routes through the current-tree path adapter and emits the bounded test-gap input schema",
        &[
            "command:highergraphen:test-gap:input-from-path",
            "adapter:test-gap:path-input",
            "schema:test-gap:input",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:runtime-export-to-runner",
        "Runtime public exports preserve access to the test-gap detector runner and report types",
        &[
            "export:test-gap:runtime-api",
            "runner:test-gap:detect",
            "contract:test-gap:runtime-report-shapes",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:workflow-registry-to-runner",
        "Workflow module registry preserves the test-gap runner connection",
        &[
            "registry:test-gap:workflow-module",
            "runner:test-gap:detect",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:runtime-shapes-to-schemas",
        "Runtime TestGap shapes preserve the input and report schema contracts",
        &[
            "contract:test-gap:runtime-report-shapes",
            "schema:test-gap:input",
            "schema:test-gap:report",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:fixtures-to-schemas",
        "Checked-in test-gap fixtures preserve their input and report schema contracts",
        &[
            "fixture:test-gap:input-example",
            "fixture:test-gap:report-example",
            "schema:test-gap:input",
            "schema:test-gap:report",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:test-gap:report-envelope-to-runtime-shapes",
        "Report envelope projection preserves the TestGap runtime report shape boundary",
        &[
            "projection:test-gap:report-envelope",
            "contract:test-gap:runtime-report-shapes",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:backend-run-to-artifact",
        "semantic-proof backend run records local proof command output as bounded artifact material before HG verification",
        &[
            "command:highergraphen:semantic-proof:backend-run",
            "runner:semantic-proof:backend-run",
            "theorem:semantic-proof:backend-run-trust-boundary",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:artifact-to-input-document",
        "semantic-proof input from-artifact preserves backend artifacts as HG theorem, law, morphism, and certificate or counterexample input",
        &[
            "command:highergraphen:semantic-proof:input-from-artifact",
            "adapter:semantic-proof:artifact-input",
            "theorem:semantic-proof:artifact-adapter-correctness",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:certificate-to-proof-object",
        "proved semantic-proof artifacts roundtrip through verify as accepted proof objects",
        &[
            "adapter:semantic-proof:artifact-input",
            "command:highergraphen:semantic-proof:verify",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:counterexample-to-refutation",
        "counterexample semantic-proof artifacts roundtrip through verify as refutations",
        &[
            "adapter:semantic-proof:artifact-input",
            "command:highergraphen:semantic-proof:verify",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:insufficient-report-to-reinput",
        "insufficient semantic-proof reports requeue open law and morphism obligations as a new bounded input",
        &[
            "command:highergraphen:semantic-proof:input-from-report",
            "adapter:semantic-proof:reinput-from-report",
            "theorem:semantic-proof:obligation-reinput-correctness",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_structural_requirement(
        &mut requirements,
        structural,
        "requirement:morphism:semantic-proof:roundtrip-tests-to-adapter-correctness",
        "semantic-proof artifact roundtrip tests verify the adapter correctness theorem at the HG structure boundary",
        &[
            "test:semantic-proof:artifact-roundtrip",
            "test:semantic-proof:backend-and-reinput",
            "theorem:semantic-proof:artifact-adapter-correctness",
            "theorem:semantic-proof:backend-run-trust-boundary",
            "theorem:semantic-proof:obligation-reinput-correctness",
        ],
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:command-routes-to-runner",
        "requirement:law:test-gap:command-routes-to-runner",
        "CLI command parsing dispatches test-gap commands to the intended runtime runner or adapter",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:json-format-required",
        "requirement:law:test-gap:json-format-required",
        "test-gap CLI commands reject missing or unsupported non-JSON formats",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:output-file-suppresses-stdout",
        "requirement:law:test-gap:output-file-suppresses-stdout",
        "test-gap CLI --output writes to the requested file and suppresses JSON stdout",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:input-from-git-is-deterministic",
        "requirement:law:test-gap:input-from-git-is-deterministic",
        "test-gap input from-git emits deterministic bounded structure from the requested git range",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
        "requirement:law:test-gap:input-from-git-does-not-prove-semantic-coverage",
        "test-gap input from-git keeps semantic coverage limits visible instead of proving full coverage",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:input-from-path-is-deterministic",
        "requirement:law:test-gap:input-from-path-is-deterministic",
        "test-gap input from-path emits deterministic bounded structure from selected current-tree files",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:input-from-path-declares-snapshot-boundary",
        "requirement:law:test-gap:input-from-path-declares-snapshot-boundary",
        "test-gap input from-path keeps current-tree scope and semantic coverage limits visible",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:test-gap-is-bounded",
        "requirement:law:test-gap:test-gap-is-bounded",
        "test-gap detector status is bounded to the supplied snapshot and detector policy",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:verification-policy-controls-test-kind",
        "requirement:law:test-gap:verification-policy-controls-test-kind",
        "detector_context.test_kinds controls which observed test kinds close obligations",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:requirements-map-to-implementation-and-test",
        "requirement:law:test-gap:requirements-map-to-implementation-and-test",
        "in-scope test-gap requirements require implementation targets and accepted verification cells",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:candidates-remain-unreviewed",
        "requirement:law:test-gap:candidates-remain-unreviewed",
        "generated completion candidates stay unreviewed until explicit review",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:projection-declares-information-loss",
        "requirement:law:test-gap:projection-declares-information-loss",
        "test-gap projections declare information loss in human, AI, and audit views",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:schema-id-preserved",
        "requirement:law:test-gap:schema-id-preserved",
        "test-gap input and report schema IDs are preserved through runtime and CLI boundaries",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:enum-casing-round-trips",
        "requirement:law:test-gap:enum-casing-round-trips",
        "test-gap enum values serialize using schema casing and round-trip",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:runtime-shapes-preserve-schema",
        "requirement:law:test-gap:runtime-shapes-preserve-schema",
        "runtime TestGap shapes preserve required schema fields and report boundaries",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:test-gap:fixtures-validate-against-schema",
        "requirement:law:test-gap:fixtures-validate-against-schema",
        "checked-in test-gap fixtures validate against their declared schemas",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:backend-run-records-trust-boundary",
        "requirement:law:semantic-proof:backend-run-records-trust-boundary",
        "semantic-proof backend run records hashes, exit status, and review state without silently accepting failing outputs",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:artifact-status-totality",
        "requirement:law:semantic-proof:artifact-status-totality",
        "semantic-proof artifact adapter handles the proved and counterexample status partition",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:certificate-policy-preservation",
        "requirement:law:semantic-proof:certificate-policy-preservation",
        "proved semantic-proof artifacts preserve backend policy, hashes, witnesses, and accepted review state",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:counterexample-refutation-preservation",
        "requirement:law:semantic-proof:counterexample-refutation-preservation",
        "counterexample semantic-proof artifacts preserve refutation paths, severity, and review state",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:backend-boundary-is-explicit",
        "requirement:law:semantic-proof:backend-boundary-is-explicit",
        "semantic-proof artifact adapter keeps proof backend execution outside the bounded HG input adapter",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:counterexample-review-policy",
        "requirement:law:semantic-proof:counterexample-review-policy",
        "semantic-proof verification keeps unaccepted counterexamples behind the review boundary when policy requires it",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:insufficient-proof-reinputs-open-obligations",
        "requirement:law:semantic-proof:insufficient-proof-reinputs-open-obligations",
        "semantic-proof input from-report preserves open obligations from insufficient proof reports",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    push_law_requirement(
        &mut requirements,
        structural,
        "law:semantic-proof:roundtrip-tests-cover-proof-and-counterexample",
        "requirement:law:semantic-proof:roundtrip-tests-cover-proof-and-counterexample",
        "semantic-proof CLI roundtrip tests cover proved and counterexample artifact paths through verify",
        diff_evidence_id,
        accepted_test_kinds,
    )?;
    Ok(requirements)
}

fn push_structural_requirement(
    requirements: &mut Vec<TestGapInputRequirement>,
    structural: &StructuralModel,
    requirement_id: &str,
    summary: &str,
    implementation_ids: &[&str],
    diff_evidence_id: &Id,
    accepted_test_kinds: &[TestGapTestType],
) -> Result<(), String> {
    let implementation_ids = implementation_ids
        .iter()
        .map(|value| id(*value))
        .collect::<Result<Vec<_>, _>>()?;
    if implementation_ids
        .iter()
        .any(|implementation_id| !has_structural_symbol(&structural.symbols, implementation_id))
    {
        return Ok(());
    }
    let mut source_ids = vec![diff_evidence_id.clone()];
    source_ids.extend(implementation_ids.iter().cloned());
    requirements.push(TestGapInputRequirement {
        id: id(requirement_id)?,
        requirement_type: TestGapRequirementType::Custom,
        summary: summary.to_owned(),
        in_scope: true,
        bug_fix: false,
        implementation_ids,
        source_ids,
        expected_verification: Some(expected_verification_label(accepted_test_kinds)),
    });
    Ok(())
}

fn push_law_requirement(
    requirements: &mut Vec<TestGapInputRequirement>,
    structural: &StructuralModel,
    law_symbol_id: &str,
    requirement_id: &str,
    summary: &str,
    diff_evidence_id: &Id,
    accepted_test_kinds: &[TestGapTestType],
) -> Result<(), String> {
    let law_symbol_id = id(law_symbol_id)?;
    if !has_structural_symbol(&structural.symbols, &law_symbol_id) {
        return Ok(());
    }
    requirements.push(TestGapInputRequirement {
        id: id(requirement_id)?,
        requirement_type: TestGapRequirementType::Custom,
        summary: summary.to_owned(),
        in_scope: true,
        bug_fix: false,
        implementation_ids: vec![law_symbol_id.clone()],
        source_ids: vec![diff_evidence_id.clone(), law_symbol_id],
        expected_verification: Some(expected_verification_label(accepted_test_kinds)),
    });
    Ok(())
}

fn tests_for_changes(
    changes: &[GitChange],
    symbols: &[TestGapInputSymbol],
    content_test_targets: &BTreeMap<String, Vec<Id>>,
    rust_test_files: &BTreeSet<String>,
    diff_evidence_id: &Id,
) -> Result<Vec<TestGapInputTest>, String> {
    let mut tests = changes
        .iter()
        .filter(|change| is_test_path(&change.path) || rust_test_files.contains(&change.path))
        .map(|change| {
            let file_id = file_id(&change.path)?;
            let mut target_ids = matching_symbol_ids(&change.path, symbols);
            if let Some(content_target_ids) = content_test_targets.get(&change.path) {
                for target_id in content_target_ids {
                    push_unique_id(&mut target_ids, target_id.clone());
                }
            }
            Ok::<TestGapInputTest, String>(TestGapInputTest {
                id: id(format!("test:{}", slug(&change.path)))?,
                name: format!("Changed test file {}", change.path),
                test_type: test_gap_test_type_for_observed_rust_test(&change.path),
                file_id: Some(file_id),
                target_ids,
                branch_ids: Vec::new(),
                requirement_ids: Vec::new(),
                is_regression: false,
                context_ids: vec![id("context:test-scope")?],
                source_ids: vec![diff_evidence_id.clone()],
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if changes.iter().any(|change| {
        change.path == "scripts/validate-json-contracts.py" || change.path.ends_with(".schema.json")
    }) && symbols
        .iter()
        .any(|symbol| symbol.id.as_str() == "validator:test-gap:json-contracts")
    {
        let target_ids = [
            "validator:test-gap:json-contracts",
            "law:test-gap:fixtures-validate-against-schema",
        ]
        .into_iter()
        .filter(|symbol_id| {
            symbols
                .iter()
                .any(|symbol| symbol.id.as_str() == *symbol_id)
        })
        .map(id)
        .collect::<Result<Vec<_>, _>>()?;
        tests.push(TestGapInputTest {
            id: id("test:validator:test-gap-json-contracts")?,
            name: "JSON contract validator command".to_owned(),
            test_type: TestGapTestType::Smoke,
            file_id: Some(file_id(
                changes
                    .iter()
                    .find(|change| change.path == "scripts/validate-json-contracts.py")
                    .or_else(|| {
                        changes
                            .iter()
                            .find(|change| change.path.ends_with(".schema.json"))
                    })
                    .map(|change| change.path.as_str())
                    .unwrap_or("scripts/validate-json-contracts.py"),
            )?),
            target_ids,
            branch_ids: Vec::new(),
            requirement_ids: Vec::new(),
            is_regression: false,
            context_ids: vec![id("context:test-scope")?],
            source_ids: vec![diff_evidence_id.clone()],
        });
    }

    Ok(tests)
}

fn verification_cells_for_tests(
    tests: &[TestGapInputTest],
    structural: &StructuralModel,
    diff_evidence_id: &Id,
) -> Result<Vec<TestGapVerificationCell>, String> {
    tests
        .iter()
        .map(|test| {
            let law_ids = structural
                .laws
                .iter()
                .filter(|law| {
                    test.target_ids.contains(&law.id)
                        || test.requirement_ids.iter().any(|requirement_id| {
                            requirement_id.as_str()
                                == format!(
                                    "requirement:law:{}",
                                    law.id.as_str().trim_start_matches("law:")
                                )
                        })
                })
                .map(|law| law.id.clone())
                .collect::<Vec<_>>();
            let morphism_ids = structural
                .morphisms
                .iter()
                .filter(|morphism| {
                    test.target_ids.contains(&morphism.id)
                        || test.target_ids.iter().any(|target_id| {
                            morphism.source_ids.contains(target_id)
                                || morphism.target_ids.contains(target_id)
                                || morphism.law_ids.contains(target_id)
                        })
                        || test_semantically_covers_morphism(test, morphism)
                        || test.requirement_ids.iter().any(|requirement_id| {
                            requirement_id.as_str()
                                == format!(
                                    "requirement:morphism:{}",
                                    morphism.id.as_str().trim_start_matches("morphism:")
                                )
                        })
                })
                .map(|morphism| morphism.id.clone())
                .collect::<Vec<_>>();
            let mut law_ids = law_ids;
            for morphism in structural
                .morphisms
                .iter()
                .filter(|morphism| morphism_ids.contains(&morphism.id))
            {
                for law_id in &morphism.law_ids {
                    push_unique_id(&mut law_ids, law_id.clone());
                }
            }
            Ok(TestGapVerificationCell {
                id: id(format!("verification:{}", slug(test.id.as_str())))?,
                name: format!("Verification cell for {}", test.name),
                verification_type: if test.test_type == TestGapTestType::Smoke {
                    "validator".to_owned()
                } else {
                    "automated_test".to_owned()
                },
                test_type: test.test_type,
                target_ids: test.target_ids.clone(),
                requirement_ids: test.requirement_ids.clone(),
                law_ids,
                morphism_ids,
                source_ids: vec![diff_evidence_id.clone(), test.id.clone()],
                confidence: Some(confidence(0.72)?),
            })
        })
        .collect()
}

fn test_semantically_covers_morphism(
    test: &TestGapInputTest,
    morphism: &TestGapInputMorphism,
) -> bool {
    if !morphism.morphism_type.starts_with("semantic_") {
        return false;
    }
    if morphism
        .source_ids
        .iter()
        .chain(morphism.target_ids.iter())
        .filter_map(semantic_endpoint_path_slug)
        .any(|path_slug| {
            matches!(
                path_slug,
                "tools-highergraphen-cli-src-semantic-proof-artifact-rs"
                    | "tools-highergraphen-cli-src-semantic-proof-backend-rs"
                    | "tools-highergraphen-cli-src-semantic-proof-reinput-rs"
            )
        })
    {
        return false;
    }
    let target_path_slugs = test
        .target_ids
        .iter()
        .filter_map(test_target_path_slug)
        .collect::<BTreeSet<_>>();
    if !target_path_slugs.is_empty()
        && morphism
            .source_ids
            .iter()
            .chain(morphism.target_ids.iter())
            .filter_map(semantic_endpoint_path_slug)
            .any(|path_slug| target_path_slugs.contains(path_slug))
    {
        return true;
    }
    let targets_json_contracts = test
        .target_ids
        .iter()
        .any(|target_id| target_id.as_str() == "validator:test-gap:json-contracts");
    targets_json_contracts
        && morphism
            .source_ids
            .iter()
            .chain(morphism.target_ids.iter())
            .any(|endpoint_id| endpoint_id.as_str().starts_with("semantic:json-schema:"))
}

fn test_target_path_slug(target_id: &Id) -> Option<&str> {
    let value = target_id.as_str();
    value
        .strip_prefix("symbol:")
        .and_then(|value| value.strip_suffix(":changed-behavior"))
        .or_else(|| structural_cell_path_slug(value))
}

fn structural_cell_path_slug(target_id: &str) -> Option<&'static str> {
    match target_id {
        "command:highergraphen:test-gap:detect"
        | "command:highergraphen:test-gap:input-from-git"
        | "command:highergraphen:test-gap:input-from-path" => {
            Some("tools-highergraphen-cli-src-main-rs")
        }
        "adapter:test-gap:git-input" | "adapter:test-gap:path-input" => {
            Some("tools-highergraphen-cli-src-test-gap-git-rs")
        }
        "runner:test-gap:detect" => Some("crates-higher-graphen-runtime-src-workflows-test-gap-rs"),
        "export:test-gap:runtime-api" => Some("crates-higher-graphen-runtime-src-lib-rs"),
        "registry:test-gap:workflow-module" => {
            Some("crates-higher-graphen-runtime-src-workflows-mod-rs")
        }
        "contract:test-gap:runtime-report-shapes" => {
            Some("crates-higher-graphen-runtime-src-test-gap-reports-rs")
        }
        "projection:test-gap:report-envelope" => {
            Some("crates-higher-graphen-runtime-src-reports-rs")
        }
        "schema:test-gap:input" => Some("schemas-inputs-test-gap-input-schema-json"),
        "schema:test-gap:report" => Some("schemas-reports-test-gap-report-schema-json"),
        "command:highergraphen:semantic-proof:input-from-artifact"
        | "command:highergraphen:semantic-proof:backend-run"
        | "command:highergraphen:semantic-proof:input-from-report"
        | "command:highergraphen:semantic-proof:verify" => {
            Some("tools-highergraphen-cli-src-main-rs")
        }
        "runner:semantic-proof:backend-run"
        | "theorem:semantic-proof:backend-run-trust-boundary" => {
            Some("tools-highergraphen-cli-src-semantic-proof-backend-rs")
        }
        "adapter:semantic-proof:artifact-input"
        | "theorem:semantic-proof:artifact-adapter-correctness" => {
            Some("tools-highergraphen-cli-src-semantic-proof-artifact-rs")
        }
        "adapter:semantic-proof:reinput-from-report"
        | "theorem:semantic-proof:obligation-reinput-correctness" => {
            Some("tools-highergraphen-cli-src-semantic-proof-reinput-rs")
        }
        "test:semantic-proof:artifact-roundtrip" => {
            Some("tools-highergraphen-cli-tests-command-rs")
        }
        "test:semantic-proof:backend-and-reinput" => {
            Some("tools-highergraphen-cli-tests-command-rs")
        }
        _ => None,
    }
}

fn semantic_endpoint_path_slug(endpoint_id: &Id) -> Option<&str> {
    let mut parts = endpoint_id.as_str().split(':');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("semantic"), Some("rust"), Some(_), Some(path_slug))
        | (Some("semantic"), Some("json-schema"), Some(_), Some(path_slug)) => Some(path_slug),
        _ => None,
    }
}

fn link_tests_to_requirements(
    tests: Vec<TestGapInputTest>,
    requirements: &[TestGapInputRequirement],
) -> Vec<TestGapInputTest> {
    tests
        .into_iter()
        .map(|mut test| {
            test.requirement_ids = matching_requirement_ids(&test.target_ids, requirements);
            test
        })
        .collect()
}

fn coverage_for_tests(
    tests: &[TestGapInputTest],
    accepted_test_kinds: &[TestGapTestType],
) -> Result<Vec<TestGapInputCoverage>, String> {
    let mut coverage = Vec::new();
    for test in tests
        .iter()
        .filter(|test| accepted_test_kinds.contains(&test.test_type))
    {
        for target_id in &test.target_ids {
            coverage.push(TestGapInputCoverage {
                id: id(format!(
                    "coverage:{}:{}",
                    slug(test.id.as_str()),
                    slug(target_id.as_str())
                ))?,
                coverage_type: TestGapCoverageType::Function,
                target_id: target_id.clone(),
                status: TestGapCoverageStatus::Covered,
                covered_by_test_ids: vec![test.id.clone()],
                source_ids: test.source_ids.clone(),
                summary: Some(format!(
                    "Git adapter matched {} to {}",
                    test.name, target_id
                )),
                confidence: Some(confidence(0.62)?),
            });
        }
    }
    Ok(coverage)
}

fn contexts_for_changes(
    changes: &[GitChange],
    change_set_id: &Id,
    review_focus_name: &str,
) -> Result<Vec<TestGapInputContext>, String> {
    let mut contexts = BTreeMap::<Id, (String, TestGapContextType)>::new();
    contexts.insert(
        id("context:repository")?,
        ("Repository".to_owned(), TestGapContextType::Repository),
    );
    contexts.insert(
        id(format!("context:{}", slug(change_set_id.as_str())))?,
        (
            review_focus_name.to_owned(),
            TestGapContextType::ReviewFocus,
        ),
    );

    for change in changes {
        for (context_id, name, context_type) in test_gap_context_descriptors_for_path(&change.path)?
        {
            contexts.insert(context_id, (name, context_type));
        }
    }

    Ok(contexts
        .into_iter()
        .map(|(id, (name, context_type))| TestGapInputContext {
            id,
            name,
            context_type,
            source_ids: Vec::new(),
        })
        .collect())
}

fn evidence_for_changes(
    changes: &[GitChange],
    commits: &[String],
    diff_evidence_id: &Id,
    commit_evidence_id: &Id,
) -> Result<Vec<TestGapInputEvidence>, String> {
    let mut evidence = vec![TestGapInputEvidence {
        id: diff_evidence_id.clone(),
        evidence_type: TestGapEvidenceType::DiffHunk,
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
        evidence.push(TestGapInputEvidence {
            id: commit_evidence_id.clone(),
            evidence_type: TestGapEvidenceType::Custom,
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
    tests: &[TestGapInputTest],
    accepted_test_kinds: &[TestGapTestType],
    diff_analysis: &GitDiffAnalysis,
    diff_evidence_id: &Id,
) -> Result<Vec<TestGapInputRiskSignal>, String> {
    let mut signals = Vec::new();
    push_missing_test_signal(&mut signals, changes, tests, accepted_test_kinds)?;
    push_diff_signal(
        &mut signals,
        "signal:test-gap:public-api-change",
        TestGapRiskSignalType::PublicApiChange,
        "Diff changes public Rust API-like declarations.",
        &diff_analysis.public_api_ids,
        Severity::High,
        0.74,
    )?;
    push_diff_signal(
        &mut signals,
        "signal:test-gap:error-path-change",
        TestGapRiskSignalType::ErrorPathChange,
        "Diff adds panic, unwrap/expect, or placeholder control-flow paths.",
        &diff_analysis.panic_or_placeholder_ids,
        Severity::Medium,
        0.7,
    )?;
    push_diff_signal(
        &mut signals,
        "signal:test-gap:boundary-change",
        TestGapRiskSignalType::BoundaryChange,
        "Diff changes boundary, incidence, or composition structure between finite code elements.",
        &diff_analysis.structural_boundary_ids,
        Severity::High,
        0.73,
    )?;
    push_size_signal(&mut signals, changes, diff_evidence_id)?;
    Ok(signals)
}

fn push_missing_test_signal(
    signals: &mut Vec<TestGapInputRiskSignal>,
    changes: &[GitChange],
    tests: &[TestGapInputTest],
    accepted_test_kinds: &[TestGapTestType],
) -> Result<(), String> {
    let changed_accepted_test_targets = tests
        .iter()
        .filter(|test| accepted_test_kinds.contains(&test.test_type))
        .flat_map(|test| test.target_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let uncovered = changes
        .iter()
        .filter(|change| is_source_code_path(&change.path))
        .map(|change| {
            let symbol_id = id(format!("symbol:{}:changed-behavior", slug(&change.path)))?;
            Ok((file_id(&change.path)?, symbol_id))
        })
        .filter_map(|result: Result<(Id, Id), String>| match result {
            Ok((file_id, symbol_id)) if !changed_accepted_test_targets.contains(&symbol_id) => {
                Some(Ok(file_id))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    if uncovered.is_empty() {
        return Ok(());
    }

    signals.push(TestGapInputRiskSignal {
        id: id("signal:test-gap:changed-source-without-accepted-test")?,
        signal_type: TestGapRiskSignalType::TestGap,
        summary: format!(
            "Input snapshot contains {} source files without a matched policy-accepted test.",
            uncovered.len()
        ),
        source_ids: uncovered,
        severity: Severity::Medium,
        confidence: confidence(0.72)?,
    });
    Ok(())
}

fn push_size_signal(
    signals: &mut Vec<TestGapInputRiskSignal>,
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<(), String> {
    let total_lines = changes
        .iter()
        .map(|change| change.additions + change.deletions)
        .sum::<u32>();
    if changes.len() < 6 && total_lines < 500 {
        return Ok(());
    }
    signals.push(TestGapInputRiskSignal {
        id: id("signal:test-gap:large-git-change")?,
        signal_type: TestGapRiskSignalType::Custom,
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
    Ok(())
}

fn push_diff_signal(
    signals: &mut Vec<TestGapInputRiskSignal>,
    id_value: &str,
    signal_type: TestGapRiskSignalType,
    summary: &str,
    source_ids: &[Id],
    severity: Severity,
    confidence_value: f64,
) -> Result<(), String> {
    if source_ids.is_empty() {
        return Ok(());
    }
    signals.push(TestGapInputRiskSignal {
        id: id(id_value)?,
        signal_type,
        summary: summary.to_owned(),
        source_ids: source_ids.to_vec(),
        severity,
        confidence: confidence(confidence_value)?,
    });
    Ok(())
}

fn matching_symbol_ids(test_path: &str, symbols: &[TestGapInputSymbol]) -> Vec<Id> {
    let test_key = comparable_path_key(test_path);
    let mut ids = symbols
        .iter()
        .filter(|symbol| {
            symbol
                .path
                .as_deref()
                .map(comparable_path_key)
                .is_some_and(|symbol_key| {
                    test_key.contains(&symbol_key) || symbol_key.contains(&test_key)
                })
        })
        .map(|symbol| symbol.id.clone())
        .collect::<Vec<_>>();

    if test_path == "tools/highergraphen-cli/tests/command.rs" {
        push_matching_symbols(
            &mut ids,
            symbols,
            &[
                "command:highergraphen:test-gap:detect",
                "command:highergraphen:test-gap:input-from-git",
                "command:highergraphen:test-gap:input-from-path",
                "adapter:test-gap:git-input",
                "adapter:test-gap:path-input",
                "law:test-gap:command-routes-to-runner",
                "law:test-gap:json-format-required",
                "law:test-gap:output-file-suppresses-stdout",
                "law:test-gap:input-from-git-is-deterministic",
                "law:test-gap:input-from-git-does-not-prove-semantic-coverage",
                "law:test-gap:input-from-path-is-deterministic",
                "law:test-gap:input-from-path-declares-snapshot-boundary",
                "symbol:tools-highergraphen-cli-src-main-rs:changed-behavior",
                "command:highergraphen:semantic-proof:backend-run",
                "command:highergraphen:semantic-proof:input-from-artifact",
                "command:highergraphen:semantic-proof:input-from-report",
                "command:highergraphen:semantic-proof:verify",
                "runner:semantic-proof:backend-run",
                "adapter:semantic-proof:artifact-input",
                "adapter:semantic-proof:reinput-from-report",
                "test:semantic-proof:artifact-roundtrip",
                "test:semantic-proof:backend-and-reinput",
                "theorem:semantic-proof:artifact-adapter-correctness",
                "theorem:semantic-proof:backend-run-trust-boundary",
                "theorem:semantic-proof:obligation-reinput-correctness",
                "law:semantic-proof:backend-run-records-trust-boundary",
                "law:semantic-proof:artifact-status-totality",
                "law:semantic-proof:certificate-policy-preservation",
                "law:semantic-proof:counterexample-refutation-preservation",
                "law:semantic-proof:counterexample-review-policy",
                "law:semantic-proof:insufficient-proof-reinputs-open-obligations",
                "law:semantic-proof:backend-boundary-is-explicit",
                "law:semantic-proof:roundtrip-tests-cover-proof-and-counterexample",
                "symbol:tools-highergraphen-cli-src-semantic-proof-backend-rs:changed-behavior",
                "symbol:tools-highergraphen-cli-src-semantic-proof-artifact-rs:changed-behavior",
                "symbol:tools-highergraphen-cli-src-semantic-proof-reinput-rs:changed-behavior",
            ],
        );
    }
    if test_path == "crates/higher-graphen-runtime/tests/test_gap.rs" {
        push_matching_symbols(
            &mut ids,
            symbols,
            &[
                "runner:test-gap:detect",
                "export:test-gap:runtime-api",
                "registry:test-gap:workflow-module",
                "contract:test-gap:runtime-report-shapes",
                "projection:test-gap:report-envelope",
                "schema:test-gap:input",
                "schema:test-gap:report",
                "fixture:test-gap:input-example",
                "fixture:test-gap:report-example",
                "law:test-gap:test-gap-is-bounded",
                "law:test-gap:verification-policy-controls-test-kind",
                "law:test-gap:requirements-map-to-implementation-and-test",
                "law:test-gap:candidates-remain-unreviewed",
                "law:test-gap:projection-declares-information-loss",
                "law:test-gap:schema-id-preserved",
                "law:test-gap:enum-casing-round-trips",
                "law:test-gap:runtime-shapes-preserve-schema",
                "symbol:crates-higher-graphen-runtime-src-test-gap-reports-rs:changed-behavior",
                "symbol:crates-higher-graphen-runtime-src-workflows-test-gap-rs:changed-behavior",
            ],
        );
    }

    ids
}

fn push_matching_symbols(ids: &mut Vec<Id>, symbols: &[TestGapInputSymbol], symbol_ids: &[&str]) {
    for symbol_id in symbol_ids {
        if symbols
            .iter()
            .any(|symbol| symbol.id.as_str() == *symbol_id)
        {
            if let Ok(id) = id(*symbol_id) {
                push_unique_id(ids, id);
            }
        }
    }
}

fn push_unique_id(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn accepted_test_kinds_for_tests(tests: &[TestGapInputTest]) -> Vec<TestGapTestType> {
    let mut accepted = vec![TestGapTestType::Unit];
    for test in tests {
        if test.test_type != TestGapTestType::Unknown && !accepted.contains(&test.test_type) {
            accepted.push(test.test_type);
        }
    }
    accepted
}

fn expected_verification_label(accepted_test_kinds: &[TestGapTestType]) -> String {
    if accepted_test_kinds.contains(&TestGapTestType::Integration) {
        "unit_or_integration_test".to_owned()
    } else {
        "unit_test".to_owned()
    }
}

fn matching_requirement_ids(
    target_ids: &[Id],
    requirements: &[TestGapInputRequirement],
) -> Vec<Id> {
    requirements
        .iter()
        .filter(|requirement| {
            requirement
                .implementation_ids
                .iter()
                .any(|implementation_id| target_ids.contains(implementation_id))
        })
        .map(|requirement| requirement.id.clone())
        .collect()
}

fn test_gap_context_ids_for_path(path: &str) -> Result<Vec<Id>, String> {
    test_gap_context_descriptors_for_path(path).map(|descriptors| {
        descriptors
            .into_iter()
            .map(|(context_id, _, _)| context_id)
            .collect()
    })
}

fn test_gap_context_descriptors_for_path(
    path: &str,
) -> Result<Vec<(Id, String, TestGapContextType)>, String> {
    let mut contexts = vec![(
        id("context:repository")?,
        "Repository".to_owned(),
        TestGapContextType::Repository,
    )];
    if is_runtime_path(path) {
        contexts.push((
            id("context:runtime")?,
            "Runtime".to_owned(),
            TestGapContextType::Module,
        ));
    }
    if is_cli_path(path) {
        contexts.push((
            id("context:cli")?,
            "CLI".to_owned(),
            TestGapContextType::Module,
        ));
    }
    if is_schema_path(path) {
        contexts.push((
            id("context:schema")?,
            "Schema".to_owned(),
            TestGapContextType::RequirementScope,
        ));
    }
    if path.contains("/workflows/") {
        contexts.push((
            id("context:workflow-logic")?,
            "Workflow Logic".to_owned(),
            TestGapContextType::SymbolScope,
        ));
    }
    if is_test_path(path) {
        contexts.push((
            id("context:test-scope")?,
            "Test Scope".to_owned(),
            TestGapContextType::TestScope,
        ));
    }
    if path.starts_with("docs/") || path.starts_with("skills/") {
        contexts.push((
            id("context:agent-guidance")?,
            "Agent Guidance".to_owned(),
            TestGapContextType::ReviewFocus,
        ));
    }
    Ok(contexts)
}

fn test_gap_test_type_for_path(path: &str) -> TestGapTestType {
    if path.ends_with("_test.rs")
        || path.ends_with(".test.rs")
        || path.contains("/unit/")
        || path.contains("/unit_tests/")
    {
        TestGapTestType::Unit
    } else if path.contains("/tests/") {
        TestGapTestType::Integration
    } else {
        TestGapTestType::Unknown
    }
}

fn test_gap_test_type_for_observed_rust_test(path: &str) -> TestGapTestType {
    if is_test_path(path) {
        test_gap_test_type_for_path(path)
    } else {
        TestGapTestType::Unit
    }
}

fn is_rust_source_path(path: &str) -> bool {
    path.ends_with(".rs") && !path.starts_with("target/")
}

fn is_source_code_path(path: &str) -> bool {
    if is_test_path(path) || path.starts_with("target/") {
        return false;
    }
    matches!(
        std::path::Path::new(path)
            .extension()
            .and_then(|value| value.to_str()),
        Some(
            "rs" | "py"
                | "js"
                | "jsx"
                | "ts"
                | "tsx"
                | "go"
                | "java"
                | "kt"
                | "kts"
                | "swift"
                | "c"
                | "cc"
                | "cpp"
                | "h"
                | "hpp"
                | "cs"
                | "ex"
                | "exs"
        )
    )
}

fn is_highergraphen_structural_path(path: &str) -> bool {
    matches!(
        path,
        "tools/highergraphen-cli/src/main.rs"
            | "tools/highergraphen-cli/src/test_gap_git.rs"
            | "scripts/validate-json-contracts.py"
            | "crates/higher-graphen-runtime/src/lib.rs"
            | "crates/higher-graphen-runtime/src/reports.rs"
            | "crates/higher-graphen-runtime/src/test_gap_reports.rs"
            | "crates/higher-graphen-runtime/src/workflows/mod.rs"
            | "crates/higher-graphen-runtime/src/workflows/test_gap.rs"
            | "tools/highergraphen-cli/src/semantic_proof_artifact.rs"
            | "tools/highergraphen-cli/src/semantic_proof_backend.rs"
            | "tools/highergraphen-cli/src/semantic_proof_reinput.rs"
    )
}

fn is_test_gap_surface_path(path: &str) -> bool {
    path.contains("test_gap") || path.contains("test-gap")
}

fn is_semantic_proof_surface_path(path: &str) -> bool {
    path.contains("semantic_proof") || path.contains("semantic-proof")
}

fn comparable_path_key(path: &str) -> String {
    let stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
        .trim_end_matches("_test")
        .trim_end_matches(".test")
        .trim_start_matches("test_")
        .to_owned();
    slug(&stem)
}

fn map_change_type(change_type: PrReviewTargetChangeType) -> TestGapChangeType {
    match change_type {
        PrReviewTargetChangeType::Added => TestGapChangeType::Added,
        PrReviewTargetChangeType::Modified => TestGapChangeType::Modified,
        PrReviewTargetChangeType::Deleted => TestGapChangeType::Deleted,
        PrReviewTargetChangeType::Renamed => TestGapChangeType::Renamed,
    }
}

trait TestGapSymbolPath {
    fn path(&self) -> &str;
}

impl TestGapSymbolPath for TestGapInputSymbol {
    fn path(&self) -> &str {
        self.path.as_deref().unwrap_or(self.name.as_str())
    }
}
