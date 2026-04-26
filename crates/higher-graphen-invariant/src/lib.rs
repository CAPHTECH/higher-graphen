//! Invariants, constraints, invariant checks, and constraint check results for HigherGraphen.

use higher_graphen_core::{CoreError, Id, Provenance, Result, Severity};
use serde::{Deserialize, Deserializer, Serialize};

/// Scope where an invariant must hold or be preserved.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InvariantScope {
    /// The invariant applies to a whole space.
    Space {
        /// Space being checked.
        space_id: Id,
    },
    /// The invariant applies to a selected set of cells in a space.
    Cells {
        /// Space containing the selected cells.
        space_id: Id,
        /// Cells that define the scoped invariant boundary.
        cell_ids: Vec<Id>,
    },
    /// The invariant applies within selected contexts in a space.
    Contexts {
        /// Space containing the selected contexts.
        space_id: Id,
        /// Contexts where the invariant is meaningful.
        context_ids: Vec<Id>,
    },
    /// The invariant applies to preservation across a morphism boundary.
    Morphism {
        /// Source space of the preservation boundary.
        source_space_id: Id,
        /// Target space of the preservation boundary.
        target_space_id: Id,
        /// Optional morphism identifier when the boundary has been materialized.
        #[serde(skip_serializing_if = "Option::is_none")]
        morphism_id: Option<Id>,
    },
}

/// Scope where a constraint can be evaluated.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConstraintScope {
    /// The constraint applies to a whole space.
    Space {
        /// Space being checked.
        space_id: Id,
    },
    /// The constraint applies to selected cells in a space.
    Cells {
        /// Space containing the selected cells.
        space_id: Id,
        /// Cells that define the constraint boundary.
        cell_ids: Vec<Id>,
    },
    /// The constraint applies within selected contexts in a space.
    Contexts {
        /// Space containing the selected contexts.
        space_id: Id,
        /// Contexts where the constraint is meaningful.
        context_ids: Vec<Id>,
    },
}

/// Property that should remain true for its scope.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Invariant {
    /// Stable invariant identifier.
    pub id: Id,
    /// Human-readable invariant name.
    pub name: String,
    /// Optional explanation of the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Scope where the invariant applies.
    pub scope: InvariantScope,
    /// Impact when the invariant is violated.
    pub severity: Severity,
    /// Source and review metadata for the invariant definition.
    pub provenance: Provenance,
}

impl Invariant {
    /// Creates an invariant definition with no description.
    pub fn new(
        id: Id,
        name: impl Into<String>,
        scope: InvariantScope,
        severity: Severity,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            scope,
            severity,
            provenance,
        }
    }
}

/// Condition that can be evaluated and reported as a violation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    /// Stable constraint identifier.
    pub id: Id,
    /// Human-readable constraint name.
    pub name: String,
    /// Optional explanation of the condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Scope where the constraint applies.
    pub scope: ConstraintScope,
    /// Impact when the constraint is violated.
    pub severity: Severity,
    /// Source and review metadata for the constraint definition.
    pub provenance: Provenance,
}

impl Constraint {
    /// Creates a constraint definition with no description.
    pub fn new(
        id: Id,
        name: impl Into<String>,
        scope: ConstraintScope,
        severity: Severity,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            scope,
            severity,
            provenance,
        }
    }
}

/// Input for invariant and constraint checks.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckInput {
    /// Space where the check runs.
    pub space_id: Id,
    /// Invariants selected for evaluation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariant_ids: Vec<Id>,
    /// Constraints selected for evaluation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraint_ids: Vec<Id>,
    /// Changed cells that bound incremental evaluation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_cell_ids: Vec<Id>,
    /// Contexts selected for contextual evaluation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Morphisms related to preservation-oriented checks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_morphism_ids: Vec<Id>,
}

impl CheckInput {
    /// Creates a whole-space check input with no selected checks.
    pub fn new(space_id: Id) -> Self {
        Self {
            space_id,
            invariant_ids: Vec::new(),
            constraint_ids: Vec::new(),
            changed_cell_ids: Vec::new(),
            context_ids: Vec::new(),
            related_morphism_ids: Vec::new(),
        }
    }

