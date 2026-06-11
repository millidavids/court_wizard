## ui-action_bar

**Scope:** `src/ui/action_bar/` — the bottom-left spell hotkey bar + its linear↔radial layout morphing system.

---

### Mental model

The action bar renders up to 5 spell/gun slots as absolute-positioned `Button` entities under a single `ActionBarRoot` node. A single set of slot entities persists across the linear (KB+M, bottom-left row) and radial (gamepad, ring near wizard) layouts; the `animate_action_bar_layout` system smoothly morphs positions, scale, and typography each frame via a normalised `ActionBarLayoutProgress` resource. Input dispatch is split by source (`handle_slot_click` for mouse, `handle_keyboard_input` for keyboard, radial commit for gamepad). The module is feature-sliced cleanly (constants, components, messages, run_conditions, radial morph, spawn, input, keyboard highlight), well-commented, and all Update systems carry `run_if` guards. The main debt items are: a dead `ActionBarSlotText` component and its associated query branches left behind when spell-name text was removed from slots; near-duplicate dispatch logic in the two input handlers; and a stale doc-comment on a constant.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| AB-01 | ArchitecturalDecay | `components.rs:19`, `systems/keyboard_highlight.rs:36,45`, `radial.rs:69,79,92` | Medium | S | `ActionBarSlotText` is defined, used as a query filter in three systems, and carries a `slot` field — but it is **never spawned**. Spell names were removed from action bar slots (spawn.rs:219-226 comment + `let _ = slot_name`). The dead component, the `slot_text_query` in `update_action_bar_slots`, and the `name_texts` query in `animate_action_bar_layout` are all unreachable code that mislead readers and add noise to Bevy's query archetypes. | Delete `ActionBarSlotText`, remove `slot_text_query` from `update_action_bar_slots`, remove the `name_texts` query and the text-display block from `animate_action_bar_layout`. Also remove the dead `slot_name` binding and suppression in `spawn.rs:119-226`. |
| AB-02 | ConsistencyRot | `systems/input.rs:22-74` | Medium | S | `handle_slot_click` and `handle_keyboard_input` contain near-identical dispatch logic (Warglock → select_gun; exclusive-casting → no-op; else → prime_spell), but check the exclusive-casting guard differently: `handle_slot_click` (line 34) calls `config.wizard_type.uses_exclusive_casting()` while `handle_keyboard_input` (line 65) does a manual `matches!(…, RuneCaster \| Randomancer)`. A new exclusive-casting archetype would update `uses_exclusive_casting()` but silently miss the manual match. | Extract a shared `dispatch_action_slot(slot_idx, &config, mp_session, &mut prime_spell, &mut select_gun)` helper called by both handlers. This eliminates the duplication and ensures consistent use of `uses_exclusive_casting()`. |
| AB-03 | ArchitecturalDecay | `radial.rs:192-198` | Low | S | `animate_action_bar_layout` iterates `slots.iter_mut()` a second time solely to write `justify_content`. The value (`text_hidden`) is computed before the first loop at line 124, so `justify_content` can be assigned inside that loop — the second pass exists only because the borrow was released to do the children walk in between. | Move the `justify_content` assignment into the existing first `for (entity, slot, mut node) in &mut slots` loop (line 124), eliminating the second mutable borrow. |
| AB-04 | DocDrift | `constants.rs:16` | Low | S | `SPELL_NAME_FONT_SIZE` carries the doc comment `/// Increased to 16.0 for better readability with multiline names.` but its current value is `7.0`. The "16.0" figure is stale and refers to a previous value. Spell names were then removed from slots entirely, making this comment doubly misleading. | Remove the stale "Increased to 16.0" sentence. Replace with a plain description of the constant's purpose. |
| AB-05 | ConsistencyRot | `systems/keyboard_highlight.rs:62` | Low | S | The gunslinger selected-gun slot highlight uses the inline magic color `Color::srgb(1.0, 0.8, 0.2)` rather than a named constant. `RADIAL_HOVER_COLOR` exists (`Color::srgba(1.0, 0.95, 0.4, 1.0)`) but the two shades are slightly different. The unnamed color makes the intent opaque. | Define a `GUN_SELECTED_SLOT_COLOR` constant in `constants.rs` (or reuse `RADIAL_HOVER_COLOR` if the shades should converge) and reference it from `keyboard_highlight.rs`. |
| AB-06 | ArchitecturalDecay | `systems/spawn.rs:119,226` | Low | S | `slot_name` is computed inside `spawn_action_bar` as the first element of a tuple destructure and immediately suppressed with `let _ = slot_name`. The comment explains spell names are no longer rendered, but the binding still forces a `spell.map(|s| s.name())` call on every slot per spawn. | Remove `slot_name` from the destructure and replace the tuple with `let icon_handle: Option<Handle<Image>> = …`. Remove `let _ = slot_name`. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| `radial.rs` | 336 | true | All four exported functions are cohesive layout-morph concerns for the same radial sub-feature; splitting would scatter a single tightly-coupled concern. |
| `systems/spawn.rs` | 271 | true | Under 300 LOC; single spawn function plus two small helpers. Not a candidate for splitting. |
| `systems/keyboard_highlight.rs` | 244 | true | Under 300 LOC; three closely related highlight/reset systems for the same feature. Fine as-is. |

---

### Looks bad but is actually fine

- **`animate_action_bar_layout` has 9 system parameters and `#[allow(clippy::too_many_arguments)]`** — idiomatic Bevy; each parameter is a distinct ECS query for a different layer of the slot hierarchy (slots, icons, name texts, hotkey texts, fronts, debug INF button). No `SystemParam` bundle needed.
- **`slot_entities: Vec<Entity>` allocation every frame in `animate_action_bar_layout`** — the `last_applied` early-out skips the allocation on nearly every frame; cost is non-zero only during the 350ms morph transition.
- **`OnEnter(InGameState::Running)` and `OnEnter(MultiplayerGameState::Running)` registered separately for the same systems in `plugin.rs`** — correct and intentional; there is no combined running state spanning both SP and MP. Duplication is forced by the state machine architecture.
- **`handle_debug_mana_click` uses `debug_button_query.get(event.button).is_ok()`** — intentional membership test; no unwrap risk.
- **`radial.rs` glob import `use super::components::*`** — acceptable within a tightly coupled sub-module where all symbols are in-module and the list is stable.
- **`update_action_bar_slots` does real work only on `config.is_changed()` / `gun_state.is_changed()`** — the broader `run_if` covers the state condition; the inner change-detection guards filter per-frame work. Belt-and-suspenders, not waste.

---

### Open questions

- With `ActionBarSlotText` never spawned, `calculate_action_bar_font_size` in `spawn.rs` is only called from `update_action_bar_slots` (which queries the dead component). Should `calculate_action_bar_font_size` also be removed, or is it expected to be revived if spell-name text returns?
- The gunslinger selected-gun highlight (`Color::srgb(1.0, 0.8, 0.2)`) writes to `BorderColor` on the slot root node, while `highlight_radial_hovered_slot` writes to the child `ButtonFront` `BorderColor`. Are these two different visual layers intentional, or should they both target the same layer for visual consistency?
- Should `clear_blocked_action_bar_spells` handle future archetype-specific restrictions generically (e.g. via a `WizardType::blocked_spells()` method), or is the current Shepherd-only hard-code sufficient?
