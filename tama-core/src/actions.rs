use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::characters::CharacterStats;
use crate::state::{LifeStage, PetState};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
    TurnAlreadyChanged,
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
    can_play(state)?;
    let wins = moves.into_iter().filter(|choice| *choice == random_look()).count() as u8;
    Ok(finish_game(state, wins))
}

fn can_play(state: &PetState) -> Result<(), ActionError> {
    if !state.is_alive { return Err(ActionError::PetIsDead); }
    if state.is_sleeping { return Err(ActionError::PetIsSleeping); }
    if state.is_sick { return Err(ActionError::PetIsSick); }
    Ok(())
}

fn random_look() -> Choice {
    if rand::thread_rng().gen_bool(0.5) { Choice::Left } else { Choice::Right }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GameProgress {
    pub turn: u64,
    pub hits: u8,
    pub look: Option<Choice>,
    pub matched: bool,
    pub stars: u32,
    pub best: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Guess {
    pub choice: Choice,
    pub turn: u64,
    pub birth_time: DateTime<Utc>,
}

/// Each view sends the turn it displayed. A repeated key or a stale view cannot
/// score the same turn twice, including the last turn of a completed game.
pub fn guess_direction(state: &mut PetState, guess: Guess) -> Result<(), ActionError> {
    can_play(state)?;
    if guess.birth_time != state.birth_time || guess.turn != state.game.turn {
        return Err(ActionError::TurnAlreadyChanged);
    }
    record_guess(state, guess.choice, random_look());
    Ok(())
}

fn record_guess(state: &mut PetState, choice: Choice, look: Choice) {
    let game = &mut state.game;
    if game.turn % 5 == 0 { game.hits = 0; }
    game.matched = choice == look;
    game.look = Some(look);
    game.hits += u8::from(game.matched);
    game.turn += 1;
    if game.turn % 5 == 0 {
        let wins = game.hits;
        finish_game(state, wins);
    }
}

fn finish_game(state: &mut PetState, wins: u8) -> GameResult {
    state.game.stars = state.game.stars.saturating_add(u32::from(wins));
    state.game.best = state.game.best.max(wins);
    let happiness_gained = if wins >= 3 {
        let old = state.happiness;
        state.happiness = (old + 1).min(4);
        state.happiness - old
    } else {
        0
    };

    state.weight = state.weight.saturating_sub(1).max(1);

    GameResult {
        rounds: 5,
        wins,
        happiness_gained,
    }
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
        if stats.is_sleep_time(now) {
            state.is_sleeping = true;
        }
    }

    Ok(ActionResult::LightsToggled)
}

#[cfg(test)]
mod guessing_tests {
    use super::*;

    #[test]
    fn a_saved_game_resumes_and_banks_each_reward_once() {
        let now = Utc::now();
        let mut pet = PetState::new_egg(now);
        let mut legacy = serde_json::to_value(&pet).unwrap();
        legacy.as_object_mut().unwrap().remove("game");
        pet = serde_json::from_value(legacy).unwrap();
        for look in [Choice::Left, Choice::Right, Choice::Left, Choice::Left] {
            record_guess(&mut pet, Choice::Left, look);
        }
        assert_eq!((pet.game.hits, pet.game.stars, pet.weight), (3, 0, 5));
        let mut saved: PetState = serde_json::from_slice(&serde_json::to_vec(&pet).unwrap()).unwrap();
        record_guess(&mut saved, Choice::Right, Choice::Left);
        assert_eq!((saved.game.stars, saved.game.best, saved.happiness, saved.weight), (3, 3, 1, 4));
        let before = saved.clone();
        assert_eq!(guess_direction(&mut saved, Guess { choice: Choice::Left, birth_time: now, turn: 4 }), Err(ActionError::TurnAlreadyChanged));
        assert_eq!(saved, before);
        record_guess(&mut saved, Choice::Right, Choice::Right);
        assert_eq!((saved.game.hits, saved.game.stars), (1, 3));
    }
}
