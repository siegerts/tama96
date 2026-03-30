//! Integration tests for lockfile mutual exclusion.
//!
//! These tests verify that the lockfile mechanism prevents concurrent access
//! to the pet state, and that lock acquisition/release works correctly across
//! the full lifecycle.
//!
//! **Validates: Requirements 7.1, 7.2, 7.3, 7.4**

use std::fs;
use std::path::PathBuf;

use tama_core::persistence::{self, LockError};

/// Create a unique temp directory for each test.
fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tama96_lock_integ_{}", rand::random::<u64>()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Requirement 7.1, 7.2, 7.4: Acquire lock, verify second acquire fails with AlreadyLocked.
#[test]
fn second_acquire_while_held_returns_already_locked() {
    let dir = temp_dir();
    let lock_path = dir.join("tama96.lock");

    // First acquire succeeds
    let guard = persistence::acquire_lock(&lock_path).unwrap();

    // Second acquire from the same (live) process must fail
    let result = persistence::acquire_lock(&lock_path);
    assert!(result.is_err(), "Second acquire should fail while lock is held");

    match result {
        Err(LockError::AlreadyLocked(msg)) => {
            assert!(
                msg.contains("Another instance"),
                "Error message should mention another instance, got: {msg}"
            );
        }
        Err(other) => panic!("Expected AlreadyLocked, got: {other:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }

    // Cleanup
    drop(guard);
    let _ = fs::remove_dir_all(&dir);
}

/// Requirement 7.3, 7.4: Release lock (drop guard), verify new acquire succeeds.
#[test]
fn acquire_succeeds_after_guard_dropped() {
    let dir = temp_dir();
    let lock_path = dir.join("tama96.lock");

    // Acquire and release via drop
    {
        let _guard = persistence::acquire_lock(&lock_path).unwrap();
        assert!(lock_path.exists(), "Lock file should exist while held");
    }
    // Guard dropped — lock released

    assert!(!lock_path.exists(), "Lock file should be removed after drop");

    // New acquire should succeed
    let guard2 = persistence::acquire_lock(&lock_path)
        .expect("Acquire should succeed after previous guard was dropped");
    assert!(lock_path.exists());

    drop(guard2);
    let _ = fs::remove_dir_all(&dir);
}

/// Requirement 7.1: Lock file contains the correct PID of the owning process.
#[test]
fn lock_file_contains_correct_pid() {
    let dir = temp_dir();
    let lock_path = dir.join("tama96.lock");

    let guard = persistence::acquire_lock(&lock_path).unwrap();

    let contents = fs::read_to_string(&lock_path).unwrap();
    let stored_pid: u32 = contents.trim().parse().expect("Lock file should contain a valid PID");
    assert_eq!(
        stored_pid,
        std::process::id(),
        "Lock file PID should match current process"
    );

    drop(guard);
    let _ = fs::remove_dir_all(&dir);
}

/// Requirement 7.3: Lock file is removed after guard is dropped.
#[test]
fn lock_file_removed_after_guard_dropped() {
    let dir = temp_dir();
    let lock_path = dir.join("tama96.lock");

    let guard = persistence::acquire_lock(&lock_path).unwrap();
    assert!(lock_path.exists(), "Lock file should exist while guard is alive");

    // Explicitly release via persistence::release_lock
    persistence::release_lock(guard);
    assert!(
        !lock_path.exists(),
        "Lock file should be removed after release_lock"
    );

    let _ = fs::remove_dir_all(&dir);
}
