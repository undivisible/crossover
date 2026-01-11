//! Mouse following module
//!
//! This module handles mouse cursor tracking functionality, allowing the
//! crosshair window to follow the mouse cursor position in real-time.
//!
//! The implementation uses the `rdev` crate for cross-platform mouse
//! event listening without requiring focus on the application window.

#![allow(dead_code)]

use crate::state::AppState;
use log::{debug, error, info, warn};
use parking_lot::Mutex;
use rdev::{listen, Button, Event, EventType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tauri::{AppHandle, Manager};

/// Global flag to control the mouse listener thread
static MOUSE_LISTENER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Handle to the mouse listener thread
static MOUSE_THREAD_HANDLE: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

/// Update mouse listener state based on preferences
pub fn update_mouse_listener_state(app: &AppHandle, state: Arc<AppState>) -> Result<(), String> {
    let follow_mouse = state.get_follow_mouse();
    let hide_on_ads = state.get_hide_on_ads();

    let should_run = follow_mouse || hide_on_ads;
    let is_running = MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst);

    if should_run && !is_running {
        start_listener(app, state)?;
    } else if !should_run && is_running {
        stop_listener()?;
    }

    Ok(())
}

/// Start the mouse listener thread
fn start_listener(app: &AppHandle, state: Arc<AppState>) -> Result<(), String> {
    if MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }

    info!("Starting mouse listener...");

    // Set the running flag
    MOUSE_LISTENER_RUNNING.store(true, Ordering::SeqCst);

    // Clone what we need for the thread
    let app_handle = app.clone();
    let state_handle = state.clone();

    // Spawn the listener thread
    let handle = thread::spawn(move || {
        mouse_listener_thread(app_handle, state_handle);
    });

    // Store the thread handle
    *MOUSE_THREAD_HANDLE.lock() = Some(handle);

    info!("Mouse listener started");
    Ok(())
}

/// Stop the mouse listener thread
fn stop_listener() -> Result<(), String> {
    if !MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }

    info!("Stopping mouse listener...");

    // Clear the running flag - this will cause the thread to exit
    MOUSE_LISTENER_RUNNING.store(false, Ordering::SeqCst);

    // Note: We don't join the thread here because rdev::listen is blocking
    // The thread will exit on its own when it detects the flag is false
    // or when the next event is processed

    info!("Mouse listener stopped");
    Ok(())
}

/// The mouse listener thread function
fn mouse_listener_thread(app: AppHandle, state: Arc<AppState>) {
    debug!("Mouse listener thread started");

    // Set up the callback for mouse events
    let callback = move |event: Event| {
        // Check if we should stop
        if !MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst) {
            return;
        }

        match event.event_type {
            EventType::MouseMove { x, y } => {
                if state.get_follow_mouse() {
                    handle_mouse_move(&app, x, y);
                }
            }
            EventType::ButtonPress(Button::Right) => {
                if state.get_hide_on_ads() {
                    set_windows_visible(&app, &state, false);
                }
            }
            EventType::ButtonRelease(Button::Right) => {
                if state.get_hide_on_ads() {
                    // Only show if globally visible
                    if state.is_visible() {
                        set_windows_visible(&app, &state, true);
                    }
                }
            }
            _ => {}
        }
    };

    // Start listening - this blocks until an error occurs
    if let Err(error) = listen(callback) {
        error!("Error in mouse listener: {:?}", error);
    }

    // Reset flag if we exited due to error
    debug!("Mouse listener thread exiting");
    MOUSE_LISTENER_RUNNING.store(false, Ordering::SeqCst);
}

/// Helper to show/hide all windows
fn set_windows_visible(app: &AppHandle, state: &AppState, visible: bool) {
    // Helper closure to avoid repetition
    let update_window = |label: &str| {
        if let Some(window) = app.get_webview_window(label) {
            if visible {
                let _ = window.show();
            } else {
                let _ = window.hide();
            }
        }
    };

    // Update main window
    update_window("main");

    // Update shadow windows
    for label in state.get_shadow_windows() {
        update_window(&label);
    }
}

/// Handle a mouse move event by updating the window position
fn handle_mouse_move(app: &AppHandle, x: f64, y: f64) {
    // Get the main window
    let window = match app.get_webview_window("main") {
        Some(w) => w,
        None => {
            // Only warn once to avoid log spam
            // warn!("Main window not found for mouse following");
            return;
        }
    };

    // Get window size to center it on the cursor
    let size = match window.outer_size() {
        Ok(s) => s,
        Err(_) => return,
    };

    // Calculate position to center window on cursor
    let new_x = x as i32 - (size.width as i32 / 2);
    let new_y = y as i32 - (size.height as i32 / 2);

    // Move the window
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
        x: new_x,
        y: new_y,
    }));
}
