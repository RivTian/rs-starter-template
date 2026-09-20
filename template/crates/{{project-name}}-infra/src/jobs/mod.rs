//! Background jobs are just services: they get a shutdown token, announce readiness and stop
//! promptly when asked. Register them in the composition root, gated by their config switch.

mod cleanup;

pub use cleanup::CleanupJob;
