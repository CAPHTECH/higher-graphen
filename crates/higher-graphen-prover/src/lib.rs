//! Conservative SAT, SMT, and theorem-proving bridge records for HigherGraphen.
//!
//! This crate deliberately keeps the built-in solver small: it enumerates
//! finite Boolean assignments, handles CNF clauses and obligation
//! counterexample queries, and records `unknown` when resource limits prevent a
//! complete search. It does not attempt arithmetic SMT, quantifier reasoning,
//! induction, proof certificates, or calls to external solver binaries. Those
//! cases should enter through [`ProverAdapter`] implementations or explicit
//! [`UnsupportedProblem`] records.

use higher_graphen_core::{CoreError, Id, Provenance, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod model;
mod solver;

pub use model::*;
pub use solver::{FiniteProver, ObligationReport, ProverAdapter, SolveReport};

fn obligation_problem_status(results: &[ObligationReport]) -> SolveStatus {
    if results
        .iter()
        .any(|result| result.status() == SolveStatus::Satisfiable)
    {
        SolveStatus::Satisfiable
    } else if results
        .iter()
        .any(|result| result.status() == SolveStatus::Unsupported)
    {
        SolveStatus::Unsupported
    } else if results
        .iter()
        .any(|result| result.status() == SolveStatus::Unknown)
    {
        SolveStatus::Unknown
    } else {
        SolveStatus::Unsatisfiable
    }
}

fn propositions_from_clauses(clauses: &[Clause]) -> Vec<Id> {
    let mut propositions = BTreeSet::new();
    for clause in clauses {
        clause.collect_propositions(&mut propositions);
    }

    propositions.into_iter().collect()
}

fn propositions_from_formula(formula: &BooleanFormula) -> Vec<Id> {
    let mut propositions = BTreeSet::new();
    formula.collect_propositions(&mut propositions);
    propositions.into_iter().collect()
}

fn assignment_count(proposition_count: usize) -> Option<usize> {
    1_usize.checked_shl(proposition_count.try_into().ok()?)
}

fn assignment_for(proposition_ids: &[Id], ordinal: usize) -> Assignment {
    let mut assignment = Assignment::new();

    for (index, proposition_id) in proposition_ids.iter().enumerate() {
        assignment.insert(proposition_id.clone(), ((ordinal >> index) & 1) == 1);
    }

    assignment
}

fn validate_status_payload(
    status: SolveStatus,
    model: Option<&Assignment>,
    unknown_reason: Option<&str>,
    unsupported_reason: Option<&str>,
    satisfiable_has_witness: bool,
) -> Result<()> {
    match status {
        SolveStatus::Satisfiable => {
            ensure_absent("unknown_reason", unknown_reason.is_none())?;
            ensure_absent("unsupported_reason", unsupported_reason.is_none())?;
            if model.is_none() && !satisfiable_has_witness {
                return Err(malformed_field(
                    "model",
                    "satisfiable results must include a model or obligation counterexample",
                ));
            }
            Ok(())
        }
        SolveStatus::Unsatisfiable => {
            ensure_absent("model", model.is_none())?;
            ensure_absent("unknown_reason", unknown_reason.is_none())?;
            ensure_absent("unsupported_reason", unsupported_reason.is_none())
        }
        SolveStatus::Unknown => {
            ensure_absent("model", model.is_none())?;
            ensure_absent("unsupported_reason", unsupported_reason.is_none())?;
            let reason = unknown_reason.ok_or_else(|| {
                malformed_field(
                    "unknown_reason",
                    "unknown results must include a diagnostic reason",
                )
            })?;
            ensure_non_empty("unknown_reason", reason)
        }
        SolveStatus::Unsupported => {
            ensure_absent("model", model.is_none())?;
            ensure_absent("unknown_reason", unknown_reason.is_none())?;
            let reason = unsupported_reason.ok_or_else(|| {
                malformed_field(
                    "unsupported_reason",
                    "unsupported results must include a diagnostic reason",
                )
            })?;
            ensure_non_empty("unsupported_reason", reason)
        }
    }
}

fn ensure_absent(field: &'static str, absent: bool) -> Result<()> {
    if absent {
        Ok(())
    } else {
        Err(malformed_field(
            field,
            "field must be absent for this status",
        ))
    }
}

fn ensure_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(malformed_field(field, "field must not be empty"))
    } else {
        Ok(())
    }
}

fn required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();
    ensure_non_empty(field, &normalized)?;
    Ok(normalized)
}

fn optional_text(field: &'static str, value: Option<String>) -> Result<Option<String>> {
    value.map(|value| required_text(field, value)).transpose()
}

fn malformed_field(field: impl Into<String>, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
