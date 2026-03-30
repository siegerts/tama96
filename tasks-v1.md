# Tasks

## Task 1: Workspace Scaffolding

- [ ] 1.1 Create workspace root Cargo.toml with members: crates/tama-core, crates/tama-tauri, crates/tama-tui
- [ ] 1.2 Create tama-core library crate with Cargo.toml (serde, serde_json, chrono, rand dependencies)
- [ ] 1.3 Scaffold tama-tauri as a Tauri v2 binary crate with React + TypeScript + Vite frontend
- [ ] 1.4 Create tama-tui binary crate with Cargo.toml (ratatui, crossterm, dirs dependencies)
- [ ] 1.5 Create mcp-server/ Node.js project with package.json (@modelcontextprotocol/sdk, zod dependencies)
- [ ] 1.6 Verify workspace compiles: `cargo build` succeeds, `cd mcp-server && npm install` succeeds

## Task 2: Core State Model (tama-core)
<!-- Depends on: Task 1 -->

- [ ] 2.1 Implement state.rs: LifeStage, Character, TeenType enums with Serialize/Deserialize derives
- [ ] 2.2 Implement state.rs: PetState struct with all fields (meters, timers, flags, deadlines) and PetState::new_egg() constructor
- [ ] 2.3 Implement state.rs: AgentPermissions, ActionType, ActionPermission, ActionLogEntry structs
- [ ] 2.4 Implement characters.rs: CharacterStats struct and CharacterStats::for_character() with all 11 character constant tables
- [ ] 2.5 Write property test: State invariants — meter bounds hold after random action sequences (Property 1)
  - [ ] 🧪 PBT: For any PetState produced by any sequence of actions and ticks, hunger ∈ [0,4], happiness ∈ [0,4], discipline ∈ [0,100] and multiple of 25, weight ≥ 1, poop_count ∈ [0,4]
- [ ] 2.6 Write property test: Stage-character consistency (Property 2)
  - [ ] 🧪 PBT: For any PetState, Character variant is valid for current Life_Stage and teen_type is Some only when stage ∈ {Teen, Adult, Special}

## Task 3: Persistence Layer (tama-core)
<!-- Depends on: Task 2 -->

- [ ] 3.1 Implement persistence.rs: save() — atomic JSON write (temp file + rename)
- [ ] 3.2 Implement persistence.rs: load() — JSON deserialization with catch-up tick simulation
- [ ] 3.3 Implement persistence.rs: acquire_lock() / release_lock() — PID-based lockfile at ~/.tama96/tama96.lock
- [ ] 3.4 Implement persistence.rs: corrupt file recovery — backup corrupt file, create fresh egg
- [ ] 3.5 Implement persistence.rs: clock skew handling — skip tick if now < last_tick, resync
- [ ] 3.6 Write property test: Persistence round-trip (Property 15)
  - [ ] 🧪 PBT: For any valid PetState, serialize then deserialize produces an equivalent PetState
- [ ] 3.7 Write property test: Catch-up convergence (Property 16)
  - [ ] 🧪 PBT: For any saved PetState and elapsed time, loading with catch-up results in last_tick == now
- [ ] 3.8 Write unit test: Lockfile mutual exclusion — acquire succeeds, double-acquire fails, release then re-acquire succeeds
- [ ] 3.9 Write unit test: Corrupt save file recovery — invalid JSON triggers backup and fresh egg creation

## Task 4: Engine Tick Loop (tama-core)
<!-- Depends on: Task 2, Task 3 -->

- [ ] 4.1 Implement engine.rs: tick() main function — dispatch to sub-checks based on stage and alive/sleeping status
- [ ] 4.2 Implement engine.rs: decay_hearts() — hunger/happiness decrement based on character decay rates and elapsed time
- [ ] 4.3 Implement engine.rs: check_care_deadlines() — create 15-min deadline when meter hits 0, increment care_mistakes on expiry
- [ ] 4.4 Implement engine.rs: check_poop() — accumulate poop at character-specific intervals
- [ ] 4.5 Implement engine.rs: check_sickness() — trigger sickness from poop accumulation
- [ ] 4.6 Implement engine.rs: maybe_generate_discipline_call() / check_discipline_deadline() — random false attention calls, 15-min deadline, discipline_mistakes on expiry
- [ ] 4.7 Implement engine.rs: check_sleep() / check_wake() — sleep/wake transitions based on character schedule, age increment on wake
- [ ] 4.8 Implement engine.rs: check_death() — old age, neglect (12h empty meters), untreated sickness (24h), baby overfeeding
- [ ] 4.9 Write property test: Heart decay correctness (Property 3)
  - [ ] 🧪 PBT: For any awake alive Pet with known Character and elapsed time, tick decrements hunger by floor(elapsed/hunger_decay_minutes) and happiness by floor(elapsed/happy_decay_minutes), clamped to 0
- [ ] 4.10 Write property test: Sleep immunity (Property 5)
  - [ ] 🧪 PBT: For any sleeping Pet, a tick does not change hunger or happiness
