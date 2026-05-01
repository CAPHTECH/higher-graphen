use super::*;

/// Report for a single obligation counterexample query.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationReport {
    obligation_id: Id,
    status: SolveStatus,
    model: Option<Assignment>,
    unknown_reason: Option<String>,
    unsupported_reason: Option<String>,
    usage: SolverUsage,
}

impl ObligationReport {
    /// Creates a report with a counterexample model.
    pub fn satisfiable(obligation_id: Id, model: Assignment, usage: SolverUsage) -> Result<Self> {
        Self::new(
            obligation_id,
            SolveStatus::Satisfiable,
            Some(model),
            None,
            None,
            usage,
        )
    }

    /// Creates a report for a proved obligation.
    pub fn unsatisfiable(obligation_id: Id, usage: SolverUsage) -> Result<Self> {
        Self::new(
            obligation_id,
            SolveStatus::Unsatisfiable,
            None,
            None,
            None,
            usage,
        )
    }

    /// Creates an unknown obligation report.
    pub fn unknown(
        obligation_id: Id,
        reason: impl Into<String>,
        usage: SolverUsage,
    ) -> Result<Self> {
        Self::new(
            obligation_id,
            SolveStatus::Unknown,
            None,
            Some(required_text("unknown_reason", reason)?),
            None,
            usage,
        )
    }

    /// Creates an unsupported obligation report.
    pub fn unsupported(
        obligation_id: Id,
        reason: impl Into<String>,
        usage: SolverUsage,
    ) -> Result<Self> {
        Self::new(
            obligation_id,
            SolveStatus::Unsupported,
            None,
            None,
            Some(required_text("unsupported_reason", reason)?),
            usage,
        )
    }

    fn new(
        obligation_id: Id,
        status: SolveStatus,
        model: Option<Assignment>,
        unknown_reason: Option<String>,
        unsupported_reason: Option<String>,
        usage: SolverUsage,
    ) -> Result<Self> {
        let report = Self {
            obligation_id,
            status,
            model,
            unknown_reason: optional_text("unknown_reason", unknown_reason)?,
            unsupported_reason: optional_text("unsupported_reason", unsupported_reason)?,
            usage,
        };
        report.validate()?;
        Ok(report)
    }

    /// Returns the obligation identifier.
    #[must_use]
    pub fn obligation_id(&self) -> &Id {
        &self.obligation_id
    }

    /// Returns the obligation query status.
    #[must_use]
    pub fn status(&self) -> SolveStatus {
        self.status
    }

    /// Returns the counterexample model when one was found.
    #[must_use]
    pub fn model(&self) -> Option<&Assignment> {
        self.model.as_ref()
    }

    /// Returns the unknown diagnostic when present.
    #[must_use]
    pub fn unknown_reason(&self) -> Option<&str> {
        self.unknown_reason.as_deref()
    }

    /// Returns the unsupported diagnostic when present.
    #[must_use]
    pub fn unsupported_reason(&self) -> Option<&str> {
        self.unsupported_reason.as_deref()
    }

    /// Returns finite solver usage for this obligation query.
    #[must_use]
    pub fn usage(&self) -> SolverUsage {
        self.usage
    }

    /// Validates that status-specific fields agree with the status.
    pub fn validate(&self) -> Result<()> {
        validate_status_payload(
            self.status,
            self.model.as_ref(),
            self.unknown_reason.as_deref(),
            self.unsupported_reason.as_deref(),
            self.model.is_some(),
        )
    }
}

impl<'de> Deserialize<'de> for ObligationReport {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            obligation_id: Id,
            status: SolveStatus,
            model: Option<Assignment>,
            unknown_reason: Option<String>,
            unsupported_reason: Option<String>,
            usage: SolverUsage,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(
            wire.obligation_id,
            wire.status,
            wire.model,
            wire.unknown_reason,
            wire.unsupported_reason,
            wire.usage,
        )
        .map_err(serde::de::Error::custom)
    }
}

/// Solver report for a whole problem.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SolveReport {
    problem_id: Id,
    status: SolveStatus,
    model: Option<Assignment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    obligation_results: Vec<ObligationReport>,
    unknown_reason: Option<String>,
    unsupported_reason: Option<String>,
    usage: SolverUsage,
}

impl SolveReport {
    /// Creates a satisfiable report with a model.
    pub fn satisfiable(problem_id: Id, model: Assignment, usage: SolverUsage) -> Result<Self> {
        Self::new(
            problem_id,
            SolveStatus::Satisfiable,
            Some(model),
            Vec::new(),
            None,
            None,
            usage,
        )
    }

    /// Creates an unsatisfiable report.
    pub fn unsatisfiable(problem_id: Id, usage: SolverUsage) -> Result<Self> {
        Self::new(
            problem_id,
            SolveStatus::Unsatisfiable,
            None,
            Vec::new(),
            None,
            None,
            usage,
        )
    }

    /// Creates an unknown report with a diagnostic reason.
    pub fn unknown(problem_id: Id, reason: impl Into<String>, usage: SolverUsage) -> Result<Self> {
        Self::new(
            problem_id,
            SolveStatus::Unknown,
            None,
            Vec::new(),
            Some(required_text("unknown_reason", reason)?),
            None,
            usage,
        )
    }

