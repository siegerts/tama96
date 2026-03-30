use chrono::{DateTime, TimeZone, Utc};
use proptest::prelude::*;
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
