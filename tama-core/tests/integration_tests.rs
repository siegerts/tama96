//! Integration tests for tama-core round-trip: action → save → load → verify.
//!
//! These tests simulate the same flow as Tauri IPC commands:
//! 1. Create/prepare a PetState
//! 2. Call the action (same logic the command invokes)
//! 3. Save to a temp file
//! 4. Load from the temp file (with same timestamp to avoid catch-up side effects)
//! 5. Verify the loaded state matches expectations
//!
//! **Validates: Requirements 3.1–3.13, 9.3, 9.4**

use chrono::{TimeZone, Utc};
use std::fs;
use std::path::PathBuf;

use tama_core::actions::{self, ActionError, ActionResult, Choice};
use tama_core::persistence;
use tama_core::state::{LifeStage, PetState};

/// Create a unique temp directory for each test.
fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tama96_integ_{}", rand::random::<u64>()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Helper: save state, then load it back at the same timestamp (no catch-up).
fn round_trip(state: &PetState, dir: &PathBuf) -> PetState {
    let path = dir.join("state.json");
    persistence::save(state, &path).unwrap();
    // Load with the same last_tick so catch-up is a no-op
    persistence::load(&path, state.last_tick).unwrap()
}

/// Create a Baby-stage pet that is alive, awake, and ready for actions.
fn alive_baby() -> PetState {
    let now = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let mut state = PetState::new_egg(now);
    // Advance to Baby stage
    state.stage = LifeStage::Baby;
    state.character = tama_core::state::Character::Babytchi;
    state.stage_start_time = now;
    state.hunger = 2;
    state.happiness = 2;
    state.is_sleeping = false;
    state.lights_on = true;
    state
}

// ── feed_meal round-trip ────────────────────────────────────────────────────

#[test]
fn feed_meal_round_trip() {
    let dir = temp_dir();
    let mut state = alive_baby();
    let old_hunger = state.hunger;
    let old_weight = state.weight;

    actions::feed_meal(&mut state).unwrap();

    let loaded = round_trip(&state, &dir);

    assert_eq!(loaded.hunger, (old_hunger + 1).min(4));
    assert_eq!(loaded.weight, old_weight + 1);
    assert_eq!(loaded.hunger, state.hunger);
    assert_eq!(loaded.weight, state.weight);

    let _ = fs::remove_dir_all(&dir);
}


// ── feed_snack round-trip ───────────────────────────────────────────────────

#[test]
fn feed_snack_round_trip() {
    let dir = temp_dir();
    let mut state = alive_baby();
    let old_happiness = state.happiness;
    let old_weight = state.weight;

    actions::feed_snack(&mut state).unwrap();

    let loaded = round_trip(&state, &dir);

    assert_eq!(loaded.happiness, (old_happiness + 1).min(4));
    assert_eq!(loaded.weight, old_weight + 2);
    assert_eq!(loaded.happiness, state.happiness);
    assert_eq!(loaded.weight, state.weight);

    let _ = fs::remove_dir_all(&dir);
}

// ── discipline with pending call round-trip ─────────────────────────────────

#[test]
fn discipline_with_pending_call_round_trip() {
    let dir = temp_dir();
    let mut state = alive_baby();
    let now = state.last_tick;

    // Set up a pending discipline call
    state.pending_discipline_deadline = Some(now + chrono::Duration::minutes(15));
    let old_discipline = state.discipline;

    let result = actions::discipline(&mut state).unwrap();
    assert_eq!(result, ActionResult::Disciplined);

    let loaded = round_trip(&state, &dir);

    assert_eq!(loaded.discipline, (old_discipline + 25).min(100));
    assert!(loaded.pending_discipline_deadline.is_none());
    assert_eq!(loaded.discipline, state.discipline);

    let _ = fs::remove_dir_all(&dir);
}

// ── give_medicine twice cures sickness round-trip ───────────────────────────

#[test]
fn give_medicine_cures_sickness_round_trip() {
    let dir = temp_dir();
    let mut state = alive_baby();
    state.is_sick = true;
    state.sick_dose_count = 0;

    // First dose
    actions::give_medicine(&mut state).unwrap();
    assert!(state.is_sick);
    assert_eq!(state.sick_dose_count, 1);

    // Second dose
    actions::give_medicine(&mut state).unwrap();
    assert!(!state.is_sick);
    assert_eq!(state.sick_dose_count, 0);

    let loaded = round_trip(&state, &dir);

    assert!(!loaded.is_sick);
    assert_eq!(loaded.sick_dose_count, 0);
    assert_eq!(loaded.is_sick, state.is_sick);

    let _ = fs::remove_dir_all(&dir);
}

// ── clean_poop round-trip ───────────────────────────────────────────────────

#[test]
fn clean_poop_round_trip() {
    let dir = temp_dir();
    let mut state = alive_baby();
    state.poop_count = 3;

    actions::clean_poop(&mut state).unwrap();

    let loaded = round_trip(&state, &dir);

    assert_eq!(loaded.poop_count, 2);
    assert_eq!(loaded.poop_count, state.poop_count);

    let _ = fs::remove_dir_all(&dir);
}

// ── toggle_lights round-trip ────────────────────────────────────────────────

#[test]
fn toggle_lights_round_trip() {
    let dir = temp_dir();
    let mut state = alive_baby();
    let was_on = state.lights_on;
    let now = state.last_tick;

    actions::toggle_lights(&mut state, now).unwrap();

    let loaded = round_trip(&state, &dir);

    assert_eq!(loaded.lights_on, !was_on);
    assert_eq!(loaded.lights_on, state.lights_on);

    let _ = fs::remove_dir_all(&dir);
}

// ── Dead pet actions return PetIsDead round-trip ────────────────────────────

#[test]
fn dead_pet_actions_return_pet_is_dead() {
    let dir = temp_dir();
    let now = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let mut state = PetState::new_egg(now);
    state.is_alive = false;
    state.stage = LifeStage::Dead;

    // Save the dead state and reload to confirm persistence
    let loaded = round_trip(&state, &dir);
    assert!(!loaded.is_alive);
    assert_eq!(loaded.stage, LifeStage::Dead);

    // All actions on the dead (loaded) state should return PetIsDead
    let mut s = loaded;

    assert_eq!(actions::feed_meal(&mut s), Err(ActionError::PetIsDead));
    assert_eq!(actions::feed_snack(&mut s), Err(ActionError::PetIsDead));
    assert_eq!(
        actions::play_game(
            &mut s,
            [
                Choice::Left,
                Choice::Left,
                Choice::Left,
                Choice::Left,
                Choice::Left,
            ]
        ),
        Err(ActionError::PetIsDead)
    );
    assert_eq!(actions::discipline(&mut s), Err(ActionError::PetIsDead));
    assert_eq!(actions::give_medicine(&mut s), Err(ActionError::PetIsDead));
    assert_eq!(actions::clean_poop(&mut s), Err(ActionError::PetIsDead));
    assert_eq!(
        actions::toggle_lights(&mut s, now),
        Err(ActionError::PetIsDead)
    );

    let _ = fs::remove_dir_all(&dir);
}
