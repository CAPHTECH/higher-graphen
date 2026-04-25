//! Runtime workflow entry points.

pub mod architecture;
mod architecture_direct_db_access_smoke;
mod architecture_input_lift;
mod completion_review;

pub use completion_review::run_completion_review;
