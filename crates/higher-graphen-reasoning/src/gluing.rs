//! Deterministic gluing checks for correspondence cells.

mod abstract_gluing;
mod structural_gluing;
#[cfg(test)]
mod tests;

pub use abstract_gluing::attempt_gluing;
pub use structural_gluing::{attempt_structural_gluing, StructuralGluing};
