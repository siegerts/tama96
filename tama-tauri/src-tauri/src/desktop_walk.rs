use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
pub struct WalkStatus {
    supported: bool,
    away: bool,
}

#[tauri::command]
pub fn desktop_walk_status(app: AppHandle) -> WalkStatus {
    #[cfg(target_os = "macos")]
    let away = app.state::<mac::Walker>().is_away();
    #[cfg(not(target_os = "macos"))]
    let away = {
        let _ = app;
        false
    };
    WalkStatus {
        supported: cfg!(target_os = "macos"),
        away,
    }
}

#[derive(serde::Deserialize)]
pub struct Departure {
    pub character: String,
    pub birth_time: chrono::DateTime<chrono::Utc>,
    pub atlas: String,
    pub x: f64,
    pub y: f64,
    pub size: f64,
}

#[tauri::command]
pub async fn start_desktop_walk(
    window: tauri::WebviewWindow,
    departure: Departure,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if window.label() != "main" || departure.atlas.len() > 3_000_000 {
            return Err("Could not open the desktop pet.".into());
        }
        let app = window.app_handle().clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        window
            .clone()
            .run_on_main_thread(move || {
                let _ = tx.send(mac::start(&app, window, departure));
            })
            .map_err(|e| e.to_string())?;
        rx.await.map_err(|e| e.to_string())?
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (window, departure);
        Err("Desktop walks are available on macOS.".into())
    }
}

#[tauri::command]
pub async fn bring_pet_home(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let owner = app.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        app.run_on_main_thread(move || {
            mac::recall(&owner);
            let _ = tx.send(());
        })
        .map_err(|e| e.to_string())?;
        rx.await.map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = app;
    Ok(())
}

#[cfg(target_os = "macos")]
pub mod mac {
    use super::*;
    use crate::commands::SharedPetState;
    use chrono::{DateTime, Utc};
    use std::sync::{Arc, Mutex};
    use tama_core::state::{Character, LifeStage, PetState};
    use tauri::Emitter;

    unsafe extern "C" {
        fn tama_walk_start(
            owner: *mut std::ffi::c_void,
            bytes: *const u8,
            len: usize,
            x: f64,
            y: f64,
            size: f64,
            left_facing: bool,
        ) -> bool;
        fn tama_walk_stop();
    }

    struct Identity {
        birth: DateTime<Utc>,
        character: Character,
        stage: LifeStage,
    }

    impl Identity {
        fn can_walk(&self, pet: &PetState) -> bool {
            self.birth == pet.birth_time
                && self.character == pet.character
                && self.stage == pet.stage
                && pet.game.stars >= 3
                && pet.is_alive
                && !pet.is_sleeping
        }
    }

    pub struct Walker {
        active: Mutex<Option<Arc<Identity>>>,
        recall_item: tauri::menu::MenuItem<tauri::Wry>,
    }

    impl Walker {
        pub fn new(recall_item: tauri::menu::MenuItem<tauri::Wry>) -> Self {
            Self {
                active: Mutex::default(),
                recall_item,
            }
        }
        pub fn is_away(&self) -> bool {
            self.active.lock().unwrap().is_some()
        }
        fn is_current(&self, identity: &Arc<Identity>) -> bool {
            self.active
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|s| Arc::ptr_eq(s, identity))
        }
    }

    // All mutations of the session and its AppKit panel happen on the main thread.
    pub fn start(
        app: &AppHandle,
        window: tauri::WebviewWindow,
        req: Departure,
    ) -> Result<(), String> {
        let pets = app.state::<SharedPetState>();
        let pet = pets.lock().map_err(|e| e.to_string())?;
        let identity = Arc::new(Identity {
            birth: req.birth_time,
            character: pet.character.clone(),
            stage: pet.stage.clone(),
        });
        let form = if pet.stage == LifeStage::Egg {
            "Egg".into()
        } else {
            format!("{:?}", pet.character)
        };
        if !identity.can_walk(&pet) || form != req.character {
            return Err("Your pet must be awake and have its 3D form unlocked to walk.".into());
        }
        let walker = app.state::<Walker>();
        let mut active = walker.active.lock().unwrap();
        if active.is_some() {
            return Err("Your pet is already out walking.".into());
        }
        let owner = window.ns_window().map_err(|e| e.to_string())?;
        // The NSWindow lives for this call. AppKit copies the encoded bytes synchronously.
        let shown = unsafe {
            tama_walk_start(
                owner,
                req.atlas.as_ptr(),
                req.atlas.len(),
                req.x,
                req.y,
                req.size,
                form == "Kuchipatchi",
            )
        };
        if !shown {
            return Err("Could not draw the pet on this desktop.".into());
        }
        *active = Some(identity.clone());
        drop(active);
        let _ = walker.recall_item.set_enabled(true);
        drop(pet);
        let _ = app.emit_to("main", "desktop-walk-changed", true);
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if !app.state::<Walker>().is_current(&identity) {
                    break;
                }
                if !identity.can_walk(&app.state::<SharedPetState>().lock().unwrap()) {
                    let owner = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        if owner.state::<Walker>().is_current(&identity) {
                            recall(&owner);
                        }
                    });
                    break;
                }
            }
        });
        Ok(())
    }

    pub fn recall(app: &AppHandle) {
        let walker = app.state::<Walker>();
        if walker.active.lock().unwrap().take().is_some() {
            let _ = walker.recall_item.set_enabled(false);
            unsafe { tama_walk_stop() };
            let _ = app.emit_to("main", "desktop-walk-changed", false);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn an_unlocked_awake_pet_can_walk_even_when_sick() {
            let mut pet = PetState::new_egg(Utc::now());
            let session = Identity {
                birth: pet.birth_time,
                stage: pet.stage.clone(),
                character: pet.character.clone(),
            };
            assert!(!session.can_walk(&pet));
            pet.game.stars = 3;
            assert!(session.can_walk(&pet));
            pet.is_sick = true;
            assert!(session.can_walk(&pet));
            for change in [
                (|p: &mut PetState| p.is_alive = false) as fn(&mut PetState),
                |p| p.is_sleeping = true,
                |p| p.stage = LifeStage::Baby,
                |p| p.character = Character::Mametchi,
                |p| p.birth_time += chrono::Duration::seconds(1),
            ] {
                let mut changed = pet.clone();
                change(&mut changed);
                assert!(!session.can_walk(&changed));
            }
        }
    }
}
