use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};

use crate::engine;
use crate::state::{AgentPermissions, PetState};

// ── Data Directory Initialization ───────────────────────────────────────────

/// Ensure the `~/.tama96/` directory exists and initialize default files if missing.
///
/// - Creates `~/.tama96/` if it doesn't exist.
/// - Creates `~/.tama96/permissions.json` with default `AgentPermissions` if missing.
pub fn init_data_dir(data_dir: &Path) -> Result<(), PersistError> {
    fs::create_dir_all(data_dir)?;

    let permissions_path = data_dir.join("permissions.json");
    if !permissions_path.exists() {
        let default_perms = AgentPermissions::default();
        let json = serde_json::to_string_pretty(&default_perms)?;
        let temp_path = permissions_path.with_extension("tmp");
        fs::write(&temp_path, json.as_bytes())?;
        fs::rename(&temp_path, &permissions_path)?;
    }

    Ok(())
}

// ── Error Types ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum PersistError {
    Io(io::Error),
    Serialize(serde_json::Error),
}

impl fmt::Display for PersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistError::Io(e) => write!(f, "I/O error: {e}"),
            PersistError::Serialize(e) => write!(f, "serialization error: {e}"),
        }
    }
}

impl std::error::Error for PersistError {}

impl From<io::Error> for PersistError {
    fn from(e: io::Error) -> Self {
        PersistError::Io(e)
    }
}

impl From<serde_json::Error> for PersistError {
    fn from(e: serde_json::Error) -> Self {
        PersistError::Serialize(e)
    }
}

// ── Save ────────────────────────────────────────────────────────────────────

/// Serialize PetState to JSON and write atomically (temp file + rename).
pub fn save(state: &PetState, path: &Path) -> Result<(), PersistError> {
    let json = serde_json::to_string_pretty(state)?;

    // Build temp file path in the same directory so rename is atomic
    let temp_path = path.with_extension("tmp");

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&temp_path, json.as_bytes())?;
    fs::rename(&temp_path, path)?;

    Ok(())
}

// ── Load ────────────────────────────────────────────────────────────────────

/// Deserialize PetState from JSON, apply catch-up ticks, and handle edge cases.
///
/// - Corrupt file: backs up as `<path>.corrupt`, logs warning, returns fresh egg.
/// - Clock skew (now < last_tick): logs warning, sets last_tick = now, skips catch-up.
pub fn load(path: &Path, now: DateTime<Utc>) -> Result<PetState, PersistError> {
    let data = fs::read_to_string(path)?;

    let mut state: PetState = match serde_json::from_str(&data) {
        Ok(s) => s,
        Err(e) => {
            // Back up corrupt file
            let corrupt_path = path.with_extension("json.corrupt");
            log::warn!(
                "Corrupt save file at {}: {}. Backing up to {} and starting fresh.",
                path.display(),
                e,
                corrupt_path.display()
            );
            // Best-effort backup — don't fail if backup itself fails
            let _ = fs::copy(path, &corrupt_path);
            return Ok(PetState::new_egg(now));
        }
    };

    // Handle clock skew: if system clock moved backwards
    if now < state.last_tick {
        log::warn!(
            "Clock skew detected: now ({}) < last_tick ({}). Skipping catch-up and resyncing.",
            now,
            state.last_tick
        );
        state.last_tick = now;
        return Ok(state);
    }

    // Apply catch-up ticks: simulate elapsed minutes one at a time
    apply_catchup(&mut state, now);

    Ok(state)
}

/// Simulate elapsed time by ticking minute-by-minute from last_tick to `now`.
fn apply_catchup(state: &mut PetState, now: DateTime<Utc>) {
    let elapsed_minutes = (now - state.last_tick).num_minutes();
    if elapsed_minutes <= 0 {
        return;
    }

    // For each elapsed minute, advance the simulation by one tick
    for i in 1..=elapsed_minutes {
        let tick_time = state.last_tick + Duration::minutes(i);
        // Don't tick past `now`
        let t = if tick_time > now { now } else { tick_time };
        engine::tick(state, t);

        // Stop early if the pet died during catch-up
        if !state.is_alive {
            break;
        }
    }

    // Ensure last_tick is exactly `now` after catch-up
    state.last_tick = now;
}

// ── Lock Error ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum LockError {
    AlreadyLocked(String),
    Io(io::Error),
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockError::AlreadyLocked(msg) => write!(f, "{msg}"),
            LockError::Io(e) => write!(f, "lock I/O error: {e}"),
        }
    }
}

