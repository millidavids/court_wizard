# Project Configuration

## Project Context

This is a Rust language project using the Bevy game engine.

**Technology Stack:**
- Language: Rust
- Game Engine: Bevy

**Development Environment:**
- Development/Compilation: WSL2 Ubuntu 24.04
- Target Platform: WASM (browser deployment)

## Documentation

Always use Context7 MCP tools to fetch up-to-date documentation for:
- Rust language features and standard library
- Bevy game engine APIs and patterns

This ensures accurate, current information for code generation and API usage.

## Architecture & Design Patterns

### Configuration System Architecture

**Single Source of Truth Pattern**

The configuration system uses Bevy components as the single source of truth:

```
localStorage (persistent storage)
    ↕ (load/save only)
ConfigFile (temporary serialization struct, NOT a Resource)
    ↕ (apply at startup, build at save)
Bevy Components (single source of truth at runtime)
    - Window component (window settings)
    - GameConfig resource (game settings)
```

**Key Principles:**
- ConfigFile is NEVER a runtime Resource - only exists briefly during load/save
- Bevy components are authoritative during runtime
- No duplicate state that can diverge
- Changes are detected on Bevy components, not config structs

**Unified Debouncing with Bridge Pattern**

All configuration changes use unified message-based debouncing:

```
Bevy Component Changes → Bridge Systems → ConfigChanged message →
Unified Debounce (2s) → Save (reads from components)
```

**Bridge Systems:**
- `bridge_window_resize_to_config_changed()` - WindowResized → ConfigChanged
- `bridge_game_config_to_config_changed()` - GameConfig changes → ConfigChanged

**Benefits:**
- Scalable: Adding new config types just requires sending ConfigChanged
- Unified: Single debounce timer for ALL config changes
- Clean: Clear separation between external events and internal changes

**Adding New Config Types:**

To add a new config type (e.g., audio settings):
1. Create/use a Bevy resource for the settings
2. Create a bridge system that detects changes and sends ConfigChanged
3. Update `build_config_from_components()` to read from the new resource
4. Update `apply_X_config()` to apply settings at startup

Example:
```rust
fn bridge_audio_to_config_changed(
    audio: Res<AudioSettings>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    if audio.is_changed() {
        config_changed.write(ConfigChanged);
    }
}
```

## Bevy Best Practices

**Always query Context7 for Bevy-specific patterns and APIs.**

When implementing Bevy features (states, messages, events, systems, queries, components, resources, etc.), use Context7 MCP tools to get up-to-date Bevy 0.17 documentation and best practices rather than relying on static documentation in this file.

### Project-Specific Bevy Decisions

**State Management:**
- All states centralized in `src/state/` module
- Primary states: `AppState` (MainMenu, InGame, Paused, GameOver)
- Sub-states defined where needed (e.g., `MenuState` only exists when in MainMenu)

**Message-Based Architecture:**
- Use Bevy Messages (Events) for cross-plugin communication
- Bridge pattern: External events → Bridge systems → Unified messages
- Example: `ConfigChanged` message for unified debounced config saves

## Code Quality

**Linting and Warnings**

Always run `cargo clippy` after making code changes to lint for errors and warnings. Fix all clippy warnings and suggestions before completing a task.

When clippy reports issues:
- First, attempt to use `cargo clippy --fix` to automatically fix issues
- After auto-fix, run `cargo clippy` again to verify all issues are resolved
- For any remaining issues that couldn't be auto-fixed, manually address them
- Follow Rust best practices and idioms

**Dead Code Policy**

Remove dead code rather than suppressing warnings with `#[allow(dead_code)]`:
- Delete unused functions, constants, types, and imports
- Only use `#[allow(dead_code)]` as a last resort when code must remain for a specific reason
- Keep the codebase clean and minimal

## Agent Usage

When spawning specialized agents for complex tasks:
- **Plan agents**: Use `opus` model for designing new features, complex architectural changes, or major refactoring efforts
- **Explore agents**: Use `haiku` model for simple codebase searches and quick file exploration
- **General-purpose agents**: Use `sonnet` model for balanced research and multi-step tasks

This ensures optimal performance and cost-efficiency for different types of work.

## Shell Configuration

Use zsh as the default shell for this project to ensure cargo and rust toolchain are available.

```json
{
  "shell": "zsh"
}
```

## WASM Build Process

**CRITICAL**: This project compiles to WASM and runs in the browser. After making ANY code changes that are ready for user testing, you MUST run:

```bash
./build_wasm.sh
```

**When to Build:**
- After completing a feature or bug fix
- Before asking the user to test changes
- Before saying "try it now" or similar phrases
- After any Rust code changes that affect gameplay or UI

**The user tests in a web browser, so changes will NOT be visible until the WASM is rebuilt.**

The build script:
1. Compiles for `wasm32-unknown-unknown` target
2. Runs `wasm-bindgen` to generate JavaScript bindings
3. Outputs to `./web/` directory
