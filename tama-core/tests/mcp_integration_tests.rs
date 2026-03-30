//! Integration tests for the MCP tool call flow.
//!
//! Since we can't easily test the full MCP → TCP → Tauri pipeline in unit tests,
//! these tests simulate the permission → action → state change → persistence flow
//! that the socket handler performs when an MCP tool call arrives:
//!
//! 1. Create AgentPermissions and PetState
//! 2. Check permission (simulating what the socket handler does)
//! 3. If allowed, execute the action
//! 4. Log the action
//! 5. Save state + permissions, reload, verify
//!
//! **Validates: Requirements 9.1–9.6, 8.1–8.3**

use chrono::{Duration, TimeZone, Utc};
use std::fs;
use std::path::PathBuf;

use tama_core::actions::{self, ActionResult};
use tama_core::permissions::{self, PermissionDenied};
use tama_core::persistence;
use tama_core::state::{
    ActionPermission, ActionType, AgentPermissions, Character, LifeStage, PetState,
};

/// Create a unique temp directory for each test.
fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tama96_mcp_integ_{}", rand::random::<u64>()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Create a Baby-stage pet that is alive, awake, and ready for actions.
fn alive_baby() -> PetState {
    let now = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let mut state = PetState::new_egg(now);
    state.stage = LifeStage::Baby;
    state.character = Character::Babytchi;
    state.stage_start_time = now;
    state.hunger = 2;
    state.happiness = 2;
    state.is_sleeping = false;
    state.lights_on = true;
    state
}

/// Helper: save state + permissions, then reload both and return them.
fn round_trip_all(
    state: &PetState,
    perms: &AgentPermissions,
    dir: &PathBuf,
) -> (PetState, AgentPermissions) {
    let state_path = dir.join("state.json");
    let perms_path = dir.join("permissions.json");

    persistence::save(state, &state_path).unwrap();
    permissions::save_permissions(perms, &perms_path).unwrap();

    let loaded_state = persistence::load(&state_path, state.last_tick).unwrap();
    let loaded_perms = permissions::load_permissions(&perms_path).unwrap();

    (loaded_state, loaded_perms)
}

// ── Allowed action: full flow ───────────────────────────────────────────────

#[test]
fn allowed_feed_meal_full_flow() {
    let dir = temp_dir();
    let mut state = alive_baby();
    let mut perms = AgentPermissions::default();
    let now = state.last_tick;
    let old_hunger = state.hunger;
    let old_weight = state.weight;

    // Step 1: Check permission (simulating socket handler)
    let check = permissions::check_permission(&mut perms, &ActionType::FeedMeal, now);
    assert!(check.is_ok());

    // Step 2: Execute action
    let result = actions::feed_meal(&mut state).unwrap();
    assert_eq!(result, ActionResult::Fed);

    // Step 3: Log the action
    permissions::log_action(&mut perms, ActionType::FeedMeal, now);

    // Step 4: Save and reload
    let (loaded_state, loaded_perms) = round_trip_all(&state, &perms, &dir);

    // Step 5: Verify state change persisted
    assert_eq!(loaded_state.hunger, (old_hunger + 1).min(4));
    assert_eq!(loaded_state.weight, old_weight + 1);

    // Verify action was logged
    assert_eq!(loaded_perms.action_log.len(), 1);
    assert_eq!(loaded_perms.action_log[0].action, ActionType::FeedMeal);

    let _ = fs::remove_dir_all(&dir);
}

// ── Master disabled: permission denied ──────────────────────────────────────

#[test]
fn master_disabled_returns_structured_error() {
    let dir = temp_dir();
    let state = alive_baby();
    let mut perms = AgentPermissions::default();
    perms.enabled = false;
    let now = state.last_tick;

    // Permission check should fail with MasterDisabled
    let check = permissions::check_permission(&mut perms, &ActionType::FeedMeal, now);
    assert_eq!(check, Err(PermissionDenied::MasterDisabled));

    // State should remain unchanged (action never executed)
    let (loaded_state, _) = round_trip_all(&state, &perms, &dir);
    assert_eq!(loaded_state.hunger, state.hunger);
    assert_eq!(loaded_state.weight, state.weight);

    let _ = fs::remove_dir_all(&dir);
}

// ── Action disabled: permission denied ──────────────────────────────────────

#[test]
fn action_disabled_returns_structured_error() {
    let dir = temp_dir();
    let state = alive_baby();
    let mut perms = AgentPermissions::default();
    perms
        .allowed_actions
        .get_mut(&ActionType::FeedSnack)
        .unwrap()
        .allowed = false;
    let now = state.last_tick;

    let check = permissions::check_permission(&mut perms, &ActionType::FeedSnack, now);
    assert_eq!(
        check,
        Err(PermissionDenied::ActionDisabled(ActionType::FeedSnack))
    );

    // State unchanged
    let (loaded_state, _) = round_trip_all(&state, &perms, &dir);
    assert_eq!(loaded_state.happiness, state.happiness);

    let _ = fs::remove_dir_all(&dir);
}

