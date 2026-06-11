## ui-overlays

**Scope:** `src/ui/focus/`, `src/ui/spell_book/`, `src/ui/game_over/`, `src/ui/pause_menu/`

---

## Mental model

This module covers four overlay-class UI surfaces that sit atop gameplay:

- **focus/** — A self-contained gamepad focus navigation engine. Provides spatial nearest-neighbor navigation (D-pad / left stick), hold-to-repeat, tab cycling (LB/RB), modal focus trapping, autoscroll-to-focused, right-stick scroll, and screen-keyed focus memory. Architecturally clean; split into `navigation.rs`, `scroll.rs`, `components.rs`, `resources.rs`, `constants.rs`, `plugin.rs`. The `systems.rs` is just a re-export shim.
- **spell_book/** — An in-game overlay (SP + MP) for spell selection, hotkey slot assignment, and detail preview. Split into `setup.rs` (UI spawn), `interaction.rs` (button handlers), `components.rs`, `constants.rs`, `plugin.rs`. Clean separation; `systems.rs` is a re-export shim.
- **game_over/** — Score-screen overlay rendered on `InGameState::ScoreScreen`. Handles stat display, efficiency calculation, roguelite run summary, and all post-battle save operations. Split into `screen.rs` (UI + button actions), `saves.rs` (save helpers), `components.rs`, `constants.rs`. Broadly clean but carries some duplicated efficiency math.
- **pause_menu/** — A three-sub-plugin pause overlay (`PauseMainPlugin`, `PauseSettingsPlugin`, `MpPauseSettingsPlugin`). `PauseSettingsPlugin` and `MpPauseSettingsPlugin` heavily reuse the main-menu settings systems; the only difference is the Back-action target state. `PauseMainPlugin` hosts the in-game stats panel + button actions.

---

## Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `ui/game_over/saves.rs:38-45` vs `saves.rs:235-241` vs `screen.rs:61-64` | Medium | S | Efficiency calculation (`1.0 - defenders_killed / total_defenders`) is duplicated three times in the same module, each with minor surface-level variation (one multiplies by 100, two differ in intermediate variable naming). This is the canonical game metric — any change to it must be made in three places. | Extract `fn calculate_efficiency(kill_stats: &KillStats) -> f32` returning a `[0,1]` ratio to a single place in `saves.rs` or a new `efficiency.rs`, then call it from all three sites. |
| F2 | ArchitecturalDecay | `ui/spell_book/interaction.rs:211-231` vs `interaction.rs:259-278` | Medium | S | The hotkey-button visual update loop (iterate `all_hotkey_buttons`, insert/remove `ButtonActive` + `ButtonColors` based on `config.action_bar_slots`) is copy-pasted identically between `handle_hotkey_click` (lines 211-231) and `handle_number_key_assignment` (lines 259-278). | Extract `fn sync_hotkey_button_visuals(commands: &mut Commands, buttons: &Query<...>, config: &GameConfig, spell: Spell)` and call it from both handlers. |
| F3 | DocDrift | `ui/game_over/screen.rs:26-29` | Low | S | The doc comment on `setup_game_over_screen` (lines 26-29) says "Saves efficiency for current level to config" and "This system runs BEFORE setup_game_over_screen". Both claims are false: the function does not save efficiency (that is `save_efficiency_to_config` in `saves.rs`), and the sentence structure refers to itself as if it is a different function. This comment was apparently copy-pasted from a sibling system. | Replace with accurate doc for `setup_game_over_screen`: it spawns the score-screen UI entities given the current game outcome and kill stats. |
| F4 | ConsistencyRot | `ui/pause_menu/main/systems.rs:22,338,387` | Low | S | `setup`, `button_action`, and `relabel_continue_for_coop` are declared `pub` (crate-public) but they are only consumed via `use super::systems::{...}` inside the sibling `plugin.rs`. No external module references them directly. | Change all three to `pub(super)` to enforce the module boundary. |
| F5 | ArchitecturalDecay | `ui/pause_menu/mp_settings.rs` vs `ui/pause_menu/settings/plugin.rs` | Low | M | `MpPauseSettingsPlugin` (138 LOC) and `PauseSettingsPlugin` (85 LOC) are structurally identical: same list of imported systems, same `Update` batches, same `ButtonActionSet` grouping, identical debug-assertions block. They differ only in the `Back` action destination state (one returns to `PauseMenuState::Main`, the other to `MultiplayerGameState::Paused`) and the escape handler function name. All shared Update systems are registered twice, independently. | Extract a shared registration function `fn add_settings_systems<S: States>(app: &mut App, state: S)` that takes the state variant, deduplicating the ~50 duplicated registration lines. The Back-action system remains state-specific; everything else unifies. |
| F6 | Performance | `ui/focus/navigation.rs:76` | Low | S | `nearest_in_direction` materialises `candidates: impl IntoIterator<Item = (Entity, Vec2)>` into a `Vec<(Entity, Vec2)>` unconditionally at the top (line 76), even though the horizontal-nav single-pass path iterates it only once. The collection happens on every focus navigation frame. | Remove the `collect()` and iterate the `IntoIterator` directly inside the horizontal-nav branch; the vertical branch already calls `nearest_pass` which accepts a `&[(Entity, Vec2)]` — pass a freshly collected slice only for that path, or restructure so the hot horizontal path does not allocate. |
| F7 | ConsistencyRot | `ui/game_over/screen.rs:413-420` vs `pause_menu/main/systems.rs:322-330` | Low | S | Seed display (small grey text, `"Seed: {}"`, `font_size 14.0`, `Color::srgba(0.6, 0.6, 0.6, 0.8)`) is spawned with identical node parameters and styling in two separate overlay setup functions. | Extract a `fn spawn_seed_label(parent: &mut ChildSpawnerCommands, seed: u64)` into `ui/systems.rs` (or a shared helper) and call it from both sites. |

---

## Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `src/ui/focus/navigation.rs` | 448 | Yes | All 448 lines are a single cohesive concern: focus direction resolution and navigation helpers. No unrelated code mixed in; splitting would scatter a tightly coupled algorithm. |
| `src/ui/focus/scroll.rs` | 363 | Yes | Scroll-specific systems and the `ScrollAnimation` component — a single cohesive scroll subsystem. No foreign concerns. |
| `src/ui/game_over/screen.rs` | 548 | No | Two distinct concerns share the file: (1) UI layout/spawning (`setup_game_over_screen`, ~250 lines), and (2) button-action dispatch (`handle_button_actions`, ~120 lines). Proposed split: `layout.rs` + `actions.rs`. |
| `src/ui/spell_book/setup.rs` | 385 | Yes | Entirely UI spawn code; the two helpers (`spawn_detail_panel`, `spawn_spell_list`) are tightly coupled sub-routines of `spawn_spell_book_ui`. Exempt as a single UI construction concern. |
| `src/ui/spell_book/interaction.rs` | 315 | Yes | Borderline (315 lines) but all systems belong to the spell-book interaction concern: spell selection, hotkey click, number-key assignment, detail-panel update, focus lifecycle. Exempt if F2 is fixed (removing the duplicate block brings it under 280 lines). |
| `src/ui/pause_menu/main/systems.rs` | 408 | No | Two concerns: (1) UI layout helpers (`setup`, `collect_left_panel_data`, `spawn_left_panel`, `spawn_right_panel`, ~270 lines) and (2) button-action + co-op relabeling logic (~138 lines). Proposed split: `layout.rs` + `actions.rs`. |

---

## Looks bad but is actually fine

- **`systems.rs` is just 4 lines of re-exports** in `focus/` and `spell_book/`: this is the project's "Phase 16" split pattern — `systems.rs` acts as a forwarding shim so `plugin.rs` can keep a single import path. Not a violation; the real code lives in named concern files.
- **`update_detail_panel` has 9 query parameters** (interaction.rs:55-108): the many `Without<T>` filter combinations are required by Bevy's aliasing rules to borrow `Text` on disjoint sets. The `#[allow(clippy::too_many_arguments)]` is appropriate here per project convention.
- **`load_unified_save()` called directly in `setup_game_over_screen` and `spawn_spell_book_ui`**: both are `OnEnter` systems that run once at state entry. The synchronous file I/O is intentional (no async runtime); it mirrors the pattern used throughout the codebase.
- **`tab_cycle` uses `unwrap_or(0)` for the current-tab index (navigation.rs:436)**: safe fallback — if no tab has `ButtonActive`, cycle starts from tab 0, which is correct behaviour.
- **`MpPauseSettingsPlugin` and `PauseSettingsPlugin` both list `spawn_confirmation_popup` in their imports but only `MpPauseSettingsPlugin` calls it directly**: `PauseSettingsPlugin` calls `pause_settings_button_action` which internally calls `spawn_confirmation_popup`. Not dead code.
- **`accumulate_mode_level_stats` is `pub(crate)`** in `saves.rs` and re-exported all the way to `crate::ui`: it is referenced from `src/steam/leaderboards/plugin.rs` for ordering (`.after(accumulate_mode_level_stats)`). The broad visibility is justified.

---

## Open questions

1. `setup_game_over_screen` calls `load_unified_save()` synchronously at score-screen entry to read lifetime kill stats. If the save file is large or on a slow path, this blocks the frame. Is there a plan to cache unified save data in a Bevy resource so overlays don't hit disk on every entry?
2. `PauseSettingsPlugin` and `MpPauseSettingsPlugin` register entirely separate copies of every settings Update system (sliders, key bindings, resolution, etc.) bound to different states. This means the compiler/scheduler sees two independent copies of e.g. `update_sliders` registered for two separate states. Is a future unification into a shared `run_if(sp_or_mp_settings_state)` planned, or is the strict state separation intentional for stability?
3. The `ScreenFocusMemory` resource grows unboundedly — one `Vec2` entry per unique `ScreenKey` ever visited. Given that `ScreenKey` is a `Copy` enum combination, the set is bounded in practice. But is there a deliberate purge strategy (e.g., on save-reset / new run) to prevent memory from accumulating across very long sessions?
