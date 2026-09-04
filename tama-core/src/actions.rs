use chrono::{DateTime, Timelike, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::characters::CharacterStats;
use crate::state::{LifeStage, PetState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Choice {
    Left,
    Right,
}

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
    PetIsSick,
    NoDisciplineCallPending,
    NoPoop,
    GameAlreadyFinished,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameResult {
    pub rounds: u8,
    pub wins: u8,
    pub happiness_gained: u8,
}

/// Feed a meal to the pet.
///
/// Preconditions: alive, awake, not sick.
/// Postconditions: hunger = min(old + 1, 4), weight += 1.
/// Returns ActionResult::Fed.
pub fn feed_meal(state: &mut PetState) -> Result<ActionResult, ActionError> {
    if !state.is_alive {
        return Err(ActionError::PetIsDead);
    }
    if state.is_sleeping {
        return Err(ActionError::PetIsSleeping);
    }
    if state.is_sick {
        return Err(ActionError::PetIsSick);
    }

    state.hunger = (state.hunger + 1).min(4);
    state.weight += 1;

    Ok(ActionResult::Fed)
}

/// Feed a snack to the pet.
///
/// Preconditions: alive, awake.
/// Postconditions: happiness = min(old + 1, 4), weight += 2,
///   snack_count_since_last_tick += 1.
///   If baby stage and snack_count_since_last_tick > 3: is_sick = true.
/// Returns ActionResult::Snacked.
pub fn feed_snack(state: &mut PetState) -> Result<ActionResult, ActionError> {
    if !state.is_alive {
        return Err(ActionError::PetIsDead);
    }
    if state.is_sleeping {
        return Err(ActionError::PetIsSleeping);
    }

    state.happiness = (state.happiness + 1).min(4);
    state.weight += 2;
    state.snack_count_since_last_tick += 1;

    if state.stage == LifeStage::Baby && state.snack_count_since_last_tick > 3 {
        state.is_sick = true;
    }

    Ok(ActionResult::Snacked)
}

/// Play the guessing game with the pet.
///
/// Preconditions: alive, awake, not sick.
/// The pet generates 5 random left/right choices. The player wins a round
/// when their move matches the pet's choice.
/// If wins >= 3: happiness = min(old + 1, 4).
/// Weight always decreases: weight = max(old - 1, 1).
/// Returns GameResult { rounds: 5, wins, happiness_gained }.
pub fn play_game(state: &mut PetState, moves: [Choice; 5]) -> Result<GameResult, ActionError> {
    let mut session = start_game(state)?;
    let mut final_outcome = None;
    for m in moves {
        final_outcome = Some(play_round(state, &mut session, m)?);
    }
    let outcome = final_outcome.expect("5 rounds played");
    Ok(GameResult {
        rounds: 5,
        wins: outcome.wins,
        happiness_gained: outcome.happiness_gained,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameSession {
    pub sequence: [Choice; 5],
    pub round: usize,
    pub wins: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoundOutcome {
    pub pet_choice: Choice,
    pub won: bool,
    pub round: usize,
    pub wins: u8,
    pub finished: bool,
    pub happiness_gained: u8,
    pub weight: u8,
}

pub fn start_game(state: &PetState) -> Result<GameSession, ActionError> {
    if !state.is_alive {
        return Err(ActionError::PetIsDead);
    }
    if state.is_sleeping {
        return Err(ActionError::PetIsSleeping);
    }
    if state.is_sick {
        return Err(ActionError::PetIsSick);
    }

    let mut rng = rand::thread_rng();
    let sequence = std::array::from_fn(|_| {
        if rng.gen_bool(0.5) {
            Choice::Left
        } else {
            Choice::Right
        }
    });

    Ok(GameSession {
        sequence,
        round: 0,
        wins: 0,
    })
}

pub fn play_round(
    state: &mut PetState,
    session: &mut GameSession,
    choice: Choice,
) -> Result<RoundOutcome, ActionError> {
    if !state.is_alive {
        return Err(ActionError::PetIsDead);
    }
    if state.is_sleeping {
        return Err(ActionError::PetIsSleeping);
    }
    if state.is_sick {
        return Err(ActionError::PetIsSick);
    }
    if session.round >= session.sequence.len() {
        return Err(ActionError::GameAlreadyFinished);
    }

    let pet_choice = session.sequence[session.round].clone();
    let won = choice == pet_choice;
    if won {
        session.wins += 1;
    }
    session.round += 1;

    let finished = session.round == session.sequence.len();
    let (happiness_gained, weight) = if finished {
        let gained = if session.wins >= 3 {
            let old = state.happiness;
            state.happiness = (old + 1).min(4);
            state.happiness - old
        } else {
            0
        };
        state.weight = state.weight.saturating_sub(1).max(1);
        (gained, state.weight)
    } else {
        (0, state.weight)
    };

    Ok(RoundOutcome {
        pet_choice,
        won,
        round: session.round,
        wins: session.wins,
        finished,
        happiness_gained,
        weight,
    })
}

/// Discipline the pet during a pending discipline call.
///
/// Preconditions: alive, pending_discipline_deadline is Some.
/// Postconditions: discipline = min(old + 25, 100), pending_discipline_deadline = None.
/// Returns ActionResult::Disciplined.
pub fn discipline(state: &mut PetState) -> Result<ActionResult, ActionError> {
    if !state.is_alive {
        return Err(ActionError::PetIsDead);
    }
    if state.pending_discipline_deadline.is_none() {
        return Err(ActionError::NoDisciplineCallPending);
    }

    state.discipline = (state.discipline + 25).min(100);
    state.pending_discipline_deadline = None;

    Ok(ActionResult::Disciplined)
}

/// Give medicine to a sick pet.
///
/// Preconditions: alive, sick.
/// Postconditions: sick_dose_count += 1. If sick_dose_count >= 2: is_sick = false, sick_dose_count = 0.
/// Returns ActionResult::MedicineGiven.
pub fn give_medicine(state: &mut PetState) -> Result<ActionResult, ActionError> {
    if !state.is_alive {
        return Err(ActionError::PetIsDead);
    }
    if !state.is_sick {
        return Err(ActionError::PetIsNotSick);
    }

    state.sick_dose_count += 1;
    if state.sick_dose_count >= 2 {
        state.is_sick = false;
        state.sick_dose_count = 0;
    }

    Ok(ActionResult::MedicineGiven)
}

/// Clean one poop from the pet's area.
///
/// Preconditions: alive, poop_count > 0.
/// Postconditions: poop_count -= 1.
/// Returns ActionResult::Cleaned.
pub fn clean_poop(state: &mut PetState) -> Result<ActionResult, ActionError> {
    if !state.is_alive {
        return Err(ActionError::PetIsDead);
    }
    if state.poop_count == 0 {
        return Err(ActionError::NoPoop);
    }

    state.poop_count -= 1;

    Ok(ActionResult::Cleaned)
}

/// Toggle the lights on or off.
///
/// Preconditions: alive.
/// Postconditions: lights_on flipped. If turning off and pending_lights_deadline exists, deadline cleared.
/// If turning off and current hour is within the pet's sleep window, is_sleeping = true.
/// Returns ActionResult::LightsToggled.
pub fn toggle_lights(state: &mut PetState, now: DateTime<Utc>) -> Result<ActionResult, ActionError> {
    if !state.is_alive {
        return Err(ActionError::PetIsDead);
    }

    let was_on = state.lights_on;
    state.lights_on = !was_on;

    // If turning lights off
    if was_on {
        // Clear pending lights deadline
        state.pending_lights_deadline = None;

        // Check if pet should be sleeping based on character sleep/wake hours
        let stats = CharacterStats::for_character(&state.character);
        let hour = now.hour() as u8;

        let should_sleep = if stats.sleep_hour > stats.wake_hour {
            // Normal case: sleep_hour (e.g. 21) > wake_hour (e.g. 9)
            // Sleep window spans midnight: hour >= sleep_hour OR hour < wake_hour
            hour >= stats.sleep_hour || hour < stats.wake_hour
        } else {
            // Edge case: sleep_hour <= wake_hour (unusual but handle it)
            hour >= stats.sleep_hour && hour < stats.wake_hour
        };

        if should_sleep {
            state.is_sleeping = true;
        }
    }

    Ok(ActionResult::LightsToggled)
}
