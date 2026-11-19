use std::{
    env,
    path::{Path, PathBuf},
    process::Output,
    time::Duration,
};

use serde_json::Value;
use tauri::AppHandle;
use tokio::process::Command;

fn executables(name: &str) -> Vec<PathBuf> {
    let filename = format!("{name}{}", env::consts::EXE_SUFFIX);
    let mut dirs: Vec<_> = env::split_paths(&env::var_os("PATH").unwrap_or_default()).collect();
    #[cfg(unix)]
    dirs.extend(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from));
    #[cfg(target_os = "macos")]
    if name == "codex" {
        dirs.extend([
            "/Applications/Codex.app/Contents/Resources",
            "/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS",
        ].map(PathBuf::from));
    }
    // Finder-launched apps don't inherit shell initialization from nvm.
    #[cfg(unix)]
    if let Some(nvm) = env::var_os("NVM_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".nvm")))
    {
        if let Ok(versions) = std::fs::read_dir(nvm.join("versions/node")) {
            let mut versions: Vec<_> = versions.flatten().map(|entry| entry.path()).collect();
            versions.sort_by_cached_key(|path| {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .trim_start_matches('v')
                    .split('.')
                    .filter_map(|part| part.parse::<u32>().ok())
                    .collect::<Vec<_>>()
            });
            dirs.extend(versions.into_iter().rev().map(|path| path.join("bin")));
        }
    }
    let mut paths = Vec::new();
    for dir in dirs.into_iter().filter(|p| p.is_absolute()) {
        let path = dir.join(&filename);
        if path.is_file() && !paths.contains(&path) {
            paths.push(path);
        }
    }
    paths
}

async fn run(program: &Path, args: &[&str]) -> Result<Output, String> {
    let mut command = Command::new(program);
    command.args(args).kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    tokio::time::timeout(Duration::from_secs(8), command.output())
        .await
        .map_err(|_| "The setup command timed out. Try again.".to_string())?
        .map_err(|e| format!("Could not start {}: {e}", program.display()))
}

async fn node_runtime() -> Result<PathBuf, String> {
    for node in executables("node") {
        if let Ok(output) = run(&node, &["--version"]).await {
            let version = String::from_utf8_lossy(&output.stdout);
            let major = version
                .trim()
                .trim_start_matches('v')
                .split('.')
                .next()
                .and_then(|s| s.parse::<u32>().ok());
            if output.status.success() && major.is_some_and(|v| v >= 18) {
                return Ok(node);
            }
        }
    }
    Err("Install Node.js 18 or newer, then try again.".into())
}

fn config_matches(value: &Value, node: &str, server: &str) -> bool {
    let transport = &value["transport"];
    value["enabled"] == true
        && transport["type"] == "stdio"
        && transport["command"] == node
        && transport["args"] == serde_json::json!([server])
}

async fn register(cli: &Path, node: &str, server: &str) -> Result<String, String> {
    let get = ["mcp", "get", "tama96", "--json"];
    let previous = run(cli, &get).await?;
    if previous.status.success() {
        let value: Value = serde_json::from_slice(&previous.stdout)
            .map_err(|_| "Could not read the Codex entry.")?;
        if config_matches(&value, node, server) {
            return Ok("Already added to Codex".into());
        }
        return Err("Codex already has a different or disabled tama96 entry. Review it in Codex MCP settings.".into());
    }
    if !String::from_utf8_lossy(&previous.stderr).contains("No MCP server named 'tama96' found") {
        return Err(
            "Could not read Codex MCP settings. Check that the Codex CLI is working.".into(),
        );
    }
    let added = run(cli, &["mcp", "add", "tama96", "--", node, server]).await?;
    if !added.status.success() {
        return Err("Codex could not save the server. Check your Codex configuration.".into());
    }
    let saved = run(cli, &get).await?;
    let value: Value = serde_json::from_slice(&saved.stdout).unwrap_or(Value::Null);
    if saved.status.success() && config_matches(&value, node, server) {
        Ok("Added to Codex".into())
    } else {
        Err("Could not verify the saved server. Check Codex MCP settings.".into())
    }
}

#[tauri::command]
pub async fn register_with_codex(app: AppHandle) -> Result<String, String> {
    let server = std::fs::canonicalize(crate::sidecar::resolve_mcp_server_path(&app))
        .map_err(|_| "The bundled MCP server is missing. Reinstall or rebuild tama96.")?;
    let cli = executables("codex").into_iter().next().ok_or(
        "Codex CLI was not found. Use Manual setup below, or install the CLI and try again.",
    )?;
    let node = node_runtime().await?;
    register(
        &cli,
        node.to_str().ok_or("Unsupported Node path.")?,
        server.to_str().ok_or("Unsupported server path.")?,
    )
    .await
}
