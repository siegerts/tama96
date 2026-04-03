use chrono::{DateTime, TimeZone, Utc};
use proptest::prelude::*;
use tama_core::actions;
use tama_core::actions::Choice;
use tama_core::characters::CharacterStats;
use tama_core::engine;
use tama_core::state::*;

// ── Arbitrary generators ────────────────────────────────────────────────────

fn arb_life_stage() -> impl Strategy<Value = LifeStage> {
    prop_oneof![
        Just(LifeStage::Egg),
        Just(LifeStage::Baby),
        Just(LifeStage::Child),
        Just(LifeStage::Teen),
        Just(LifeStage::Adult),
        Just(LifeStage::Special),
        Just(LifeStage::Dead),
    ]
}

fn arb_character() -> impl Strategy<Value = Character> {
    prop_oneof![
        Just(Character::Babytchi),
        Just(Character::Marutchi),
        Just(Character::Tamatchi),
        Just(Character::Kuchitamatchi),
        Just(Character::Mametchi),
        Just(Character::Ginjirotchi),
        Just(Character::Maskutchi),
        Just(Character::Kuchipatchi),
        Just(Character::Nyorotchi),
        Just(Character::Tarakotchi),
        Just(Character::Oyajitchi),
    ]
}

fn arb_teen_type() -> impl Strategy<Value = TeenType> {
    prop_oneof![Just(TeenType::Type1), Just(TeenType::Type2)]
}

fn arb_datetime() -> impl Strategy<Value = DateTime<Utc>> {
    (1_704_067_200i64..1_735_689_600i64).prop_map(|secs| Utc.timestamp_opt(secs, 0).unwrap())
}

fn arb_optional_datetime() -> impl Strategy<Value = Option<DateTime<Utc>>> {
    prop_oneof![Just(None), arb_datetime().prop_map(Some)]
}

fn character_for_stage(stage: &LifeStage) -> BoxedStrategy<Character> {
    match stage {
        LifeStage::Egg | LifeStage::Baby => Just(Character::Babytchi).boxed(),
        LifeStage::Child => Just(Character::Marutchi).boxed(),
        LifeStage::Teen => {
            prop_oneof![Just(Character::Tamatchi), Just(Character::Kuchitamatchi)].boxed()
        }
        LifeStage::Adult => prop_oneof![
            Just(Character::Mametchi),
            Just(Character::Ginjirotchi),
            Just(Character::Maskutchi),
            Just(Character::Kuchipatchi),
            Just(Character::Nyorotchi),
            Just(Character::Tarakotchi),
        ]
        .boxed(),
        LifeStage::Special => Just(Character::Oyajitchi).boxed(),
        LifeStage::Dead => arb_character().boxed(),
    }
}

fn teen_type_for_stage(stage: &LifeStage) -> BoxedStrategy<Option<TeenType>> {
    match stage {
        LifeStage::Teen | LifeStage::Adult | LifeStage::Special => {
            arb_teen_type().prop_map(Some).boxed()
        }
        _ => Just(None).boxed(),
    }
}

/// Generate a PetState that satisfies all data model invariants.
fn arb_valid_pet_state() -> BoxedStrategy<PetState> {
    arb_life_stage()
        .prop_flat_map(|stage| {
            let char_strat = character_for_stage(&stage);
            let teen_strat = teen_type_for_stage(&stage);
            (
                Just(stage),
                char_strat,
                teen_strat,
                0u8..=4,                         // hunger
                0u8..=4,                         // happiness
                (0u8..=4).prop_map(|v| v * 25),  // discipline
                1u8..=100,                       // weight
                0u16..=30,                       // age
                0u8..=20,                        // care_mistakes
                0u8..=20,                        // discipline_mistakes
            )
        })
        .prop_flat_map(
            |(stage, character, teen_type, hunger, happiness, discipline, weight, age, care_mistakes, discipline_mistakes)| {
                (
                    Just((stage, character, teen_type, hunger, happiness, discipline, weight, age, care_mistakes, discipline_mistakes)),
                    0u8..=4,                     // poop_count
                    proptest::bool::ANY,         // is_sick
                    0u8..=2,                     // sick_dose_count
                    proptest::bool::ANY,         // is_sleeping
                    proptest::bool::ANY,         // lights_on
                    0u8..=10,                    // snack_count_since_last_tick
                    arb_datetime(),              // last_tick
                    arb_datetime(),              // birth_time
                    arb_datetime(),              // stage_start_time
                    arb_datetime(),              // last_poop_time
                    arb_optional_datetime(),     // pending_care_deadline
                )
            },
        )
        .prop_flat_map(
            |(core, poop_count, is_sick, sick_dose_count, is_sleeping, lights_on, snack_count, last_tick, birth_time, stage_start_time, last_poop_time, pending_care_deadline)| {
                (
                    Just((core, poop_count, is_sick, sick_dose_count, is_sleeping, lights_on, snack_count, last_tick, birth_time, stage_start_time, last_poop_time, pending_care_deadline)),
                    arb_optional_datetime(),     // pending_discipline_deadline
                    arb_optional_datetime(),     // pending_lights_deadline
                )
            },
        )
        .prop_map(
            |((core, poop_count, is_sick, sick_dose_count, is_sleeping, lights_on, snack_count, last_tick, birth_time, stage_start_time, last_poop_time, pending_care_deadline), pending_discipline_deadline, pending_lights_deadline)| {
                let (stage, character, teen_type, hunger, happiness, discipline, weight, age, care_mistakes, discipline_mistakes) = core;
                let is_alive = stage != LifeStage::Dead;
                PetState {
                    stage,
                    character,
                    teen_type,
                    hunger,
                    happiness,
                    discipline,
                    weight,
                    age,
                    care_mistakes,
                    discipline_mistakes,
                    poop_count,
                    is_sick,
                    sick_dose_count,
                    is_sleeping,
                    is_alive,
                    lights_on,
                    last_tick,
                    birth_time,
                    stage_start_time,
                    last_poop_time,
                    pending_care_deadline,
                    pending_discipline_deadline,
                    pending_lights_deadline,
                    snack_count_since_last_tick: snack_count,
                }
            },
        )
        .boxed()
}

fn make_alive(mut state: PetState) -> PetState {
    if state.stage == LifeStage::Dead {
        state.stage = LifeStage::Baby;
        state.character = Character::Babytchi;
        state.teen_type = None;
    }
    state.is_alive = true;
    state
}

fn arb_alive_pet_state() -> BoxedStrategy<PetState> {
    arb_valid_pet_state().prop_map(make_alive).boxed()
}

fn arb_awake_alive_pet_state() -> BoxedStrategy<PetState> {
    arb_alive_pet_state()
        .prop_map(|mut state| {
            state.is_sleeping = false;
            state
        })
        .boxed()
}

