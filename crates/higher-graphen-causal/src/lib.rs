//! Cautious causal graph reasoning records for HigherGraphen.
//!
//! The crate records observations and claims separately. An observed
//! correlation can support an association report, but this kernel never
//! promotes correlation by itself into a supported causal claim.
//! Statistical effect estimation, identifiability proof search, and
//! domain-specific causal semantics are intentionally out of scope.

use higher_graphen_core::{CoreError, Id, Provenance, Result, Severity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Obstruction type emitted when a causal claim has no explicit causal support.
pub const UNSUPPORTED_CAUSAL_CLAIM_OBSTRUCTION: &str = "unsupported_causal_claim";
/// Obstruction type emitted when a claim is blocked by active unadjusted confounders.
pub const CONFOUNDED_OBSTRUCTION: &str = "confounded";
/// Obstruction type emitted when only association evidence is available.
pub const CORRELATION_ONLY_OBSTRUCTION: &str = "correlation_only";
/// Obstruction type emitted when an intervention has no observed outcome.
pub const UNSUPPORTED_INTERVENTION_CONCLUSION_OBSTRUCTION: &str =
    "unsupported_intervention_conclusion";

mod graph;
mod model;

pub use graph::CausalGraph;
pub use model::*;

fn ensure_unique_ids<'a, I>(field: &str, ids: I) -> Result<()>
where
    I: IntoIterator<Item = &'a Id>,
{
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id.clone()) {
            return Err(malformed(
                field,
                format!("duplicate identifier {id} is not allowed"),
            ));
        }
    }
    Ok(())
}

fn required_text(field: &str, value: impl Into<String>) -> Result<String> {
    let raw = value.into();
    required_text_ref(field, &raw)
}

fn required_text_ref(field: &str, value: &str) -> Result<String> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        Err(malformed(field, "value must not be empty after trimming"))
    } else {
        Ok(normalized)
    }
}

fn ensure_finite(field: &str, value: f64) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(malformed(field, "value must be finite"))
    }
}

fn unique_ids(ids: impl IntoIterator<Item = Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn normalized_ids(ids: &[Id]) -> Vec<Id> {
    unique_ids(ids.iter().cloned())
}

fn normalize_ids_in_place(ids: &mut Vec<Id>) {
    *ids = normalized_ids(ids);
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn join_ids(ids: &[Id]) -> String {
    ids.iter().map(Id::as_str).collect::<Vec<_>>().join(", ")
}

fn malformed(field: impl Into<String>, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.into(),
        reason: reason.into(),
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests;
