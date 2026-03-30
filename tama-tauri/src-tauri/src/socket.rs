use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use tama_core::actions::{self, Choice};
use tama_core::permissions;
use tama_core::persistence;
use tama_core::state::{ActionType, PetState};

use crate::commands::{SharedPermissions, SharedPetState};

// ── JSON Protocol Types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Request {
    action: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct Response {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<PetState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn state_path() -> PathBuf {
    dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".tama96")
        .join("state.json")
}

fn port_path() -> PathBuf {
    dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".tama96")
        .join("mcp_port")
}

fn ok_response(state: PetState) -> Response {
    Response {
        ok: true,
        state: Some(state),
        error: None,
    }
}

fn err_response(msg: impl Into<String>) -> Response {
    Response {
        ok: false,
        state: None,
        error: Some(msg.into()),
    }
}

fn save_and_snapshot(pet: &SharedPetState) -> Result<PetState, String> {
    let state = pet.lock().map_err(|e| e.to_string())?;
    persistence::save(&state, &state_path()).map_err(|e| e.to_string())?;
    Ok(state.clone())
}

/// Map action string to ActionType for permission checking.
fn action_type_for(action: &str) -> Option<ActionType> {
    match action {
        "feed_meal" => Some(ActionType::FeedMeal),
        "feed_snack" => Some(ActionType::FeedSnack),
        "play_game" => Some(ActionType::PlayGame),
        "discipline" => Some(ActionType::Discipline),
        "give_medicine" => Some(ActionType::GiveMedicine),
        "clean_poop" => Some(ActionType::CleanPoop),
        "toggle_lights" => Some(ActionType::ToggleLights),
        "get_status" => Some(ActionType::GetStatus),
        _ => None,
    }
}

// ── Request Handler ─────────────────────────────────────────────────────────

fn handle_request(
    req: &Request,
    pet: &SharedPetState,
    perms: &SharedPermissions,
) -> Response {
    let now = Utc::now();

    // Resolve action type
    let action_type = match action_type_for(&req.action) {
        Some(at) => at,
        None => return err_response(format!("unknown action: {}", req.action)),
    };

    // Check permissions
    {
        let mut p = match perms.lock() {
            Ok(p) => p,
            Err(e) => return err_response(format!("lock error: {e}")),
        };
        if let Err(denied) = permissions::check_permission(&mut p, &action_type, now) {
            return err_response(denied.to_string());
        }
    }

    // Execute action
    let result = match req.action.as_str() {
        "get_status" => {
            let state = match pet.lock() {
                Ok(s) => s.clone(),
                Err(e) => return err_response(format!("lock error: {e}")),
            };
            // Log the action
            if let Ok(mut p) = perms.lock() {
                permissions::log_action(&mut p, action_type, now);
            }
            return ok_response(state);
        }
        "feed_meal" => {
            let mut state = match pet.lock() {
                Ok(s) => s,
                Err(e) => return err_response(format!("lock error: {e}")),
            };
            actions::feed_meal(&mut state).map_err(|e| format!("{e:?}"))
        }
        "feed_snack" => {
            let mut state = match pet.lock() {
                Ok(s) => s,
                Err(e) => return err_response(format!("lock error: {e}")),
            };
            actions::feed_snack(&mut state).map_err(|e| format!("{e:?}"))
        }
        "play_game" => {
            // Parse moves from params
            let moves: [Choice; 5] = match serde_json::from_value(
                req.params.get("moves").cloned().unwrap_or_default(),
            ) {
                Ok(m) => m,
                Err(e) => return err_response(format!("invalid moves param: {e}")),
            };
            let mut state = match pet.lock() {
                Ok(s) => s,
                Err(e) => return err_response(format!("lock error: {e}")),
            };
            actions::play_game(&mut state, moves).map(|_| actions::ActionResult::Fed).map_err(|e| format!("{e:?}"))
            // Note: play_game returns GameResult, but we just need to know it succeeded
        }
        "discipline" => {
            let mut state = match pet.lock() {
                Ok(s) => s,
                Err(e) => return err_response(format!("lock error: {e}")),
            };
            actions::discipline(&mut state).map_err(|e| format!("{e:?}"))
        }
        "give_medicine" => {
            let mut state = match pet.lock() {
                Ok(s) => s,
                Err(e) => return err_response(format!("lock error: {e}")),
            };
            actions::give_medicine(&mut state).map_err(|e| format!("{e:?}"))
        }
        "clean_poop" => {
            let mut state = match pet.lock() {
                Ok(s) => s,
                Err(e) => return err_response(format!("lock error: {e}")),
            };
            actions::clean_poop(&mut state).map_err(|e| format!("{e:?}"))
        }
        "toggle_lights" => {
            let mut state = match pet.lock() {
                Ok(s) => s,
                Err(e) => return err_response(format!("lock error: {e}")),
            };
            actions::toggle_lights(&mut state, now).map_err(|e| format!("{e:?}"))
        }
        _ => return err_response(format!("unknown action: {}", req.action)),
    };

    match result {
        Ok(_) => {
            // Log the action
            if let Ok(mut p) = perms.lock() {
                permissions::log_action(&mut p, action_type, now);
            }
            // Save and return snapshot
            match save_and_snapshot(pet) {
                Ok(state) => ok_response(state),
                Err(e) => err_response(format!("save error: {e}")),
            }
        }
        Err(e) => err_response(e),
    }
}

// ── Public Entry Point ──────────────────────────────────────────────────────

/// Start the TCP socket server for MCP bridge communication.
///
/// Binds to 127.0.0.1:0 (random available port), writes the port number
/// to ~/.tama96/mcp_port, then loops accepting connections and processing
/// newline-delimited JSON requests.
pub async fn start_socket_server(pet: SharedPetState, perms: SharedPermissions) {
    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[socket] failed to bind TCP listener: {e}");
            return;
        }
    };

    let addr = match listener.local_addr() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("[socket] failed to get local address: {e}");
            return;
        }
    };

    let port = addr.port();
    eprintln!("[socket] listening on 127.0.0.1:{port}");

    // Write port file for sidecar discovery
    let port_file = port_path();
    if let Some(parent) = port_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&port_file, port.to_string()) {
        eprintln!("[socket] failed to write port file: {e}");
    }

    // Accept loop — one connection at a time
    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("[socket] accept error: {e}");
                continue;
            }
        };

        eprintln!("[socket] connection from {peer}");

        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<Request>(&line) {
                Ok(req) => handle_request(&req, &pet, &perms),
                Err(e) => err_response(format!("invalid JSON: {e}")),
            };

            let mut resp_json = match serde_json::to_string(&response) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("[socket] serialize error: {e}");
                    continue;
                }
            };
            resp_json.push('\n');

            if let Err(e) = writer.write_all(resp_json.as_bytes()).await {
                eprintln!("[socket] write error: {e}");
                break;
            }
        }

        eprintln!("[socket] connection from {peer} closed");
    }
}