fn arb_feedable_pet_state() -> BoxedStrategy<PetState> {
    arb_awake_alive_pet_state()
        .prop_map(|mut state| {
            state.is_sick = false;
            state.sick_dose_count = 0;
            state
        })
        .boxed()
}


// ── Property 1: State invariants (meter bounds) ─────────────────────────────
// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**

proptest! {
    #[test]
    fn prop_state_invariants_meter_bounds(state in arb_valid_pet_state()) {
        // Requirement 1.1: hunger in [0, 4]
        prop_assert!(state.hunger <= 4, "hunger {} out of bounds [0,4]", state.hunger);

        // Requirement 1.2: happiness in [0, 4]
        prop_assert!(state.happiness <= 4, "happiness {} out of bounds [0,4]", state.happiness);

        // Requirement 1.3: discipline in [0, 100] and multiple of 25
        prop_assert!(state.discipline <= 100, "discipline {} out of bounds [0,100]", state.discipline);
        prop_assert!(
            state.discipline % 25 == 0,
            "discipline {} is not a multiple of 25",
            state.discipline
        );

        // Requirement 1.4: weight >= 1
        prop_assert!(state.weight >= 1, "weight {} is below minimum 1", state.weight);

        // Requirement 1.5: poop_count in [0, 4]
        prop_assert!(state.poop_count <= 4, "poop_count {} out of bounds [0,4]", state.poop_count);
    }
}

// ── Property 2: Stage-character consistency ─────────────────────────────────
// **Validates: Requirements 1.6, 1.7**

fn is_valid_character_for_stage(stage: &LifeStage, character: &Character) -> bool {
    match stage {
        LifeStage::Egg | LifeStage::Baby => matches!(character, Character::Babytchi),
        LifeStage::Child => matches!(character, Character::Marutchi),
        LifeStage::Teen => matches!(character, Character::Tamatchi | Character::Kuchitamatchi),
        LifeStage::Adult => matches!(
            character,
            Character::Mametchi
                | Character::Ginjirotchi
                | Character::Maskutchi
                | Character::Kuchipatchi
                | Character::Nyorotchi
                | Character::Tarakotchi
        ),
        LifeStage::Special => matches!(character, Character::Oyajitchi),
        LifeStage::Dead => true,
    }
}

proptest! {
    #[test]
    fn prop_stage_character_consistency(state in arb_valid_pet_state()) {
        // Requirement 1.6: Character variant must be valid for current LifeStage
        prop_assert!(
            is_valid_character_for_stage(&state.stage, &state.character),
            "Character {:?} is not valid for stage {:?}",
            state.character,
            state.stage
        );

        // Requirement 1.7: teen_type is Some only when stage is Teen, Adult, or Special
        match state.stage {
            LifeStage::Teen | LifeStage::Adult | LifeStage::Special => {
                prop_assert!(
                    state.teen_type.is_some(),
                    "teen_type should be Some for stage {:?}",
                    state.stage
                );
            }
            _ => {
                prop_assert!(
                    state.teen_type.is_none(),
                    "teen_type should be None for stage {:?}, got {:?}",
                    state.stage,
                    state.teen_type
                );
            }
        }
    }
}


// ── Property 7: Feed meal correctness ───────────────────────────────────────
// **Validates: Requirements 3.1, 3.11**

proptest! {
    #[test]
    fn prop_feed_meal_correctness(mut state in arb_feedable_pet_state()) {
        let old_hunger = state.hunger;
        let old_weight = state.weight;

        let result = actions::feed_meal(&mut state);
        prop_assert!(result.is_ok(), "feed_meal should succeed for alive, awake, non-sick pet");

        // Requirement 3.1: hunger = min(old + 1, 4)
        let expected_hunger = (old_hunger + 1).min(4);
        prop_assert_eq!(
            state.hunger, expected_hunger,
            "hunger should be min(old_hunger + 1, 4) = min({} + 1, 4) = {}, got {}",
            old_hunger, expected_hunger, state.hunger
        );

        // Requirement 3.11: weight = old + 1 (even when hunger is already at 4)
        let expected_weight = old_weight + 1;
        prop_assert_eq!(
            state.weight, expected_weight,
            "weight should be old_weight + 1 = {} + 1 = {}, got {}",
            old_weight, expected_weight, state.weight
        );
    }
}


// ── Property 8: Feed snack correctness ──────────────────────────────────────
// **Validates: Requirement 3.2**

proptest! {
    #[test]
    fn prop_feed_snack_correctness(mut state in arb_awake_alive_pet_state()) {
        let old_happiness = state.happiness;
        let old_weight = state.weight;

        let result = actions::feed_snack(&mut state);
        prop_assert!(result.is_ok(), "feed_snack should succeed for alive, awake pet");

        // Requirement 3.2: happiness = min(old + 1, 4)
        let expected_happiness = (old_happiness + 1).min(4);
        prop_assert_eq!(
            state.happiness, expected_happiness,
            "happiness should be min(old_happiness + 1, 4) = min({} + 1, 4) = {}, got {}",
            old_happiness, expected_happiness, state.happiness
        );

        // Weight = old + 2
        let expected_weight = old_weight + 2;
        prop_assert_eq!(
            state.weight, expected_weight,
            "weight should be old_weight + 2 = {} + 2 = {}, got {}",
            old_weight, expected_weight, state.weight
        );
    }
}


// ── Choice generator ────────────────────────────────────────────────────────

fn arb_choice() -> impl Strategy<Value = Choice> {
    prop_oneof![Just(Choice::Left), Just(Choice::Right)]
}


// ── Property 9: Game outcome correctness ────────────────────────────────────
// **Validates: Requirements 3.4, 3.5**

