## ui-cauldron_menu

**Scope:** `src/ui/cauldron_menu/` — cauldron ingredient-selection overlay UI (spawn, interaction, detail panel, ingredient list, Philosopher's Stone selector).

---

### Mental model

The cauldron menu is a two-panel overlay: a left detail/preview panel (shows selected-ingredient brew stats) and a right categorized ingredient grid. It operates in both SP (`InGameState::CauldronMenu`) and MP (`MultiplayerGameState::CauldronMenu`) using the same UI handlers. `IngredientSelection` is a `Resource` that tracks the current selection; button actions mutate it and two `resource_changed`-gated systems then sync button visual state and rebuild the detail panel content. The menu is torn down on exit by despawning all `OnCauldronMenuScreen` entities. The Philosopher's Stone selector is kept alive as a persistent child (`StoneSelectorPanel`) to avoid dropping gamepad focus when the detail panel is rebuilt.

The module is structured across 7 files: `plugin.rs` (registration), `mod.rs` (re-exports), `components.rs` (all data types), `constants.rs` (all visual constants), `systems.rs` (re-export hub), `setup.rs` (spawn + rebuild), `interaction.rs` (button actions + detail panel update + sync).

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| CM-01 | ArchitecturalDecay | `setup.rs:245–376` vs `interaction.rs:143–291` | High | M | The "brew preview" detail-panel content (~130 lines) is rendered in two completely independent places: `spawn_detail_panel` in `setup.rs` (initial build) and `update_detail_panel_on_selection_change` in `interaction.rs` (incremental rebuild). Both implement identical logic — ingredient count, brew time with Alchemist bonus, buff duration with Alchemist bonus, effects list, dilution warning, combo list. A bug fix or wording change must be applied in both places. | Extract a `spawn_brew_preview_content(panel: &mut ChildSpawnerCommands, selection, config, unlocked_combos)` helper in a new `detail_panel.rs` (or inline in `setup.rs`) and call it from both sites. |
| CM-02 | Performance | `interaction.rs:99` | Low | S | `update_detail_panel_on_selection_change` already has `.run_if(resource_changed::<IngredientSelection>)` in `plugin.rs:47`. The manual `if !selection.is_changed() { return; }` guard at line 99 is therefore dead code — `selection.is_changed()` will always be `true` when the system runs. | Remove the redundant early-return guard. |
| CM-03 | Performance | `setup.rs:49–64` | Low | S | `respawn_menu_on_toggle` runs every Update frame while in CauldronMenu state. It performs two ECS queries on every frame to check `menu_query.iter().next().is_none()`. Under normal play the menu is always present, so this is constant wasted work. The condition it checks (menu entity absent) is only true for exactly one frame after `rebuild_menu_on_brew_state_change` despawns it. | Gate with a `Local<bool>` dirty flag or simply run the system as a follow-up `OnExit`/`OnEnter` callback after brew completes rather than polling every frame. |
| CM-04 | ConsistencyRot | `setup.rs:141` | Low | S | `crate::ui::main_menu::BACK_BUTTON_STYLE` is referenced by full module path. The canonical constant lives at `crate::ui::constants::BACK_BUTTON_STYLE` (re-exported through `main_menu::mod.rs`). Other files using the same style reach it via `crate::ui::constants`. Using the `main_menu` re-export creates an implicit coupling to that module's re-export chain. | Import from `crate::ui::constants::BACK_BUTTON_STYLE` directly (as done in `manual/constants.rs` and `wizard_tower/constants.rs`). |
| CM-05 | TypeContract | `setup.rs:427–428` | Medium | S | Ingredient unlock checks serialize enum variants via `format!("{:?}", i)` (debug format) and compare against the stored string. If an `Ingredient` enum variant is ever renamed or a custom `Debug` impl added, stored save data becomes inconsistent without any compile-time warning. The same fragile pattern appears in `interaction.rs:247` (`c.name.to_string()` for combos) and is widespread in the compendium module. | Add a `fn save_key(&self) -> &'static str` method to `Ingredient` (and `Recipe`/combo) that returns an explicit stable string, and use it everywhere. This is a cross-cutting issue but originates in cauldron_menu and compendium; the save-data auditor (config) should own the final fix. |
| CM-06 | ArchitecturalDecay | `interaction.rs:57–83` | Low | S | The "set both SP and MP state to Running" pattern (`if let Some(ref mut s) = next_in_game_state { s.set(Running); } if let Some(ref mut mp) = next_mp_state { mp.set(Running); }`) is repeated three times within `button_action` (for StartBrew, CancelBrew, and Close). | Extract a local closure or free function `close_menu(sp, mp)` to de-duplicate the three identical transition blocks. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|-------------------------|
| `setup.rs` | 580 | No | Large but not a match/registry monolith. Split into: `build_menu.rs` (top-level `build_menu` + page container), `detail_panel.rs` (shared `spawn_brew_preview_content` helper + `spawn_detail_panel`), `ingredient_list.rs` (`spawn_ingredient_list` + `spawn_ingredient_card`), `stone_selector.rs` (`spawn_philosophers_stone_selector`). |
| `interaction.rs` | 357 | No | Split into `button_action.rs` and `detail_panel_update.rs`. The detail-panel update logic is the bulk of the file and is already conceptually separate from button dispatch. |
| `constants.rs` | 236 | Yes | All constants; no logic. Under the project's 300-LOC limit for a purely constant file of this kind. |

---

### Looks bad but is actually fine

- **`systems.rs` is a 4-line re-export hub with a "Phase 16" comment.** This looks like leftover scaffolding but is the correct `mod.rs`-like indirection the project uses to keep plugin.rs clean. The phase comment is stale but harmless.
- **`update_detail_panel_on_selection_change` despawns children while skipping `StoneSelectorPanel` entities.** The selective child despawn with a `Contains` check looks fragile but is intentional and well-commented: it preserves gamepad focus on the Stone toggle button.
- **`respawn_menu_on_toggle` calls `build_menu` which calls `load_unified_save`.** `load_unified_save` is cache-backed (mutex-protected in-memory cache; disk only on first miss), so calling it from a spawn path is not a per-frame disk I/O issue.
- **`button_action` takes `Option<ResMut<NextState<InGameState>>>` and `Option<ResMut<NextState<MultiplayerGameState>>>`** — looks odd but is the established project pattern for systems that run in both SP and MP context; the comment at interaction.rs:29 explains why non-optional access would panic.
- **`STONE_SELECTED_STYLE.text_color` used as a color value inside detail panel render** — looks like leaking style internals, but the constant is a `ButtonStyle` whose `text_color` field is the correct gold color for the stone-selected state. No better named constant exists for this color.
- **All Update systems are gated by `run_if(in_state(InGameState::CauldronMenu).or(in_state(MultiplayerGameState::CauldronMenu)))` applied as a set-level condition in plugin.rs.** The outer `run_if` covers all six Update systems simultaneously, satisfying the "every Update system must have a run_if guard" requirement.

---

### Open questions

1. Should `respawn_menu_on_toggle` be replaced with a message-driven approach (e.g. a `RebuildCauldronMenuMessage` sent by `rebuild_menu_on_brew_state_change`) instead of the current polling design?
2. The `format!("{:?}", i)` ingredient key pattern is present here, in `compendium`, and in `save_data`. Is there a tracked issue to introduce a stable `save_key()` trait, or is this an accepted trade-off?
3. `spawn_philosophers_stone_selector` is `pub(super)` — it is only called from `setup.rs` (both `spawn_detail_panel` and `update_detail_panel_on_selection_change`). After the CM-01 refactor it would only need to be `pub(super)` within its own file, which is fine, but the current split between setup.rs and interaction.rs means `interaction.rs` calls it through `pub(super)`.
