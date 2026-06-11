## ui-action_bar

**Scope:** `src/ui/action_bar/` — action bar UI: slot spawning, keyboard/mouse/gamepad input routing, linear↔radial layout morphing, and spell assignment.

---

### Mental model

The action bar is a single set of five `ActionBarSlot` button entities that physically reposition between a bottom-left horizontal row (keyboard/mouse) and a compact ring near the wizard (gamepad) via a smoothstep morph driven by `ActionBarLayoutProgress`. A single source of truth avoids spawning/despawning two separate layouts on device switch. Slot clicks and hotkey presses both resolve to `PrimeSpellMessage` (or `SelectGunMessage` for the Warglock). Radial-specific features (hover highlight, commit flash) live in `radial.rs`. The module is clean in intent, but carries dead code from a removed "spell name on button" feature, a plugin.rs purity violation, and several cross-file duplications of the 3D button press/rest child-walk pattern.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| AB-01 | ArchitecturalDecay | systems.rs:117,224; components.rs:19 | High | S | `ActionBarSlotText` component is defined and queried in `update_action_bar_slots` (lines 348–440) but is **never spawned anywhere**. The `slot_name` variable is computed at spawn time (line 117) and immediately discarded via `let _ = slot_name` (line 224). The comment at line 219 confirms spell names were removed, but the dead component, dead query, and dead `slot_name` binding were not cleaned up. Every iteration of `slot_text_query` in `update_action_bar_slots` silently iterates zero entities. | Delete `ActionBarSlotText` component, remove `slot_text_query` from `update_action_bar_slots`, remove the `(slot_name, icon_handle)` tuple and the `let _ = slot_name` discard in `spawn_action_bar`. |
| AB-02 | ArchitecturalDecay | plugin.rs:140,149 | Medium | S | `plugin.rs` defines two system bodies — `action_bar_enabled` (line 140) and `reset_layout_progress` (line 149) — directly in the plugin file. Project convention is strict: `plugin.rs` is for Bevy registration only; system bodies and helpers live in sibling files. | Move both functions to `systems.rs` (or a new `layout.rs`). `pub(super)` them so they can be referenced from `plugin.rs`. |
| AB-03 | ConsistencyRot | systems.rs:292; systems.rs:322–325 | Medium | S | `handle_slot_click` checks exclusive casting via `config.wizard_type.uses_exclusive_casting()`, while `handle_keyboard_input` re-implements the same check inline as `matches!(config.wizard_type, WizardType::RuneCaster \| WizardType::Randomancer)`. Both mean the same thing but diverge: if a new exclusive-casting archetype is added, `uses_exclusive_casting()` is the single source of truth and the inline `matches!` would silently miss it. | Replace the inline `matches!` in `handle_keyboard_input` with `config.wizard_type.uses_exclusive_casting()`. |
| AB-04 | ConsistencyRot | systems.rs:540,580 | Low | S | `BUTTON_REST_OUTLINE` is used fully-qualified (`crate::ui::constants::BUTTON_REST_OUTLINE`) at lines 540 and 580, but `BUTTON_3D_OFFSET_PRESSED`, `BUTTON_3D_OFFSET_REST`, and `BUTTON_PRESSED_OUTLINE` are imported at lines 19–21. The import is incomplete and the inconsistency is likely an oversight from when the import was last edited. | Add `BUTTON_REST_OUTLINE` to the `use crate::ui::constants::{...}` import block and remove the full-path references. |
| AB-05 | ArchitecturalDecay | systems.rs:515–544; systems.rs:570–584; radial.rs:304–334 | Medium | M | The "walk slot children, apply 3D pressed/rest border + outline + anim target" pattern is duplicated across three sites: `highlight_keyboard_pressed_slots`, `reset_action_bar_on_device_change`, and `tick_commit_flash`. The same child-walk pattern also appears in `ui/rune_display/systems.rs` (out of scope but confirms it should be a shared helper). Each site fetches `front_query`+`edge_query` independently and performs the same `if let Ok(mut bc) = front_query.get_mut(child)` + `if let Ok(mut outline) = edge_query.get_mut(child)` loop. | Extract a helper function (e.g., `apply_slot_button_state(pressed: bool, ...)`) into `src/ui/button_systems.rs` or a new `src/ui/action_bar/button_visuals.rs` that accepts the child slice plus mutable `front_query`/`edge_query` refs. All three call sites reduce to one line each. |
| AB-06 | Performance | systems.rs:338–458 | Low | S | `update_action_bar_slots` runs every frame during `InGameState::SpellBook` and `MultiplayerGameState::SpellBook` (plugin.rs line 110–114) and is not gated by `resource_changed::<GameConfig>`. Inside, it guards the heavy icon/text update behind `if config.is_changed()`, so no writes occur — but the system still evaluates its full parameter list and the `gun_state.is_changed()` branch every frame while the spellbook is open. Adding `resource_changed::<GameConfig>.or(resource_changed::<GunState>)` to the `run_if` would eliminate even the O(1) overhead. | Add `.run_if(resource_changed::<GameConfig>.or(resource_changed::<GunState>).or(is_local_wizard_active))` — or split the gunslinger highlight into a separate system gated on `resource_changed::<GunState>`. |
| AB-07 | ArchitecturalDecay | systems.rs:77–269 | Medium | M | `spawn_action_bar` is 193 lines — a single function that is almost half of `systems.rs`. It handles both the main slot spawning loop (lines 100–226) and the `#[cfg(debug_assertions)]` debug button (lines 230–266). While not exceeding any single exemption, it is the reason `systems.rs` hits 603 lines total. Extracting the debug button spawn to a separate `fn spawn_debug_mana_button` (also `#[cfg(debug_assertions)]`) and the per-slot child spawning to a `fn spawn_slot_children` helper would reduce cognitive load and bring `systems.rs` under the 300-line target. | Split into `spawn_debug_mana_button` and a per-slot helper. `spawn_action_bar` becomes a thin orchestrator. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `src/ui/action_bar/systems.rs` | 603 | No | Split into: `spawn.rs` (spawn_action_bar + helpers), `input.rs` (handle_slot_click, handle_keyboard_input, handle_spell_assignment), `keyboard_highlight.rs` (highlight_keyboard_pressed_slots, reset_action_bar_on_device_change) |
| `src/ui/action_bar/radial.rs` | 336 | Yes | Marginally over the 300-line limit; all content is cohesive (one layout concern + two flash/hover sub-systems). The exemption for "genuinely cohesive" content applies. |

