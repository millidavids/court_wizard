## ui-wt-roguelite

Scope: `src/ui/wizard_tower/roguelite_tab/` (11 files, 1786 LOC total)

---

### Mental model

`roguelite_tab/` is the "Run Setup & Status" panel of the Wizard Tower. It has two modes: **no active run** (shows modifier sliders, toggle modifiers, seed input, Start/Switch Wizard buttons) and **active run** (shows per-level stats and Continue/Abandon buttons). The module is cleanly feature-sliced: `init.rs` bootstraps resources, `panel_no_run.rs` / `panel_active_run.rs` do spawn-time layout, `slider.rs` / `seed_input.rs` / `toggle_spawn.rs` / `toggle_systems.rs` handle runtime interaction, `run_summary.rs` rebuilds the left-panel summary on change, and `actions.rs` handles top-level button actions (start/continue/end/switch wizard). All Update systems are properly gated via `in_state(MetaGameState::WizardTower)`, `roguelite_tab_active`, and `resource_exists::<SeedInputState>` in `plugin.rs`. No file exceeds 300 LOC. No `.unwrap()` calls. The module is in generally good shape — findings are low-to-medium severity.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| R01 | ArchitecturalDecay | `init.rs:9`, `seed_input.rs:11` | Medium | S | `random_seed()` is defined identically in both `init.rs` and `seed_input.rs`. Both call `rand::rng().random_range(0..constants::MAX_SEED)`. | Extract into a single `pub(super) fn random_seed() -> u64` in `constants.rs` and call it from both sites. |
| R02 | TypeContract | `components.rs:26,35,48,52,56,60` | Low | S | `ModifierSliderValue` methods (`get`, `set`, `min_value`, `max_value`, `step`, `label`) are declared `pub` instead of `pub(crate)`. The enum itself is `pub(crate)` so the extra visibility is noise. | Change all six method signatures to `pub(crate)`. |
| R03 | ArchitecturalDecay | `toggle_systems.rs:18,21` | Low | S | `toggle_expand_action` has two separate queries that both target `ToggleExpandButton`: `expand_buttons: Query<&ToggleExpandButton>` (used only for the early-return lookup) and `expand_btn_entities: Query<(Entity, &ToggleExpandButton)>` (used to mutate `ButtonActive`). They are redundant — one `Query<(Entity, &ToggleExpandButton)>` covers both use cases. | Merge into a single `Query<(Entity, &ToggleExpandButton)>` and obtain the component via `.get(event.button)` directly. |
| R04 | Performance | `seed_input.rs:138-145` | Low | S | The border-color update loop (`for mut border in &mut border_query`) is located **outside** the `for event in button_clicked.read()` loop, so it writes to `BorderColor` every frame regardless of whether any click occurred. Writes to `BorderColor` mark entities changed, causing unnecessary Bevy change-detection work downstream. | Gate the loop behind a `focus_changed` boolean set inside the event loop, or move the border update to only run when `seed_state.is_changed()`. |
| R05 | ConsistencyRot | `seed_input.rs:52,79,83,141,143,245` | Low | S | Seed input border/background colors are hardcoded as `Color::hsla(270.0, 0.35, 0.35, 1.0)` (unfocused, appears 5×) and `Color::hsla(270.0, 0.65, 0.55, 1.0)` (focused, appears 2×). These are absent from `constants.rs` and cannot be changed in one place. | Add `SEED_INPUT_BORDER_UNFOCUSED` and `SEED_INPUT_BORDER_FOCUSED` constants to `constants.rs` and replace all occurrences. |
| R06 | DocDrift | `seed_input.rs:233` | Low | S | Comment reads `// Enter/Escape to unfocus` but the condition on line 234 only checks `KeyCode::Enter` and `KeyCode::NumpadEnter`. `KeyCode::Escape` is absent — the feature was either never implemented or silently removed. | If Escape-to-unfocus is desired, add `|| keyboard.just_pressed(KeyCode::Escape)` to the condition; otherwise update the comment to say "Enter to unfocus". |
| R07 | ArchitecturalDecay | `seed_input.rs:118-134`, `237-242` | Low | S | The "apply a fresh random seed when text is empty" pattern is duplicated three times across `seed_input_click` (lines 118-122 and 128-133) and `seed_input_keyboard` (lines 237-242). Each copies seed to `seed_state.text`, sets `config.seed`, and updates all `SeedInputText` nodes. | Extract a `fn apply_random_seed(seed_state, config, text_query)` helper and call it from the three sites. |

---

### Oversized files

No files in scope exceed 300 LOC.

---

### Looks bad but is actually fine

- **`slider_interaction` requires `ResMut<RogueliteModifiers>` but is gated only by `resource_exists::<SeedInputState>`.** Looks like a potential panic, but `init_roguelite_tab_resources` inserts both resources in the same `Commands` batch. They are always co-present; no panic risk.
- **`played_coop: false` / `coop_peer_name: None` hardcoded in `actions.rs:132-133`.** Looks like a stale placeholder, but the accompanying comment (`// Roguelite co-op tagging lands with multi-level co-op continuation (WS6)`) confirms it is intentionally deferred. Not dead code.
- **`min_value()` / `max_value()` / `step()` return identical constants regardless of `ModifierSliderValue` variant.** Looks redundant, but all four slider variants intentionally share the same 0.2–3.0 / 0.1 range. The API is kept variant-dispatched for future per-variant divergence without breaking callers.
- **Two `arboard::Clipboard::new()` calls in `seed_input_keyboard`.** Both are behind independent `if ctrl && keyboard.just_pressed(...)` guards that cannot both fire in the same frame. No real cost.
- **`seed_input_click` toggles focus on repeated clicks (`!seed_state.focused`, line 110).** Deliberate: click once to enter editing mode, click again to commit and exit. Consistent with similar inputs elsewhere in the project.
- **`update_run_summary` uses `Option<Res<...>>` despite being gated by `resource_exists::<SeedInputState>`.** Defensive coding. `RogueliteModifiers` and `PendingToggles` are always inserted alongside `SeedInputState`, but the `Option` wrapper is a sensible safety valve against future resource-lifecycle changes.
- **`save_data::get_unlocked_toggles()` called at UI spawn time (`panel_no_run.rs:150`) and `save_data::get_insight()` in `spawn_unlock_popup` (`toggle_spawn.rs:153`).** These functions read from an in-memory `Mutex<Option<UnifiedSaveFile>>` cache, not from disk after first load. Not a hot-path concern.
- **All Update systems are gated with `run_if(roguelite_tab_active)` at the plugin level.** No unconditional Update systems leak from this tab.

---

### Open questions

1. Should `seed_input_keyboard` unfocus on `Escape`? The comment at line 233 implies it should. Intentional omission or oversight?
2. All four modifier sliders share the same 0.2–3.0 / 0.1 range. Is this a permanent design decision, or is per-slider range customization expected in a future milestone? If future divergence is planned, the `min_value`/`max_value`/`step` method bodies should gain `match self` arms now to signal intent.
3. `ModifierSliderValue` methods are `pub` — are they called from outside the crate? If not, `pub(crate)` is more appropriate and prevents accidental external stabilization.
