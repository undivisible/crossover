//! Application state management
//!
//! This module handles the global application state including:
//! - Crosshair settings (image, size, color, opacity)
//! - Window state (locked, visible, position)
//! - User preferences persistence

#![allow(dead_code)]

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

/// Default crosshair image
pub const DEFAULT_CROSSHAIR: &str = "crosshair-default.png";

/// Default crosshair size in pixels
pub const DEFAULT_SIZE: u32 = 100;

/// Default crosshair opacity (0.0 - 1.0)
pub const DEFAULT_OPACITY: f64 = 1.0;

/// Default crosshair color
pub const DEFAULT_COLOR: &str = "#00FF00";

/// Store filename for preferences
const STORE_FILENAME: &str = "crossover-settings.json";

/// Serializable preferences that are persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    /// Current crosshair image filename
    pub crosshair: String,

    /// Crosshair size in pixels
    pub size: u32,

    /// Crosshair opacity (0.0 - 1.0)
    pub opacity: f64,

    /// Crosshair color (hex string)
    pub color: String,

    /// Whether the crosshair is locked (click-through)
    pub locked: bool,

    /// Whether the crosshair is visible
    pub visible: bool,

    /// Whether to follow the mouse cursor
    pub follow_mouse: bool,

    /// Saved X position
    pub position_x: Option<i32>,

    /// Saved Y position
    pub position_y: Option<i32>,

    /// Whether to start on system boot
    pub start_on_boot: bool,

    /// Custom keybinds
    pub keybinds: KeybindPreferences,

    /// Whether to hide when aiming down sights (right click)
    pub hide_on_ads: bool,

    /// Helper reticle type (none, dot, cross, circle)
    pub reticle: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            crosshair: "target-dot.png".to_string(),
            size: DEFAULT_SIZE,
            opacity: DEFAULT_OPACITY,
            color: DEFAULT_COLOR.to_string(),
            locked: false,
            visible: true,
            follow_mouse: false,
            position_x: None,
            position_y: None,
            start_on_boot: false,
            keybinds: KeybindPreferences::default(),
            hide_on_ads: false,
            reticle: "dot".to_string(),
        }
    }
}

/// Keybind preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindPreferences {
    pub toggle_lock: String,
    pub center: String,
    pub hide: String,
    pub reset: String,
    pub move_up: String,
    pub move_down: String,
    pub move_left: String,
    pub move_right: String,
    pub change_display: String,
    pub duplicate: String,
    pub quit: String,
}

impl Default for KeybindPreferences {
    fn default() -> Self {
        Self {
            toggle_lock: "Control+Shift+Alt+X".to_string(),
            center: "Control+Shift+Alt+C".to_string(),
            hide: "Control+Shift+Alt+H".to_string(),
            reset: "Control+Shift+Alt+R".to_string(),
            move_up: "Control+Shift+Alt+Up".to_string(),
            move_down: "Control+Shift+Alt+Down".to_string(),
            move_left: "Control+Shift+Alt+Left".to_string(),
            move_right: "Control+Shift+Alt+Right".to_string(),
            change_display: "Control+Shift+Alt+M".to_string(),
            duplicate: "Control+Shift+Alt+D".to_string(),
            quit: "Control+Shift+Alt+Q".to_string(),
        }
    }
}

/// Global application state
pub struct AppState {
    /// Current preferences
    pub preferences: RwLock<Preferences>,

    /// Set of shadow window labels
    pub shadow_windows: RwLock<HashSet<String>>,

    /// Counter for shadow window IDs
    shadow_counter: RwLock<u32>,

    /// Whether mouse following is currently active
    pub mouse_following_active: RwLock<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            preferences: RwLock::new(Preferences::default()),
            shadow_windows: RwLock::new(HashSet::new()),
            shadow_counter: RwLock::new(0),
            mouse_following_active: RwLock::new(false),
        }
    }
}

impl AppState {
    /// Create a new AppState with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current crosshair
    pub fn get_crosshair(&self) -> String {
        self.preferences.read().crosshair.clone()
    }

    /// Set the current crosshair
    pub fn set_crosshair(&self, crosshair: String) {
        self.preferences.write().crosshair = crosshair;
    }

    /// Get the current size
    pub fn get_size(&self) -> u32 {
        self.preferences.read().size
    }

    /// Set the current size
    pub fn set_size(&self, size: u32) {
        self.preferences.write().size = size;
    }

    /// Get the current opacity
    pub fn get_opacity(&self) -> f64 {
        self.preferences.read().opacity
    }

    /// Set the current opacity
    pub fn set_opacity(&self, opacity: f64) {
        self.preferences.write().opacity = opacity.clamp(0.0, 1.0);
    }

