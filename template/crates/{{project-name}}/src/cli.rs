//! Command line: only *how this process starts*. Everything about *how the service runs*
//! belongs in configuration.

use std::net::SocketAddr;
use std::path::PathBuf;

use {{crate_name}}_config::Sources;
use clap::{Parser, Subcommand};

use crate::ENV_PREFIX;

/// Top-level arguments.
#[derive(Parser, Debug)]
#[command(name = "{{project-name}}", version = version(), about)]
pub struct Cli {
    /// Read exactly this configuration file instead of config/default.toml + config/local.toml.
    #[arg(short, long, env = "{{env_prefix}}_CONFIG", value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Override the HTTP bind address (highest priority).
    #[arg(long, value_name = "ADDR")]
    pub bind: Option<SocketAddr>,

    /// Load and validate configuration, print it with secrets redacted, then exit.
    #[arg(long)]
    pub check_config: bool,

    /// Optional subcommand; without one the service starts.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Request a URL and exit 0 on HTTP 2xx. Used as the container HEALTHCHECK.
    Probe {
        /// Plain `http://host:port/path` URL.
        url: String,
        /// Connect/read timeout in seconds.
        #[arg(long, default_value_t = 2)]
        timeout_secs: u64,
    },
}

impl Cli {
    /// Turns the arguments into configuration sources.
    #[must_use]
    pub fn sources(&self) -> Sources {
        let mut sources = Sources::new(ENV_PREFIX).with_file(self.config.clone());
        if let Some(bind) = self.bind {
            sources = sources.with_override("http.bind", bind.to_string());
        }
        sources
    }
}

fn version() -> String {
    let build = crate::build_info::current();
    format!(
        "{}\ncommit:  {}\ndescribe: {}\nbuilt:   {}\nrustc:   {}\nprofile: {}",
        build.version,
        build.git_sha,
        build.git_describe,
        build.build_timestamp,
        build.rustc,
        build.profile
    )
}