    /// Creates a changed-cell-scoped input for incremental evaluation.
    pub fn changed_cells(space_id: Id, changed_cell_ids: Vec<Id>) -> Self {
        Self {
            changed_cell_ids,
            ..Self::new(space_id)
        }
    }

    /// Returns this input with selected invariant identifiers.
    pub fn with_invariants(mut self, invariant_ids: Vec<Id>) -> Self {
        self.invariant_ids = invariant_ids;
        self
    }

    /// Returns this input with selected constraint identifiers.
    pub fn with_constraints(mut self, constraint_ids: Vec<Id>) -> Self {
        self.constraint_ids = constraint_ids;
        self
    }

    /// Returns this input with selected context identifiers.
    pub fn with_contexts(mut self, context_ids: Vec<Id>) -> Self {
        self.context_ids = context_ids;
        self
    }

    /// Returns this input with related morphism identifiers.
    pub fn with_related_morphisms(mut self, related_morphism_ids: Vec<Id>) -> Self {
        self.related_morphism_ids = related_morphism_ids;
        self
    }

    /// Returns true when evaluation is scoped to one or more changed cells.
    pub fn is_changed_cell_scoped(&self) -> bool {
        !self.changed_cell_ids.is_empty()
    }
}

/// Kind of definition targeted by a check result.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckTargetKind {
    /// The result applies to an invariant.
    Invariant,
    /// The result applies to a constraint.
    Constraint,
}

/// Outcome state for a check result.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// The checked invariant or constraint was satisfied.
    Satisfied,
    /// The checked invariant or constraint was violated.
    Violated,
    /// The check could not be evaluated by the current checker.
    Unsupported,
}

/// Structured violation details without constructing an obstruction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Violation {
    /// Human-readable explanation of the failed check.
    pub message: String,
    /// Impact of this violation.
    pub severity: Severity,
    /// Cells where the violation is located.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub location_cell_ids: Vec<Id>,
    /// Contexts where the violation is located.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub location_context_ids: Vec<Id>,
    /// Morphisms related to the violation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_morphism_ids: Vec<Id>,
}

impl Violation {
    /// Creates a violation with no location data.
    pub fn new(message: impl Into<String>, severity: Severity) -> Self {
        Self {
            message: message.into().trim().to_owned(),
            severity,
            location_cell_ids: Vec::new(),
            location_context_ids: Vec::new(),
            related_morphism_ids: Vec::new(),
        }
    }

    /// Returns this violation with cell locations.
    pub fn with_location_cells(mut self, location_cell_ids: Vec<Id>) -> Self {
        self.location_cell_ids = location_cell_ids;
        self
    }

    /// Returns this violation with context locations.
    pub fn with_location_contexts(mut self, location_context_ids: Vec<Id>) -> Self {
        self.location_context_ids = location_context_ids;
        self
    }

    /// Returns this violation with related morphisms.
    pub fn with_related_morphisms(mut self, related_morphism_ids: Vec<Id>) -> Self {
        self.related_morphism_ids = related_morphism_ids;
        self
    }
}

/// Result for a single invariant or constraint check.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckResult {
    /// Kind of checked definition.
    pub target_kind: CheckTargetKind,
    /// Identifier of the checked invariant or constraint.
    pub target_id: Id,
    /// Outcome state of the check.
    pub status: CheckStatus,
    /// Violation details when the status is violated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violation: Option<Violation>,
    /// Explanation when the status is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsupported_reason: Option<String>,
}

impl CheckResult {
    /// Creates a satisfied result.
    pub fn satisfied(target_kind: CheckTargetKind, target_id: Id) -> Self {
        Self {
            target_kind,
            target_id,
            status: CheckStatus::Satisfied,
            violation: None,
            unsupported_reason: None,
        }
    }

