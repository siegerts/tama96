// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod sidecar;
mod socket;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

use tauri_plugin_notification::NotificationExt;

use tama_core::engine;
use tama_core::permissions;
use tama_core::persistence;
use tama_core::state::{AgentPermissions, PetState};

fn main() {
    let tama_dir = dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".tama96");

    // ── Initialize data directory and default files ─────────────────────
    if let Err(e) = persistence::init_data_dir(&tama_dir) {
        eprintln!("tama96: failed to initialize data directory: {e}");
        std::process::exit(1);
    }

    // ── Lockfile: single-instance guard ─────────────────────────────────
    let lock_path = tama_dir.join("tama96.lock");
    let _lock_guard = match persistence::acquire_lock(&lock_path) {
        Ok(guard) => guard,
        Err(persistence::LockError::AlreadyLocked(msg)) => {
            eprintln!("tama96: {msg}");
            std::process::exit(1);
        }
        Err(persistence::LockError::Io(e)) => {
            eprintln!("tama96: failed to acquire lockfile: {e}");
            std::process::exit(1);
        }
    };

    let save_path = tama_dir.join("state.json");
    let permissions_path = tama_dir.join("permissions.json");

    let now = Utc::now();

    // Load or create pet state
    let pet_state = if save_path.exists() {
        persistence::load(&save_path, now).unwrap_or_else(|_| PetState::new_egg(now))
    } else {
        let state = PetState::new_egg(now);
        let _ = persistence::save(&state, &save_path);
        state
    };

    // Load or create agent permissions
    let agent_permissions = if permissions_path.exists() {
        permissions::load_permissions(&permissions_path)
            .unwrap_or_else(|_| AgentPermissions::default())
    } else {
        let perms = AgentPermissions::default();
        let _ = permissions::save_permissions(&perms, &permissions_path);
        perms
    };

    let shared_pet: commands::SharedPetState = Arc::new(Mutex::new(pet_state));
    let shared_perms: commands::SharedPermissions = Arc::new(Mutex::new(agent_permissions));

    // Clone for the tick loop and tray setup (moved into .setup())
    let tick_pet = Arc::clone(&shared_pet);
    let tick_save_path = save_path.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .manage(shared_pet)
        .manage(shared_perms)
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::feed_meal,
            commands::feed_snack,
            commands::play_game,
            commands::discipline,
            commands::give_medicine,
            commands::clean_poop,
            commands::toggle_lights,
            commands::hatch_new_egg,
            commands::get_permissions,
            commands::update_permissions,
            commands::get_mcp_config,
        ])
        .setup(move |app| {
            // ── Create main window with transparent titlebar ────────────
            let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("tama96")
                .inner_size(280.0, 384.0)
                .resizable(false)
                .center()
                .always_on_top(true);

            #[cfg(target_os = "macos")]
            let win_builder = win_builder.title_bar_style(tauri::TitleBarStyle::Transparent);

            let window = win_builder.build().unwrap();

            // Set macOS window background color (default teal shell)
            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::{NSColor, NSWindow};
                use cocoa::base::{id, nil};

                let ns_window = window.ns_window().unwrap() as id;
                unsafe {
                    // Teal: #5b9a9a
                    let bg_color = NSColor::colorWithRed_green_blue_alpha_(
                        nil,
                        91.0 / 255.0,
                        154.0 / 255.0,
                        154.0 / 255.0,
                        1.0,
                    );
                    ns_window.setBackgroundColor_(bg_color);
                }
            }

            // ── System tray ─────────────────────────────────────────────
            let show_item = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
            let status_item = MenuItemBuilder::with_id("status", "Pet Status").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&status_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Build tooltip from current state
            let tooltip = {
                let state = tick_pet.lock().unwrap();
                format!("tama96 — {:?} ({:?})", state.character, state.stage)
            };

            let tray_pet = Arc::clone(&tick_pet);

            // Load tray icon from embedded PNG bytes
            let icon_bytes = include_bytes!("../icons/32x32.png");
            let icon_image = image::load_from_memory(icon_bytes).expect("failed to decode tray icon");
            let rgba = icon_image.to_rgba8();
            let (w, h) = rgba.dimensions();
            let tray_icon = Image::new_owned(rgba.into_raw(), w, h);

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip(&tooltip)
                .menu(&tray_menu)
                .on_menu_event(move |app_handle, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "status" => {
                        // Update the status menu item text with current pet info
                        if let Ok(state) = tray_pet.lock() {
                            let status_text = format!(
                                "{:?} | Age {} | ❤{}/4 😊{}/4",
                                state.character, state.age, state.hunger, state.happiness
                            );
                            let _ = status_item.set_text(&status_text);
                        }
                    }
                    "quit" => {
                        // Signal sidecar to stop before exiting
                        if let Some(cancel) = app_handle.try_state::<Arc<AtomicBool>>() {
                            cancel.store(true, Ordering::Relaxed);
                        }
                        app_handle.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── TCP socket server for MCP bridge ────────────────────────
            let socket_pet = Arc::clone(&tick_pet);
            let socket_perms: commands::SharedPermissions =
                Arc::clone(app.state::<commands::SharedPermissions>().inner());
            tauri::async_runtime::spawn(async move {
                socket::start_socket_server(socket_pet, socket_perms).await;
            });

            // ── MCP sidecar lifecycle ───────────────────────────────────
            sidecar::write_mcp_config();
            let sidecar_cancel = Arc::new(AtomicBool::new(false));
            let sidecar_cancel_clone = Arc::clone(&sidecar_cancel);
            tauri::async_runtime::spawn(async move {
                sidecar::start_sidecar(sidecar_cancel_clone).await;
            });
            // Store cancel token so we can signal shutdown on quit
            app.manage(sidecar_cancel);

            // ── Background tick loop ────────────────────────────────────
            let tick_pet_loop = Arc::clone(&tick_pet);
            let tick_path = tick_save_path.clone();
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    let now = Utc::now();
                    if let Ok(mut state) = tick_pet_loop.lock() {
                        // Capture pre-tick state for change detection
                        let was_alive = state.is_alive;
                        let old_stage = state.stage.clone();
                        let old_hunger = state.hunger;
                        let old_happiness = state.happiness;

                        engine::tick(&mut state, now);
                        let _ = persistence::save(&state, &tick_path);

                        // Send notifications on critical events
                        if was_alive && !state.is_alive {
                            let _ = app_handle
                                .notification()
                                .builder()
                                .title("tama96")
                                .body("Your pet has died.")
                                .show();
                        } else if state.stage != old_stage && state.is_alive {
                            let _ = app_handle
                                .notification()
                                .builder()
                                .title("tama96")
                                .body(format!(
                                    "Your pet evolved to {:?} ({:?}).",
                                    state.character, state.stage
                                ))
                                .show();
                        }

                        if state.is_alive {
                            if state.hunger == 0 && old_hunger > 0 {
                                let _ = app_handle
                                    .notification()
                                    .builder()
                                    .title("tama96")
                                    .body("Your pet is starving.")
                                    .show();
                            }
                            if state.happiness == 0 && old_happiness > 0 {
                                let _ = app_handle
                                    .notification()
                                    .builder()
                                    .title("tama96")
                                    .body("Your pet is unhappy.")
                                    .show();
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide window instead of closing — keep tray alive
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
