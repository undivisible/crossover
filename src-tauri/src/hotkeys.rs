//! Global hotkeys module
//!
//! This module handles global keyboard shortcuts that work even when
//! the application doesn't have focus. It provides functionality for:
//! - Toggling window lock (click-through mode)
//! - Centering the crosshair
//! - Hiding/showing the crosshair
//! - Moving between displays
//! - Moving the crosshair position
//! - Creating shadow windows
//! - Quitting the application

#![allow(dead_code)]

use crate::state::AppState;
use crate::window;
use log::{debug, error, info, warn};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Set up all global hotkeys for the application
/// Note: The global-shortcut plugin must be registered in main.rs before calling this
pub fn setup_hotkeys(app: &AppHandle) -> Result<(), String> {
    info!("Setting up global hotkeys...");

    // Register default shortcuts with their handlers
    register_default_shortcuts(app)?;

    info!("Global hotkeys setup complete");
    Ok(())
}

/// Register the default keyboard shortcuts
fn register_default_shortcuts(app: &AppHandle) -> Result<(), String> {
    let shortcuts_config = vec![
        ("Control+Shift+Alt+X", "toggle_lock"),
        ("Control+Shift+Alt+C", "center"),
        ("Control+Shift+Alt+H", "hide"),
        ("Control+Shift+Alt+R", "reset"),
        ("Control+Shift+Alt+M", "change_display"),
        ("Control+Shift+Alt+D", "duplicate"),
        ("Control+Shift+Alt+Q", "quit"),
        ("Control+Shift+Alt+Up", "move_up"),
        ("Control+Shift+Alt+Down", "move_down"),
        ("Control+Shift+Alt+Left", "move_left"),
        ("Control+Shift+Alt+Right", "move_right"),
    ];

    for (shortcut_str, action) in shortcuts_config {
        if let Err(e) = register_shortcut_with_handler(app, shortcut_str, action) {
            warn!(
                "Failed to register shortcut {} for {}: {}",
                shortcut_str, action, e
            );
        } else {
            debug!("Registered shortcut: {} -> {}", shortcut_str, action);
        }
    }

    Ok(())
}

/// Register a single shortcut with its handler
fn register_shortcut_with_handler(
    app: &AppHandle,
    shortcut_str: &str,
    action: &'static str,
) -> Result<(), String> {
    let shortcut: Shortcut = shortcut_str
        .parse()
        .map_err(|e| format!("Failed to parse shortcut '{}': {:?}", shortcut_str, e))?;

    let app_handle = app.clone();

    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                handle_action(&app_handle, action);
            }
        })
        .map_err(|e| format!("Failed to register shortcut '{}': {}", shortcut_str, e))?;

    Ok(())
}

/// Register a single shortcut (without handler, for custom shortcuts)
fn register_shortcut(app: &AppHandle, shortcut_str: &str) -> Result<(), String> {
    let shortcut: Shortcut = shortcut_str
        .parse()
        .map_err(|e| format!("Failed to parse shortcut '{}': {:?}", shortcut_str, e))?;

    app.global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Failed to register shortcut '{}': {}", shortcut_str, e))?;

    Ok(())
}

/// Unregister a single shortcut
pub fn unregister_shortcut(app: &AppHandle, shortcut_str: &str) -> Result<(), String> {
    let shortcut: Shortcut = shortcut_str
        .parse()
        .map_err(|e| format!("Failed to parse shortcut '{}': {:?}", shortcut_str, e))?;

    app.global_shortcut()
        .unregister(shortcut)
        .map_err(|e| format!("Failed to unregister shortcut '{}': {}", shortcut_str, e))?;

    Ok(())
}

/// Unregister all shortcuts
pub fn unregister_all(app: &AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("Failed to unregister all shortcuts: {}", e))?;

    Ok(())
}

/// Handle a named action
fn handle_action(app: &AppHandle, action: &str) {
    debug!("Action triggered: {}", action);

    let result = match action {
        "toggle_lock" => handle_toggle_lock(app),
        "center" => handle_center(app),
        "hide" => handle_hide(app),
        "reset" => handle_reset(app),
        "change_display" => handle_change_display(app),
        "duplicate" => handle_duplicate(app),
        "quit" => handle_quit(app),
        "move_up" => handle_move(app, 0, -1),
        "move_down" => handle_move(app, 0, 1),
        "move_left" => handle_move(app, -1, 0),
        "move_right" => handle_move(app, 1, 0),
        _ => {
            warn!("Unknown action: {}", action);
            Ok(())
        }
    };

    if let Err(e) = result {
        error!("Error handling action {}: {}", action, e);
    }
}

/// Toggle the window lock state
fn handle_toggle_lock(app: &AppHandle) -> Result<(), String> {
    info!("Toggle lock triggered");

    let state = app.state::<Arc<AppState>>();
    let locked = state.toggle_locked();

    // Update main window
    if let Some(window) = app.get_webview_window("main") {
        window::set_click_through(&window, locked)?;
    }

    // Update shadow windows
    for label in state.get_shadow_windows() {
        if let Some(window) = app.get_webview_window(&label) {
            window::set_click_through(&window, locked)?;
        }
    }

    // Emit event to update UI
    app.emit("lock-changed", locked)
        .map_err(|e| e.to_string())?;

    // Play sound feedback
    let sound = if locked { "lock" } else { "unlock" };
    app.emit("play-sound", sound).ok();

    Ok(())
}

