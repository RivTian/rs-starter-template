//! Process entry point. Deliberately synchronous until the runtime exists:
//! panic hook → CLI → configuration → runtime → `bootstrap::run` inside the runtime.

use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;

use {{crate_name}}::cli::{Cli, Command};
use {{crate_name}}::{bootstrap, probe};

use {{crate_name}}_config::Config;
use {{crate_name}}_core::error::chain;
use {{crate_name}}_core::runtime;

fn main() -> ExitCode {
    {{crate_name}}_core::telemetry::install_panic_hook();
    let cli = Cli::parse();

    if let Some(Command::Probe { url, timeout_secs }) = &cli.command {
        return probe::run(url, Duration::from_secs(*timeout_secs));
    }

    let cfg = match Config::load(&cli.sources()) {
        Ok(cfg) => cfg,
        Err(error) => return fail("configuration", &error),
    };

    if cli.check_config {
        return match cfg.redacted() {
            Ok(text) => {
                print_config(&text);
                ExitCode::SUCCESS
            }
            Err(error) => fail("configuration", &error),
        };
    }

    let runtime = match runtime::build(&cfg.runtime, &format!("{}-worker", cfg.app.name)) {
        Ok(runtime) => runtime,
        Err(error) => return fail("runtime", &error),
    };

    let outcome = runtime.block_on(bootstrap::run(cfg));
    // Give blocking tasks a moment, then let go regardless.
    runtime.shutdown_timeout(Duration::from_secs(5));

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail("fatal", error.as_ref()),
    }
}

#[expect(
    clippy::print_stdout,
    reason = "--check-config output is the program's result"
)]
fn print_config(text: &str) {
    println!("{text}");
}

#[expect(clippy::print_stderr, reason = "telemetry may not be installed yet")]
fn fail(stage: &str, error: &dyn std::error::Error) -> ExitCode {
    eprintln!("{stage} error: {}", chain(error));
    ExitCode::FAILURE
}
