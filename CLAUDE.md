# Court Wizard - Project Documentation

## Project Overview

Court Wizard is a real-time strategy game built with Rust and the Bevy game engine (v0.18.1). The game targets native desktop platforms (Windows, Linux, macOS).

## Technology Stack

- **Language**: Rust (edition 2024)
- **Game Engine**: Bevy 0.18.1
- **Target Platforms**: Native (Windows, Linux, macOS)
- **Key Dependencies**: serde, toml, thiserror, anyhow, rand, dirs

## Project Structure

```
src/
├── config/           # Game configuration, save data, input bindings
├── game/             # Core game logic
│   ├── battlefield/  # Battlefield setup and rendering
│   ├── input/        # Input handling
│   ├── pathfinding/  # Flow field pathfinding
│   ├── runes/        # Rune system for spell activation
│   ├── achievements/ # Achievement tracking
│   ├── crt_effect/   # CRT post-processing pipeline
│   ├── cauldron/     # Brew/ingredient system
│   ├── multiplayer/  # P2P multiplayer
│   ├── loading/      # Queue-based progressive entity spawn
│   ├── game_mode/    # Endless / roguelite mode rules
│   ├── terrain/      # Boulders, ponds, props
│   ├── drops/        # Pickups
│   ├── benchmarking/ # Perf instrumentation
│   ├── seeded_rng/   # Deterministic RNG
│   └── units/        # All unit types and behaviors
│       ├── archer/, infantry/, king/, brute/, healer/, ...  # Per-unit modules
│       ├── boss/     # Boss enemies (hags, ogre, lich, dark_mage, ray)
│       └── wizard/   # Wizard, archetypes, spells (~30), talents
├── state/            # Game state machines (AppState, MenuState, etc.)
├── ui/               # User interface (one folder per screen / overlay)
├── networking/       # Iroh-backed transport for multiplayer
├── steam/            # Steam achievements / leaderboards
└── music/            # Background music tracks
```

## Architecture Patterns

### ECS (Entity Component System)
The game uses Bevy's ECS architecture:
- **Entities**: Units, spells, UI elements
- **Components**: Data attached to entities (Transform, Velocity, Health, Team, etc.)
- **Systems**: Functions that operate on entities with specific components
- **Resources**: Global state (PathfindingGrid, DefendersActivated, GameOutcome, etc.)

### Module Structure — Feature-Sliced & Granular

**Prefer many small concern-focused files over a few large canonical ones.** A `damage.rs` file holding `DamageMultiplier` component + `apply_damage` system + damage constants together is preferred over the same code split across `components.rs`/`systems.rs`/`constants.rs` mixed with 30 unrelated entries.

**Hard rules:**
- `plugin.rs` does Bevy plugin registration ONLY. Move system bodies and helpers to sibling files.
- `mod.rs` does `mod` declarations + `pub use` re-exports ONLY. No logic, no constants, no types.
- One `plugin.rs` per module. No per-concern micro-plugins inside a module.
- Files exceeding ~300 lines must be split unless every line is genuinely cohesive (e.g., a single large match-on-enum or a single asset registry).
- `styles.rs` is forbidden. Constants live with their feature, or in a `constants.rs` for cross-cutting values only.

**Feature-slicing rule:** When splitting a module with multiple concerns, group by concern, not by file type. Reserve canonical names (`components.rs`, `systems.rs`, `constants.rs`) for genuinely cross-cutting / shared content.

**Module-shape examples:**

Self-contained spell module (`units/wizard/spells/fireball/`):
```
plugin.rs    # registration only
casting.rs   # input + cast initiation
projectile.rs # component + movement + collision
explosion.rs # secondary visuals + AoE damage
talents.rs   # talent params for fireball
```

Cross-cutting module (`units/`):
```
plugin.rs
core.rs           # Health, Team, Hitbox (truly shared)
combat.rs         # MeleeRangeBonus + melee targeting
movement.rs       # MovementSpeed + weighted movement
status_effects.rs # CC components + their tick systems
dots.rs           # FireDoT/ElectricCharge/etc. + processing
animation.rs      # animation components + animation systems
sprites.rs        # sprite materials + factories
```

Boss module (`units/boss/hags/`):
```
plugin.rs
spawn.rs
movement.rs
eye_transfer.rs
justina.rs
josephina.rs
martina.rs
constants.rs   # only if many shared constants; otherwise inline
```

**Constants:**
- A single `constants.rs` is fine for small modules.
- When a `constants.rs` exceeds ~200 lines or mixes visual and gameplay concerns, split into named files (`colors.rs`, `dimensions.rs`, `tuning.rs`) or inline constants into the feature files that own them.
- Constants used by exactly one feature file should be inlined there.

