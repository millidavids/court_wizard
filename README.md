# Court Wizard

A real-time strategy / castle-defense game where you play the court wizard — hurling spells over the heads of your own troops to turn the tide of battle. Built entirely in **Rust** with the **Bevy** game engine, shipped on Steam, and open source under the GPL.

This README doubles as a guided tour of the project. Beyond the usual "how to build it," it explains *how the codebase is put together and why* — if you're learning Rust, curious about Bevy, or wondering what a shipped, non-trivial ECS game actually looks like, start here.

## About the Game

You are the wizard. Your army fights on its own; your job is to shape the battle with magic — and magic is indiscriminate, so every fireball that lands on the enemy line can just as easily land on your own infantry.

- **Endless** — infinitely scaling waves; how far can you go?
- **Roguelite** — a fixed 25-level run with modifiers and progression between battles
- **Multiplayer** — peer-to-peer co-op (a friend casts alongside you) and 1v1 versus duels

30+ spells, a dozen unit types on both sides, boss encounters, talents, brewing, achievements, controller support, and a CRT screen filter for good measure.

- **Player instructions:** [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md)
- **Changelog:** [docs/CHANGELOG.md](docs/CHANGELOG.md)

## Why This Codebase Is Worth Reading

Court Wizard is roughly **164,000 lines of Rust across ~1,400 files**, and it's a *complete* game — not a demo or tutorial project. That makes it a useful reference for things that are hard to find realistic examples of:

- **ECS architecture at scale** — hundreds of units pathfinding, flocking, and fighting every frame, organized with Bevy's Entity-Component-System model
- **Peer-to-peer multiplayer** — NAT-traversing networking with host-authoritative simulation and snapshot replication
- **Steam integration** — achievements, leaderboards, stats, Steam Input, and rich presence through safe Rust bindings
- **Cross-compilation** — the game is developed on Linux (WSL2) and cross-compiled for Windows
- **Release engineering** — CI that builds every platform and deploys to Steam on a push to `main`

## The Tooling

### Rust

