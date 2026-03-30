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
    fn prop_feed_meal_correctness(mut state in arb_valid_pet_state()) {
        // Only test on alive, awake, non-sick pets (feed_meal preconditions)
        prop_assume!(state.is_alive && !state.is_sleeping && !state.is_sick);

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
    fn prop_feed_snack_correctness(mut state in arb_valid_pet_state()) {
        // Only test on alive, awake pets (feed_snack preconditions — no sick check needed)
        prop_assume!(state.is_alive && !state.is_sleeping);

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
        mut state in arb_valid_pet_state(),
        m0 in arb_choice(),
        m1 in arb_choice(),
        m2 in arb_choice(),
        m3 in arb_choice(),
        m4 in arb_choice(),
    ) {
        // Filter to alive, awake, non-sick pets (play_game preconditions)
        prop_assume!(state.is_alive && !state.is_sleeping && !state.is_sick);

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
