//! System tray module
//!
//! This module handles the system tray icon and menu for CrossOver.
//! The tray provides quick access to common actions without needing
//! to interact with the crosshair window directly.

use crate::state::AppState;
use crate::window;
use log::{debug, error, info};
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

/// Set up the system tray icon and menu
pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    info!("Setting up system tray...");

    // Create the tray menu
    let menu = create_tray_menu(app)?;

    // Load tray icon
    let icon = load_tray_icon(app, false)?;

    // Build the tray icon
    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("CrossOver - Crosshair Overlay")
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(handle_tray_event)
        .build(app)
        .map_err(|e| format!("Failed to build tray icon: {}", e))?;

    info!("System tray setup complete");
    Ok(())
}

/// Create the tray context menu
fn create_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, String> {
    // Create menu items
    let toggle_lock = MenuItem::with_id(app, "toggle_lock", "Lock/Unlock", true, None::<&str>)
        .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let center = MenuItem::with_id(app, "center", "Center", true, None::<&str>)
        .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let hide = MenuItem::with_id(app, "hide", "Hide/Show", true, None::<&str>)
        .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)
        .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let choose_crosshair = MenuItem::with_id(
        app,
        "choose_crosshair",
        "Choose Crosshair...",
        true,
        None::<&str>,
    )
    .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let next_display = MenuItem::with_id(
        app,
        "next_display",
        "Move to Next Display",
        true,
        None::<&str>,
    )
    .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let reset = MenuItem::with_id(app, "reset", "Reset", true, None::<&str>)
        .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let about = MenuItem::with_id(app, "about", "About CrossOver", true, None::<&str>)
        .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let separator1 = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to create separator: {}", e))?;

    let separator2 = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to create separator: {}", e))?;

    let separator3 = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to create separator: {}", e))?;

    // Build the menu
    Menu::with_items(
        app,
        &[
            &toggle_lock,
            &center,
            &hide,
            &separator1,
            &settings,
            &choose_crosshair,
            &separator2,
            &next_display,
            &reset,
            &separator3,
            &about,
            &quit,
        ],
    )
    .map_err(|e| format!("Failed to create menu: {}", e))
}

/// Load the tray icon image
/// If `locked` is true, loads the locked variant of the icon
fn load_tray_icon<R: Runtime>(app: &AppHandle<R>, locked: bool) -> Result<Image<'static>, String> {
    // Determine icon filename based on lock state
    let icon_name = if locked {
        "icon-locked.png"
    } else {
        "icon.png"
    };

    // Try to load from resources
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?
        .join("icons")
        .join(icon_name);

    if resource_path.exists() {
        return Ok(
            Image::from_path(&resource_path).map_err(|e| format!("Failed to load icon: {}", e))?
        );
    }

    // Fallback: try the default icon if locked variant doesn't exist
    if locked {
        let fallback_path = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?
            .join("icons")
            .join("icon.png");

        if fallback_path.exists() {
            return Ok(Image::from_path(&fallback_path)
                .map_err(|e| format!("Failed to load fallback icon: {}", e))?);
        }
    }

    // Final fallback: create a simple colored icon
    debug!(
        "Tray icon not found at {:?}, generating default",
        resource_path
    );
    Ok(generate_default_icon(locked))
}

/// Generate a default icon programmatically
fn generate_default_icon(locked: bool) -> Image<'static> {
    let size = 32usize;
    let mut rgba = vec![0u8; size * size * 4];

    // Choose color based on lock state
    let (r, g, b) = if locked {
        (255u8, 100u8, 100u8) // Red-ish when locked
    } else {
        (100u8, 255u8, 100u8) // Green when unlocked
    };

    // Draw a simple cross pattern
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let is_cross = (x == size / 2 || y == size / 2)
                && x > size / 4
                && x < size * 3 / 4
                && y > size / 4
                && y < size * 3 / 4;

            // Add a circle around the cross when locked
            let dx = (x as i32 - size as i32 / 2).abs();
            let dy = (y as i32 - size as i32 / 2).abs();
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            let is_circle = locked && dist >= 10.0 && dist <= 12.0;

            if is_cross || is_circle {
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            } else {
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0; // Transparent
            }
        }
    }

    Image::new_owned(rgba, size as u32, size as u32)
}

/// Handle tray menu events
fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    debug!("Tray menu event: {}", id);

    let result = match id {
        "toggle_lock" => handle_toggle_lock(app),
        "center" => handle_center(app),
        "hide" => handle_hide(app),
        "settings" => handle_settings(app),
        "choose_crosshair" => handle_choose_crosshair(app),
        "next_display" => handle_next_display(app),
        "reset" => handle_reset(app),
        "about" => handle_about(app),
        "quit" => handle_quit(app),
        _ => {
            debug!("Unknown menu item: {}", id);
            Ok(())
        }
    };

    if let Err(e) = result {
        error!("Error handling menu event '{}': {}", id, e);
    }
}

