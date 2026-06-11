## state

Scope: `src/state/` — state machine definitions and their registration plugin.

---

### Mental model

The `state` module is the authoritative source for all Bevy `States`/`SubStates` used across the game. It contains four files:

- `states.rs` (204 LOC): Enum definitions for `AppState`, `MenuState`, `InGameState`, `PauseMenuState`, `MetaGameState`, `SplashState`, and `MultiplayerGameState`, each with doc-comments describing transitions.
- `plugin.rs` (103 LOC): `StatePlugin` that initialises the primary state, registers sub-states, and — in debug builds only — wires up seven per-state transition-logging systems.
- `systems.rs` (92 LOC): Seven `#[cfg(debug_assertions)]` logging system bodies.
- `mod.rs` (9 LOC): Clean `mod` + `pub use` re-exports. No logic.

The module is small, well-structured, and stable. The most significant issues are: seven `Update` systems with no `run_if` guard (against project convention), stale `#[allow(dead_code)]` suppressors on fully-used enums, doc examples that cannot compile (binary crate, no lib.rs), and a single-variant sub-state that adds indirection with no current benefit.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| S1 | Performance | `plugin.rs:90–101` | Medium | S | Seven debug-logging systems are registered unconditionally in `Update` with no `run_if` guard. Each runs every frame and checks `Res::is_changed()` internally. Project convention requires every `Update` system to carry a `run_if` guard. Bevy 0.18 ships `state_changed::<S>()` in `bevy_state::condition` for exactly this pattern. | Replace each registration with `.run_if(state_changed::<S>())`, e.g. `log_app_state_transitions.run_if(state_changed::<AppState>())`. This eliminates seven per-frame resource fetches and aligns with the project run_if policy. |
| S2 | ConsistencyRot | `systems.rs:12` vs `systems.rs:22,34,47,60,72,86` | Low | S | `log_app_state_transitions` takes `Res<State<AppState>>` (non-Optional), while every other logging system takes `Option<Res<State<...>>>`. The asymmetry is correct but unexplained — a reader trap that implies the other states might not exist for an undocumented reason. | Add a brief `// AppState is always registered (init_state), so Option<> is unnecessary` comment on line 12, or make all seven `Option<Res<...>>` for visual uniformity. |
| S3 | ArchitecturalDecay | `states.rs:19` / `states.rs:56` | Low | S | `#[allow(dead_code)]` on `AppState` and `MenuState` with comment "Variants will be used as game features are implemented." Both enums are fully implemented: all 7 `AppState` variants and all 4 `MenuState` variants are actively referenced in the codebase. The suppressors are stale and could mask genuinely dead variants in future. | Remove both `#[allow(dead_code)]` attributes. |
| S4 | DocDrift | `plugin.rs:25,42,58` | Low | S | Three doc examples use `use court_wizard::state::AppState` — a public crate path that does not exist. The crate is a binary (`src/main.rs`; no `src/lib.rs`), so the `court_wizard::` namespace is inaccessible. `cargo test --doc` would fail on these examples. | Change the `use` paths in doc examples to `use crate::state::AppState` or add a `# fn main() {}` wrapper noting these are illustrative only. |
| S5 | ArchitecturalDecay | `states.rs:113–119` | Low | S | `MetaGameState` has exactly one variant (`WizardTower`). A single-variant sub-state provides no discriminated behavior — all code conditioning on `in_state(MetaGameState::WizardTower)` is equivalent to `in_state(AppState::MetaGame)`. The sub-state incurs registration cost and a pointless import in every consuming file. | Either add the anticipated additional variants now (e.g. `StudyDesk`, `Talents`) or collapse this sub-state and use `AppState::MetaGame` directly until a second variant is genuinely needed. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split into |
|------|-----|--------|---------------------|
| `src/state/states.rs` | 204 | true | Single-concern registry: pure enum definitions with doc-comments. Every line is cohesive state-machine surface area. No logic. |

No other file in scope exceeds 300 LOC.

---

### Looks bad but is actually fine

- **Seven separate logging functions instead of one generic helper**: The repetition looks like it should be `fn log_state_transition<S: States + Debug>`. In practice each function is guarded independently by `#[cfg(debug_assertions)]`, and extracting a generic helper would require the same attribute at the call site anyway. The ~5-line repetition poses no maintenance burden.
- **`Option<Res<State<T>>>` parameters in logging systems**: Using `Option<Res<…>>` for sub-states that may not exist is the correct Bevy pattern. It is not a bug — it handles the window before the parent state activates.
- **`MultiplayerGameState` duplicating `InGameState` variants** (`Paused`, `SpellBook`, `CauldronMenu`, `ScoreScreen`): Intentional isolation; the states.rs doc on line 179 explicitly says "Separate from InGameState to avoid any coupling with single-player systems." This is not accidental duplication.
- **Long doc comment on `StatePlugin` (plugin.rs:14–71)**: 58 lines of rustdoc examples look like over-documentation, but they are the canonical usage guide for downstream systems authors and are appropriate for a cross-cutting infrastructure plugin.

---

### Open questions

1. Is `MetaGameState::WizardTower` kept to future-proof the API for `StudyDesk` / `Talents` sub-tabs, or has the plan to split the WizardTower into sub-states been abandoned? Clarifying this determines whether S5 should be resolved now or closed as intentional.
2. `InGameState::Tutorial` (line 106) has no entry in the `InGameState` doc-comment transition table (lines 79–86). Is `Tutorial` entered from `Running` and exits back to `Running`? The transition should be documented.
3. `MultiplayerGameState::Settings` has no equivalent in `InGameState` (SP uses the deeper `PauseMenuState::Settings`). Is that asymmetry intentional (MP flattened its pause hierarchy) or an oversight?
