# Implementation Plan: tama96

## Overview

Implement a faithful 1996 Tamagotchi P1 virtual pet as a Cargo workspace with four crates: `tama-core` (shared Rust library), `tama-tauri` (Tauri v2 desktop app with React frontend), `tama-tui` (ratatui terminal app), and `mcp-server` (Node.js sidecar). Tasks proceed bottom-up: core engine first, then persistence, then frontends, then MCP integration, then wiring and polish.

## Tasks

- [x] 1. Initialize Cargo workspace and project scaffolding
  - [x] 1.1 Create Cargo workspace with members: `tama-core` (lib), `tama-tauri` (bin), `tama-tui` (bin)
    - Create root `Cargo.toml` with workspace members
    - `cargo init --lib tama-core`, `cargo init tama-tauri`, `cargo init tama-tui`
    - Add shared dependencies to workspace `[dependencies]`: `serde`, `serde_json`, `chrono`, `rand`
    - _Requirements: None (scaffolding)_

  - [x] 1.2 Scaffold Tauri v2 React frontend in `tama-tauri`
    - Run `npm create tauri-app` or manually set up Vite + React + `@tauri-apps/api` inside `tama-tauri/ui`
    - Configure `tauri.conf.json` with app name "tama96", window title, default size
    - _Requirements: None (scaffolding)_

  - [x] 1.3 Scaffold MCP server Node.js project in `mcp-server/`
    - `npm init`, add `@modelcontextprotocol/sdk` and `zod` dependencies
    - Create `tsconfig.json` for TypeScript compilation
    - Create entry point `src/index.ts`
    - _Requirements: None (scaffolding)_

- [x] 2. Implement tama-core data models and enums
  - [x] 2.1 Define core enums and PetState struct in `tama-core/src/state.rs`
    - Implement `LifeStage`, `Character`, `TeenType`, `ActionType` enums with Serialize/Deserialize
    - Implement `PetState` struct with all fields from design (hunger, happiness, discipline, weight, age, care_mistakes, discipline_mistakes, poop_count, is_sick, sick_dose_count, is_sleeping, is_alive, lights_on, timestamps, deadlines, snack_count_since_last_tick)
    - Implement `PetState::new_egg(now)` constructor
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7_

  - [x] 2.2 Define AgentPermissions structs in `tama-core/src/state.rs`
    - Implement `ActionPermission`, `ActionLogEntry`, `AgentPermissions` structs
    - Implement `AgentPermissions::default()` with all actions allowed and no rate limits
    - _Requirements: 8.1, 8.2, 8.3, 8.5_

  - [x] 2.3 Define CharacterStats and constants table in `tama-core/src/characters.rs`
    - Implement `CharacterStats` struct
    - Implement `CharacterStats::for_character()` with all 11 character stat blocks from design
    - _Requirements: 2.1, 2.2, 2.5_

  - [x] 2.4 Define ActionResult, ActionError, and GameResult types in `tama-core/src/actions.rs`
    - `ActionResult` enum: Fed, Snacked, Disciplined, MedicineGiven, Cleaned, LightsToggled
    - `ActionError` enum: PetIsDead, PetIsSleeping, PetIsNotSick, NoDisciplineCallPending, NoPoop
    - `GameResult` struct: rounds, wins, happiness_gained
    - _Requirements: 3.7, 3.12, 3.13_

  - [x] 2.5 Write property test: State invariants (meter bounds)
    - **Property 1: State invariants (meter bounds)**
    - Generate arbitrary PetState values and apply random sequences of actions/ticks; assert hunger in [0,4], happiness in [0,4], discipline in [0,100] and multiple of 25, weight >= 1, poop_count in [0,4]
    - Use `proptest` crate
    - **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**

  - [x] 2.6 Write property test: Stage-character consistency
    - **Property 2: Stage-character consistency**
    - For any PetState produced by actions/ticks, assert Character variant is valid for current LifeStage and teen_type is Some only when stage is Teen, Adult, or Special
    - **Validates: Requirements 1.6, 1.7**