/// Handle tray icon click events
fn handle_tray_event(tray: &TrayIcon, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
            debug!("Tray icon left-clicked");
            // Show/focus the main window on left click
            if let Some(window) = tray.app_handle().get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => {
            debug!("Tray icon double-clicked");
            // Toggle lock on double click
            let app = tray.app_handle();
            if let Some(state) = app.try_state::<Arc<AppState>>() {
                let locked = state.toggle_locked();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = window::set_click_through(&win, locked);
                }
                let _ = app.emit("lock-changed", locked);
            }
        }
        _ => {}
    }
}

// Menu action handlers

fn handle_toggle_lock(app: &AppHandle) -> Result<(), String> {
    info!("Tray: Toggle lock");

    let state = app.try_state::<Arc<AppState>>().ok_or("State not found")?;
    let locked = state.toggle_locked();

    if let Some(win) = app.get_webview_window("main") {
        window::set_click_through(&win, locked)?;
    }

    // Update shadow windows
    for label in state.get_shadow_windows() {
        if let Some(win) = app.get_webview_window(&label) {
            window::set_click_through(&win, locked)?;
        }
    }

    app.emit("lock-changed", locked)
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn handle_center(app: &AppHandle) -> Result<(), String> {
    info!("Tray: Center");

    if let Some(win) = app.get_webview_window("main") {
        window::center_on_current_monitor(&win)?;
    }

    Ok(())
}

fn handle_hide(app: &AppHandle) -> Result<(), String> {
    info!("Tray: Hide/Show");

    let state = app.try_state::<Arc<AppState>>().ok_or("State not found")?;
    let visible = state.toggle_visible();

    if let Some(win) = app.get_webview_window("main") {
        if visible {
            win.show().map_err(|e| e.to_string())?;
        } else {
            win.hide().map_err(|e| e.to_string())?;
        }
    }

    // Update shadow windows
    for label in state.get_shadow_windows() {
        if let Some(win) = app.get_webview_window(&label) {
            if visible {
                win.show().map_err(|e| e.to_string())?;
            } else {
                win.hide().map_err(|e| e.to_string())?;
            }
        }
    }

    app.emit("visibility-changed", visible)
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn handle_settings(app: &AppHandle) -> Result<(), String> {
    info!("Tray: Settings");

    // Emit event to open settings window
    app.emit("open-settings", ()).map_err(|e| e.to_string())?;

    Ok(())
}

fn handle_choose_crosshair(app: &AppHandle) -> Result<(), String> {
    info!("Tray: Choose crosshair");

    // Emit event to open crosshair chooser modal in main window
    app.emit("open-chooser", ()).map_err(|e| e.to_string())?;

    // Unlock window temporarily so user can interact with chooser
    let state = app.try_state::<Arc<AppState>>().ok_or("State not found")?;
    if state.is_locked() {
        state.set_locked(false);
        if let Some(win) = app.get_webview_window("main") {
            window::set_click_through(&win, false)?;
        }
        app.emit("lock-changed", false).map_err(|e| e.to_string())?;
    }

    if let Some(win) = app.get_webview_window("main") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn handle_next_display(app: &AppHandle) -> Result<(), String> {
    info!("Tray: Next display");

    if let Some(win) = app.get_webview_window("main") {
        window::move_to_next_display(&win)?;
    }

    Ok(())
}

fn handle_reset(app: &AppHandle) -> Result<(), String> {
    info!("Tray: Reset");

    let state = app.try_state::<Arc<AppState>>().ok_or("State not found")?;
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

    // Center the window
    handle_center(app)?;

    Ok(())
}

fn handle_about(app: &AppHandle) -> Result<(), String> {
    info!("Tray: About");

    // Emit event to show about dialog in frontend
    app.emit("show-about", ()).map_err(|e| e.to_string())?;

    // Ensure window is visible and focused for the about dialog
    if let Some(win) = app.get_webview_window("main") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn handle_quit(app: &AppHandle) -> Result<(), String> {
    info!("Tray: Quit");

    // Save preferences before quitting
    if let Some(state) = app.try_state::<Arc<AppState>>() {
        if let Err(e) = state.save_preferences(app) {
            error!("Failed to save preferences on quit: {}", e);
        }
    }

    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_default_icon_unlocked() {
        let icon = generate_default_icon(false);
        // Just verify it doesn't panic and returns valid dimensions
        assert!(icon.rgba().len() > 0);
    }

    #[test]
    fn test_generate_default_icon_locked() {
        let icon = generate_default_icon(true);
        // Just verify it doesn't panic and returns valid dimensions
        assert!(icon.rgba().len() > 0);
    }
}