    /// Creates a violated result with structured violation details.
    pub fn violated(target_kind: CheckTargetKind, target_id: Id, violation: Violation) -> Self {
        Self {
            target_kind,
            target_id,
            status: CheckStatus::Violated,
            violation: Some(violation),
            unsupported_reason: None,
        }
    }

    /// Creates an unsupported result with a diagnostic reason.
    pub fn unsupported(
        target_kind: CheckTargetKind,
        target_id: Id,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            target_kind,
            target_id,
            status: CheckStatus::Unsupported,
            violation: None,
            unsupported_reason: Some(reason.into().trim().to_owned()),
        }
    }

    /// Returns true when this result is satisfied.
    pub fn is_satisfied(&self) -> bool {
        matches!(self.status, CheckStatus::Satisfied)
    }

    /// Returns true when this result is violated.
    pub fn is_violated(&self) -> bool {
        matches!(self.status, CheckStatus::Violated)
    }

    /// Returns true when this result is unsupported.
    pub fn is_unsupported(&self) -> bool {
        matches!(self.status, CheckStatus::Unsupported)
    }

    /// Validates that the status-specific payload fields agree with `status`.
    pub fn validate(&self) -> Result<()> {
        match self.status {
            CheckStatus::Satisfied => {
                ensure_absent("violation", self.violation.is_none())?;
                ensure_absent("unsupported_reason", self.unsupported_reason.is_none())
            }
            CheckStatus::Violated => {
                ensure_absent("unsupported_reason", self.unsupported_reason.is_none())?;
                let violation = self.violation.as_ref().ok_or_else(|| {
                    malformed_field(
                        "violation",
                        "violated results must include violation details",
                    )
                })?;
                ensure_non_empty("violation.message", &violation.message)
            }
            CheckStatus::Unsupported => {
                ensure_absent("violation", self.violation.is_none())?;
                let reason = self.unsupported_reason.as_deref().ok_or_else(|| {
                    malformed_field(
                        "unsupported_reason",
                        "unsupported results must include a diagnostic reason",
                    )
                })?;
                ensure_non_empty("unsupported_reason", reason)
            }
        }
    }
}

impl<'de> Deserialize<'de> for CheckResult {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            target_kind: CheckTargetKind,
            target_id: Id,
            status: CheckStatus,
            violation: Option<Violation>,
            unsupported_reason: Option<String>,
        }

        let result = {
            let wire = Wire::deserialize(deserializer)?;
            Self {
                target_kind: wire.target_kind,
                target_id: wire.target_id,
                status: wire.status,
                violation: wire.violation,
                unsupported_reason: wire.unsupported_reason,
            }
        };
        result.validate().map_err(serde::de::Error::custom)?;
        Ok(result)
    }
}

fn ensure_absent(field: &'static str, is_absent: bool) -> Result<()> {
    if is_absent {
        Ok(())
    } else {
        Err(malformed_field(
            field,
            "field must be absent for this check status",
        ))
    }
}

fn ensure_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(malformed_field(
            field,
            "value must not be empty after trimming",
        ))
    } else {
        Ok(())
    }
}