---

### Looks bad but is actually fine

- **`animate_action_bar_layout` parameter count (9 params):** The `#[allow(clippy::too_many_arguments)]` is already present (radial.rs:55) and each param is a distinct Bevy query needed for the single-pass morph. This is idiomatic Bevy.
- **`last_applied: Local<Option<f32>>` early-out:** The `last_applied` guard (radial.rs:109–114) looks fragile with `f32::EPSILON`, but is correct here: the only values that would trigger a false-negative are two consecutive ticks with identical floating-point results, which would mean the animation hasn't moved and applying the layout again would be a no-op anyway.
- **`reset_layout_progress` runs on both `OnEnter(InGameState::Running)` and `OnEnter(MultiplayerGameState::Running)` (plugin.rs:29–36):** Registering the same system twice under different states is the correct Bevy idiom — the two states are mutually exclusive and both need initialization.
- **`update_action_bar_slots` running during `SpellBook` state:** Intentional — the spellbook assigns spells into slots, so the bar must update when the player drags a spell there even though the wizard is not "active."
- **Inline `WizardType::Warglock` check vs a named method like `uses_gunslinger_casting()`:** The `is_gunslinger` local is repeated across several functions, but the duplication is at the "read a resource field" level, not at behavioral logic. A dedicated method would be marginally cleaner but is not materially harmful.
- **`Color::srgb(1.0, 0.8, 0.2)` for the selected-gun slot border (systems.rs:379):** A magic color literal rather than a named constant. It is used only in one place and is a pure cosmetic tuning value — not worth a constant until it needs to be matched elsewhere.

---

### Open questions

1. Are there plans to restore spell names on buttons, or should `ActionBarSlotText`, `slot_name`, and the dead `slot_text_query` be pruned in the next cleanup pass?
2. Should the child-walk "apply button state" helper live in `ui/button_systems.rs` (cross-cutting) or in `ui/action_bar/button_visuals.rs` (action-bar-local)? The rune_display duplication suggests `ui/button_systems.rs` is the right home, but that file is owned by `ui-root`.
3. `update_action_bar_slots` does not respond to `layout_progress` changing (e.g. switching input device mid-SpellBook). Icon sizes set via `config.is_changed()` would stay at the wrong scale until the next config change. Is this intentional or a latent visual glitch?
