use higher_graphen_core::{CoreError, Id, Result};
use std::collections::BTreeSet;

pub(super) fn required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();

    if normalized.is_empty() {
        return Err(CoreError::MalformedField {
            field: field.to_owned(),
            reason: "value must not be empty after trimming".to_owned(),
        });
    }

    Ok(normalized)
}

pub(super) fn id_set(ids: impl IntoIterator<Item = Id>) -> BTreeSet<Id> {
    ids.into_iter().collect()
}

pub(super) fn joined_vec<T>(left: &[T], right: &[T]) -> Vec<T>
where
    T: Clone + PartialEq,
{
    let mut items = left.to_vec();
    for item in right {
        if !items.contains(item) {
            items.push(item.clone());
        }
    }
    items
}

pub(super) fn malformed(field: &str, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: reason.into(),
    }
}
