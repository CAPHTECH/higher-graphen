//! Runtime workflow orchestration APIs for HigherGraphen.

pub mod error;
pub mod reports;
pub mod workflows;

pub use error::{RuntimeError, RuntimeResult};
pub use reports::{
    ArchitectureDirectDbAccessSmokeProjection, ArchitectureDirectDbAccessSmokeReport,
    ArchitectureDirectDbAccessSmokeResult, ArchitectureDirectDbAccessSmokeScenario,
    ArchitectureSmokeStatus, ProjectionAudience, ProjectionPurpose, ReportEnvelope, ReportMetadata,
};
pub use workflows::architecture::run_architecture_direct_db_access_smoke;