    /// Creates an unsupported report with a diagnostic reason.
    pub fn unsupported(
        problem_id: Id,
        reason: impl Into<String>,
        usage: SolverUsage,
    ) -> Result<Self> {
        Self::new(
            problem_id,
            SolveStatus::Unsupported,
            None,
            Vec::new(),
            None,
            Some(required_text("unsupported_reason", reason)?),
            usage,
        )
    }

    fn for_obligations(
        problem_id: Id,
        obligation_results: Vec<ObligationReport>,
        limits: ResourceLimits,
    ) -> Result<Self> {
        let status = obligation_problem_status(&obligation_results);
        let unknown_reason = obligation_results
            .iter()
            .find_map(|result| result.unknown_reason().map(str::to_owned));
        let unsupported_reason = obligation_results
            .iter()
            .find_map(|result| result.unsupported_reason().map(str::to_owned));
        let usage = SolverUsage::aggregate(
            obligation_results.iter().map(ObligationReport::usage_ref),
            limits,
        );

        Self::new(
            problem_id,
            status,
            None,
            obligation_results,
            unknown_reason,
            unsupported_reason,
            usage,
        )
    }

    fn new(
        problem_id: Id,
        status: SolveStatus,
        model: Option<Assignment>,
        obligation_results: Vec<ObligationReport>,
        unknown_reason: Option<String>,
        unsupported_reason: Option<String>,
        usage: SolverUsage,
    ) -> Result<Self> {
        let report = Self {
            problem_id,
            status,
            model,
            obligation_results,
            unknown_reason: optional_text("unknown_reason", unknown_reason)?,
            unsupported_reason: optional_text("unsupported_reason", unsupported_reason)?,
            usage,
        };
        report.validate()?;
        Ok(report)
    }

    /// Returns the problem identifier.
    #[must_use]
    pub fn problem_id(&self) -> &Id {
        &self.problem_id
    }

    /// Returns the overall problem status.
    #[must_use]
    pub fn status(&self) -> SolveStatus {
        self.status
    }

    /// Returns a satisfying model for clause or formula problems.
    #[must_use]
    pub fn model(&self) -> Option<&Assignment> {
        self.model.as_ref()
    }

    /// Returns per-obligation reports.
    #[must_use]
    pub fn obligation_results(&self) -> &[ObligationReport] {
        &self.obligation_results
    }

    /// Returns the unknown diagnostic when present.
    #[must_use]
    pub fn unknown_reason(&self) -> Option<&str> {
        self.unknown_reason.as_deref()
    }

    /// Returns the unsupported diagnostic when present.
    #[must_use]
    pub fn unsupported_reason(&self) -> Option<&str> {
        self.unsupported_reason.as_deref()
    }

    /// Returns finite solver usage for the whole problem.
    #[must_use]
    pub fn usage(&self) -> SolverUsage {
        self.usage
    }

