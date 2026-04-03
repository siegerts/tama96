mod sprites;

use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use chrono::Utc;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame, Terminal,
};

use tama_core::{
    actions::{self, Choice},
    engine,
    persistence::{self, LockError, LockGuard},
    state::{LifeStage, PetState},
};

// ── Run mode ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum RunMode {
    /// Standalone: owns the lock, ticks, reads/writes state directly.
    Standalone,
    /// Client: app is running. Read state from disk, send actions via TCP.
    Client,
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Normal,
    Feed,
    About,
}

fn tama_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".tama96")
}

// ── TCP client for sending actions to the app ───────────────────────────────

fn read_port() -> Option<u16> {
    let port_path = tama_dir().join("mcp_port");
    std::fs::read_to_string(&port_path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

fn send_action(port: u16, action: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .map_err(|e| format!("connect failed: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("timeout error: {e}"))?;

    let req = if let Some(p) = params {
        serde_json::json!({"action": action, "params": p})
    } else {
        serde_json::json!({"action": action})
    };

    let mut line = serde_json::to_string(&req).map_err(|e| format!("serialize: {e}"))?;
    line.push('\n');
    stream.write_all(line.as_bytes()).map_err(|e| format!("write: {e}"))?;

    let mut reader = io::BufReader::new(&stream);
    let mut resp_line = String::new();
    reader.read_line(&mut resp_line).map_err(|e| format!("read: {e}"))?;

    let resp: serde_json::Value = serde_json::from_str(&resp_line)
        .map_err(|e| format!("parse response: {e}"))?;

    if resp.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        Ok(resp)
    } else {
        let err = resp.get("error").and_then(|v| v.as_str()).unwrap_or("unknown error");
        Err(err.to_string())
    }
}

// ── Display helpers ─────────────────────────────────────────────────────────

fn hearts_string(filled: u8, max: u8) -> String {
    let mut s = String::new();
    for i in 0..max {
        if i > 0 { s.push(' '); }
        if i < filled {
            s.push_str("\u{2588}\u{2588}");
        } else {
            s.push_str("\u{2591}\u{2591}");
        }
    }
    s
}

fn status_indicators(state: &PetState) -> String {
    let mut parts: Vec<String> = Vec::new();
    if state.poop_count > 0 {
        parts.push(format!("POOP:{}", "o".repeat(state.poop_count as usize)));
    }
    if state.pending_lights_deadline.is_some() {
        parts.push("BEDTIME".to_string());
    }
    if state.is_sick { parts.push("SICK".to_string()); }
    if state.is_sleeping { parts.push("ZZZ".to_string()); }
    if parts.is_empty() { String::new() } else { parts.join(" ") }
}

fn character_name(state: &PetState) -> &'static str {
    match state.character {
        tama_core::state::Character::Babytchi => "Babytchi",
        tama_core::state::Character::Marutchi => "Marutchi",
        tama_core::state::Character::Tamatchi => "Tamatchi",
        tama_core::state::Character::Kuchitamatchi => "Kuchitamatchi",
        tama_core::state::Character::Mametchi => "Mametchi",
        tama_core::state::Character::Ginjirotchi => "Ginjirotchi",
        tama_core::state::Character::Maskutchi => "Maskutchi",
        tama_core::state::Character::Kuchipatchi => "Kuchipatchi",
        tama_core::state::Character::Nyorotchi => "Nyorotchi",
        tama_core::state::Character::Tarakotchi => "Tarakotchi",
        tama_core::state::Character::Oyajitchi => "Oyajitchi",
    }
}

fn stage_name(stage: &LifeStage) -> &'static str {
    match stage {
        LifeStage::Egg => "Egg",
        LifeStage::Baby => "Baby",
        LifeStage::Child => "Child",
        LifeStage::Teen => "Teen",
        LifeStage::Adult => "Adult",
        LifeStage::Special => "Special",
        LifeStage::Dead => "Dead",
    }
}

// ── Rendering ───────────────────────────────────────────────────────────────

fn render_ui(
    frame: &mut Frame,
    state: &PetState,
    input_mode: &InputMode,
    anim_frame: u8,
    run_mode: &RunMode,
    status_msg: &str,
) {
    let outer = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Length(6),  // sprite
            Constraint::Length(6),  // meters
            Constraint::Length(3),  // status
            Constraint::Length(1),  // mode indicator
            Constraint::Min(1),    // keybinds / messages
        ])
        .split(outer);

    // ── Header ──
    let header_text = if state.is_alive {
        format!(
            "{} — {} (Age {})",
            character_name(state), stage_name(&state.stage), state.age
        )
    } else {
        format!("{} — R.I.P. (Age {})", character_name(state), state.age)
    };
    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // ── Sprite with idle animation ──
    let sprite_text = sprites::get_sprite(state);
    let animated_sprite = if state.is_alive && !state.is_sleeping && !state.is_sick {
        let pad = match anim_frame % 4 {
            0 => "", 1 => " ", 2 => "", _ => "  ",
        };
        sprite_text.lines().map(|l| format!("{}{}", pad, l)).collect::<Vec<_>>().join("\n")
    } else {
        sprite_text.clone()
    };
    let sprite = Paragraph::new(animated_sprite)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(sprite, chunks[1]);

    // ── Meters ──
    let meter_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(chunks[2]);

    let hunger_line = Line::from(vec![
        Span::styled("Hunger:    ", Style::default().fg(Color::Yellow)),
        Span::styled(hearts_string(state.hunger, 4), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(hunger_line), meter_chunks[0]);

    let happy_line = Line::from(vec![
        Span::styled("Happiness: ", Style::default().fg(Color::Yellow)),
        Span::styled(hearts_string(state.happiness, 4), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(happy_line), meter_chunks[1]);

    let disc_ratio = state.discipline as f64 / 100.0;
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(disc_ratio)
        .label(format!("Discipline: {}%", state.discipline));
    frame.render_widget(gauge, meter_chunks[2]);

    let info_line = Line::from(vec![
        Span::styled(
            format!("Age: {} yr   Weight: {} lb", state.age, state.weight),
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(info_line), meter_chunks[3]);

    // ── Status indicators ──
    let indicators = status_indicators(state);
    let status_text = if indicators.is_empty() {
        "Status: OK".to_string()
    } else {
        format!("Status: {}", indicators)
    };
    let status = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(status, chunks[3]);

    // ── Mode indicator ──
    let mode_text = match run_mode {
        RunMode::Standalone => "[ standalone — owns simulation ]",
        RunMode::Client => "[ client — app is running, actions sent via network ]",
    };
    let mode_color = match run_mode {
        RunMode::Standalone => Color::Green,
        RunMode::Client => Color::Cyan,
    };
    let mode_line = Paragraph::new(mode_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(mode_color));
    frame.render_widget(mode_line, chunks[4]);

    // ── Keybind hints / status message ──
    let hints_text = if !status_msg.is_empty() {
        status_msg.to_string()
    } else {
        match input_mode {
            InputMode::Feed => "Feed: [m]eal or [s]nack? (Esc to cancel)".to_string(),
            InputMode::About => "Press Esc to close".to_string(),
            InputMode::Normal => {
                if state.pending_lights_deadline.is_some() {
                    "BEDTIME: press l to turn the lights off".to_string()
                } else {
                    "f:feed  g:game  d:discipline  c:clean  l:lights  i:med  a:about  q:quit".to_string()
                }
            }
        }
    };
    let hints = Paragraph::new(hints_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hints, chunks[5]);

    // ── About overlay ──
    if *input_mode == InputMode::About {
        let about_text = vec![
            Line::from(""),
            Line::from(Span::styled("tama96 v0.1.0", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Made by @siegerts", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled("https://x.com/siegerts", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("Built with Kiro", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled("https://kiro.dev/", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("https://github.com/siegerts/tama96", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("\"Tamagotchi\" is a trademark of Bandai Co., Ltd.", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled("Not affiliated with or endorsed by Bandai.", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("MIT License", Style::default().fg(Color::DarkGray))),
        ];
        let about = Paragraph::new(about_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" About "));
        // Render over the sprite + meters area
        let about_area = ratatui::layout::Rect {
            x: outer.x + 2,
            y: outer.y + 1,
            width: outer.width.saturating_sub(4),
            height: outer.height.saturating_sub(3),
        };
        frame.render_widget(ratatui::widgets::Clear, about_area);
        frame.render_widget(about, about_area);
    }
}

// ── Main run loop ───────────────────────────────────────────────────────────

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let base = tama_dir();
    let save_path = base.join("state.json");
    let lock_path = base.join("tama96.lock");

    persistence::init_data_dir(&base)?;

    // Try to acquire lock — determines run mode
    let (run_mode, _lock_guard): (RunMode, Option<LockGuard>) = match persistence::acquire_lock(&lock_path) {
        Ok(guard) => (RunMode::Standalone, Some(guard)),
        Err(LockError::AlreadyLocked(_)) => (RunMode::Client, None),
        Err(LockError::Io(e)) => return Err(format!("lockfile error: {e}").into()),
    };

    // In client mode, find the app's TCP port
    let client_port: Option<u16> = if run_mode == RunMode::Client {
        read_port()
    } else {
        None
    };

    // Load initial state
    let now = Utc::now();
    let mut state: PetState = if save_path.exists() {
        persistence::load(&save_path, now).unwrap_or_else(|_| PetState::new_egg(now))
    } else {
        PetState::new_egg(now)
    };

    // Initialize terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let tick_interval = Duration::from_secs(60);
    let mut last_tick = Instant::now();
    let mut last_file_reload = Instant::now();
    let mut input_mode = InputMode::Normal;
    let mut anim_frame: u8 = 0;
    let mut status_msg = String::new();
    let mut status_msg_time: Option<Instant> = None;

    // Helper to show a temporary status message
    let set_status = |msg: &str, status_msg: &mut String, status_msg_time: &mut Option<Instant>| {
        *status_msg = msg.to_string();
        *status_msg_time = Some(Instant::now());
    };

    loop {
        // Clear expired status messages (after 3 seconds)
        if let Some(t) = status_msg_time {
            if t.elapsed() > Duration::from_secs(3) {
                status_msg.clear();
                status_msg_time = None;
            }
        }

        // Render
        terminal.draw(|frame| {
            render_ui(frame, &state, &input_mode, anim_frame, &run_mode, &status_msg);
        })?;

        // Poll keyboard
        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                match (&input_mode, &run_mode) {
                    // ── About mode ──
                    (InputMode::About, _) => match key.code {
                        KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('q') => {
                            input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                    // ── Feed sub-mode ──
                    (InputMode::Feed, RunMode::Standalone) => match key.code {
                        KeyCode::Char('m') => {
                            match actions::feed_meal(&mut state) {
                                Ok(_) => { persistence::save(&state, &save_path)?; }
                                Err(e) => set_status(&format!("{e:?}"), &mut status_msg, &mut status_msg_time),
                            }
                            input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('s') => {
                            match actions::feed_snack(&mut state) {
                                Ok(_) => { persistence::save(&state, &save_path)?; }
                                Err(e) => set_status(&format!("{e:?}"), &mut status_msg, &mut status_msg_time),
                            }
                            input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => { input_mode = InputMode::Normal; }
                        _ => {}
                    },
                    (InputMode::Feed, RunMode::Client) => match key.code {
                        KeyCode::Char('m') => {
                            if let Some(port) = client_port {
                                match send_action(port, "feed_meal", None) {
                                    Ok(_) => set_status("meal sent", &mut status_msg, &mut status_msg_time),
                                    Err(e) => set_status(&e, &mut status_msg, &mut status_msg_time),
                                }
                            } else {
                                set_status("no app connection", &mut status_msg, &mut status_msg_time);
                            }
                            input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('s') => {
                            if let Some(port) = client_port {
                                match send_action(port, "feed_snack", None) {
                                    Ok(_) => set_status("snack sent", &mut status_msg, &mut status_msg_time),
                                    Err(e) => set_status(&e, &mut status_msg, &mut status_msg_time),
                                }
                            } else {
                                set_status("no app connection", &mut status_msg, &mut status_msg_time);
                            }
                            input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => { input_mode = InputMode::Normal; }
                        _ => {}
                    },
                    // ── Normal mode ──
                    (InputMode::Normal, _) => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('a') => { input_mode = InputMode::About; }
                        KeyCode::Char('f') => { input_mode = InputMode::Feed; }
                        KeyCode::Char('g') => {
                            let mut rng = rand::thread_rng();
                            let moves: [Choice; 5] = std::array::from_fn(|_| {
                                if rng.gen_bool(0.5) { Choice::Left } else { Choice::Right }
                            });
                            match &run_mode {
                                RunMode::Standalone => {
                                    match actions::play_game(&mut state, moves) {
                                        Ok(r) => {
                                            persistence::save(&state, &save_path)?;
                                            set_status(&format!("game: {}/{} wins", r.wins, r.rounds), &mut status_msg, &mut status_msg_time);
                                        }
                                        Err(e) => set_status(&format!("{e:?}"), &mut status_msg, &mut status_msg_time),
                                    }
                                }
                                RunMode::Client => {
                                    if let Some(port) = client_port {
                                        let params = serde_json::json!({"moves": moves});
                                        match send_action(port, "play_game", Some(params)) {
                                            Ok(_) => set_status("game played", &mut status_msg, &mut status_msg_time),
                                            Err(e) => set_status(&e, &mut status_msg, &mut status_msg_time),
                                        }
                                    } else {
                                        set_status("no app connection", &mut status_msg, &mut status_msg_time);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('d') => {
                            handle_simple_action("discipline", &run_mode, client_port, &mut state, &save_path,
                                |s| actions::discipline(s), &mut status_msg, &mut status_msg_time);
                        }
                        KeyCode::Char('c') => {
                            handle_simple_action("clean_poop", &run_mode, client_port, &mut state, &save_path,
                                |s| actions::clean_poop(s), &mut status_msg, &mut status_msg_time);
                        }
                        KeyCode::Char('l') => {
                            match &run_mode {
                                RunMode::Standalone => {
                                    match actions::toggle_lights(&mut state, Utc::now()) {
                                        Ok(_) => { persistence::save(&state, &save_path)?; }
                                        Err(e) => set_status(&format!("{e:?}"), &mut status_msg, &mut status_msg_time),
                                    }
                                }
                                RunMode::Client => {
                                    if let Some(port) = client_port {
                                        match send_action(port, "toggle_lights", None) {
                                            Ok(_) => set_status("lights toggled", &mut status_msg, &mut status_msg_time),
                                            Err(e) => set_status(&e, &mut status_msg, &mut status_msg_time),
                                        }
                                    } else {
                                        set_status("no app connection", &mut status_msg, &mut status_msg_time);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('i') => {
                            handle_simple_action("give_medicine", &run_mode, client_port, &mut state, &save_path,
                                |s| actions::give_medicine(s), &mut status_msg, &mut status_msg_time);
                        }
                        _ => {}
                    },
                }
            }
        }

        // Standalone: tick every 60s
        if run_mode == RunMode::Standalone && last_tick.elapsed() >= tick_interval {
            engine::tick(&mut state, Utc::now());
            persistence::save(&state, &save_path)?;
            last_tick = Instant::now();
        }

        // Client: reload state from disk every second
        if run_mode == RunMode::Client && last_file_reload.elapsed() >= Duration::from_secs(1) {
            if let Ok(reloaded) = persistence::load(&save_path, Utc::now()) {
                state = reloaded;
            }
            last_file_reload = Instant::now();
        }

        anim_frame = anim_frame.wrapping_add(1);
    }

    // Cleanup
    if run_mode == RunMode::Standalone {
        persistence::save(&state, &save_path)?;
    }
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

/// Handle a simple action (no extra params) in both modes.
fn handle_simple_action(
    action_name: &str,
    run_mode: &RunMode,
    client_port: Option<u16>,
    state: &mut PetState,
    save_path: &std::path::Path,
    standalone_fn: impl FnOnce(&mut PetState) -> Result<actions::ActionResult, actions::ActionError>,
    status_msg: &mut String,
    status_msg_time: &mut Option<Instant>,
) {
    let set_status = |msg: &str, sm: &mut String, smt: &mut Option<Instant>| {
        *sm = msg.to_string();
        *smt = Some(Instant::now());
    };

    match run_mode {
        RunMode::Standalone => {
            match standalone_fn(state) {
                Ok(_) => {
                    if let Err(e) = persistence::save(state, save_path) {
                        set_status(&format!("save error: {e}"), status_msg, status_msg_time);
                    }
                }
                Err(e) => set_status(&format!("{e:?}"), status_msg, status_msg_time),
            }
        }
        RunMode::Client => {
            if let Some(port) = client_port {
                match send_action(port, action_name, None) {
                    Ok(_) => set_status(&format!("{} done", action_name), status_msg, status_msg_time),
                    Err(e) => set_status(&e, status_msg, status_msg_time),
                }
            } else {
                set_status("no app connection", status_msg, status_msg_time);
            }
        }
    }
}

fn main() {
    if let Err(e) = run() {
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        eprintln!("tama-tui error: {e}");
        std::process::exit(1);
    }
}
