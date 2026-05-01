use super::common::ReviewRequirement;
use crate::{CoreError, Result};

pub(super) fn require_declared_scope(scope_declared: Option<bool>) -> Result<()> {
    if scope_declared != Some(true) {
        return Err(CoreError::malformed_field(
            "scope",
            "accepted object requires an explicit scope",
        ));
    }
    Ok(())
}

pub(super) fn require_min_len(field: &'static str, len: usize, minimum: usize) -> Result<()> {
    if len < minimum {
        return Err(CoreError::malformed_field(
            field,
            format!("expected at least {minimum} entries"),
        ));
    }
    Ok(())
}

pub(super) fn require_non_empty<T>(field: &'static str, values: &[T]) -> Result<()> {
    require_min_len(field, values.len(), 1)
}

pub(super) fn require_some<'a, T>(field: &'static str, value: Option<&'a T>) -> Result<&'a T> {
    value.ok_or_else(|| CoreError::malformed_field(field, "field is required"))
}

pub(super) fn require_reviewed(
    review: Option<&ReviewRequirement>,
    field: &'static str,
) -> Result<()> {
    let Some(review) = review else {
        return Err(CoreError::malformed_field(
            field,
            "explicit review record is required",
        ));
    };
    if review.required && review.decision_reason.is_none() {
        return Err(CoreError::malformed_field(
            "review.decision_reason",
            "required review must include a decision reason",
        ));
    }
    Ok(())
}
