//! Cross-cutting request handling. Each file is one concern; [`crate::router`] fixes the order.

pub mod metrics;
pub mod panic;
pub mod request_id;
pub mod timeout;
pub mod trace;
