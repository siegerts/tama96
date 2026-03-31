# tama96

A virtual pet inspired by the 1996 Tamagotchi P1. Play on desktop (Tauri + React), in the terminal (ratatui TUI), or let an AI agent care for your pet via the bundled MCP server.

```
┌──────────────────────────────────────────────────┐
│  Egg → Babytchi → Marutchi → Teen → Adult → …    │
│  Real-time lifecycle: 1 real day = 1 pet year    │
│  Your care choices shape who your pet becomes    │
└──────────────────────────────────────────────────┘
```

## How it works

Your pet lives in real time. Every minute the simulation ticks — hunger and happiness decay, poop accumulates, sickness can set in. One real day equals one pet year. Leave your pet alone too long and it dies. Take good care of it and it evolves into a better character.

The LCD screen has two rows of icons you can click (or tap the keyboard shortcuts in the TUI). Here's what they all do:

### Top row

| Icon | Name | What it does |
|------|------|-------------|
| Fork/knife | Feed | Opens a submenu: Meal restores 1 hunger heart (+1 weight), Snack restores 1 happiness heart (+2 weight). Can't feed a sick pet. |
| Diamond | Light | Toggles lights on/off. Turn lights off at bedtime so your pet can sleep. If you leave them on too long after bedtime, you get a care mistake. |
| Ball | Game | Plays a left/right guessing game (5 rounds, random). Win 3+ rounds to gain 1 happiness heart. Also burns 1 weight. |
| Cross | Medicine | Gives one dose of medicine. Takes 2 doses to cure sickness. Only works when the pet is actually sick. |

### Bottom row

| Icon | Name | What it does |
|------|------|-------------|
| Bathtub | Clean | Removes one poop. Poop accumulates over time based on your character's poop interval. 4+ poops makes your pet sick. |
| Meter | Stats | Shows current hunger, happiness, and weight. Just an info check, no action. |
| Person | Discipline | Scolds your pet when it's acting out. A discipline call appears randomly (~10% chance per tick) with a 15-minute deadline. Miss it and you get a discipline mistake. More discipline = better evolution outcomes. |
| Exclamation | Attention | Shows what needs your attention right now — sick, poop, pending discipline call. Another info-only button. |

### Meters and hearts

- Hunger and happiness each have 4 hearts. They decay over time at rates that depend on your character (harder characters decay faster).
- When either meter hits 0, a 15-minute care deadline starts. Miss it = care mistake.
- Discipline is a percentage (0–100%). Goes up by 25% each time you successfully discipline.

### Sickness

Your pet gets sick from:
- Too much poop (4+ poops)
- Overfeeding snacks as a baby (3+ snacks)

A sick pet can't eat or play. Give 2 doses of medicine to cure it. Leave it untreated too long and it dies.

### Death

Your pet can die from:
- Old age (each character has a max lifespan)
- Neglect (hunger AND happiness at 0 for 12+ hours)
- Untreated sickness
- Baby snack overfeeding (5+ snacks)

When your pet dies, you can hatch a new egg from the death screen.

### Sleep

Each character has a bedtime and wake time. When bedtime hits:
1. If lights are on, you get a 15-minute window to turn them off
2. Once lights are off, the pet sleeps (no decay, no actions)
3. At wake time, the pet wakes up and age increments by 1

## Evolution

How you care for your pet determines what it becomes. Care mistakes and discipline mistakes during each stage feed into the evolution matrix:

```
Egg ──(5min)──► Babytchi ──(65min)──► Marutchi ──(age 3)──► Teen ──(age 6)──► Adult

Teen selection:
  care_mistakes 0–2 → Tamatchi
  care_mistakes 3+  → Kuchitamatchi
  discipline_mistakes 0–2 → Type1
  discipline_mistakes 3+  → Type2

Adult selection (full P1 matrix):
  Tamatchi T1 + good care → Mametchi (best)
  Tamatchi T1 + ok care   → Ginjirotchi
  Tamatchi T1 + poor care → Maskutchi / Kuchipatchi / Nyorotchi / Tarakotchi
  Kuchitamatchi path      → Kuchipatchi / Nyorotchi / Tarakotchi

Special: Maskutchi (from Tamatchi T2 path) ──(4 days)──► Oyajitchi
```

