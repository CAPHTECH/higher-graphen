//! Deterministic finite distances between two persistence diagrams.
//!
//! This module compares two multisets of persistence intervals grouped by
//! homology dimension and reports exact bottleneck and Wasserstein distances.

mod algorithm;
mod types;

pub use algorithm::persistence_distance;
pub use types::*;

#[cfg(test)]
mod tests;
