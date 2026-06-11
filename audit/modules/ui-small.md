## ui-small

**Scope:** `src/ui/manual/`, `src/ui/rune_display/`, `src/ui/notification/`, `src/ui/weather_bar/`, `src/ui/roulette_display/`, `src/ui/splash_screen/`, `src/ui/arcanorouter_display/`, `src/ui/gamepad_glyphs/`, `src/ui/concentration/`, `src/ui/version/`, `src/ui/loading/`

---

### Mental model

These 11 modules cover small, self-contained HUD elements and screens: a multi-tab markdown manual, three archetype-specific gameplay overlays (RuneCaster rune buttons, Randomancer roulette wheel, Arcanorouter sliders), the Meteorologist weather bar, a notification toast queue, the gamepad-glyph font infrastructure, a concentration-spell cancel UI, a splash screen sequence, a version link, and a dormant loading-screen skeleton. Most are feature-sliced cleanly with `plugin.rs`, `systems.rs`, `components.rs`, and `constants.rs`. The dominant tech-debt theme is an asymmetric cleanup pattern: `roulette_display` has explicit `OnExit` cleanup for SP but skips the equivalent for MP, and `rune_display`/`arcanorouter_display` have no `OnExit` at all, relying on global `OnGameplayScreen` cleanup that fires at game exit but not at pause/unpause boundaries. A second theme is mild consistency rot: three adapter systems share the "swap text/font for gamepad glyph" pattern without sharing any code, and the notification queue uses O(n) front-removal instead of a `VecDeque`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `src/ui/rune_display/plugin.rs:17` | High | S | `rune_display` has no `OnExit(InGameState::Running)` cleanup. When the player pauses (Running→Paused) and resumes (Paused→Running) in SP, `OnEnter(InGameState::Running)` fires again and spawns a second rune display on top of the first. `roulette_display` explicitly guards against this (line 21); `rune_display` does not. | Add `OnExit(InGameState::Running)` with `cleanup_screen::<RuneDisplayRoot>` and `OnExit(MultiplayerGameState::Running)` with the same, mirroring `roulette_display`. |
| F2 | ArchitecturalDecay | `src/ui/arcanorouter_display/plugin.rs:14` | High | S | `arcanorouter_display` has no `OnExit` cleanup at all (neither SP nor MP). `ArcanoRouterDisplay` is tagged `OnGameplayScreen`, which is only cleaned at `AppState::InGame` exit — not at pause/unpause boundaries. Same duplicate-spawn-on-pause class of bug as F1. | Add `OnExit(InGameState::Running)` and `OnExit(MultiplayerGameState::Running)` with `cleanup_screen::<ArcanoRouterDisplay>`. |
| F3 | ArchitecturalDecay | `src/ui/roulette_display/plugin.rs:25` | High | S | `roulette_display` spawns on `OnEnter(MultiplayerGameState::Running)` but has no matching `OnExit(MultiplayerGameState::Running)` cleanup. The SP comment at line 15 ("cleanup prevents duplicates on pause/unpause") was not extended to the MP spawn path. | Add `OnExit(MultiplayerGameState::Running)` with `cleanup_screen::<RouletteDisplayRoot>`, matching the SP pattern at line 21–22. |
| F4 | Performance | `src/ui/notification/plugin.rs:15` | Medium | S | `queue_notifications` runs unconditionally in `Update` every frame across all app states — splash, menus, loading — where all five message channels will always be empty. The two sibling systems in the same block both have `run_if` guards; this one does not. | Wrap the entire `(queue_notifications, ...).chain()` set with a `run_if(is_gameplay_running.or(in_state(AppState::MainMenu)))` condition, covering the states where toasts can actually appear. |
| F5 | ConsistencyRot | `src/ui/weather_bar/systems.rs:244` | Medium | M | `update_weather_buttons` hard-codes the keyboard hint "Clear Skies — Press Q/W/E" with no gamepad adaptation, while the two peer archetype overlays (`adapt_rune_labels_to_input_device`, `adapt_prompt_to_input_device`) both swap text/font for the active controller style. A Meteorologist using a gamepad sees a keyboard prompt. | Add an `adapt_weather_prompt_to_input_device` system (modeled on `roulette_display`'s adapter) that replaces "Q/W/E" with D-pad glyphs when a gamepad is active, and register it in `weather_bar/plugin.rs`. |
| F6 | ArchitecturalDecay | `src/ui/manual/systems.rs:27` | Low | S | `ParsedManualContent` is a `Resource` defined inside `systems.rs`. Project convention places components and resources in `components.rs` (or a dedicated file), keeping `systems.rs` free of type definitions. | Move `ParsedManualContent` to `manual/components.rs` alongside `ManualTab` and the other manual-screen types. |
| F7 | Performance | `src/ui/notification/components.rs:46` | Low | S | `NotificationQueue::pop` calls `Vec::remove(0)`, an O(n) front-removal that shifts all remaining entries. Notification bursts (new wizard + chain of ingredient/spell toasts) make this worst-case linear. | Replace `Vec<NotificationEntry>` with `std::collections::VecDeque<NotificationEntry>` and change `remove(0)` to `pop_front()`. `push` stays as-is (`push_back`). |
| F8 | Performance | `src/ui/rune_display/systems/animation.rs:49` | Low | S | `adapt_rune_labels_to_input_device` runs every gameplay frame with no change guard, iterating all four rune-button entities each tick. The body's equality checks prevent actual writes but the iteration still runs. | Add `.run_if(resource_changed::<ActiveInputDevice>.or(resource_changed::<CurrentControllerGlyphStyle>))` in the plugin; keep the body's equality checks as safety nets. |
| F9 | ArchitecturalDecay | `src/ui/roulette_display/systems.rs:176` | Low | S | `animate_wheel_spin` accesses `SPIN_DURATION` via a 7-segment absolute crate path (`crate::game::units::wizard::archetypes::roulette::constants::SPIN_DURATION`) instead of a `use` import at the top of the file. | Add a `use crate::game::units::wizard::archetypes::roulette::constants::SPIN_DURATION;` import at the top of `roulette_display/systems.rs`. |
| F10 | ErrorObservability | `src/ui/concentration/systems.rs:49` | Low | S | `.expect("UI root exists")` is used inside a branch entered only when `has_ui == true`, making the panic logically unreachable. A future refactor changing the early-return structure could silently introduce a real panic. | Replace with `if let Some(root) = ui_root_query.iter().next() { root } else { return; }` to make the invariant structural rather than asserted at runtime. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `src/ui/manual/systems.rs` | 398 | Yes | 97 of the 398 lines are inline `#[cfg(test)]` unit tests for `truncate_changelog_to_recent`. The remaining ~300 lines are cohesive manual-screen lifecycle code. Exempt under the ~300-line threshold when the tests block is excluded. |

---

### Looks bad but is actually fine

- **`src/ui/gamepad_glyphs/resources.rs:46-135` — `glyph_char` is 90 lines of nested `match`:** This is a pure lookup table over 4 controller styles × 16 buttons. The project convention explicitly exempts "a single large match-on-enum"; `#[allow(clippy::too_many_lines)]` is correct.
- **`src/ui/loading/systems.rs:1` — `#![allow(dead_code)]`:** The entire file is suppressed because the plugin body is intentionally empty (loading completes in imperceptible frames). The module-level comment explains this; it is kept intentionally for future use, not accidentally abandoned.
- **`src/ui/notification/plugin.rs:16` — inline closure `run_if`:** `spawn_next_notification.run_if(|q: Res<NotificationQueue>| !q.is_empty())` uses a closure rather than a named run-condition. The inline closure is readable and idiomatic for a one-off condition.
- **`src/ui/manual/plugin.rs:34-35` — duplicated `run_if` condition:** `rebuild_content_on_tab_change` and `update_tab_active_state` both use `resource_exists::<ManualTab>.and(resource_changed::<ManualTab>)`. This is correct: two independent systems both react to the same resource change and cannot be merged as they query different entities.
- **`src/ui/concentration/systems.rs:55-65` — O(n²) `Vec::contains`:** The linear search over existing spell entities looks like a performance issue, but `concentration_spells` is bounded to 1–3 active entities in practice (lightning rod, magic missile, squall), making the cost negligible.
- **`src/ui/rune_display/systems/mod.rs` — `pub(crate) mod systems`:** Exposing the `systems` module is intentional; `game_mode` calls `spawn_rune_display` etc. directly when switching archetypes mid-game. The visibility leak is load-bearing.
- **`src/ui/manual/systems.rs:188-194` — `setup_main_menu` / `setup_pause_menu` are one-liner wrappers with a `bool` parameter:** The bool parameter shares spawn logic between two plugin entry points. Neither wrapper is itself a Bevy system; they are called by `OnEnter` hooks. Correct design.

---

### Open questions

1. **Pause/unpause duplication confirmed?** F1 and F2 assume `OnEnter(InGameState::Running)` fires on every Paused→Running re-entry. If Bevy's `SubState` skips `OnEnter` when returning to a previously-active sub-state, these findings would be non-issues. Worth a quick play-test: enter SP as RuneCaster, pause, unpause, verify whether two rune displays appear.
2. **Weather bar gamepad prompts (F5):** Is "Clear Skies — Press Q/W/E" intentionally keyboard-only because no gamepad binding exists for weather switching, or does weather switching work on gamepad and just lacks a prompt update?
3. **`NotificationEntry` lifetime:** `ComboDiscovered.name/.description` and `Toast.message` are `&'static str`. Is there a known upcoming case where dynamically composed strings (non-`'static`) need to appear in toasts? If so, migrating to `String` or `Cow<'static, str>` before adding that call site is cheaper than a post-hoc migration.
