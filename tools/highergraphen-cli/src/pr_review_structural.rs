use higher_graphen_core::Id;
use higher_graphen_space::{StructuralBoundaryAnalyzer, StructuralObservation, StructuralRole};

pub(crate) fn changed_structural_boundary(
    path: &str,
    added_lines: &[String],
    removed_lines: &[String],
    subject_id: &Id,
) -> Result<bool, String> {
    let observations = structural_observations(path, added_lines, removed_lines, subject_id)?;
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
    path: &str,
    added_lines: &[String],
    removed_lines: &[String],
    subject_id: &Id,
) -> Result<Vec<StructuralObservation>, String> {
    if !path.ends_with(".rs") || is_test_path(path) {
        return Ok(Vec::new());
    }

    added_lines
        .iter()
        .chain(removed_lines.iter())
        .enumerate()
        .filter_map(|(index, line)| structural_role_for_line(line).map(|role| (index, role)))
        .map(|(index, role)| {
            Ok(StructuralObservation::new(
                id(format!("observation:structural:{}:{}", slug(path), index))?,
                subject_id.clone(),
                role,
            )
            .with_source(subject_id.clone()))
        })
        .collect()
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

fn is_test_path(path: &str) -> bool {
    path.contains("/tests/") || path.ends_with("_test.rs") || path.ends_with(".test.rs")
}

fn id(value: impl Into<String>) -> Result<Id, String> {
    Id::new(value).map_err(|error| error.to_string())
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