    /// Get the current color
    pub fn get_color(&self) -> String {
        self.preferences.read().color.clone()
    }

    /// Set the current color
    pub fn set_color(&self, color: String) {
        self.preferences.write().color = color;
    }

    /// Check if window is locked
    pub fn is_locked(&self) -> bool {
        self.preferences.read().locked
    }

    /// Set the locked state
    pub fn set_locked(&self, locked: bool) {
        self.preferences.write().locked = locked;
    }

    /// Toggle the locked state
    pub fn toggle_locked(&self) -> bool {
        let mut prefs = self.preferences.write();
        prefs.locked = !prefs.locked;
        prefs.locked
    }

    /// Check if window is visible
    pub fn is_visible(&self) -> bool {
        self.preferences.read().visible
    }

    /// Set the visibility state
    pub fn set_visible(&self, visible: bool) {
        self.preferences.write().visible = visible;
    }

    /// Toggle the visibility state
    pub fn toggle_visible(&self) -> bool {
        let mut prefs = self.preferences.write();
        prefs.visible = !prefs.visible;
        prefs.visible
    }

    /// Check if follow mouse is enabled
    pub fn get_follow_mouse(&self) -> bool {
        self.preferences.read().follow_mouse
    }

    /// Set follow mouse state
    pub fn set_follow_mouse(&self, follow: bool) {
        self.preferences.write().follow_mouse = follow;
    }

    /// Check if hide on ADS is enabled
    pub fn get_hide_on_ads(&self) -> bool {
        self.preferences.read().hide_on_ads
    }

    /// Set hide on ADS state
    pub fn set_hide_on_ads(&self, hide: bool) {
        self.preferences.write().hide_on_ads = hide;
    }

    /// Get reticle type
    pub fn get_reticle(&self) -> String {
        self.preferences.read().reticle.clone()
    }

    /// Set reticle type
    pub fn set_reticle(&self, reticle: String) {
        self.preferences.write().reticle = reticle;
    }

    /// Get saved position
    pub fn get_position(&self) -> (Option<i32>, Option<i32>) {
        let prefs = self.preferences.read();
        (prefs.position_x, prefs.position_y)
    }

    /// Set saved position
    pub fn set_position(&self, x: i32, y: i32) {
        let mut prefs = self.preferences.write();
        prefs.position_x = Some(x);
        prefs.position_y = Some(y);
    }

    /// Generate a new shadow window ID
    pub fn next_shadow_id(&self) -> String {
        let mut counter = self.shadow_counter.write();
        *counter += 1;
        format!("shadow-{}", *counter)
    }

    /// Add a shadow window
    pub fn add_shadow_window(&self, label: String) {
        self.shadow_windows.write().insert(label);
    }

    /// Remove a shadow window
    pub fn remove_shadow_window(&self, label: &str) {
        self.shadow_windows.write().remove(label);
    }

    /// Get all shadow window labels
    pub fn get_shadow_windows(&self) -> Vec<String> {
        self.shadow_windows.read().iter().cloned().collect()
    }

    /// Get shadow window count
    pub fn shadow_window_count(&self) -> usize {
        self.shadow_windows.read().len()
    }

    /// Clear all shadow windows
    pub fn clear_shadow_windows(&self) {
        self.shadow_windows.write().clear();
    }

    /// Save preferences to disk
    pub fn save_preferences(&self, app: &AppHandle) -> Result<(), String> {
        let store = app
            .store(STORE_FILENAME)
            .map_err(|e| format!("Failed to get store: {}", e))?;

        let prefs = self.preferences.read().clone();

        store.set("preferences", serde_json::to_value(&prefs).unwrap());

        store
            .save()
            .map_err(|e| format!("Failed to save store: {}", e))?;

        log::info!("Preferences saved");
        Ok(())
    }

    /// Load preferences from disk
    pub fn load_preferences(&self, app: &AppHandle) -> Result<(), String> {
        let store = app
            .store(STORE_FILENAME)
            .map_err(|e| format!("Failed to get store: {}", e))?;

        if let Some(value) = store.get("preferences") {
            match serde_json::from_value::<Preferences>(value.clone()) {
                Ok(prefs) => {
                    *self.preferences.write() = prefs;
                    log::info!("Preferences loaded");
                }
                Err(e) => {
                    log::warn!("Failed to parse preferences, using defaults: {}", e);
                }
            }
        } else {
            log::info!("No saved preferences found, using defaults");
        }

        Ok(())
    }

    /// Reset preferences to defaults
    pub fn reset_preferences(&self) {
        *self.preferences.write() = Preferences::default();
        log::info!("Preferences reset to defaults");
    }

    /// Get a clone of current preferences
    pub fn get_preferences(&self) -> Preferences {
        self.preferences.read().clone()
    }
}
