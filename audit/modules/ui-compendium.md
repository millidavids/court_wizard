## ui-compendium

**Scope:** `src/ui/compendium/` — 15 `.rs` files, 2422 LOC total (post Phase-16 split)

---

### Mental model

The Compendium is a full-screen encyclopedia rendered both in the main menu (`MenuState::Compendium`) and the in-game pause menu (`PauseMenuState::Compendium`). Two thin wrapper plugins (`MainMenuCompendiumPlugin`, `PauseMenuCompendiumPlugin`) share the same system functions, which are re-exported through a `systems.rs` shim. The UI is a two-column layout: a left detail panel and a right tabbed list area. Eight tabs (Spells, Ingredients, Units, Wizards, Achievements, Stats, Endless, Roguelite) each populate an `ItemsContainer` on tab switch, while the detail panel updates on every selection change.

All compendium state lives in `CompendiumState` (resource inserted on `OnEnter`, removed on `OnExit`). Tab switching and item selection are driven by `MouseClicked` messages. The central system `rebuild_on_state_change` (rebuild.rs) handles tab visual updates, save-data loading, item list rebuilding, and detail panel refresh. Item identity is encoded as `format!("{:?}", value)` debug strings stored in `CompendiumItemId` enum variants. Save data is served from an in-memory `SAVE_CACHE` Mutex, so `load_unified_save()` is not disk I/O on repeated calls.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | ConsistencyRot | `plugin.rs:57–91` | High | S | `PauseMenuCompendiumPlugin` does not register `systems::update_item_active_state` (only in main-menu plugin, line 46) or `systems::handle_copy_seed` (only in main-menu plugin, line 34). In the pause-menu compendium: selected item buttons are never highlighted, and the "Copy Seed" button on a roguelite run has no click handler and silently does nothing. | Add both systems to `PauseMenuCompendiumPlugin`'s `Update` blocks, mirroring the main-menu plugin registration exactly. |
| F02 | ArchitecturalDecay | `setup/rebuild.rs:17–270` | Medium | M | `rebuild_on_state_change` is 253 lines with four distinct responsibilities: (a) tab-button visual update (~35 lines), (b) save-data load + item-list rebuild (~40 lines), (c) `update_detail_panel` dispatch (~25 lines), (d) level-history container show/hide (~50 lines). The function also carries a Bevy 16-param tuple workaround comment at line 20. | Extract `update_tab_visuals` and `update_level_history_container` as free helper functions called from `rebuild_on_state_change`, reducing the system body to an orchestrator. |
| F03 | ArchitecturalDecay | `rows/detail_panel.rs:22–437` | Medium | M | `update_detail_panel` is 415 lines and accepts 14 parameters. Each of the seven `CompendiumItemId` match arms independently updates the same five query references (`title_q`, `category_q`, `desc_q`, `flavor_q`, `cat_color_q`) plus icon. | Extract each match arm to a `show_spell_detail`, `show_unit_detail`, etc. function; `update_detail_panel` becomes a thin dispatcher. |
| F04 | ConsistencyRot | `setup/item_spawners.rs:118` | Low | S | Team grouping uses `&["Defender", "Attacker", "Boss"]` raw string literals that must exactly match `UnitType::team_label()` return values. A rename of any label string silently breaks the grouping with no compile error. | Extract to a `const TEAM_LABELS: &[&str]` driven from an enum, or add a dedicated `UnitType::teams()` iterator that yields `(label, color)` pairs. |
| F05 | ConsistencyRot | `setup/roguelite.rs:95–96,144,162,234` | Low | S | Victory green `Color::srgb(0.3, 0.8, 0.3)`, defeat red `Color::srgb(0.8, 0.3, 0.3)`, and accent blue `Color::srgb(0.3, 0.6, 0.9)` appear as inline literals across at least four sites in `roguelite.rs`. | Add `VICTORY_COLOR`, `DEFEAT_COLOR`, `ACCENT_COLOR` to `constants.rs` (or re-use any existing global equivalents) and reference them. |
| F06 | ArchitecturalDecay | `systems.rs:1–11` | Low | S | `systems.rs` is a pure re-export shim that exists so `plugin.rs` can write `systems::handle_tab_click` rather than importing from `setup` directly. The "Phase 16" comment suggests this is a migration artifact. It adds an opaque indirection layer (three-hop: plugin → systems → setup/mod → handlers). | Either remove `systems.rs` and import directly from `super::setup` in `plugin.rs`, or document the shim's role clearly and drop the historical comment. |
| F07 | TypeContract | `setup/rebuild.rs:153` | Low | S | `items_container.single()` failure is silently skipped (`if tab_changed && let Ok(container) = ...`). If more than one `ItemsContainer` entity exists the item list is never rebuilt, with no log warning. | Add a `warn!("Expected exactly one ItemsContainer, got none/many")` in the failed branch; this is a programming invariant violation, not a normal runtime condition. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| `rows/detail_panel.rs` | 437 | No | `update_detail_panel` (415 lines) is a single large match on `CompendiumItemId`. It is close to a "single match" monolith but each arm is sizeable enough to extract. Propose: `detail_panel.rs` (dispatcher + shared helpers), `detail_spell.rs`, `detail_unit.rs`. |
| `setup/item_spawners.rs` | 364 | No | Five independent item-list spawners (`spawn_spell_items`, `spawn_ingredient_items`, `spawn_unit_items`, `spawn_wizard_items`, `spawn_achievement_items`) plus `spawn_stats_items`. Propose: `item_spawners/spells.rs`, `item_spawners/units.rs`, `item_spawners/stats.rs`. |
| `setup/roguelite.rs` | 323 | No | `spawn_roguelite_run_detail` is 205 lines. Propose: `roguelite_items.rs` (list, button, collect helper) and `roguelite_detail.rs` (run detail spawner). |
| `setup/rebuild.rs` | 270 | No | See F02. The file itself is fine post-split but the central function can be reduced. |
| `setup/layout.rs` | 268 | Yes | Two pure layout builder functions (`spawn_detail_panel`, `spawn_right_panel`) plus two thin entry points. All lines are cohesive UI construction with no mixed logic — exempt. |

