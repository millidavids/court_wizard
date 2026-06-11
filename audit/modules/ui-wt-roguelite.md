## ui-wt-roguelite

**Scope:** `src/ui/wizard_tower/roguelite_tab.rs` (1 682 lines, single flat file)

---

### Mental model

`roguelite_tab.rs` is the sole source of truth for the Roguelite tab of the Wizard Tower: it defines every component, every resource, every panel-builder function, every spawner helper, and every interactive system for that tab. On entering the tab with no active run, `init_roguelite_tab_resources` creates `SeedInputState`, `ExpandedToggles`, `PendingToggles`, and (conditionally) `RogueliteModifiers`, then `build_roguelite_no_run_*_panel` constructs modifier sliders, toggle rows, seed input, and action buttons. When a run is already in progress the active-run builders show level stats, Continue/End buttons, and skips resource creation entirely. Systems handle slider drag + step, seed keyboard/clipboard I/O, toggle expand/collapse, toggle unlock confirm popup, left-panel summary refresh, and the master action dispatcher (`handle_roguelite_action`) that kicks off `AppState::Loading`.

The file is correct in behavior but violates the project's granular file-structure mandate by collapsing seven distinct concerns into one 1 682-line monolith.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| R01 | ArchitecturalDecay | roguelite_tab.rs:1 | High | M | 1 682-line god file mixes constants, ~15 components, 4 resources, 4 panel-builder functions, 6 spawner helpers, and 10 systems. Project rule is >300 LOC must be split unless the file is a single match-on-enum or asset registry. This file is neither. | Split into at minimum: `constants.rs`, `components.rs`, `panel_builders.rs` (or two files for no-run/active-run panels), `spawners.rs`, `systems.rs` (or per-concern: `slider.rs`, `seed_input.rs`, `toggle.rs`). See proposed split below. |
| R02 | ArchitecturalDecay | roguelite_tab.rs:1395–1418, 1450–1459 | Medium | S | `toggle_row_action` and `handle_unlock_confirmation` each contain an identical loop that locates a `ToggleRowContainer` entity and writes `TOGGLE_ON_BG`/`TOGGLE_ON_BORDER` to `BackgroundColor`, `BorderColor`, and `ButtonColors`, then inserts `ButtonActive`. The only difference is that `toggle_row_action` can also set the OFF state. | Extract a private helper `fn set_toggle_row_visual(commands, containers, toggle, enabled)` and call it from both systems. |
| R03 | ArchitecturalDecay | roguelite_tab.rs:249 | Low | S | `SeedInputBox` is `pub(crate)` but is not referenced anywhere outside `roguelite_tab.rs`. The marker component only appears in spawner and system code within the same file. | Change to `pub(super)` to match the rest of the file's marker components. |
| R04 | ArchitecturalDecay | roguelite_tab.rs:32 | Low | S | `LABEL_COLOR` is defined as a `const` that simply re-aliases `TEXT_PRIMARY` (`const LABEL_COLOR: Color = TEXT_PRIMARY`). It adds an indirection with zero semantic distinction — `TEXT_PRIMARY` is already a named constant with clear intent. | Delete `LABEL_COLOR` and use `TEXT_PRIMARY` directly at each call site (9 references). |
| R05 | ConsistencyRot | roguelite_tab.rs:653, 661, 686, 687, 717, 811, 844, 886, 987 | Low | S | Nine inline `crate::ui::constants::…` and `crate::ui::focus::…` path references appear throughout the spawner functions despite the file already importing `crate::ui::constants::{…}` at the top. This is inconsistent with the rest of the file's import style and makes the spawner code harder to read. | Add `use crate::ui::focus::{Focusable, FocusableFlatBackground, ModalOverlay};` and `use crate::ui::constants::{SLIDER_GAP, SLIDER_LABEL_FONT_SIZE, INSIGHT_COLOR};` to the top-level use block, then replace the inline paths. |
| R06 | TypeContract | roguelite_tab.rs:1211–1269 | Medium | S | `slider_interaction` (and `update_sliders`, `update_slider_text`) take `ResMut<RogueliteModifiers>` (and `Res<RogueliteModifiers>`) as required params. In `plugin.rs` they are gated only on `resource_exists::<SeedInputState>`. The contract that "SeedInputState present ⟹ RogueliteModifiers present" is implicit — it lives in `init_roguelite_tab_resources` — and will panic at runtime if broken by a future refactor that inserts `SeedInputState` for any other reason. | Add `.run_if(resource_exists::<RogueliteModifiers>())` to the second `add_systems` block in `plugin.rs` that contains `slider_interaction`, or unify the two groups so both share the same `SeedInputState && RogueliteModifiers` guard. (Alternatively, use `Option<Res<RogueliteModifiers>>` inside the systems, but the explicit guard in the plugin is more discoverable.) |
| R07 | ConsistencyRot | roguelite_tab.rs:1603–1638 | Low | S | `seed_input_keyboard` contains two structurally-identical loops over digit keys — one for `KeyCode::Digit*` and one for `KeyCode::Numpad*`. Both loops have the same body and the same length-guard. | Merge the two arrays into one: `[(KeyCode::Digit0, '0'), …, (KeyCode::Numpad0, '0'), …]` and iterate a single loop, or chain the two arrays. Reduces 20 lines of boilerplate to ~12. |
| R08 | Performance | roguelite_tab.rs:1497–1509 | Low | S | `update_run_summary` despawns all children of `RunSummaryContent` and re-spawns them every time `RogueliteModifiers` or `PendingToggles` is marked changed. Bevy marks resources changed whenever they are accessed through `ResMut` even if no value changed. If any other system takes `ResMut<RogueliteModifiers>` unnecessarily, this will respawn text every frame. Currently `slider_interaction` and `slider_button_action` both write `ResMut<RogueliteModifiers>`. The `is_changed()` guard at line 1491 catches true no-ops, but the underlying "changed" bit is set by any `ResMut` borrow regardless. | This is low risk given the tab-active gate, but consider adding a hash or value-equality check before despawn, or using Bevy change detection more carefully (ensure only actual mutations touch `ResMut`). |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `roguelite_tab.rs` | 1 682 | No | Contains 7+ distinct concerns. Proposed split: `constants.rs` (lines 22–142, ~120 LOC), `components.rs` (lines 143–330, ~190 LOC), `panel_no_run.rs` (lines 337–460 + 642–946, panel-builder + spawner helpers, ~430 LOC), `panel_active_run.rs` (lines 462–601, ~140 LOC), `init.rs` (lines 603–640, ~40 LOC), `slider.rs` (~200 LOC: `slider_button_action`, `slider_interaction`, `update_sliders`, `update_slider_text`), `seed_input.rs` (~120 LOC), `toggle.rs` (~200 LOC: `toggle_expand_action`, `toggle_row_action`, `handle_unlock_confirmation`), `run_summary.rs` (~30 LOC). |

