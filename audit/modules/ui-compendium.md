## ui-compendium

**Scope:** `src/ui/compendium/` — 7 files, ~2401 LOC total.

---

### Mental model

The Compendium is a full-screen encyclopedia rendered both in the main menu (`MenuState::Compendium`) and the in-game pause menu (`PauseMenuState::Compendium`). Two thin wrapper plugins share the same system functions. The UI is a two-column layout: a left detail panel and a right tabbed list area. Eight tabs (Spells, Ingredients, Units, Wizards, Achievements, Stats, Endless, Roguelite) each populate an `ItemsContainer` on tab switch, while the detail panel updates on every selection change.

All compendium state lives in `CompendiumState` (resource inserted/removed on enter/exit). Tab switching and item selection are driven by `MouseClicked` messages. A single god system `rebuild_on_state_change` (450+ lines from lines 377–623 in setup.rs) handles tab visual updates, save-data loading, item list rebuilding, and detail panel refresh. Item identity is encoded as `format!("{:?}", value)` debug strings stored in `CompendiumItemId` enum variants.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | ArchitecturalDecay | `setup.rs:1–1435` | High | L | `setup.rs` is 1435 lines holding: layout builders (`spawn_detail_panel`, `spawn_right_panel`), click handlers, item-list spawners for all 8 tabs, and the 450-line `rebuild_on_state_change` mega-system. Far exceeds the 300-line rule and mixes several distinct concerns. | Split into: `layout.rs` (panel/tab skeleton builders), `item_spawners.rs` (all `spawn_*_items` functions), `detail.rs` (moved from rows.rs), keeping `handlers.rs` for click handlers and `rebuild.rs` for the state-change system. |
| F02 | ArchitecturalDecay | `plugin.rs:98–126` | Medium | S | `plugin.rs` defines three system bodies: `handle_main_menu_back_button`, `handle_pause_menu_back_button`, `cleanup_compendium_state`. Convention requires plugin.rs to be registration-only. | Move these three functions to `handlers.rs` (or `setup.rs`) and import them. |
| F03 | ConsistencyRot | `plugin.rs:55–91` | High | S | `PauseMenuCompendiumPlugin` is missing `systems::update_item_active_state` and `systems::handle_copy_seed` compared to `MainMenuCompendiumPlugin`. The pause-menu compendium will not highlight the active item button on selection, and the seed copy button is silently non-functional. | Add `systems::update_item_active_state` and `systems::handle_copy_seed` to `PauseMenuCompendiumPlugin`'s Update sets, mirroring the main-menu plugin. |
| F04 | Performance | `setup.rs:490` | Medium | S | `load_unified_save()` (a full filesystem read + TOML parse) is called every time `CompendiumState::is_changed()` is true inside `rebuild_on_state_change`. This fires on every tab switch AND every item click — O(disk I/O) per user interaction. | Pass `save` as a `Res<>` — or at minimum cache the loaded data in `CompendiumState` when the resource is first inserted, refreshing only on actual mutation (toggle save/copy seed). |
| F05 | ConsistencyRot | `setup.rs:648,688,746,776,999,1013,1030,1057` / `rows.rs:322,403,435,521,584` | Medium | M | Item identity is encoded and decoded via `format!("{:?}", value)` at 13+ call sites across setup.rs and rows.rs. This ties display key identity to Rust's `Debug` impl formatting — a rename of any enum variant silently breaks save-file cross-reference without a compile error. | Replace with a stable `.canonical_name() -> &'static str` method on `Spell`, `Ingredient`, `UnitType`, `WizardType` that returns a fixed string (the existing debug name, but explicit). This guarantees contract and makes breakage visible. |
| F06 | ArchitecturalDecay | `setup.rs:377–623` | High | M | `rebuild_on_state_change` is a single 246-line system body that does: (a) tab button visual update, (b) save-data loading, (c) item list despawn+respawn, (d) detail panel refresh, (e) level-history show/hide. This is five distinct responsibilities in one function, inflating the already oversized setup.rs. | Extract into: `update_tab_visuals(...)`, `rebuild_items_list(...)`, `update_detail_and_history(...)`. Each can be chained with `.chain()` in the plugin registration so ordering is preserved. |
| F07 | ConsistencyRot | `setup.rs:1207,1209,1256,1258,1344,1347,1356` | Low | S | Inline `Color::srgb(0.3, 0.8, 0.3)` (victory green), `Color::srgb(0.8, 0.3, 0.3)` (defeat red), and `Color::srgb(0.3, 0.6, 0.9)` (accent blue) are repeated 7 times in `setup.rs` without being named constants. The same shades exist unnamed in multiple places. | Add `VICTORY_COLOR`, `DEFEAT_COLOR`, `ACCENT_COLOR` to `constants.rs`. |
| F08 | ArchitecturalDecay | `systems.rs:1–3` | Low | S | `systems.rs` is a pure re-export shim (`pub(super) use super::setup::*;`). The indirection adds a layer of opacity — callers in plugin.rs use `systems::rebuild_on_state_change` but the code lives in setup.rs. | If the separation is intentional (a migration artifact), document it with a comment; otherwise collapse `systems.rs` and import from `setup` directly in plugin.rs, or complete the split so systems.rs is a real module. |
| F09 | TypeContract | `setup.rs:506` | Low | S | `items_container.single()` is called with `let Ok(container) = ...` and the branch is silently skipped if it fails. If somehow two `ItemsContainer` entities exist (e.g. bug in despawn path) the rebuild is silently suppressed with no log. | Add a `warn!` inside the `else` path or use `.single().expect("ItemsContainer must exist")` since this is a programming invariant, not a runtime condition. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| `setup.rs` | 1435 | No | Multiple concerns: layout builders, click handlers, tab item spawners, rebuild system, roguelite/endless detail spawners. Propose splitting into: `layout.rs`, `handlers.rs`, `item_spawners.rs`, `rebuild.rs` |
| `rows.rs` | 625 | No | Two concerns: generic stat-row builders (lines 65–202) and `update_detail_panel` (lines 210–625, a 415-line match-on-enum). The large match is a single registry over `CompendiumItemId` variants — borderline exempt. Propose: `stat_rows.rs` + `detail_panel.rs`. |

---

### Looks bad but is actually fine

- **`unwrap_or` throughout `spawn_stats_items`** (setup.rs:843–892): All uses are `.map(|s| ...).unwrap_or(0)` — the `Option<&UnifiedSaveFile>` pattern is safe; these are not `.unwrap()` calls on `Result`.
- **`rebuild_on_state_change` system param bundling** (setup.rs:376–450): The comment explains this is deliberately bundled to dodge Bevy's 16-param tuple limit. `#[allow(clippy::too_many_arguments)]` is present; this is idiomatic Bevy workaround, not sloppiness.
- **`plugin.rs` size (126 lines)**: Although it contains system bodies (F02), the plugin registration itself is straightforward and readable.
- **`arboard::Clipboard::new()` with `let Ok(...)` guard** (setup.rs:365): Clipboard creation can fail on headless/Wayland environments; the silent skip is correct defensive behavior. The `let _ = clipboard.set_text(...)` discard is also correct (failure to copy to clipboard is not fatal).
- **`locked_silhouette_corona()` as a function in constants.rs** (constants.rs:53): Returns a `ShadowStyle` value type; it cannot be a const because `Val::Px` is not const-constructible in current Bevy. The function form is the only option.

---

### Open questions

1. Is the `systems.rs` re-export shim (`pub(super) use super::setup::*`) a migration artifact from a planned-but-incomplete split? If so, should setup.rs be fully dismantled into the originally-intended siblings?
2. The `PauseMenuCompendiumPlugin` omits `handle_copy_seed` and `update_item_active_state` (F03) — is this intentional (e.g. pause-menu compendium is read-only by design) or an oversight? Clarifying this affects whether F03 is a bug or a feature.
3. `load_unified_save()` is called on every item click (F04). Is there a performance budget concern on slower machines, or is the save file small enough that disk cache makes this acceptable?