proptest! {
    #[test]
    fn prop_game_outcome_correctness(
        mut state in arb_feedable_pet_state(),
        m0 in arb_choice(),
        m1 in arb_choice(),
        m2 in arb_choice(),
        m3 in arb_choice(),
        m4 in arb_choice(),
    ) {
        let old_happiness = state.happiness;
        let old_weight = state.weight;
        let moves = [m0, m1, m2, m3, m4];

        let result = actions::play_game(&mut state, moves);
        prop_assert!(result.is_ok(), "play_game should succeed for alive, awake, non-sick pet");
        let game = result.unwrap();

        // 1. Always 5 rounds
        prop_assert_eq!(game.rounds, 5, "rounds should always be 5, got {}", game.rounds);

        // 2. Wins in [0, 5]
        prop_assert!(game.wins <= 5, "wins {} should be in [0, 5]", game.wins);

        // 3. Requirement 3.4: If wins >= 3, happiness = min(old + 1, 4)
        if game.wins >= 3 {
            let expected_happiness = (old_happiness + 1).min(4);
            prop_assert_eq!(
                state.happiness, expected_happiness,
                "wins >= 3: happiness should be min({} + 1, 4) = {}, got {}",
                old_happiness, expected_happiness, state.happiness
            );
            // happiness_gained should reflect the actual increase
            let expected_gained = expected_happiness - old_happiness;
            prop_assert_eq!(
                game.happiness_gained, expected_gained,
                "happiness_gained should be {}, got {}",
                expected_gained, game.happiness_gained
            );
        } else {
            // Happiness unchanged when wins < 3
            prop_assert_eq!(
                state.happiness, old_happiness,
                "wins < 3: happiness should remain {}, got {}",
                old_happiness, state.happiness
            );
            prop_assert_eq!(
                game.happiness_gained, 0,
                "wins < 3: happiness_gained should be 0, got {}",
                game.happiness_gained
            );
        }

        // 4. Requirement 3.5: Weight always decreases by 1, floored at 1
        let expected_weight = old_weight.saturating_sub(1).max(1);
        prop_assert_eq!(
            state.weight, expected_weight,
            "weight should be max({} - 1, 1) = {}, got {}",
            old_weight, expected_weight, state.weight
        );
    }
}


// ── Property 10: Discipline action correctness ──────────────────────────────
// **Validates: Requirements 3.6, 3.7**

proptest! {
    #[test]
    fn prop_discipline_with_pending_call(mut state in arb_valid_pet_state(), deadline in arb_datetime()) {
        // Filter to alive pets only
        prop_assume!(state.is_alive);

        // Set up precondition: pending discipline call exists
        state.pending_discipline_deadline = Some(deadline);
        let old_discipline = state.discipline;

        let result = actions::discipline(&mut state);
        prop_assert!(result.is_ok(), "discipline should succeed when pending call exists");
        prop_assert_eq!(result.unwrap(), actions::ActionResult::Disciplined);

        // Requirement 3.6: discipline = min(old + 25, 100)
        let expected_discipline = (old_discipline + 25).min(100);
        prop_assert_eq!(
            state.discipline, expected_discipline,
            "discipline should be min({} + 25, 100) = {}, got {}",
            old_discipline, expected_discipline, state.discipline
        );

        // Requirement 3.6: pending deadline cleared
        prop_assert!(
            state.pending_discipline_deadline.is_none(),
            "pending_discipline_deadline should be None after discipline, got {:?}",
            state.pending_discipline_deadline
        );
    }

    #[test]
    fn prop_discipline_without_pending_call(mut state in arb_valid_pet_state()) {
        // Filter to alive pets only
        prop_assume!(state.is_alive);

        // Set up precondition: no pending discipline call
        state.pending_discipline_deadline = None;

        let result = actions::discipline(&mut state);

        // Requirement 3.7: returns NoDisciplineCallPending error
        prop_assert!(result.is_err(), "discipline should fail when no pending call");
        prop_assert_eq!(
            result.unwrap_err(),
            actions::ActionError::NoDisciplineCallPending,
            "should return NoDisciplineCallPending error"
        );
    }
}


// ── Property 11: Medicine curing ────────────────────────────────────────────
// **Validates: Requirement 3.8**

proptest! {
    #[test]
    fn prop_medicine_curing(mut state in arb_valid_pet_state()) {
        // Filter to alive pets and force sick state with 0 doses to test full two-dose cycle
        prop_assume!(state.is_alive);
        state.is_sick = true;
        state.sick_dose_count = 0;

        // First dose
        let result1 = actions::give_medicine(&mut state);
        prop_assert!(result1.is_ok(), "first give_medicine should succeed for alive, sick pet");
        prop_assert!(state.is_sick, "pet should still be sick after first dose");
        prop_assert_eq!(
            state.sick_dose_count, 1,
            "sick_dose_count should be 1 after first dose, got {}",
            state.sick_dose_count
        );

        // Second dose
        let result2 = actions::give_medicine(&mut state);
        prop_assert!(result2.is_ok(), "second give_medicine should succeed for alive, sick pet");
        prop_assert!(
            !state.is_sick,
            "pet should not be sick after second dose"
        );
        prop_assert_eq!(
            state.sick_dose_count, 0,
            "sick_dose_count should be 0 after cure, got {}",
            state.sick_dose_count
        );
    }
}


// ── Property 12: Action precondition enforcement ────────────────────────────
// **Validates: Requirements 3.12, 3.13**

proptest! {
    /// Case 1: Dead pet returns PetIsDead for all actions.
    #[test]
    fn prop_dead_pet_returns_pet_is_dead(mut state in arb_valid_pet_state()) {
        // Force dead state
        state.is_alive = false;
        state.stage = LifeStage::Dead;

        let now = Utc::now();
        let dummy_moves = [Choice::Left, Choice::Left, Choice::Left, Choice::Left, Choice::Left];

        // Every action must return Err(ActionError::PetIsDead)
        prop_assert_eq!(
            actions::feed_meal(&mut state),
            Err(actions::ActionError::PetIsDead),
            "feed_meal should return PetIsDead for dead pet"
        );
        prop_assert_eq!(
            actions::feed_snack(&mut state),
            Err(actions::ActionError::PetIsDead),
            "feed_snack should return PetIsDead for dead pet"
        );
        prop_assert_eq!(
            actions::play_game(&mut state, dummy_moves),
            Err(actions::ActionError::PetIsDead),
            "play_game should return PetIsDead for dead pet"
        );
        prop_assert_eq!(
            actions::discipline(&mut state),
            Err(actions::ActionError::PetIsDead),
            "discipline should return PetIsDead for dead pet"
        );
        prop_assert_eq!(
            actions::give_medicine(&mut state),
            Err(actions::ActionError::PetIsDead),
            "give_medicine should return PetIsDead for dead pet"
        );
        prop_assert_eq!(
            actions::clean_poop(&mut state),
            Err(actions::ActionError::PetIsDead),
            "clean_poop should return PetIsDead for dead pet"
        );
        prop_assert_eq!(
            actions::toggle_lights(&mut state, now),
            Err(actions::ActionError::PetIsDead),
            "toggle_lights should return PetIsDead for dead pet"
        );
    }

    /// Case 2: Sleeping pet returns PetIsSleeping for feed_meal and play_game.
    #[test]
    fn prop_sleeping_pet_returns_pet_is_sleeping(mut state in arb_valid_pet_state()) {
        // Filter to alive pets, then force sleeping
        prop_assume!(state.is_alive);
        state.is_sleeping = true;

        let dummy_moves = [Choice::Left, Choice::Left, Choice::Left, Choice::Left, Choice::Left];

        prop_assert_eq!(
            actions::feed_meal(&mut state),
            Err(actions::ActionError::PetIsSleeping),
            "feed_meal should return PetIsSleeping for sleeping pet"
        );
        prop_assert_eq!(
            actions::play_game(&mut state, dummy_moves),
            Err(actions::ActionError::PetIsSleeping),
            "play_game should return PetIsSleeping for sleeping pet"
        );
    }
}


