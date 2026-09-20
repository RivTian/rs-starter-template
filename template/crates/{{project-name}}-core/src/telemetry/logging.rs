use std::io::IsTerminal;

use {{crate_name}}_config::{LogFormat, TelemetryConfig};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

use super::TelemetryError;

pub(super) fn init(cfg: &TelemetryConfig, log_env_var: &str) -> Result<(), TelemetryError> {
    let filter = EnvFilter::try_from_env(log_env_var)
        .or_else(|_| EnvFilter::try_from_default_env())
        .or_else(|_| EnvFilter::try_new(&cfg.log_filter))?;

    let output = match cfg.log_format {
        LogFormat::Pretty => fmt::layer()
            .with_target(true)
            .with_ansi(std::io::stdout().is_terminal())
            .boxed(),
        LogFormat::Json => fmt::layer()
            .json()
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(false)
            .boxed(),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(output)
        .try_init()?;
    Ok(())
}
