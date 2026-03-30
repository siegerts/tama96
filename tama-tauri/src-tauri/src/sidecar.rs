use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::sleep;

/// Default path to the MCP server entry point, relative to the app working directory.
const DEFAULT_MCP_SERVER_PATH: &str = "mcp-server/dist/index.js";

/// Maximum backoff delay between restart attempts (30 seconds).
const MAX_BACKOFF_SECS: u64 = 30;

/// Initial backoff delay (2 seconds).
const INITIAL_BACKOFF_SECS: u64 = 2;

/// Manages the MCP sidecar process lifecycle.
///
/// Spawns the MCP server as a child process, monitors it, and restarts
/// with exponential backoff (2s, 4s, 8s, 16s, max 30s) on unexpected exit.
/// The loop terminates when the cancellation token is set (on app quit).
pub async fn start_sidecar(cancel: Arc<AtomicBool>) {
    let server_path = std::env::var("TAMA96_MCP_SERVER_PATH")
        .unwrap_or_else(|_| DEFAULT_MCP_SERVER_PATH.to_string());

    let mut backoff_secs = INITIAL_BACKOFF_SECS;

    loop {
        if cancel.load(Ordering::Relaxed) {
            eprintln!("[sidecar] shutdown requested, stopping sidecar loop");
            return;
        }

        eprintln!("[sidecar] spawning MCP server: node {server_path}");

        let mut child = match Command::new("node")
            .arg(&server_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                eprintln!("[sidecar] failed to spawn MCP server: {e}");
                if cancel.load(Ordering::Relaxed) {
                    return;
                }
                eprintln!("[sidecar] retrying in {backoff_secs}s");
                sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = next_backoff(backoff_secs);
                continue;
            }
        };

        // Wait for the child to exit
        let status = tokio::select! {
            result = child.wait() => result,
            _ = wait_for_cancel(&cancel) => {
                eprintln!("[sidecar] shutdown requested, killing MCP server");
                let _ = child.kill().await;
                return;
            }
        };

        match status {
            Ok(exit) => {
                if cancel.load(Ordering::Relaxed) {
                    eprintln!("[sidecar] MCP server exited (shutdown in progress)");
                    return;
                }
                eprintln!("[sidecar] MCP server exited unexpectedly: {exit}");
            }
            Err(e) => {
                if cancel.load(Ordering::Relaxed) {
                    return;
                }
                eprintln!("[sidecar] error waiting for MCP server: {e}");
            }
        }

        // Exponential backoff before restart
        eprintln!("[sidecar] restarting in {backoff_secs}s");
        sleep(Duration::from_secs(backoff_secs)).await;
        backoff_secs = next_backoff(backoff_secs);
    }
}

/// Compute the next backoff delay, doubling up to MAX_BACKOFF_SECS.
fn next_backoff(current: u64) -> u64 {
    (current * 2).min(MAX_BACKOFF_SECS)
}

/// Poll the cancellation flag until it becomes true.
async fn wait_for_cancel(cancel: &AtomicBool) {
    loop {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        sleep(Duration::from_millis(500)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_backoff() {
        assert_eq!(next_backoff(2), 4);
        assert_eq!(next_backoff(4), 8);
        assert_eq!(next_backoff(8), 16);
        assert_eq!(next_backoff(16), 30); // capped at MAX_BACKOFF_SECS
        assert_eq!(next_backoff(30), 30); // stays at max
    }

    #[test]
    fn test_initial_constants() {
        assert_eq!(INITIAL_BACKOFF_SECS, 2);
        assert_eq!(MAX_BACKOFF_SECS, 30);
    }

    #[tokio::test]
    async fn test_cancel_stops_sidecar() {
        let cancel = Arc::new(AtomicBool::new(true));
        // Should return immediately since cancel is already set
        start_sidecar(cancel).await;
    }
}
