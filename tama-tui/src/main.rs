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
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
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
    Game,
    Score(actions::GameProgress),
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
    (0..max)
        .map(|i| if i < filled { "■" } else { "□" })
        .collect::<Vec<_>>()
        .join(" ")
}

fn status_indicators(state: &PetState) -> String {
    let mut parts: Vec<String> = Vec::new();
    if state.hunger == 0 { parts.push("Hungry".into()); }
    if state.happiness == 0 { parts.push("Wants to play".into()); }
    if state.pending_discipline_deadline.is_some() { parts.push("Calling".into()); }
    if state.poop_count > 0 {
        parts.push(format!("Clean up ({})", state.poop_count));
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

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(area.x + (area.width - width) / 2, area.y + (area.height - height) / 2, width, height)
}

fn render_ui(
    frame: &mut Frame,
    state: &PetState,
    input_mode: &InputMode,
    pose: &sprites::PetFrame,
    run_mode: &RunMode,
    status_msg: &str,
) {
    let outer = frame.area();
    let wide = outer.width >= 52 && outer.height >= 16;
    let area = centered(outer, 60, if wide { 18 } else { 19 });
    let [header, body, status, keys] = Layout::vertical([
        Constraint::Length(1), Constraint::Min(3), Constraint::Length(2), Constraint::Length(if area.width < 42 { 4 } else { 3 })
    ]).areas(area);
    let mode = match run_mode {
        RunMode::Standalone => "Standalone",
        RunMode::Client => "Desktop client",
    };
    let bold = Style::default().add_modifier(Modifier::BOLD);
    frame.render_widget(Paragraph::new(" tama96").style(bold), header);
    frame.render_widget(Paragraph::new(format!("{mode} ")).alignment(Alignment::Right), header);

    let lcd = Style::default().fg(Color::Rgb(45, 51, 32)).bg(Color::Rgb(192, 203, 155));
    frame.render_widget(Block::default().style(lcd), body);
    let name = if state.stage == LifeStage::Egg { "Egg" } else { character_name(state) };
    let subtitle = if state.is_alive {
        format!("{} · age {} · {} lb", stage_name(&state.stage), state.age, state.weight)
    } else {
        format!("R.I.P. · age {}", state.age)
    };
    let meter = |label, value| Line::from(format!("{label:<10}{}", hearts_string(value, 4)));
    let show_score = matches!(input_mode, InputMode::Score(_));
    let game = match input_mode {
        InputMode::Score(finished) => finished,
        _ => &state.game,
    };
    let playing = matches!(input_mode, InputMode::Game | InputMode::Score(_));
    let round = game.turn % 5;
    let info = if playing {
        let heading = if show_score { "Final score".to_owned() }
            else { format!("Round {} of 5", round + 1) };
        let hits = if *input_mode == InputMode::Game && round == 0 { 0 } else { game.hits };
        let last = if show_score {
            format!("{} · {}", if game.look == Some(Choice::Left) { "← Left" } else { "Right →" }, if game.matched { "Match!" } else { "Miss" })
        } else { String::new() };
        vec![Line::styled("Left or right?", bold), Line::from(heading), Line::from(last),
             Line::from(format!("Matches    {hits}/5")), Line::from(format!("Stars      {}", game.stars)),
             Line::from(format!("Best       {}/5", game.best))]
    } else { vec![
        Line::styled(name, bold), Line::from(subtitle), Line::from(""),
        meter("Hunger", state.hunger), meter("Happiness", state.happiness),
        Line::from(format!("Discipline  {}%", state.discipline)),
    ] };
    let facing_right = playing && (round != 0 || show_score) && game.look == Some(Choice::Right);
    if wide {
        let [left, right] = Layout::horizontal([Constraint::Percentage(45), Constraint::Fill(1)]).areas(body);
        let sprite = Paragraph::new(pose.render(true, facing_right)).style(lcd).alignment(Alignment::Center);
        frame.render_widget(sprite, centered(left, left.width, 8));
        frame.render_widget(Paragraph::new(info).style(lcd), centered(right, right.width, 6));
    } else {
        let [sprite, details] = Layout::vertical([Constraint::Max(4), Constraint::Min(4)]).areas(body);
        frame.render_widget(Paragraph::new(pose.render(false, facing_right)).style(lcd).alignment(Alignment::Center), sprite);
        frame.render_widget(Paragraph::new(info).style(lcd), centered(details, 26, 6));
    }

    let conditions = status_indicators(state);
    let message = if !status_msg.is_empty() {
        status_msg.to_owned()
    } else if playing {
        if show_score {
            format!("{} stars earned · {}", game.hits,
                if game.stars >= 3 { "3D unlocked in desktop Game" } else { "3 stars unlock 3D" })
        } else if round != 0 {
            format!("Looked {} · {}", if game.look == Some(Choice::Left) { "left ←" } else { "right →" },
                if game.matched { "Match!" } else { "Miss" })
        } else { "Which way will your pet look?".into() }
    } else if !state.is_alive {
        "Your pet has passed away.".to_owned()
    } else if state.pending_lights_deadline.is_some() {
        format!("{conditions} · Press l for lights")
    } else if !conditions.is_empty() {
        conditions
    } else if state.stage == LifeStage::Egg {
        "Waiting to hatch".to_owned()
    } else {
        "Feeling well".to_owned()
    };
    frame.render_widget(Paragraph::new(message).alignment(Alignment::Center).wrap(Wrap { trim: true }), status);
    let key = |letter: &'static str, action: &'static str| [Span::styled(letter, bold), Span::raw(action)];
    let hints = match input_mode {
        InputMode::Feed => vec![Line::from("Feed your pet"), Line::from([key("m", " meal   "), key("s", " snack   "), key("Esc", " cancel")].concat())],
        InputMode::Game => vec![Line::from("← left     → right"), Line::from("Esc to pause")],
        InputMode::Score(_) => vec![Line::from("Enter to play again"), Line::from("Esc to close")],
        InputMode::About => vec![Line::from("Esc to close")],
        InputMode::Normal => vec![
            Line::from([key("f", " feed   "), key("g", " game   "), key("c", " clean   "), key("i", " medicine")].concat()),
            Line::from([key("l", " lights   "), key("d", " discipline   "), key("a", " about   "), key("q", " quit")].concat()),
        ],
    };
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Center).wrap(Wrap { trim: true }), keys);

    // ── About overlay ──
    if *input_mode == InputMode::About {
        let about_text = vec![
            Line::from(""),
            Line::from(Span::styled(concat!("tama96 v", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Made by @siegerts", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled("https://x.com/siegerts", Style::default().fg(Color::DarkGray))),
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
    let mut motion_start = Instant::now();
    let mut last_stage = state.stage_start_time;
    let mut last_weight = state.weight;
    let mut last_bite: Option<Instant> = None;
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

        if state.stage_start_time != last_stage || state.is_sleeping || state.is_sick {
            last_bite = None;
            motion_start = Instant::now();
        } else if state.weight > last_weight {
            // Both local and remote meals/snacks increase weight, even at full hearts.
            last_bite = Some(Instant::now());
        }
        last_stage = state.stage_start_time;
        last_weight = state.weight;
        let bite_left = last_bite.map_or(Duration::ZERO, |t| {
            Duration::from_millis(450).saturating_sub(t.elapsed())
        });
        let pose = sprites::pet_frame(&state, motion_start.elapsed(), bite_left);
        let until_update = match run_mode {
            RunMode::Standalone => tick_interval.saturating_sub(last_tick.elapsed()),
            RunMode::Client => Duration::from_secs(1).saturating_sub(last_file_reload.elapsed()),
        };
        let wait = pose.redraw_after.min(until_update);

        // Render
        terminal.draw(|frame| {
            render_ui(frame, &state, &input_mode, &pose, &run_mode, &status_msg);
        })?;

        // Poll keyboard
        if event::poll(wait)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != event::KeyEventKind::Press { continue; }
                // The desktop can publish its port after this terminal has opened.
                let client_port = if run_mode == RunMode::Client { read_port() } else { None };
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
                                Ok(_) => {
                                    persistence::save(&state, &save_path)?;
                                    set_status("Meal eaten", &mut status_msg, &mut status_msg_time);
                                }
                                Err(e) => set_status(&format!("{e:?}"), &mut status_msg, &mut status_msg_time),
                            }
                            input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('s') => {
                            match actions::feed_snack(&mut state) {
                                Ok(_) => {
                                    persistence::save(&state, &save_path)?;
                                    set_status("Snack eaten", &mut status_msg, &mut status_msg_time);
                                }
                                Err(e) => set_status(&format!("{e:?}"), &mut status_msg, &mut status_msg_time),
                            }
                            input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => { input_mode = InputMode::Normal; }
                        _ => {}
                    },
                    (InputMode::Feed, RunMode::Client) => match key.code {
                        KeyCode::Char(key @ ('m' | 's')) => {
                            let (action, message) = if key == 'm' {
                                ("feed_meal", "meal sent")
                            } else {
                                ("feed_snack", "snack sent")
                            };
                            if let Some(port) = client_port {
                                match send_action(port, action, None) {
                                    Ok(mut response) => {
                                        if let Ok(snapshot) = serde_json::from_value(response["state"].take()) {
                                            state = snapshot;
                                        }
                                        set_status(message, &mut status_msg, &mut status_msg_time);
                                    }
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
                    (InputMode::Game, _) => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => { input_mode = InputMode::Normal; }
                        KeyCode::Left | KeyCode::Right => {
                            let guess = actions::Guess {
                                choice: if key.code == KeyCode::Left { Choice::Left } else { Choice::Right },
                                birth_time: state.birth_time,
                                turn: state.game.turn,
                            };
                            let outcome = match run_mode {
                                RunMode::Standalone => {
                                    let mut next = state.clone();
                                    actions::guess_direction(&mut next, guess).map_err(|e| format!("{e:?}"))
                                        .and_then(|_| persistence::save(&next, &save_path).map(|_| next).map_err(|e| e.to_string()))
                                }
                                RunMode::Client => client_port.ok_or_else(|| "No desktop connection".to_owned())
                                    .and_then(|port| send_action(port, "guess_direction", Some(serde_json::json!(guess))))
                                    .and_then(|mut response| serde_json::from_value(response["state"].take()).map_err(|e| e.to_string())),
                            };
                            match outcome {
                                Ok(next) => {
                                    state = next;
                                    status_msg.clear();
                                    if state.game.turn % 5 == 0 { input_mode = InputMode::Score(state.game.clone()); }
                                }
                                Err(e) => set_status(&e, &mut status_msg, &mut status_msg_time),
                            }
                        }
                        _ => {}
                    },
                    (InputMode::Score(_), _) => match key.code {
                        KeyCode::Enter => { input_mode = InputMode::Game; }
                        KeyCode::Esc | KeyCode::Char('q') => { input_mode = InputMode::Normal; }
                        _ => {}
                    },
                    // ── Normal mode ──
                    (InputMode::Normal, _) => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('a') => { input_mode = InputMode::About; }
                        KeyCode::Char('f') => { input_mode = InputMode::Feed; }
                        KeyCode::Char('g') => {
                            status_msg.clear();
                            input_mode = InputMode::Game;
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

#[cfg(test)]
mod layout_tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn feed_choices_remain_visible_during_action_feedback() {
        let pet = PetState::new_egg(Utc::now());
        let pose = sprites::pet_frame(&pet, Duration::ZERO, Duration::ZERO);
        for (width, height) in [(72, 22), (42, 18), (32, 16)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| {
                render_ui(frame, &pet, &InputMode::Feed, &pose, &RunMode::Client, "Meal eaten");
            }).unwrap();
            let screen = terminal.backend().buffer().content.chunks(width as usize)
                .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
                .collect::<Vec<_>>().join("\n");
            for text in ["Meal eaten", "meal", "snack", "cancel", "Desktop client"] {
                assert!(screen.contains(text), "{width}x{height} omitted {text}");
            }
        }
    }
}
