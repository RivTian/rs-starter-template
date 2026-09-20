//! A dependency-free HTTP probe for container health checks (distroless images have no curl).

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::ExitCode;
use std::time::Duration;

/// Requests `url` and maps a 2xx status to exit code 0.
#[must_use]
#[expect(
    clippy::print_stderr,
    reason = "diagnostic for the container runtime; no logger here"
)]
pub fn run(url: &str, timeout: Duration) -> ExitCode {
    match status_of(url, timeout) {
        Ok(status) if (200..300).contains(&status) => ExitCode::SUCCESS,
        Ok(status) => {
            eprintln!("probe {url}: HTTP {status}");
            ExitCode::FAILURE
        }
        Err(reason) => {
            eprintln!("probe {url}: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn status_of(url: &str, timeout: Duration) -> Result<u16, String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or("only plain http:// URLs are supported")?;
    let (host_port, path) = rest
        .split_once('/')
        .map_or((rest, "/"), |(h, _)| (h, &rest[h.len()..]));
    let addr = host_port
        .to_socket_addrs()
        .map_err(|e| format!("resolve {host_port}: {e}"))?
        .next()
        .ok_or_else(|| format!("resolve {host_port}: no address"))?;

    let mut stream =
        TcpStream::connect_timeout(&addr, timeout).map_err(|e| format!("connect: {e}"))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| format!("timeout: {e}"))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| format!("timeout: {e}"))?;
    write!(stream, "GET {path} HTTP/1.0\r\nHost: {host_port}\r\nUser-Agent: {{project-name}}-probe\r\nConnection: close\r\n\r\n")
        .map_err(|e| format!("write: {e}"))?;

    let mut response = String::new();
    stream
        .take(4096)
        .read_to_string(&mut response)
        .map_err(|e| format!("read: {e}"))?;
    let status_line = response.lines().next().ok_or("empty response")?;
    status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| format!("malformed status line: {status_line:?}"))
}
