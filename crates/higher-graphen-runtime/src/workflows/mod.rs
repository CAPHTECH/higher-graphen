//! Runtime workflow entry points.

pub mod architecture;
mod architecture_direct_db_access_smoke;
mod architecture_input_lift;
mod completion_review;
mod feed_reader;
mod feed_reader_projection;
mod feed_reader_validation;
mod pr_review_target;

pub use completion_review::run_completion_review;
pub use feed_reader::run_feed_reader;
pub use pr_review_target::run_pr_review_target_recommend;
