## game-input

**Scope:** `src/game/input/` — centralized input detection, mouse/keyboard message pipeline, and gamepad translation layer.

---

### Mental model

The input module is a clean dispatch hub: it reads raw Bevy `ButtonInput` state once per frame and broadcasts typed messages (`MouseLeftPressed`, `SpacebarPressed`, `ActionBarKeyPressed`, etc.) so every downstream spell or UI system stays source-agnostic. A gamepad sub-module mirrors the same message set via trigger → mouse-message translation and D-pad → archetype-specific messages. Frame-based boolean resources (`SpellInputBlockedThisFrame`, `MouseLeftHeldThisFrame`) are updated by a single sentinel system so run-conditions can be pure functions without consuming MessageReaders. The overall design is sound and well-decomposed; the findings below are focused spots of drift.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| I-01 | TypeContract | `src/game/input/messages.rs:13,21,33,41` | High | S | `cursor_position: Option<Vec2>` fields on `MouseLeftPressed`, `MouseLeftHeld`, `MouseRightPressed`, and `MouseRightHeld` are all marked `#[allow(dead_code)]` and are never read by any consumer — all 30+ spell systems read `CorrectedCursorPosition` directly. The fields add noise and mislead readers into thinking downstream code branches on cursor presence. | Remove the `cursor_position` field from all four message types and the corresponding struct-literal fill sites in `systems.rs` (lines 80, 84, 99, 103) and `gamepad/systems.rs` (lines 262–287). |
| I-02 | ArchitecturalDecay | `src/game/input/gamepad/systems.rs:1–393` | Medium | M | File is 393 lines mixing five unrelated concerns: device detection (`detect_active_input_device`, `toggle_cursor_visibility`), virtual cursor integration (`update_virtual_cursor`, `read_left_stick_shaped`, `apply_deadzone_and_curve`), trigger-to-mouse translation (`translate_triggers_to_mouse_messages`), radial slot mapping (`right_stick_to_slot`, `update_radial_hovered_slot`), and settings sync + UI confirm (`sync_gamepad_settings`, `emit_ui_confirm_back_messages`). Exceeds the 300-line project ceiling and is not a single-concern monolith. | Split into `device_detection.rs`, `virtual_cursor.rs`, `trigger_translation.rs`, `radial_slot.rs`. Keep `sync_gamepad_settings` and `emit_ui_confirm_back_messages` in a new `ui_systems.rs` or fold them into `virtual_cursor.rs` / `device_detection.rs` by affinity. |
| I-03 | ArchitecturalDecay | `src/game/input/gamepad/action_translation.rs:75–78` | Medium | S | Comment on lines 75–77 is factually wrong: it claims `detect_rune_input` "consumes `SpacebarPressed`", but `detect_rune_input` reads `ButtonInput<KeyCode>` directly and never reads the `SpacebarPressed` message. This misleads anyone tracing the gamepad rune-activation flow. | Correct the comment: `detect_rune_input` reads the physical keyboard binding and writes `ActivateRuneSequence` when the key is pressed. On gamepad the A-button sends `SpacebarPressed` (via `translate_activate_button`), but no system converts `SpacebarPressed` → `ActivateRuneSequence`, so the current design relies on the keyboard path running simultaneously. If gamepad-only rune activation is a future requirement, `translate_runes` would need to emit `ActivateRuneSequence` directly. |
| I-04 | ArchitecturalDecay | `src/game/input/systems.rs:169–173` | Low | S | `detect_rune_input` re-reads `bindings.universal.activate` and issues `ActivateRuneSequence` from `ButtonInput<KeyCode>`. This is a second keyboard-polling site for the same physical key that `detect_keyboard_input` already polls (lines 125–134) for `SpacebarPressed`. The two outputs differ (`SpacebarPressed` vs `ActivateRuneSequence`), so it is not a true duplication, but the pattern creates a hidden ordering dependency and makes the "activate key" semantics split across two functions. | Document clearly in both functions that `detect_keyboard_input` owns the universal `SpacebarPressed` family while `detect_rune_input` exclusively owns `ActivateRuneSequence`. Alternatively, have the rune system listen to `SpacebarPressed` and retire `ActivateRuneSequence` to unify the pathway (higher refactor impact, but cleaner). |
| I-05 | DocDrift | `src/game/input/gamepad/action_translation.rs:8` | Low | S | Module-level doc comment "Bindings are hardcoded here for now; rebinding UI lives in a later phase" is stale — controller rebinding was already delivered in Phase 1+2 (v0.6.286 per project memory). Gamepad action bindings here are still hardcoded (D-pad ↑ for multiple archetypes) but the framing implies this is a known placeholder that will be addressed, which could confuse future contributors. | Update the comment to note that these gamepad action bindings are intentionally hardcoded and do not yet participate in the config-based rebinding system, or remove the forward-looking reference if no such plan exists. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `src/game/input/gamepad/systems.rs` | 393 | false | Mixed concerns: device detection, virtual cursor, trigger translation, radial slot mapping, settings sync, UI confirm/back. Propose: `device_detection.rs`, `virtual_cursor.rs`, `trigger_translation.rs`, `radial_slot.rs` |

