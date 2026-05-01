//! Reasoning primitives and engines for HigherGraphen.
//!
//! This crate groups validation and acceptance-oriented APIs that were
//! previously split across several small packages.

pub mod abstract_interpretation;
pub mod completion;
pub mod invariant;
pub mod model_checking;
pub mod obstruction;