**Messages:**
- A single `messages.rs` is fine when messages span features.
- If only one feature file owns a message, put the message in that file.

**Cross-cutting markers:** Truly shared types (e.g., `OnGameplayScreen`, base `Health`, `Team`) may stay in a `core.rs` or `components.rs` — pick whichever name reads best.

### Module Visibility
- Use `pub(super)` for items only needed within a module
- Use `pub(crate)` for crate-internal APIs
- Use `pub(in crate::game)` for items shared inside the game module tree
- Only use `pub` for true public API (typically just Plugin types)

### System Organization
Systems are grouped into sets with explicit ordering:
- `VelocitySystemSet` - Calculate targeting and flocking (parallel, immutable queries)
- `MovementSystemSet` - Apply movement to units (after velocity calculations)
- Post-movement: Combat, death conversion, win/lose checks

### Code Sharing
**CRITICAL**: Maximize code sharing between systems, units, and spells:
- Units (infantry, behemoth, king, archer) share movement and targeting helpers in the `units/` cross-cutting feature files
- Spells share casting boilerplate via `src/game/units/wizard/spells/utils.rs` — use `build_wizard_input`, `cleanup_spell_caster`, `handle_spell_release`, `update_indicator_position`, `try_start_cast_with_indicator`, and `commit_spell_cast` instead of duplicating input/indicator/cleanup logic
- Bosses share telegraph/spawn helpers via `src/game/units/boss/utils.rs`
- When adding new units or spells, check existing implementations for shared patterns
- Extract common logic into shared functions rather than duplicating code
- Unit-specific or spell-specific behavior should be minimal overrides on top of shared helpers

### Function Arguments & SystemParam
- **Bevy systems** naturally have many injected parameters (Res, ResMut, Query, Commands, etc.). When a system exceeds 7 arguments, add `#[allow(clippy::too_many_arguments)]` — this is idiomatic Bevy, not a code smell.
- **Helper functions** (non-system functions called from other code) should keep argument counts reasonable. When multiple helpers share the same parameter group, extract a params struct (e.g., `CrystalAutocastParams` in arcane_crystal).
- **Constructors** with many fields: prefer `#[allow(clippy::too_many_arguments)]` on `new()` since the arguments map 1:1 to struct fields.
- **Do NOT** create `#[derive(SystemParam)]` bundles just to reduce argument counts — this obscures what systems actually access and makes code harder to follow. Only use `SystemParam` when a group of resources/queries is reused across 3+ systems with identical parameter sets.

### Component Design
**Prefer small, focused Components over monolithic data structs.** This is core to Bevy's ECS design:
- **Status effects and conditions** (e.g., sleepwalking, burning, comatose) should be their own `#[derive(Component)]` structs — not boolean flags or optional fields inside a larger modifier component.
- **Behavioral modifiers** that drive dedicated systems (DPS ticks, movement overrides, spreading effects) should be separate components so systems can query/filter on them directly with `With<T>`, `Without<T>`, and `any_with_component::<T>`.
- **Numeric-only modifiers** applied at cast time (damage multipliers, radius multipliers) are fine as fields on the parent component or in a params struct — they don't need their own component since no system queries on them independently.
- **Rule of thumb**: If a piece of data drives its own system or needs to be queried/filtered independently, it should be its own component. If it's just a number read once at cast time, it can stay as a field.

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

### Iterative compile checks (USE THIS during work)
While iterating — refactoring, splitting files, fixing import errors — use `cargo check` instead of the full build script. It runs the full compiler frontend (so it catches every type error, missing import, visibility issue, etc.) but skips codegen, making it ~5-10× faster.

```bash
cargo check --target=x86_64-pc-windows-gnu
```

This is the right tool for the inner loop of mechanical refactors (file splits, import fixes, visibility tweaks). Don't run `./scripts/build_native.sh windows` between every change — it links the binary and copies assets, which is wasteful when you're just verifying the code compiles.

You can also use `cargo fix --bin court_wizard --allow-dirty --target=x86_64-pc-windows-gnu` to auto-remove unused imports surfaced by `cargo check`.

### Final build (when handing off to the user)
Run the full build only when you're done with a feature/task and ready to hand the work back.

The game is play-tested on Windows, so the usual hand-off build is the Windows cross-compile:
```bash
./scripts/build_native.sh windows
```

**Check the toolchain is actually present before promising this build.** The Windows target needs both the Rust target and a MinGW linker:
- `rustup target add x86_64-pc-windows-gnu`
- the linker — `apt install gcc-mingw-w64-x86-64` on Linux/WSL2, `brew install mingw-w64` on macOS

