// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod crosshair;
mod hotkeys;
mod mouse;
mod state;
mod tray;
mod window;

use log::info;
use state::AppState;
use std::sync::Arc;
use tauri::Manager;

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting CrossOver v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .manage(Arc::new(AppState::default()))
        .setup(|app| {
            info!("Setting up application...");

            // Get the main window
            let main_window = app
                .get_webview_window("main")
                .expect("main window not found");

            // Apply platform-specific window settings
            window::setup_overlay_window(&main_window)?;

            // Ensure window starts unlocked (not click-through)
            // This is critical for dragging and interacting
            if let Err(e) = main_window.set_ignore_cursor_events(false) {
                log::warn!("Failed to set ignore cursor events to false: {}", e);
            } else {
                info!("Window starts unlocked and draggable");
            }

            // Setup system tray using app handle
            let app_handle = app.handle().clone();
            tray::setup_tray(&app_handle)?;

            // Setup global hotkeys using app handle
            hotkeys::setup_hotkeys(&app_handle)?;

            // Load saved preferences
            let state = app.state::<Arc<AppState>>();
            if let Err(e) = state.load_preferences(&app_handle) {
                log::warn!("Failed to load preferences: {}", e);
            }

            // Log initial state
            info!("Initial state - Locked: {}, Visible: {}",
                  state.is_locked(), state.is_visible());

            info!("Application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_crosshair,
            commands::get_crosshair,
            commands::set_opacity,
            commands::get_opacity,
            commands::set_size,
            commands::get_size,
            commands::set_color,
            commands::get_color,
            commands::toggle_lock,
            commands::is_locked,
            commands::center_window,
            commands::move_to_next_display,
            commands::toggle_visibility,
            commands::is_visible,
            commands::get_crosshair_list,
            commands::save_preferences,
            commands::load_preferences,
            commands::reset_preferences,
            commands::set_follow_mouse,
            commands::get_follow_mouse,
            commands::create_shadow_window,
            commands::close_shadow_window,
            commands::close_all_shadow_windows,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Hide window instead of closing when it's the main window
                if window.label() == "main" {
                    window.hide().unwrap_or_default();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
