## ui-in_game

**Scope:** `src/ui/in_game/` — in-game HUD: mana/cast/boss/king health bars, buff tracker, wave display, flash banners, and HUD input handling.

---

### Mental model

The module is feature-sliced into five focused files managed through a `systems.rs` re-export hub. `plugin.rs` is registration-only and well-commented. `spawn.rs` drives HUD layout (SP and MP variants). `bars.rs` contains all bar-update systems plus boss-bar spawn logic. `displays.rs` owns wave, buff-tracker, and flash-banner systems. `setup_banner.rs` is a self-contained MP-only countdown widget. `constants.rs` and `components.rs` are clean. The overall architecture is sound — all `Update` systems carry `run_if` guards, no `.unwrap()` in production paths, messaging is `#[derive(Message)]`-based throughout.

The two main pressure points are: (1) `spawn.rs` is 726 LOC with roughly 130 lines of near-identical bottom-bar widget code duplicated between `spawn_hud` and `spawn_mp_hud`, and (2) `bars.rs` is 780 LOC — legitimately cohesive (one concern: bar-update systems), but the boss-bar spawn logic for five boss types inflates it.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | DocDrift | `bars.rs:22` | Low | S | Doc comment "Blocks spell input when the mouse is interacting with a UI button…" is copied verbatim from `block_spell_input_on_button_interaction` onto `update_mana_bar`. The comment describes a completely different system. | Replace with a correct one-liner describing `update_mana_bar`. |
| F02 | DocDrift | `displays.rs:17` | Low | S | Same stale "Blocks spell input…" doc comment copied onto `update_wave_display`. | Replace with a correct doc comment describing `update_wave_display`. |
| F03 | ArchitecturalDecay | `spawn.rs:177` + `spawn.rs:464` | Medium | M | `spawn_hud` and `spawn_mp_hud` share ~130 lines of identical bottom-bar widget code: the ammo-display branch (lines 329–361 vs 561–591), the mana-bar-with-reserved-fill branch (362–408 vs 592–638), and the cast-bar-with-overlay section (410–457 vs 640–686). Every future change to these widgets must be made twice. | Extract `spawn_bottom_bars(parent: &mut ChildSpawnerCommands, config: &GameConfig)` and `spawn_hud_button_row(row, config)` helpers and call them from both functions. The "Spells" button hide list (lines 217–221 and 497–501) is also duplicated and belongs in the helper. |
| F04 | ConsistencyRot | `spawn.rs:315` + `spawn.rs:560` | Low | S | Two different spellings for "is this wizard the gunslinger?": line 315 creates `let is_gunslinger = config.wizard_type == WizardType::Warglock` and uses it a few lines later in `spawn_hud`; `spawn_mp_hud` (line 560) inlines `config.wizard_type == WizardType::Warglock` directly. The variable name `is_gunslinger` also does not match the public archetype name `Warglock`. | After extracting F03, a single `is_warglock` (or `is_gunslinger`) local survives in the shared helper. Align the name with either the WizardType variant or add a `WizardType::is_gunslinger()` predicate. |
| F05 | ConsistencyRot | `bars.rs:657` + `bars.rs:670` + `bars.rs:683` | Low | S | `HagIdentity` is mapped to a `[0, 1, 2]` array index via a three-way `match` block that is repeated three times in `update_boss_health_bar`. `RayEyeType` (same file) exposes an `index()` method that eliminates this repetition for ray eyes. `HagIdentity` has no such method, causing redundant match arms. | Add `pub const fn index(self) -> usize` to `HagIdentity` (in `game/units/boss/hags/components.rs`) and replace all three match blocks with `identity.index()`. |
| F06 | ConsistencyRot | `spawn.rs:219` vs `spawn.rs:705` | Low | S | The "hide Spells button" check (lines 219–221, also 498–501) lists `Warglock \| Randomancer \| RuneCaster` explicitly, while the runtime guard in `hud_button_action` (line 705) uses `config.wizard_type.uses_exclusive_casting()` — which only covers `RuneCaster \| Randomancer`, not `Warglock`. A future archetype that `uses_exclusive_casting()` would automatically get the runtime guard but not the spawn-time button hide. | Either extend `uses_exclusive_casting` to cover `Warglock`, or add a separate `fn hides_spell_button()` predicate that all three sites call. |
| F07 | ConsistencyRot | `spawn.rs:348` + `spawn.rs:578` | Low | S | The ammo display is initially populated using `GunType::MachineGun.max_ammo() / GunType::MachineGun.ammo_per_ui_piece()` (= 12 pieces) in both `spawn_hud` and `spawn_mp_hud`. This hardcodes the maximum piece count to MachineGun. If a future gun requires more UI pieces (Flamethrower currently needs 10, so 12 covers it, but only by coincidence), the initial spawn will be wrong. | Replace with `GunType::all().iter().map(|g| g.max_ammo() / g.ammo_per_ui_piece()).max().unwrap_or(12)` or extract a `GUN_MAX_UI_PIECES` constant derived from all variants. |
| F08 | DocDrift | `displays.rs:353` | Low | S | Tooltip top-offset is `BUFF_BOX_SIZE + BUFF_BOX_GAP + 20.0 + 10.0`. The `20.0` and `10.0` addends are unexplained magic numbers (likely HUD margin + extra padding). | Replace with named constants or a comment explaining what the addends represent. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `bars.rs` | 780 | No | Contains both bar-update systems and boss-bar spawn logic for five boss types. Proposed split: `bars.rs` (mana, ammo, cast, king bar update systems, ~300 LOC) + `boss_bar.rs` (boss health bar spawn + update for Ogre/Hag/Lich/DarkMage/Ray, ~480 LOC). |
| `spawn.rs` | 726 | No | Two large HUD spawn functions with duplicated widget code, plus input handling. Proposed split: `spawn.rs` (SP+MP spawn orchestration, ~300 LOC after F03 extraction) + `input.rs` (keyboard_input, gamepad_hud_shortcuts, hud_button_action, block_spell_input, ~120 LOC). After F03, widget helpers could live in a `widgets.rs` (~100 LOC). |

