## state

Scope: `src/state/` — state machine definitions and their registration plugin.

---

### Mental model

The `state` module is the authoritative source for all Bevy `States`/`SubStates` used across the game. It contains three files:

- `states.rs` (204 LOC): Enum definitions for `AppState`, `MenuState`, `InGameState`, `PauseMenuState`, `MetaGameState`, `SplashState`, and `MultiplayerGameState`, each with doc-comments describing transitions.
- `plugin.rs` (179 LOC): `StatePlugin` that initialises the primary state, registers sub-states, and — in debug builds only — wires up seven per-state transition-logging systems.
- `mod.rs` (8 LOC): Clean `mod` + `pub use` re-exports. No logic.

The module is small, well-structured, and stable. The most significant issue is a **plugin.rs purity violation**: seven non-trivial system bodies (`log_*_state_transitions`) live directly in `plugin.rs`. Everything else is low-severity polish.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| S1 | ArchitecturalDecay | `plugin.rs:99–179` | Medium | S | Seven `log_*_state_transitions` system bodies defined in `plugin.rs`. Project convention forbids system bodies in `plugin.rs` (registration only). | Move the seven logging systems to a new `src/state/debug_logging.rs` (gated `#[cfg(debug_assertions)]`), `pub(super)` visibility, and import from `plugin.rs`. |
| S2 | Performance | `plugin.rs:84–95` | Medium | S | The seven debug-logging systems run unconditionally every Update frame (no `run_if` guard). They check `is_changed()` internally, but Bevy still schedules and invokes them every frame, including the `Option<Res<State<T>>>` parameter resolution. | Add `.run_if(in_state(...))` where the sub-state is active, or consolidate into a single generic helper scheduled only when any state changes. For sub-states that can be absent (`Option<Res<…>>`), at minimum wrap in `.run_if(state_exists::<T>())` to avoid scheduling overhead when the sub-state is not alive. |
| S3 | ConsistencyRot | `states.rs:19,56` | Low | S | `AppState` and `MenuState` carry `#[allow(dead_code)]` with the comment "Variants will be used as game features are implemented." Both states are fully implemented. `InGameState`, `MetaGameState`, `PauseMenuState`, `MultiplayerGameState`, and `SplashState` have no such attribute. The attributes are stale. | Remove the two `#[allow(dead_code)]` attributes. |
| S4 | DocDrift | `states.rs:14–16` | Low | S | The `AppState` doc-comment state-transition table lists `InGame → MetaGame` (win) and `InGame → Loading` (lose/retry), but does not document `InGame → MainMenu` (quit) or `MetaGame → Loading` (start next battle) symmetrically — one is listed, the other is in the prose below. Minor inconsistency between the transitions table and the comment body. | Align the transitions table so all valid arcs are listed once, consistently. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split into |
|------|-----|--------|---------------------|
| `src/state/states.rs` | 204 | true | Single-concern registry: pure enum definitions with doc-comments. Every line is cohesive state-machine surface area. No logic. |
| `src/state/plugin.rs` | 179 | true | Under 300 LOC. Noted violation (S1) is medium severity but the file size itself is not the threshold concern. |

---

### Looks bad but is actually fine

- **`Option<Res<State<T>>>` parameters in logging systems**: Using `Option<Res<…>>` for sub-states that may not exist is the correct Bevy pattern. It is not a bug.
- **No `run_if` on `OnEnter`/`OnExit` schedules**: The plugin only registers sub-states; the `OnEnter`/`OnExit` guards happen in consuming plugins. Not a violation here.
- **`MetaGameState` has only one variant (`WizardTower`)**: Looks like a placeholder, but the sub-state is used as a gating condition in UI plugins — having a single-variant state is a valid design for future extensibility.
- **`MultiplayerGameState` duplicating `InGameState` variants**: `Paused`, `SpellBook`, `CauldronMenu`, `ScoreScreen` appear in both. This is intentional isolation (comment in `states.rs` line 179: "Separate from InGameState to avoid any coupling with single-player systems") — not accidental duplication.

---

### Open questions

1. `MultiplayerGameState::Settings` has no equivalent in `InGameState` (SP uses `PauseMenuState::Settings` as a sub-sub-state). Is that asymmetry intentional, or did settings navigation get flattened for MP and nobody updated the SP path?
2. `InGameState::Tutorial` has no corresponding entry in the `AppState` doc-comment transition table. Is `Tutorial` entered from `Running` and exits back to `Running`? The transitions should be documented.
3. `MultiplayerLoading` (an `AppState` variant) has no corresponding sub-state, while `Loading` is bare too. Is there a `MultiplayerLoadingState` planned, or is the loading sequence driven purely by `AppState`-gated systems?
