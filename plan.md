Implementation Plan — tama96

  Problem Statement
  Faithful recreation of the original 1996 Tamagotchi P1 as a cross-platform desktop app (Tauri v2) and terminal app (TUI), with a bundled MCP
  server exposing the pet to AI agents under human-configurable permission gating.
  ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Requirements
  - Faithful P1 mechanics: 4-stage lifecycle (baby, child, teen, adult + secret), hunger/happiness/discipline/weight meters, care/discipline
  mistake tracking, full evolution branching tree, character-specific lifespans, death
  - Real-time clock: 1 day = 1 year, passive heart drain, pet persists when app is closed
  - System tray persistence: Tauri Rust process stays alive when window is hidden
  - Two UI surfaces: clean minimal desktop GUI (React) and terminal TUI (ratatui)
  - Shared game engine: Rust library crate used by both frontends
  - Bundled MCP server (stdio transport) for AI agent interaction
  - Gated autonomy: human configures per-action allow/deny and rate limits for agents
  ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Research Findings
  - Tauri v2 tray-icon feature keeps the Rust process alive when the window is closed. JS APIs for tray exist but the JS context dies with the
  webview — game logic must live in Rust.
  - Tauri v2 sidecar support (externalBin in tauri.conf.json) bundles and spawns external binaries. Supports stdin/stdout communication via the
  shell plugin.
  - @modelcontextprotocol/sdk (TypeScript) supports stdio transport natively. Standard connection method for Claude Desktop, Cursor, Kiro, etc.
  - Ratatui is the standard Rust TUI library (19k+ stars). Its canvas widget supports braille character rendering (2x4 dots per cell), meaning a
  32x16 pixel Tamagotchi sprite fits in ~16x4 terminal cells.
  - Cargo workspaces allow a shared tama-core library crate consumed by both the Tauri binary and the TUI binary, keeping game logic in one place.
  - Single-instance constraint: only one process (Tauri OR TUI) should own the pet at a time. A lockfile prevents simultaneous access.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Original Tamagotchi P1 Mechanics Reference

  Life Stages & Timing
  | Stage   | Duration   | Characters (P1)              | Sleep Times |
  |---------|------------|------------------------------|-------------|
  | Egg     | 5 min      | —                            | —           |
  | Baby    | 65 min     | Babytchi                     | 5 min nap   |
  | Child   | 48 hrs     | Marutchi                     | 8pm–9am     |
  | Teen    | 48–72 hrs  | Tamatchi / Kuchitamatchi     | 9pm–9am     |
  | Adult   | 2–16 days  | 6 characters (see below)     | varies      |
  | Special | 15–16 days | Oyajitchi (JP) / Bill (Intl) | 10pm–9am    |

  Meters
  - Hunger: 4 hearts. Meal fills 1 heart, +1 weight.
  - Happiness: 4 hearts. Game win fills 1 heart, -1 weight. Snack fills 1 heart, +2 weight.
  - Discipline: 0–100% in 25% increments. Each successful discipline call = +25%.
  - Weight: increases from food, decreases from games.
  - Age: 1 year = 1 real day (increments on wake).

  Care & Discipline Mistakes
  - Care mistake: hunger or happiness at 0 for 15 minutes without response. Also: not turning off lights within 15 min of sleep.
  - Discipline mistake: Tamagotchi makes a false attention call (meters not empty) and user doesn't discipline within 15 minutes.
  - Mistakes are cumulative across child + teen stages for determining adult evolution.

  Evolution Matrix (P1 Original)
  Child to Teen:

  | Teen          | Care Mistakes | Discipline Mistakes      |
  |---------------|---------------|--------------------------|
  | Tamatchi      | 0–2           | Type 1: 0–2 / Type 2: 3+ |
  | Kuchitamatchi | 3+            | Type 1: 0–2 / Type 2: 3+ |
  Teen to Adult:

  | Adult       | Lifespan   | Requirements                                                      |
  |-------------|------------|-------------------------------------------------------------------|
  | Mametchi    | 15–16 days | Tamatchi T1, 0–2 care, 0 discipline mistakes                      |
  | Ginjirotchi | 11–12 days | Tamatchi T1, 0–2 care, 1 discipline mistake                       |
  | Maskutchi   | 15–16 days | Tamatchi T1: 0–2 care + 2+ disc / Tamatchi T2: 0–3 care + 2+ disc |
  | Kuchipatchi | 5–6 days   | 3+ care, 0–1 discipline (from Tamatchi T1 or Kuchitamatchi T1)    |
  | Nyorotchi   | 2–3 days   | 3+ care + moderate discipline (varies by teen path)               |
  | Tarakotchi  | 3–4 days   | 3+ care + 4+ discipline (varies by teen path)                     |
  Special: Oyajitchi/Bill evolves from Maskutchi (raised via Tamatchi T2) after 4 days.

  Heart Decay Rates
  Hearts drain passively. Rate accelerates with age:
  - Old adult cap (original): 1 hunger heart / 6 min, 1 happy heart / 7 min
  - Baby/child/teen: slower rates, character-specific

  Actions
  - Feed meal: +1 hunger heart, +1 weight
  - Feed snack: +1 happy heart, +2 weight, sickness risk if overfed
  - Play game (Left or Right): 5 rounds, 3+ wins = +1 happy, -1 weight
  - Medicine: cure sickness (1–2 doses)
  - Bathroom: clean poop
  - Lights: toggle for sleep/wake
  - Discipline: respond to false attention calls, +25% discipline meter

  Death Conditions
  - Old age (character-specific max lifespan)
  - Neglect (sustained empty meters, uncured sickness)
  - Snack overfeeding (only way to kill baby on original hardware)
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Architecture

  graph TB
      subgraph Workspace["Cargo Workspace"]
          Core["tama-core\n(shared library)\ngame engine, state,\nevolution, persistence"]

          subgraph Tauri["tama-tauri (desktop binary)"]
              TauriApp["Tauri IPC + Tray\n+ Sidecar Mgmt"]
              ReactUI["React Frontend\npet display, controls,\nagent permission settings"]
          end

          subgraph TUI["tama-tui (terminal binary)"]
              Ratatui["Ratatui Renderer\nbraille sprites, keyboard input"]
          end

          TauriApp --> Core
          Ratatui --> Core
      end

      subgraph MCP["mcp-server (Node.js sidecar)"]
          MCPServer["stdio MCP server\n@modelcontextprotocol/sdk"]
      end

      TauriApp <-->|"local TCP socket"| MCPServer
      Agent["AI Agent\n(Claude, Cursor, etc.)"] <-->|"stdio"| MCPServer
      ReactUI <-->|"Tauri IPC"| TauriApp


  Project Structure

  tama96/
  ├── PLAN.md
  ├── Cargo.toml                      # workspace root
  ├── crates/
  │   ├── tama-core/                  # shared library — pure game logic
  │   │   ├── Cargo.toml
  │   │   └── src/
  │   │       ├── lib.rs
  │   │       ├── state.rs            # PetState, AgentPermissions, Character enums
  │   │       ├── engine.rs           # tick loop, heart decay, care mistake timers
  │   │       ├── evolution.rs        # stage transitions, branching matrix
  │   │       ├── actions.rs          # feed, play, discipline, medicine, bathroom, lights
  │   │       ├── characters.rs       # per-character stats (sleep times, decay rates, lifespan)
  │   │       └── persistence.rs      # save/load JSON, catch-up elapsed time logic
  │   ├── tama-tauri/                 # Tauri v2 desktop binary
  │   │   ├── Cargo.toml
  │   │   ├── tauri.conf.json
  │   │   ├── capabilities/
  │   │   │   └── default.json
  │   │   ├── binaries/               # compiled MCP sidecar goes here
  │   │   └── src/
  │   │       ├── main.rs             # Tauri setup, tray, sidecar spawn
  │   │       ├── commands.rs         # Tauri IPC command handlers wrapping tama-core
  │   │       └── socket.rs           # local TCP socket server for MCP bridge
  │   └── tama-tui/                   # terminal UI binary
  │       ├── Cargo.toml
  │       └── src/
  │           ├── main.rs             # entry point, event loop
  │           ├── ui.rs               # ratatui layout and rendering
  │           └── sprites.rs          # pixel-to-braille sprite data
  ├── mcp-server/                     # Node.js MCP sidecar
  │   ├── package.json
  │   ├── tsconfig.json
  │   └── src/
  │       ├── index.ts                # MCP server entry, stdio transport
  │       ├── tools.ts                # tool definitions (feed, play, discipline, etc.)
  │       ├── resources.ts            # resource definitions (pet://status, etc.)
  │       └── bridge.ts               # TCP client connecting to Rust backend
  └── src/                            # React frontend (for Tauri webview)
      ├── index.html
      ├── main.tsx
      ├── App.tsx
      ├── components/
      │   ├── PetDisplay.tsx          # pixel-art pet sprite renderer
      │   ├── IconBar.tsx             # 8 action icons
      │   ├── StatusScreen.tsx        # hearts, discipline, age, weight
      │   ├── GameScreen.tsx          # Left or Right minigame
      │   └── SettingsPanel.tsx       # agent permission configuration
      ├── hooks/
      │   └── usePetState.ts          # Tauri command bridge hook
      └── styles/
          └── index.css               # minimal pixel aesthetic

  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task Breakdown
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 1: Workspace Scaffolding
  Objective: Initialize the Cargo workspace, Tauri v2 + React project, and TUI crate stubs.

  Implementation guidance:
  - Create workspace Cargo.toml with members: crates/tama-core, crates/tama-tauri, crates/tama-tui
  - Scaffold tama-tauri using create-tauri-app with React + TypeScript + Vite, then restructure into the workspace layout
  - Create tama-core as a library crate with serde, serde_json, chrono dependencies
  - Create tama-tui as a binary crate with ratatui, crossterm dependencies
  - Create mcp-server/ with package.json and @modelcontextprotocol/sdk dependency
  - Verify: cargo tauri dev launches a window, cargo run -p tama-tui prints hello, cd mcp-server && npm install succeeds

  Tests: Workspace compiles. Each binary runs without error.

  Demo: Tauri window opens. TUI binary prints to terminal. MCP server dependencies install.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 2: Core Game State Model
  Objective: Define the complete pet state, character data, and persistence layer in tama-core.

  Implementation guidance:
  - state.rs: PetState struct — stage (enum: Egg, Baby, Child, Teen, Adult, Special, Dead), character (enum of all P1 characters), age, weight,
  hunger (0–4), happiness (0–4), discipline (0–100), caremistakes, disciplinemistakes, poopcount, issick, issleeping, isalive, lasttick (DateTime),
  birthtime, pendingcaredeadline (Option DateTime), pendingdisciplinedeadline (Option DateTime)
  - state.rs: AgentPermissions struct — enabled (bool), per-action allow/deny map, per-action rate limits (max count per hour), action_log (Vec of
  timestamped actions for rate limit enforcement)
  - characters.rs: per-character constants — sleeptime, waketime, basehungerdecayminutes, basehappydecayminutes, baseweight, maxlifespan_days
  - persistence.rs: save(state, path) and load(path) -> PetState using serdejson. Lockfile acquisition/release to prevent dual access. Catch-up
  logic: calculate elapsed minutes since `lasttick`, simulate that many ticks on load.

  Tests:
  - Serialization round-trip (save, load, compare)
  - Catch-up: save state, advance clock 30 min, load, verify hearts decremented appropriately
  - Lockfile: attempt double-acquire fails
  Demo: App creates a default pet state, persists to ~/.tama96/state.json, reloads it on restart with correct values.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 3: Game Engine — Tick Loop & Core Mechanics
  Objective: Implement the real-time simulation that drives the pet's life in tama-core.

  Implementation guidance:
  - engine.rs: tick(state: &mut PetState, now: DateTime) — the core function called every ~60 seconds:
    - Decrement hunger/happiness hearts based on character-specific decay rates
    - Check if hunger or happiness hit 0, start 15-min care deadline if not already pending
    - Check if care deadline expired, increment care_mistakes, clear deadline
    - Increment poop timer, add poop at character-specific intervals
    - Check poop count, trigger sickness if too many
    - Check sickness from old age (increasing frequency as adult ages)
    - Check death conditions (neglect thresholds, max lifespan reached)
    - Update last_tick timestamp
  - actions.rs:
    - feed_meal(state): if not sleeping/sick, +1 hunger (cap 4), +1 weight, clear care deadline if hunger was 0
    - feed_snack(state): +1 happiness (cap 4), +2 weight, sickness risk counter
    - play_game(state, moves: [Choice; 5]) -> GameResult: resolve 5 rounds of Left/Right, 3+ wins = +1 happy + -1 weight
    - discipline(state) -> bool: if false-call pending, +25% discipline, clear discipline deadline. Return false if no call pending.
    - give_medicine(state): cure sickness (may need 2 doses, track dose count)
    - clean_poop(state): decrement poop_count
    - toggle_lights(state): flip lights for sleep/wake, clear care deadline if was a sleep-lights call
  - Discipline calls: engine randomly generates false attention calls during each life stage. Track with pending_discipline_deadline. If
  unresponded in 15 min, increment discipline_mistakes.

  Tests:
  - Heart decay: set hunger to 4, tick N times, verify correct decrement
  - Care mistake: set hunger to 0, advance 15+ min without feeding, verify care_mistakes incremented
  - Feed meal: verify hunger +1, weight +1, capped at 4
  - Snack sickness: feed many snacks rapidly, verify sickness triggers
  - Game: verify 3+ wins = happy +1, weight -1; 2 or fewer wins = no happy change, still -1 weight
  - Discipline: verify +25% per successful call, no effect when no call pending
  Demo: Pet hatches from egg, hearts visibly decay over time, feeding and playing change meters, poop appears periodically.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 4: Evolution System & Character Branching
  Objective: Implement the full P1 evolution tree in tama-core.

  Implementation guidance:
  - evolution.rs: check_evolution(state: &mut PetState, now: DateTime) -> bool — called each tick, returns true if evolution occurred:
    - Egg to Babytchi: after 5 min
    - Babytchi to Marutchi: after 65 min
    - Marutchi to Teen: at age 3 (3 days). Branch based on cumulative care_mistakes:
      - 0–2 care mistakes = Tamatchi. Sub-type: 0–2 discipline mistakes = T1, 3+ = T2
      - 3+ care mistakes = Kuchitamatchi. Same sub-type split.
    - Teen to Adult: at age 6 (6 days). Full branching matrix per the reference table above. Reset discipline meter on evolution (may carry
  partial).
    - Maskutchi to Oyajitchi/Bill: after 4 additional days, if raised from Tamatchi T2 with 0 discipline
  - On evolution: set evolution animation flag, reset discipline meter to stage-appropriate level, adjust sleep schedule to new character's times
  - Track teen_type (T1/T2) and teen_character on state for adult branching lookup

  Tests:
  - Perfect care path: 0 care + 0 discipline mistakes from Tamatchi T1 = Mametchi
  - Each adult reachable: unit test per evolution path verifying correct character
  - Special character: Tamatchi T2 to Maskutchi to Oyajitchi after 4 days
  - Edge case: evolution during sleep (should wait until wake)
  - Discipline meter reset on evolution
  Demo: Raise multiple pets with different care patterns, observe different teen and adult evolutions.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 5: System Tray & Background Persistence (Tauri)
  Objective: App minimizes to system tray on close; Rust engine keeps ticking.

  Implementation guidance:
  - tama-tauri/Cargo.toml: enable tray-icon feature
  - main.rs: on window close event, hide window instead of quitting
  - Create system tray with:
    - Icon (small pet sprite or egg icon)
    - Tooltip: pet name, age, current status
    - Menu items: Show Window, Pet Status (submenu with hearts/age), Quit
    - Double-click to restore and focus window
  - Spawn a background tokio task running the tick loop (calls tama_core::engine::tick() every 60 seconds), persists state after each tick
  - Desktop notifications via tauri-plugin-notification:
    - Hunger or happiness at 0 (attention needed)
    - Pet is sick
    - Discipline call pending
    - Pet died

  Tests:
  - Close window, verify Rust process still alive (tray visible)
  - Reopen from tray, verify state advanced (hearts lower, age correct)
  - Notification fires when hunger hits 0
  Demo: Close the app, wait a few minutes, reopen from tray — pet has aged, hearts have drained, poop may have appeared.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 6: React UI — Pet Display & Interaction
  Objective: Build the minimal pixel-art desktop interface.

  Implementation guidance:
  - PetDisplay.tsx: render the pet sprite in a scaled pixel grid. Use a canvas element or CSS grid mapping 32x16 pixel data to colored cells.
  Animate idle, happy, eating, sleeping states.
  - IconBar.tsx: 8 clickable icons matching the original layout (top row: Feed, Light, Game, Medicine; bottom row: Bathroom, Meter, Discipline,
  Attention). Attention icon lights up automatically when pet needs something.
  - StatusScreen.tsx: toggled via Meter icon. Shows hunger hearts, happy hearts, discipline gauge bar, age in years, weight.
  - GameScreen.tsx: Left or Right minigame. 5 rounds, left/right buttons, shows pet turning, round results, final score.
  - SettingsPanel.tsx: accessed via menu/gear icon. Agent permission toggles (see Task 9).
  - usePetState.ts: hook that polls Rust backend via invoke() for current state, provides action dispatch functions.
  - Aesthetic: monochrome or very limited palette, pixel/bitmap font, no gradients/shadows, clean borders. Think "original LCD scaled up with
  taste."
  - All interactions dispatch Tauri commands to Rust backend. UI is a pure state reflection.

  Tests: Each UI action invokes the correct Tauri command. State updates reflect in UI within one render cycle.

  Demo: Full interactive pet experience — hatch egg, feed, play Left or Right, discipline, toggle lights, view status, watch evolution.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 7: Terminal UI (Ratatui)
  Objective: Build a fully functional TUI frontend sharing the same game engine.

  Implementation guidance:
  - tama-tui/src/main.rs: initialize crossterm raw mode, create ratatui terminal, run event loop (tick every 1s for UI refresh, game tick every 60s
  via tama_core)
  - ui.rs: ratatui layout:
    - Top: pet name, age, stage
    - Center: pet sprite rendered via canvas widget using braille characters (32x16 pixels in ~16x4 terminal cells). Map sprite pixel data to
  braille dot patterns.
    - Middle: hunger hearts (filled/empty unicode hearts), happy hearts, discipline bar as gauge widget
    - Bottom: status line (weight, sick indicator, poop count) + keybind hints
  - sprites.rs: define pixel data for each character as 32x16 bool arrays. Convert to braille on render.
  - Keyboard mappings:
    - f = feed (then m for meal, s for snack)
    - g = play game (then left/right arrow or a/d for choices)
    - d = discipline
    - c = clean poop
    - l = toggle lights
    - i = medicine
    - tab = toggle status view
    - q = quit
  - Same persistence file as Tauri (~/.tama96/state.json). Lockfile prevents simultaneous Tauri + TUI access.
  - On startup: acquire lock, load state with catch-up. On quit: save state, release lock.

  Tests:
  - Sprite rendering: verify braille output for a known pixel pattern
  - Keyboard input: verify each key dispatches correct tama_core action
  - Lockfile: TUI refuses to start if Tauri already holds the lock
  Demo: Run tama-tui in a terminal. Pet displays as braille art. Feed it, play games, watch hearts change — same pet state as the desktop app.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 8: MCP Server Sidecar
  Objective: Build the MCP server that exposes the pet to AI agents.

  Implementation guidance:
  - mcp-server/src/index.ts: create MCP server using @modelcontextprotocol/sdk with stdio transport. Register tools and resources.
  - tools.ts — MCP tools:
    - feed — params: { type: "meal" | "snack" } — feeds the pet
    - play_game — params: { moves: ["left"|"right", ...] } — plays Left or Right (5 moves)
    - discipline — no params — disciplines the pet if a call is pending
    - give_medicine — no params — administers medicine
    - clean_poop — no params — cleans poop
    - toggle_lights — no params — toggles lights
    - get_status — no params — returns full pet state (convenience tool alternative to resource)
  - resources.ts — MCP resources:
    - pet://status — full state snapshot (stage, character, all meters, age, weight, alive, sleeping, sick)
    - pet://evolution-chart — possible evolutions from current state with requirements
    - pet://permissions — current agent permission config so agents can introspect constraints
  - bridge.ts: TCP client that connects to the Rust backend's local socket. Sends JSON action requests, receives state responses. Reconnects on
  failure.
  - Build: compile to standalone binary using Node.js SEA (Single Executable Application) or pkg. Place compiled binary in tama-tauri/binaries/
  with target triple suffix.
  - Tauri config: add to externalBin in tauri.conf.json. Rust spawns sidecar on app startup, monitors health, restarts on crash.
  - Ship example mcp-config.json:

  {
    "mcpServers": {
      "tama96": {
        "command": "/path/to/tama96-mcp",
        "transport": "stdio"
      }
    }
  }


  Tests:
  - MCP Inspector connects, lists tools and resources
  - Call feed tool, verify pet state changes
  - Read pet://status resource, verify correct JSON snapshot
  - Kill MCP server, verify Tauri restarts it
  Demo: Configure Claude Desktop with the MCP server. Agent reads pet status, feeds the pet, plays a game — all reflected in the desktop/TUI UI.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 9: Gated Autonomy — Agent Permission System
  Objective: Human-configurable permission layer controlling what agents can do.

  Implementation guidance:
  - tama-core/state.rs AgentPermissions:
    - enabled: bool — master kill switch
    - allowed_actions: HashMap<ActionType, ActionPermission> where ActionPermission has allowed: bool and max_per_hour: Option<u32>
    - action_log: Vec<(ActionType, DateTime)> — rolling log for rate limit enforcement
  - tama-core permission check function: check_permission(permissions, action, now) -> Result<(), PermissionDenied> — returns error with
  human-readable reason (e.g. "feedsnack is disabled by owner" or "feedmeal rate limit exceeded: 5/hour, 5 used")
  - MCP server: calls permission check before every tool execution. Denied actions return MCP error with the reason string.
  - pet://permissions resource: agents can read this to understand their constraints before attempting actions.
  - React SettingsPanel.tsx:
    - Master toggle: enable/disable all agent access
    - Per-action rows: action name, allow/deny toggle, rate limit input (actions per hour)
    - Visual indicator showing recent agent activity (last 5 actions with timestamps)
  - Permissions persist in ~/.tama96/permissions.json (separate from pet state)

  Tests:
  - Disable feed_snack, MCP feed with type=snack returns permission error
  - Set rate limit 3/hour on feed_meal, 4th call within the hour is rejected
  - Master disable, all tools return permission error
  - pet://permissions resource reflects current config accurately
  Demo: Human disables "feed snack" in settings panel. Agent attempts to feed a snack via MCP, gets a clear permission denied message. Agent reads
  pet://permissions and adjusts strategy.
  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

  Task 10: Polish & End-to-End Integration
  Objective: Wire everything together, handle edge cases, ship-ready quality.

  Implementation guidance:
  - Startup flow:
    - If no save file: first-launch onboarding, hatch egg animation
    - If pet is dead: death screen with option to hatch new egg
    - If pet is alive: catch-up elapsed time, show current state
  - Death screen: show ghost/angel (matching original), display final age, button to start over
  - Edge cases:
    - MCP server crash: Tauri auto-restarts sidecar
    - Simultaneous agent connections: MCP server handles one at a time (stdio is inherently single-client)
    - Clock manipulation: detect if system clock jumped backwards, handle gracefully
    - Corrupt save file: fallback to fresh state with warning
  - UI polish:
    - Consistent pixel/bitmap font across desktop UI
    - Smooth sprite animation transitions (idle, eating, sleeping)
    - Responsive layout (window resizable within reasonable bounds)
    - TUI: graceful terminal resize handling
  - Documentation:
    - README.md with setup instructions, screenshots, MCP config example
    - Agent guide: what tools/resources are available, example prompts

  Tests:
  - Full end-to-end: launch app, hatch, interact via UI, interact via MCP, close app, reopen, verify continuity
  - Death and restart cycle
  - Corrupt save recovery
  - TUI and desktop produce identical game outcomes for identical inputs
  Demo: Complete working app — human and AI agent co-caring for a Tamagotchi across desktop and terminal interfaces, with the human controlling
  agent permissions. Pet lives, evolves, and eventually dies faithfully to the 1996 original.