## units-batch1

**Scope:** `src/game/units/archer/`, `src/game/units/infantry/`, `src/game/units/king/`, `src/game/units/healer/`

---

### Mental model

These four modules handle the game's core non-wizard units. **Infantry** and **Archer** are the numerically dominant armies (defenders + attackers). **King** is a single defender-side commander with an aura, spell shield (MP only), and a guard ring. **Healer** is a late-game attacker upgrade (converted from archers) that channels a bolt-heal.

All four share the same movement boilerplate — `is_cc_immobilized → polymorphed wander → calculate_weighted_movement` — delegating to cross-cutting `units/systems.rs` helpers. Every plugin uses `is_gameplay_running` + `any_exist`/`any_with_component` guards on batches of systems. The codebase is structurally sound and feature-sliced: no file exceeds 300 lines, `plugin.rs` files are registration-only, `mod.rs` files are re-export-only, systems submodules are cleanly split by concern.

The main debt is: (a) dead query fields in `archer_ranged_combat` forcing unnecessary write-locks, (b) a dead asset loaded but never referenced, (c) hardcoded spawn coordinates in `spawn_single_kings_guard` that diverge from the King's own spawn formula, (d) two king systems running unconditionally without `is_gameplay_running`, (e) near-identical ally-snapshot build repeated across healer systems, and (f) two copy-pasted doc comments on archer functions.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B1-01 | Performance | `archer/combat/ranged.rs:108,111` | High | S | `archer_ranged_combat` queries `&Hitbox` (destructured as `_archer_hitbox`) and `&mut AttackTiming` (destructured as `_attack_timing`) but neither field is read or written in the system body. The `&mut AttackTiming` write-lock prevents Bevy from running this system in parallel with any other system that needs `AttackTiming` (including `archer_melee_combat`). | Remove both fields from the query tuple. Ranged cooldown is tracked via `ArcherMovementTimer`, not `AttackTiming`. The hitbox is irrelevant for range checks. |
| B1-02 | ArchitecturalDecay | `healer/resources.rs:11,19,41` | Medium | S | `HealerAssets.attacking_texture` is loaded at startup (`asset_server.load(...)`) but is never read anywhere in the codebase. The field is wrapped by `#[allow(dead_code)]` on the struct, silently suppressing the compiler warning and masking the waste. | Remove `attacking_texture`, its load call, and its assignment in `preload_healer_assets`. If a healer melee animation is planned, add it back with a `// TODO:` comment explaining the intent. |
| B1-03 | DependencyConfig | `infantry/systems/spawn.rs:159-163` | Medium | S | `spawn_single_kings_guard` hard-codes spawn position as a manual centroid of four raw castle corner coordinates (`(-1700 + -1400 + -1700 + -1400) / 4 + 100`, `(1200+1200+1500+1500) / 4`). The King itself spawns via `WIZARD_POSITION + radius * angle` in `king/systems/spawn.rs`. If `CASTLE_POSITION` or `WIZARD_OFFSET` changes, King and King's Guard will silently diverge. | Replace the four magic literals with `crate::game::constants::defender_spawn_center()` (already exists in `constants/spawn_math.rs:100`), or derive position from `WIZARD_POSITION` directly, to keep guard spawn in sync with the canonical constants. |
| B1-04 | ArchitecturalDecay | `king/plugin.rs:40-58` | Medium | S | `attach_king_spell_shield` (line 43) and `despawn_king_aura_on_death` (line 56) are registered in an `add_systems(Update, ...)` block with **no** `run_if(is_gameplay_running)` guard. Both run every frame regardless of app state (menus, loading, game over). `attach_king_spell_shield` checks a session resource every frame; `despawn_king_aura_on_death` uses `Added<Corpse>` so is very cheap, but both violate the project convention that every Update system must have a `run_if` guard. | Add `.run_if(is_gameplay_running)` to the tuple at lines 40-58. For `attach_king_spell_shield`, also consider adding `Added<King>` as a change-detection filter (it already internally gates on `Added<King>` but at runtime, not at scheduling). |
| B1-05 | Performance | `healer/systems/targeting.rs:54-75` and `healer/systems/channel.rs:57-78` | Medium | M | Both `update_healer_targeting` and `healer_start_heal_channel` independently build an identical `Vec<(Entity, Vec3, Team, f32, f32, u32)>` ally snapshot from the same `potential_targets` query every frame, mapping 9 query fields through `find_heal_priority`. `healer_tick_heal_channel` at least uses lazy `get_or_insert_with` to avoid the build when no channel fires, but the other two systems do not benefit from that. | Extract a `build_ally_snapshot(q: &Query<...>) -> Vec<AllySnapshotEntry>` helper called once per system. If the systems run sequentially (they do — same Update batch), a `Local<Vec<_>>` populated once on the first system that needs it and passed as a slice would avoid both allocations. |
| B1-06 | DocDrift | `archer/arrows.rs:15-16` | Low | S | `spawn_arrow` has the doc comment "Updates archer movement timers to track time since stopped moving." — clearly copy-pasted from `update_archer_movement_timers` in `combat/ranged.rs`. The function actually spawns a projectile entity. | Replace with an accurate doc comment, e.g. "Spawns a ballistic arrow from `origin` aimed at `target` with physics-based trajectory." |
| B1-07 | DocDrift | `archer/movement/targeting.rs:13` | Low | S | `update_archer_targeting` has the same wrong doc comment: "Updates archer movement timers to track time since stopped moving." This function sets targeting velocity. | Replace with e.g. "Sets each archer's `TargetingVelocity` toward the nearest enemy in attack range, applying LOS checks." |
| B1-08 | ConsistencyRot | `archer/arrows.rs:179-186` | Low | S | `check_arrow_collisions` performs a redundant two-step same-team skip: first `if *team == arrow.source_team { continue }` (line 179), then `let is_enemy = arrow.source_team.is_enemy(team); if !is_enemy { continue }` (lines 183-186). The first check is a strict subset of the second; any non-enemy team (same team or neutral) would have been caught by the second check already. | Remove the `if *team == arrow.source_team` guard; `is_enemy(team)` handles all team combinations. |
| B1-09 | TypeContract | `archer/components.rs:47` and `healer/components.rs:33` | Low | S | `ArcherMovementTimer::new()` and `HealerAttackTimer::new()` both initialise `time_since_last_attack` to `999.0` with a comment "Start high so can attack immediately." The magic `999.0` has no relationship to any named cooldown constant (`HEAL_COOLDOWN`, `ARCHER_ATTACK_COOLDOWN_MULTIPLIER`). | Define a named sentinel (e.g. `const ATTACK_TIMER_READY: f32 = f32::MAX;` or a large named constant) and use it in both `new()` implementations. |