// ── Rate limited: permission denied after max_per_hour ──────────────────────

#[test]
fn rate_limited_returns_structured_error() {
    let dir = temp_dir();
    let mut state = alive_baby();
    let mut perms = AgentPermissions::default();
    let now = state.last_tick;

    // Set rate limit of 2 per hour for CleanPoop
    perms.allowed_actions.insert(
        ActionType::CleanPoop,
        ActionPermission {
            allowed: true,
            max_per_hour: Some(2),
        },
    );

    // Perform 2 allowed actions
    state.poop_count = 4;
    for i in 0..2u32 {
        let t = now - Duration::minutes(i as i64);
        let check = permissions::check_permission(&mut perms, &ActionType::CleanPoop, t);
        assert!(check.is_ok(), "Action {} should be allowed", i);
        actions::clean_poop(&mut state).unwrap();
        permissions::log_action(&mut perms, ActionType::CleanPoop, t);
    }

    // Third attempt should be rate limited
    let check = permissions::check_permission(&mut perms, &ActionType::CleanPoop, now);
    assert_eq!(
        check,
        Err(PermissionDenied::RateLimited {
            action: ActionType::CleanPoop,
            limit: 2,
            used: 2,
        })
    );

    // State reflects only the 2 successful cleans
    assert_eq!(state.poop_count, 2);

    let (loaded_state, loaded_perms) = round_trip_all(&state, &perms, &dir);
    assert_eq!(loaded_state.poop_count, 2);
    assert_eq!(loaded_perms.action_log.len(), 2);

    let _ = fs::remove_dir_all(&dir);
}

// ── Full flow: check → execute → log → save → load → verify ────────────────

#[test]
fn full_flow_feed_snack_with_permission_log_and_persistence() {
    let dir = temp_dir();
    let mut state = alive_baby();
    let mut perms = AgentPermissions::default();
    let now = state.last_tick;
    let old_happiness = state.happiness;
    let old_weight = state.weight;

    // Check permission
    assert!(permissions::check_permission(&mut perms, &ActionType::FeedSnack, now).is_ok());

    // Execute
    let result = actions::feed_snack(&mut state).unwrap();
    assert_eq!(result, ActionResult::Snacked);

    // Log
    permissions::log_action(&mut perms, ActionType::FeedSnack, now);

    // Save both state and permissions
    let state_path = dir.join("state.json");
    let perms_path = dir.join("permissions.json");
    persistence::save(&state, &state_path).unwrap();
    permissions::save_permissions(&perms, &perms_path).unwrap();

    // Load from disk
    let loaded_state = persistence::load(&state_path, state.last_tick).unwrap();
    let loaded_perms = permissions::load_permissions(&perms_path).unwrap();

    // Verify state
    assert_eq!(loaded_state.happiness, (old_happiness + 1).min(4));
    assert_eq!(loaded_state.weight, old_weight + 2);

    // Verify permission log persisted
    assert_eq!(loaded_perms.action_log.len(), 1);
    assert_eq!(loaded_perms.action_log[0].action, ActionType::FeedSnack);
    assert_eq!(loaded_perms.action_log[0].timestamp, now);

    let _ = fs::remove_dir_all(&dir);
}

// ── Multiple tools in sequence with rate limiting ───────────────────────────

#[test]
fn sequential_tool_calls_with_rate_limit_enforcement() {
    let dir = temp_dir();
    let mut state = alive_baby();
    let mut perms = AgentPermissions::default();
    let now = state.last_tick;

    // Set rate limit: 1 feed_meal per hour
    perms.allowed_actions.insert(
        ActionType::FeedMeal,
        ActionPermission {
            allowed: true,
            max_per_hour: Some(1),
        },
    );

    // First feed_meal: allowed
    assert!(permissions::check_permission(&mut perms, &ActionType::FeedMeal, now).is_ok());
    actions::feed_meal(&mut state).unwrap();
    permissions::log_action(&mut perms, ActionType::FeedMeal, now);

    // Second feed_meal: rate limited
    let check = permissions::check_permission(&mut perms, &ActionType::FeedMeal, now);
    assert_eq!(
        check,
        Err(PermissionDenied::RateLimited {
            action: ActionType::FeedMeal,
            limit: 1,
            used: 1,
        })
    );

    // But a different action (feed_snack) should still work
    assert!(permissions::check_permission(&mut perms, &ActionType::FeedSnack, now).is_ok());
    actions::feed_snack(&mut state).unwrap();
    permissions::log_action(&mut perms, ActionType::FeedSnack, now);

    // Verify final state
    let (loaded_state, loaded_perms) = round_trip_all(&state, &perms, &dir);
    assert_eq!(loaded_state.hunger, 3); // 2 + 1
    assert_eq!(loaded_state.happiness, 3); // 2 + 1
    assert_eq!(loaded_perms.action_log.len(), 2);

    let _ = fs::remove_dir_all(&dir);
}
