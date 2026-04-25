//! Stable report shapes returned by runtime workflows.

use higher_graphen_completion::CompletionCandidate;
use higher_graphen_core::Id;
use higher_graphen_invariant::CheckResult;
use higher_graphen_obstruction::Obstruction;
use higher_graphen_projection::InformationLoss;
use higher_graphen_space::{Cell, Incidence};
use serde::{Deserialize, Serialize};

/// Reusable runtime report envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportEnvelope<S, R, P> {
    /// Stable schema identifier.
    pub schema: String,
    /// Stable report type identifier.
    pub report_type: String,
    /// Report schema version.
    pub report_version: u32,
    /// Runtime and consumer metadata.
    pub metadata: ReportMetadata,
    /// Deterministic or input scenario represented by the report.
    pub scenario: S,
    /// Machine-checkable workflow result.
    pub result: R,
    /// Audience-specific projection of the result.
    pub projection: P,
}

/// Metadata shared by runtime workflow reports.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportMetadata {
    /// CLI command represented by this runtime report.
    pub command: String,
    /// Runtime Cargo package name.
    pub runtime_package: String,
    /// Runtime Rust crate name.
    pub runtime_crate: String,
    /// CLI Cargo package name expected to consume this report.
    pub cli_package: String,
}

impl ReportMetadata {
    /// Creates metadata for the architecture direct DB access smoke workflow.
    #[must_use]
    pub fn architecture_direct_db_access_smoke() -> Self {
        Self {
            command: "highergraphen architecture smoke direct-db-access".to_owned(),
            runtime_package: "higher-graphen-runtime".to_owned(),
            runtime_crate: "higher_graphen_runtime".to_owned(),
            cli_package: "highergraphen-cli".to_owned(),
        }
    }
}

/// Architecture smoke workflow report envelope.
pub type ArchitectureDirectDbAccessSmokeReport = ReportEnvelope<
    ArchitectureDirectDbAccessSmokeScenario,
    ArchitectureDirectDbAccessSmokeResult,
    ArchitectureDirectDbAccessSmokeProjection,
>;

/// Report view of the deterministic architecture direct DB access scenario.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDirectDbAccessSmokeScenario {
    /// Architecture smoke space identifier.
    pub space_id: Id,
    /// Workflow context identifier.
    pub workflow_context_id: Id,
    /// Context identifiers represented by the scenario.
    pub context_ids: Vec<Id>,
    /// Accepted scenario cells.
    pub cells: Vec<Cell>,
    /// Accepted scenario incidences.
    pub incidences: Vec<Incidence>,
    /// Invariant identifier evaluated by the workflow.
    pub invariant_id: Id,
    /// Human-readable invariant name.
    pub invariant_name: String,
}

/// Machine-checkable architecture smoke workflow outcome.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDirectDbAccessSmokeResult {
    /// Deterministic workflow status.
    pub status: ArchitectureSmokeStatus,
    /// Violated invariant identifier.
    pub violated_invariant_id: Id,
    /// Lower-crate check result proving the violation.
    pub check_result: CheckResult,
    /// Obstructions produced by the violation.
    pub obstructions: Vec<Obstruction>,
    /// Reviewable completion candidates inferred from the obstruction.
    pub completion_candidates: Vec<CompletionCandidate>,
}

/// Runtime-owned status values for the architecture smoke workflow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureSmokeStatus {
    /// The checked invariant was satisfied.
    Satisfied,
    /// The workflow found the deterministic architecture violation.
    ViolationDetected,
}

/// Stable architecture review projection for humans.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDirectDbAccessSmokeProjection {
    /// Target projection audience.
    pub audience: ProjectionAudience,
    /// Projection purpose.
    pub purpose: ProjectionPurpose,
    /// Human-readable summary.
    pub summary: String,
    /// Recommended follow-up actions.
    pub recommended_actions: Vec<String>,
    /// Declared projection information loss.
    pub information_loss: Vec<InformationLoss>,
    /// Source identifiers represented in the projection.
    pub source_ids: Vec<Id>,
}

/// Runtime projection audience values required by workflow reports.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionAudience {
    /// A human reviewer.
    Human,
}

/// Runtime projection purpose values required by workflow reports.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPurpose {
    /// Architecture review workflow.
    ArchitectureReview,
}
