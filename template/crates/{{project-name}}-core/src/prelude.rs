//! `use {{crate_name}}_core::prelude::*;` — the names every service author needs.

pub use crate::build_info::BuildInfo;
pub use crate::error::{Classify, ErrorKind, ErrorSource, Severity};
pub use crate::event::{EventBus, Publisher, Subscription};
pub use crate::health::{HealthCheck, HealthRegistry, HealthReport, HealthStatus};
pub use crate::readiness::{Readiness, ReadinessWatch};
pub use crate::server::{Phase, PhaseWatch, Server, ServerError};
pub use crate::service::{Service, ServiceError};
pub use crate::shutdown::{
    ChannelSignals, ShutdownSignal, ShutdownToken, SignalSource, SignalTrigger,
};
