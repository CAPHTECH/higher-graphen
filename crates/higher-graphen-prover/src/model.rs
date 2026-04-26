use super::*;

/// Default number of Boolean propositions the internal finite solver will enumerate.
pub const DEFAULT_MAX_PROPOSITIONS: usize = 12;
/// Default number of assignments the internal finite solver may inspect.
pub const DEFAULT_MAX_ASSIGNMENTS: usize = 4_096;

/// Truth assignment returned as a model or counterexample.
pub type Assignment = BTreeMap<Id, bool>;

/// Outcome vocabulary shared by SAT, SMT-like, and theorem-proving adapters.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SolveStatus {
    /// The supplied constraints have at least one model.
    Satisfiable,
    /// The supplied constraints have no model under the checked semantics.
    Unsatisfiable,
    /// The adapter could not finish a sound answer within its resource limits.
    Unknown,
    /// The adapter does not support the requested theory or problem shape.
    Unsupported,
}

impl SolveStatus {
    /// Stable lower snake case representation used by serde and text protocols.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Satisfiable => "satisfiable",
            Self::Unsatisfiable => "unsatisfiable",
            Self::Unknown => "unknown",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Bounded resources used by the internal finite solver.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLimits {
    /// Maximum number of distinct propositions the internal solver may enumerate.
    pub max_propositions: usize,
    /// Maximum number of assignments the internal solver may inspect.
    pub max_assignments: usize,
}

impl ResourceLimits {
    /// Creates non-zero resource limits.
    pub fn new(max_propositions: usize, max_assignments: usize) -> Result<Self> {
        if max_propositions == 0 {
            return Err(malformed_field(
                "max_propositions",
                "resource limit must be greater than zero",
            ));
        }
        if max_assignments == 0 {
            return Err(malformed_field(
                "max_assignments",
                "resource limit must be greater than zero",
            ));
        }

        Ok(Self {
            max_propositions,
            max_assignments,
        })
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_propositions: DEFAULT_MAX_PROPOSITIONS,
            max_assignments: DEFAULT_MAX_ASSIGNMENTS,
        }
    }
}

/// Resource usage observed while solving one query.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SolverUsage {
    /// Number of distinct propositions in the query.
    pub proposition_count: usize,
    /// Number of assignments inspected by the solver.
    pub assignments_checked: usize,
    /// Assignment limit in force for this query.
    pub max_assignments: usize,
    /// Proposition limit in force for this query.
    pub max_propositions: usize,
    /// True when the finite assignment space was exhausted.
    pub exhaustive: bool,
}

impl SolverUsage {
    pub(crate) fn new(proposition_count: usize, limits: ResourceLimits) -> Self {
        Self {
            proposition_count,
            assignments_checked: 0,
            max_assignments: limits.max_assignments,
            max_propositions: limits.max_propositions,
            exhaustive: false,
        }
    }

    pub(crate) fn not_started(limits: ResourceLimits) -> Self {
        Self::new(0, limits)
    }

    pub(crate) fn aggregate<'a>(
        usages: impl IntoIterator<Item = &'a Self>,
        limits: ResourceLimits,
    ) -> Self {
        let mut aggregate = Self::not_started(limits);
        let mut saw_usage = false;
        aggregate.exhaustive = true;

        for usage in usages {
            saw_usage = true;
            aggregate.proposition_count = aggregate.proposition_count.max(usage.proposition_count);
            aggregate.assignments_checked += usage.assignments_checked;
            aggregate.exhaustive &= usage.exhaustive;
        }

        if !saw_usage {
            aggregate.exhaustive = true;
        }

        aggregate
    }
}

/// One signed Boolean proposition inside a clause.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Literal {
    /// Proposition identifier.
    pub proposition_id: Id,
    /// True for a positive literal and false for a negated literal.
    pub polarity: bool,
}

impl Literal {
    /// Creates a signed literal.
    #[must_use]
    pub fn new(proposition_id: Id, polarity: bool) -> Self {
        Self {
            proposition_id,
            polarity,
        }
    }

    /// Creates a positive literal.
    #[must_use]
    pub fn positive(proposition_id: Id) -> Self {
        Self::new(proposition_id, true)
    }

    /// Creates a negated literal.
    #[must_use]
    pub fn negative(proposition_id: Id) -> Self {
        Self::new(proposition_id, false)
    }

    /// Returns this literal with the opposite polarity.
    #[must_use]
    pub fn negated(&self) -> Self {
        Self::new(self.proposition_id.clone(), !self.polarity)
    }

    /// Evaluates this literal against a complete assignment.
    pub fn evaluate(&self, assignment: &Assignment) -> Result<bool> {
        let value = assignment.get(&self.proposition_id).ok_or_else(|| {
            malformed_field(
                "assignment",
                format!("missing value for proposition {}", self.proposition_id),
            )
        })?;

        Ok(if self.polarity { *value } else { !*value })
    }

    pub(crate) fn collect_propositions(&self, propositions: &mut BTreeSet<Id>) {
        propositions.insert(self.proposition_id.clone());
    }
}

