use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use tauri::State;

use tama_core::actions::{self, Choice, GameResult};
use tama_core::permissions;
use tama_core::persistence;
use tama_core::state::{AgentPermissions, PetState};

/// Shared application state managed by Tauri.
pub type SharedPetState = Arc<Mutex<PetState>>;
pub type SharedPermissions = Arc<Mutex<AgentPermissions>>;

/// Returns the path to ~/.tama96/state.json
fn state_path() -> PathBuf {
    dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".tama96")
        .join("state.json")
}

/// Returns the path to ~/.tama96/permissions.json
fn permissions_path() -> PathBuf {
    dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".tama96")
        .join("permissions.json")
}

/// Helper: lock state, save to disk, return a clone of the current state.
fn save_and_snapshot(pet: &Mutex<PetState>) -> Result<PetState, String> {
    let state = pet.lock().map_err(|e| e.to_string())?;
    persistence::save(&state, &state_path()).map_err(|e| e.to_string())?;
    Ok(state.clone())
}

// ── Pet action commands ─────────────────────────────────────────────────────

#[tauri::command]
pub fn get_state(pet: State<'_, SharedPetState>) -> Result<PetState, String> {
    let state = pet.lock().map_err(|e| e.to_string())?;
    Ok(state.clone())
}

#[tauri::command]
pub fn feed_meal(pet: State<'_, SharedPetState>) -> Result<PetState, String> {
    {
        let mut state = pet.lock().map_err(|e| e.to_string())?;
        let pre_hunger = state.hunger;
        let pre_weight = state.weight;
        let pre_sleeping = state.is_sleeping;
        let pre_sick = state.is_sick;
        let pre_alive = state.is_alive;
        
        let result = actions::feed_meal(&mut state).map_err(|e| format!("{e:?}"))?;
        
        println!("feed_meal: pre hunger={}, post hunger={}, pre_weight={}, post_weight={}, alive={}, sleeping={}, sick={}, result={:?}", 
            pre_hunger, state.hunger, pre_weight, state.weight, pre_alive, pre_sleeping, pre_sick, result);
    }
    save_and_snapshot(&pet)
}

#[tauri::command]
pub fn feed_snack(pet: State<'_, SharedPetState>) -> Result<PetState, String> {
    {
        let mut state = pet.lock().map_err(|e| e.to_string())?;
        actions::feed_snack(&mut state).map_err(|e| format!("{e:?}"))?;
    }
    save_and_snapshot(&pet)
}

#[tauri::command]
pub fn play_game(
    pet: State<'_, SharedPetState>,
    moves: [Choice; 5],
) -> Result<GameResult, String> {
    let game_result;
    {
        let mut state = pet.lock().map_err(|e| e.to_string())?;
        game_result = actions::play_game(&mut state, moves).map_err(|e| format!("{e:?}"))?;
    }
    // Save after the game
    save_and_snapshot(&pet)?;
    Ok(game_result)
}

#[tauri::command]
pub fn discipline(pet: State<'_, SharedPetState>) -> Result<PetState, String> {
    {
        let mut state = pet.lock().map_err(|e| e.to_string())?;
        actions::discipline(&mut state).map_err(|e| format!("{e:?}"))?;
    }
    save_and_snapshot(&pet)
}

#[tauri::command]
pub fn give_medicine(pet: State<'_, SharedPetState>) -> Result<PetState, String> {
    {
        let mut state = pet.lock().map_err(|e| e.to_string())?;
        actions::give_medicine(&mut state).map_err(|e| format!("{e:?}"))?;
    }
    save_and_snapshot(&pet)
}

#[tauri::command]
pub fn clean_poop(pet: State<'_, SharedPetState>) -> Result<PetState, String> {
    {
        let mut state = pet.lock().map_err(|e| e.to_string())?;
        actions::clean_poop(&mut state).map_err(|e| format!("{e:?}"))?;
    }
    save_and_snapshot(&pet)
}

#[tauri::command]
pub fn toggle_lights(pet: State<'_, SharedPetState>) -> Result<PetState, String> {
    let now = Utc::now();
    {
        let mut state = pet.lock().map_err(|e| e.to_string())?;
        actions::toggle_lights(&mut state, now).map_err(|e| format!("{e:?}"))?;
    }
    save_and_snapshot(&pet)
}

// ── Lifecycle commands ──────────────────────────────────────────────────────

#[tauri::command]
pub fn hatch_new_egg(pet: State<'_, SharedPetState>) -> Result<PetState, String> {
    let now = Utc::now();
    {
        let mut state = pet.lock().map_err(|e| e.to_string())?;
        *state = PetState::new_egg(now);
    }
    save_and_snapshot(&pet)
}

// ── MCP config command ───────────────────────────────────────────────────

#[tauri::command]
pub fn get_mcp_config() -> Result<String, String> {
    let config_path = dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".tama96")
        .join("mcp_config.json");

    std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Could not read MCP config: {e}"))
}

// ── Permission commands ─────────────────────────────────────────────────────

#[tauri::command]
pub fn get_permissions(
    perms: State<'_, SharedPermissions>,
) -> Result<AgentPermissions, String> {
    let p = perms.lock().map_err(|e| e.to_string())?;
    Ok(p.clone())
}

#[tauri::command]
pub fn update_permissions(
    perms: State<'_, SharedPermissions>,
    new_permissions: AgentPermissions,
) -> Result<AgentPermissions, String> {
    {
        let mut p = perms.lock().map_err(|e| e.to_string())?;
        *p = new_permissions;
        permissions::save_permissions(&p, &permissions_path()).map_err(|e| e.to_string())?;
    }
    let p = perms.lock().map_err(|e| e.to_string())?;
    Ok(p.clone())
}