---

### Looks bad but is actually fine

- **`load_unified_save()` on every `is_changed()` tick** (`rebuild.rs:137`): The call hits the in-memory `SAVE_CACHE` Mutex on a cache hit, which is O(clone) on a small struct — not disk I/O. Not a performance problem.
- **`rebuild_on_state_change` parameter tuple** (`rebuild.rs:20`): The `detail_panel_inputs` tuple is a documented workaround for Bevy's 16-element `SystemParam` tuple limit. Acceptable per project convention.
- **`arboard::Clipboard::new()` with `let Ok` guard** (`handlers.rs:83–86`): Clipboard creation can fail on Wayland/headless environments. The silent skip and `let _ = clipboard.set_text(...)` discard are correct defensive behavior.
- **`locked_silhouette_corona()` as a function in `constants.rs`** (`constants.rs:53`): Returns a `ShadowStyle` value type. It cannot be a `const` because `Val::Px` is not const-constructible in current Bevy. Function form is the only option; placement in `constants.rs` is reasonable since it is a visual tuning value.
- **`unwrap_or` throughout `spawn_stats_items`** (`item_spawners.rs:230–280`): All uses are `save.map(|s| ...).unwrap_or(0)` on `Option<&UnifiedSaveFile>` — safe defaults on missing save, not `.unwrap()` on `Result`.
- **Wildcard `pub(super) use detail_panel::*`** in `rows/mod.rs`: Visibility is `pub(super)` so nothing leaks outside the `compendium` module.
- **`update_item_active_state` early return** (`handlers.rs:47–49`): `if !state.is_changed() { return; }` is correct change-detection gating; runs every frame but exits immediately when nothing changed.

---

### Open questions

1. Is the absence of `update_item_active_state` and `handle_copy_seed` in `PauseMenuCompendiumPlugin` intentional (pause-menu compendium is read-only by design) or an oversight? This affects whether F01 is a bug or a feature decision.
2. `CompendiumTab`, `CompendiumItemId`, and `CompendiumState` live in `components.rs` alongside pure marker components. Should they move to a `state.rs` or `tabs.rs` file to better match the project's feature-sliced convention?
3. The `prev_tab: Option<CompendiumTab>` field on `CompendiumState` exists solely to suppress spurious item-list rebuilds. Could this be replaced by splitting the rebuild system into two — one for tab changes and one for selection changes — using Bevy's change-detection at the resource level?