---

### Looks bad but is actually fine

- **`handle_roguelite_action` at line 1045 is 140 lines with 11 system params.** The `#[allow(clippy::too_many_arguments)]` is correct per project convention; the params map 1:1 to Bevy injected resources and this is idiomatic. The length is due to three distinct match arms with non-trivial logic; splitting is possible but not required.
- **`build_roguelite_no_run_right_panel` takes 6 params.** One is a comment-explained optional (`guest_pending`). The signature is intentional and documented.
- **`save_data::get_unlocked_toggles()` called at UI spawn time (line 418), and `save_data::get_insight()` called in `spawn_unlock_popup` (line 954).** These functions call `load_unified_save()` which reads from an in-memory `Mutex<Option<UnifiedSaveFile>>` cache, not from disk after first load. Disk I/O only occurs once per session. Not a hot-path concern.
- **Wildcard `use super::constants::*` at line 20.** The module's `constants.rs` is a sibling that is explicitly project-sanctioned for cross-cutting constants. Wildcard import here is acceptable rather than listing 20+ names.
- **`slider_interaction` reads both `Interaction::Pressed` and `Interaction::Hovered` from track query (line 1228).** This is intentional — clicking anywhere on the track should start dragging, not just the handle.
- **`pos.x + 0.5` in slider normalization (line 1252).** `RelativeCursorPosition.normalized` is centered at (0,0) per Bevy docs; adding 0.5 correctly converts to 0..1 range. Same pattern appears in `main_menu/settings/interaction.rs:287` and `study_tab/interaction.rs:906`, confirming this is the established project idiom.
- **`ToggleUnlockButton` labeled as "Insight cost text" (line 270–271 comment) but named `UnlockButton`.** The struct wraps the `ToggleModifier` for identification; the component is queried by `handle_unlock_confirmation` to despawn the cost label on unlock. The naming is slightly confusing but correct.
- **All systems are gated with `run_if(roguelite_tab_active)` at the plugin level** — no unconditional `Update` systems leak from this tab.

---

### Open questions

1. **Should `PendingToggles` and `roguelite_summary_lines` move to a shared module?** They are currently `pub(super)` / `pub(crate)` and referenced from `multiplayer_tab/sync.rs`. As the file splits, these will need a stable home (e.g., a new `roguelite_tab/shared.rs`) so `sync.rs` can still import them.
2. **Is the `SeedInputBox → pub(crate)` a dead remnant of an earlier design where focus.rs queried it directly?** No external consumer found today; safe to downgrade to `pub(super)`.
3. **Should the active-run left panel (level-by-level stats, aggregate totals) grow further?** Currently ~85 lines of inline text spawning. If more run stats are added, extracting a `stats_panel.rs` may be warranted before the active-run builder also exceeds 300 LOC.
