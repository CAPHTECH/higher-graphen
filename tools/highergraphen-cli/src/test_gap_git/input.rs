const INPUT_SCHEMA: &str = "highergraphen.test_gap.input.v1";
const BINDING_RULES_SCHEMA: &str = "highergraphen.test_gap.binding_rules.input.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GitInputRequest {
    pub(crate) repo: PathBuf,
    pub(crate) base: String,
    pub(crate) head: String,
    pub(crate) binding_rules: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathInputRequest {
    pub(crate) repo: PathBuf,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) include_tests: bool,
    pub(crate) binding_rules: Option<PathBuf>,
}

struct InputParts {
    changed_files: Vec<TestGapInputChangedFile>,
    symbols: Vec<TestGapInputSymbol>,
    requirements: Vec<TestGapInputRequirement>,
    tests: Vec<TestGapInputTest>,
    coverage: Vec<TestGapInputCoverage>,
    structural: StructuralModel,
    verification_cells: Vec<TestGapVerificationCell>,
    contexts: Vec<TestGapInputContext>,
    evidence: Vec<TestGapInputEvidence>,
    signals: Vec<TestGapInputRiskSignal>,
    accepted_test_kinds: Vec<TestGapTestType>,
    declared_obligation_ids: Vec<Id>,
}

pub(crate) fn input_from_git(request: GitInputRequest) -> Result<TestGapInputDocument, String> {
    let metadata = GitInputMetadata::read(&request)?;
    let range = format!("{}..{}", request.base, request.head);
    let binding_rules = binding_rules_from_path(request.binding_rules.as_deref())?;
    let commits = commit_summaries(&metadata.repo_path, &range)?;
    let changes = non_empty_git_changes(&metadata.repo_path, &range)?;
    let diff_analysis = diff_analysis(&metadata.repo_path, &range, &changes)?;
    let ids = InputEvidenceIds::new("evidence:git-diff", "evidence:git-commits")?;
    let structural = git_structural_model(&metadata, &request, &changes, &binding_rules, &ids)?;
    let change_set_id = git_change_set_id(&metadata.repo_name, &request.base, &request.head)?;
    let parts = build_input_parts(
        &changes,
        structural,
        &diff_analysis,
        &commits,
        &ids,
        change_set_id.clone(),
        "Git Range",
    )?;
    git_input_document(metadata, request, range, change_set_id, parts)
}

pub(crate) fn input_from_path(request: PathInputRequest) -> Result<TestGapInputDocument, String> {
    if request.paths.is_empty() {
        return Err("--path <path> is required".to_owned());
    }

    let metadata = GitInputMetadata::read_repo(&request.repo)?;
    let changes = non_empty_path_changes(&metadata.repo_path, &request)?;
    let binding_rules = binding_rules_from_path(request.binding_rules.as_deref())?;
    let diff_analysis = current_tree_analysis(&metadata.repo_path, &changes)?;
    let path_scope = path_scope(&request.paths);
    let ids = InputEvidenceIds::new("evidence:path-scan", "evidence:current-head")?;
    let structural = path_structural_model(&metadata, &changes, &binding_rules, &ids)?;
    let change_set_id = path_change_set_id(&metadata.repo_name, &path_scope)?;
    let parts = build_input_parts(
        &changes,
        structural,
        &diff_analysis,
        &[],
        &ids,
        change_set_id.clone(),
        "Path Scan",
    )?;
    path_input_document(metadata, request, path_scope, change_set_id, parts)
}

struct InputEvidenceIds {
    diff: Id,
    commit: Id,
}

impl InputEvidenceIds {
    fn new(diff: &str, commit: &str) -> Result<Self, String> {
        Ok(Self {
            diff: id(diff)?,
            commit: id(commit)?,
        })
    }
}

fn non_empty_git_changes(repo: &Path, range: &str) -> Result<Vec<GitChange>, String> {
    let changes = changed_files(repo, range)?;
    if changes.is_empty() {
        Err(format!("git range {range} has no changed files"))
    } else {
        Ok(changes)
    }
}

fn non_empty_path_changes(
    repo: &Path,
    request: &PathInputRequest,
) -> Result<Vec<GitChange>, String> {
    let changes = path_changes(repo, &request.paths, request.include_tests)?;
    if changes.is_empty() {
        Err("path scan has no supported files".to_owned())
    } else {
        Ok(changes)
    }
}

fn path_scope(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>()
        .join(",")
}

fn git_change_set_id(repo_name: &str, base: &str, head: &str) -> Result<Id, String> {
    id(format!(
        "change:test-gap:{}:{}..{}",
        slug(repo_name),
        slug(base),
        slug(head)
    ))
}

fn path_change_set_id(repo_name: &str, path_scope: &str) -> Result<Id, String> {
    id(format!(
        "change:test-gap:{}:path:{}",
        slug(repo_name),
        slug(path_scope)
    ))
}