## Architecture

```
tama-core/          Shared Rust library — game engine, state, evolution, persistence
tama-tauri/         Tauri v2 desktop app (system tray, background ticks, notifications)
  src-tauri/          Rust backend (IPC commands, TCP socket, sidecar management)
  ui/                 React frontend (LCD canvas, clickable icons, permissions panel)
tama-tui/           Terminal app — ratatui with braille sprites and keyboard controls
mcp-server/         Node.js MCP sidecar — exposes pet to AI agents via stdio
```

All frontends share `tama-core` for game logic. A lockfile at `~/.tama96/tama96.lock` ensures only one process owns the simulation at a time. State persists to `~/.tama96/state.json` with elapsed-time catch-up on load.

The TUI can run alongside the desktop app in client mode — it reads state from disk and sends actions over TCP instead of owning the simulation directly. A status line at the bottom tells you which mode you're in.

## Prerequisites

- Rust (edition 2024, stable toolchain)
- Node.js >= 18
- Tauri v2 system dependencies: https://v2.tauri.app/start/prerequisites/
- Tauri CLI: `cargo install tauri-cli --version "^2"`

## Getting started

### Tests

```bash
cargo test -p tama-core
```

### Desktop app (Tauri)

```bash
cd tama-tauri/ui && npm install && cd ../..
cd tama-tauri && npx @tauri-apps/cli dev
```

The app lives in the system tray when you close the window. Your pet keeps ticking in the background. Notifications fire on hunger/happiness hitting zero, evolution, and death.

Click the LCD icons directly to interact. Hover for tooltips. The info line below the LCD shows your pet's name, age, and weight — hover it for detailed timing info (next hunger drop, next poop, evolution countdown).

### Terminal app (TUI)

```bash
cargo run -p tama-tui
```

If the desktop app is already running, the TUI enters client mode automatically — it shows your pet's state and sends actions to the app over the network. No simulation conflicts.

| Key | Action |
|-----|--------|
| `f` | Feed (then `m` for meal, `s` for snack) |
| `g` | Play game |
| `d` | Discipline |
| `c` | Clean poop |
| `l` | Toggle lights |
| `i` | Give medicine |
| `q` | Quit |

### MCP server

```bash
cd mcp-server && npm install && npm run build
```

Normally spawned automatically by the Tauri app as a sidecar. For standalone testing: `npm start`

Communicates with the Tauri backend over a localhost TCP socket (port written to `~/.tama96/mcp_port`).

Tools: `feed`, `play_game`, `discipline`, `give_medicine`, `clean_poop`, `toggle_lights`, `get_status`

Resources: `pet://status`, `pet://evolution-chart`, `pet://permissions`

## Agent permissions

AI agents are gated by a permission system you control:

- Master kill switch (enable/disable all agent access)
- Per-action allow/deny toggles
- Per-action rate limits (max calls per hour)

Configure via the "settings" button in the desktop app or edit `~/.tama96/permissions.json` directly. The settings panel explains what "agent" means — it's an AI tool connected via MCP that can perform actions on your pet's behalf.

## Data directory

Everything lives in `~/.tama96/`:

| File | Purpose |
|------|---------|
| `state.json` | Pet state (created on first run) |
| `permissions.json` | Agent permission config |
| `tama96.lock` | Single-instance lockfile |
| `mcp_port` | TCP port for MCP bridge |

## Disclaimer

This is an independent hobby project. It is not affiliated with, endorsed by, or connected to Bandai Co., Ltd. in any way. "Tamagotchi" is a registered trademark of Bandai. This project is a clean-room reimplementation inspired by the original gameplay mechanics — no original code, assets, or ROM data from Bandai products were used.

## License

MIT
