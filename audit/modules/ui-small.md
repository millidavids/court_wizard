# ui-small

**Scope:** `src/ui/manual/`, `src/ui/rune_display/`, `src/ui/notification/`, `src/ui/weather_bar/`, `src/ui/roulette_display/`, `src/ui/splash_screen/`, `src/ui/arcanorouter_display/`, `src/ui/gamepad_glyphs/`, `src/ui/concentration/`, `src/ui/version/`, `src/ui/loading/`

## Mental model

These 11 modules cover small, self-contained HUD elements and screens: a multi-tab markdown manual, three archetype-specific gameplay overlays (RuneCaster rune buttons, Randomancer roulette wheel, Arcanorouter sliders), the Meteorologist weather bar, a notification toast queue, the gamepad-glyph font infrastructure, a concentration-spell cancel UI, a splash screen sequence, a version link, and a dormant loading-screen skeleton. Most are feature-sliced cleanly with `plugin.rs`, `systems.rs`, `components.rs`, and `constants.rs`. The dominant tech-debt theme is an asymmetric cleanup pattern: roulette_display has explicit `OnExit` cleanup for SP but skips the equivalent for MP, and rune_display/arcanorouter_display have no `OnExit` at all, relying on global `OnGameplayScreen` cleanup that fires at game exit but not at pause/unpause boundaries. This means pause/resume cycles in SP, and MP pause/resume for Randomancer, silently accumulate duplicate UI roots.

---

## Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `src/ui/rune_display/plugin.rs:17-37` | High | S | `rune_display` has no `OnExit(InGameState::Running)` cleanup. When the player pauses (Running→Paused) and resumes (Paused→Running) in SP, `OnEnter(InGameState::Running)` fires again and spawns a second rune display on top of the first. Roulette_display explicitly guards against this at line 15 with a comment explaining the problem; rune_display does not. | Add `OnExit(InGameState::Running)` with `cleanup_screen::<RuneDisplayRoot>` and `OnExit(MultiplayerGameState::Running)` with the same, mirroring `roulette_display`'s SP pattern. |
| F2 | ArchitecturalDecay | `src/ui/arcanorouter_display/plugin.rs:14-29` | High | S | `arcanorouter_display` has no `OnExit` cleanup at all (neither SP nor MP). `ArcanoRouterDisplay` is tagged `OnGameplayScreen` which is only cleaned at `AppState::InGame` exit, not at pause boundaries. Same duplicate-spawn-on-pause bug as F1. | Add `OnExit(InGameState::Running)` and `OnExit(MultiplayerGameState::Running)` with `cleanup_screen::<ArcanoRouterDisplay>`. |
| F3 | ArchitecturalDecay | `src/ui/roulette_display/plugin.rs:25-28` | High | S | `roulette_display` spawns on `OnEnter(MultiplayerGameState::Running)` but has no `OnExit(MultiplayerGameState::Running)` cleanup. The existing SP comment at line 15 ("cleanup prevents duplicates on pause/unpause") was not extended to the MP spawn path. MP Randomancer pause→resume yields a stacked wheel. | Add `OnExit(MultiplayerGameState::Running)` with `cleanup_screen::<RouletteDisplayRoot>`, matching the SP pattern on line 21-22. |
| F4 | ArchitecturalDecay | `src/ui/manual/plugin.rs:77-102` | Medium | S | `plugin.rs` contains two non-trivial Bevy system bodies: `handle_main_menu_back_button` and `handle_pause_menu_back_button`. Project convention is plugin.rs for registration only; system bodies belong in `systems.rs`. | Move both handlers into `systems.rs` (they can take a generic `S: States` type parameter or be two `pub(super)` free functions), then reference them from `plugin.rs`. |
| F5 | Performance | `src/ui/notification/plugin.rs:15` | Medium | S | `queue_notifications` runs unconditionally in `Update` every frame across all app states — including menus and loading screens where the 5 message readers will always be empty. The two sibling systems have `run_if` guards; this one does not. | Wrap the whole `(queue_notifications, ...).chain()` set with a `run_if(is_gameplay_running)` or `run_if(is_local_wizard_active)` condition, consistent with the other guarded systems. |
| F6 | ConsistencyRot | `src/ui/weather_bar/systems.rs:244` | Medium | M | `update_weather_buttons` hard-codes the keyboard prompt "Clear Skies — Press Q/W/E" with no gamepad adaptation, while the two peer archetype overlays (`rune_display::adapt_rune_labels_to_input_device`, `roulette_display::adapt_prompt_to_input_device`) both swap text/font for the active controller style. A Meteorologist using a gamepad sees a keyboard prompt. | Add an `adapt_weather_prompt_to_input_device` system (modeled on `roulette_display`'s adapter) that replaces "Q/W/E" with the appropriate D-pad glyph text when a gamepad is active, and register it in `weather_bar/plugin.rs`. |
| F7 | ArchitecturalDecay | `src/ui/manual/systems.rs:27-48` | Low | S | `ParsedManualContent` is a `Resource` defined inside `systems.rs`. Project convention is that components and resources live in `components.rs` (or a dedicated file). Pollutes the systems file with a data type that is only incidentally spawned there. | Move `ParsedManualContent` to `components.rs` or a new `resources.rs` alongside the other manual types. |
| F8 | Performance | `src/ui/notification/components.rs:46` | Low | S | `NotificationQueue::pop` uses `Vec::remove(0)`, an O(n) front-removal that shifts all remaining entries. Notification bursts (unlocking a new wizard that triggers a chain of ingredient/spell toasts) make this worst-case linear. | Replace `Vec<NotificationEntry>` with `std::collections::VecDeque<NotificationEntry>` and change `remove(0)` to `pop_front()`. |
| F9 | ConsistencyRot | `src/ui/rune_display/systems.rs:203,328,437` | Low | S | Three sites use `format!("{}", x)` where `x` is a `char` or implements `Display`, when `.to_string()` is the idiomatic spelling. Minor style inconsistency throughout the file. | Replace `format!("{}", rune.as_char())` with `rune.as_char().to_string()` and `format!("{}", *sequence)` with `sequence.to_string()`. |
| F10 | ErrorObservability | `src/ui/concentration/systems.rs:49` | Low | S | `.expect("UI root exists")` is used to unwrap an iterator `.next()` inside a branch that is only reached when `has_ui == true`. While logically safe given the surrounding flow, it's a fragile invariant — a future refactor that changes the early-return structure would silently introduce a panic. | Replace with an explicit `if let Some(root) = ui_root_query.iter().next()` and an early `return` or `warn!` on `None`, making the invariant structural rather than runtime-asserted. |

---

## Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `src/ui/rune_display/systems.rs` | 447 | No | Multiple concerns: spawn, button press visuals, click handler, sequence display, spell-name fade, gamepad label adaptation. Split into `spawn.rs` (spawn_rune_display + spawn_rune_button), `visuals.rs` (update_rune_button_press_visuals), `input.rs` (handle_rune_button_click), `sequence.rs` (update_rune_display + show_spell_name_on_activation + update_spell_name_fade), `glyphs.rs` (adapt_rune_labels_to_input_device). |
| `src/ui/manual/systems.rs` | 398 | Yes | 97 of the 398 lines are inline `#[cfg(test)]` tests for `truncate_changelog_to_recent`. The remaining ~300 lines are cohesive manual-screen lifecycle code. Exempt under the ~300-line threshold when tests are excluded. |

---

## Looks bad but is actually fine

- **`src/ui/gamepad_glyphs/resources.rs:46-135`** — The `glyph_char` function is a 90-line match-on-style-then-match-on-button. It looks like a candidate for splitting but is a pure lookup table; the `#[allow(clippy::too_many_lines)]` annotation is correct and this is exempt as a match monolith.
- **`src/ui/rune_display/systems.rs:203`** — `Text::new(format!("{}", rune.as_char()))` looks like it could be `Text::new(rune.as_char())` but `Text::new` takes `Into<String>`, not `Into<char>`, so the explicit format is needed (unless using `.to_string()`).
- **`src/ui/loading/systems.rs:5`** — The entire file has `#![allow(dead_code)]` because the plugin body is intentionally empty (loading completes in imperceptible frames). The file comment explains this clearly and the code is kept intentionally for future use. This is not abandoned code.
- **`src/ui/notification/plugin.rs:16`** — `spawn_next_notification.run_if(|q: Res<NotificationQueue>| !q.is_empty())` uses a closure rather than a named run-condition. The inline closure is readable and this is a one-off condition; extracting it to a named function would add noise.
- **`src/ui/manual/plugin.rs:34-35`** — `rebuild_content_on_tab_change` and `update_tab_active_state` both run under `resource_exists::<ManualTab>.and(resource_changed::<ManualTab>)`. This duplicated condition looks verbose but is the correct pattern for two independent systems that both react to the same resource change — they cannot be merged because they query different entities.
- **`src/ui/rune_display/plugin.rs` and `src/ui/arcanorouter_display/plugin.rs` — `pub(crate) mod systems`** — exposing the `systems` module is intentional: `game_mode/systems.rs` calls `spawn_rune_display`, `spawn_roulette_display`, and `spawn_arcanorouter_display` directly when switching archetypes mid-game. The visibility leak is load-bearing.
- **`src/ui/concentration/systems.rs:55-65`** — Manual `Vec::contains` check to find existing spell entities is O(n²) in theory but `concentration_spells` is bounded to 1-3 entities in practice (lightning rod, magic missile, squall), so the cost is negligible.

---

## Open questions

1. **Pause/unpause duplication confirmed or mitigated elsewhere?** The rune_display and arcanorouter_display missing-OnExit analysis assumes `OnEnter(InGameState::Running)` fires on every Paused→Running transition. If Bevy's SubState implementation skips `OnEnter` when returning to a previously-active state (rather than re-entering from outside), F1 and F2 would be non-issues. Worth running a quick test: enter SP as RuneCaster, pause, unpause, confirm whether two rune displays appear.
2. **Does `is_randomancer` / `is_rune_caster` survive pause state transitions?** If these run-conditions return `false` during `Paused`, the Update system set would not fire during Paused, but the spawn still fires on re-entering `Running`. The duplication question is still open.
3. **Weather bar gamepad prompts (F6):** Is the "Clear Skies — Press Q/W/E" text intentionally keyboard-only because no gamepad binding exists for weather switching, or is weather switching supported on gamepad but just missing a prompt update?
