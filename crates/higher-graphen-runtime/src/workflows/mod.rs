//! Runtime workflow entry points.

pub mod architecture;
mod architecture_direct_db_access_smoke;
mod architecture_input_lift;
mod completion_review;
mod feed_reader;
mod feed_reader_projection;
mod feed_reader_validation;

pub use completion_review::run_completion_review;
pub use feed_reader::run_feed_reader;