impl std::error::Error for LockError {}

impl From<io::Error> for LockError {
    fn from(e: io::Error) -> Self {
        LockError::Io(e)
    }
}

// ── Lockfile ────────────────────────────────────────────────────────────────

/// RAII guard that removes the lock file when dropped.
pub struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    /// Returns the path of the lock file held by this guard.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Check whether a process with the given PID is still alive.
///
/// Uses `kill(pid, 0)` which checks for process existence without sending a signal.
fn is_pid_alive(pid: u32) -> bool {
    // SAFETY: kill with signal 0 only checks process existence, no signal is sent.
    let ret = unsafe { libc::kill(pid as libc::pid_t, 0) };
    ret == 0
}

/// Acquire a lockfile at `path`. The lock file contains the PID of the owning process.
///
/// - If the lock file does not exist, creates it with the current PID and returns a `LockGuard`.
/// - If the lock file exists and the PID inside is still alive, returns `LockError::AlreadyLocked`.
/// - If the lock file exists but the PID is dead (stale lock), removes the stale file and acquires.
pub fn acquire_lock(path: &Path) -> Result<LockGuard, LockError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Check for existing lock file
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(pid) = contents.trim().parse::<u32>() {
                if is_pid_alive(pid) {
                    return Err(LockError::AlreadyLocked(format!(
                        "Another instance (PID {pid}) is already running. \
                         Only one frontend may access the pet at a time."
                    )));
                }
                // Stale lock — owning process is dead, remove it
                log::warn!(
                    "Removing stale lock file at {} (PID {pid} is no longer running)",
                    path.display()
                );
            }
        }
        // Lock file is unreadable or contains garbage — treat as stale
        let _ = fs::remove_file(path);
    }

    // Write current PID to lock file
    let my_pid = std::process::id();
    fs::write(path, my_pid.to_string())?;

    Ok(LockGuard {
        path: path.to_path_buf(),
    })
}

