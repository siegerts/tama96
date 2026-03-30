use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Core Enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LifeStage {
    Egg,
    Baby,
    Child,
    Teen,
    Adult,
    Special,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Character {
    // Baby
    Babytchi,
    // Child
    Marutchi,
    // Teen
    Tamatchi,
    Kuchitamatchi,
    // Adult
    Mametchi,
    Ginjirotchi,
    Maskutchi,
    Kuchipatchi,
    Nyorotchi,
    Tarakotchi,
    // Special
    Oyajitchi,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TeenType {
    Type1,
    Type2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActionType {
    FeedMeal,
    FeedSnack,
    PlayGame,
    Discipline,
    GiveMedicine,
    CleanPoop,
    ToggleLights,
    GetStatus,
}


// ── PetState ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetState {
    // Identity
    pub stage: LifeStage,
    pub character: Character,
    pub teen_type: Option<TeenType>,

    // Meters
    pub hunger: u8,
    pub happiness: u8,
    pub discipline: u8,
    pub weight: u8,
    pub age: u16,

    // Mistake tracking
    pub care_mistakes: u8,
    pub discipline_mistakes: u8,

    // Status flags
    pub poop_count: u8,
    pub is_sick: bool,
    pub sick_dose_count: u8,
    pub is_sleeping: bool,
    pub is_alive: bool,
    pub lights_on: bool,

    // Timers
    pub last_tick: DateTime<Utc>,
    pub birth_time: DateTime<Utc>,
    pub stage_start_time: DateTime<Utc>,
    pub last_poop_time: DateTime<Utc>,

    // Pending deadlines
    pub pending_care_deadline: Option<DateTime<Utc>>,
    pub pending_discipline_deadline: Option<DateTime<Utc>>,
    pub pending_lights_deadline: Option<DateTime<Utc>>,

    // Snack overfeeding tracker
    pub snack_count_since_last_tick: u8,
}

impl PetState {
    pub fn new_egg(now: DateTime<Utc>) -> Self {
        Self {
            stage: LifeStage::Egg,
            character: Character::Babytchi,
            teen_type: None,
            hunger: 0,
            happiness: 0,
            discipline: 0,
            weight: 5,
            age: 0,
            care_mistakes: 0,
            discipline_mistakes: 0,
            poop_count: 0,
            is_sick: false,
            sick_dose_count: 0,
            is_sleeping: false,
            is_alive: true,
            lights_on: true,
            last_tick: now,
            birth_time: now,
            stage_start_time: now,
            last_poop_time: now,
            pending_care_deadline: None,
            pending_discipline_deadline: None,
            pending_lights_deadline: None,
            snack_count_since_last_tick: 0,
        }
    }
}


// ── Agent Permissions ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPermission {
    pub allowed: bool,
    pub max_per_hour: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLogEntry {
    pub action: ActionType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub enabled: bool,
    pub allowed_actions: HashMap<ActionType, ActionPermission>,
    pub action_log: Vec<ActionLogEntry>,
}

impl Default for AgentPermissions {
    fn default() -> Self {
        let mut allowed_actions = HashMap::new();
        for action in [
            ActionType::FeedMeal,
            ActionType::FeedSnack,
            ActionType::PlayGame,
            ActionType::Discipline,
            ActionType::GiveMedicine,
            ActionType::CleanPoop,
            ActionType::ToggleLights,
            ActionType::GetStatus,
        ] {
            allowed_actions.insert(
                action,
                ActionPermission {
                    allowed: true,
                    max_per_hour: None,
                },
            );
        }
        Self {
            enabled: true,
            allowed_actions,
            action_log: Vec::new(),
        }
    }
}
