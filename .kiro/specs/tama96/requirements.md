# Requirements Document

## Introduction

tama96 is a faithful recreation of the original 1996 Tamagotchi P1 virtual pet, delivered as a cross-platform desktop app (Tauri v2 with React frontend) and a terminal app (ratatui TUI). A shared Rust game engine (`tama-core`) implements the complete P1 lifecycle — egg through death — with real-time clock simulation where 1 real day equals 1 pet year. A bundled MCP server (Node.js sidecar, stdio transport) exposes the pet to AI agents under human-configurable permission gating. The system enforces single-instance access via lockfile.

## Glossary

- **Engine**: The `tama-core` Rust library crate containing all game logic, state mutations, and persistence
- **Pet**: The virtual Tamagotchi P1 creature managed by the Engine
- **Tick**: A periodic simulation step (every 60 seconds) that advances the Pet's state
- **Meter**: A bounded numeric attribute of the Pet (hunger, happiness, discipline, weight)
- **Care_Mistake**: An event recorded when hunger or happiness remains at 0 for 15 minutes without player response, or when lights are not turned off within 15 minutes of sleep time
- **Discipline_Mistake**: An event recorded when a false attention call goes unanswered for 15 minutes
- **Evolution**: A stage transition where the Pet advances to the next life stage and potentially changes character
- **Life_Stage**: One of Egg, Baby, Child, Teen, Adult, Special, or Dead
- **Character**: A specific Tamagotchi identity within a Life_Stage (e.g., Mametchi, Tarakotchi)
- **MCP_Server**: The Node.js sidecar process exposing Pet actions to AI agents via stdio MCP protocol
- **Agent**: An external AI client (e.g., Claude, Cursor) interacting with the Pet through the MCP_Server
- **Permission_System**: The human-configurable layer that gates Agent actions with per-action allow/deny toggles and rate limits
- **Lockfile**: A file-based mutual exclusion mechanism preventing simultaneous access from multiple frontends
- **Desktop_App**: The Tauri v2 binary with React frontend
- **Terminal_App**: The ratatui-based terminal UI binary
- **Catch_Up**: The process of simulating elapsed ticks when loading a saved state after the app was closed

## Requirements

### Requirement 1: Pet State Model

**User Story:** As a developer, I want a well-defined pet state model, so that all game logic operates on consistent, bounded data.

#### Acceptance Criteria

1. THE Engine SHALL maintain hunger as an integer in the range [0, 4]
2. THE Engine SHALL maintain happiness as an integer in the range [0, 4]
3. THE Engine SHALL maintain discipline as an integer in the range [0, 100] that increments in steps of 25
4. THE Engine SHALL maintain weight as an integer with a minimum value of 1
5. THE Engine SHALL maintain poop_count as an integer in the range [0, 4]
6. THE Engine SHALL enforce that the Character variant is valid for the current Life_Stage
7. THE Engine SHALL store teen_type only when the Life_Stage is Teen, Adult, or Special

### Requirement 2: Tick Simulation

**User Story:** As a player, I want my pet to age and change in real time, so that it feels like a living creature.

#### Acceptance Criteria

1. WHEN a Tick occurs, THE Engine SHALL decrement hunger hearts based on the Character-specific hunger_decay_minutes rate
2. WHEN a Tick occurs, THE Engine SHALL decrement happiness hearts based on the Character-specific happy_decay_minutes rate
3. WHEN hunger or happiness reaches 0, THE Engine SHALL start a 15-minute care deadline if no deadline is already pending
4. WHEN a care deadline expires without player response, THE Engine SHALL increment care_mistakes by 1
5. WHEN the Character-specific poop interval elapses, THE Engine SHALL increment poop_count by 1
6. WHILE the Pet is sleeping, THE Engine SHALL skip hunger and happiness decay
7. WHILE the Pet is dead, THE Engine SHALL treat the Tick as a no-op
8. WHEN a Tick completes, THE Engine SHALL set last_tick to the current timestamp

### Requirement 3: Player Actions

**User Story:** As a player, I want to feed, play with, and care for my pet, so that I can keep it alive and healthy.

#### Acceptance Criteria