---

### Oversized files

No `.rs` file in scope exceeds 300 lines. All files pass the size threshold.

| File | LOC |
|------|-----|
| `archer/arrows.rs` | 206 (largest in scope) |

---

### Looks bad but is actually fine

- **Per-frame `Vec` snapshots in archer targeting/combat** — `wall_snapshot`, `rock_snapshot`, `tree_snapshot` are collected independently in `update_archer_targeting` and `archer_ranged_combat`. This is the idiomatic Bevy pattern for avoiding aliased borrows across query splits. The collections are small (terrain objects are sparse) and cannot be shared across system boundaries without introducing a resource.
- **`archer/systems.rs` is a 5-line re-export hub** — appears to be a vestigial shim from the Phase 18 module split. It performs no logic and is explicitly documented as a re-export hub. Not a violation.
- **Ghost gating absent in targeting systems** — none of the gameplay systems in scope add `Without<GhostEntity>` to their queries. This is safe: `VelocitySystemSet` and `MovementCalculationSet` are configured at `game/plugin.rs` level to gate on `is_not_mp_setup_phase` and `is_gameplay_running`. On the guest, ghost entities follow the snapshot replication path, not the local simulation path. The per-query ghost filter is only needed in systems that run on both peers simultaneously (e.g., animation systems), not in host-only simulation systems.
- **`king_cohesion_force:192 .unwrap_or(f32::MAX)`** — this is a safe iterator `.unwrap_or`, not a production `.unwrap()`. The `f32::MAX` fallback correctly means "no threat", producing minimum cohesion.
- **`attach_king_spell_shield` `Added<King>` filter** — the internal `Added` change detection makes this system free on most frames. Finding B1-04 is still worth fixing for correctness, but the per-frame cost is negligible.
- **`healer_tick_heal_channel` lazy snapshot** — uses `get_or_insert_with` so the ally snapshot is skipped entirely if no channel fires that frame. This is a deliberate micro-optimization and correct design.
- **`HEALER_SPRITE_TINT` value identical to archer `ATTACKER_SPRITE_TINT`** — both are `Color::srgb(0.75, 0.65, 0.65)`. These are intentionally separate constants for separate unit types. The comment on `archer/constants.rs:46` even flags it as "Lighter attacker tint for archers."

---

### Open questions

1. **Can the King be polymorphed?** `king/systems/movement.rs` queries `Option<&PolymorphedModifier>` as `_polymorphed` and never uses it — no wander branch exists for the King. If polymorph can target the King, the effect is silently swallowed. If it cannot, `Without<King>` should be added to the polymorph spell's targets query.
2. **King's Guard spawn offset (B1-03):** The hardcoded centroid resolves to approximately `(-1450, 1350)` while `WIZARD_POSITION ≈ (-1400, 1700)`. Are guards intentionally placed at a different position than the King, or has this drifted since the castle position was established?
3. **`_level` parameter in `spawn_single_attacker` / `spawn_single_attacker_archer`:** Both accept a `_level: u32` that is completely ignored. Is level-based stat scaling planned? If not, this should be removed. If yes, a `// TODO:` comment would prevent it from being silently deleted.
