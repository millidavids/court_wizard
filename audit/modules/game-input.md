## game-input

**Scope:** `src/game/input/` — centralized input detection, mouse/keyboard message pipeline, and gamepad translation layer.

---

### Mental model

The input module is a clean dispatch hub: raw Bevy `ButtonInput` state is sampled once per frame and broadcast as typed messages (`MouseLeftPressed`, `SpacebarPressed`, `ActionBarKeyPressed`, etc.) so downstream spell and UI systems stay source-agnostic. A gamepad sub-module (`gamepad/`) mirrors the same message vocabulary via trigger → mouse-message and D-pad → archetype-specific message translation. Frame-scoped boolean resources (`SpellInputBlockedThisFrame`, `MouseLeftHeldThisFrame`, `MouseRightHeldThisFrame`) bridge the message bus to `run_if` conditions without consuming message readers inside conditions. The gamepad layer is feature-sliced into `connection.rs` (device detection/cursor toggle), `cursor.rs` (virtual cursor math), `navigation.rs` (trigger → mouse, radial slot), `action_translation.rs` (per-archetype D-pad bindings), `rumble.rs`, `resources.rs`, and `constants.rs`. The overall design is sound and well-decomposed; findings are Low–Medium severity.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| I-01 | TypeContract | `messages.rs:13,21,33,41` | Medium | S | `cursor_position: Option<Vec2>` is present on `MouseLeftPressed`, `MouseLeftHeld`, `MouseRightPressed`, and `MouseRightHeld`, suppressed with `#[allow(dead_code)]`. A codebase-wide search confirms the field is never read by any consumer — all downstream spell systems read `CorrectedCursorPosition` directly. The field is populated at write sites but silently discarded, creating a misleading contract. | Remove the `cursor_position` field from all four message structs and all write sites in `systems.rs` (lines 80, 84, 99, 103) and `gamepad/systems/navigation.rs` (lines 58, 66–67, 72, 87–88, 91). |
| I-02 | ArchitecturalDecay | `systems.rs:158–208` | Low | S | `detect_rune_input` uses fully qualified 5-segment paths (`crate::game::units::wizard::archetypes::runes::messages::RunePressed` repeated 3×, `crate::game::units::wizard::archetypes::runes::resources::Rune` 4×) while `detect_roulette_input` repeats `crate::game::units::wizard::archetypes::roulette::messages::RouletteSpinMessage` twice. The existing `use` imports at the top of the file are not used for these types, cluttering function bodies. | Add `use` imports for `RunePressed`, `ActivateRuneSequence`, `Rune`, and `RouletteSpinMessage` at the top of `systems.rs`, matching the clean import style already established in `gamepad/action_translation.rs`. |
| I-03 | DocDrift | `messages.rs:74–76` | Low | S | `ActionBarKeyPressed.slot` documentation says "0-9, where 0 represents the 10th slot" but the implementation in `detect_keyboard_input` only binds 5 slots (indices 0–4), and `RADIAL_SLOT_COUNT` is 5. The doc is aspirational/stale relative to the current code. | Update the field doc to "0–4, mapping to the five action bar slots" or, if 10-slot expansion is planned, add an explicit note marking it as a reserved range. |
| I-04 | DocDrift | `gamepad/action_translation.rs:75–78` | Low | S | The comment on `translate_runes` states that the activate action is "handled by the keyboard-side `detect_rune_input` which consumes `SpacebarPressed`." This is factually wrong: `detect_rune_input` reads `ButtonInput<KeyCode>` directly — it does not consume the `SpacebarPressed` message. The comment misleads anyone tracing the gamepad rune-activation code path. | Correct the comment: `detect_rune_input` polls the keyboard binding directly and emits `ActivateRuneSequence`; `translate_activate_button` independently emits `SpacebarPressed` from the South button. |
| I-05 | Performance | `gamepad/systems/connection.rs:40–46` | Low | S | `detect_active_input_device` computes `mag_sq.sqrt()` to compare against the `DEVICE_SWITCH_STICK_MAGNITUDE` constant (0.25). The sqrt is unnecessary — comparing `mag_sq > 0.25 * 0.25` is mathematically identical and avoids the float sqrt entirely. Runs every PreUpdate frame. | Replace `mag_sq.sqrt() > DEVICE_SWITCH_STICK_MAGNITUDE` with `mag_sq > DEVICE_SWITCH_STICK_MAGNITUDE * DEVICE_SWITCH_STICK_MAGNITUDE`. Optionally rename the constant `DEVICE_SWITCH_STICK_MAGNITUDE_SQ` if the squaring is applied at the constant definition site. |

