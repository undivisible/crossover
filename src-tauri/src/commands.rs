//! Tauri commands for IPC with the frontend
//!
//! These commands are exposed to the JavaScript frontend via Tauri's invoke system.

use crate::state::AppState;
use crate::window;
use std::sync::Arc;
use tauri::{command, AppHandle, Emitter, Manager};

/// Set the current crosshair image
#[command]
pub async fn set_crosshair(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    crosshair: String,
) -> Result<(), String> {
    state.set_crosshair(crosshair.clone());

    // Emit event to all windows to update crosshair
    app.emit("crosshair-changed", &crosshair)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get the current crosshair image
#[command]
pub fn get_crosshair(state: tauri::State<'_, Arc<AppState>>) -> String {
    state.get_crosshair()
}

/// Set the crosshair opacity
#[command]
pub async fn set_opacity(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    opacity: f64,
) -> Result<(), String> {
    state.set_opacity(opacity);

    // Emit event to all windows to update opacity
    app.emit("opacity-changed", opacity)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get the current opacity
#[command]
pub fn get_opacity(state: tauri::State<'_, Arc<AppState>>) -> f64 {
    state.get_opacity()
}

/// Set the crosshair size
#[command]
pub async fn set_size(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    size: u32,
) -> Result<(), String> {
    state.set_size(size);

    // Emit event to all windows to update size
    app.emit("size-changed", size).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get the current size
#[command]
pub fn get_size(state: tauri::State<'_, Arc<AppState>>) -> u32 {
    state.get_size()
}

/// Set the crosshair color
#[command]
pub async fn set_color(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    color: String,
) -> Result<(), String> {
    state.set_color(color.clone());

    // Emit event to all windows to update color
    app.emit("color-changed", &color)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get the current color
#[command]
pub fn get_color(state: tauri::State<'_, Arc<AppState>>) -> String {
    state.get_color()
}

/// Toggle the window lock state
#[command]
pub async fn toggle_lock(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let locked = state.toggle_locked();

    // Get main window and update ignore mouse events
    if let Some(window) = app.get_webview_window("main") {
        window::set_click_through(&window, locked)?;
    }

    // Update all shadow windows
    for label in state.get_shadow_windows() {
        if let Some(window) = app.get_webview_window(&label) {
            window::set_click_through(&window, locked)?;
        }
    }

    // Emit event to all windows
    app.emit("lock-changed", locked)
        .map_err(|e| e.to_string())?;

    Ok(locked)
}

/// Check if the window is locked
#[command]
pub fn is_locked(state: tauri::State<'_, Arc<AppState>>) -> bool {
    state.is_locked()
}

/// Center the window on the current display
#[command]
pub async fn center_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.center().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Move the window to the next display
#[command]
pub async fn move_to_next_display(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window::move_to_next_display(&window)?;
    }
    Ok(())
}

/// Toggle window visibility
#[command]
pub async fn toggle_visibility(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let visible = state.toggle_visible();

    // Get main window and update visibility
    if let Some(window) = app.get_webview_window("main") {
        if visible {
            window.show().map_err(|e| e.to_string())?;
        } else {
            window.hide().map_err(|e| e.to_string())?;
        }
    }

    // Update all shadow windows
    for label in state.get_shadow_windows() {
        if let Some(window) = app.get_webview_window(&label) {
            if visible {
                window.show().map_err(|e| e.to_string())?;
            } else {
                window.hide().map_err(|e| e.to_string())?;
            }
        }
    }

    // Emit event to all windows
    app.emit("visibility-changed", visible)
        .map_err(|e| e.to_string())?;

    Ok(visible)
}

/// Check if the window is visible
#[command]
pub fn is_visible(state: tauri::State<'_, Arc<AppState>>) -> bool {
    state.is_visible()
}

/// Get list of available crosshair images
#[command]
pub async fn get_crosshair_list(app: AppHandle) -> Result<Vec<String>, String> {
    let mut crosshairs = Vec::new();
    let file_extensions = ["png", "svg", "gif", "jpg", "jpeg", "webp"];

    // Helper to read directory
    let read_dir = |path: std::path::PathBuf, list: &mut Vec<String>| {
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    let lower_name = name.to_lowercase();
                    if file_extensions.iter().any(|ext| lower_name.ends_with(ext)) {
                        list.push(name.to_string());
                    }
                }
            }
        }
    };

    // 1. Resource directory
    if let Ok(resource_path) = app.path().resource_dir() {
        read_dir(resource_path.join("crosshairs"), &mut crosshairs);
    }

    // 2. App Data directory (UserData)
    if let Ok(app_data_path) = app.path().app_data_dir() {
        read_dir(app_data_path.join("crosshairs"), &mut crosshairs);
    }

    // Sort alphabetically and deduplicate behavior if needed (names are unique keys in frontend usually)
    crosshairs.sort();
    crosshairs.dedup(); // In case name collides, though filesystem usually prevents exact collisions in same dir.
                        // Here we might have collision between resource and app_data.
                        // If we have same filename in both, frontend will probably just pick one by URL path logic.
                        // Ideally we might want to prioritize one, but simple dedup is fine for now.

    Ok(crosshairs)
}

