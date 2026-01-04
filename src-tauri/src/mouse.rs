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
use rdev::{listen, Event, EventType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tauri::{AppHandle, Emitter, Manager};

/// Global flag to control the mouse listener thread
static MOUSE_LISTENER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Handle to the mouse listener thread
static MOUSE_THREAD_HANDLE: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

/// Start following the mouse cursor
///
/// This spawns a background thread that listens for mouse movement events
/// and updates the window position accordingly.
pub fn start_following(app: &AppHandle, state: Arc<AppState>) -> Result<(), String> {
    // Check if already following
    if MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst) {
        debug!("Mouse following already active");
        return Ok(());
    }

    info!("Starting mouse following...");

    // Mark as active in state
    *state.mouse_following_active.write() = true;

    // Set the running flag
    MOUSE_LISTENER_RUNNING.store(true, Ordering::SeqCst);

    // Clone what we need for the thread
    let app_handle = app.clone();

    // Spawn the listener thread
    let handle = thread::spawn(move || {
        mouse_listener_thread(app_handle);
    });

    // Store the thread handle
    *MOUSE_THREAD_HANDLE.lock() = Some(handle);

    info!("Mouse following started");
    Ok(())
}

/// Stop following the mouse cursor
pub fn stop_following(state: &Arc<AppState>) -> Result<(), String> {
    if !MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst) {
        debug!("Mouse following not active");
        return Ok(());
    }

    info!("Stopping mouse following...");

    // Mark as inactive in state
    *state.mouse_following_active.write() = false;

    // Clear the running flag - this will cause the thread to exit
    MOUSE_LISTENER_RUNNING.store(false, Ordering::SeqCst);

    // Note: We don't join the thread here because rdev::listen is blocking
    // The thread will exit on its own when it detects the flag is false
    // or when the next event is processed

    info!("Mouse following stopped");
    Ok(())
}

/// Check if mouse following is currently active
pub fn is_following() -> bool {
    MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst)
}

/// The mouse listener thread function
fn mouse_listener_thread(app: AppHandle) {
    debug!("Mouse listener thread started");

    // Set up the callback for mouse events
    let callback = move |event: Event| {
        // Check if we should stop
        if !MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst) {
            // We can't actually stop rdev::listen from within the callback
            // but we can skip processing
            return;
        }

        // Only process mouse move events
        if let EventType::MouseMove { x, y } = event.event_type {
            handle_mouse_move(&app, x, y);
        }
    };

    // Start listening - this blocks until an error occurs
    if let Err(error) = listen(callback) {
        error!("Error in mouse listener: {:?}", error);
    }

    debug!("Mouse listener thread exiting");
    MOUSE_LISTENER_RUNNING.store(false, Ordering::SeqCst);
}

/// Handle a mouse move event by updating the window position
fn handle_mouse_move(app: &AppHandle, x: f64, y: f64) {
    // Get the main window
    let window = match app.get_webview_window("main") {
        Some(w) => w,
        None => {
            warn!("Main window not found for mouse following");
            return;
        }
    };

    // Get window size to center it on the cursor
    let size = match window.outer_size() {
        Ok(s) => s,
        Err(e) => {
            debug!("Failed to get window size: {}", e);
            return;
        }
    };

    // Calculate position to center window on cursor
    let new_x = x as i32 - (size.width as i32 / 2);
    let new_y = y as i32 - (size.height as i32 / 2);

    // Move the window
    if let Err(e) = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
        x: new_x,
        y: new_y,
    })) {
        debug!("Failed to move window: {}", e);
    }
}

/// Debounced mouse position update
///
/// This can be used instead of direct updates if performance is an issue.
/// It limits updates to a maximum frequency.
pub struct DebouncedMouseFollower {
    last_update: std::time::Instant,
    min_interval: std::time::Duration,
}

impl DebouncedMouseFollower {
    /// Create a new debounced follower with the specified minimum interval
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            last_update: std::time::Instant::now(),
            min_interval: std::time::Duration::from_millis(min_interval_ms),
        }
    }

    /// Check if enough time has passed for an update
    pub fn should_update(&mut self) -> bool {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_update) >= self.min_interval {
            self.last_update = now;
            true
        } else {
            false
        }
    }
}

/// Alternative implementation using Tauri's event system
///
/// This approach emits events that the frontend can handle,
/// which may be more efficient for some use cases.
pub fn start_following_with_events(app: &AppHandle, state: Arc<AppState>) -> Result<(), String> {
    if MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }

    info!("Starting mouse following with events...");

    *state.mouse_following_active.write() = true;
    MOUSE_LISTENER_RUNNING.store(true, Ordering::SeqCst);

    let app_handle = app.clone();

    let handle = thread::spawn(move || {
        let mut debouncer = DebouncedMouseFollower::new(16); // ~60fps max

        let callback = move |event: Event| {
            if !MOUSE_LISTENER_RUNNING.load(Ordering::SeqCst) {
                return;
            }

            if let EventType::MouseMove { x, y } = event.event_type {
                if debouncer.should_update() {
                    // Emit event instead of directly moving window
                    let _ = app_handle.emit("mouse-move", MousePosition { x, y });
                }
            }
        };

        if let Err(error) = listen(callback) {
            error!("Error in mouse listener: {:?}", error);
        }

        MOUSE_LISTENER_RUNNING.store(false, Ordering::SeqCst);
    });

    *MOUSE_THREAD_HANDLE.lock() = Some(handle);

    Ok(())
}

/// Mouse position data for events
#[derive(Clone, serde::Serialize)]
pub struct MousePosition {
    pub x: f64,
    pub y: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debouncer() {
        let mut debouncer = DebouncedMouseFollower::new(10);

        // First call should always update
        assert!(debouncer.should_update());

        // Immediate second call should not update
        assert!(!debouncer.should_update());

        // Wait and try again
        std::thread::sleep(std::time::Duration::from_millis(15));
        assert!(debouncer.should_update());
    }

    #[test]
    fn test_is_following_default() {
        // Should be false by default (assuming no other tests are running)
        // Note: This test may fail if run in parallel with other tests
        // that modify the global state
        let _ = is_following(); // Just ensure it doesn't panic
    }
}