Without the linker, `cargo check --target=x86_64-pc-windows-gnu` fails at `cc-rs`, not at your code. If it isn't installed, say so rather than reporting an unverified build; plain `cargo check` / `cargo build` on the host target still validates the code.

A Linux build additionally needs system dev packages (libasound2-dev, libwayland-dev, libxkbcommon-dev, libudev-dev).

### Release Build
```bash
./scripts/build_native.sh windows --release
```

This script:
- Builds the native binary with the specified target and profile
- Copies assets alongside the binary so the game can find them at runtime
- Save data is stored in the platform data directory (e.g., `%APPDATA%/court_wizard/` on Windows)

### Changelog
- Make sure that the changelog is generated in laymans terms, no need to reference code changes
- Add new features and improvements
- Fix bugs and issues
- Never add [Unreleased] sections to the changelog
- Don't spoil achievements or unlockables in the changelog
- Don't reveal any secrets or hidden content in the changelog
- `docs/CHANGELOG.md` is `include_str!`'d into the binary (`src/ui/manual/systems.rs`) for the in-game Changelog screen. Editing it changes the shipped bits — which is why the release flow builds the changelog-lock commit rather than promoting an earlier build.

### Release pipeline
Driven by the `/game-release` skill; see `.claude/skills/game-release/SKILL.md` and `steam/README.md`.

```
/game-release        → push dev  → dev-release.yml → test/clippy/fmt, build all 3 platforms, Steam `staging`
/game-release main   → push main → release.yml     → tag + GitHub Release from that build, request Steam `default`
```

- **The version is assigned on `dev`, not at promotion.** `docs/CHANGELOG.md` is `include_str!`'d into the binary, so every changelog edit changes the shipped bits. The top block of the changelog is always the open, already-numbered, already-dated version; each dev release appends to it. There is no `[pending]` block.
- **A dev release is a full three-platform signed build.** `dev-release.yml` fires on any push to `dev` that touches `docs/CHANGELOG.md`.
- **Promotion changes no files and builds nothing.** `/game-release main` is a pure fast-forward, so the binary that goes live is byte-identical to the one play-tested on `staging`. The corollary is that the dev tip must already have a `dev-release.yml` run — promoting a commit that was never built leaves `release.yml` with nothing to promote.
- **`release.yml` builds nothing.** It reuses the `dev-release.yml` run for the same commit, which works because promotion fast-forwards `main` onto the built dev commit.
- **Promotion to Steam's default branch needs a phone tap.** Valve always sends a Steam Mobile confirmation for a released app's default branch. CI picks the build id and requests it; never describe a `main` release as live until that's approved.
- Steamworks announcements are generated, not posted (`scripts/changelog_to_bbcode.sh`) — Steam has no supported events API.

### Testing
```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Important Conventions

### Constants Organization
- Game-wide constants live in `src/game/constants.rs`. Split by concern when that file exceeds ~200 lines.
- Module-specific constants either live in feature files alongside the code that uses them, or in a `constants.rs` if they are shared across feature files.
- **`styles.rs` is forbidden.** Colors, dimensions, and styling go in feature files or `constants.rs`.
- Use `pub(super)` for module-internal constants.

### Messages
- Use Bevy 0.17 **Messages** (`#[derive(Message)]`) for broadcast inter-system communication. Name message structs with a `Message` suffix (e.g., `StartBrewMessage`), never `Event`.
- A single `messages.rs` per module is fine when messages span features. If only one feature file owns a message, put the message in that file.
- **Never use `events.rs`.** **Bevy Events** (`#[derive(Event)]`) are a separate, newer mechanism for reactive observer/trigger callbacks (entity-targeted via `world.trigger()` / `On<E>`). We don't currently use them; if we do, they go in a separate `events.rs` file and use the `Event` suffix.

### Code Simplification
- Always `/simplify` before updating the changelog and before releasing to optimize the codebase from a duplication standpoint.

### Save Data Backwards Compatibility
- **All changes to types in `src/config/save_data.rs` must be backwards compatible with existing save files on disk.** Players have persistent progress and their saves cannot break across updates.
- Add new fields with `#[serde(default)]` so older saves without them still deserialize.
- Never rename or remove an existing field without a read-fallback path (keep the old field with `#[serde(default, skip_serializing_if = ...)]` and migrate its contents on load).
- When the on-disk format needs to change shape meaningfully, introduce a new field name and support both on read, writing only the new one. Don't rely on wiping player saves.

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
- This file (CLAUDE.md) is checked into the repo as development reference; it ships publicly with the open-source release, so keep it free of secrets and private machine details

## MCP Usage
- Context7 for fetching documentation
- code-indexing should be run when builind context for quick lookups and code base structure knowledge
