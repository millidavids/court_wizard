## ui-in_game

**Scope:** `src/ui/in_game/` — in-game HUD: mana/cast/king/boss bars, wave/buff/flash displays, keyboard and gamepad input.

---

### Mental model

The module is a well-decomposed feature-sliced HUD. `plugin.rs` is pure registration. Sub-modules group coherently: `bars/` holds all bar update logic split by concern (resource bars, boss-bar spawn, boss-bar update); `spawn/` holds HUD construction (SP, MP, input handlers); `displays.rs` covers flashes and the buff tracker. Components and constants live in dedicated files. The `systems.rs` shim is a 5-line re-export hub that keeps `plugin.rs` stable across future file splits.

The principal weaknesses are: (1) `hud_sp.rs` and `hud_mp.rs` share 80–90 lines of nearly-identical bottom-bar construction that was never extracted into a shared helper; (2) two doc-comment strings were copy-pasted to the wrong functions during the split; (3) `spawn_boss_health_bar` runs six queries every frame even after the bar exists; (4) `HagIdentity` lacks the `index()` method that `RayEyeType` has, forcing the same three-way `match` to appear three times; (5) `spawn_ray_eye_bar_section` is structurally identical to `spawn_hag_bar_section` and should be unified.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | DocDrift | `bars/resource_bars.rs:16` | Low | S | Doc comment on `update_mana_bar` reads "Blocks spell input when the mouse is interacting with a UI button…" — the doc for `block_spell_input_on_button_interaction`, copy-pasted during the Phase-16 file split. | Replace with a correct doc describing mana + reserved-mana bar width updates. |
| F02 | DocDrift | `displays.rs:17` | Low | S | Identical stale "Blocks spell input…" comment appears above `update_wave_display`. Same artifact. | Replace with a correct one-liner: "Updates the wave counter text when `WaveState` changes." |
| F03 | ArchitecturalDecay | `spawn/hud_sp.rs:208–349` + `spawn/hud_mp.rs:101–239` | Medium | M | The bottom-bars section (ammo-display branch, mana+reserved-fill branch, cast-bar+brewing-overlay block) is duplicated verbatim across both HUD builders (~85–90 lines each). `spawn_king_health_bar` was extracted for exactly this reason; the bottom bars were not. | Extract `fn spawn_bottom_bars(bars: &mut ChildSpawnerCommands, config: &GameConfig)` into a shared helper (either in `hud_sp.rs` or a new `spawn/hud_shared.rs`) and call it from both builders. |
| F04 | ArchitecturalDecay | `bars/boss_bar_spawn.rs:215–281` + `bars/boss_bar_spawn.rs:283–345` | Medium | M | `spawn_hag_bar_section` and `spawn_ray_eye_bar_section` are structurally identical (same child layout, background, border, text overlay). They differ only in the component marker types attached to fill/text nodes. | Extract `fn spawn_boss_bar_section<F: Component, T: Component>(parent, fill_marker: F, text_marker: T, name, color, ...)` and call it from both. |
| F05 | ConsistencyRot | `bars/boss_bar_update.rs:103,114,127` | Low | S | `HagIdentity` → `usize` index mapping (`Justina=0, Martina=1, Josephina=2`) is repeated three times in `update_boss_health_bar`. `RayEyeType` has an `index()` method for the same purpose; `HagIdentity` does not. | Add `pub const fn index(self) -> usize` to `HagIdentity` (in `game/units/boss/hags/components.rs`) and replace all three `match` blocks. |
| F06 | ConsistencyRot | `bars/boss_bar_spawn.rs:28–33` | Low | S | Boss-type detection uses `.iter().next().is_some()` six times. The idiomatic Bevy form is `!query.is_empty()`. | Replace all six occurrences with `!query.is_empty()`. |
| F07 | Performance | `bars/boss_bar_spawn.rs:13` (run_if context: `plugin.rs:132`) | Low | S | `spawn_boss_health_bar` runs every frame during gameplay, evaluating six queries and two boolean guards even after the bar has been spawned once. After the first spawn `boss_exists && !bar_exists` is always false, but the queries still execute at 60 fps for the entire boss fight. | Add `.run_if(not(any_with_component::<BossHealthBarRoot>))` alongside the existing `is_gameplay_running` gate in `plugin.rs`. |
| F08 | ArchitecturalDecay | `bars/boss_bar_spawn.rs:122–135` | Low | S | Ray eye bar colors (`Color::srgb(0.7, 0.7, 0.7)` etc.) are inline literals. The hag identity colors are in `constants.rs`; the Ray eye colors should be too for consistency. | Add `RAY_EYE_PETRIFICATION_COLOR`, `RAY_EYE_DISINTEGRATION_COLOR`, etc. to `constants.rs`. |
| F09 | DocDrift | `displays.rs:353` | Low | S | Tooltip top-offset is `BUFF_BOX_SIZE + BUFF_BOX_GAP + 20.0 + 10.0`; the two bare addends are unexplained magic numbers. | Replace with named constants (e.g., `BUFF_TRACKER_TOP_OFFSET: f32 = 20.0`) or add an inline comment explaining what each addend represents. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `displays.rs` | 385 | No | Mixed concerns: wave display, generic timed-flash infrastructure (`spawn_flash_banner_with_marker`, `update_timed_flash`), retreat/shield flash systems, and the full buff-tracker subsystem. Proposed split: `flash.rs` (flash helper + `update_timed_flash` + wave/retreat/shield flash systems, ~140 LOC) + `buff_tracker.rs` (`update_buff_tracker`, `update_buff_timers`, `show_buff_tooltip`, color/abbrev helpers, ~170 LOC) + `wave_display.rs` (`update_wave_display`, `update_wave_incoming_flash`, `banner_alpha`, ~100 LOC). |
| `spawn/hud_sp.rs` | 349 | No | After extracting `spawn_bottom_bars` (F03) the file drops to ~210 lines naturally. No further split needed. |
| `bars/boss_bar_spawn.rs` | 345 | No | After merging `spawn_hag_bar_section` / `spawn_ray_eye_bar_section` into a generic helper (F04), the file shrinks to ~250 lines. If a sixth boss type is added it will cross 300 again; at that point split by boss family (`boss_bar_hags.rs`, `boss_bar_simple.rs`, `boss_bar_ray.rs`). |

