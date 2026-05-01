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
    test_files: BTreeSet<String>,
    target_ids_by_file: BTreeMap<String, Vec<Id>>,
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
