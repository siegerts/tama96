use chrono::{DateTime, Duration, Timelike, Utc};
use rand::Rng;

use crate::characters::CharacterStats;
use crate::state::{Character, LifeStage, PetState};

/// Decrement hunger and happiness hearts based on elapsed time and character decay rates.
///
/// - hunger decremented by floor(elapsed / hunger_decay_minutes), clamped to 0
/// - happiness decremented by floor(elapsed / happy_decay_minutes), clamped to 0
/// - If hunger or happiness reaches 0 AND pending_care_deadline is None,
///   set pending_care_deadline to now + 15 minutes
pub fn decay_hearts(
    state: &mut PetState,
    stats: &CharacterStats,
    elapsed_minutes: u16,
    now: DateTime<Utc>,
) {
    let hunger_lost = elapsed_minutes / stats.hunger_decay_minutes;
    let happy_lost = elapsed_minutes / stats.happy_decay_minutes;

    state.hunger = state.hunger.saturating_sub(hunger_lost as u8);
    state.happiness = state.happiness.saturating_sub(happy_lost as u8);

    // Start care deadline if a meter just hit 0 and no deadline is pending
    if (state.hunger == 0 || state.happiness == 0) && state.pending_care_deadline.is_none() {
        state.pending_care_deadline = Some(now + Duration::minutes(15));
    }
}

/// Accumulate poop based on character poop_interval_minutes.
///
/// Calculates how many poops should have accumulated since `state.last_poop_time`,
/// increments `state.poop_count` by that amount (capped at 4), and advances
/// `last_poop_time` to reflect the poops added.
pub fn check_poop(state: &mut PetState, stats: &CharacterStats, now: DateTime<Utc>) {
    let elapsed = (now - state.last_poop_time).num_minutes();
    if elapsed < 0 {
        return;
    }
    let new_poops = elapsed as u16 / stats.poop_interval_minutes;
    if new_poops > 0 {
        state.poop_count = (state.poop_count + new_poops as u8).min(4);
        state.last_poop_time =
            state.last_poop_time + Duration::minutes((new_poops * stats.poop_interval_minutes) as i64);
    }
}

/// Trigger sickness from poop threshold.
///
/// If `poop_count >= 4` and the pet is not already sick, sets `is_sick = true`.
/// (Baby snack overfeeding sickness is handled in `actions::feed_snack`.)
pub fn check_sickness(state: &mut PetState) {
    if state.poop_count >= 4 && !state.is_sick {
        state.is_sick = true;
    }
}

/// Randomly generate a false attention (discipline) call with a 15-minute deadline.
///
/// If no pending discipline deadline exists, there is a ~10% chance per tick
/// of generating one. The deadline is set to `now + 15 minutes`.
pub fn maybe_generate_discipline_call(state: &mut PetState, now: DateTime<Utc>) {
    if state.pending_discipline_deadline.is_none() {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.1) {
            state.pending_discipline_deadline = Some(now + Duration::minutes(15));
        }
    }
}

/// Increment discipline_mistakes if the discipline deadline has expired.
///
/// If `pending_discipline_deadline` is `Some` and `now >= deadline`,
/// increments `discipline_mistakes` by 1 and clears the deadline.
pub fn check_discipline_deadline(state: &mut PetState, now: DateTime<Utc>) {
    if let Some(deadline) = state.pending_discipline_deadline {
        if now >= deadline {
            state.discipline_mistakes += 1;
            state.pending_discipline_deadline = None;
        }
    }
}

/// Check if the care deadline has expired.
///
/// If `pending_care_deadline` is `Some` and `now >= deadline`,
/// increments `care_mistakes` by 1 and clears the deadline.
pub fn check_care_deadlines(state: &mut PetState, now: DateTime<Utc>) {
    if let Some(deadline) = state.pending_care_deadline {
        if now >= deadline {
            state.care_mistakes += 1;
            state.pending_care_deadline = None;
        }
    }
}

/// Manage sleep transition based on character sleep hours.
///
/// Determines if the current hour falls within the pet's sleep window.
/// - If the pet should be sleeping and lights are on: sets a 15-minute
///   `pending_lights_deadline` (if none exists) so the player can turn them off.
/// - If the pet should be sleeping and lights are already off: sets `is_sleeping = true`.
pub fn check_sleep(state: &mut PetState, stats: &CharacterStats, now: DateTime<Utc>) {
    let hour = now.hour() as u8;

    let should_sleep = if stats.sleep_hour > stats.wake_hour {
        // Sleep window spans midnight: e.g. sleep_hour=21, wake_hour=9
        hour >= stats.sleep_hour || hour < stats.wake_hour
    } else {
        // Unusual case where sleep_hour <= wake_hour
        hour >= stats.sleep_hour && hour < stats.wake_hour
    };

    if should_sleep {
        if state.lights_on {
            if state.pending_lights_deadline.is_none() {
                state.pending_lights_deadline = Some(now + Duration::minutes(15));
            }
        } else {
            state.is_sleeping = true;
        }
    }
}