    /// Validates that status-specific fields agree with the status.
    pub fn validate(&self) -> Result<()> {
        if !self.obligation_results.is_empty() {
            let expected = obligation_problem_status(&self.obligation_results);
            if self.status != expected {
                return Err(malformed_field(
                    "status",
                    format!(
                        "status {} does not match aggregate obligation status {}",
                        self.status.as_str(),
                        expected.as_str()
                    ),
                ));
            }
        }

        let satisfiable_has_witness = self.model.is_some()
            || self
                .obligation_results
                .iter()
                .any(|result| result.status() == SolveStatus::Satisfiable);
        validate_status_payload(
            self.status,
            self.model.as_ref(),
            self.unknown_reason.as_deref(),
            self.unsupported_reason.as_deref(),
            satisfiable_has_witness,
        )?;

        for result in &self.obligation_results {
            result.validate()?;
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for SolveReport {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            problem_id: Id,
            status: SolveStatus,
            model: Option<Assignment>,
            #[serde(default)]
            obligation_results: Vec<ObligationReport>,
            unknown_reason: Option<String>,
            unsupported_reason: Option<String>,
            usage: SolverUsage,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(
            wire.problem_id,
            wire.status,
            wire.model,
            wire.obligation_results,
            wire.unknown_reason,
            wire.unsupported_reason,
            wire.usage,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl ObligationReport {
    fn usage_ref(&self) -> &SolverUsage {
        &self.usage
    }
}

/// Common adapter boundary for internal and future external solvers.
pub trait ProverAdapter {
    /// Solves the supplied problem and returns a conservative report.
    fn solve(&self, problem: &ProverProblem) -> Result<SolveReport>;
}

/// Deterministic finite Boolean solver that requires no external solver.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FiniteProver {
    /// Resource bounds for exhaustive enumeration.
    pub limits: ResourceLimits,
}

impl FiniteProver {
    /// Creates a finite prover with explicit resource limits.
    #[must_use]
    pub fn new(limits: ResourceLimits) -> Self {
        Self { limits }
    }

    fn solve_clauses(&self, problem_id: &Id, clauses: &[Clause]) -> Result<SolveReport> {
        let propositions = propositions_from_clauses(clauses);
        let outcome = self.search_satisfying_assignment(propositions, |assignment| {
            for clause in clauses {
                if !clause.evaluate(assignment)? {
                    return Ok(false);
                }
            }

            Ok(true)
        })?;

        self.report_from_search(problem_id.clone(), outcome)
    }

    fn solve_formula(&self, problem_id: &Id, formula: &BooleanFormula) -> Result<SolveReport> {
        let propositions = propositions_from_formula(formula);
        let outcome = self.search_satisfying_assignment(propositions, |assignment| {
            formula.evaluate(assignment)
        })?;

        self.report_from_search(problem_id.clone(), outcome)
    }

    fn solve_obligations(
        &self,
        problem_id: &Id,
        assumptions: &[BooleanFormula],
        obligations: &[Obligation],
    ) -> Result<SolveReport> {
        let mut results = Vec::with_capacity(obligations.len());

        for obligation in obligations {
            let counterexample_formula = obligation.counterexample_formula(assumptions);
            let propositions = propositions_from_formula(&counterexample_formula);
            let outcome = self.search_satisfying_assignment(propositions, |assignment| {
                counterexample_formula.evaluate(assignment)
            })?;
            let report = match outcome {
                SearchOutcome::Satisfied { model, usage } => {
                    ObligationReport::satisfiable(obligation.id.clone(), model, usage)?
                }
                SearchOutcome::Exhausted { usage } => {
                    ObligationReport::unsatisfiable(obligation.id.clone(), usage)?
                }
                SearchOutcome::Unknown { reason, usage } => {
                    ObligationReport::unknown(obligation.id.clone(), reason, usage)?
                }
            };
            results.push(report);
        }

        SolveReport::for_obligations(problem_id.clone(), results, self.limits)
    }

    fn report_from_search(&self, problem_id: Id, outcome: SearchOutcome) -> Result<SolveReport> {
        match outcome {
            SearchOutcome::Satisfied { model, usage } => {
                SolveReport::satisfiable(problem_id, model, usage)
            }
            SearchOutcome::Exhausted { usage } => SolveReport::unsatisfiable(problem_id, usage),
            SearchOutcome::Unknown { reason, usage } => {
                SolveReport::unknown(problem_id, reason, usage)
            }
        }
    }

    fn search_satisfying_assignment<F>(
        &self,
        proposition_ids: Vec<Id>,
        mut is_satisfied: F,
    ) -> Result<SearchOutcome>
    where
        F: FnMut(&Assignment) -> Result<bool>,
    {
        let mut usage = SolverUsage::new(proposition_ids.len(), self.limits);
        if proposition_ids.len() > self.limits.max_propositions {
            return Ok(SearchOutcome::Unknown {
                reason: format!(
                    "proposition count {} exceeds finite solver limit {}",
                    proposition_ids.len(),
                    self.limits.max_propositions
                ),
                usage,
            });
        }

        let Some(total_assignments) = assignment_count(proposition_ids.len()) else {
            return Ok(SearchOutcome::Unknown {
                reason: format!(
                    "assignment count for {} propositions exceeds platform capacity",
                    proposition_ids.len()
                ),
                usage,
            });
        };

        for ordinal in 0..total_assignments {
            if usage.assignments_checked >= self.limits.max_assignments {
                return Ok(SearchOutcome::Unknown {
                    reason: format!(
                        "assignment search reached finite solver limit {} before exhaustion",
                        self.limits.max_assignments
                    ),
                    usage,
                });
            }

            let assignment = assignment_for(&proposition_ids, ordinal);
            usage.assignments_checked += 1;

            if is_satisfied(&assignment)? {
                return Ok(SearchOutcome::Satisfied {
                    model: assignment,
                    usage,
                });
            }
        }

        usage.exhaustive = true;
        Ok(SearchOutcome::Exhausted { usage })
    }
}

impl Default for FiniteProver {
    fn default() -> Self {
        Self::new(ResourceLimits::default())
    }
}

impl ProverAdapter for FiniteProver {
    fn solve(&self, problem: &ProverProblem) -> Result<SolveReport> {
        match &problem.kind {
            ProverProblemKind::Clauses { clauses } => self.solve_clauses(&problem.id, clauses),
            ProverProblemKind::Formula { formula } => self.solve_formula(&problem.id, formula),
            ProverProblemKind::Obligations {
                assumptions,
                obligations,
            } => self.solve_obligations(&problem.id, assumptions, obligations),
            ProverProblemKind::Unsupported {
                problem: unsupported,
            } => SolveReport::unsupported(
                problem.id.clone(),
                unsupported.diagnostic(),
                SolverUsage::not_started(self.limits),
            ),
        }
    }
}

enum SearchOutcome {
    Satisfied {
        model: Assignment,
        usage: SolverUsage,
    },
    Exhausted {
        usage: SolverUsage,
    },
    Unknown {
        reason: String,
        usage: SolverUsage,
    },
}
