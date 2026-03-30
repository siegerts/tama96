use chrono::{DateTime, Utc};

use crate::state::{Character, LifeStage, PetState, TeenType};

/// Check and apply evolution if the pet has reached the threshold for its current stage.
///
/// This handles Baby→Child, Child→Teen, Teen→Adult, Adult→Special.
/// Egg→Baby is handled separately by `engine::check_egg_hatch`.
///
/// Returns true if evolution occurred.
pub fn check_evolution(state: &mut PetState, now: DateTime<Utc>) -> bool {
    let elapsed_in_stage = (now - state.stage_start_time).num_minutes();

    match state.stage {
        LifeStage::Baby => {
            if elapsed_in_stage >= 65 {
                evolve_to(state, LifeStage::Child, Character::Marutchi, now);
                return true;
            }
        }
        LifeStage::Child => {
            if state.age >= 3 {
                let (teen_char, teen_type) =
                    resolve_teen(state.care_mistakes, state.discipline_mistakes);
                state.teen_type = Some(teen_type);
                evolve_to(state, LifeStage::Teen, teen_char, now);
                return true;
            }
        }
        LifeStage::Teen => {
            if state.age >= 6 {
                let adult_char = resolve_adult(
                    &state.character,
                    state.teen_type.unwrap_or(TeenType::Type1),
                    state.care_mistakes,
                    state.discipline_mistakes,
                );
                evolve_to(state, LifeStage::Adult, adult_char, now);
                return true;
            }
        }
        LifeStage::Adult => {
            // Special evolution: Maskutchi from Tamatchi T2 path → Oyajitchi after 4 days
            if state.character == Character::Maskutchi
                && state.teen_type == Some(TeenType::Type2)
                && elapsed_in_stage >= 4 * 24 * 60
            {
                evolve_to(state, LifeStage::Special, Character::Oyajitchi, now);
                return true;
            }
        }
        _ => {}
    }
    false
}

/// Resolve teen character from cumulative mistakes during child stage.
///
/// - care_mistakes 0–2 → Tamatchi; 3+ → Kuchitamatchi
/// - discipline_mistakes 0–2 → Type1; 3+ → Type2
pub fn resolve_teen(care_mistakes: u8, discipline_mistakes: u8) -> (Character, TeenType) {
    let character = if care_mistakes <= 2 {
        Character::Tamatchi
    } else {
        Character::Kuchitamatchi
    };
    let teen_type = if discipline_mistakes <= 2 {
        TeenType::Type1
    } else {
        TeenType::Type2
    };
    (character, teen_type)
}

/// Resolve adult character from teen character, teen type, and cumulative mistakes.
///
/// Full P1 evolution matrix:
///   Tamatchi T1 + 0–2 care + 0 disc   → Mametchi
///   Tamatchi T1 + 0–2 care + 1 disc   → Ginjirotchi
///   Tamatchi T1 + 0–2 care + 2+ disc  → Maskutchi
///   Tamatchi T1 + 3+ care + 0–1 disc  → Kuchipatchi
///   Tamatchi T1 + 3+ care + 2–3 disc  → Nyorotchi
///   Tamatchi T1 + 3+ care + 4+ disc   → Tarakotchi
///   Tamatchi T2 + 0–3 care + 2+ disc  → Maskutchi
///   Tamatchi T2 + 3+ care + 0–1 disc  → Kuchipatchi
///   Tamatchi T2 + 3+ care + 2–3 disc  → Nyorotchi
///   Tamatchi T2 + 3+ care + 4+ disc   → Tarakotchi
///   Tamatchi T2 fallback              → Nyorotchi
///   Kuchitamatchi T1 + 0–1 disc       → Kuchipatchi
///   Kuchitamatchi T1 + 2–3 disc       → Nyorotchi
///   Kuchitamatchi T1 + 4+ disc        → Tarakotchi
///   Kuchitamatchi T2 + 0–1 disc       → Kuchipatchi
///   Kuchitamatchi T2 + 2–3 disc       → Nyorotchi
///   Kuchitamatchi T2 + 4+ disc        → Tarakotchi
///   Safety fallback                   → Nyorotchi
pub fn resolve_adult(
    teen_char: &Character,
    teen_type: TeenType,
    care_mistakes: u8,
    discipline_mistakes: u8,
) -> Character {
    match (teen_char, teen_type) {
        (Character::Tamatchi, TeenType::Type1) => {
            if care_mistakes <= 2 {
                match discipline_mistakes {
                    0 => Character::Mametchi,
                    1 => Character::Ginjirotchi,
                    _ => Character::Maskutchi,
                }
            } else {
                match discipline_mistakes {
                    0..=1 => Character::Kuchipatchi,
                    2..=3 => Character::Nyorotchi,
                    _ => Character::Tarakotchi,
                }
            }
        }
        (Character::Tamatchi, TeenType::Type2) => {
            if care_mistakes <= 3 && discipline_mistakes >= 2 {
                Character::Maskutchi
            } else if care_mistakes >= 3 {
                match discipline_mistakes {
                    0..=1 => Character::Kuchipatchi,
                    2..=3 => Character::Nyorotchi,
                    _ => Character::Tarakotchi,
                }
            } else {
                Character::Nyorotchi // fallback
            }
        }
        (Character::Kuchitamatchi, TeenType::Type1) => match discipline_mistakes {
            0..=1 => Character::Kuchipatchi,
            2..=3 => Character::Nyorotchi,
            _ => Character::Tarakotchi,
        },
        (Character::Kuchitamatchi, TeenType::Type2) => match discipline_mistakes {
            0..=1 => Character::Kuchipatchi,
            2..=3 => Character::Nyorotchi,
            _ => Character::Tarakotchi,
        },
        _ => Character::Nyorotchi, // safety fallback
    }
}

/// Transition the pet to a new stage and character.
/// Resets discipline to 0, clears pending deadlines, updates stage_start_time.
pub fn evolve_to(
    state: &mut PetState,
    stage: LifeStage,
    character: Character,
    now: DateTime<Utc>,
) {
    state.stage = stage;
    state.character = character;
    state.stage_start_time = now;
    state.discipline = 0;
    state.pending_care_deadline = None;
    state.pending_discipline_deadline = None;
}
