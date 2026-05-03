//! Reachability, path walking, and path-pattern matching over a space graph.

mod algorithms;
mod normalized;
mod types;

use higher_graphen_core::CoreError;

pub use types::*;

pub(crate) use normalized::{
    NormalizedCellPattern, NormalizedCycleSearchOptions, NormalizedPathPattern,
    NormalizedPathPatternSegment, NormalizedTraversalOptions,
};

pub(crate) fn malformed(field: &str, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: reason.into(),
    }
}