/// Explicitly release a lock by consuming the guard (which triggers Drop).
pub fn release_lock(guard: LockGuard) {
    drop(guard);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PetState;
    use chrono::Utc;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tama96_test_{}", rand::random::<u64>()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = temp_dir();
        let path = dir.join("state.json");
        let now = Utc::now();
        let state = PetState::new_egg(now);

        save(&state, &path).unwrap();
        let loaded = load(&path, now).unwrap();

        assert_eq!(state.stage, loaded.stage);
        assert_eq!(state.character, loaded.character);
        assert_eq!(state.hunger, loaded.hunger);
        assert_eq!(state.happiness, loaded.happiness);
        assert_eq!(state.is_alive, loaded.is_alive);

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_corrupt_file_returns_fresh_egg() {
        let dir = temp_dir();
        let path = dir.join("state.json");
        let now = Utc::now();

        // Write invalid JSON
        fs::write(&path, "not valid json {{{").unwrap();

        let state = load(&path, now).unwrap();
        assert_eq!(state.stage, crate::state::LifeStage::Egg);
        assert!(state.is_alive);

        // Verify backup was created
        let corrupt_path = path.with_extension("json.corrupt");
        assert!(corrupt_path.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_with_clock_skew_resyncs() {
        let dir = temp_dir();
        let path = dir.join("state.json");
        let future = Utc::now() + Duration::hours(1);
        let now = Utc::now();

        // Save state with a future last_tick
        let mut state = PetState::new_egg(future);
        state.last_tick = future;
        save(&state, &path).unwrap();

        // Load with "now" that is before last_tick
        let loaded = load(&path, now).unwrap();
        assert_eq!(loaded.last_tick, now);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_creates_parent_directories() {
        let dir = temp_dir();
        let path = dir.join("nested").join("deep").join("state.json");
        let now = Utc::now();
        let state = PetState::new_egg(now);

        save(&state, &path).unwrap();
        assert!(path.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_is_atomic_no_partial_writes() {
        let dir = temp_dir();
        let path = dir.join("state.json");
        let now = Utc::now();
        let state = PetState::new_egg(now);

        save(&state, &path).unwrap();

        // Temp file should not remain
        let temp_path = path.with_extension("tmp");
        assert!(!temp_path.exists());

        // Target file should be valid JSON
        let data = fs::read_to_string(&path).unwrap();
        let _: PetState = serde_json::from_str(&data).unwrap();

        let _ = fs::remove_dir_all(&dir);
    }

    // ── Lockfile tests ──────────────────────────────────────────────────────

    #[test]
    fn acquire_and_release_lock() {
        let dir = temp_dir();
        let lock_path = dir.join("tama96.lock");

        let guard = acquire_lock(&lock_path).unwrap();
        assert!(lock_path.exists());

        // Lock file should contain our PID
        let contents = fs::read_to_string(&lock_path).unwrap();
        let pid: u32 = contents.trim().parse().unwrap();
        assert_eq!(pid, std::process::id());

        // Release
        release_lock(guard);
        assert!(!lock_path.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn acquire_lock_fails_when_held_by_live_process() {
        let dir = temp_dir();
        let lock_path = dir.join("tama96.lock");

        let _guard = acquire_lock(&lock_path).unwrap();

        // Second acquire should fail — same process is alive
        let result = acquire_lock(&lock_path);
        assert!(result.is_err());
        if let Err(LockError::AlreadyLocked(msg)) = result {
            assert!(msg.contains("Another instance"));
        } else {
            panic!("Expected AlreadyLocked error");
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn acquire_lock_removes_stale_lock() {
        let dir = temp_dir();
        let lock_path = dir.join("tama96.lock");

        // Write a lock file with a PID that (almost certainly) doesn't exist
        // PID 999999 is extremely unlikely to be running
        fs::create_dir_all(&dir).unwrap();
        fs::write(&lock_path, "999999").unwrap();

        // Should succeed because PID 999999 is not alive
        let guard = acquire_lock(&lock_path).unwrap();

        // Lock file should now contain our PID
        let contents = fs::read_to_string(&lock_path).unwrap();
        let pid: u32 = contents.trim().parse().unwrap();
        assert_eq!(pid, std::process::id());

        release_lock(guard);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn acquire_lock_handles_garbage_lock_file() {
        let dir = temp_dir();
        let lock_path = dir.join("tama96.lock");

        // Write garbage to lock file
        fs::create_dir_all(&dir).unwrap();
        fs::write(&lock_path, "not-a-pid").unwrap();

        // Should succeed — garbage is treated as stale
        let guard = acquire_lock(&lock_path).unwrap();
        assert!(lock_path.exists());

        release_lock(guard);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lock_guard_drop_cleans_up() {
        let dir = temp_dir();
        let lock_path = dir.join("tama96.lock");

        {
            let _guard = acquire_lock(&lock_path).unwrap();
            assert!(lock_path.exists());
        }
        // Guard dropped — lock file should be gone
        assert!(!lock_path.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn acquire_lock_creates_parent_directories() {
        let dir = temp_dir();
        let lock_path = dir.join("nested").join("deep").join("tama96.lock");

        let guard = acquire_lock(&lock_path).unwrap();
        assert!(lock_path.exists());

        release_lock(guard);
        let _ = fs::remove_dir_all(&dir);
    }

    // ── init_data_dir tests ─────────────────────────────────────────────────

    #[test]
    fn init_data_dir_creates_directory_and_permissions() {
        let dir = std::env::temp_dir().join(format!("tama96_init_test_{}", rand::random::<u64>()));
        // Ensure it doesn't exist yet
        let _ = fs::remove_dir_all(&dir);

        init_data_dir(&dir).unwrap();

        assert!(dir.exists());
        let perms_path = dir.join("permissions.json");
        assert!(perms_path.exists());

        // Verify it's valid default AgentPermissions
        let data = fs::read_to_string(&perms_path).unwrap();
        let loaded: AgentPermissions = serde_json::from_str(&data).unwrap();
        assert!(loaded.enabled);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_data_dir_does_not_overwrite_existing_permissions() {
        let dir = std::env::temp_dir().join(format!("tama96_init_test_{}", rand::random::<u64>()));
        fs::create_dir_all(&dir).unwrap();

        let perms_path = dir.join("permissions.json");
        // Write custom permissions with master switch off
        let mut custom = AgentPermissions::default();
        custom.enabled = false;
        let json = serde_json::to_string_pretty(&custom).unwrap();
        fs::write(&perms_path, &json).unwrap();

        // init_data_dir should NOT overwrite
        init_data_dir(&dir).unwrap();

        let data = fs::read_to_string(&perms_path).unwrap();
        let loaded: AgentPermissions = serde_json::from_str(&data).unwrap();
        assert!(!loaded.enabled); // still false — not overwritten

        let _ = fs::remove_dir_all(&dir);
    }
}
