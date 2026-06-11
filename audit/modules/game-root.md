## game-root

**Scope**: `src/game/*.rs` — root-level files only (15 files, 3 528 LOC total)

---

### Mental Model

`src/game/` is the cross-cutting hub of the game's simulation. It contains the primary Bevy `GamePlugin` (`plugin.rs`) which orchestrates every sub-plugin, three game-wide system-set definitions (`sets.rs`), top-level gameplay systems split across `shared_systems.rs` (timers, ambience, shadows, resource resets), `movement_systems.rs` (separation/flocking, wall collision), `wave_systems.rs` (wave spawning + upgrade application), `win_lose_systems.rs`, and `run_conditions.rs`. Supporting files hold the cross-cutting component/resource types (`components.rs`, `resources.rs`), a constants registry (`constants.rs`, 697 LOC), and small concern files (`debug_ui.rs`, `systems.rs`, `insight_bonuses.rs`, `messages.rs`, `sets.rs`). `plugin.rs` is intentionally large because it is primarily registration, but it also houses `GlobalAttackCycle`, `DebugHitboxes`, and two non-trivial helper functions — a mild but real purity violation.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| G-01 | ArchitecturalDecay | `plugin.rs:50-81` | High | S | `GlobalAttackCycle` (struct + `Default` + `tick()`) is defined inside `plugin.rs` rather than in a concern-focused file. Convention says `plugin.rs` is registration only. | Move `GlobalAttackCycle` to `shared_systems.rs` or a new `attack_cycle.rs`. |
| G-02 | ArchitecturalDecay | `plugin.rs:350-449` | High | S | `apply_game_speed`, `auto_pause_on_focus_loss`, `DebugHitboxes`, `DebugHitboxMarker`, `sync_debug_hitboxes_resource`, and `update_debug_hitboxes` are all defined in `plugin.rs`. This is a direct violation of the "registration only" rule. | Move the debug hitbox types and systems to `debug_ui.rs`. Move `apply_game_speed` and `auto_pause_on_focus_loss` to `shared_systems.rs`. |
| G-03 | ConsistencyRot | `shared_systems.rs:182` vs `constants.rs:446` | Medium | S | `ENGAGEMENT_RANGE: f32 = 800.0` is defined as an inline `const` inside `activate_defenders_on_proximity` while `DEFENDER_ACTIVATION_RANGE: f32 = 800.0` already exists in `constants.rs` for exactly this purpose. Both are 800.0; the inline one is silently diverging. | Remove the inline const; use `super::constants::DEFENDER_ACTIVATION_RANGE`. |
| G-04 | ArchitecturalDecay | `constants.rs:664-696` | Low | S | `calculate_defender_grid_position` carries two stacked doc-comments (lines 664–677 describe an attacker grid; lines 673–687 are the real defender grid doc). The first block is a leftover paste artifact. | Remove the first (wrong) doc block; keep only the defender-specific comment. |
| G-05 | ArchitecturalDecay | `constants.rs:518-521` | Low | S | `is_ray_level()` is annotated `#[allow(dead_code)]` and has no callers anywhere in the codebase. The Ray boss exists but the predicate is never used. | Either wire it up to the boss spawn logic, or delete it to keep the API clean. |
| G-06 | ArchitecturalDecay | `messages.rs:27-31` | Low | S | `WaveSpawnedMessage` (including its `wave_number` field, annotated `#[allow(dead_code)]`) is written by `tick_wave_timer` but has zero `MessageReader` subscribers. The message is registered and sent but never consumed. | Either wire up a subscriber (e.g., UI wave-number toast) or remove the message and the `MessageWriter` injection from `tick_wave_timer`. |
| G-07 | Performance | `movement_systems.rs:65-72` | Medium | M | Inside `apply_separation`, `current_positions` is re-collected as a fresh `Vec` on every iteration of the `COLLISION_ITERATIONS=4` loop. That means 4 heap allocations per frame over all units (called every tick in `VelocitySystemSet`). | Collect `current_positions` once before the loop and update positions in-place, or use a single snapshot with a second pass to avoid re-querying. |
| G-08 | Performance | `shared_systems.rs:68-111` | Medium | M | `calculate_effectiveness` is an O(N²) double-pass over all non-boss non-corpse units every frame. For a large battle (200+ units) this is ~40 000 distance checks per frame. There is no early-out for units already at min/max effectiveness. | Add a dirty flag or coarse spatial hash to skip recalculation for units whose neighborhood hasn't changed. At minimum, clamp and skip recalculation if already at `EFFECTIVENESS_MIN` or `EFFECTIVENESS_MAX` from the previous frame. |
| G-09 | Performance | `movement_systems.rs:42-57` | Low | S | `unit_data` (the shared snapshot for the flocking pass) is collected unconditionally before the `COLLISION_ITERATIONS` hard-collision loop. This second `Vec` allocation per frame is fine, but inside the loop `current_positions` is a *duplicate* of `unit_data` with fewer fields. The positions snapshot already exists in `unit_data`; the extra collection buys nothing. | Reuse `unit_data` for the position-snapshot pass instead of collecting `current_positions` separately. |
| G-10 | ConsistencyRot | `debug_ui.rs:52` | Low | S | `toggle_debug_ui_visible` is registered in `Update` with no `run_if` guard. All other `Update` systems in the project carry explicit run conditions. The keyboard read is cheap, but the convention is violated and it runs even outside gameplay. | Add `.run_if(not(in_state(AppState::Loading)))` or equivalent to match project convention. |
| G-11 | DocDrift | `constants.rs:123-135` | Low | S | The doc-comment on `SPELL_ORIGIN` at line 122 says "Offset from wizard position to place the cauldron beside the wizard" (inherited from the `SPELL_OFFSET` constant above it whose own doc says the same). The actual doc for `SPELL_ORIGIN` should describe spell projectile origin, not cauldron placement. | Fix the stale copy-paste doc on line 122 to correctly describe `SPELL_ORIGIN`. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|-------------------------|
| `constants.rs` | 697 | No | Mixed concerns: visual colors (lines 12–77), world positions (88–195), MP constants + spawn math functions (197–233), gameplay tuning (235–460), level-progression logic (462–578), grid position helpers (580–697). Propose split into: `colors.rs`, `positions.rs`, `spawn_math.rs`, `tuning.rs` (or inline small groups into their feature files). |
| `shared_systems.rs` | 521 | No | Contains battle ambience subsystem (lines 246–374) + `ShadowMaterial` + unit shadow subsystem (376–521) + gameplay helpers (1–245). Propose split: `ambience.rs` (battle/crowd sound), `shadows.rs` (shadow material, asset, spawn/update), keep helpers in `shared_systems.rs`. |
| `movement_systems.rs` | 581 | No | Contains flocking/separation (1–283), rough terrain (285–328), wall suppression (330–363), wall avoidance (365–449), wall/boulder collision (451–581). Propose split: `flocking.rs`, `wall_collision.rs` (avoidance + enforce), `rough_terrain.rs`. |
| `plugin.rs` | 449 | No | Core registration is fine, but non-registration types and system bodies are embedded (see G-01, G-02). After moving those out, the file will be ~340 LOC of pure registration, which is acceptable. |
| `wave_systems.rs` | 345 | No | Two logically distinct systems: `tick_wave_timer` (wave clock + spawn) and `apply_wave_upgrades` (elite/commander/dispeller upgrade application). Propose split into `spawn.rs` and `upgrades.rs` within a `wave/` subfolder, or keep in root as `wave_spawn.rs` + `wave_upgrades.rs`. |