/// Center the crosshair on the current display
fn handle_center(app: &AppHandle) -> Result<(), String> {
    info!("Center triggered");

    if let Some(window) = app.get_webview_window("main") {
        window::center_on_current_monitor(&window)?;
    }

    // Play sound feedback
    app.emit("play-sound", "center").ok();

    Ok(())
}

/// Toggle crosshair visibility
fn handle_hide(app: &AppHandle) -> Result<(), String> {
    info!("Hide triggered");

    let state = app.state::<Arc<AppState>>();

    // Only allow hiding when locked
    if !state.is_locked() {
        debug!("Hide ignored - window not locked");
        return Ok(());
    }

    let visible = state.toggle_visible();

    // Update main window
    if let Some(window) = app.get_webview_window("main") {
        if visible {
            window.show().map_err(|e| e.to_string())?;
        } else {
            window.hide().map_err(|e| e.to_string())?;
        }
    }

    // Update shadow windows
    for label in state.get_shadow_windows() {
        if let Some(window) = app.get_webview_window(&label) {
            if visible {
                window.show().map_err(|e| e.to_string())?;
            } else {
                window.hide().map_err(|e| e.to_string())?;
            }
        }
    }

    // Emit event to update UI
    app.emit("visibility-changed", visible)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Reset the crosshair to default settings
fn handle_reset(app: &AppHandle) -> Result<(), String> {
    info!("Reset triggered");

    let state = app.state::<Arc<AppState>>();
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

/// Move the crosshair to the next display
fn handle_change_display(app: &AppHandle) -> Result<(), String> {
    info!("Change display triggered");

    if let Some(window) = app.get_webview_window("main") {
        window::move_to_next_display(&window)?;
    }

    Ok(())
}

/// Create a duplicate (shadow) window
fn handle_duplicate(app: &AppHandle) -> Result<(), String> {
    info!("Duplicate triggered");

    let state = app.state::<Arc<AppState>>();

    // Don't create shadow windows when locked
    if state.is_locked() {
        debug!("Duplicate ignored - window locked");
        return Ok(());
    }

    // Check limit
    if state.shadow_window_count() >= 14 {
        warn!("Maximum shadow windows reached");
        return Ok(());
    }

    // Emit event to let frontend handle creation
    app.emit("create-shadow", ()).map_err(|e| e.to_string())?;

    Ok(())
}

/// Quit the application
fn handle_quit(app: &AppHandle) -> Result<(), String> {
    info!("Quit triggered");

    // Save preferences before quitting
    let state = app.state::<Arc<AppState>>();
    if let Err(e) = state.save_preferences(app) {
        error!("Failed to save preferences on quit: {}", e);
    }

    app.exit(0);
    Ok(())
}

/// Move the crosshair by a pixel offset
fn handle_move(app: &AppHandle, dx: i32, dy: i32) -> Result<(), String> {
    let state = app.state::<Arc<AppState>>();

    // Only allow movement when unlocked
    if state.is_locked() {
        debug!("Move ignored - window locked");
        return Ok(());
    }

    if let Some(window) = app.get_webview_window("main") {
        window::move_window_by(&window, dx, dy)?;
    }

    Ok(())
}

/// Re-register shortcuts with custom keybinds from preferences
pub fn update_shortcuts_from_preferences(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<Arc<AppState>>();
    let prefs = state.get_preferences();

    // Unregister all existing shortcuts
    unregister_all(app)?;

    // Register shortcuts from preferences with their handlers
    let keybinds = &prefs.keybinds;

    let shortcuts_with_actions = vec![
        (&keybinds.toggle_lock, "toggle_lock"),
        (&keybinds.center, "center"),
        (&keybinds.hide, "hide"),
        (&keybinds.reset, "reset"),
        (&keybinds.change_display, "change_display"),
        (&keybinds.duplicate, "duplicate"),
        (&keybinds.quit, "quit"),
        (&keybinds.move_up, "move_up"),
        (&keybinds.move_down, "move_down"),
        (&keybinds.move_left, "move_left"),
        (&keybinds.move_right, "move_right"),
    ];

    for (shortcut_str, action) in shortcuts_with_actions {
        if !shortcut_str.is_empty() {
            if let Err(e) = register_shortcut_with_handler(app, shortcut_str, action) {
                warn!(
                    "Failed to register custom shortcut '{}' for {}: {}",
                    shortcut_str, action, e
                );
            }
        }
    }

    info!("Shortcuts updated from preferences");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_parsing() {
        // Test that default shortcuts can be parsed
        let shortcuts = vec![
            "Control+Shift+Alt+X",
            "Control+Shift+Alt+C",
            "Control+Shift+Alt+Up",
        ];

        for s in shortcuts {
            let result: Result<Shortcut, _> = s.parse();
            assert!(result.is_ok(), "Failed to parse shortcut: {}", s);
        }
    }
}