- [x] 3. Implement player actions in tama-core
  - [x] 3.1 Implement `feed_meal` and `feed_snack` in `tama-core/src/actions.rs`
    - `feed_meal`: check preconditions (alive, awake, not sick), increment hunger (cap 4), increment weight
    - `feed_snack`: check preconditions (alive, awake), increment happiness (cap 4), add weight +2, increment snack counter, trigger sickness if baby + snack_count > 3
    - _Requirements: 3.1, 3.2, 3.3, 3.11_

  - [x] 3.2 Write property test: Feed meal correctness
    - **Property 7: Feed meal correctness**
    - For any alive, awake, non-sick Pet, feeding a meal sets hunger to min(old+1, 4) and weight to old+1
    - **Validates: Requirements 3.1, 3.11**

  - [x] 3.3 Write property test: Feed snack correctness
    - **Property 8: Feed snack correctness**
    - For any alive, awake Pet, feeding a snack sets happiness to min(old+1, 4) and weight to old+2
    - **Validates: Requirement 3.2**

  - [x] 3.4 Implement `play_game` in `tama-core/src/actions.rs`
    - Check preconditions (alive, awake, not sick), generate 5 random pet choices, compare with player moves, if wins >= 3 increment happiness (cap 4), always decrement weight (floor 1)
    - _Requirements: 3.4, 3.5_

  - [x] 3.5 Write property test: Game outcome correctness
    - **Property 9: Game outcome correctness**
    - For any game with 5 moves, wins >= 3 means happiness +1 (cap 4), weight always -1 (floor 1)
    - **Validates: Requirements 3.4, 3.5**

  - [x] 3.6 Implement `discipline` in `tama-core/src/actions.rs`
    - Check pending_discipline_deadline exists, increment discipline by 25 (cap 100), clear deadline
    - Return NoDisciplineCallPending error if no pending call
    - _Requirements: 3.6, 3.7_

  - [x] 3.7 Write property test: Discipline action correctness
    - **Property 10: Discipline action correctness**
    - With pending call: discipline += 25 (cap 100), deadline cleared. Without: NoDisciplineCallPending error
    - **Validates: Requirements 3.6, 3.7**

  - [x] 3.8 Implement `give_medicine`, `clean_poop`, `toggle_lights` in `tama-core/src/actions.rs`
    - `give_medicine`: check is_sick, increment sick_dose_count, cure at 2 doses
    - `clean_poop`: check poop_count > 0, decrement by 1
    - `toggle_lights`: flip lights_on, if turning off during sleep time set is_sleeping = true, clear pending_lights_deadline
    - _Requirements: 3.8, 3.9, 3.10_

  - [x] 3.9 Write property test: Medicine curing
    - **Property 11: Medicine curing**
    - For any sick Pet, two doses of medicine set is_sick to false and sick_dose_count to 0
    - **Validates: Requirement 3.8**

  - [x] 3.10 Write property test: Action precondition enforcement
    - **Property 12: Action precondition enforcement**
    - Dead pet + any action returns PetIsDead error. Sleeping pet + feed_meal/play_game returns error
    - **Validates: Requirements 3.12, 3.13**

- [x] 4. Checkpoint - Core actions
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Implement tick engine in tama-core
  - [x] 5.1 Implement heart decay in `tama-core/src/engine.rs`
    - `decay_hearts(state, stats, elapsed_minutes)`: decrement hunger/happiness by floor(elapsed / decay_rate), clamp to 0
    - Start 15-minute care deadline when a meter hits 0 (if no deadline pending)
    - _Requirements: 2.1, 2.2, 2.3_

  - [x] 5.2 Write property test: Heart decay correctness
    - **Property 3: Heart decay correctness**
    - For any awake, alive Pet with known Character and elapsed time, hunger decrements by floor(elapsed / hunger_decay_minutes) and happiness by floor(elapsed / happy_decay_minutes), each clamped to 0
    - **Validates: Requirements 2.1, 2.2**

  - [x] 5.3 Write property test: Care deadline lifecycle
    - **Property 4: Care deadline lifecycle**
    - When hunger or happiness reaches 0, a 15-minute deadline is created if none exists. When deadline expires, care_mistakes increments by 1
    - **Validates: Requirements 2.3, 2.4**

  - [x] 5.4 Implement poop, sickness, discipline call, and sleep checks in `tama-core/src/engine.rs`
    - `check_poop`: accumulate poop based on character poop_interval_minutes
    - `check_sickness`: trigger sickness from poop threshold or snack overfeeding
    - `maybe_generate_discipline_call`: random false attention calls with 15-minute deadline
    - `check_discipline_deadline`: increment discipline_mistakes if deadline expired
    - `check_sleep` / `check_wake`: manage sleep/wake transitions based on character sleep/wake hours, age increment on wake
    - _Requirements: 2.5, 2.6_

  - [x] 5.5 Write property test: Sleep immunity
    - **Property 5: Sleep immunity**
    - For any sleeping Pet, a tick shall not change hunger or happiness values
    - **Validates: Requirement 2.6**

  - [x] 5.6 Implement death checks in `tama-core/src/engine.rs`
    - `check_death`: old age (age >= max_lifespan_days), neglect (hunger + happiness == 0 for 12h), untreated sickness (24h), baby snack overfeeding (snack_count > 5)
    - `kill(state)`: set is_alive = false, stage = Dead
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

  - [x] 5.7 Write property test: Dead pets don't tick
    - **Property 6: Dead pets don't tick**
    - For any dead Pet, a tick leaves the entire PetState unchanged
    - **Validates: Requirement 2.7**

  - [x] 5.8 Write property test: Death from old age
    - **Property 20: Death from old age**
    - For any Character with max_lifespan_days > 0, when age reaches that value, is_alive becomes false and stage becomes Dead
    - **Validates: Requirement 5.1**

  - [x] 5.9 Implement main `tick()` function in `tama-core/src/engine.rs`
    - Orchestrate: egg hatch check, sleep skip, decay, care deadlines, poop, sickness, discipline, sleep, death, evolution, update last_tick
    - Handle egg stage separately (only check hatch timer)
    - _Requirements: 2.1-2.8_