---

### Looks bad but is actually fine

- **`systems.rs` is a 5-line re-export hub.** Intentional Phase-16 indirection so `plugin.rs` keeps a stable `use super::systems::*` import regardless of future sub-splits. Not a violation.
- **`update_boss_health_bar` has 12 query parameters.** `#[allow(clippy::too_many_arguments)]` is present. The system genuinely needs separate, mutually-exclusive queries for each boss variant's fill/text markers. This is idiomatic Bevy.
- **`update_buff_timers` allocates `format!` strings every frame.** Active buff counts are small (0–8 practical max), allocations are minor, and the system already runs only during `InGameState::Running | MultiplayerGameState::Running`. Not worth a change-detection workaround at this scale.
- **`spawn_hud` reads `WaveState` at spawn time only to check `total_waves > 1`.** One-time read; no per-frame cost.
- **`hud_mp.rs` importing `spawn_king_health_bar` from `hud_sp.rs` (`pub(super)`).** The function is module-internal to `spawn/`, the import is correct, and a comment explains the sharing. Clean.
- **`GunType::MachineGun` hardcoded in both HUD builders** for the initial ammo piece count. `GunState::default()` also selects `MachineGun`; both sites are consistent with the initial resource state.
- **`boss_bar_spawn.rs` using `is_hags / is_lich / is_dark_mage / is_ray` booleans as an `if/else-if` chain.** A `match`-on-enum would be cleaner but the boss type is not a single discriminant enum — it requires querying component presence. The chain is the correct Bevy approach.

---

### Open questions

1. Should `HagIdentity::index()` live in `game/units/boss/hags/components.rs` (owned by the hags auditor) or should the UI components store a `usize` index directly? The former is cleaner; the latter avoids touching an out-of-scope file.
2. Do bosses appear in multiplayer at all? If the boss bar is SP/co-op-host only, the `spawn_boss_health_bar` + `update_boss_health_bar` pair would benefit from an explicit `in_state(AppState::InGame)` guard alongside `is_gameplay_running` to make the intent clearer.
3. Ray eye section abbreviations ("Pet", "Dis", "MC", "Tele") are UI-layout-driven choices embedded in the spawn code. Should they be defined on `RayEyeType` itself (alongside the existing `index()` method) to keep all per-type data in one place?
