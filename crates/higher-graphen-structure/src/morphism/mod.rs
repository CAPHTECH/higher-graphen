//! Structure mappings, composition, preservation checks, lost structure, and
//! distortion for HigherGraphen.

mod candidates;
mod composition;
mod diagram;
mod helpers;
mod pullback;
mod pushout;
mod types;

pub(super) use crate::space::{
    Cell, Complex, ComplexType, Dimension, Incidence, IncidenceOrientation, Space,
};
pub(super) use higher_graphen_core::{Id, Provenance, ReviewStatus};
pub(super) use std::collections::{BTreeMap, BTreeSet};

pub use candidates::*;
pub use composition::*;
pub use diagram::*;
pub use pullback::*;
pub use pushout::*;
pub use types::*;

#[cfg(test)]
mod tests;
