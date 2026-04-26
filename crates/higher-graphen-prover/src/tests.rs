use super::*;
use higher_graphen_core::{Confidence, ReviewStatus, SourceKind, SourceRef};

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

fn atom(value: &str) -> BooleanFormula {
    BooleanFormula::atom(id(value))
}

fn solve(problem: ProverProblem) -> SolveReport {
    FiniteProver::default()
        .solve(&problem)
        .expect("finite solve succeeds")
}

#[test]
fn finite_clauses_report_satisfiable_with_model() {
    let problem = ProverProblem::clauses(
        id("problem.sat"),
        vec![
            Clause::new(vec![Literal::positive(id("a")), Literal::positive(id("b"))]),
            Clause::unit(Literal::negative(id("b"))),
        ],
    );

    let report = solve(problem);

    assert_eq!(report.status(), SolveStatus::Satisfiable);
    let model = report.model().expect("satisfying model");
    assert_eq!(model.get(&id("a")), Some(&true));
    assert_eq!(model.get(&id("b")), Some(&false));
    assert!(!report.usage().exhaustive);
}

#[test]
fn finite_clauses_report_unsatisfiable_after_exhaustion() {
    let problem = ProverProblem::clauses(
        id("problem.unsat"),
        vec![
            Clause::unit(Literal::positive(id("a"))),
            Clause::unit(Literal::negative(id("a"))),
        ],
    );

    let report = solve(problem);

    assert_eq!(report.status(), SolveStatus::Unsatisfiable);
    assert!(report.model().is_none());
    assert!(report.usage().exhaustive);
    assert_eq!(report.usage().assignments_checked, 2);
}

#[test]
fn finite_formula_supports_boolean_connectives() {
    let formula = BooleanFormula::and(vec![
        BooleanFormula::implies(atom("a"), atom("b")),
        BooleanFormula::negate(atom("b")),
    ]);
    let problem = ProverProblem::formula(id("problem.formula"), formula);

    let report = solve(problem);

    assert_eq!(report.status(), SolveStatus::Satisfiable);
    let model = report.model().expect("model");
    assert_eq!(model.get(&id("a")), Some(&false));
    assert_eq!(model.get(&id("b")), Some(&false));
}

#[test]
fn valid_obligation_reports_unsatisfiable_counterexample_query() {
    let assumptions = vec![BooleanFormula::implies(atom("a"), atom("b")), atom("a")];
    let obligation = Obligation::new(id("obligation.b"), atom("b"));
    let problem = ProverProblem::obligations(id("problem.valid"), assumptions, vec![obligation]);

    let report = solve(problem);

    assert_eq!(report.status(), SolveStatus::Unsatisfiable);
    assert_eq!(report.obligation_results().len(), 1);
    assert_eq!(
        report.obligation_results()[0].status(),
        SolveStatus::Unsatisfiable
    );
    assert!(report.obligation_results()[0].usage().exhaustive);
}

#[test]
fn invalid_obligation_reports_satisfiable_counterexample() {
    let assumptions = vec![BooleanFormula::implies(atom("a"), atom("b"))];
    let obligation = Obligation::new(id("obligation.b"), atom("b"));
    let problem = ProverProblem::obligations(id("problem.invalid"), assumptions, vec![obligation]);

    let report = solve(problem);

    assert_eq!(report.status(), SolveStatus::Satisfiable);
    let obligation_report = &report.obligation_results()[0];
    assert_eq!(obligation_report.status(), SolveStatus::Satisfiable);
    let counterexample = obligation_report.model().expect("counterexample");
    assert_eq!(counterexample.get(&id("a")), Some(&false));
    assert_eq!(counterexample.get(&id("b")), Some(&false));
}

#[test]
fn obligation_report_status_must_match_aggregate_status() {
    let malformed = r#"{
        "problem_id": "problem.bad_obligation_status",
        "status": "unsatisfiable",
        "model": null,
        "obligation_results": [{
            "obligation_id": "obligation.bad",
            "status": "satisfiable",
            "model": {"a": false},
            "unknown_reason": null,
            "unsupported_reason": null,
            "usage": {
                "proposition_count": 1,
                "assignments_checked": 1,
                "max_assignments": 4,
                "max_propositions": 4,
                "exhaustive": false
            }
        }],
        "unknown_reason": null,
        "unsupported_reason": null,
        "usage": {
            "proposition_count": 1,
            "assignments_checked": 1,
            "max_assignments": 4,
            "max_propositions": 4,
            "exhaustive": false
        }
    }"#;

    let error = serde_json::from_str::<SolveReport>(malformed).expect_err("invalid report");

    assert!(error.to_string().contains("aggregate obligation status"));
}

#[test]
fn resource_limits_report_unknown_without_claiming_unsat() {
    let problem = ProverProblem::clauses(
        id("problem.unknown"),
        vec![
            Clause::unit(Literal::positive(id("a"))),
            Clause::unit(Literal::positive(id("b"))),
        ],
    );
    let prover = FiniteProver::new(ResourceLimits::new(4, 1).expect("valid limits"));

    let report = prover.solve(&problem).expect("finite solve succeeds");

    assert_eq!(report.status(), SolveStatus::Unknown);
    assert!(report.unknown_reason().is_some());
    assert_eq!(report.usage().assignments_checked, 1);
    assert!(!report.usage().exhaustive);
}

#[test]
fn explicit_unsupported_problem_reports_unsupported() {
    let problem = ProverProblem::unsupported(
        id("problem.smt"),
        "linear_integer_arithmetic",
        "finite Boolean bridge has no arithmetic solver",
    )
    .expect("valid unsupported problem");

    let report = solve(problem);

    assert_eq!(report.status(), SolveStatus::Unsupported);
    assert!(report
        .unsupported_reason()
        .expect("unsupported reason")
        .contains("linear_integer_arithmetic"));
}

#[test]
fn reports_reject_malformed_status_payloads_at_serde_boundary() {
    let malformed = r#"{
        "problem_id": "problem.bad",
        "status": "unknown",
        "model": null,
        "unknown_reason": "   ",
        "unsupported_reason": null,
        "usage": {
            "proposition_count": 1,
            "assignments_checked": 0,
            "max_assignments": 1,
            "max_propositions": 1,
            "exhaustive": false
        }
    }"#;

    let error = serde_json::from_str::<SolveReport>(malformed).expect_err("invalid report");

    assert!(error.to_string().contains("malformed_field"));
}

#[test]
fn problem_and_report_roundtrip_with_provenance_metadata() {
    let provenance = Provenance::new(
        SourceRef::new(SourceKind::Code),
        Confidence::new(0.9).expect("valid confidence"),
    )
    .with_review_status(ReviewStatus::Reviewed);
    let problem = ProverProblem::formula(
        id("problem.roundtrip"),
        BooleanFormula::iff(atom("a"), BooleanFormula::negate(atom("b"))),
    )
    .with_provenance(provenance);

    let problem_json = serde_json::to_string(&problem).expect("serialize problem");
    let problem_roundtrip: ProverProblem =
        serde_json::from_str(&problem_json).expect("deserialize problem");
    assert_eq!(problem_roundtrip, problem);

    let report = solve(problem_roundtrip);
    let report_json = serde_json::to_string(&report).expect("serialize report");
    let report_roundtrip: SolveReport =
        serde_json::from_str(&report_json).expect("deserialize report");

    assert_eq!(report_roundtrip, report);
}
