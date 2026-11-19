use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

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

/// Map action string to ActionType for permission checking.
fn action_type_for(action: &str) -> Option<ActionType> {
    match action {
        "feed_meal" => Some(ActionType::FeedMeal),
        "feed_snack" => Some(ActionType::FeedSnack),
        "play_game" | "guess_direction" => Some(ActionType::PlayGame),
        "discipline" => Some(ActionType::Discipline),
        "give_medicine" => Some(ActionType::GiveMedicine),
        "clean_poop" => Some(ActionType::CleanPoop),
        "toggle_lights" => Some(ActionType::ToggleLights),
        "get_status" => Some(ActionType::GetStatus),
        _ => None,
    }
}

// ── Request Handler ─────────────────────────────────────────────────────────

fn handle_request(req: &Request, pet: &SharedPetState, perms: &SharedPermissions, save_path: &Path) -> Response {
    let now = Utc::now();

    // Resolve action type
    let action_type = match action_type_for(&req.action) {
        Some(at) => at,
        None => return err_response(format!("unknown action: {}", req.action)),
    };

    // Keep the permission check, action, and usage count in one transaction.
    // Neither lock is held while waiting for a client to read or write.
    let mut permissions = match perms.lock() {
        Ok(permissions) => permissions,
        Err(e) => return err_response(format!("lock error: {e}")),
    };
    if let Err(denied) = permissions::check_permission(&mut permissions, &action_type, now) {
        return err_response(denied.to_string());
    }
    let mut state = match pet.lock() {
        Ok(state) => state,
        Err(e) => return err_response(format!("lock error: {e}")),
    };
    if req.action == "guess_direction" {
        let guess = match serde_json::from_value(req.params.clone()) {
            Ok(guess) => guess,
            Err(e) => return err_response(format!("invalid guess: {e}")),
        };
        let mut next = state.clone();
        if let Err(e) = actions::guess_direction(&mut next, guess) {
            return err_response(format!("{e:?}"));
        }
        if let Err(e) = persistence::save(&next, save_path) {
            return err_response(format!("save error: {e}"));
        }
        if next.game.turn % 5 == 0 {
            permissions::log_action(&mut permissions, action_type, now);
        }
        *state = next.clone();
        return ok_response(next);
    }
    let result = match action_type {
        ActionType::GetStatus => Ok(()),
        ActionType::FeedMeal => actions::feed_meal(&mut state).map(drop),
        ActionType::FeedSnack => actions::feed_snack(&mut state).map(drop),
        ActionType::PlayGame => {
            let moves: [Choice; 5] = match serde_json::from_value(
                req.params.get("moves").cloned().unwrap_or_default(),
            ) {
                Ok(moves) => moves,
                Err(e) => return err_response(format!("invalid moves param: {e}")),
            };
            actions::play_game(&mut state, moves).map(drop)
        }
        ActionType::Discipline => actions::discipline(&mut state).map(drop),
        ActionType::GiveMedicine => actions::give_medicine(&mut state).map(drop),
        ActionType::CleanPoop => actions::clean_poop(&mut state).map(drop),
        ActionType::ToggleLights => actions::toggle_lights(&mut state, now).map(drop),
    };
    if let Err(e) = result {
        return err_response(format!("{e:?}"));
    }
    let changed = action_type != ActionType::GetStatus;
    permissions::log_action(&mut permissions, action_type, now);
    if changed {
        if let Err(e) = persistence::save(&state, save_path) {
            return err_response(format!("save error: {e}"));
        }
    }
    ok_response(state.clone())
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

    serve(listener, pet, perms).await;
}

async fn serve(listener: TcpListener, pet: SharedPetState, perms: SharedPermissions) {
    // Each MCP session can keep its connection open across many tool calls.
    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("[socket] accept error: {e}");
                continue;
            }
        };

        eprintln!("[socket] connection from {peer}");

        let pet = pet.clone();
        let perms = perms.clone();
        tokio::spawn(async move {
            serve_client(stream, pet, perms).await;
            eprintln!("[socket] connection from {peer} closed");
        });
    }
}

async fn serve_client(stream: TcpStream, pet: SharedPetState, perms: SharedPermissions) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    let save_path = state_path();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => handle_request(&req, &pet, &perms, &save_path),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tama_core::state::AgentPermissions;

    #[test]
    fn interactive_game_uses_one_permission_and_commits_five_distinct_turns() {
        let birth = Utc::now();
        let path = std::env::temp_dir().join(format!("tama96-rounds-{}-{}.json", std::process::id(), birth.timestamp_micros()));
        let pet = Arc::new(Mutex::new(PetState::new_egg(birth)));
        let mut limits = AgentPermissions::default();
        limits.allowed_actions.get_mut(&ActionType::PlayGame).unwrap().max_per_hour = Some(1);
        let permissions = Arc::new(Mutex::new(limits));
        let guess = |turn| handle_request(&Request {
            action: "guess_direction".into(),
            params: serde_json::json!({ "turn": turn, "choice": "Left", "birth_time": birth }),
        }, &pet, &permissions, &path);
        assert!(guess(0).ok);
        assert!(!guess(0).ok);
        for turn in 1..5 { assert!(guess(turn).ok); }
        let saved: PetState = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved, *pet.lock().unwrap());
        assert_eq!(saved.game.stars, u32::from(saved.game.hits));
        assert_eq!(saved.weight, 4);
        assert_eq!(permissions.lock().unwrap().action_log.len(), 1);
        assert!(!guess(5).ok);
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn status_is_available_while_another_client_remains_connected() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let pet = Arc::new(Mutex::new(PetState::new_egg(Utc::now())));
        let perms = Arc::new(Mutex::new(AgentPermissions::default()));
        let server = tokio::spawn(serve(listener, pet, perms));

        let mut first = BufReader::new(TcpStream::connect(address).await.unwrap());
        let mut second = BufReader::new(TcpStream::connect(address).await.unwrap());
        // The first client stays connected and can issue another request afterwards.
        async fn status(client: &mut BufReader<TcpStream>) {
            client
                .get_mut()
                .write_all(b"{\"action\":\"get_status\"}\n")
                .await
                .unwrap();
            let mut response = String::new();
            tokio::time::timeout(Duration::from_secs(3), client.read_line(&mut response))
                .await
                .expect("client should receive its own response")
                .unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&response).unwrap()["ok"],
                true
            );
        }
        status(&mut first).await;
        status(&mut second).await;
        status(&mut first).await;
        server.abort();
    }
}