fn git_structural_model(
    metadata: &GitInputMetadata,
    request: &GitInputRequest,
    changes: &[GitChange],
    binding_rules: &HgRustTestBindingRules,
    ids: &InputEvidenceIds,
) -> Result<StructuralModel, String> {
    let mut structural = structural_model_for_changes(changes, &ids.diff)?;
    structural.extend(semantic_model_for_changes(
        &metadata.repo_path,
        &request.base,
        &request.head,
        changes,
        &ids.diff,
    )?);
    let test_content = rust_test_content_model_for_changes(
        &metadata.repo_path,
        &request.head,
        changes,
        &structural,
        binding_rules,
        &ids.diff,
    )?;
    Ok(merge_test_content(structural, test_content))
}

fn path_structural_model(
    metadata: &GitInputMetadata,
    changes: &[GitChange],
    binding_rules: &HgRustTestBindingRules,
    ids: &InputEvidenceIds,
) -> Result<StructuralModel, String> {
    let mut structural = structural_model_for_changes(changes, &ids.diff)?;
    structural.extend(semantic_model_for_paths(
        &metadata.repo_path,
        changes,
        &ids.diff,
    )?);
    let test_content = rust_test_content_model_for_paths(
        &metadata.repo_path,
        changes,
        &structural,
        binding_rules,
        &ids.diff,
    )?;
    Ok(merge_test_content(structural, test_content))
}

fn merge_test_content(
    mut structural: StructuralModel,
    test_content: RustTestContentModel,
) -> StructuralModel {
    structural.test_files = test_content.test_files;
    structural.target_ids_by_file = test_content.target_ids_by_file;
    structural.extend(test_content.structural);
    structural
}

fn build_input_parts(
    changes: &[GitChange],
    structural: StructuralModel,
    diff_analysis: &GitDiffAnalysis,
    commits: &[String],
    ids: &InputEvidenceIds,
    change_set_id: Id,
    review_focus_name: &str,
) -> Result<InputParts, String> {
    let mut symbols = symbols_for_changes(changes, &ids.diff, diff_analysis)?;
    let rust_test_files = structural.test_files.clone();
    let content_test_targets = structural.target_ids_by_file.clone();
    symbols.extend(structural.symbols.clone());
    let tests = tests_for_changes(
        changes,
        &symbols,
        &content_test_targets,
        &rust_test_files,
        &ids.diff,
    )?;
    let accepted_test_kinds = accepted_test_kinds_for_tests(&tests);
    let requirements = input_requirements(&symbols, &structural, &ids.diff, &accepted_test_kinds)?;
    let tests = link_tests_to_requirements(tests, &requirements);
    let declared_obligation_ids = declared_obligation_ids(&requirements);
    Ok(InputParts {
        changed_files: changed_files_for_input(changes, &symbols, &ids.diff)?,
        verification_cells: verification_cells_for_tests(&tests, &structural, &ids.diff)?,
        coverage: coverage_for_tests(&tests, &accepted_test_kinds)?,
        contexts: contexts_for_changes(changes, &change_set_id, review_focus_name)?,
        evidence: evidence_for_changes(changes, commits, &ids.diff, &ids.commit)?,
        signals: signals_for_changes(changes, &tests, &accepted_test_kinds, diff_analysis, &ids.diff)?,
        symbols,
        requirements,
        tests,
        structural,
        accepted_test_kinds,
        declared_obligation_ids,
    })
}

fn input_requirements(
    symbols: &[TestGapInputSymbol],
    structural: &StructuralModel,
    diff_evidence_id: &Id,
    accepted_test_kinds: &[TestGapTestType],
) -> Result<Vec<TestGapInputRequirement>, String> {
    let mut requirements =
        requirements_for_symbols(symbols, diff_evidence_id, accepted_test_kinds)?;
    requirements.extend(structural_requirements(
        structural,
        diff_evidence_id,
        accepted_test_kinds,
    )?);
    Ok(requirements)
}

fn declared_obligation_ids(requirements: &[TestGapInputRequirement]) -> Vec<Id> {
    requirements
        .iter()
        .map(|requirement| requirement.id.clone())
        .collect()
}

fn git_input_document(
    metadata: GitInputMetadata,
    request: GitInputRequest,
    range: String,
    change_set_id: Id,
    parts: InputParts,
) -> Result<TestGapInputDocument, String> {
    input_document(
        git_source(&metadata, &range)?,
        repository(&metadata, Some(request.base.clone()))?,
        git_change_set(&metadata.repo_path, &request, &range, change_set_id)?,
        parts,
        vec![
            "policy-accepted automated tests for changed source behavior".to_owned(),
            "git-derived deterministic evidence only".to_owned(),
        ],
        vec!["target/".to_owned()],
    )
}

fn path_input_document(
    metadata: GitInputMetadata,
    request: PathInputRequest,
    path_scope: String,
    change_set_id: Id,
    parts: InputParts,
) -> Result<TestGapInputDocument, String> {
    input_document(
        path_source(&metadata, &path_scope)?,
        repository(&metadata, None)?,
        path_change_set(&metadata.repo_path, &request, &path_scope, change_set_id)?,
        parts,
        vec![
            "policy-accepted automated tests for selected path behavior".to_owned(),
            "current-tree path-derived deterministic evidence only".to_owned(),
        ],
        vec![".git/".to_owned(), "target/".to_owned()],
    )
}