fn malformed_field(field: &'static str, reason: &'static str) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use higher_graphen_core::{Confidence, SourceKind, SourceRef};

    fn id(value: &str) -> Id {
        Id::new(value).expect("test id should be valid")
    }

    fn provenance() -> Provenance {
        let source = SourceRef::new(SourceKind::Code);
        let confidence = Confidence::new(1.0).expect("test confidence should be valid");
        Provenance::new(source, confidence)
    }

    #[test]
    fn constructs_invariant_and_constraint_definitions() {
        let invariant = Invariant::new(
            id("inv:acyclic"),
            "Acyclic ownership",
            InvariantScope::Space {
                space_id: id("space:architecture"),
            },
            Severity::High,
            provenance(),
        );
        let constraint = Constraint::new(
            id("constraint:no-cross-context-db"),
            "No cross-context database access",
            ConstraintScope::Contexts {
                space_id: id("space:architecture"),
                context_ids: vec![id("context:billing")],
            },
            Severity::Critical,
            provenance(),
        );

        assert_eq!(invariant.description, None);
        assert_eq!(
            constraint.scope,
            ConstraintScope::Contexts {
                space_id: id("space:architecture"),
                context_ids: vec![id("context:billing")],
            }
        );
    }

    #[test]
    fn builds_changed_cell_scoped_input() {
        let input = CheckInput::changed_cells(
            id("space:architecture"),
            vec![id("cell:service-a"), id("cell:service-b")],
        )
        .with_invariants(vec![id("inv:ownership")])
        .with_constraints(vec![id("constraint:dependency-direction")])
        .with_contexts(vec![id("context:runtime")])
        .with_related_morphisms(vec![id("morphism:migration")]);

        assert!(input.is_changed_cell_scoped());
        assert_eq!(input.changed_cell_ids.len(), 2);
        assert_eq!(input.invariant_ids, vec![id("inv:ownership")]);
        assert_eq!(input.related_morphism_ids, vec![id("morphism:migration")]);
    }

    #[test]
    fn creates_satisfied_violated_and_unsupported_results() {
        let satisfied = CheckResult::satisfied(CheckTargetKind::Invariant, id("inv:shape"));
        let violation = Violation::new("context boundary crossed", Severity::High)
            .with_location_cells(vec![id("cell:repository")])
            .with_location_contexts(vec![id("context:billing")]);
        let violated = CheckResult::violated(
            CheckTargetKind::Constraint,
            id("constraint:boundary"),
            violation,
        );
        let unsupported = CheckResult::unsupported(
            CheckTargetKind::Invariant,
            id("inv:morphism-preservation"),
            " morphism summary missing ",
        );

        assert!(satisfied.is_satisfied());
        assert!(violated.is_violated());
        assert_eq!(
            violated.violation.as_ref().map(|item| &item.message),
            Some(&"context boundary crossed".to_owned())
        );
        assert!(unsupported.is_unsupported());
        assert_eq!(
            unsupported.unsupported_reason,
            Some("morphism summary missing".to_owned())
        );
        assert!(satisfied.validate().is_ok());
        assert!(violated.validate().is_ok());
        assert!(unsupported.validate().is_ok());
    }

    #[test]
    fn validates_status_specific_result_payloads() {
        let satisfied_with_violation = CheckResult {
            violation: Some(Violation::new("should not be present", Severity::Low)),
            ..CheckResult::satisfied(CheckTargetKind::Invariant, id("inv:shape"))
        };
        let violated_without_violation = CheckResult {
            violation: None,
            ..CheckResult::violated(
                CheckTargetKind::Constraint,
                id("constraint:shape"),
                Violation::new("missing", Severity::Medium),
            )
        };
        let unsupported_without_reason = CheckResult {
            unsupported_reason: None,
            ..CheckResult::unsupported(CheckTargetKind::Invariant, id("inv:future"), "missing")
        };
        let unsupported_blank_reason =
            CheckResult::unsupported(CheckTargetKind::Invariant, id("inv:future"), " ");

        assert!(satisfied_with_violation.validate().is_err());
        assert!(violated_without_violation.validate().is_err());
        assert!(unsupported_without_reason.validate().is_err());
        assert!(unsupported_blank_reason.validate().is_err());
    }

    #[test]
    fn deserialization_rejects_status_specific_payload_mismatch() {
        let satisfied_with_violation = serde_json::json!({
            "target_kind": "invariant",
            "target_id": "inv:shape",
            "status": "satisfied",
            "violation": {
                "message": "should not be present",
                "severity": "low"
            }
        });
        let unsupported_without_reason = serde_json::json!({
            "target_kind": "invariant",
            "target_id": "inv:future",
            "status": "unsupported"
        });

        assert!(serde_json::from_value::<CheckResult>(satisfied_with_violation).is_err());
        assert!(serde_json::from_value::<CheckResult>(unsupported_without_reason).is_err());
    }
}