- [ ] 6. Implement evolution system in tama-core
  - [ ] 6.1 Implement evolution logic in `tama-core/src/evolution.rs`
    - `check_evolution(state, now)`: Egg to Baby (5min), Baby to Child (65min), Child to Teen (age 3), Teen to Adult (age 6), Adult to Special (Maskutchi T2 path, 4 days)
    - `resolve_teen(care_mistakes, discipline_mistakes)`: branch on care_mistakes <= 2 gives Tamatchi, else Kuchitamatchi; discipline_mistakes <= 2 gives Type1, else Type2
    - `resolve_adult(teen_char, teen_type, care_mistakes, discipline_mistakes)`: full P1 branching matrix from design
    - `evolve_to(state, stage, character, now)`: reset discipline, clear deadlines, update stage_start_time
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

  - [ ] 6.2 Write property test: Evolution determinism
    - **Property 13: Evolution determinism**
    - For any two calls to resolve_adult with identical (teen_character, teen_type, care_mistakes, discipline_mistakes), the resulting adult Character is identical
    - **Validates: Requirements 4.3, 4.4, 4.7**

  - [ ] 6.3 Write property test: Evolution reset postconditions
    - **Property 14: Evolution reset postconditions**
    - For any evolution event, discipline is reset to 0 and pending care/discipline deadlines are cleared
    - **Validates: Requirement 4.6**

  - [ ] 6.4 Write unit tests for all evolution paths
    - Test each path: Egg to Baby, Baby to Child, Child to Tamatchi, Child to Kuchitamatchi, Teen to each of 6 adults, Maskutchi to Oyajitchi
    - Verify the full P1 branching matrix exhaustively
    - _Requirements: 4.1-4.7_

- [ ] 7. Implement persistence and lockfile in tama-core
  - [ ] 7.1 Implement save/load with atomic writes in `tama-core/src/persistence.rs`
    - `save(state, path)`: serialize to JSON, write to temp file, rename to target (atomic)
    - `load(path, now)`: deserialize JSON, apply catch-up ticks from last_tick to now
    - Handle corrupt file: back up as `.corrupt`, log warning, return fresh egg
    - Handle clock skew: if now < last_tick, skip tick, log warning, set last_tick = now
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.6, 6.7_

  - [ ] 7.2 Write property test: Persistence round-trip
    - **Property 15: Persistence round-trip**
    - For any valid PetState, serialize to JSON then deserialize produces an equivalent PetState
    - **Validates: Requirements 6.1, 6.2, 6.5**

  - [ ] 7.3 Write property test: Catch-up convergence
    - **Property 16: Catch-up convergence**
    - For any saved PetState and elapsed time, loading with catch-up results in last_tick equal to current timestamp
    - **Validates: Requirements 6.3, 6.4**

  - [ ] 7.4 Implement lockfile in `tama-core/src/persistence.rs`
    - `acquire_lock(path)`: create lock file with PID, fail if already held by live process
    - `release_lock(guard)`: remove lock file
    - Implement `LockGuard` with Drop trait for automatic cleanup
    - _Requirements: 7.1, 7.2, 7.3, 7.4_

  - [ ] 7.5 Write property test: Lockfile mutual exclusion
    - **Property 17: Lockfile mutual exclusion**
    - For any sequence of acquire/release operations, at most one process holds the lock at any time
    - **Validates: Requirement 7.4**

