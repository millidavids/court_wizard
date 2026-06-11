## ui-overlays

**Scope:** `src/ui/focus/`, `src/ui/spell_book/`, `src/ui/game_over/`, `src/ui/pause_menu/`

---

## Mental model

These four modules form the in-game and post-game overlay layer. `focus/` is the gamepad navigation substrate used by all overlays: it tracks the currently focused entity, drives spatial nearest-neighbour navigation, provides scroll animation, and scopes focus to modal roots. `spell_book/` is a transient overlay (enter/exit `InGameState::SpellBook` or `MultiplayerGameState::SpellBook`) that lets the player browse, preview, and hotkey-assign spells. `game_over/` fires on `OnEnter(InGameState::ScoreScreen)`, flushes all end-of-level state to disk (walls, crystals, terrain, run stats, efficiency), and presents the victory/defeat UI. `pause_menu/` wraps four sub-plugins: the main pause screen (stats + nav), SP settings (reused from main menu), MP settings (same systems, different back-state), and delegating sub-screens (manual, compendium) implemented elsewhere. All Update systems are state-gated, no bare `.unwrap()` calls exist in scope, and module purity is broadly respected.

---

## Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `ui/game_over/saves.rs:38-45` vs `saves.rs:235-241` vs `screen/layout.rs:57` | Medium | S | Efficiency calculation (`1.0 - defenders_killed / total_defenders`) is duplicated three times in the same module, each with minor surface-level variation (one multiplies by 100, two differ in variable naming). This is the canonical game metric — any change must be made in three places. | Extract `fn calculate_efficiency(kill_stats: &KillStats) -> f32` returning a `[0,1]` ratio to one place in `saves.rs`, then call it from all three sites. |
| F2 | ArchitecturalDecay | `ui/spell_book/interaction.rs:211-231` vs `interaction.rs:259-278` | Medium | S | The hotkey-button visual update loop (iterate `all_hotkey_buttons`, insert/remove `ButtonActive` + `ButtonColors` based on `config.action_bar_slots`) is copy-pasted identically between `handle_hotkey_click` (lines 211–231) and `handle_number_key_assignment` (lines 259–278). | Extract `fn sync_hotkey_button_visuals(commands: &mut Commands, buttons: &Query<...>, config: &GameConfig, spell: Spell)` and call it from both handlers. |
| F3 | DocDrift | `ui/game_over/screen/layout.rs:19-22` | Low | S | The doc comment on `setup_game_over_screen` says "Saves efficiency for current level to config" and "This system runs BEFORE setup_game_over_screen" — both are false. The function only spawns the score-screen UI. The comment was copy-pasted from the sibling `save_efficiency_to_config` system. | Replace with an accurate one-liner describing what the function actually does: spawns the score-screen UI entities. |
| F4 | ConsistencyRot | `ui/pause_menu/main/systems/actions.rs:12,61` | Low | S | `button_action` and `relabel_continue_for_coop` are declared `pub` (bare crate-public). They are only ever consumed by the sibling `plugin.rs` via `use super::systems::*`. No external module imports them. | Change both to `pub(super)` and update the re-export in `systems/mod.rs` to `pub(super) use actions::*`. |
| F5 | ArchitecturalDecay | `ui/pause_menu/mp_settings.rs` vs `ui/pause_menu/settings/plugin.rs` | Low | M | `MpPauseSettingsPlugin` (138 LOC) and `PauseSettingsPlugin` (85 LOC) are structurally identical: same imported systems, same `Update` batches, same `ButtonActionSet` grouping, identical debug-assertions block. They differ only in the back-state target and escape handler name. All shared Update systems are registered twice, independently. | Extract a shared registration function `fn register_settings_systems<S: States + FreelyMutableState>(app: &mut App, state: S)` and call it from both plugins, keeping only the state-specific back-action and escape handler separate. |
| F6 | Performance | `ui/focus/navigation.rs:76` | Low | S | `nearest_in_direction` materialises `candidates: impl IntoIterator` into a `Vec` unconditionally at line 76, even though the horizontal-nav single-pass branch only iterates it once. This allocation occurs on every focus-navigation frame. | For the horizontal branch, iterate the `IntoIterator` directly without collecting. Collect to a slice only for the vertical two-pass branch that calls `nearest_pass` twice. |
| F7 | ConsistencyRot | `ui/game_over/screen/layout.rs:403-416` vs `pause_menu/main/systems/spawn.rs:315-327` | Low | S | Seed display (small grey text, `"Seed: {}"`, `font_size 14.0`, `Color::srgba(0.6, 0.6, 0.6, 0.8)`, absolute-positioned) is copy-pasted with identical node parameters and styling in both overlay setup functions. | Extract `fn spawn_seed_label(parent: &mut ChildSpawnerCommands, seed: u64)` into `ui/systems.rs` and call it from both sites. |

