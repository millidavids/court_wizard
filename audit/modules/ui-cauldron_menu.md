## ui-cauldron_menu

**Scope:** `src/ui/cauldron_menu/` — the in-game overlay for selecting ingredients and starting brews.

---

### Mental model

The cauldron menu is a two-panel overlay (left: detail/preview, right: ingredient grid) that runs in both `InGameState::CauldronMenu` and `MultiplayerGameState::CauldronMenu`. It is driven by the `IngredientSelection` resource, which acts as the source of truth for what the player has toggled. On entering the state, `spawn_cauldron_menu_ui` builds the full tree; `rebuild_menu_on_brew_state_change` despawns it when a brew completes mid-display; `respawn_menu_on_toggle` re-creates it on the next frame if missing. Selection changes drive two partial-update systems: `update_detail_panel_on_selection_change` (rebuilds left panel children) and `sync_toggle_button_states` (restyles buttons in-place without despawning, preserving gamepad focus). The Philosopher's Stone selector is intentionally kept as a persistent child entity to survive the partial rebuild. The module is well-structured and granular.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| C1 | ArchitecturalDecay | `interaction/ingredient_clicks.rs:81–203` vs `setup/detail_panel.rs:77–207` | High | M | **Brew-preview rendering is copy-pasted between two files.** Both `update_detail_panel_on_selection_change` (the incremental rebuild path) and `spawn_detail_panel` (the full-spawn path) contain byte-identical logic for: ingredient count string, brew-time Alchemist multiplier, duration multiplier, effects list, dilution block, and combo block (~100 lines duplicated). Any future change (e.g., a new Alchemist bonus) must be applied twice and is silently inconsistent. | Extract a `spawn_brew_preview_children(parent, selection, recipe, is_alchemist, unlocked_combos)` helper into `setup/detail_panel.rs` and call it from both sites. |
| C2 | DocDrift | `interaction/brew_actions.rs:10` | Low | S | The doc comment on `button_action` reads `"Spawns the cauldron menu UI when entering the CauldronMenu state."` This is the doc comment for the spawner function (`spawn_cauldron_menu_ui`) copy-pasted onto the wrong function. `button_action` handles click/action dispatch, not spawning. | Replace with an accurate doc comment describing click handling and state transition. |
| C3 | ConsistencyRot | `constants.rs:52,70,88` | Low | S | `DETAIL_LABEL_COLOR`, `INGREDIENT_COUNT_FULL_COLOR`, and `COMBO_COLOR` are three separately-named constants all equal to `Color::hsla(270.0, 0.50, 0.60, 1.0)`. This creates confusion about whether they are intentionally distinct values. | Unify as a single named constant (e.g., `ACCENT_PURPLE`) and alias or replace the three names, or add a comment explaining they are intentionally the same accent color. |
| C4 | ConsistencyRot | `constants.rs:215–236` | Low | S | `BREW_BUTTON_STYLE` and `CANCEL_BUTTON_STYLE` hardcode `width: 160.0`, `height: 40.0`, `border_width: 1.0` as raw literals, while all five ingredient/stone button styles (lines 147–208) correctly reference `BUTTON_WIDTH`, `BUTTON_HEIGHT`, `BUTTON_BORDER_WIDTH`. If button size is changed, action buttons will silently diverge. | Replace the raw literals in `BREW_BUTTON_STYLE` and `CANCEL_BUTTON_STYLE` with the named constants. |
| C5 | ArchitecturalDecay | `interaction/ingredient_clicks.rs:26` | Low | S | `update_detail_panel_on_selection_change` has a redundant internal early-exit guard (`if !selection.is_changed() { return; }`) even though the system is already registered with `.run_if(resource_changed::<IngredientSelection>)` in `plugin.rs:47`. The outer `run_if` is sufficient; the inner guard is dead code in normal operation. | Remove the `if !selection.is_changed()` guard from inside the system body. |
| C6 | ConsistencyRot | `setup/build_menu.rs:137` | Low | S | `build_menu` references `crate::ui::main_menu::BACK_BUTTON_STYLE` rather than `crate::ui::constants::BACK_BUTTON_STYLE`. Both are different sizes (150×50 vs 140×50) so the in-game cauldron Back button uses the main-menu large style rather than the shared in-game back style. Other in-game overlays use the shared constant. | Confirm whether this is intentional. If not, switch to `crate::ui::constants::BACK_BUTTON_STYLE`. |

---

### Oversized files

| File | LOC | Exempt | Reason | Proposed split |
|------|-----|--------|--------|----------------|
| `interaction/ingredient_clicks.rs` | 274 | No | Large primarily because it contains ~100 lines of brew-preview rendering duplicated from `detail_panel.rs`. After extracting the shared helper (finding C1), this file should fall well under 200 LOC. | Extract brew-preview child-spawning helper to `setup/detail_panel.rs`; remainder stays in `ingredient_clicks.rs`. |

No other files exceed 300 LOC.

---

### Looks bad but is actually fine

- **`format!("{:?}", i)` as unlock key** (`setup/ingredient_list.rs:42`): Using the `Debug` representation of `Ingredient` as the string key for `unlocked_ingredients` looks fragile, but this is a project-wide convention enforced by `wizard_crud.rs:55` and used identically in `compendium/` — renaming an enum variant would break saves regardless of this pattern, so it is load-bearing and consistent.
- **`respawn_menu_on_toggle` runs every frame** (`setup/build_menu.rs:53`): Polling `menu_query.iter().next().is_none()` every frame of `CauldronMenu` state looks expensive, but the query is O(1) (at most one entity), the state is short-lived, and this is the standard Bevy single-spawn-menu pattern.
- **`load_unified_save()` calls in `build_menu.rs:71` and `ingredient_clicks.rs:51`**: Looks like repeated disk reads, but `save_cache.rs` maintains an in-memory `Mutex<Option<UnifiedSaveFile>>` — all reads after the first serve from cache.
- **`Option<ResMut<NextState<...>>>` dual-optional state transitions in `brew_actions.rs:23–24`**: Looks like missing error handling, but is the correct Bevy idiom when a SubState may or may not exist depending on SP vs MP mode — the in-code comment correctly explains the reasoning.
- **`rebuild_menu_on_brew_state_change` + `respawn_menu_on_toggle` no explicit ordering**: Both fire in the same `run_if` group. `rebuild` only fires when `CauldronState` is `Changed`, and `respawn` only fires when no menu entity exists — they complement each other cleanly.
- **`IngredientSelection` is `pub(super)` but methods are `pub`**: Technically wider than needed, but the resource has module-internal visibility and methods are needed by sibling files; this is idiomatic Rust visibility layering.

---

### Open questions

1. Is the `crate::ui::main_menu::BACK_BUTTON_STYLE` reference in `build_menu.rs:137` intentional (larger 150×50 in-game back style) or an oversight (should be the shared 140×50 `ui::constants::BACK_BUTTON_STYLE`)?
2. Does `IngredientSelection::build_ingredients()` including `Ingredient::PhilosophersStone` in the returned `Vec` cause any double-application risk in the cauldron brew system, or does the system handle the Stone specially?
3. The `respawn_menu_on_toggle` doc comment says "Re-spawns the menu UI if it was despawned by a toggle action" — but the despawn is done by `rebuild_menu_on_brew_state_change` (brew completion), not a toggle. Is the comment misleading?