/// Save current preferences to disk
#[command]
pub async fn save_preferences(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.save_preferences(&app)
}

/// Load preferences from disk
#[command]
pub async fn load_preferences(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.load_preferences(&app)
}

/// Reset preferences to defaults
#[command]
pub async fn reset_preferences(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.reset_preferences();

    // Emit events to update UI
    let prefs = state.get_preferences();
    app.emit("crosshair-changed", &prefs.crosshair)
        .map_err(|e| e.to_string())?;
    app.emit("opacity-changed", prefs.opacity)
        .map_err(|e| e.to_string())?;
    app.emit("size-changed", prefs.size)
        .map_err(|e| e.to_string())?;
    app.emit("color-changed", &prefs.color)
        .map_err(|e| e.to_string())?;
    app.emit("reticle-changed", &prefs.reticle) // Add this
        .map_err(|e| e.to_string())?;
    // No event for hide_on_ads as it's just a setting

    Ok(())
}

/// Set follow mouse mode
#[command]
pub async fn set_follow_mouse(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    follow: bool,
) -> Result<(), String> {
    state.set_follow_mouse(follow);
    crate::mouse::update_mouse_listener_state(&app, state.inner().clone())?;
    Ok(())
}

/// Get follow mouse state
#[command]
pub fn get_follow_mouse(state: tauri::State<'_, Arc<AppState>>) -> bool {
    state.get_follow_mouse()
}

/// Set hide on ADS mode
#[command]
pub async fn set_hide_on_ads(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    hide: bool,
) -> Result<(), String> {
    state.set_hide_on_ads(hide);
    crate::mouse::update_mouse_listener_state(&app, state.inner().clone())?;
    Ok(())
}

/// Get hide on ADS state
#[command]
pub fn get_hide_on_ads(state: tauri::State<'_, Arc<AppState>>) -> bool {
    state.get_hide_on_ads()
}

/// Set the reticle type
#[command]
pub async fn set_reticle(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    reticle: String,
) -> Result<(), String> {
    state.set_reticle(reticle.clone());
    app.emit("reticle-changed", &reticle)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Get the reticle type
#[command]
pub fn get_reticle(state: tauri::State<'_, Arc<AppState>>) -> String {
    state.get_reticle()
}

/// Import a custom crosshair
#[command]
pub async fn import_crosshair(app: AppHandle, path: String) -> Result<String, String> {
    // Determine destination in app_data_dir (userData)
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let custom_dir = app_data_dir.join("crosshairs");

    // Ensure directory exists
    if !custom_dir.exists() {
        std::fs::create_dir_all(&custom_dir).map_err(|e| e.to_string())?;
    }

    // Get filename from path
    let src_path = std::path::Path::new(&path);
    let filename = src_path
        .file_name()
        .ok_or("Invalid path")?
        .to_str()
        .ok_or("Invalid filename")?
        .to_string();

    // Destination path
    let dest_path = custom_dir.join(&filename);

    // Copy file
    std::fs::copy(&path, &dest_path).map_err(|e| e.to_string())?;

    // Return the filename to be set as current crosshair
    Ok(filename)
}

/// Create a shadow (duplicate) window
#[command]
pub async fn create_shadow_window(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    // Limit to 14 shadow windows
    if state.shadow_window_count() >= 14 {
        return Err("Maximum shadow windows reached".to_string());
    }

    // Don't create shadow windows when locked
    if state.is_locked() {
        return Err("Cannot create shadow window while locked".to_string());
    }

    let label = state.next_shadow_id();

    // Get main window position for offset
    let main_window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let position = main_window.outer_position().map_err(|e| e.to_string())?;
    let size = main_window.outer_size().map_err(|e| e.to_string())?;

    // Calculate offset based on number of shadow windows
    let offset = (state.shadow_window_count() as i32 + 1) * 20;

    // Create the shadow window
    let shadow_window =
        tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::App("index.html".into()))
            .title("Shadow")
            .inner_size(size.width as f64, size.height as f64)
            .position((position.x + offset) as f64, (position.y + offset) as f64)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .visible_on_all_workspaces(true)
            .build()
            .map_err(|e| e.to_string())?;

    // Apply overlay settings
    window::setup_overlay_window(&shadow_window)?;

    // Apply lock state
    if state.is_locked() {
        window::set_click_through(&shadow_window, true)?;
    }

    state.add_shadow_window(label.clone());

    // Notify the shadow window to sync with main
    shadow_window
        .emit("sync-settings", state.get_preferences())
        .map_err(|e| e.to_string())?;

    Ok(label)
}

/// Close a specific shadow window
#[command]
pub async fn close_shadow_window(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    label: String,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
    }
    state.remove_shadow_window(&label);
    Ok(())
}

/// Close all shadow windows
#[command]
pub async fn close_all_shadow_windows(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    for label in state.get_shadow_windows() {
        if let Some(window) = app.get_webview_window(&label) {
            window.close().map_err(|e| e.to_string())?;
        }
    }
    state.clear_shadow_windows();
    Ok(())
}
