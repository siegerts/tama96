use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use chrono::{DateTime, Duration, Utc};

use crate::state::{ActionLogEntry, ActionType, AgentPermissions};

// ── Error Types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionDenied {
    MasterDisabled,
    ActionDisabled(ActionType),
    RateLimited {
        action: ActionType,
        limit: u32,
        used: u32,
    },
}

impl fmt::Display for PermissionDenied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionDenied::MasterDisabled => {
                write!(f, "Agent permissions are disabled (master switch off)")
            }
            PermissionDenied::ActionDisabled(action) => {
                write!(f, "Action {:?} is disabled", action)
            }
            PermissionDenied::RateLimited {
                action,
                limit,
                used,
            } => {
                write!(
                    f,
                    "Action {:?} rate-limited: {used}/{limit} per hour",
                    action
                )
            }
        }
    }
}

impl std::error::Error for PermissionDenied {}

#[derive(Debug)]
pub enum PermissionPersistError {
    Io(io::Error),
    Serialize(serde_json::Error),
}

impl fmt::Display for PermissionPersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionPersistError::Io(e) => write!(f, "I/O error: {e}"),
            PermissionPersistError::Serialize(e) => write!(f, "serialization error: {e}"),
        }
    }
}

impl std::error::Error for PermissionPersistError {}

impl From<io::Error> for PermissionPersistError {
    fn from(e: io::Error) -> Self {
        PermissionPersistError::Io(e)
    }
}

impl From<serde_json::Error> for PermissionPersistError {
    fn from(e: serde_json::Error) -> Self {
        PermissionPersistError::Serialize(e)
    }
}

// ── Permission Checking ─────────────────────────────────────────────────────

/// Check whether an agent is allowed to perform the given action.
///
/// 1. If the master switch is disabled, deny with `MasterDisabled`.
/// 2. If the action is not in `allowed_actions` or `allowed == false`, deny with `ActionDisabled`.
/// 3. If `max_per_hour` is set and usage in the last hour meets or exceeds the limit, deny with `RateLimited`.
/// 4. Prune action_log entries older than 1 hour.
pub fn check_permission(
    permissions: &mut AgentPermissions,
    action: &ActionType,
    now: DateTime<Utc>,
) -> Result<(), PermissionDenied> {
    // Prune stale log entries (older than 1 hour)
    let cutoff = now - Duration::hours(1);
    permissions.action_log.retain(|entry| entry.timestamp >= cutoff);

    // 1. Master switch
    if !permissions.enabled {
        return Err(PermissionDenied::MasterDisabled);
    }

    // 2. Per-action allow/deny
    let action_perm = match permissions.allowed_actions.get(action) {
        Some(perm) if perm.allowed => perm,
        _ => return Err(PermissionDenied::ActionDisabled(action.clone())),
    };

    // 3. Rate limit
    if let Some(limit) = action_perm.max_per_hour {
        let used = permissions
            .action_log
            .iter()
            .filter(|entry| &entry.action == action && entry.timestamp >= cutoff)
            .count() as u32;

        if used >= limit {
            return Err(PermissionDenied::RateLimited {
                action: action.clone(),
                limit,
                used,
            });
        }
    }

    Ok(())
}

// ── Action Logging ──────────────────────────────────────────────────────────

/// Append an entry to the action log.
pub fn log_action(permissions: &mut AgentPermissions, action: ActionType, now: DateTime<Utc>) {
    permissions.action_log.push(ActionLogEntry {
        action,
        timestamp: now,
    });
}

// ── Persistence ─────────────────────────────────────────────────────────────

/// Serialize AgentPermissions to JSON and write atomically (temp file + rename).
pub fn save_permissions(
    permissions: &AgentPermissions,
    path: &Path,
) -> Result<(), PermissionPersistError> {
    let json = serde_json::to_string_pretty(permissions)?;

    let temp_path = path.with_extension("tmp");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&temp_path, json.as_bytes())?;
    fs::rename(&temp_path, path)?;

    Ok(())
}