// ── Property 5: Sleep immunity ───────────────────────────────────────────────
// **Validates: Requirement 2.6**

proptest! {
    #[test]
    fn prop_sleep_immunity(mut state in arb_valid_pet_state()) {
        // Filter to alive, sleeping, non-egg pets
        prop_assume!(state.is_alive && state.is_sleeping && state.stage != LifeStage::Egg);

        let old_hunger = state.hunger;
        let old_happiness = state.happiness;
        let stats = CharacterStats::for_character(&state.character);

        // Create a time that's within the sleep window
        // Use sleep_hour + 1 (mod 24) to ensure we're in the sleep window
        let sleep_hour = stats.sleep_hour as u32;
        let now = Utc.with_ymd_and_hms(2024, 6, 15, (sleep_hour + 1) % 24, 30, 0).unwrap();
        state.last_tick = now - chrono::Duration::minutes(60);

        // Simulate what tick() does for sleeping pets: only check_wake, no decay
        // Since we're in the sleep window, check_wake should NOT wake the pet
        engine::check_wake(&mut state, &stats, now);

        // If pet is still sleeping (which it should be since we're in sleep window),
        // hunger and happiness must be unchanged
        if state.is_sleeping {
            prop_assert_eq!(state.hunger, old_hunger, "hunger should not change during sleep");
            prop_assert_eq!(state.happiness, old_happiness, "happiness should not change during sleep");
        }
    }
}


// ── Property 3: Heart decay correctness ─────────────────────────────────────
// **Validates: Requirements 2.1, 2.2**

proptest! {
    #[test]
    fn prop_heart_decay_correctness(
        mut state in arb_valid_pet_state(),
        elapsed in 0u16..=120,
    ) {
        // Filter to alive, awake, non-egg pets
        prop_assume!(state.is_alive && !state.is_sleeping && state.stage != LifeStage::Egg);

        let stats = CharacterStats::for_character(&state.character);
        let old_hunger = state.hunger;
        let old_happiness = state.happiness;
        let now = state.last_tick;

        engine::decay_hearts(&mut state, &stats, elapsed, now);

        // Requirement 2.1: hunger decremented by floor(elapsed / hunger_decay_minutes), clamped to 0
        let expected_hunger = old_hunger.saturating_sub((elapsed / stats.hunger_decay_minutes) as u8);
        prop_assert_eq!(
            state.hunger, expected_hunger,
            "hunger should be {}.saturating_sub({} / {}) = {}, got {}",
            old_hunger, elapsed, stats.hunger_decay_minutes, expected_hunger, state.hunger
        );

        // Requirement 2.2: happiness decremented by floor(elapsed / happy_decay_minutes), clamped to 0
        let expected_happiness = old_happiness.saturating_sub((elapsed / stats.happy_decay_minutes) as u8);
        prop_assert_eq!(
            state.happiness, expected_happiness,
            "happiness should be {}.saturating_sub({} / {}) = {}, got {}",
            old_happiness, elapsed, stats.happy_decay_minutes, expected_happiness, state.happiness
        );
    }
}


// ── Property 4: Care deadline lifecycle ─────────────────────────────────────
// **Validates: Requirements 2.3, 2.4**

proptest! {
    /// Test case 1: When hunger or happiness reaches 0 and no deadline exists,
    /// a 15-minute care deadline is created.
    #[test]
    fn prop_care_deadline_created_when_meter_hits_zero(mut state in arb_valid_pet_state(), now in arb_datetime()) {
        // Filter to alive, awake, non-egg pets
        prop_assume!(state.is_alive && !state.is_sleeping && state.stage != LifeStage::Egg);

        // Force hunger or happiness to 0, and no pending deadline
        state.hunger = 0;
        state.pending_care_deadline = None;

        let stats = CharacterStats::for_character(&state.character);

        // Call with 0 elapsed so no further decay, but the function checks if meters are at 0
        engine::decay_hearts(&mut state, &stats, 0, now);

        // Requirement 2.3: a 15-minute deadline should be created
        prop_assert!(
            state.pending_care_deadline.is_some(),
            "pending_care_deadline should be Some when hunger is 0 and no deadline existed"
        );
        let expected_deadline = now + chrono::Duration::minutes(15);
        prop_assert_eq!(
            state.pending_care_deadline.unwrap(),
            expected_deadline,
            "deadline should be now + 15 minutes = {:?}, got {:?}",
            expected_deadline,
            state.pending_care_deadline.unwrap()
        );
    }

    /// Test case 2: An existing care deadline is not overwritten when meters are still at 0.
    #[test]
    fn prop_care_deadline_not_overwritten(
        mut state in arb_valid_pet_state(),
        existing_deadline in arb_datetime(),
        now in arb_datetime(),
    ) {
        // Filter to alive, awake, non-egg pets
        prop_assume!(state.is_alive && !state.is_sleeping && state.stage != LifeStage::Egg);

        // Force hunger to 0 and set an existing deadline
        state.hunger = 0;
        state.pending_care_deadline = Some(existing_deadline);

        let stats = CharacterStats::for_character(&state.character);

        // Call with 0 elapsed — deadline should not be overwritten
        engine::decay_hearts(&mut state, &stats, 0, now);

        // Requirement 2.3: existing deadline must be preserved
        prop_assert_eq!(
            state.pending_care_deadline,
            Some(existing_deadline),
            "pending_care_deadline should remain {:?}, got {:?}",
            existing_deadline,
            state.pending_care_deadline.unwrap()
        );
    }
}


// ── Property 6: Dead pets don't tick ─────────────────────────────────────────
// **Validates: Requirement 2.7**

proptest! {
    #[test]
    fn prop_dead_pets_dont_tick(state in arb_valid_pet_state()) {
        // Force dead state
        let mut state = state;
        state.is_alive = false;
        state.stage = LifeStage::Dead;

        let snapshot = state.clone();
        let now = state.last_tick + chrono::Duration::minutes(60);

        // tick() is a complete no-op for dead pets — not even last_tick changes
        engine::tick(&mut state, now);

        prop_assert_eq!(state, snapshot, "dead pet state should be completely unchanged after tick");
    }
}


// ── Property 20: Death from old age ─────────────────────────────────────────
// **Validates: Requirement 5.1**

