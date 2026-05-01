use crate::{CoreError, Result};

pub(crate) fn normalize_optional_text(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>> {
    value
        .map(|raw| normalize_required_text(field, raw))
        .transpose()
}

pub(crate) fn normalize_optional_text_ref(
    field: &'static str,
    value: Option<&String>,
) -> Result<Option<String>> {
    normalize_optional_text(field, value.cloned())
}

pub(crate) fn normalize_required_text(
    field: &'static str,
    value: impl Into<String>,
) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();

    if normalized.is_empty() {
        return Err(CoreError::malformed_field(
            field,
            "field must not be empty after trimming",
        ));
    }

    Ok(normalized)
}

pub(crate) fn normalize_required_text_vec(field: &'static str, values: &[String]) -> Result<()> {
    for value in values {
        normalize_required_text(field, value)?;
    }
    Ok(())
}