/// A disjunction of signed literals.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Clause {
    /// Literals in the disjunction. Empty clauses evaluate to false.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub literals: Vec<Literal>,
}

impl Clause {
    /// Creates a clause from literals.
    #[must_use]
    pub fn new(literals: Vec<Literal>) -> Self {
        Self { literals }
    }

    /// Creates a unit clause from one literal.
    #[must_use]
    pub fn unit(literal: Literal) -> Self {
        Self {
            literals: vec![literal],
        }
    }

    /// Evaluates this clause against a complete assignment.
    pub fn evaluate(&self, assignment: &Assignment) -> Result<bool> {
        for literal in &self.literals {
            if literal.evaluate(assignment)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Converts this clause to an equivalent Boolean formula.
    #[must_use]
    pub fn to_formula(&self) -> BooleanFormula {
        BooleanFormula::or(
            self.literals
                .iter()
                .cloned()
                .map(BooleanFormula::from)
                .collect(),
        )
    }

    pub(crate) fn collect_propositions(&self, propositions: &mut BTreeSet<Id>) {
        for literal in &self.literals {
            literal.collect_propositions(propositions);
        }
    }
}

/// Finite Boolean formula language supported by the internal solver.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "spec", rename_all = "snake_case")]
pub enum BooleanFormula {
    /// Formula that always evaluates to true.
    True,
    /// Formula that always evaluates to false.
    False,
    /// Atomic proposition identified by a stable ID.
    Atom(Id),
    /// Negation.
    Not(Box<BooleanFormula>),
    /// Conjunction. An empty conjunction evaluates to true.
    And(Vec<BooleanFormula>),
    /// Disjunction. An empty disjunction evaluates to false.
    Or(Vec<BooleanFormula>),
    /// Material implication.
    Implies {
        /// Antecedent formula.
        premise: Box<BooleanFormula>,
        /// Consequent formula.
        conclusion: Box<BooleanFormula>,
    },
    /// Logical equivalence.
    Iff {
        /// Left-hand formula.
        left: Box<BooleanFormula>,
        /// Right-hand formula.
        right: Box<BooleanFormula>,
    },
}

impl BooleanFormula {
    /// Creates an atomic proposition formula.
    #[must_use]
    pub fn atom(proposition_id: Id) -> Self {
        Self::Atom(proposition_id)
    }

    /// Creates a negated formula.
    #[must_use]
    pub fn negate(formula: Self) -> Self {
        Self::Not(Box::new(formula))
    }

    /// Creates a conjunction.
    #[must_use]
    pub fn and(terms: Vec<Self>) -> Self {
        Self::And(terms)
    }

    /// Creates a disjunction.
    #[must_use]
    pub fn or(terms: Vec<Self>) -> Self {
        Self::Or(terms)
    }

    /// Creates a material implication.
    #[must_use]
    pub fn implies(premise: Self, conclusion: Self) -> Self {
        Self::Implies {
            premise: Box::new(premise),
            conclusion: Box::new(conclusion),
        }
    }

    /// Creates a logical equivalence.
    #[must_use]
    pub fn iff(left: Self, right: Self) -> Self {
        Self::Iff {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Evaluates this formula against a complete assignment.
    pub fn evaluate(&self, assignment: &Assignment) -> Result<bool> {
        match self {
            Self::True => Ok(true),
            Self::False => Ok(false),
            Self::Atom(proposition_id) => {
                assignment.get(proposition_id).copied().ok_or_else(|| {
                    malformed_field(
                        "assignment",
                        format!("missing value for proposition {proposition_id}"),
                    )
                })
            }
            Self::Not(formula) => Ok(!formula.evaluate(assignment)?),
            Self::And(terms) => {
                for term in terms {
                    if !term.evaluate(assignment)? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            Self::Or(terms) => {
                for term in terms {
                    if term.evaluate(assignment)? {
                        return Ok(true);
                    }
                }

                Ok(false)
            }
            Self::Implies {
                premise,
                conclusion,
            } => Ok(!premise.evaluate(assignment)? || conclusion.evaluate(assignment)?),
            Self::Iff { left, right } => {
                Ok(left.evaluate(assignment)? == right.evaluate(assignment)?)
            }
        }
    }

    pub(crate) fn collect_propositions(&self, propositions: &mut BTreeSet<Id>) {
        match self {
            Self::True | Self::False => {}
            Self::Atom(proposition_id) => {
                propositions.insert(proposition_id.clone());
            }
            Self::Not(formula) => formula.collect_propositions(propositions),
            Self::And(terms) | Self::Or(terms) => {
                for term in terms {
                    term.collect_propositions(propositions);
                }
            }
            Self::Implies {
                premise,
                conclusion,
            } => {
                premise.collect_propositions(propositions);
                conclusion.collect_propositions(propositions);
            }
            Self::Iff { left, right } => {
                left.collect_propositions(propositions);
                right.collect_propositions(propositions);
            }
        }
    }
}

impl From<Literal> for BooleanFormula {
    fn from(literal: Literal) -> Self {
        let atom = Self::atom(literal.proposition_id);
        if literal.polarity {
            atom
        } else {
            Self::negate(atom)
        }
    }
}

/// One theorem-proving obligation interpreted as a counterexample query.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    /// Stable obligation identifier.
    pub id: Id,
    /// Additional assumptions scoped to this obligation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<BooleanFormula>,
    /// Formula that should follow from global and local assumptions.
    pub conclusion: BooleanFormula,
    /// Optional human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Obligation {
    /// Creates an obligation with no local assumptions.
    #[must_use]
    pub fn new(id: Id, conclusion: BooleanFormula) -> Self {
        Self {
            id,
            assumptions: Vec::new(),
            conclusion,
            description: None,
        }
    }

    /// Returns this obligation with local assumptions.
    #[must_use]
    pub fn with_assumptions(mut self, assumptions: Vec<BooleanFormula>) -> Self {
        self.assumptions = assumptions;
        self
    }

    /// Returns this obligation with a validated description.
    pub fn with_description(mut self, description: impl Into<String>) -> Result<Self> {
        self.description = Some(required_text("description", description)?);
        Ok(self)
    }

    pub(crate) fn counterexample_formula(
        &self,
        global_assumptions: &[BooleanFormula],
    ) -> BooleanFormula {
        let mut terms = Vec::with_capacity(global_assumptions.len() + self.assumptions.len() + 1);
        terms.extend(global_assumptions.iter().cloned());
        terms.extend(self.assumptions.iter().cloned());
        terms.push(BooleanFormula::negate(self.conclusion.clone()));
        BooleanFormula::and(terms)
    }
}

impl<'de> Deserialize<'de> for Obligation {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            id: Id,
            #[serde(default)]
            assumptions: Vec<BooleanFormula>,
            conclusion: BooleanFormula,
            description: Option<String>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let description =
            optional_text("description", wire.description).map_err(serde::de::Error::custom)?;

        Ok(Self {
            id: wire.id,
            assumptions: wire.assumptions,
            conclusion: wire.conclusion,
            description,
        })
    }
}

/// Explicit unsupported problem payload for optional future solver adapters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedProblem {
    /// Theory or feature that requires a solver not provided by this crate.
    pub theory: String,
    /// Diagnostic reason for the unsupported result.
    pub reason: String,
}

impl UnsupportedProblem {
    /// Creates an unsupported problem payload with validated text.
    pub fn new(theory: impl Into<String>, reason: impl Into<String>) -> Result<Self> {
        Ok(Self {
            theory: required_text("theory", theory)?,
            reason: required_text("reason", reason)?,
        })
    }

    pub(crate) fn diagnostic(&self) -> String {
        format!("unsupported theory {}: {}", self.theory, self.reason)
    }
}

impl<'de> Deserialize<'de> for UnsupportedProblem {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            theory: String,
            reason: String,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.theory, wire.reason).map_err(serde::de::Error::custom)
    }
}

/// Solver-neutral problem shapes accepted by the bridge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProverProblemKind {
    /// A finite CNF problem represented as a conjunction of clauses.
    Clauses {
        /// Clauses that must all hold.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        clauses: Vec<Clause>,
    },
    /// A finite Boolean formula satisfiability problem.
    Formula {
        /// Formula that must hold.
        formula: BooleanFormula,
    },
    /// Theorem-style obligations checked by searching for counterexamples.
    Obligations {
        /// Global assumptions shared by every obligation.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        assumptions: Vec<BooleanFormula>,
        /// Obligations to check.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        obligations: Vec<Obligation>,
    },
    /// Problem shape that is known but unsupported by the internal finite solver.
    Unsupported {
        /// Unsupported theory payload.
        problem: UnsupportedProblem,
    },
}