proptest! {
    #[test]
    fn prop_death_from_old_age(mut state in arb_valid_pet_state()) {
        let stats = CharacterStats::for_character(&state.character);
        // Only test characters with a finite lifespan
        prop_assume!(state.is_alive && stats.max_lifespan_days > 0);

        // Set age to exactly the max lifespan
        state.age = stats.max_lifespan_days;
        let now = state.last_tick + chrono::Duration::minutes(1);

        engine::check_death(&mut state, &stats, now);

        prop_assert!(!state.is_alive, "pet should be dead when age >= max_lifespan_days");
        prop_assert_eq!(
            state.stage, LifeStage::Dead,
            "stage should be Dead when age >= max_lifespan_days"
        );
    }
}


// ── Property 13: Evolution determinism ──────────────────────────────────────
// **Validates: Requirements 4.3, 4.4, 4.7**

proptest! {
    #[test]
    fn prop_evolution_determinism(
        teen_char in prop_oneof![Just(Character::Tamatchi), Just(Character::Kuchitamatchi)],
        teen_type in prop_oneof![Just(TeenType::Type1), Just(TeenType::Type2)],
        care_mistakes in 0u8..=20,
        discipline_mistakes in 0u8..=20,
    ) {
        let result1 = tama_core::evolution::resolve_adult(&teen_char, teen_type, care_mistakes, discipline_mistakes);
        let result2 = tama_core::evolution::resolve_adult(&teen_char, teen_type, care_mistakes, discipline_mistakes);

        prop_assert_eq!(
            result1, result2,
            "resolve_adult must be deterministic for ({:?}, {:?}, care={}, disc={})",
            teen_char, teen_type, care_mistakes, discipline_mistakes
        );
    }
}


// ── Property 14: Evolution reset postconditions ─────────────────────────────
// **Validates: Requirement 4.6**

proptest! {
    #[test]
    fn prop_evolution_reset_postconditions(mut state in arb_valid_pet_state(), now in arb_datetime()) {
        // Only test stages that can evolve (Baby, Child, Teen, Adult)
        prop_assume!(state.is_alive);
        prop_assume!(matches!(
            state.stage,
            LifeStage::Baby | LifeStage::Child | LifeStage::Teen | LifeStage::Adult
        ));

        // Set up conditions so evolution will trigger
        match state.stage {
            LifeStage::Baby => {
                // Need 65 minutes elapsed in stage
                state.stage_start_time = now - chrono::Duration::minutes(70);
            }
            LifeStage::Child => {
                state.age = 3;
            }
            LifeStage::Teen => {
                state.age = 6;
                // Ensure teen_type is set
                if state.teen_type.is_none() {
                    state.teen_type = Some(TeenType::Type1);
                }
            }
            LifeStage::Adult => {
                // Maskutchi from T2 path, 4 days elapsed
                state.character = Character::Maskutchi;
                state.teen_type = Some(TeenType::Type2);
                state.stage_start_time = now - chrono::Duration::days(5);
            }
            _ => {}
        }

        // Set non-zero discipline and pending deadlines to verify they get reset
        state.discipline = 75;
        state.pending_care_deadline = Some(now);
        state.pending_discipline_deadline = Some(now);

        let evolved = tama_core::evolution::check_evolution(&mut state, now);
        prop_assert!(evolved, "evolution should have occurred for stage {:?}", state.stage);

        // Requirement 4.6: discipline reset to 0
        prop_assert_eq!(
            state.discipline, 0,
            "discipline should be 0 after evolution, got {}",
            state.discipline
        );

        // Requirement 4.6: pending deadlines cleared
        prop_assert!(
            state.pending_care_deadline.is_none(),
            "pending_care_deadline should be None after evolution, got {:?}",
            state.pending_care_deadline
        );
        prop_assert!(
            state.pending_discipline_deadline.is_none(),
            "pending_discipline_deadline should be None after evolution, got {:?}",
            state.pending_discipline_deadline
        );

        // stage_start_time should be updated to now
        prop_assert_eq!(
            state.stage_start_time, now,
            "stage_start_time should be updated to now after evolution"
        );
    }
}


// ── Property 15: Persistence round-trip ──────────────────────────────────────
// **Validates: Requirements 6.1, 6.2, 6.5**

proptest! {
    #[test]
    fn prop_persistence_round_trip(state in arb_valid_pet_state()) {
        // Serialize to JSON (Requirement 6.1)
        let json = serde_json::to_string(&state)
            .expect("PetState should serialize to JSON");

        // Deserialize from JSON (Requirement 6.2)
        let deserialized: PetState = serde_json::from_str(&json)
            .expect("PetState JSON should deserialize back");

        // Requirement 6.5: round-trip produces an equivalent PetState
        prop_assert_eq!(
            state, deserialized,
            "PetState should be equivalent after serialize → deserialize round-trip"
        );
    }
}


// ── Property 16: Catch-up convergence ────────────────────────────────────────
// **Validates: Requirements 6.3, 6.4**

