#![allow(missing_docs)]
//! File-based structured case graph tooling for HigherGraphen.

pub mod cli;
pub mod eval;
pub mod model;
pub mod native_cli;
pub mod native_eval;
pub mod native_model;
pub mod native_report;
pub mod native_review;
pub mod native_store;
pub mod report;
pub mod store;
pub mod topology;
pub mod workflow_eval;
pub mod workflow_model;
pub mod workflow_report;
pub mod workflow_workspace;

#[cfg(test)]
pub(crate) mod fixtures {
    use crate::model::{
        CaseGraph, CaseRecord, CaseRelation, CoverageGoal, CoveragePolicy, Outcome, Scenario,
        CASE_GRAPH_SCHEMA, COVERAGE_POLICY_SCHEMA,
    };
    use higher_graphen_core::{
        Confidence, Id, Provenance, ReviewStatus, Severity, SourceKind, SourceRef,
    };
    use serde_json::Map;

    pub(crate) fn sample_graph() -> CaseGraph {
        CaseGraph {
            schema: CASE_GRAPH_SCHEMA.to_owned(),
            case_graph_id: id("case_graph:architecture-smoke"),
            space_id: id("space:architecture-product-smoke"),
            cases: vec![sample_case()],
            scenarios: vec![sample_scenario()],
            coverage_goals: vec![sample_coverage_goal()],
            relations: vec![sample_relation()],
            review_records: Vec::new(),
            metadata: Map::new(),
        }
    }

    pub(crate) fn coverage_policy() -> CoveragePolicy {
        CoveragePolicy {
            schema: COVERAGE_POLICY_SCHEMA.to_owned(),
            policy_id: id("coverage-policy:architecture-boundary"),
            coverage_goal_ids: Vec::new(),
            require_explicit_relations: false,
            metadata: Map::new(),
        }
    }

    fn sample_case() -> CaseRecord {
        CaseRecord {
            id: id("case:direct-db-access-smoke"),
            space_id: id("space:architecture-product-smoke"),
            title: "Direct DB access smoke scenario".to_owned(),
            case_type: "smoke".to_owned(),
            situation_summary: "Order Service reads Billing DB directly.".to_owned(),
            scenario_ids: vec![id("scenario:service-db-boundary")],
            cell_ids: vec![id("cell:order-service"), id("cell:billing-db")],
            incidence_ids: vec![id("incidence:order-service-reads-billing-db")],
            context_ids: vec![id("context:orders")],
            expected_outcomes: vec![Outcome {
                id: id("outcome:violation-detected"),
                summary: "Direct cross-context DB access is reported.".to_owned(),
            }],
            observed_outcomes: vec![Outcome {
                id: id("outcome:violation-detected"),
                summary: "No violation was reported.".to_owned(),
            }],
            source_ids: vec![id("source:architecture-input")],
            tags: vec!["architecture".to_owned(), "boundary".to_owned()],
            provenance: provenance(SourceKind::Document),
        }
    }

    fn sample_scenario() -> Scenario {
        Scenario {
            id: id("scenario:service-db-boundary"),
            space_id: id("space:architecture-product-smoke"),
            title: "Service crosses a DB ownership boundary".to_owned(),
            scenario_type: "boundary".to_owned(),
            parameters: Map::new(),
            required_context_ids: vec![id("context:orders"), id("context:billing")],
            required_cell_types: vec!["service".to_owned(), "database".to_owned()],
            coverage_target_ids: vec![id("coverage:owned-db-access")],
            source_ids: vec![id("source:architecture-scenario-template")],
            provenance: provenance(SourceKind::Document),
        }
    }

    fn sample_coverage_goal() -> CoverageGoal {
        CoverageGoal {
            id: id("coverage:owned-db-access"),
            space_id: id("space:architecture-product-smoke"),
            coverage_type: "boundary".to_owned(),
            required_ids: vec![
                id("cell:order-service"),
                id("cell:billing-db"),
                id("context:orders"),
                id("context:billing"),
            ],
            dimensions: vec![0, 1],
            min_cases_per_target: 1,
            severity_if_uncovered: Severity::High,
            source_ids: vec![id("source:architecture-requirements")],
            provenance: provenance(SourceKind::Document),
        }
    }

    fn sample_relation() -> CaseRelation {
        CaseRelation {
            id: id("relation:case-covers-boundary"),
            relation_type: "covers".to_owned(),
            from_id: id("case:direct-db-access-smoke"),
            to_id: id("coverage:owned-db-access"),
            evidence_ids: Vec::new(),
            provenance: provenance(SourceKind::Document),
        }
    }

    fn provenance(kind: SourceKind) -> Provenance {
        Provenance::new(
            SourceRef::new(kind),
            Confidence::new(1.0).expect("confidence"),
        )
        .with_review_status(ReviewStatus::Unreviewed)
    }

    fn id(value: &str) -> Id {
        Id::new(value).expect("fixture id")
    }
}
