# GitHub Instructions for CrossOver

This document provides guidance for GitHub Copilot and other AI assistants working with this repository.

## Project Overview

CrossOver is a crosshair overlay desktop application built with **Tauri 2** (Rust backend + TypeScript/SCSS frontend). It creates a transparent, always-on-top window displaying a customizable crosshair for gaming applications.

## Repository Structure

```
crossover/
├── src-tauri/           # Rust backend (Tauri)
│   ├── src/             # Rust source files
│   │   ├── main.rs      # App entry point
│   │   ├── commands.rs  # IPC commands
│   │   ├── state.rs     # Application state
│   │   ├── window.rs    # Window management
│   │   ├── hotkeys.rs   # Global shortcuts
│   │   ├── mouse.rs     # Mouse following
│   │   ├── tray.rs      # System tray
│   │   ├── crosshair.rs # Crosshair utilities
│   │   └── config.rs    # Constants
│   ├── Cargo.toml       # Rust dependencies
│   └── tauri.conf.json  # Tauri config
├── src-frontend/        # TypeScript frontend
│   ├── main.ts          # Frontend entry
│   └── styles/          # SCSS styles
├── public/              # Static assets
│   ├── crosshairs/      # Crosshair images
│   └── sounds/          # Sound effects
├── index.html           # HTML entry
├── vite.config.ts       # Vite config
├── tsconfig.json        # TypeScript config
└── package.json         # Node dependencies
```

## Technology Stack

- **Tauri 2** - Desktop app framework
- **Rust** - Backend language
- **TypeScript** - Frontend language
- **Vite** - Build tool
- **SCSS** - Stylesheets

## Key Commands

```bash
# Development
npm run tauri:dev        # Run app in dev mode
npm run dev              # Vite dev server only

# Building
npm run tauri:build      # Production build
npm run build            # Frontend only

# Code Quality
npm run lint             # ESLint
npm run format           # Prettier
npm run check            # TypeScript check

# Rust specific
cd src-tauri && cargo check
cd src-tauri && cargo clippy
cd src-tauri && cargo test
```

## Code Style Guidelines

### TypeScript
- Use tabs for indentation
- No semicolons
- Double quotes for strings
- Follow ESLint/Prettier config

### Rust
- Follow standard rustfmt style
- Use Clippy recommendations
- Document public APIs with doc comments

### SCSS
- Use variables from `_variables.scss`
- Follow BEM-like naming conventions
- Keep specificity low

## Important Patterns

### IPC Communication
Frontend communicates with Rust via Tauri commands:

```typescript
// Frontend (TypeScript)
import { invoke } from '@tauri-apps/api/core';
await invoke('command_name', { arg1: value1 });
```

```rust
// Backend (Rust)
#[tauri::command]
async fn command_name(arg1: String) -> Result<(), String> {
    // implementation
}
```

### Event System
Use Tauri events for backend → frontend communication:

```rust
// Emit from Rust
app.emit("event-name", payload)?;
```

```typescript
// Listen in TypeScript
import { listen } from '@tauri-apps/api/event';
await listen('event-name', (event) => { ... });
```

### State Management
App state is managed in `src-tauri/src/state.rs` using `parking_lot::RwLock` for thread-safe access.

## Platform Considerations

- **macOS**: Requires Cocoa APIs for proper overlay behavior
- **Windows**: Uses Win32 layered window APIs
- **Linux**: Behavior depends on window manager/compositor

## Common Tasks

### Adding a new Tauri command:
1. Add function in `src-tauri/src/commands.rs`
2. Register in `main.rs` invoke handler
3. Call from frontend via `invoke()`

### Adding a new hotkey:
1. Update `src-tauri/src/hotkeys.rs`
2. Add handler function
3. Update `src-tauri/src/state.rs` if preference needed

### Adding a new UI component:
1. Add HTML in `index.html`
2. Add TypeScript logic in `src-frontend/main.ts`
3. Add styles in `src-frontend/styles/index.scss`

## Dependencies

### Key npm packages:
- `@tauri-apps/api` - Tauri JavaScript API
- `@tauri-apps/plugin-*` - Official plugins

### Key Cargo crates:
- `tauri` - Core framework
- `rdev` - Mouse/keyboard hooks
- `parking_lot` - Synchronization
- `serde` - Serialization

## Testing

Currently, testing is done via:
- `cargo test` for Rust unit tests
- Manual testing for UI/integration

## Resources

- [Tauri Documentation](https://tauri.app/v2/guides/)
- [Tauri API Reference](https://tauri.app/v2/api/)
- [Rust Book](https://doc.rust-lang.org/book/)