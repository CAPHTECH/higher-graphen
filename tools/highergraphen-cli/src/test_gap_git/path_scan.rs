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
        push_current_tree_analysis(&mut analysis, repo, change)?;
    }
    Ok(analysis)
}

fn push_current_tree_analysis(
    analysis: &mut GitDiffAnalysis,
    repo: &Path,
    change: &GitChange,
) -> Result<(), String> {
    let file_id = file_id(&change.path)?;
    let Ok(contents) = fs::read_to_string(repo.join(&change.path)) else {
        return Ok(());
    };
    if current_tree_has_public_api(change, &contents) {
        push_unique_id(&mut analysis.public_api_ids, file_id.clone());
    }
    if current_tree_has_serde_contract(change, &contents) {
        push_unique_id(&mut analysis.serde_contract_ids, file_id.clone());
    }
    if current_tree_has_panic_or_placeholder(change, &contents) {
        push_unique_id(&mut analysis.panic_or_placeholder_ids, file_id.clone());
    }
    if current_tree_has_external_effect(change, &contents) {
        push_unique_id(&mut analysis.external_effect_ids, file_id.clone());
    }
    if current_tree_has_structural_boundary(change) {
        push_unique_id(&mut analysis.structural_boundary_ids, file_id);
    }
    Ok(())
}

fn current_tree_has_public_api(change: &GitChange, contents: &str) -> bool {
    is_source_code_path(&change.path) && has_public_api_like_text(contents)
}

fn current_tree_has_serde_contract(change: &GitChange, contents: &str) -> bool {
    change.path.ends_with(".schema.json")
        || contents.contains("#[serde")
        || contents.contains("deny_unknown_fields")
        || contents.contains("rename_all")
}

fn current_tree_has_panic_or_placeholder(change: &GitChange, contents: &str) -> bool {
    !is_test_path(&change.path) && has_panic_or_placeholder_text(contents)
}

fn current_tree_has_external_effect(change: &GitChange, contents: &str) -> bool {
    !is_test_path(&change.path) && has_external_effect_text(contents)
}

fn current_tree_has_structural_boundary(change: &GitChange) -> bool {
    is_highergraphen_structural_path(&change.path)
        || is_test_gap_surface_path(&change.path)
        || is_semantic_proof_surface_path(&change.path)
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