- [ ] 4.11 Write property test: Dead pets don't tick (Property 6)
  - [ ] 🧪 PBT: For any dead Pet, a tick leaves the entire PetState unchanged
- [ ] 4.12 Write property test: Death from old age (Property 20)
  - [ ] 🧪 PBT: For any Character with max_lifespan_days > 0, when age reaches that value, is_alive becomes false and stage becomes Dead
- [ ] 4.13 Write unit test: Care deadline lifecycle — meter hits 0 creates deadline, expiry increments care_mistakes

## Task 5: Player Actions (tama-core)
<!-- Depends on: Task 2, Task 4 -->

- [ ] 5.1 Implement actions.rs: feed_meal() — +1 hunger (cap 4), +1 weight, clear care deadline if hunger was 0
- [ ] 5.2 Implement actions.rs: feed_snack() — +1 happiness (cap 4), +2 weight, snack sickness risk for babies
- [ ] 5.3 Implement actions.rs: play_game() — 5 rounds Left/Right, 3+ wins = +1 happy, always -1 weight (min 1)
- [ ] 5.4 Implement actions.rs: discipline() — +25 discipline if pending call, error if no call
- [ ] 5.5 Implement actions.rs: give_medicine() — increment dose count, cure at 2 doses
- [ ] 5.6 Implement actions.rs: clean_poop() — decrement poop_count
- [ ] 5.7 Implement actions.rs: toggle_lights() — flip lights, trigger sleep if appropriate
- [ ] 5.8 Implement action precondition checks — PetIsDead error, sleeping error, sick error as appropriate
- [ ] 5.9 Write property test: Feed meal correctness (Property 7)
  - [ ] 🧪 PBT: For any alive awake non-sick Pet, feed_meal sets hunger to min(old+1, 4) and weight to old+1
- [ ] 5.10 Write property test: Feed snack correctness (Property 8)
  - [ ] 🧪 PBT: For any alive awake Pet, feed_snack sets happiness to min(old+1, 4) and weight to old+2
- [ ] 5.11 Write property test: Game outcome correctness (Property 9)
  - [ ] 🧪 PBT: For any game with 5 moves, 3+ wins increases happiness by 1 (cap 4), weight always decreases by 1 (floor 1)
- [ ] 5.12 Write property test: Discipline action correctness (Property 10)
  - [ ] 🧪 PBT: For any Pet with pending call, discipline increases by 25 (cap 100) and clears deadline; without pending call, returns error
- [ ] 5.13 Write property test: Medicine curing (Property 11)
  - [ ] 🧪 PBT: For any sick Pet, two medicine doses set is_sick to false and sick_dose_count to 0
- [ ] 5.14 Write property test: Action precondition enforcement (Property 12)
  - [ ] 🧪 PBT: For any dead Pet and any action, returns PetIsDead; for any sleeping Pet, feed_meal and play_game return error

## Task 6: Evolution System (tama-core)
<!-- Depends on: Task 2, Task 4 -->

- [ ] 6.1 Implement evolution.rs: check_evolution() — stage transition dispatcher
- [ ] 6.2 Implement evolution.rs: check_egg_hatch() — Egg → Baby after 5 minutes
- [ ] 6.3 Implement evolution.rs: Baby → Child (Marutchi) after 65 minutes
- [ ] 6.4 Implement evolution.rs: resolve_teen() — Child → Teen branching on care_mistakes/discipline_mistakes
- [ ] 6.5 Implement evolution.rs: resolve_adult() — Teen → Adult full P1 branching matrix
- [ ] 6.6 Implement evolution.rs: Adult → Special (Maskutchi T2 → Oyajitchi after 4 days)
- [ ] 6.7 Implement evolution.rs: evolve_to() helper — set stage, character, reset discipline, clear deadlines
- [ ] 6.8 Write property test: Evolution determinism (Property 13)
  - [ ] 🧪 PBT: For any two calls to resolve_adult with identical (teen_char, teen_type, care_mistakes, discipline_mistakes), the result is identical
- [ ] 6.9 Write property test: Evolution reset postconditions (Property 14)
  - [ ] 🧪 PBT: For any evolution event, discipline is 0 and pending deadlines are None
- [ ] 6.10 Write unit tests: Each adult character reachable — one test per evolution path (Mametchi, Ginjirotchi, Maskutchi, Kuchipatchi, Nyorotchi, Tarakotchi)
- [ ] 6.11 Write unit test: Oyajitchi special evolution from Maskutchi (Tamatchi T2 path) after 4 days

## Task 7: Permission System (tama-core)
<!-- Depends on: Task 2 -->

- [ ] 7.1 Implement permissions.rs: check_permission() — master switch, per-action allow/deny, rate limit enforcement
- [ ] 7.2 Implement permissions.rs: action_log pruning — remove entries older than 1 hour
- [ ] 7.3 Implement permissions.rs: save/load permissions to ~/.tama96/permissions.json
- [ ] 7.4 Write property test: Permission gating (Property 18)
  - [ ] 🧪 PBT: For any action type with master switch disabled, returns MasterDisabled; for any disabled action, returns PermissionDenied with action name