proptest! {
    #[test]
    fn prop_catchup_convergence(state in arb_valid_pet_state(), elapsed_minutes in 0u64..=120) {
        // Use a well-defined base time for last_tick to avoid edge cases
        let base_time = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let mut state = state;
        state.last_tick = base_time;
        // Ensure birth_time and stage_start_time are not in the future relative to last_tick
        state.birth_time = base_time;
        state.stage_start_time = base_time;
        state.last_poop_time = base_time;

        let now = base_time + chrono::Duration::minutes(elapsed_minutes as i64);

        // Create a unique temp directory for this test run
        let dir = std::env::temp_dir().join(format!("tama96_prop16_{}", rand::random::<u64>()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");

        // Save the state
        tama_core::persistence::save(&state, &path).unwrap();

        // Load with catch-up
        let loaded = tama_core::persistence::load(&path, now).unwrap();

        // Requirement 6.3 & 6.4: After catch-up, last_tick must equal `now`
        prop_assert_eq!(
            loaded.last_tick, now,
            "After catch-up with {} elapsed minutes, last_tick should be {:?}, got {:?}",
            elapsed_minutes, now, loaded.last_tick
        );

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}


// ── Unit tests: Evolution paths ─────────────────────────────────────────────
// **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7**

#[cfg(test)]
mod evolution_tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    use tama_core::evolution::*;

    fn base_time() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap()
    }

    fn make_baby(now: DateTime<Utc>) -> PetState {
        let mut s = PetState::new_egg(now);
        s.stage = LifeStage::Baby;
        s.character = Character::Babytchi;
        s.stage_start_time = now;
        s.is_alive = true;
        s
    }

    fn make_child(now: DateTime<Utc>) -> PetState {
        let mut s = make_baby(now);
        s.stage = LifeStage::Child;
        s.character = Character::Marutchi;
        s.stage_start_time = now;
        s
    }

    fn make_teen(now: DateTime<Utc>, character: Character, teen_type: TeenType) -> PetState {
        let mut s = make_child(now);
        s.stage = LifeStage::Teen;
        s.character = character;
        s.teen_type = Some(teen_type);
        s.stage_start_time = now;
        s
    }

    fn make_adult(now: DateTime<Utc>, character: Character, teen_type: TeenType) -> PetState {
        let mut s = make_teen(now, Character::Tamatchi, teen_type);
        s.stage = LifeStage::Adult;
        s.character = character;
        s.stage_start_time = now;
        s
    }

    // ── Baby → Child (65 minutes) ──────────────────────────────────────

    #[test]
    fn baby_to_child_after_65_minutes() {
        let now = base_time();
        let mut state = make_baby(now);
        state.discipline = 50;
        state.pending_care_deadline = Some(now);
        state.pending_discipline_deadline = Some(now);

        let tick_time = now + Duration::minutes(65);
        let evolved = check_evolution(&mut state, tick_time);

        assert!(evolved);
        assert_eq!(state.stage, LifeStage::Child);
        assert_eq!(state.character, Character::Marutchi);
        assert_eq!(state.discipline, 0);
        assert!(state.pending_care_deadline.is_none());
        assert!(state.pending_discipline_deadline.is_none());
        assert_eq!(state.stage_start_time, tick_time);
    }

    #[test]
    fn baby_no_evolution_before_65_minutes() {
        let now = base_time();
        let mut state = make_baby(now);
        let tick_time = now + Duration::minutes(64);
        assert!(!check_evolution(&mut state, tick_time));
        assert_eq!(state.stage, LifeStage::Baby);
    }

    // ── Child → Teen (resolve_teen) ────────────────────────────────────

    #[test]
    fn child_to_tamatchi_type1() {
        let now = base_time();
        let mut state = make_child(now);
        state.age = 3;
        state.care_mistakes = 0;
        state.discipline_mistakes = 0;

        assert!(check_evolution(&mut state, now));
        assert_eq!(state.stage, LifeStage::Teen);
        assert_eq!(state.character, Character::Tamatchi);
        assert_eq!(state.teen_type, Some(TeenType::Type1));
    }

    #[test]
    fn child_to_tamatchi_type2() {
        let now = base_time();
        let mut state = make_child(now);
        state.age = 3;
        state.care_mistakes = 2; // <= 2 → Tamatchi
        state.discipline_mistakes = 3; // 3+ → Type2

        assert!(check_evolution(&mut state, now));
        assert_eq!(state.character, Character::Tamatchi);
        assert_eq!(state.teen_type, Some(TeenType::Type2));
    }

    #[test]
    fn child_to_kuchitamatchi_type1() {
        let now = base_time();
        let mut state = make_child(now);
        state.age = 3;
        state.care_mistakes = 3; // 3+ → Kuchitamatchi
        state.discipline_mistakes = 2; // <= 2 → Type1

        assert!(check_evolution(&mut state, now));
        assert_eq!(state.character, Character::Kuchitamatchi);
        assert_eq!(state.teen_type, Some(TeenType::Type1));
    }

    #[test]
    fn child_to_kuchitamatchi_type2() {
        let now = base_time();
        let mut state = make_child(now);
        state.age = 3;
        state.care_mistakes = 5;
        state.discipline_mistakes = 4;

        assert!(check_evolution(&mut state, now));
        assert_eq!(state.character, Character::Kuchitamatchi);
        assert_eq!(state.teen_type, Some(TeenType::Type2));
    }

    #[test]
    fn child_no_evolution_before_age_3() {
        let now = base_time();
        let mut state = make_child(now);
        state.age = 2;
        assert!(!check_evolution(&mut state, now));
        assert_eq!(state.stage, LifeStage::Child);
    }

    // ── resolve_teen unit tests ────────────────────────────────────────

    #[test]
    fn resolve_teen_boundary_care_2() {
        let (c, _) = resolve_teen(2, 0);
        assert_eq!(c, Character::Tamatchi);
    }

    #[test]
    fn resolve_teen_boundary_care_3() {
        let (c, _) = resolve_teen(3, 0);
        assert_eq!(c, Character::Kuchitamatchi);
    }

    #[test]
    fn resolve_teen_boundary_disc_2() {
        let (_, t) = resolve_teen(0, 2);
        assert_eq!(t, TeenType::Type1);
    }

    #[test]
    fn resolve_teen_boundary_disc_3() {
        let (_, t) = resolve_teen(0, 3);
        assert_eq!(t, TeenType::Type2);
    }

    // ── Teen → Adult (resolve_adult full P1 matrix) ────────────────────

    // Tamatchi T1 paths
    #[test]
    fn tamatchi_t1_low_care_0_disc_mametchi() {
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 0, 0),
            Character::Mametchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 2, 0),
            Character::Mametchi
        );
    }

    #[test]
    fn tamatchi_t1_low_care_1_disc_ginjirotchi() {
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 0, 1),
            Character::Ginjirotchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 2, 1),
            Character::Ginjirotchi
        );
    }

    #[test]
    fn tamatchi_t1_low_care_2plus_disc_maskutchi() {
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 0, 2),
            Character::Maskutchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 2, 5),
            Character::Maskutchi
        );
    }

    #[test]
    fn tamatchi_t1_high_care_0_1_disc_kuchipatchi() {
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 3, 0),
            Character::Kuchipatchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 5, 1),
            Character::Kuchipatchi
        );
    }

    #[test]
    fn tamatchi_t1_high_care_2_3_disc_nyorotchi() {
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 3, 2),
            Character::Nyorotchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 5, 3),
            Character::Nyorotchi
        );
    }

    #[test]
    fn tamatchi_t1_high_care_4plus_disc_tarakotchi() {
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 3, 4),
            Character::Tarakotchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type1, 10, 10),
            Character::Tarakotchi
        );
    }

    // Tamatchi T2 paths
    #[test]
    fn tamatchi_t2_low_care_2plus_disc_maskutchi() {
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 0, 2),
            Character::Maskutchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 3, 2),
            Character::Maskutchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 3, 5),
            Character::Maskutchi
        );
    }

    #[test]
    fn tamatchi_t2_high_care_0_1_disc_kuchipatchi() {
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 4, 0),
            Character::Kuchipatchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 3, 1),
            Character::Kuchipatchi
        );
    }

    #[test]
    fn tamatchi_t2_high_care_2_3_disc_nyorotchi() {
        // care >= 3 and disc 2-3 but care_mistakes <= 3 && disc >= 2 → Maskutchi takes priority
        // So we need care > 3 to avoid the Maskutchi branch
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 4, 2),
            Character::Nyorotchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 4, 3),
            Character::Nyorotchi
        );
    }

    #[test]
    fn tamatchi_t2_high_care_4plus_disc_tarakotchi() {
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 4, 4),
            Character::Tarakotchi
        );
    }

    #[test]
    fn tamatchi_t2_fallback_nyorotchi() {
        // Low care (<=3), low disc (<2) → fallback
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 0, 0),
            Character::Nyorotchi
        );
        assert_eq!(
            resolve_adult(&Character::Tamatchi, TeenType::Type2, 2, 1),
            Character::Nyorotchi
        );
    }

    // Kuchitamatchi T1 paths
    #[test]
    fn kuchitamatchi_t1_0_1_disc_kuchipatchi() {
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 0, 0),
            Character::Kuchipatchi
        );
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 5, 1),
            Character::Kuchipatchi
        );
    }

    #[test]
    fn kuchitamatchi_t1_2_3_disc_nyorotchi() {
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 0, 2),
            Character::Nyorotchi
        );
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 5, 3),
            Character::Nyorotchi
        );
    }

    #[test]
    fn kuchitamatchi_t1_4plus_disc_tarakotchi() {
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 0, 4),
            Character::Tarakotchi
        );
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 10, 10),
            Character::Tarakotchi
        );
    }

    // Kuchitamatchi T2 paths
    #[test]
    fn kuchitamatchi_t2_0_1_disc_kuchipatchi() {
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 0, 0),
            Character::Kuchipatchi
        );
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 5, 1),
            Character::Kuchipatchi
        );
    }

    #[test]
    fn kuchitamatchi_t2_2_3_disc_nyorotchi() {
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 0, 2),
            Character::Nyorotchi
        );
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 5, 3),
            Character::Nyorotchi
        );
    }

    #[test]
    fn kuchitamatchi_t2_4plus_disc_tarakotchi() {
        assert_eq!(
            resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 0, 4),
            Character::Tarakotchi
        );
    }

    // Safety fallback
    #[test]
    fn safety_fallback_nyorotchi() {
        // Non-teen character should hit safety fallback
        assert_eq!(
            resolve_adult(&Character::Mametchi, TeenType::Type1, 0, 0),
            Character::Nyorotchi
        );
    }

    // ── Teen → Adult via check_evolution ────────────────────────────────

    #[test]
    fn teen_to_adult_at_age_6() {
        let now = base_time();
        let mut state = make_teen(now, Character::Tamatchi, TeenType::Type1);
        state.age = 6;
        state.care_mistakes = 0;
        state.discipline_mistakes = 0;
        state.discipline = 75;

        assert!(check_evolution(&mut state, now));
        assert_eq!(state.stage, LifeStage::Adult);
        assert_eq!(state.character, Character::Mametchi);
        assert_eq!(state.discipline, 0);
    }

    #[test]
    fn teen_no_evolution_before_age_6() {
        let now = base_time();
        let mut state = make_teen(now, Character::Tamatchi, TeenType::Type1);
        state.age = 5;
        assert!(!check_evolution(&mut state, now));
        assert_eq!(state.stage, LifeStage::Teen);
    }

    // ── Adult → Special (Maskutchi T2 → Oyajitchi) ─────────────────────

    #[test]
    fn maskutchi_t2_to_oyajitchi_after_4_days() {
        let now = base_time();
        let mut state = make_adult(now, Character::Maskutchi, TeenType::Type2);
        state.discipline = 50;

        let tick_time = now + Duration::days(4);
        assert!(check_evolution(&mut state, tick_time));
        assert_eq!(state.stage, LifeStage::Special);
        assert_eq!(state.character, Character::Oyajitchi);
        assert_eq!(state.discipline, 0);
    }

    #[test]
    fn maskutchi_t2_no_evolution_before_4_days() {
        let now = base_time();
        let mut state = make_adult(now, Character::Maskutchi, TeenType::Type2);
        let tick_time = now + Duration::days(4) - Duration::minutes(1);
        assert!(!check_evolution(&mut state, tick_time));
        assert_eq!(state.stage, LifeStage::Adult);
    }

    #[test]
    fn maskutchi_t1_no_special_evolution() {
        // Maskutchi from T1 path should NOT evolve to Oyajitchi
        let now = base_time();
        let mut state = make_adult(now, Character::Maskutchi, TeenType::Type1);
        let tick_time = now + Duration::days(10);
        assert!(!check_evolution(&mut state, tick_time));
        assert_eq!(state.stage, LifeStage::Adult);
    }

    #[test]
    fn non_maskutchi_adult_no_special_evolution() {
        let now = base_time();
        let mut state = make_adult(now, Character::Mametchi, TeenType::Type1);
        let tick_time = now + Duration::days(10);
        assert!(!check_evolution(&mut state, tick_time));
        assert_eq!(state.stage, LifeStage::Adult);
    }

    // ── Egg stage is not handled by check_evolution ────────────────────

    #[test]
    fn egg_not_handled_by_check_evolution() {
        let now = base_time();
        let mut state = PetState::new_egg(now);
        let tick_time = now + Duration::minutes(10);
        assert!(!check_evolution(&mut state, tick_time));
        assert_eq!(state.stage, LifeStage::Egg);
    }

    // ── Exhaustive resolve_adult matrix ─────────────────────────────────

    #[test]
    fn exhaustive_p1_branching_matrix() {
        // Tamatchi T1
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 0, 0), Character::Mametchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 1, 0), Character::Mametchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 2, 0), Character::Mametchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 0, 1), Character::Ginjirotchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 1, 1), Character::Ginjirotchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 2, 1), Character::Ginjirotchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 0, 2), Character::Maskutchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 0, 5), Character::Maskutchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 3, 0), Character::Kuchipatchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 3, 1), Character::Kuchipatchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 3, 2), Character::Nyorotchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 3, 3), Character::Nyorotchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 3, 4), Character::Tarakotchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type1, 3, 10), Character::Tarakotchi);

        // Tamatchi T2
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 0, 2), Character::Maskutchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 3, 2), Character::Maskutchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 3, 5), Character::Maskutchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 4, 0), Character::Kuchipatchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 4, 1), Character::Kuchipatchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 4, 2), Character::Nyorotchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 4, 3), Character::Nyorotchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 4, 4), Character::Tarakotchi);
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 0, 0), Character::Nyorotchi); // fallback
        assert_eq!(resolve_adult(&Character::Tamatchi, TeenType::Type2, 2, 1), Character::Nyorotchi); // fallback

        // Kuchitamatchi T1
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 0, 0), Character::Kuchipatchi);
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 0, 1), Character::Kuchipatchi);
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 0, 2), Character::Nyorotchi);
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 0, 3), Character::Nyorotchi);
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 0, 4), Character::Tarakotchi);
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type1, 0, 10), Character::Tarakotchi);

        // Kuchitamatchi T2
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 0, 0), Character::Kuchipatchi);
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 0, 1), Character::Kuchipatchi);
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 0, 2), Character::Nyorotchi);
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 0, 3), Character::Nyorotchi);
        assert_eq!(resolve_adult(&Character::Kuchitamatchi, TeenType::Type2, 0, 4), Character::Tarakotchi);
    }
}