1. WHEN the player feeds a meal, THE Engine SHALL increase hunger by 1 (capped at 4) and increase weight by 1
2. WHEN the player feeds a snack, THE Engine SHALL increase happiness by 1 (capped at 4) and increase weight by 2
3. WHEN the player feeds a snack and snack_count_since_last_tick exceeds 3 during the Baby stage, THE Engine SHALL set is_sick to true
4. WHEN the player plays a game with 5 moves and wins 3 or more rounds, THE Engine SHALL increase happiness by 1 (capped at 4)
5. WHEN the player plays a game, THE Engine SHALL decrease weight by 1 (floored at 1) regardless of outcome
6. WHEN the player disciplines the Pet while a discipline call is pending, THE Engine SHALL increase discipline by 25 (capped at 100) and clear the pending deadline
7. IF the player attempts to discipline the Pet with no pending discipline call, THEN THE Engine SHALL return a NoDisciplineCallPending error
8. WHEN the player gives medicine and sick_dose_count reaches 2, THE Engine SHALL set is_sick to false and reset sick_dose_count to 0
9. WHEN the player cleans poop, THE Engine SHALL decrement poop_count by 1
10. WHEN the player toggles lights off while the Pet should be sleeping, THE Engine SHALL set is_sleeping to true
11. WHEN the player feeds a meal while hunger is at 4, THE Engine SHALL keep hunger at 4 and still increase weight by 1
12. IF the player attempts an action while the Pet is dead, THEN THE Engine SHALL return a PetIsDead error
13. IF the player attempts to feed a meal or play a game while the Pet is sleeping, THEN THE Engine SHALL return an error

### Requirement 4: Evolution System

**User Story:** As a player, I want my pet to evolve based on how I care for it, so that my care choices have meaningful consequences.

#### Acceptance Criteria

1. WHEN 5 minutes have elapsed since the Egg stage started, THE Engine SHALL evolve the Pet to Baby stage as Babytchi
2. WHEN 65 minutes have elapsed since the Baby stage started, THE Engine SHALL evolve the Pet to Child stage as Marutchi
3. WHEN the Pet reaches age 3 during the Child stage, THE Engine SHALL evolve the Pet to Teen stage, selecting Tamatchi (0–2 care_mistakes) or Kuchitamatchi (3+ care_mistakes)
4. WHEN the Pet reaches age 6 during the Teen stage, THE Engine SHALL evolve the Pet to Adult stage using the full P1 branching matrix based on Character, teen_type, care_mistakes, and discipline_mistakes
5. WHEN Maskutchi (raised via Tamatchi Type2 path) has been in the Adult stage for 4 days, THE Engine SHALL evolve the Pet to Special stage as Oyajitchi
6. WHEN evolution occurs, THE Engine SHALL reset discipline to 0 and clear pending care and discipline deadlines
7. THE Engine SHALL produce the same adult Character for identical combinations of teen Character, teen_type, care_mistakes, and discipline_mistakes

### Requirement 5: Death Conditions

**User Story:** As a player, I want realistic consequences for neglect, so that caring for my pet feels meaningful.

#### Acceptance Criteria

1. WHEN the Pet's age reaches the Character-specific max_lifespan_days, THE Engine SHALL set is_alive to false and stage to Dead
2. WHEN hunger and happiness are both 0 for 12 consecutive hours, THE Engine SHALL set is_alive to false and stage to Dead
3. WHEN the Pet is sick and untreated for 24 hours, THE Engine SHALL set is_alive to false and stage to Dead
4. WHEN snack_count exceeds 5 during the Baby stage, THE Engine SHALL set is_alive to false and stage to Dead

### Requirement 6: Persistence and State Recovery

**User Story:** As a player, I want my pet to persist across app restarts, so that I don't lose my progress.

#### Acceptance Criteria

1. THE Engine SHALL serialize PetState to JSON and write it atomically (write to temp file, then rename)
2. THE Engine SHALL deserialize PetState from JSON on load
3. WHEN loading a saved state, THE Engine SHALL apply Catch_Up by simulating elapsed ticks from last_tick to the current time
4. WHEN Catch_Up completes, THE Engine SHALL set last_tick equal to the current timestamp
5. THE Engine SHALL produce an equivalent PetState when serializing then deserializing (round-trip property)
6. IF the save file contains invalid JSON, THEN THE Engine SHALL back up the corrupt file, log a warning, and create a fresh Egg state
7. IF the system clock has moved backwards (now < last_tick), THEN THE Engine SHALL skip the Tick, log a warning, and set last_tick to the current time

### Requirement 7: Single-Instance Lockfile

**User Story:** As a player, I want only one frontend to access my pet at a time, so that state corruption is prevented.

#### Acceptance Criteria

1. WHEN a frontend starts, THE Engine SHALL attempt to acquire the Lockfile at ~/.tama96/tama96.lock
2. IF another process already holds the Lockfile, THEN THE Engine SHALL return an AlreadyLocked error with a descriptive message
3. WHEN a frontend exits, THE Engine SHALL release the Lockfile
4. THE Engine SHALL ensure at most one process holds the Lockfile at any time