---

### Looks Bad But Is Actually Fine

- **`plugin.rs` length (449 LOC)**: The bulk is legitimate Bevy system registration chains with inline documentation. Long registration blocks are common in Bevy codebases and not a violation per project rules — only embedded logic is.
- **`calculate_effectiveness` querying `Without<Boss>`**: Looks like an ad-hoc exclusion, but bossses have their own team-effectiveness model; excluding them from the shared calculation is intentional.
- **`apply_wave_upgrades` missing `is_gameplay_running`**: The function is gated only on `resource_exists::<PendingWaveUpgrades>`. The resource is only inserted by `tick_wave_timer` (which is SP+InGame only), so `apply_wave_upgrades` can never fire in MP. The simpler gate is safe.
- **`WaveSpawnedMessage` registered in `GamePlugin` not in a sub-plugin**: Messages shared across multiple consumers (or potential future consumers) belong at the top-level plugin, even if currently only written by one system.
- **O(N²) in `apply_separation`**: The outer query already runs inside `VelocitySystemSet` which is gated to `is_gameplay_running` and `is_not_mp_setup_phase`. The current load (typically 150–250 units) is within acceptable range; the double-collect finding (G-07) is the actionable part.
- **`GlobalAttackCycle::tick()` method in plugin.rs**: It is a two-line method on a resource, not a standalone system function. Moving the resource out (G-01) is still correct, but the method itself is not a system-body violation.
- **`is_not_mp_setup_phase` having a constant (`MP_SETUP_DURATION`) in `run_conditions.rs`**: The constant is used exclusively by this function in this file; per project convention, constants used by exactly one feature file should be inlined there. This is already compliant.

---

### Open Questions

1. **WaveSpawnedMessage subscribers**: Was there a UI wave-number overlay that was removed without also removing the message emit? Or is there a planned subscriber?
2. **`is_ray_level` dead code**: Ray boss presumably exists as a spawnable — why doesn't the wave/boss selector call `is_ray_level`? Is it intentionally bypassed in the current boss cycling logic?
3. **`constants.rs` scope creep**: As level-progression logic grows (e.g., `calculate_total_aerialists` importing from `units::aerialist::constants`), should this file become a module `constants/` to prevent a hard circular-dependency wall?
4. **`apply_separation` COLLISION_ITERATIONS inner-loop re-collection**: Would switching to a parallel `par_iter_mut` for the hard-collision pass be feasible given Bevy 0.18's parallel query support, and would that eliminate the need for the snapshot vec entirely?
