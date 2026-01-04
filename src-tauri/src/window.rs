//! Window management utilities
//!
//! This module handles platform-specific window configuration for creating
//! a transparent, click-through overlay window that stays on top of all
//! other windows, including fullscreen applications.

use log::{debug, info};
use tauri::{Monitor, WebviewWindow};

#[cfg(target_os = "linux")]
use log::warn;

/// Set up the overlay window with platform-specific settings
///
/// This configures the window to:
/// - Be transparent and borderless
/// - Stay on top of all windows (including fullscreen)
/// - Be visible on all workspaces/virtual desktops
/// - Initially accept mouse events (unlocked state)
pub fn setup_overlay_window(window: &WebviewWindow) -> Result<(), String> {
    info!("Setting up overlay window: {}", window.label());

    // Ensure window is always on top with highest level
    window
        .set_always_on_top(true)
        .map_err(|e| format!("Failed to set always on top: {}", e))?;

    // Make visible on all workspaces
    window
        .set_visible_on_all_workspaces(true)
        .map_err(|e| format!("Failed to set visible on all workspaces: {}", e))?;

    // Platform-specific setup
    #[cfg(target_os = "macos")]
    setup_macos_overlay(window)?;

    #[cfg(target_os = "windows")]
    setup_windows_overlay(window)?;

    #[cfg(target_os = "linux")]
    setup_linux_overlay(window)?;

    info!("Overlay window setup complete");
    Ok(())
}

/// Set the click-through (ignore mouse events) state of a window
///
/// When `enabled` is true, all mouse events pass through the window
/// to whatever is beneath it.
pub fn set_click_through(window: &WebviewWindow, enabled: bool) -> Result<(), String> {
    debug!(
        "Setting click-through for window {}: {}",
        window.label(),
        enabled
    );

    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| format!("Failed to set ignore cursor events: {}", e))?;

    Ok(())
}

/// Move the window to the next display/monitor
pub fn move_to_next_display(window: &WebviewWindow) -> Result<(), String> {
    // Get all available monitors
    let monitors: Vec<Monitor> = window
        .available_monitors()
        .map_err(|e| format!("Failed to get monitors: {}", e))?;

    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    // Get current monitor
    let current_monitor = window
        .current_monitor()
        .map_err(|e| format!("Failed to get current monitor: {}", e))?
        .ok_or("No current monitor")?;

    // Find current monitor index
    let current_index = monitors
        .iter()
        .position(|m| m.name() == current_monitor.name())
        .unwrap_or(0);

    // Get next monitor (wrap around)
    let next_index = (current_index + 1) % monitors.len();
    let next_monitor = &monitors[next_index];

    // Get window size
    let window_size = window
        .outer_size()
        .map_err(|e| format!("Failed to get window size: {}", e))?;

    // Calculate center position on next monitor
    let monitor_pos = next_monitor.position();
    let monitor_size = next_monitor.size();

    let new_x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
    let new_y = monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;

    // Move window
    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: new_x,
            y: new_y,
        }))
        .map_err(|e| format!("Failed to move window: {}", e))?;

    info!(
        "Moved window to monitor {} at ({}, {})",
        next_monitor.name().unwrap_or(&"Unknown".to_string()),
        new_x,
        new_y
    );

    Ok(())
}

/// Move the window by a relative offset
pub fn move_window_by(window: &WebviewWindow, dx: i32, dy: i32) -> Result<(), String> {
    let position = window
        .outer_position()
        .map_err(|e| format!("Failed to get window position: {}", e))?;

    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: position.x + dx,
            y: position.y + dy,
        }))
        .map_err(|e| format!("Failed to move window: {}", e))?;

    Ok(())
}

/// Center the window on its current monitor
pub fn center_on_current_monitor(window: &WebviewWindow) -> Result<(), String> {
    let monitor = window
        .current_monitor()
        .map_err(|e| format!("Failed to get current monitor: {}", e))?
        .ok_or("No current monitor")?;

    let window_size = window
        .outer_size()
        .map_err(|e| format!("Failed to get window size: {}", e))?;

    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();

    let new_x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
    let new_y = monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;

    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: new_x,
            y: new_y,
        }))
        .map_err(|e| format!("Failed to center window: {}", e))?;

    Ok(())
}