---

## Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `src/ui/focus/navigation.rs` | 448 | Yes | Single cohesive concern: spatial focus direction resolution and navigation helpers. All functions share the same query types and geometry constants. Splitting would scatter a tightly coupled algorithm. |
| `src/ui/focus/scroll.rs` | 363 | Yes | Single cohesive concern: scroll subsystem — `ScrollAnimation` component, `autoscroll_to_focused`, `animate_scroll`, `right_stick_scroll`, focus-memory helpers. No foreign concerns. |
| `src/ui/spell_book/setup.rs` | 385 | Yes | Entirely UI spawn code for a single screen. The two private helpers (`spawn_detail_panel`, `spawn_spell_list`) are tightly coupled sub-routines of `spawn_spell_book_ui`. Exempt as a single UI construction concern. |
| `src/ui/spell_book/interaction.rs` | 315 | No | Three concerns: button-action dispatch (`button_action`, `despawn_*`), detail-panel update (`update_detail_panel`), and duplicated hotkey assignment (`handle_hotkey_click` / `handle_number_key_assignment`). Proposed split: `button_actions.rs`, `detail_panel.rs`, `hotkey.rs`. Fixing F2 alone drops this below 280 lines. |
| `src/ui/game_over/saves.rs` | 302 | No | Mixes two concerns: game-world persistence on victory (walls, crystals, terrain, efficiency) and run-stats accumulation (`accumulate_mode_level_stats`). Proposed split: `persistence.rs` and `stats.rs`. |

---

## Looks bad but is actually fine

- **`systems.rs` 4-line re-export shims** in `focus/` and `spell_book/`: the "Phase 16" split pattern — `systems.rs` forwards from named concern files so `plugin.rs` keeps a single import path. Not a violation.
- **`update_detail_panel` has 9 system parameters with `Without<T>` filter stacks** (interaction.rs:55–108): required by Bevy's aliasing rules to borrow multiple `&mut Text` in one system. The `#[allow(clippy::too_many_arguments)]` is appropriate per project convention.
- **`load_unified_save()` in `setup_game_over_screen` and `spawn_spell_book_ui`**: both are `OnEnter` systems that fire once at state entry. The synchronous file I/O is the project-wide pattern; no async runtime is available.
- **`tab_cycle` uses `unwrap_or(0)` for current-tab index (navigation.rs:436)**: safe fallback — if no tab has `ButtonActive`, cycle starts from tab 0, which is the correct default behaviour.
- **`accumulate_mode_level_stats` is `pub(crate)`** and re-exported to `crate::ui`: it is referenced from `src/steam/leaderboards/plugin.rs` for system ordering. The broad visibility is load-bearing.
- **`ScreenFocusMemory` grows per unique `ScreenKey` visited**: `ScreenKey` is a small `Copy` enum combination; the set of reachable keys is bounded by the number of distinct screens, not by session length.

---

## Open questions

1. `setup_game_over_screen` calls `load_unified_save()` synchronously at score-screen entry to read lifetime kill stats. Is there a plan to cache the unified save data in a Bevy resource so overlays don't hit disk on every screen entry?
2. `PauseSettingsPlugin` and `MpPauseSettingsPlugin` register entirely separate copies of every settings Update system bound to different states. Is a future unification into a shared `run_if(sp_or_mp_settings_state)` planned, or is strict state separation intentional for stability?
3. The `game_over/screen/` sub-directory has its own `mod.rs` wrapping `actions.rs` and `layout.rs`. The `game_over/systems.rs` is just a re-export hub forwarding from both `saves` and `screen`. Is the `screen/` sub-directory level of nesting intended to stay, or is it a Phase 16 artefact that could be flattened?
