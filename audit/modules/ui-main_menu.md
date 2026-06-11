## ui-main_menu

**Scope:** `src/ui/main_menu/` — landing screen, parallax background, and settings UI (main-menu flavour).

---

### Mental model

The main menu is composed of three independently-plugged sub-modules:

- **background/** — a three-layer parallax scroll (skybox / mid / foreground) that lives for the full `AppState::MainMenu` lifetime and animates unconditionally (properly state-gated).
- **landing/** — the title screen with five nav buttons, a studio logo link, and an `OnLandingScreen` cleanup marker.
- **settings/** — a tabbed settings screen shared with the pause menu and MP pause menu. It is the heaviest sub-module (1650 LOC in `builders.rs` alone) and owns sliders, key-binding capture, confirmation popups, and per-tab content reconstruction.

The settings module is designed for **shared use across three contexts** (main menu → `MenuState::Settings`, pause menu → `PauseMenuState::Settings`, MP pause → `MultiplayerGameState::Settings`). Components, builders, and interaction logic live in `src/ui/main_menu/settings/`; each context registers its own thin plugin that wires the shared systems to the correct state.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `settings/builders.rs:1` | High | M | `builders.rs` (1650 LOC) mixes two distinct concerns: UI spawn helpers (`spawn_graphics_tab`, `spawn_controls_tab`, etc.) and fully-fledged Bevy systems (`key_binding_button_action`, `capture_key_input`, `rebuild_settings_content`, `resolution_button_action`, `update_key_binding_text`, `update_resolution_selection`, `update_resolution_visibility`, `handle_settings_tab_click`, `setup_main_menu`, `setup_pause_menu`). The convention states files >300 LOC must be split unless genuinely cohesive. These are two separate concerns. | Extract the Bevy system functions (lines ~1014–1401) into a dedicated `keybind_systems.rs` and `tab_systems.rs` (or `rebuild.rs`). Keep `builders.rs` for pure spawn helpers only. |
| F2 | ArchitecturalDecay | `settings/systems.rs:3-4` | Medium | S | `systems.rs` is documented as "Re-export hub for settings systems split (Phase 16)" and contains only `pub use super::builders::*; pub use super::interaction::*;`. This glob re-export flattens the two concern-files into one public surface, obscuring what comes from where. The phrase "Phase 16" is a stale migration note that no longer means anything to a reader. | Remove the intermediary re-export module and have `plugin.rs` import directly from `builders` and `interaction` (as the pause-menu settings plugin already does). Delete the "Phase 16" comment. |
| F3 | ConsistencyRot | `settings/interaction.rs:77-150` | Medium | M | Three near-identical `settings_button_action` functions exist across three files: `interaction.rs:77` (main menu), `interaction.rs:116` (pause menu), and `src/ui/pause_menu/mp_settings.rs:92` (MP pause). All three share the same match arms (`ResetTutorials` → popup, `ClearProgress` → popup, `ResetControls` → reset bindings) and differ only in the `Back` arm's state transition target. This is 3-site duplication of non-trivial logic. | Extract a `handle_shared_settings_action(action, commands, bindings)` helper that handles the three common arms and returns a `bool` indicating whether `Back` was pressed. Each context-specific function calls the helper then handles its own `Back` navigation. |
| F4 | Performance | `settings/builders.rs:669` | Medium | S | `spawn_controls_tab` calls `crate::config::save_data::load_unified_save()` (synchronous disk I/O) every time the Controls tab is rebuilt. Tab rebuilds happen on every tab switch, including switching *away from* Controls and back. This reads from disk on every Controls tab activation. | Cache the unlocked wizard types in a `Resource` (already loaded at startup) and pass it as a parameter instead of re-reading the save file on each tab rebuild. |
| F5 | DocDrift | `settings/interaction.rs:19-25` | Low | S | The doc comment on `handle_confirmation_popup` (lines 19–25) is a verbatim copy-paste of the `setup` function's doc comment from `builders.rs`: "Sets up the settings menu UI with a tabbed interface. Creates a settings screen with tabs for Graphics, Audio, Game, and Controls…". This is entirely wrong for a confirmation-popup handler. | Replace with a correct one-line doc: `/// Handles confirmation popup button clicks and gamepad Back presses.` |
| F6 | DocDrift | `settings/builders.rs:905-906` | Low | S | Stale development comment: `// NOTE: The rest of this function was the old placeholder. The spawn_controls_subsection and spawn_key_binding_row helpers are defined below.` This refers to a now-completed refactor and adds no value to a reader. | Remove the comment. |
| F7 | TypeContract | `settings/interaction.rs:203` | Low | S | `update_slider_text` formats all slider values as `"{}%", (value * 100.0) as u8`. For sliders with ranges outside [0,1] this produces misleading output: `GameSpeed` at its minimum (0.5) shows "50%", at maximum (2.0) shows "200%"; `UiBrightness` max shows "200%"; `GamepadResponseCurve` max shows "350%". The label "200%" for max brightness is confusing to players. | Add a `display_text(value: f32) -> String` method to `SliderValue` that returns the value in appropriate units (e.g. `"×2.0"` for GameSpeed, `"0.30"` for deadzone, `"×1.5"` for response curve). |
| F8 | ArchitecturalDecay | `settings/builders.rs:39` | Low | S | The private `setup` function takes a `pause_menu: bool` parameter and uses it only to pass to `spawn_page_container`. The two public wrappers `setup_main_menu` and `setup_pause_menu` differ by this single flag. This is fine, but the parameter name `pause_menu` is undocumented on the private function, and the logic of what changes (opaque vs transparent background / GlobalZIndex) is invisible at the call site. | Add a brief inline comment on `setup`'s `pause_menu` parameter explaining what it affects, so future maintainers can understand the distinction. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `settings/builders.rs` | 1650 | No | Mixed spawn helpers + Bevy system bodies. Propose split into: `builders.rs` (spawn helpers only, ~900 LOC), `keybind_systems.rs` (key capture + binding systems ~200 LOC), `tab_systems.rs` (tab switching + content rebuild + resolution systems ~350 LOC). |
| `settings/components.rs` | 365 | Yes | All lines are `#[derive(Component)]` / `#[derive(Resource)]` struct/enum definitions + `impl` blocks mapping to `GameConfig` fields. This is a single-concern registry; exemption applies. |
| `settings/interaction.rs` | 353 | No | Mixes option-button handling, slider handling, slider drag interaction, and selected-state sync. Propose split into: `option_interaction.rs` (option buttons + selected-state), `slider_interaction.rs` (slider drag + update systems). |

---

### Looks bad but is actually fine

- **`slider_interaction` checking `Interaction::Hovered` to start drag** — looks like a quirky way to start a drag (line 259), but it prevents missed clicks when the cursor lands slightly off the track handle; intentional UX decision.
- **`escape_to_landing` lacking its own `run_if(in_state(MenuState::Settings))`** — it's registered inside a tuple that has `.run_if(in_state(MenuState::Settings))` applied to the whole tuple (plugin.rs:56-72), so it is correctly gated.
- **`rebuild_settings_content` calling `despawn_related::<Children>` and re-spawning every tab switch** — looks expensive, but tabs are switched rarely and the content is shallow enough that this is fine and simpler than diffing.
- **`systems.rs` glob re-exporting everything from `builders` and `interaction`** — this is flagged as F2 for the stale comment, but the underlying glob-re-export pattern is a documented "Phase 16" migration convention; other auditors should not flag this a second time.
- **Hardcoded wizard debug names in `spawn_controls_tab`** (`"RuneCaster"`, `"Swordcerer"`, etc.) — strings match the enum variant names used in save data. They would ideally use typed constants, but this is an existing pattern across the codebase and the risk of drift is low.
- **`update_studio_link_visuals` in `landing/studio_link.rs` using `Changed<Interaction>`** instead of the project's custom `MouseClicked` message — correctly using Bevy's built-in interaction for a link button that does not participate in `ButtonActionSet`; not a violation.

---

### Open questions

1. **`save_data::load_unified_save()` in `spawn_controls_tab`** — is there a canonical `UnlockedContent` resource already loaded at startup that could be queried instead of hitting disk here?
2. **`settings/interaction.rs:pause_settings_button_action`** — this function lives in the main-menu settings module but is exported to and registered by the pause-menu settings plugin. Should it move to `src/ui/pause_menu/settings/` to clarify ownership, or is the current cross-plugin sharing intentional per the project's three-plugin note?
3. **`slider_text` showing `%` for non-percentage sliders** — is the "50%–200%" display for GameSpeed intentional (relative-to-default mental model), or should it show the raw multiplier value (e.g. "×0.5")?