/// Manage wake transition.
///
/// If the current hour is within the pet's awake window (>= wake_hour and < sleep_hour),
/// wakes the pet up: sets `is_sleeping = false`, `lights_on = true`,
/// increments `age` by 1 (1 day = 1 year), and resets `snack_count_since_last_tick`.
pub fn check_wake(state: &mut PetState, stats: &CharacterStats, now: DateTime<Utc>) {
    let hour = now.hour() as u8;

    let should_be_awake = if stats.sleep_hour > stats.wake_hour {
        // Normal: awake window is wake_hour..sleep_hour
        hour >= stats.wake_hour && hour < stats.sleep_hour
    } else {
        // Unusual case
        hour >= stats.wake_hour || hour < stats.sleep_hour
    };

    if should_be_awake {
        state.is_sleeping = false;
        state.lights_on = true;
        state.age += 1;
        state.snack_count_since_last_tick = 0;
    }
}

/// Check all death conditions.
///
/// Death conditions:
/// 1. Old age: age >= character's max_lifespan_days (if > 0)
/// 2. Neglect: hunger == 0 AND happiness == 0 for > 12 consecutive hours
/// 3. Untreated sickness: sick for > 24 hours (TODO: needs sick_since field)
/// 4. Baby snack overfeeding: snack_count > 5 during Baby stage
pub fn check_death(state: &mut PetState, stats: &CharacterStats, now: DateTime<Utc>) {
    // 1. Old age
    if stats.max_lifespan_days > 0 && state.age >= stats.max_lifespan_days {
        kill(state);
        return;
    }

    // 2. Neglect — sustained empty meters for 12+ hours
    if state.hunger == 0 && state.happiness == 0 {
        if let Some(deadline) = state.pending_care_deadline {
            if (now - deadline).num_hours() >= 12 {
                kill(state);
                return;
            }
        }
    }

    // 3. Untreated sickness for 24 hours
    // TODO: Add sick_since field to PetState for accurate tracking

    // 4. Baby snack overfeeding
    if state.stage == LifeStage::Baby && state.snack_count_since_last_tick > 5 {
        kill(state);
    }
}

/// Kill the pet — set is_alive to false and stage to Dead.
pub fn kill(state: &mut PetState) {
    state.is_alive = false;
    state.stage = LifeStage::Dead;
}

/// Core tick function — called every ~60 seconds by the host process.
/// Advances the pet simulation by the elapsed time since last_tick.
///
/// - Dead pets: complete no-op (state unchanged)
/// - Egg stage: only check hatch timer, then update last_tick
/// - Sleeping pets: only check wake, then update last_tick (no decay)
/// - Awake pets: full simulation cycle
pub fn tick(state: &mut PetState, now: DateTime<Utc>) {
    // Dead pets are a complete no-op
    if !state.is_alive {
        return;
    }

    // Egg stage: only check hatch timer
    if state.stage == LifeStage::Egg {
        check_egg_hatch(state, now);
        state.last_tick = now;
        return;
    }

    // Sleeping pets: only check wake, no decay
    if state.is_sleeping {
        let stats = CharacterStats::for_character(&state.character);
        check_wake(state, &stats, now);
        state.last_tick = now;
        return;
    }

    let stats = CharacterStats::for_character(&state.character);
    let elapsed = (now - state.last_tick).num_minutes().max(0) as u16;

    // 1. Heart decay
    decay_hearts(state, &stats, elapsed, now);

    // 2. Care deadline management
    check_care_deadlines(state, now);

    // 3. Poop accumulation
    check_poop(state, &stats, now);

    // 4. Sickness checks
    check_sickness(state);

    // 5. Discipline call generation and deadline check
    maybe_generate_discipline_call(state, now);
    check_discipline_deadline(state, now);

    // 6. Sleep time check
    check_sleep(state, &stats, now);

    // 7. Death conditions
    check_death(state, &stats, now);

    // 8. Evolution
    crate::evolution::check_evolution(state, now);

    // 9. Update last_tick
    state.last_tick = now;
}

/// Check if the egg should hatch into a baby.
/// Egg hatches after 5 minutes since stage_start_time.
fn check_egg_hatch(state: &mut PetState, now: DateTime<Utc>) {
    let elapsed = (now - state.stage_start_time).num_minutes();
    if elapsed >= 5 {
        state.stage = LifeStage::Baby;
        state.character = Character::Babytchi;
        state.stage_start_time = now;
        state.hunger = 0;
        state.happiness = 0;
    }
}