---

### Oversized files

All files in scope are under 300 LOC. No oversized files to report.

---

### Looks bad but is actually fine

- **PreUpdate systems without `run_if`** (`detect_active_input_device`, `toggle_cursor_visibility`, `update_virtual_cursor` in `gamepad/plugin.rs:38–52`): Device detection and cursor management must run unconditionally every frame regardless of game state, including menus. The project convention "every Update system must have run_if" applies to the `Update` schedule only.
- **`update_virtual_cursor` returning early via `let Some(...) = active.gamepad_entity() else { return }` rather than a `run_if` guard**: For a PreUpdate system that must also park `corrected.0 = None` when no gamepad is active, an internal early-return is cleaner than an external run-condition (a run-condition can't write the None case when false).
- **`translate_triggers_to_mouse_messages` emitting both `MouseLeftPressed` and `MouseLeftHeld` on the same frame** (`navigation.rs:66–67`): Mirrors the physical mouse path in `detect_mouse_input` (lines 79–84). Intentional — spells that check either `Pressed` or `Held` both see the event on the first frame.
- **`sync_gamepad_settings` gated on `resource_changed::<GameConfig>`**: `GameConfig` is mutated by many unrelated systems, but the function has an explicit early-return guard comparing all four fields before writing, so no false `Changed<GamepadAimSettings>` triggers are produced.
- **`#[allow(clippy::too_many_arguments)]` on `detect_mouse_input`, `translate_triggers_to_mouse_messages`, and others**: All are Bevy systems with injected parameters — idiomatic per project convention.
- **`clear_mouse_input_state` guards re-centering on `virtual_cursor.screen_pos == Vec2::ZERO`** (`systems.rs:41`): `OnEnter(Running)` fires on pause→resume too; unconditional re-centering would yank the cursor mid-match. The ZERO-guard seeds on first gameplay entry only.
- **`unwrap_or(0.0)` on all `gamepad.get(axis)` calls**: The API returns `Option<f32>` for axes not present on all controller models; 0.0 (neutral) is the correct safe default.
- **`components.rs` is named `components.rs` rather than a feature-sliced name**: All four structs are genuinely cross-cutting input-state resources used by `run_conditions.rs`, `systems.rs`, and the gamepad sub-layer. The shared canonical name is appropriate here.
- **`detect_rune_input` polling the activate key a second time** (after `detect_keyboard_input` already does for `SpacebarPressed`): The two outputs differ — `SpacebarPressed` vs `ActivateRuneSequence` — so this is not logic duplication. Having the rune activate message originate from the keyboard polling path (not from a `SpacebarPressed` consumer) is a deliberate design that avoids an extra message-reader dependency in the rune module.

---

### Open questions

1. **Is `cursor_position` on mouse messages intentionally reserved for future use?** If so, document it as such rather than using `#[allow(dead_code)]`. If not, removal (I-01) simplifies the API surface and eliminates the dead-code lint suppression.
2. **Gamepad-only RuneCaster rune activation gap**: `translate_activate_button` emits `SpacebarPressed` on South button press. `detect_rune_input` listens to `ButtonInput<KeyCode>` directly — it does not read `SpacebarPressed`. If a player uses only a gamepad with the RuneCaster archetype, pressing South will fire `SpacebarPressed` but `ActivateRuneSequence` will never be emitted (the keyboard binding is not pressed). Is rune activation confirmed working on gamepad, or is this an untested gap?
3. **Is the 5-slot action bar limit final?** Resolves whether the `ActionBarKeyPressed.slot` doc (I-03) needs correcting or is genuinely aspirational.