- [ ] 8. Implement agent permission system in tama-core
  - [ ] 8.1 Implement permission checking in `tama-core/src/permissions.rs`
    - `check_permission(permissions, action, now)`: check master switch, check action allowed, check rate limit
    - Prune action_log entries older than 1 hour during each check
    - `log_action(permissions, action, now)`: append to action_log
    - `save_permissions` / `load_permissions`: persist to `~/.tama96/permissions.json`
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

  - [ ] 8.2 Write property test: Permission gating
    - **Property 18: Permission gating**
    - Master switch disabled returns MasterDisabled. Action disabled returns PermissionDenied with action name
    - **Validates: Requirements 8.1, 8.2**

  - [ ] 8.3 Write property test: Rate limiting
    - **Property 19: Rate limiting**
    - If action_log contains n >= max_per_hour entries within last hour, next check returns RateLimited
    - **Validates: Requirement 8.3**

- [ ] 9. Checkpoint - Core engine complete
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Implement tama-tauri backend
  - [ ] 10.1 Set up Tauri IPC commands in `tama-tauri/src-tauri/src/commands.rs`
    - Expose tama-core actions as Tauri commands: `get_state`, `feed_meal`, `feed_snack`, `play_game`, `discipline`, `give_medicine`, `clean_poop`, `toggle_lights`
    - Each command acquires shared state via Tauri managed state (Arc Mutex PetState), calls tama-core, saves, returns snapshot
    - Add `hatch_new_egg` command for restarting after death
    - Add `get_permissions` and `update_permissions` commands for agent permission management
    - _Requirements: 3.1-3.13, 8.5, 8.6_

  - [ ] 10.2 Implement background tick loop and system tray in `tama-tauri/src-tauri/src/main.rs`
    - Spawn tokio task that calls `engine::tick()` every 60 seconds on the shared state
    - Configure system tray with icon, tooltip showing pet name/stage
    - Tray menu: Show Window, Pet Status summary, Quit
    - On window close: hide window instead of exiting (keep tray alive)
    - On tray double-click: restore and focus window
    - _Requirements: 10.1, 10.2, 10.3, 10.4_

  - [ ] 10.3 Implement desktop notifications in tama-tauri
    - Use `tauri-plugin-notification` to send alerts when hunger or happiness reaches 0
    - Send notification on evolution events and death
    - _Requirements: 10.5_

  - [ ] 10.4 Implement TCP socket server for MCP bridge in `tama-tauri/src-tauri/src/socket.rs`
    - Bind to `127.0.0.1` with random available port
    - Accept JSON requests from MCP sidecar, check permissions via `permissions::check_permission`, execute action, return JSON response
    - Write port number to `~/.tama96/mcp_port` for sidecar discovery
    - _Requirements: 9.3, 9.4, 9.5, 9.6_

  - [ ] 10.5 Implement MCP sidecar lifecycle management in tama-tauri
    - On app start: spawn `mcp-server` as Tauri sidecar process
    - Monitor sidecar health; on unexpected exit, restart with exponential backoff (2s, 4s, 8s, max 30s)
    - On app quit: terminate sidecar
    - _Requirements: 13.1, 13.2_

  - [ ] 10.6 Wire up lockfile acquisition on startup in tama-tauri
    - Acquire lockfile at `~/.tama96/tama96.lock` on launch
    - If locked, show error dialog and exit
    - Release lockfile on quit via Drop guard
    - _Requirements: 7.1, 7.2, 7.3_

- [ ] 11. Implement React frontend for tama-tauri
  - [ ] 11.1 Create `usePetState` hook and state polling in `tama-tauri/ui/src/hooks/usePetState.ts`
    - Invoke `get_state` via Tauri IPC, poll every 1 second
    - Expose action functions: feedMeal, feedSnack, playGame, discipline, giveMedicine, cleanPoop, toggleLights
    - _Requirements: 11.1, 11.2, 11.3_

  - [ ] 11.2 Build main pet display component
    - Render pet sprite area with animation states (idle, eating, sleeping, happy, sick, dead)
    - Display hunger hearts (0-4), happiness hearts (0-4), discipline gauge (0-100), age, weight
    - Show poop indicators and sickness icon
    - _Requirements: 11.1, 11.2_

  - [ ] 11.3 Build action button bar matching P1 icon layout
    - Buttons: Feed (meal/snack submenu), Light, Game, Medicine, Bathroom, Meter, Discipline, Attention
    - Disable buttons based on state (e.g., no feed when dead, no medicine when not sick)
    - _Requirements: 11.3_

  - [ ] 11.4 Build agent permissions settings panel
    - Toggle master switch, per-action allow/deny toggles, rate limit inputs
    - Invoke `get_permissions` / `update_permissions` Tauri commands
    - _Requirements: 11.4, 8.5_

  - [ ] 11.5 Build death screen and hatch-new-egg flow
    - Display death screen with final age and character
    - "Hatch New Egg" button invokes `hatch_new_egg` command
    - _Requirements: 11.5_