/// Deserialize AgentPermissions from JSON.
pub fn load_permissions(path: &Path) -> Result<AgentPermissions, PermissionPersistError> {
    let data = fs::read_to_string(path)?;
    let permissions: AgentPermissions = serde_json::from_str(&data)?;
    Ok(permissions)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ActionPermission, AgentPermissions, ActionType};
    use chrono::{Duration, Utc};

    fn default_permissions() -> AgentPermissions {
        AgentPermissions::default()
    }

    #[test]
    fn master_disabled_denies_all() {
        let mut perms = default_permissions();
        perms.enabled = false;
        let now = Utc::now();

        let result = check_permission(&mut perms, &ActionType::FeedMeal, now);
        assert_eq!(result, Err(PermissionDenied::MasterDisabled));
    }

    #[test]
    fn action_disabled_denies() {
        let mut perms = default_permissions();
        perms
            .allowed_actions
            .get_mut(&ActionType::FeedSnack)
            .unwrap()
            .allowed = false;
        let now = Utc::now();

        let result = check_permission(&mut perms, &ActionType::FeedSnack, now);
        assert_eq!(result, Err(PermissionDenied::ActionDisabled(ActionType::FeedSnack)));
    }

    #[test]
    fn missing_action_treated_as_disabled() {
        let mut perms = default_permissions();
        perms.allowed_actions.remove(&ActionType::Discipline);
        let now = Utc::now();

        let result = check_permission(&mut perms, &ActionType::Discipline, now);
        assert_eq!(result, Err(PermissionDenied::ActionDisabled(ActionType::Discipline)));
    }

    #[test]
    fn allowed_action_passes() {
        let mut perms = default_permissions();
        let now = Utc::now();

        let result = check_permission(&mut perms, &ActionType::GetStatus, now);
        assert!(result.is_ok());
    }

    #[test]
    fn rate_limit_enforced() {
        let mut perms = default_permissions();
        perms
            .allowed_actions
            .insert(
                ActionType::FeedMeal,
                ActionPermission {
                    allowed: true,
                    max_per_hour: Some(3),
                },
            );
        let now = Utc::now();

        // Log 3 actions within the last hour
        for i in 0..3 {
            log_action(&mut perms, ActionType::FeedMeal, now - Duration::minutes(i));
        }

        let result = check_permission(&mut perms, &ActionType::FeedMeal, now);
        assert_eq!(
            result,
            Err(PermissionDenied::RateLimited {
                action: ActionType::FeedMeal,
                limit: 3,
                used: 3,
            })
        );
    }

    #[test]
    fn rate_limit_allows_when_under() {
        let mut perms = default_permissions();
        perms
            .allowed_actions
            .insert(
                ActionType::FeedMeal,
                ActionPermission {
                    allowed: true,
                    max_per_hour: Some(3),
                },
            );
        let now = Utc::now();

        // Log 2 actions — under the limit
        log_action(&mut perms, ActionType::FeedMeal, now - Duration::minutes(10));
        log_action(&mut perms, ActionType::FeedMeal, now - Duration::minutes(5));

        let result = check_permission(&mut perms, &ActionType::FeedMeal, now);
        assert!(result.is_ok());
    }

    #[test]
    fn old_log_entries_pruned() {
        let mut perms = default_permissions();
        perms
            .allowed_actions
            .insert(
                ActionType::FeedMeal,
                ActionPermission {
                    allowed: true,
                    max_per_hour: Some(3),
                },
            );
        let now = Utc::now();

        // Log 3 actions, but all older than 1 hour
        for i in 0..3 {
            log_action(
                &mut perms,
                ActionType::FeedMeal,
                now - Duration::hours(2) + Duration::minutes(i),
            );
        }

        // Should pass because old entries get pruned
        let result = check_permission(&mut perms, &ActionType::FeedMeal, now);
        assert!(result.is_ok());
        // Verify pruning happened
        assert!(perms.action_log.is_empty());
    }

    #[test]
    fn log_action_appends_entry() {
        let mut perms = default_permissions();
        let now = Utc::now();

        assert!(perms.action_log.is_empty());
        log_action(&mut perms, ActionType::PlayGame, now);
        assert_eq!(perms.action_log.len(), 1);
        assert_eq!(perms.action_log[0].action, ActionType::PlayGame);
        assert_eq!(perms.action_log[0].timestamp, now);
    }

    #[test]
    fn save_and_load_permissions_round_trip() {
        let dir = std::env::temp_dir().join(format!("tama96_perm_test_{}", rand::random::<u64>()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("permissions.json");

        let mut perms = default_permissions();
        perms.enabled = false;
        perms
            .allowed_actions
            .get_mut(&ActionType::FeedMeal)
            .unwrap()
            .max_per_hour = Some(5);

        save_permissions(&perms, &path).unwrap();
        let loaded = load_permissions(&path).unwrap();

        assert_eq!(loaded.enabled, perms.enabled);
        assert_eq!(
            loaded.allowed_actions.get(&ActionType::FeedMeal).unwrap().max_per_hour,
            Some(5)
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_permissions_creates_parent_dirs() {
        let dir = std::env::temp_dir().join(format!("tama96_perm_test_{}", rand::random::<u64>()));
        let path = dir.join("nested").join("permissions.json");

        let perms = default_permissions();
        save_permissions(&perms, &path).unwrap();
        assert!(path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_permissions_atomic_no_temp_remains() {
        let dir = std::env::temp_dir().join(format!("tama96_perm_test_{}", rand::random::<u64>()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("permissions.json");

        let perms = default_permissions();
        save_permissions(&perms, &path).unwrap();

        let temp_path = path.with_extension("tmp");
        assert!(!temp_path.exists());
        assert!(path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