### Requirement 8: Agent Permission Gating

**User Story:** As a pet owner, I want to control what AI agents can do with my pet, so that I retain full authority over my pet's care.

#### Acceptance Criteria

1. WHILE the Permission_System master switch is disabled, THE Permission_System SHALL deny all Agent actions with a MasterDisabled reason
2. WHEN an Agent attempts a disabled action, THE Permission_System SHALL return a PermissionDenied error with the action name and reason
3. WHEN an Agent exceeds the max_per_hour rate limit for an action, THE Permission_System SHALL deny the action with a RateLimited reason including the limit and usage count
4. THE Permission_System SHALL prune action_log entries older than 1 hour during each permission check
5. THE Permission_System SHALL persist permissions to ~/.tama96/permissions.json separately from pet state
6. WHEN an Agent reads the pet://permissions resource, THE MCP_Server SHALL return the current permission configuration

### Requirement 9: MCP Server Integration

**User Story:** As an AI agent developer, I want to interact with the pet through MCP tools and resources, so that agents can care for the pet programmatically.

#### Acceptance Criteria

1. THE MCP_Server SHALL register tools for feed, play_game, discipline, give_medicine, clean_poop, toggle_lights, and get_status
2. THE MCP_Server SHALL register resources for pet://status, pet://evolution-chart, and pet://permissions
3. WHEN an Agent calls an MCP tool, THE MCP_Server SHALL check permissions before executing the action
4. WHEN an MCP tool call succeeds, THE MCP_Server SHALL return the updated PetState as a JSON response
5. IF an MCP tool call is denied by the Permission_System, THEN THE MCP_Server SHALL return a structured error with a human-readable reason
6. THE MCP_Server SHALL communicate with the Desktop_App backend via a localhost-only TCP socket

### Requirement 10: System Tray Persistence

**User Story:** As a player, I want the desktop app to keep running in the background, so that my pet continues to live even when I close the window.

#### Acceptance Criteria

1. WHEN the player closes the Desktop_App window, THE Desktop_App SHALL hide the window and continue running in the system tray
2. WHILE the Desktop_App is in the system tray, THE Desktop_App SHALL continue running the Tick loop in the background
3. WHEN the player double-clicks the tray icon, THE Desktop_App SHALL restore and focus the window
4. THE Desktop_App SHALL display a tray menu with Show Window, Pet Status, and Quit options
5. WHEN hunger or happiness reaches 0, THE Desktop_App SHALL send a desktop notification

### Requirement 11: Desktop UI

**User Story:** As a player, I want a pixel-art desktop interface, so that I can interact with my pet visually.

#### Acceptance Criteria

1. THE Desktop_App SHALL render the Pet sprite with animation states (idle, eating, sleeping, happy)
2. THE Desktop_App SHALL display hunger hearts, happiness hearts, discipline gauge, age, and weight
3. THE Desktop_App SHALL provide action buttons matching the original P1 icon layout (Feed, Light, Game, Medicine, Bathroom, Meter, Discipline, Attention)
4. THE Desktop_App SHALL include a settings panel for configuring Agent permissions
5. WHEN the Pet dies, THE Desktop_App SHALL display a death screen with the final age and a Hatch New Egg option

### Requirement 12: Terminal UI

**User Story:** As a player, I want a terminal interface, so that I can care for my pet without a graphical environment.

#### Acceptance Criteria

1. THE Terminal_App SHALL render Pet sprites as braille characters using a 32x16 pixel grid
2. THE Terminal_App SHALL display hunger hearts, happiness hearts, and a discipline gauge using unicode widgets
3. THE Terminal_App SHALL map keyboard inputs to game actions (f=feed, g=game, d=discipline, c=clean, l=lights, i=medicine, q=quit)
4. THE Terminal_App SHALL run its own Tick loop using the shared Engine
5. WHEN the Terminal_App starts, THE Terminal_App SHALL acquire the Lockfile and load state with Catch_Up
6. WHEN the Terminal_App exits, THE Terminal_App SHALL save state and release the Lockfile

### Requirement 13: MCP Sidecar Lifecycle

**User Story:** As a player, I want the MCP server to be managed automatically, so that agent access is always available when the desktop app is running.

#### Acceptance Criteria

1. WHEN the Desktop_App starts, THE Desktop_App SHALL spawn the MCP_Server sidecar process
2. IF the MCP_Server process exits unexpectedly, THEN THE Desktop_App SHALL restart it with exponential backoff (2s, 4s, 8s, max 30s)
3. THE MCP_Server SHALL use stdio transport for Agent communication
4. THE MCP_Server SHALL bind its TCP bridge to 127.0.0.1 only