---

### Looks bad but is actually fine

- **PreUpdate systems without `run_if`** (`detect_active_input_device`, `toggle_cursor_visibility`, `update_virtual_cursor` in `gamepad/plugin.rs:42–52`): Device detection and cursor management intentionally run unconditionally in PreUpdate — they must be authoritative on every frame regardless of game state, including menus. The convention "every Update system must have run_if" applies to the `Update` schedule only; PreUpdate infrastructure systems are exempt.
- **`detect_active_input_device` calling `gamepad.get_pressed().next().is_some()` on every frame** (`systems.rs:54`): This iterates pressed buttons each frame for all gamepads, but the set is typically tiny and the loop exits at first match. Not a hot-path concern.
- **`sync_gamepad_settings` gated only on `resource_changed::<GameConfig>`** (`plugin.rs:67`): `GameConfig` is mutated by many unrelated systems (noted in project memory), but the function has an explicit early-return guard comparing all four fields before writing, so it produces no false `Changed<GamepadAimSettings>` triggers. Looks like it could cascade but does not.
- **`translate_triggers_to_mouse_messages` emitting both `MouseLeftPressed` and `MouseLeftHeld` on the same frame** (`systems.rs:262–263`): This mirrors the physical mouse path in `detect_mouse_input` which also writes both on `just_pressed` frames (lines 79–84). It is intentional — spells that check `Pressed` or `Held` both work on the first frame of a trigger press.
- **`#[allow(clippy::too_many_arguments)]` on several systems**: All are Bevy systems with injected parameters. This is idiomatic and covered by project convention.
- **`update_input_state_for_run_conditions` consuming both `MouseLeftHeld` and `BlockSpellInput` with `.next().is_some()`** (`systems.rs:241–242`): This drains the message queues as a side effect. This is the intended design — the doc comment explains the resource-based run-condition pattern explicitly.

---

### Open questions

1. **Gamepad-only RuneCaster rune activation**: `translate_activate_button` sends `SpacebarPressed` but no system converts that to `ActivateRuneSequence`. If a player is using a gamepad with the RuneCaster archetype, pressing South button fires `SpacebarPressed`, `detect_rune_input` then checks `ButtonInput<KeyCode>` (not pressed), and `ActivateRuneSequence` is never emitted. Is this intentional (rune activation requires a physical keyboard), or is it an untested gap?
2. **`cursor_position` field removal risk**: Before removing the dead field, confirm there are no reflection-based or serialized usages outside of what `grep` surfaces (the field is public so external plugins or test harnesses could theoretically read it).
3. **`is_warglock` / `is_gunslinger` naming**: The system is named `translate_warglock` and the run condition is `is_warglock`, but the backing domain type is `WizardType::Warglock`. The imported gunslinger messages (`gunslinger::messages::ReloadMessage`) suggest an older archetype name was "Gunslinger". Is there a pending rename, or is this the final stable naming?
