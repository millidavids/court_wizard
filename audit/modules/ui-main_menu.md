## ui-main_menu

**Scope:** `src/ui/main_menu/` — 31 `.rs` files, ~3 473 LOC.

---

### Mental Model

The main menu is divided into three sub-modules: `background` (parallax scrolling layers across all menu states), `landing` (the root button screen), and `settings` (a tabbed settings panel shared by main menu, pause menu, and multiplayer pause via re-exports).

`settings/` is the most complex area. Its components live in `components.rs` (a data-only file with all setting enums and marker types). The UI is split into `builders/` (spawn functions per tab, keybind overlay, confirmation popup) and `interaction/` (slider + toggle event handlers). `systems.rs` is a two-line re-export hub. `plugin.rs` registers everything under `MenuState::Settings` with proper `run_if` guards.

The pause menu and MP pause menu each have a parallel plugin (`pause_menu/settings/plugin.rs` and `pause_menu/mp_settings.rs`) that import and reuse every system from this module, differing only in their state type and the Back-button action.

Overall the module is well-structured and the feature-slicing is correct. The main issues are: (1) a stale copy-paste doc comment on the wrong function; (2) ~90 lines of duplicated inline button-spawning in the "action button" rows that bypass the existing `spawn_option_button` helper; (3) `controls_tab.rs` reads the save file from disk every time the Controls tab is rendered, which is a synchronous I/O call inside a Bevy spawn function invoked from an `Update` system; and (4) a stale development-phase comment in `systems.rs`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `settings/builders/controls_tab.rs:22-24` | High | M | `spawn_controls_tab` calls `crate::config::save_data::load_unified_save()` — a synchronous filesystem read — directly inside a builder function. This is called from `rebuild_settings_content`, an `Update` system triggered every time `SettingsTabState` changes. Any I/O latency or failure is silently swallowed via `.unwrap_or_default()`, and the Controls tab renders with all archetypes locked if the save is unreadable for transient reasons. The save file is already loaded and cached at startup; there is no reason to re-read it here. | Pass the unlocked wizard list as a parameter derived from an already-loaded Bevy resource (e.g., `Res<SaveData>` or a thin `Res<UnlockedWizards>` resource). Eliminate the direct disk read from the spawn path entirely. |
| F2 | ConsistencyRot | `settings/builders/game_tab.rs:84-131, 133-181` and `settings/builders/controls_tab.rs:208-255` | Medium | M | Three "action button" rows (Reset Tutorials, Clear Progress, Reset Controls) each inline ~47 lines of identical `Node` spawn boilerplate instead of using the existing `spawn_option_row` + `spawn_option_button` helpers from `setup.rs`. They differ only in label, background/border colors, and the action component. The `spawn_option_button` helper already accepts a generic `Bundle` action — action buttons are just option buttons without a selected state. | Add a `spawn_action_button(parent, label, action: impl Bundle, is_danger: bool)` helper in `setup.rs` and replace all three inline blocks. Removes ~90 lines of duplication. |
| F3 | DocDrift | `settings/interaction/toggle_input.rs:17-23` | Low | S | `handle_confirmation_popup` carries a copy-paste doc comment that says it "Sets up the settings menu UI with a tabbed interface" — copied verbatim from `builders/setup.rs`. The actual function handles confirmation popup clicks and Back-gamepad dismissal. | Replace with a correct doc comment: `/// Handles confirmation popup button clicks and Back-gamepad dismissal.` |
| F4 | DocDrift | `settings/builders/keybind_systems.rs:20-21` | Low | S | The doc comment `/// Opens key capture overlay when a binding button is clicked.` appears on two consecutive lines — a paste artifact. | Delete the duplicate line (line 21). |
| F5 | ArchitecturalDecay | `settings/builders/controls_tab.rs:258-259` | Low | S | Stale comment: `// NOTE: The rest of this function was the old placeholder. The spawn_controls_subsection and spawn_key_binding_row helpers are defined below.` — Refers to a removed previous implementation and adds no information. | Remove the comment; the helper names are self-documenting. |
| F6 | DocDrift | `settings/systems.rs:1` | Low | S | The module doc `//! Re-export hub for settings systems split (Phase 16).` references an internal development phase that is meaningless to future readers. | Drop the phase reference: `//! Re-exports all settings systems for external use.` |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|--------------------------|
| `settings/components.rs` | 365 | true | Pure data: component types, enums with `get`/`set`/`min`/`max`/`step` methods, and marker structs. The match-arm tables for `SliderValue` and `OptionButtonValue` constitute a single cohesive asset registry. Exempt per convention. |
| `settings/builders/controls_tab.rs` | 363 | false | After F1+F2 fixes this drops to ~295 LOC. If it remains over 300, split `spawn_controls_subsection`, `spawn_locked_subsection`, and `spawn_key_binding_row` into a `binding_row.rs` sibling and keep only `spawn_controls_tab` in `controls_tab.rs`. |
| `settings/builders/keybind_systems.rs` | 324 | true | Exactly one concern: key-capture input handling, overlay spawn, and binding update. All 324 lines are tightly cohesive. Exempt. |
| `settings/builders/tab_panels.rs` | 319 | true | Each function is a separate tab's spawn logic; the file is a cohesive registry of tab panel builders. Exempt. |
| `settings/builders/setup.rs` | 309 | true | All 309 lines are shared widget helpers consumed by sibling builder files. Canonical shared-helpers file, not a god file. Exempt. |

---

### Looks Bad But Is Actually Fine

- **`toggle_input.rs` defines both `settings_button_action` and `pause_settings_button_action`** — they differ only in the `Back` arm (transition to `MenuState::Landing` vs `PauseMenuState::Main`). The state types differ so the functions cannot be merged without a trait/enum dispatch that would obscure the difference. Intentional.
- **Three parallel settings plugins** (main menu, pause, MP pause) register the same ~12 systems under different states — looks like copy-paste rot but is intentional per the project MEMORY note ("Settings UI spans 3 plugins"). The duplication is in plugin registration boilerplate, not system bodies.
- **`rebuild_settings_content` has 11 system parameters** — carries `#[allow(clippy::too_many_arguments)]` and the params are genuine Bevy injected dependencies. Not a violation.
- **`systems.rs` uses wildcard re-exports** (`pub use builders::*; pub use interaction::*;`) — wildcard is normally discouraged but this is an explicit project re-export hub pattern, consumed by three external plugins. Fine as-is.
- **`update_studio_link_visuals` lacks an explicit `run_if` guard** — the system is gated at the plugin level by `.run_if(in_state(MenuState::Landing))`, and its inner query uses `Changed<Interaction>` so it is a no-op when nothing changes. Not a problem.

---

### Open Questions

1. Is there a `SavedProgress` or `UnlockedWizards` Bevy resource already populated at startup from the save file? If so, F1 is a one-line fix (pass as `Res<>` to `spawn_controls_tab`). If not, should one be introduced as a thin wrapper resource, or should the controls tab receive a pre-fetched `Vec<String>` slice passed down from `rebuild_settings_content`?
2. The scope note calls out `settings/builders.rs 1650` — this appears to reference a previous monolithic file that was already split. No such file exists today; confirm this note is stale.