[Rust](https://www.rust-lang.org/) is a systems programming language that delivers C/C++-class performance without a garbage collector — which matters for games, where a GC pause in the middle of a frame is a visible stutter. What you get instead is *ownership*: the compiler tracks who owns every piece of memory and when it can be freed, so whole categories of crashes (use-after-free, data races, null dereferences) are compile-time errors rather than 2 a.m. bug reports.

A few Rust features do heavy lifting throughout this codebase:

- **Enums + pattern matching.** Game logic is full of "this thing is in exactly one of N states" — game modes, spell phases, button actions, network messages. Rust enums make illegal states unrepresentable, and `match` forces every case to be handled. See `src/state/states.rs` for the game's state machines.
- **`Result` and explicit errors.** Fallible operations (file I/O, save parsing, networking) return `Result` and are handled at the call site — the project bans `.unwrap()` in production code. Error types are derived with the `thiserror` crate.
- **Conditional compilation.** Developer tools (debug overlays, cheat buttons, hitbox visualizers) are wrapped in `#[cfg(debug_assertions)]`, so they are *compiled out* of release builds entirely — not hidden behind a runtime flag, but absent from the shipped binary. `src/game/debug_ui.rs` is a clean example.
- **Traits and derive macros.** One line — `#[derive(Component)]` — turns a plain struct into something the engine can attach to entities. Serialization (`#[derive(Serialize)]`), error types, and more work the same way.

**Cargo** is Rust's build system and package manager, configured by `Cargo.toml`. Three parts of ours are worth understanding:

- **Dependencies** are declared once with versions; `Cargo.lock` pins the exact resolved set so every build is reproducible.
- **Features** are compile-time switches. Our `benchmarking` feature compiles in an FPS/diagnostics overlay for performance testing — and because it defaults to off, none of that code exists in a normal build.
- **Profiles** control optimization. The `[profile.release]` section enables fat LTO (whole-program optimization), a single codegen unit, `panic = "abort"`, and symbol stripping — trading long compile times for the fastest, smallest shipping binary. The `dev` profile keeps builds fast for iteration.

### Bevy

[Bevy](https://bevy.org/) (we're on **0.18**) is a free, open-source game engine written in Rust. Unlike Unity or Unreal, there's no editor application — the engine is a library, and your game is a Rust program that uses it. Bevy's defining idea is that *everything* is built on an **Entity-Component-System (ECS)** architecture:

- An **Entity** is just an ID — a soldier, a fireball, a UI button.
- **Components** are plain data structs attached to entities: `Health`, `Team`, `MovementSpeed`, `Transform` (position). An "archer" is nothing more than an entity that happens to have archer-ish components.
- **Systems** are ordinary functions that run every frame and operate on *all* entities with a given component combination. A system with the query `Query<(&mut Transform, &Velocity)>` moves everything that has both a position and a velocity — units, projectiles, whatever — with no inheritance hierarchy in sight.
- **Resources** are global singletons: the pathfinding grid, the kill counter, the current level.

Why this shape? Two reasons. **Cache-friendliness:** components of the same type are stored contiguously, so iterating "all Health values" streams through memory the way CPUs like. **Parallelism:** Bevy inspects what each system reads and writes and automatically runs non-conflicting systems on different threads — hundreds of units' targeting, flocking, and animation update concurrently with no locks in game code.

On top of the core model, this project leans on:

- **Plugins.** Every feature module exposes a `Plugin` that registers its systems — the whole game assembles itself from dozens of plugins in `src/main.rs` → `src/game/plugin.rs`. This is what keeps a 164k-line project navigable.
- **States.** Menus, loading, gameplay, and pause are `AppState`/`InGameState` machines (`src/state/`); systems declare which state they run in, and entering/exiting a state triggers setup/teardown systems.
- **Run conditions.** Every frame-by-frame system is gated with `run_if(...)` so it only executes when relevant — a project-wide rule that keeps idle screens from burning CPU.
- **System sets & ordering.** Where order matters, systems are grouped into sets: targeting/steering runs in `VelocitySystemSet` (parallel, read-only queries), then movement applies in `MovementSystemSet`, then combat resolves. See `src/game/sets.rs`.
- **Messages.** Decoupled modules communicate by broadcasting typed messages (`StartBrewMessage`-style) instead of calling each other directly — a writer fires, any number of readers react next frame.
- **Custom shaders.** The CRT filter (`src/game/crt_effect/`) is a WGSL post-processing pipeline, including cursor-position correction to compensate for the barrel distortion it applies — a fun read if you're into rendering.

### The Supporting Cast

The rest of the dependency list, and what each crate does:

| Crate | Role |
|---|---|
| `iroh` | Peer-to-peer QUIC networking with NAT traversal — the multiplayer transport (`src/networking/`) |
| `tokio` | Async runtime that drives the networking I/O alongside Bevy's synchronous frame loop |
| `bevy-steamworks` / `steamworks` | Safe Rust bindings to the Steamworks SDK — achievements, leaderboards, stats, Steam Input (`src/steam/`) |
| `serde` + `toml` | Serialization of config and save data to human-readable TOML |
| `bincode` | Compact binary encoding for network messages |
| `thiserror` | Ergonomic custom error types |
| `rand` | Randomness — routed through a seeded RNG layer (`src/game/seeded_rng/`) so multiplayer peers generate identical battlefields from a shared seed |
| `tracing` | Structured logging with *compile-time* level stripping: `info!` calls literally don't exist in release builds |
| `dirs` | Cross-platform "where do I save files?" paths |
| `crossbeam-channel` | Lock-free channels bridging async networking threads and the game loop |
| `arboard`, `image`, `winit`, `webbrowser` | Clipboard, icon decoding, windowing, and opening URLs |

### Developer Tooling

- **`cargo check`** — compiles without generating code; the fast inner loop (seconds, not minutes) used constantly during development.
- **`cargo clippy -- -D warnings`** — Rust's lint suite, treated as errors. **`cargo fmt`** — non-negotiable formatting. Both run before any release.
- **Cross-compilation** — `cargo build --target=x86_64-pc-windows-gnu` produces a Windows `.exe` from Linux via the MinGW toolchain. The build scripts wrap this.
- **CI/CD** — `.github/workflows/build.yml` and `release.yml`: a push to `main` builds Windows/Linux, attaches zips to a GitHub Release, and uploads to Steam. macOS is built separately by `macos-release.yml` (manual dispatch or a pushed `v*` tag), which signs, notarizes, and staples a universal `Court Wizard.app` before uploading — macOS runners bill at 10x, so it never runs on ordinary pushes.
- **Benchmark builds** — `./scripts/build_native.sh --benchmarking` produces a release-speed binary with diagnostics compiled in, for profiling on real hardware.

## Project Setup

### Prerequisites

1. **Rust** via [rustup](https://rustup.rs/) — the standard toolchain installer. The stable channel is fine (the project uses edition 2024, so keep the toolchain current).
2. **Linux native builds** need system dev packages for audio/windowing:
   ```bash
   sudo apt install libasound2-dev libwayland-dev libxkbcommon-dev libudev-dev
   ```
3. **Cross-compiling for Windows** (how this project is primarily developed, from WSL2):
   ```bash
   rustup target add x86_64-pc-windows-gnu
   sudo apt install mingw-w64
   ```

### Building

```bash
./scripts/build_native.sh                   # debug build, host platform
./scripts/build_native.sh windows           # debug build, Windows (from Linux/WSL2)
./scripts/build_native.sh windows --release # optimized release build
./scripts/build_native.sh macos             # macOS dev build only (Apple Silicon; macos-intel for x86)
./scripts/package.sh macos                  # macOS shippable build: universal Court Wizard.app (unsigned locally)
```

The script builds the binary **and copies `assets/` next to it** so the game finds its sprites, audio, and shaders at runtime — run the produced binary from its output folder, not via `cargo run`. Two behaviors worth knowing: debug builds **auto-bump the patch version** in `Cargo.toml` (pass `--no-bump` to skip), and release builds never bump.

During development, skip the full build:

```bash
cargo check --target=x86_64-pc-windows-gnu   # fast: full type checking, no codegen
cargo clippy -- -D warnings                  # lints
cargo test                                   # tests
```

Steam features (achievements, leaderboards, controller glyphs) activate when Steam is running; the debug build copies a `steam_appid.txt` beside the binary so a local build can talk to your Steam client.

### Save Data & Crash Logs

| OS | Location |
|---|---|
| Windows | `%APPDATA%\court_wizard\` |
| Linux | `~/.local/share/court_wizard/` |
| macOS | `~/Library/Application Support/court_wizard/` |

If the game crashes, a `crash.log` with the panic details is written to that folder (most recent crash only). Please attach it to bug reports.

## Directory Structure

Top level:

```
court_wizard/
├── assets/          # Sprites, audio, shaders, fonts — copied next to the binary at build time
├── docs/            # Changelog, player instructions, credits, policies
├── scripts/         # build_native.sh, package.sh, upload_to_steam.sh
├── src/             # All game code (see below)
├── Cargo.toml       # Dependencies, features, build profiles
└── CLAUDE.md        # The project's architecture conventions, kept in-repo
```

Inside `src/`, the top level separates the game from its supporting concerns:

```
src/
├── main.rs           # Entry point: window setup, plugin registration
├── crash_handler.rs  # Panic hook that writes crash.log
├── config/           # Settings, input bindings, save data (with strict backwards compatibility)
├── state/            # AppState / InGameState / MetaGameState machines
├── game/             # The game itself (below)
├── ui/               # One folder per screen: main_menu, wizard_tower, pause_menu, spell_book, ...
├── networking/       # iroh transport, snapshots, CRDTs for multiplayer
├── steam/            # Steamworks: achievements, leaderboards, stats, Steam Input
└── music/            # Background music tracks
```

And `game/` is where most of the action is:

```
src/game/
├── battlefield/      # Arena setup and rendering
├── pathfinding/      # Flow-field pathfinding (units navigate a shared vector field)
├── units/            # Every unit type — infantry, archer, brute, healer, assassin,
│   │                 #   shielder, teleporter, king, undead, bosses, ...
│   └── wizard/       # The player: archetypes, talents, and spells/ (30+ spell modules)
├── multiplayer/      # Co-op & versus session logic on top of networking/
├── loading/          # Queue-based progressive spawning (no loading hitches)
├── game_mode/        # Endless / roguelite rules and modifiers
├── cauldron/         # Brewing system
├── crt_effect/       # CRT post-processing shader pipeline
├── seeded_rng/       # Deterministic randomness for multiplayer lockstep terrain
├── terrain/, drops/, achievements/, input/, runes/, benchmarking/
└── plugin.rs         # Assembles all of the above
```

**The navigation trick:** every module follows the same shape, so once you've read one, you can read them all. A module has a `plugin.rs` (system registration *only* — no logic), a `mod.rs` (declarations and re-exports *only*), and then one file per concern — `fireball/`, for example, keeps its cast handling (`casting.rs`) separate from its projectile flight (`projectile/`). Code is grouped by *feature*, not by kind: the damage component, the damage system, and the damage constants live together in the file about damage, not scattered across a `components.rs` / `systems.rs` / `constants.rs`. The full conventions — including how shared helpers keep the 30+ spells from duplicating casting boilerplate — are written down in [CLAUDE.md](CLAUDE.md).

A good first reading path: `src/main.rs` → `src/game/plugin.rs` → `src/state/states.rs` → pick one small spell module (e.g. `src/game/units/wizard/spells/fireball/`) and trace it from cast to explosion.

## License & Credits

Court Wizard is developed by **Blackhearth Games LLC** and released under the **GNU General Public License** — see [LICENSE](LICENSE). Art and audio attribution lives in [docs/CREDITS.md](docs/CREDITS.md).
