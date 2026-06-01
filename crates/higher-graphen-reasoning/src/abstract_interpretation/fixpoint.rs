//! Monotone least-fixpoint engine over the membership lattice.
//!
//! This module preserves the original public API while keeping the graph
//! types, solver, and tests in focused submodules.

mod helpers;
mod solver;
#[cfg(test)]
mod tests;
mod types;

pub use solver::run_fixpoint;
pub use types::{
    AbstractDomain, AbstractEdge, AbstractElement, AbstractGraph, AbstractGraphNode,
    AbstractInterpretationReport, AbstractJoin, AbstractMembership, AbstractRegion,
    ConcreteWitnessSummary, FixpointObstruction, FixpointObstructionType, FixpointOptions,
    LossRegionKind, MembershipCheck, NodeAbstractState, NodeSeed, SoundnessStatus, WideningEvent,
    WitnessRelation,
};
