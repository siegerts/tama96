use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionResult {
    Fed,
    Snacked,
    Disciplined,
    MedicineGiven,
    Cleaned,
    LightsToggled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionError {
    PetIsDead,
    PetIsSleeping,
    PetIsNotSick,
    NoDisciplineCallPending,
    NoPoop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameResult {
    pub rounds: u8,
    pub wins: u8,
    pub happiness_gained: u8,
}
