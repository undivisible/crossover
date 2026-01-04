//! Configuration constants for the CrossOver application
//!
//! This module contains all the constant values used throughout the application,
//! including default sizes, limits, and other configuration values.

#![allow(dead_code)]

/// Default window width in pixels
pub const DEFAULT_WINDOW_WIDTH: u32 = 200;

/// Default window height in pixels
pub const DEFAULT_WINDOW_HEIGHT: u32 = 200;

/// Minimum window width in pixels
pub const MIN_WINDOW_WIDTH: u32 = 50;

/// Minimum window height in pixels
pub const MIN_WINDOW_HEIGHT: u32 = 50;

/// Maximum window width in pixels
pub const MAX_WINDOW_WIDTH: u32 = 800;

/// Maximum window height in pixels
pub const MAX_WINDOW_HEIGHT: u32 = 800;

/// Default crosshair size in pixels
pub const DEFAULT_CROSSHAIR_SIZE: u32 = 100;

/// Minimum crosshair size in pixels
pub const MIN_CROSSHAIR_SIZE: u32 = 10;

/// Maximum crosshair size in pixels
pub const MAX_CROSSHAIR_SIZE: u32 = 500;

/// Default crosshair opacity (0.0 - 1.0)
pub const DEFAULT_OPACITY: f64 = 1.0;

/// Default crosshair color (hex)
pub const DEFAULT_COLOR: &str = "#00FF00";

/// Default crosshair image filename
pub const DEFAULT_CROSSHAIR: &str = "crosshair-default.png";

/// Maximum number of shadow (duplicate) windows allowed
pub const MAX_SHADOW_WINDOWS: usize = 14;

/// Offset in pixels for each new shadow window position
pub const SHADOW_WINDOW_OFFSET: i32 = 20;

/// Window aspect ratio (width / height)
pub const WINDOW_ASPECT_RATIO: f64 = 1.0;

/// Debounce interval for mouse following in milliseconds
pub const MOUSE_FOLLOW_DEBOUNCE_MS: u64 = 16; // ~60 FPS

/// Save debounce interval in milliseconds
pub const SAVE_DEBOUNCE_MS: u64 = 500;

/// Move increment in pixels for keyboard movement
pub const MOVE_INCREMENT: i32 = 1;

/// Fast move increment in pixels (when shift is held)
pub const FAST_MOVE_INCREMENT: i32 = 10;

/// Application name
pub const APP_NAME: &str = "CrossOver";

/// Application identifier
pub const APP_ID: &str = "com.lacymorrow.crossover";

/// Settings store filename
pub const SETTINGS_STORE_FILENAME: &str = "crossover-settings.json";

/// Supported crosshair image extensions
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["png", "svg", "gif", "jpg", "jpeg", "webp"];

/// Default keybind modifier (Control+Shift+Alt)
pub const DEFAULT_MODIFIER: &str = "Control+Shift+Alt";

/// Default keybinds
pub mod keybinds {
    pub const TOGGLE_LOCK: &str = "Control+Shift+Alt+X";
    pub const CENTER: &str = "Control+Shift+Alt+C";
    pub const HIDE: &str = "Control+Shift+Alt+H";
    pub const RESET: &str = "Control+Shift+Alt+R";
    pub const CHANGE_DISPLAY: &str = "Control+Shift+Alt+M";
    pub const DUPLICATE: &str = "Control+Shift+Alt+D";
    pub const QUIT: &str = "Control+Shift+Alt+Q";
    pub const MOVE_UP: &str = "Control+Shift+Alt+Up";
    pub const MOVE_DOWN: &str = "Control+Shift+Alt+Down";
    pub const MOVE_LEFT: &str = "Control+Shift+Alt+Left";
    pub const MOVE_RIGHT: &str = "Control+Shift+Alt+Right";
    pub const NEXT_WINDOW: &str = "Control+Shift+Alt+O";
}

/// Sound effect names
pub mod sounds {
    pub const LOCK: &str = "lock";
    pub const UNLOCK: &str = "unlock";
    pub const CENTER: &str = "center";
}

/// Reticle types
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReticleType {
    None,
    Circle,
    Cross,
    Dot,
}

impl Default for ReticleType {
    fn default() -> Self {
        Self::None
    }
}

impl ReticleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReticleType::None => "none",
            ReticleType::Circle => "circle",
            ReticleType::Cross => "cross",
            ReticleType::Dot => "dot",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "circle" => ReticleType::Circle,
            "cross" => ReticleType::Cross,
            "dot" => ReticleType::Dot,
            _ => ReticleType::None,
        }
    }
}

/// Theme options
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Self::System
    }
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(DEFAULT_WINDOW_WIDTH, 200);
        assert_eq!(DEFAULT_WINDOW_HEIGHT, 200);
        assert_eq!(DEFAULT_OPACITY, 1.0);
        assert_eq!(MAX_SHADOW_WINDOWS, 14);
    }

    #[test]
    fn test_reticle_type_conversion() {
        assert_eq!(ReticleType::Circle.as_str(), "circle");
        assert_eq!(ReticleType::from_str("circle"), ReticleType::Circle);
        assert_eq!(ReticleType::from_str("unknown"), ReticleType::None);
    }

    #[test]
    fn test_theme_conversion() {
        assert_eq!(Theme::Dark.as_str(), "dark");
        assert_eq!(Theme::from_str("dark"), Theme::Dark);
        assert_eq!(Theme::from_str("unknown"), Theme::System);
    }
}
