use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{path::BaseDirectory, AppHandle, Manager, Runtime};
use tokio::process::Command;
use tokio::time::sleep;

/// MCP server entry point used in local development.
const DEV_MCP_SERVER_ENTRY: &str = "mcp-server/dist/index.js";

/// MCP server entry point used in packaged builds.
///
/// Tauri bundles the contents of `mcp-server-dist/` at the resource root.
const BUNDLED_MCP_SERVER_ENTRY: &str = "index.js";

/// Resolve the MCP server path relative to the workspace root in development.
/// In dev mode the working directory is typically `tama-tauri/src-tauri/`,
/// so we walk up to find the workspace root (where `Cargo.toml` has [workspace]).
fn resolve_dev_mcp_server_path() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;

    // Try current dir first
    let candidate = cwd.join(DEV_MCP_SERVER_ENTRY);
    if candidate.exists() {
        return Some(candidate.to_string_lossy().into_owned());
    }

    // Walk up to 4 parent directories looking for the workspace root
    let mut dir = cwd.as_path();
    for _ in 0..4 {
        if let Some(parent) = dir.parent() {
            let candidate = parent.join(DEV_MCP_SERVER_ENTRY);
            if candidate.exists() {
                return Some(candidate.to_string_lossy().into_owned());
            }
            dir = parent;
        }
    }

    None
}

/// Resolve the MCP server path for either a packaged app or a local dev run.
fn resolve_mcp_server_path<R: Runtime>(app: &AppHandle<R>) -> String {
    if let Ok(p) = std::env::var("TAMA96_MCP_SERVER_PATH") {
        return p;
    }

    if let Ok(candidate) = app.path().resolve(BUNDLED_MCP_SERVER_ENTRY, BaseDirectory::Resource) {
        if candidate.exists() {
            return candidate.to_string_lossy().into_owned();
        }
    }

    resolve_dev_mcp_server_path().unwrap_or_else(|| DEV_MCP_SERVER_ENTRY.to_string())
}

/// Write a ready-to-use MCP config snippet to ~/.tama96/mcp_config.json
/// so users can easily copy it into their AI tool's config.
pub fn write_mcp_config<R: Runtime>(app: &AppHandle<R>) {
    let server_path = resolve_mcp_server_path(app);
    let config = serde_json::json!({
        "mcpServers": {
            "tama96": {
                "command": "node",
                "args": [server_path]
            }
        }
    });

    let config_path = dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".tama96")
        .join("mcp_config.json");

    match serde_json::to_string_pretty(&config) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&config_path, json) {
                eprintln!("[sidecar] failed to write mcp_config.json: {e}");
            } else {
                eprintln!("[sidecar] wrote MCP config to {}", config_path.display());
            }
        }
        Err(e) => eprintln!("[sidecar] failed to serialize mcp config: {e}"),
    }
}

/// Maximum backoff delay between restart attempts (30 seconds).
const MAX_BACKOFF_SECS: u64 = 30;

/// Initial backoff delay (2 seconds).
const INITIAL_BACKOFF_SECS: u64 = 2;

/// Manages the MCP sidecar process lifecycle.
///
/// Spawns the MCP server as a child process, monitors it, and restarts
/// with exponential backoff (2s, 4s, 8s, 16s, max 30s) on unexpected exit.
/// The loop terminates when the cancellation token is set (on app quit).
pub async fn start_sidecar<R: Runtime>(app: AppHandle<R>, cancel: Arc<AtomicBool>) {
    let server_path = resolve_mcp_server_path(&app);

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
    #[test]
    fn test_next_backoff() {
        assert_eq!(super::next_backoff(2), 4);
        assert_eq!(super::next_backoff(4), 8);
        assert_eq!(super::next_backoff(8), 16);
        assert_eq!(super::next_backoff(16), 30); // capped at MAX_BACKOFF_SECS
        assert_eq!(super::next_backoff(30), 30); // stays at max
    }

    #[test]
    fn test_initial_constants() {
        assert_eq!(super::INITIAL_BACKOFF_SECS, 2);
        assert_eq!(super::MAX_BACKOFF_SECS, 30);
    }
}
