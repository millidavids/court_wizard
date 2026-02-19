# Court Wizard - Project Documentation

## Project Overview

Court Wizard is a real-time strategy game built with Rust and the Bevy game engine (v0.17.3). The game compiles to WebAssembly (WASM) for browser deployment.

## Technology Stack

- **Language**: Rust (edition 2024)
- **Game Engine**: Bevy 0.17.3
- **Target Platforms**: Native (Linux/Windows/Mac) and WebAssembly (browser)
- **Key Dependencies**: serde, toml, thiserror, anyhow, rand

## Project Structure

```
src/
├── config/           # Game configuration and level definitions
├── game/             # Core game logic
│   ├── battlefield/  # Battlefield setup and rendering
│   ├── input/        # Input handling systems
│   ├── pathfinding/  # Flow field pathfinding system
│   │   ├── constants.rs    # Pathfinding constants (rally points, satisfaction radii)
│   │   ├── components.rs   # FlowFieldInfluence component
│   │   ├── messages.rs      # ObstacleChanged messages
│   │   ├── flow_field.rs   # FlowField struct and generation
│   │   ├── resources.rs    # PathfindingGrid resource
│   │   └── systems.rs      # Pathfinding systems
│   ├── runes/        # Rune system for spell activation
│   └── units/        # All unit types and behaviors
│       ├── archer/   # Archer defender units
│       ├── infantry/ # Infantry defender units (includes DefendersActivated resource)
│       ├── king/     # King unit and systems
│       └── wizard/   # Wizard and spell systems
│           └── spells/
│               ├── black_hole/      # Black hole gravity spell
│               ├── wall_of_stone/   # Wall obstacle spell
│               └── ...              # Other spells
├── state/            # Game state management
└── ui/               # User interface components
    ├── action_bar/   # Bottom action bar with spell slots
    ├── game_over/    # Game over screen
    ├── in_game/      # In-game UI overlays
    ├── instructions/ # Instructions screen
    ├── main_menu/    # Main menu
    ├── pause_menu/   # Pause menu
    ├── rune_display/ # Rune display system
    ├── spell_book/   # Spell selection interface
    └── version/      # Version display
```

## Architecture Patterns

### ECS (Entity Component System)
The game uses Bevy's ECS architecture:
- **Entities**: Units, spells, UI elements
- **Components**: Data attached to entities (Transform, Velocity, Health, Team, etc.)
- **Systems**: Functions that operate on entities with specific components
- **Resources**: Global state (PathfindingGrid, DefendersActivated, GameOutcome, etc.)

### Module Visibility
- Use `pub(super)` for items only needed within a module
- Use `pub(crate)` for crate-internal APIs
- Only use `pub` for true public API (typically just plugins and exported types)
- Keep sub-modules private, only re-export the plugin in `mod.rs`

### Plugin Architecture
Each major system is organized as a Bevy plugin:
- `GamePlugin` - Main game coordinator
- `PathfindingPlugin` - Flow field pathfinding
- `UnitsPlugin` - All unit types
- `RunePlugin` - Rune system
- UI plugins for each screen

### System Organization
Systems are grouped into sets with explicit ordering:
- `VelocitySystemSet` - Calculate targeting and flocking (parallel, immutable queries)
- `MovementSystemSet` - Apply movement to units (after velocity calculations)
- Post-movement: Combat, death conversion, win/lose checks

### Code Sharing
**CRITICAL**: Maximize code sharing between systems, units, and spells:
- Units (infantry, behemoth, king, archer) share movement and targeting code via `src/game/units/systems.rs`
- Spells likely have similar patterns (spawning, lifetime, effects, cleanup) that should be extracted into shared functions
- When adding new units or spells, check existing implementations for shared patterns
- Extract common logic into shared functions rather than duplicating code
- Unit-specific or spell-specific behavior should be minimal overrides on top of shared systems

## Key Systems

### General Systems
- You **MUST** only run systems when they need to be run leveraging run_if conditionals.

### Movement System
- Force-based physics with acceleration and velocity
- External forces: black hole gravity, wall avoidance, flocking
- Flow fields replace direct targeting for pathfinding around obstacles
- Smooth turning with lerp interpolation
- Speed modifiers: effectiveness, king aura, melee slowdown, rough terrain

### Combat System
- Global attack cycle timer (staggers attacks across all units)
- Team-based targeting (Defenders vs Attackers vs Undead)
- Health, damage multipliers, effectiveness calculations

### Win/Lose Conditions
- Victory: All attackers and undead dead
- Defeat: King dies (immediate) OR all defenders dead
- Persistent spell effects delay victory/defeat (except king death)

## Build Instructions

### Development (Wasm)
**IMPORTANT**: After completing ANY task that modifies Rust code, you MUST run (except changelog changes):
```bash
./build_wash.sh
```

### Production Build (WASM)
```bash
./build_wasm.sh --release
```

This script:
- Builds the WASM binary with optimizations
- Copies assets to the web directory
- Prepares the game for browser deployment

The user tests in a web browser, so changes will NOT be visible until the WASM is rebuilt.

### Changelog
- Make sure that the changelog is generated in laymans terms, no need to reference code changes
- Add new features and improvements
- Fix bugs and issues
- Never add [Unreleased] sections to the changelog
- Don't spoil achievements or unlockables in the changelog
- Don't reveal any secrets or hidden content in the changelog

### Testing
```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Important Conventions

### Constants and Styles Organization
- Game-wide constants in `src/game/constants.rs`
- Module-specific constants in dedicated `constants.rs` files (e.g., `pathfinding/constants.rs`)
- **Colors and style values go in `constants.rs`** — do NOT create separate `styles.rs` files. Keep all constants (dimensions, colors, positions, etc.) together in one file per module.
- **Messages go in `messages.rs`** — This project uses Bevy 0.17 **Messages** (`#[derive(Message)]`) for broadcast inter-system communication. All message types in a module should live in a dedicated `messages.rs` file, never `events.rs`. Name message structs with a `Message` suffix (e.g., `StartBrewMessage`), never `Event`. **Bevy Events** (`#[derive(Event)]`) are a separate, newer mechanism for reactive observer/trigger callbacks (entity-targeted via `world.trigger()` / `On<E>`). We don't currently use Events, but if we do, they would go in a separate `events.rs` file and use the `Event` suffix.
- Use `pub(super)` for module-internal constants

### Error Handling
- Use `Result<T, E>` for fallible operations
- Use `thiserror` for error types
- Never use `.unwrap()` in production code
- Use `.expect()` only for invariants with descriptive messages

### Logging
- Use `info!`, `warn!`, `error!` from `bevy::log`
- Avoid excessive logging in production code
- Remove debug logging after debugging is complete

### Git Workflow
- Never commit unless explicitly instructed
- This file (CLAUDE.md) is gitignored - it's for development reference only

## MCP Usage
- Context7 for fetching documentation
- code-indexing should be run when builind context for quick lookups and code base structure knowledge
