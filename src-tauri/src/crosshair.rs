//! Crosshair module
//!
//! This module handles crosshair-specific logic including:
//! - Loading and validating crosshair images
//! - Crosshair rendering utilities
//! - Custom crosshair management

#![allow(dead_code)]

use log::{debug, info};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};

use crate::config::SUPPORTED_IMAGE_EXTENSIONS;

/// Crosshair image information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrosshairInfo {
    /// Filename of the crosshair image
    pub filename: String,

    /// Display name (filename without extension)
    pub name: String,

    /// Full path to the crosshair file
    pub path: PathBuf,

    /// Whether this is a built-in crosshair
    pub is_builtin: bool,

    /// Whether this is a custom user crosshair
    pub is_custom: bool,
}

impl CrosshairInfo {
    /// Create a new CrosshairInfo from a path
    pub fn from_path(path: PathBuf, is_builtin: bool) -> Option<Self> {
        let filename = path.file_name()?.to_str()?.to_string();
        let name = path.file_stem()?.to_str()?.to_string();

        Some(Self {
            filename,
            name,
            path,
            is_builtin,
            is_custom: !is_builtin,
        })
    }
}

/// Get the path to the built-in crosshairs directory
pub fn get_builtin_crosshairs_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .resource_dir()
        .map(|p| p.join("crosshairs"))
        .map_err(|e| format!("Failed to get resource directory: {}", e))
}

/// Get the path to the custom crosshairs directory
pub fn get_custom_crosshairs_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|p| p.join("crosshairs"))
        .map_err(|e| format!("Failed to get app data directory: {}", e))
}

/// List all available crosshair images
pub fn list_crosshairs<R: Runtime>(app: &AppHandle<R>) -> Result<Vec<CrosshairInfo>, String> {
    let mut crosshairs = Vec::new();

    // List built-in crosshairs
    if let Ok(builtin_dir) = get_builtin_crosshairs_dir(app) {
        if builtin_dir.exists() {
            crosshairs.extend(list_crosshairs_in_dir(&builtin_dir, true)?);
        }
    }

    // List custom crosshairs
    if let Ok(custom_dir) = get_custom_crosshairs_dir(app) {
        if custom_dir.exists() {
            crosshairs.extend(list_crosshairs_in_dir(&custom_dir, false)?);
        }
    }

    // Sort by name
    crosshairs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    info!("Found {} crosshairs", crosshairs.len());
    Ok(crosshairs)
}

/// List crosshair images in a specific directory
fn list_crosshairs_in_dir(dir: &Path, is_builtin: bool) -> Result<Vec<CrosshairInfo>, String> {
    let mut crosshairs = Vec::new();

    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // Check if it's a supported image format
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                if let Some(info) = CrosshairInfo::from_path(path, is_builtin) {
                    debug!("Found crosshair: {}", info.filename);
                    crosshairs.push(info);
                }
            }
        }
    }

    Ok(crosshairs)
}

/// Validate that a crosshair file exists and is a valid image
pub fn validate_crosshair<R: Runtime>(
    app: &AppHandle<R>,
    filename: &str,
) -> Result<PathBuf, String> {
    // Check built-in crosshairs first
    if let Ok(builtin_dir) = get_builtin_crosshairs_dir(app) {
        let builtin_path = builtin_dir.join(filename);
        if builtin_path.exists() && builtin_path.is_file() {
            return Ok(builtin_path);
        }
    }

    // Check custom crosshairs
    if let Ok(custom_dir) = get_custom_crosshairs_dir(app) {
        let custom_path = custom_dir.join(filename);
        if custom_path.exists() && custom_path.is_file() {
            return Ok(custom_path);
        }
    }

    Err(format!("Crosshair not found: {}", filename))
}

/// Import a custom crosshair from an external path
pub fn import_crosshair<R: Runtime>(
    app: &AppHandle<R>,
    source_path: &Path,
) -> Result<CrosshairInfo, String> {
    // Validate it's an image file
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or("Invalid file extension")?;

    if !SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
        return Err(format!("Unsupported image format: {}", ext));
    }

    // Get the custom crosshairs directory
    let custom_dir = get_custom_crosshairs_dir(app)?;

    // Create directory if it doesn't exist
    if !custom_dir.exists() {
        std::fs::create_dir_all(&custom_dir)
            .map_err(|e| format!("Failed to create custom crosshairs directory: {}", e))?;
    }

    // Get the filename
    let filename = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?;

    let dest_path = custom_dir.join(filename);

    // Copy the file
    std::fs::copy(source_path, &dest_path)
        .map_err(|e| format!("Failed to copy crosshair: {}", e))?;

    info!("Imported custom crosshair: {}", filename);

    CrosshairInfo::from_path(dest_path, false)
        .ok_or_else(|| "Failed to create crosshair info".to_string())
}

/// Delete a custom crosshair
pub fn delete_crosshair<R: Runtime>(app: &AppHandle<R>, filename: &str) -> Result<(), String> {
    let custom_dir = get_custom_crosshairs_dir(app)?;
    let path = custom_dir.join(filename);

    if !path.exists() {
        return Err(format!("Crosshair not found: {}", filename));
    }

    // Make sure we're not deleting a built-in crosshair
    if let Ok(builtin_dir) = get_builtin_crosshairs_dir(app) {
        if path.starts_with(&builtin_dir) {
            return Err("Cannot delete built-in crosshairs".to_string());
        }
    }

    std::fs::remove_file(&path).map_err(|e| format!("Failed to delete crosshair: {}", e))?;

    info!("Deleted custom crosshair: {}", filename);
    Ok(())
}

/// Get the URL for a crosshair image (for use in the webview)
pub fn get_crosshair_url<R: Runtime>(app: &AppHandle<R>, filename: &str) -> Result<String, String> {
    let path = validate_crosshair(app, filename)?;

    // Convert to a file:// URL or asset URL depending on platform
    // For Tauri, we use the asset protocol
    let url = format!("asset://localhost/{}", path.display());

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crosshair_info_from_path() {
        let path = PathBuf::from("/path/to/crosshair.png");
        let info = CrosshairInfo::from_path(path, true).unwrap();

        assert_eq!(info.filename, "crosshair.png");
        assert_eq!(info.name, "crosshair");
        assert!(info.is_builtin);
        assert!(!info.is_custom);
    }

    #[test]
    fn test_supported_extensions() {
        assert!(SUPPORTED_IMAGE_EXTENSIONS.contains(&"png"));
        assert!(SUPPORTED_IMAGE_EXTENSIONS.contains(&"svg"));
        assert!(!SUPPORTED_IMAGE_EXTENSIONS.contains(&"txt"));
    }
}
