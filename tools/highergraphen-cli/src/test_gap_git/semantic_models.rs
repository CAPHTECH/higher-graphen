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
    binding_rules: &HgRustTestBindingRules,
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
                binding_rules,
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
    binding_rules: &HgRustTestBindingRules,
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
            binding_rules,
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct HgRustTestBindingRules {
    rules: Vec<HgRustTestBindingRule>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HgRustTestBindingRule {
    trigger_terms: Vec<String>,
    cli_label: Option<String>,
    target_ids: Vec<String>,
}