---

### Looks bad but is actually fine

- **`update_boss_health_bar` has 9 query parameters** (`bars.rs:615`): This is idiomatic Bevy — the system genuinely needs separate queries for Ogre, Hag (fill + text), Lich (health + soul power + phase), ray bar fill, and despawn — all filtered against each other with `Without<T>`. `#[allow(clippy::too_many_arguments)]` is present and appropriate.
- **`systems.rs` is a pure re-export hub** (5 lines): This looks like an anti-pattern at first glance, but it is the deliberate indirection hub cited in the module comment ("Phase 16 split"). `plugin.rs` imports `systems::*` so all system names stay stable as files are rearranged. Not a violation.
- **`update_buff_timers` formats strings every frame**: It calls `format!("{:.0}s", ...)` once per active buff per frame. Active buff counts are small (0–8) and the allocation overhead is negligible. No change needed.
- **`spawn_hud` reads `WaveState` only to check `total_waves > 1`**: This is a one-time spawn read, not a per-frame query. No run_if issue.
- **`update_level_clock` iterates `clock_query` in a `for` loop**: There is at most one `LevelClockDisplay` node; the loop is equivalent to `single_mut()`. The loop form safely handles the "no node yet" case without a branch.

---

### Open questions

1. Should `HagIdentity::index()` live in the `hags/components.rs` file (owned by `units-boss-hags`) or should the UI's `HagHealthBarFill.identity` carry a `usize` index directly? The former is cleaner but requires touching a file outside this scope.
2. Do bosses appear in multiplayer at all (the snapshot code has no Boss/Hag/Lich entries), or is the boss health bar exclusively single-player + co-op-host? If MP-exclusive-host, the `spawn_boss_health_bar` / `update_boss_health_bar` pair's `is_gameplay_running` gate is already correct, but worth explicitly documenting.
3. After extracting the bottom-bar widgets (F03), should the extracted `spawn_bottom_bars` helper live in `spawn.rs` or in a new `widgets.rs`?
