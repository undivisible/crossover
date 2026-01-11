# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CrossOver is a customizable crosshair overlay application built with **Tauri 2** (Rust backend + Web frontend). It allows users to place a crosshair overlay above any application window for improved aim in games.

## Recent Migration Notes (2026-01-04)

### Successfully Migrated from Electron to Tauri 2
- ✅ Removed all Electron-specific code
- ✅ Tauri project moved from subdirectory to repository root
- ✅ All plugin configurations fixed for Tauri 2.x API
- ✅ Migrated from deprecated `cocoa` crate to modern `objc2` ecosystem for macOS
- ✅ Implemented real platform-specific overlay functionality (no stubs)
- ✅ All global hotkeys working with proper per-shortcut handlers
- ✅ System tray fully functional with lock state indication
- ✅ Application compiles and runs without errors

### Key Technical Changes
1. **Plugin Configurations**: Tauri 2.x plugins don't accept configuration objects in `tauri.conf.json`
   - Removed `global-shortcut` config (expects unit type)
   - Removed `fs` config (uses capabilities system instead)
   - Removed `autostart` config (expects unit type)

2. **macOS Native APIs**: Replaced `cocoa 0.26` with `objc2 0.5` ecosystem
   - Using `objc2-app-kit` for NSWindow APIs
   - Direct pointer casting instead of Retained wrapper
   - Proper window level, collection behavior, and transparency settings

3. **Global Shortcuts**: Rewrote to use per-shortcut handlers
   - Each shortcut registered with `on_shortcut(shortcut, handler)`
   - Action-based dispatch instead of string parsing
   - Support for custom keybinds from preferences

## Commands

### Development
- `bun dev` - Start Vite dev server for frontend
- `bun tauri:dev` - Run the full Tauri app in development mode
- `bun run check` - TypeScript type checking

### Building
- `bun run build` - Build frontend assets only
- `bun tauri:build` - Build production app for current platform
- `bun tauri:build:mac` - Build for macOS (universal binary)
- `bun tauri:build:win` - Build for Windows
- `bun tauri:build:linux` - Build for Linux

### Code Quality
- `bun run lint` - Run ESLint on frontend source
- `bun run lint:fix` - Run ESLint with auto-fix
- `bun run format` - Format code with Prettier
- `bun run format:check` - Check formatting without changes

### Rust-specific
- `cd src-tauri && cargo build` - Build Rust backend
- `cd src-tauri && cargo check` - Check Rust code
- `cd src-tauri && cargo clippy` - Run Rust linter
- `cd src-tauri && cargo test` - Run Rust tests

## Architecture

### Technology Stack
- **Tauri 2** - Cross-platform desktop framework (Rust + WebView)
- **Rust** - Backend for window management, hotkeys, system integration
- **TypeScript** - Frontend application logic
- **Vite** - Frontend build tool
- **SCSS** - Stylesheet preprocessing

### Project Structure

```
crossover/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Entry point, Tauri setup
│   │   ├── commands.rs  # IPC commands (frontend → backend)
│   │   ├── state.rs     # App state & preferences
│   │   ├── window.rs    # Platform-specific window setup
│   │   ├── hotkeys.rs   # Global keyboard shortcuts
│   │   ├── mouse.rs     # Mouse following feature
│   │   ├── tray.rs      # System tray menu
│   │   ├── crosshair.rs # Crosshair image utilities
│   │   └── config.rs    # Constants and configuration
│   ├── Cargo.toml       # Rust dependencies
│   └── tauri.conf.json  # Tauri configuration
├── src-frontend/        # TypeScript/SCSS frontend
│   ├── main.ts          # Frontend entry point
│   └── styles/          # SCSS stylesheets
├── public/              # Static assets
│   ├── crosshairs/      # Default crosshair images
│   ├── crosshairs-library/  # Full crosshair library
│   └── sounds/          # Sound effects
├── index.html           # Main HTML entry
├── vite.config.ts       # Vite configuration
├── tsconfig.json        # TypeScript configuration
└── package.json         # Node.js dependencies
```

### Key Features Implementation

- **Crosshair Overlay** - Transparent, click-through window using platform-specific APIs
- **Global Hotkeys** - Default: `Ctrl+Alt+Shift+[Key]` via `tauri-plugin-global-shortcut`
- **Multiple Monitors** - Support for moving crosshair between displays
- **Shadow Windows** - Up to 14 duplicate crosshair windows
- **Mouse Following** - Optional tracking via `rdev` crate
- **System Tray** - Quick access menu for common actions
- **Preferences** - Persistent storage via `tauri-plugin-store`

### Platform-Specific Code

The `window.rs` module contains platform-specific implementations:
- **macOS**: Uses Cocoa APIs for overlay window level and collection behavior
- **Windows**: Uses Win32 APIs for layered windows and extended styles
- **Linux**: Standard X11/Wayland support (compositor-dependent)

## Code Style

### TypeScript/Frontend
- **Indentation**: Tabs
- **Semicolons**: None (enforced by ESLint)
- **Quotes**: Double quotes
- **Formatting**: Prettier

### Rust/Backend
- **Style**: Standard Rust formatting (`cargo fmt`)
- **Linting**: Clippy for additional checks
- **Documentation**: Doc comments for public APIs

## Default Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Lock/Unlock | `Ctrl+Shift+Alt+X` |
| Center | `Ctrl+Shift+Alt+C` |
| Hide/Show | `Ctrl+Shift+Alt+H` |
| Reset | `Ctrl+Shift+Alt+R` |
| Next Display | `Ctrl+Shift+Alt+M` |
| Duplicate | `Ctrl+Shift+Alt+D` |
| Quit | `Ctrl+Shift+Alt+Q` |
| Move Up/Down/Left/Right | `Ctrl+Shift+Alt+Arrow` |

**Note**: On macOS, use `Option` instead of `Alt`.

## Important Notes

- Requires **Rust toolchain** (install via `rustup`)
- Requires **Node.js 18+** for frontend tooling
- macOS builds may require signing for distribution
- Windows builds create NSIS installer by default
- Linux requires WebKit2GTK development libraries
- **macOS transparency**: Enable `macos-private-api` in `tauri.conf.json` for full transparency support

## Current Status

- ✅ **Compiles without errors or warnings**
- ✅ **Runs successfully on macOS**
- ✅ **All platform-specific code implemented** (no TODOs or stubs)
- ✅ **Modern dependencies** (objc2, Tauri 2.x plugins)
- ⚠️ **Testing needed**: Windows and Linux platform-specific features
- ⚠️ **Frontend**: Needs development (Vite dev server ready)

## Tauri Plugins Used

- `tauri-plugin-global-shortcut` - Global hotkeys
- `tauri-plugin-store` - Preferences persistence
- `tauri-plugin-autostart` - Start on boot
- `tauri-plugin-dialog` - File dialogs
- `tauri-plugin-fs` - File system access
- `tauri-plugin-notification` - System notifications
- `tauri-plugin-shell` - Open URLs/files
- `tauri-plugin-process` - App lifecycle
- `tauri-plugin-os` - OS information
- `tauri-plugin-updater` - Auto-updates