- [ ] 12. Checkpoint - Desktop app functional
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 13. Implement tama-tui terminal app
  - [ ] 13.1 Set up ratatui app skeleton in `tama-tui/src/main.rs`
    - Initialize crossterm backend, acquire lockfile, load state with catch-up
    - Set up main event loop: keyboard input + 60-second tick timer
    - On exit: save state, release lockfile, restore terminal
    - _Requirements: 12.4, 12.5, 12.6_

  - [ ] 13.2 Implement braille sprite rendering in `tama-tui/src/sprites.rs`
    - Define 32x16 pixel grids for each Character and animation state
    - Convert pixel grids to braille characters (2x4 pixel blocks per braille cell)
    - Render into ratatui canvas widget
    - _Requirements: 12.1_

  - [ ] 13.3 Build TUI layout with meters and status display
    - Hunger hearts row (unicode filled/empty hearts)
    - Happiness hearts row
    - Discipline gauge bar
    - Age and weight display
    - Poop indicators, sickness icon, sleep indicator
    - _Requirements: 12.2_

  - [ ] 13.4 Implement keyboard input handling
    - Map keys: f=feed (then m/s for meal/snack), g=game, d=discipline, c=clean, l=lights, i=medicine, q=quit
    - Dispatch to tama-core actions, save state after each action
    - _Requirements: 12.3_

- [ ] 14. Implement MCP server (Node.js sidecar)
  - [ ] 14.1 Implement TCP bridge client in `mcp-server/src/bridge.ts`
    - Read port from `~/.tama96/mcp_port`
    - Connect to Tauri TCP socket, send JSON requests, receive JSON responses
    - Handle connection errors with retry logic
    - _Requirements: 9.6, 13.4_

  - [ ] 14.2 Register MCP tools in `mcp-server/src/tools.ts`
    - Tools: `feed` (type: meal|snack), `play_game` (moves array), `discipline`, `give_medicine`, `clean_poop`, `toggle_lights`, `get_status`
    - Each tool calls bridge, returns state snapshot or structured error
    - _Requirements: 9.1, 9.3, 9.4, 9.5_

  - [ ] 14.3 Register MCP resources in `mcp-server/src/resources.ts`
    - `pet://status`: current pet state summary
    - `pet://evolution-chart`: P1 evolution branching matrix
    - `pet://permissions`: current agent permission configuration
    - _Requirements: 9.2, 8.6_

  - [ ] 14.4 Wire up MCP server entry point in `mcp-server/src/index.ts`
    - Create McpServer with stdio transport
    - Register tools and resources
    - Initialize bridge connection
    - _Requirements: 9.1, 13.3_

- [ ] 15. Integration wiring and final polish
  - [ ] 15.1 Wire tama-core module exports in `tama-core/src/lib.rs`
    - Re-export all public modules: state, engine, evolution, actions, characters, persistence, permissions
    - Ensure tama-tauri and tama-tui both depend on tama-core in their Cargo.toml
    - _Requirements: All (integration)_

  - [ ] 15.2 Create `~/.tama96/` directory initialization utility
    - On first run from either frontend, create `~/.tama96/` if it doesn't exist
    - Initialize default `permissions.json` if missing
    - _Requirements: 6.1, 8.5_

  - [ ] 15.3 Write integration tests for Tauri IPC round-trip
    - Invoke command, verify state change, verify response matches expected state
    - _Requirements: 3.1-3.13, 9.3, 9.4_

  - [ ] 15.4 Write integration tests for MCP tool call flow
    - MCP tool call to TCP bridge to permission check to action to state change to response
    - Test permission denied flow returns structured error
    - _Requirements: 9.1-9.6, 8.1-8.3_

  - [ ] 15.5 Write integration test for lockfile exclusion
    - Acquire lock from one process, attempt from second, verify AlreadyLocked error
    - _Requirements: 7.1-7.4_

- [ ] 16. Final checkpoint - All components integrated
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests use the `proptest` crate and validate universal correctness properties from the design document
- The implementation language is Rust for all core/backend code, TypeScript for React frontend and MCP server
- tama-core has zero I/O dependencies: all persistence is injected via path parameters