/// Solver-neutral SAT, SMT-like, or theorem-proving problem record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProverProblem {
    /// Stable problem identifier.
    pub id: Id,
    /// Problem payload.
    pub kind: ProverProblemKind,
    /// Optional source and review metadata for the problem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl ProverProblem {
    /// Creates a CNF clause problem.
    #[must_use]
    pub fn clauses(id: Id, clauses: Vec<Clause>) -> Self {
        Self {
            id,
            kind: ProverProblemKind::Clauses { clauses },
            provenance: None,
        }
    }

    /// Creates a finite Boolean formula satisfiability problem.
    #[must_use]
    pub fn formula(id: Id, formula: BooleanFormula) -> Self {
        Self {
            id,
            kind: ProverProblemKind::Formula { formula },
            provenance: None,
        }
    }

    /// Creates an obligation problem with shared assumptions.
    #[must_use]
    pub fn obligations(
        id: Id,
        assumptions: Vec<BooleanFormula>,
        obligations: Vec<Obligation>,
    ) -> Self {
        Self {
            id,
            kind: ProverProblemKind::Obligations {
                assumptions,
                obligations,
            },
            provenance: None,
        }
    }

    /// Creates an explicitly unsupported problem.
    pub fn unsupported(
        id: Id,
        theory: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            kind: ProverProblemKind::Unsupported {
                problem: UnsupportedProblem::new(theory, reason)?,
            },
            provenance: None,
        })
    }

    /// Returns this problem with source and review metadata.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}
