# tama96

A faithful recreation of the 1996 Tamagotchi P1 virtual pet. Play on desktop (Tauri + React), in the terminal (ratatui TUI), or let an AI agent care for your pet via the bundled MCP server.

```
┌──────────────────────────────────────────────────┐
│  Egg → Babytchi → Marutchi → Teen → Adult → …   │
│  Real-time lifecycle: 1 real day = 1 pet year    │
│  Your care choices shape who your pet becomes     │
└──────────────────────────────────────────────────┘
```

## Architecture

```
tama-core/          Shared Rust library — game engine, state, evolution, persistence
tama-tauri/         Tauri v2 desktop app (system tray, background ticks, notifications)
  src-tauri/          Rust backend (IPC commands, TCP socket, sidecar management)
  ui/                 React frontend (pet display, action buttons, permissions panel)
tama-tui/           Terminal app — ratatui with braille sprites and keyboard controls
mcp-server/         Node.js MCP sidecar — exposes pet to AI agents via stdio
```

All frontends share `tama-core` for game logic. A lockfile at `~/.tama96/tama96.lock` ensures only one frontend accesses the pet at a time. State persists to `~/.tama96/state.json` with elapsed-time catch-up on load.

## Prerequisites

- Rust (edition 2024, stable toolchain)
- Node.js >= 18
- Tauri v2 system dependencies: https://v2.tauri.app/start/prerequisites/
- Tauri CLI: `cargo install tauri-cli --version "^2"`

## Getting Started

### Run Tests

```bash
cargo test -p tama-core
```

This runs all 102 tests — unit, property-based (proptest), and integration.

### Desktop App (Tauri)

```bash
# Install frontend dependencies
cd tama-tauri/ui && npm install && cd ..

# Run in development mode (from tama-tauri/)
npx @tauri-apps/cli dev
```

The app lives in the system tray when you close the window. Your pet keeps ticking in the background. Desktop notifications fire when hunger or happiness hits zero, on evolution, and on death.

### Terminal App (TUI)

```bash
cargo run -p tama-tui
```

Keybindings:

| Key | Action |
|-----|--------|
| `f` | Feed (then `m` for meal, `s` for snack) |
| `g` | Play game |
| `d` | Discipline |
| `c` | Clean poop |
| `l` | Toggle lights |
| `i` | Give medicine |
| `q` | Quit |

### MCP Server

```bash
cd mcp-server && npm install && npm run build
```

The MCP server is normally spawned automatically by the Tauri app as a sidecar. For standalone testing:

```bash
npm start
```

It communicates with the Tauri backend over a localhost TCP socket (port written to `~/.tama96/mcp_port`).

**MCP Tools:** `feed`, `play_game`, `discipline`, `give_medicine`, `clean_poop`, `toggle_lights`, `get_status`

**MCP Resources:** `pet://status`, `pet://evolution-chart`, `pet://permissions`

## Agent Permissions

AI agents are gated by a human-configurable permission system:

- Master kill switch (enable/disable all agent access)
- Per-action allow/deny toggles
- Per-action rate limits (max calls per hour)

Configure via the ⚙️ button in the desktop app or edit `~/.tama96/permissions.json` directly.

## Evolution Chart

```
Egg ──(5min)──► Babytchi ──(65min)──► Marutchi ──(age 3)──► Teen ──(age 6)──► Adult
                                                    │                           │
                                              care_mistakes              full P1 matrix
                                              0–2 → Tamatchi            based on teen char,
                                              3+  → Kuchitamatchi       teen type, care &
                                                                        discipline mistakes
Adult (Maskutchi from Tamatchi T2 path) ──(4 days)──► Oyajitchi
```

## Data Directory

All persistent data lives in `~/.tama96/`:

| File | Purpose |
|------|---------|
| `state.json` | Pet state (created on first run) |
| `permissions.json` | Agent permission config |
| `tama96.lock` | Single-instance lockfile |
| `mcp_port` | TCP port for MCP bridge |

## License

MIT