fn input_document(
    source: TestGapSource,
    repository: TestGapRepository,
    change_set: TestGapChangeSet,
    parts: InputParts,
    required_focus: Vec<String>,
    excluded_paths: Vec<String>,
) -> Result<TestGapInputDocument, String> {
    let detector_context = detector_context(
        parts.accepted_test_kinds.clone(),
        parts.declared_obligation_ids.clone(),
        required_focus,
        excluded_paths,
    );
    Ok(TestGapInputDocument {
        schema: INPUT_SCHEMA.to_owned(),
        source,
        repository,
        change_set,
        changed_files: parts.changed_files,
        symbols: parts.symbols,
        branches: Vec::new(),
        requirements: parts.requirements,
        tests: parts.tests,
        coverage: parts.coverage,
        dependency_edges: parts.structural.dependency_edges,
        higher_order_cells: parts.structural.higher_order_cells,
        higher_order_incidences: parts.structural.higher_order_incidences,
        morphisms: parts.structural.morphisms,
        laws: parts.structural.laws,
        verification_cells: parts.verification_cells,
        contexts: parts.contexts,
        evidence: parts.evidence,
        signals: parts.signals,
        detector_context: Some(detector_context),
    })
}

fn git_source(metadata: &GitInputMetadata, range: &str) -> Result<TestGapSource, String> {
    Ok(TestGapSource {
        kind: SourceKind::Code,
        uri: Some(format!("git:{}:{range}", metadata.repo_root)),
        title: Some(format!("Git range {range} in {}", metadata.repo_name)),
        captured_at: None,
        confidence: confidence(1.0)?,
        adapters: vec!["git-diff.v1".to_owned(), "test-gap-from-git.v1".to_owned()],
    })
}

fn path_source(metadata: &GitInputMetadata, path_scope: &str) -> Result<TestGapSource, String> {
    Ok(TestGapSource {
        kind: SourceKind::Code,
        uri: Some(format!("path:{}:{path_scope}", metadata.repo_root)),
        title: Some(format!("Path scan {path_scope} in {}", metadata.repo_name)),
        captured_at: None,
        confidence: confidence(1.0)?,
        adapters: vec![
            "current-tree.v1".to_owned(),
            "test-gap-from-path.v1".to_owned(),
        ],
    })
}

fn repository(
    metadata: &GitInputMetadata,
    fallback_branch: Option<String>,
) -> Result<TestGapRepository, String> {
    Ok(TestGapRepository {
        id: id(format!("repo:{}", slug(&metadata.repo_name)))?,
        name: metadata.repo_name.clone(),
        uri: non_empty_trimmed(metadata.remote_url.as_deref()),
        default_branch: non_empty_trimmed(metadata.default_branch.as_deref()).or(fallback_branch),
    })
}

fn non_empty_trimmed(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn git_change_set(
    repo: &Path,
    request: &GitInputRequest,
    range: &str,
    change_set_id: Id,
) -> Result<TestGapChangeSet, String> {
    Ok(TestGapChangeSet {
        id: change_set_id,
        base_ref: request.base.clone(),
        head_ref: request.head.clone(),
        base_commit: optional_git(repo, &["rev-parse", &request.base]).and_then(trimmed_git),
        head_commit: optional_git(repo, &["rev-parse", &request.head]).and_then(trimmed_git),
        boundary: format!("git diff {range}"),
        excluded_paths: vec!["target/".to_owned()],
    })
}

fn path_change_set(
    repo: &Path,
    request: &PathInputRequest,
    path_scope: &str,
    change_set_id: Id,
) -> Result<TestGapChangeSet, String> {
    Ok(TestGapChangeSet {
        id: change_set_id,
        base_ref: "current-tree".to_owned(),
        head_ref: "current-tree".to_owned(),
        base_commit: None,
        head_commit: optional_git(repo, &["rev-parse", "HEAD"]).and_then(trimmed_git),
        boundary: path_boundary(path_scope, request.include_tests),
        excluded_paths: vec![".git/".to_owned(), "target/".to_owned()],
    })
}

fn trimmed_git(value: String) -> Option<String> {
    non_empty_trimmed(Some(&value))
}

fn path_boundary(path_scope: &str, include_tests: bool) -> String {
    format!(
        "current tree path scan {path_scope}{}",
        if include_tests { " with tests" } else { "" }
    )
}

fn detector_context(
    accepted_test_kinds: Vec<TestGapTestType>,
    declared_obligation_ids: Vec<Id>,
    required_focus: Vec<String>,
    excluded_paths: Vec<String>,
) -> TestGapDetectorContext {
    TestGapDetectorContext {
        required_focus,
        excluded_paths,
        test_kinds: accepted_test_kinds,
        declared_obligation_ids,
    }
}
