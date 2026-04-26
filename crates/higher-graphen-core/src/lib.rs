//! Shared primitive types and contracts for HigherGraphen.

mod confidence;
mod error;
mod id;
mod provenance;
mod review;
mod source;
mod text;

pub use confidence::Confidence;
pub use error::{CoreError, Result};
pub use id::Id;
pub use provenance::Provenance;
pub use review::{ReviewStatus, Severity};
pub use source::{SourceKind, SourceRef};

#[cfg(test)]
mod tests;