// ── Property 17: Lockfile mutual exclusion ──────────────────────────────────
// **Validates: Requirement 7.4**

proptest! {
    /// For any sequence of acquire/release cycles, at most one process holds the lock at any time.
    /// A second acquire while the first guard is alive must fail with AlreadyLocked.
    /// After release (drop), a new acquire must succeed.
    #[test]
    fn prop_lockfile_mutual_exclusion(cycles in 1u8..=10) {
        let dir = std::env::temp_dir().join(format!("tama96_prop17_{}", rand::random::<u64>()));
        std::fs::create_dir_all(&dir).unwrap();
        let lock_path = dir.join("tama96.lock");

        for _ in 0..cycles {
            // Acquire the lock — must succeed since no one holds it
            let guard = tama_core::persistence::acquire_lock(&lock_path)
                .expect("acquire_lock should succeed when no lock is held");

            // Lock file should exist and contain our PID
            prop_assert!(lock_path.exists(), "lock file should exist after acquire");
            let contents = std::fs::read_to_string(&lock_path).unwrap();
            let pid: u32 = contents.trim().parse().unwrap();
            prop_assert_eq!(pid, std::process::id(), "lock file should contain our PID");

            // A second acquire while the first guard is alive must fail with AlreadyLocked
            let second = tama_core::persistence::acquire_lock(&lock_path);
            prop_assert!(second.is_err(), "second acquire should fail while lock is held");
            match second {
                Err(tama_core::persistence::LockError::AlreadyLocked(msg)) => {
                    prop_assert!(
                        msg.contains("Another instance"),
                        "AlreadyLocked message should mention 'Another instance', got: {}",
                        msg
                    );
                }
                Err(other) => {
                    prop_assert!(false, "expected AlreadyLocked, got: {:?}", other);
                }
                Ok(_) => {
                    prop_assert!(false, "second acquire should not succeed while lock is held");
                }
            }

            // Release the lock (drop the guard)
            tama_core::persistence::release_lock(guard);

            // Lock file should be gone after release
            prop_assert!(!lock_path.exists(), "lock file should be removed after release");
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}


// ── ActionType generator ────────────────────────────────────────────────────

fn arb_action_type() -> impl Strategy<Value = ActionType> {
    prop_oneof![
        Just(ActionType::FeedMeal),
        Just(ActionType::FeedSnack),
        Just(ActionType::PlayGame),
        Just(ActionType::Discipline),
        Just(ActionType::GiveMedicine),
        Just(ActionType::CleanPoop),
        Just(ActionType::ToggleLights),
        Just(ActionType::GetStatus),
    ]
}


// ── Property 18: Permission gating ──────────────────────────────────────────
// **Validates: Requirements 8.1, 8.2**

proptest! {
    /// Master switch disabled returns MasterDisabled for any action type.
    #[test]
    fn prop_permission_master_disabled(action in arb_action_type()) {
        let mut perms = AgentPermissions::default();
        perms.enabled = false;
        let now = Utc::now();

        let result = tama_core::permissions::check_permission(&mut perms, &action, now);
        prop_assert_eq!(
            result,
            Err(tama_core::permissions::PermissionDenied::MasterDisabled),
            "With master switch disabled, check_permission should return MasterDisabled for {:?}",
            action
        );
    }

    /// Action disabled returns ActionDisabled with the action name.
    #[test]
    fn prop_permission_action_disabled(action in arb_action_type()) {
        let mut perms = AgentPermissions::default();
        // Master switch is on, but disable the specific action
        perms.enabled = true;
        perms.allowed_actions.get_mut(&action).unwrap().allowed = false;
        let now = Utc::now();

        let result = tama_core::permissions::check_permission(&mut perms, &action, now);
        prop_assert_eq!(
            result,
            Err(tama_core::permissions::PermissionDenied::ActionDisabled(action.clone())),
            "With action {:?} disabled, check_permission should return ActionDisabled",
            action
        );
    }
}


// ── Property 19: Rate limiting ──────────────────────────────────────────────
// **Validates: Requirement 8.3**

proptest! {
    /// If action_log contains n >= max_per_hour entries within the last hour,
    /// check_permission returns RateLimited with correct limit and used values.
    /// With fewer than limit entries, check_permission succeeds.
    #[test]
    fn prop_rate_limiting(
        action in arb_action_type(),
        limit in 1u32..=10,
    ) {
        let now = Utc::now();

        // ── Case 1: Exactly `limit` entries → RateLimited ──────────────

        let mut perms = AgentPermissions::default();
        perms.enabled = true;
        perms.allowed_actions.insert(
            action.clone(),
            tama_core::state::ActionPermission {
                allowed: true,
                max_per_hour: Some(limit),
            },
        );

        // Log exactly `limit` actions within the last hour
        for i in 0..limit {
            tama_core::permissions::log_action(
                &mut perms,
                action.clone(),
                now - chrono::Duration::minutes(i as i64),
            );
        }

        let result = tama_core::permissions::check_permission(&mut perms, &action, now);
        prop_assert_eq!(
            result,
            Err(tama_core::permissions::PermissionDenied::RateLimited {
                action: action.clone(),
                limit,
                used: limit,
            }),
            "With {} entries (limit={}), check_permission should return RateLimited for {:?}",
            limit, limit, action
        );

        // ── Case 2: Fewer than `limit` entries → Ok ────────────────────

        let mut perms2 = AgentPermissions::default();
        perms2.enabled = true;
        perms2.allowed_actions.insert(
            action.clone(),
            tama_core::state::ActionPermission {
                allowed: true,
                max_per_hour: Some(limit),
            },
        );

        // Log (limit - 1) actions within the last hour
        for i in 0..(limit.saturating_sub(1)) {
            tama_core::permissions::log_action(
                &mut perms2,
                action.clone(),
                now - chrono::Duration::minutes(i as i64),
            );
        }

        let result2 = tama_core::permissions::check_permission(&mut perms2, &action, now);
        prop_assert!(
            result2.is_ok(),
            "With {} entries (limit={}), check_permission should succeed for {:?}, got {:?}",
            limit.saturating_sub(1), limit, action, result2
        );
    }
}