- [ ] 7.5 Write property test: Rate limiting (Property 19)
  - [ ] 🧪 PBT: For any action with max_per_hour limit, if action_log has n ≥ limit entries in last hour, returns RateLimited

## Task 8: Tauri Desktop App (tama-tauri)
<!-- Depends on: Task 4, Task 5, Task 6, Task 7 -->

- [ ] 8.1 Implement main.rs: Tauri app setup with system tray (icon, tooltip, menu: Show Window, Pet Status, Quit)
- [ ] 8.2 Implement main.rs: window close handler — hide to tray instead of quit
- [ ] 8.3 Implement main.rs: background tick loop — tokio task calling engine::tick() every 60 seconds, auto-save after each tick
- [ ] 8.4 Implement commands.rs: Tauri IPC command handlers wrapping tama-core actions (get_state, feed_meal, feed_snack, play_game, discipline, give_medicine, clean_poop, toggle_lights)
- [ ] 8.5 Implement socket.rs: local TCP socket server for MCP bridge communication
- [ ] 8.6 Implement main.rs: MCP sidecar spawn and health monitoring with exponential backoff restart (2s, 4s, 8s, max 30s)
- [ ] 8.7 Implement desktop notifications via tauri-plugin-notification (hunger/happiness at 0, sick, discipline call, death)
- [ ] 8.8 Configure tauri.conf.json: tray-icon feature, externalBin for MCP sidecar, shell plugin

## Task 9: React Frontend
<!-- Depends on: Task 8 -->

- [ ] 9.1 Implement usePetState.ts hook: Tauri IPC bridge with 1Hz polling and action dispatch functions
- [ ] 9.2 Implement PetDisplay.tsx: pixel-art sprite renderer with animation states (idle, eating, sleeping, happy)
- [ ] 9.3 Implement IconBar.tsx: 8 action icons matching P1 layout (Feed, Light, Game, Medicine, Bathroom, Meter, Discipline, Attention)
- [ ] 9.4 Implement StatusScreen.tsx: hunger hearts, happiness hearts, discipline gauge, age, weight display
- [ ] 9.5 Implement GameScreen.tsx: Left/Right minigame with 5 rounds, move buttons, round results, score
- [ ] 9.6 Implement SettingsPanel.tsx: agent permission master toggle, per-action allow/deny toggles, rate limit inputs, recent agent activity
- [ ] 9.7 Implement death screen: ghost/angel display, final age, Hatch New Egg button
- [ ] 9.8 Apply pixel aesthetic: monochrome/limited palette, pixel font, clean borders, no gradients

## Task 10: Terminal UI (tama-tui)
<!-- Depends on: Task 4, Task 5, Task 6, Task 3 -->

- [ ] 10.1 Implement main.rs: crossterm raw mode init, ratatui terminal, event loop (1s UI refresh, 60s game tick)
- [ ] 10.2 Implement sprites.rs: 32x16 bool arrays for each character, pixel-to-braille conversion
- [ ] 10.3 Implement ui.rs: ratatui layout — pet name/age/stage header, braille sprite center, hearts/gauge meters, status line, keybind hints
- [ ] 10.4 Implement keyboard input handling: f=feed, g=game, d=discipline, c=clean, l=lights, i=medicine, tab=status, q=quit
- [ ] 10.5 Implement startup: acquire lockfile, load state with catch-up; shutdown: save state, release lockfile
- [ ] 10.6 Implement terminal resize handling

## Task 11: MCP Server (Node.js Sidecar)
<!-- Depends on: Task 8 (socket.rs), Task 7 -->

- [ ] 11.1 Implement index.ts: MCP server setup with stdio transport using @modelcontextprotocol/sdk
- [ ] 11.2 Implement tools.ts: register tools (feed, play_game, discipline, give_medicine, clean_poop, toggle_lights, get_status) with zod schemas
- [ ] 11.3 Implement resources.ts: register resources (pet://status, pet://evolution-chart, pet://permissions)
- [ ] 11.4 Implement bridge.ts: TCP client connecting to Rust backend, JSON request/response, reconnect on failure
- [ ] 11.5 Implement permission check integration: check permissions before every tool execution, return structured errors on denial
- [ ] 11.6 Build configuration: compile to standalone binary (Node.js SEA or pkg), place in tama-tauri/binaries/

## Task 12: Integration & Polish
<!-- Depends on: Task 8, Task 9, Task 10, Task 11 -->

- [ ] 12.1 Implement first-launch onboarding: no save file → hatch egg animation
- [ ] 12.2 Implement dead pet flow: death screen → option to hatch new egg (both Desktop and Terminal)
- [ ] 12.3 Write integration test: full lifecycle egg → baby → child → teen → adult → death
- [ ] 12.4 Write integration test: MCP tool call → TCP bridge → Rust action → state change → response
- [ ] 12.5 Write integration test: lockfile exclusion — start one frontend, verify second is rejected
- [ ] 12.6 Create example mcp-config.json for Claude Desktop / Cursor / Kiro configuration
- [ ] 12.7 Write README.md with setup instructions, screenshots placeholder, and MCP config example
