mod sprites;

use std::io;
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
    persistence::{self, LockGuard},
    state::{LifeStage, PetState},
};

/// Tracks the current keyboard input mode for multi-key sequences.
#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    /// Normal mode — waiting for a top-level key.
    Normal,
    /// Feed sub-mode — waiting for 'm' (meal) or 's' (snack), Esc to cancel.
    Feed,
}

fn tama_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".tama96")
}

/// Build a hearts row: ♥ for filled, ♡ for empty, up to `max` hearts.
fn hearts_string(filled: u8, max: u8) -> String {
    let mut s = String::new();
    for i in 0..max {
        if i < filled {
            s.push('♥');
        } else {
            s.push('♡');
        }
    }
    s
}

/// Build the status indicators line (poop, sickness, sleep).
fn status_indicators(state: &PetState) -> String {
    let mut parts: Vec<String> = Vec::new();

    if state.poop_count > 0 {
        parts.push("💩".repeat(state.poop_count as usize));
    }
    if state.is_sick {
        parts.push("🤒".to_string());
    }
    if state.is_sleeping {
        parts.push("💤".to_string());
    }

    if parts.is_empty() {
        String::new()
    } else {
        parts.join(" ")
    }
}

/// Format the character name for display.
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

/// Format the life stage for display.
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

/// Render the full TUI layout into the given frame.
fn render_ui(frame: &mut Frame, state: &PetState, input_mode: &InputMode) {
    let outer = frame.area();

    // Main vertical layout:
    //   [Header: 1 line]  [Sprite: 6 lines]  [Meters: 5 lines]  [Status: 2 lines]  [Keybinds: 1 line]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header (name + stage)
            Constraint::Length(6),  // sprite area (4 braille lines + border)
            Constraint::Length(6),  // meters (hunger, happiness, discipline, age/weight)
            Constraint::Length(3),  // status indicators
            Constraint::Min(1),    // keybind hints
        ])
        .split(outer);

    // ── Header ──
    let header_text = if state.is_alive {
        format!(
            "{} — {} (Age {})",
            character_name(state),
            stage_name(&state.stage),
            state.age
        )
    } else {
        format!("{} — R.I.P. (Age {})", character_name(state), state.age)
    };
    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // ── Sprite ──
    let sprite_text = sprites::get_sprite(state);
    let sprite = Paragraph::new(sprite_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(sprite, chunks[1]);

    // ── Meters area ──
    let meter_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // hunger hearts
            Constraint::Length(1), // happiness hearts
            Constraint::Length(1), // discipline gauge
            Constraint::Length(1), // age + weight
        ])
        .margin(1)
        .split(chunks[2]);

    // Hunger hearts
    let hunger_hearts = hearts_string(state.hunger, 4);
    let hunger_line = Line::from(vec![
        Span::styled("Hunger:    ", Style::default().fg(Color::Yellow)),
        Span::styled(hunger_hearts, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(hunger_line), meter_chunks[0]);

    // Happiness hearts
    let happy_hearts = hearts_string(state.happiness, 4);
    let happy_line = Line::from(vec![
        Span::styled("Happiness: ", Style::default().fg(Color::Yellow)),
        Span::styled(happy_hearts, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(happy_line), meter_chunks[1]);

    // Discipline gauge
    let disc_ratio = state.discipline as f64 / 100.0;
    let disc_label = format!("Discipline: {}%", state.discipline);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(disc_ratio)
        .label(disc_label);
    frame.render_widget(gauge, meter_chunks[2]);

    // Age + Weight
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

    // ── Keybind hints ──
    let hints_text = match input_mode {
        InputMode::Feed => "Feed: [m]eal or [s]nack? (Esc to cancel)",
        InputMode::Normal => "f:feed  g:game  d:discipline  c:clean  l:lights  i:medicine  q:quit",
    };
    let hints = Paragraph::new(hints_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hints, chunks[4]);
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let base = tama_dir();
    let save_path = base.join("state.json");
    let lock_path = base.join("tama96.lock");

    // Initialize data directory and default files
    persistence::init_data_dir(&base)?;

    // 1. Acquire lockfile
    let _lock: LockGuard = persistence::acquire_lock(&lock_path)?;

    // 2. Load state with catch-up, or create a fresh egg
    let now = Utc::now();
    let mut state: PetState = if save_path.exists() {
        persistence::load(&save_path, now)?
    } else {
        PetState::new_egg(now)
    };

    // 3. Initialize terminal (raw mode + alternate screen)
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // 4. Main loop: poll keyboard with 1-second timeout, tick every 60 seconds
    let tick_interval = Duration::from_secs(60);
    let mut last_tick = Instant::now();
    let mut input_mode = InputMode::Normal;

    loop {
        // Render full UI
        terminal.draw(|frame| {
            render_ui(frame, &state, &input_mode);
        })?;

        // Poll for keyboard events with a 1-second timeout
        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                match &input_mode {
                    InputMode::Feed => {
                        match key.code {
                            KeyCode::Char('m') => {
                                let _ = actions::feed_meal(&mut state);
                                persistence::save(&state, &save_path)?;
                                input_mode = InputMode::Normal;
                            }
                            KeyCode::Char('s') => {
                                let _ = actions::feed_snack(&mut state);
                                persistence::save(&state, &save_path)?;
                                input_mode = InputMode::Normal;
                            }
                            KeyCode::Esc => {
                                input_mode = InputMode::Normal;
                            }
                            _ => {} // ignore other keys in feed sub-mode
                        }
                    }
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('f') => {
                                input_mode = InputMode::Feed;
                            }
                            KeyCode::Char('g') => {
                                let mut rng = rand::thread_rng();
                                let moves: [Choice; 5] = std::array::from_fn(|_| {
                                    if rng.gen_bool(0.5) {
                                        Choice::Left
                                    } else {
                                        Choice::Right
                                    }
                                });
                                let _ = actions::play_game(&mut state, moves);
                                persistence::save(&state, &save_path)?;
                            }
                            KeyCode::Char('d') => {
                                let _ = actions::discipline(&mut state);
                                persistence::save(&state, &save_path)?;
                            }
                            KeyCode::Char('c') => {
                                let _ = actions::clean_poop(&mut state);
                                persistence::save(&state, &save_path)?;
                            }
                            KeyCode::Char('l') => {
                                let _ = actions::toggle_lights(&mut state, Utc::now());
                                persistence::save(&state, &save_path)?;
                            }
                            KeyCode::Char('i') => {
                                let _ = actions::give_medicine(&mut state);
                                persistence::save(&state, &save_path)?;
                            }
                            _ => {} // ignore unmapped keys
                        }
                    }
                }
            }
        }

        // Tick every 60 seconds
        if last_tick.elapsed() >= tick_interval {
            engine::tick(&mut state, Utc::now());
            persistence::save(&state, &save_path)?;
            last_tick = Instant::now();
        }
    }

    // 5. On exit: save state, restore terminal (lockfile released via Drop)
    persistence::save(&state, &save_path)?;

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        // Attempt to restore terminal even on error
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        eprintln!("tama-tui error: {e}");
        std::process::exit(1);
    }
}
