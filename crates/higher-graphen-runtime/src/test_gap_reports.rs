//! Missing unit test detector report shapes.
#![allow(missing_docs)]

mod enums;
mod input;
mod lifted;
mod observed;
mod result;
mod scenario;

use crate::reports::{ProjectionViewSet, ReportEnvelope};

pub use enums::*;
pub use input::*;
pub use lifted::*;
pub use observed::*;
pub use result::*;
pub use scenario::*;

pub type TestGapReport = ReportEnvelope<TestGapScenario, TestGapResult, TestGapProjection>;

pub type TestGapProjection = ProjectionViewSet;