// ============================================================================
// Platform-specific implementations
// ============================================================================

/// macOS-specific overlay window setup using objc2
#[cfg(target_os = "macos")]
fn setup_macos_overlay(window: &WebviewWindow) -> Result<(), String> {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    info!("Applying macOS-specific overlay settings");

    // Get the NSWindow handle from Tauri
    let ns_window_ptr = window.ns_window().map_err(|e| e.to_string())?;

    unsafe {
        // Cast the raw pointer to NSWindow
        let ns_window: &NSWindow = &*(ns_window_ptr as *const NSWindow);

        // Set window level to be above screen savers and fullscreen apps
        // CGWindowLevelForKey(kCGScreenSaverWindowLevelKey) is typically 1000
        // We set it to 1001 to be above screen savers
        let screen_saver_level: isize = 1001;
        ns_window.setLevel(screen_saver_level);

        // Set collection behavior to work with fullscreen apps and spaces
        // NSWindowCollectionBehaviorCanJoinAllSpaces: Window appears on all spaces
        // NSWindowCollectionBehaviorStationary: Window doesn't move when switching spaces
        // NSWindowCollectionBehaviorFullScreenAuxiliary: Works with fullscreen apps
        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::FullScreenAuxiliary;
        ns_window.setCollectionBehavior(behavior);

        // Prevent window from hiding when app is deactivated
        ns_window.setHidesOnDeactivate(false);

        // Make window non-opaque for transparency support
        ns_window.setOpaque(false);

        // Disable shadow for cleaner overlay appearance
        ns_window.setHasShadow(false);
    }

    debug!("macOS overlay settings applied successfully");
    Ok(())
}

/// Windows-specific overlay window setup
#[cfg(target_os = "windows")]
fn setup_windows_overlay(window: &WebviewWindow) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOMOVE,
        SWP_NOSIZE, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    };

    info!("Applying Windows-specific overlay settings");

    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    let hwnd = HWND(hwnd.0);

    unsafe {
        // Get current extended style
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);

        // Add layered and toolwindow styles
        // WS_EX_TOOLWINDOW: Doesn't appear in taskbar
        // WS_EX_LAYERED: Required for transparency
        // WS_EX_TOPMOST: Always on top
        // Note: WS_EX_TRANSPARENT is controlled by set_ignore_cursor_events
        let new_style = ex_style
            | WS_EX_LAYERED.0 as isize
            | WS_EX_TOOLWINDOW.0 as isize
            | WS_EX_TOPMOST.0 as isize;

        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);

        // Ensure topmost positioning
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)
            .map_err(|e| format!("Failed to set window position: {}", e))?;
    }

    debug!("Windows overlay settings applied successfully");
    Ok(())
}

/// Linux-specific overlay window setup
#[cfg(target_os = "linux")]
fn setup_linux_overlay(_window: &WebviewWindow) -> Result<(), String> {
    info!("Applying Linux-specific overlay settings");

    // Most Linux functionality is handled by Tauri's built-in APIs
    // Additional X11/Wayland specific handling could be added here if needed

    // For X11, we might want to set _NET_WM_STATE atoms for:
    // - _NET_WM_STATE_ABOVE (always on top)
    // - _NET_WM_STATE_STICKY (visible on all workspaces)
    // - _NET_WM_WINDOW_TYPE_DOCK or _NET_WM_WINDOW_TYPE_UTILITY

    // For Wayland, overlay behavior depends heavily on compositor support
    // Most Wayland compositors don't allow true overlay windows for security

    warn!(
        "Linux overlay: Some features may be limited depending on your window manager/compositor"
    );
    debug!("Linux overlay settings applied");

    Ok(())
}

#[cfg(test)]
mod tests {
    // Unit tests would go here
    // Note: Most window tests require a running Tauri app context
}
