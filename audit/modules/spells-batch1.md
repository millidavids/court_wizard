## spells-batch1

**Scope:** `src/game/units/wizard/spells/fireball/`, `wall_of_fire/`, `magic_missile/`

---

### Mental Model

Three foundational fire/arcane spells that together form the core of the wizard's combat toolkit. Each follows a consistent three-layer architecture: `casting.rs` handles input and state-machine transitions, a physics/collision file (`projectile.rs`, `damage.rs`, `missile.rs`) handles simulation, and `components.rs`/`constants.rs` hold data. All three spells route through shared helpers in `spells/utils.rs` and `spells/audio.rs`. Talent systems are fully inline (no separate file), with talent flags baked into component fields at spawn time.

Fireball is the most complex: it has a multi-stage explosion lifecycle with sub-bubble VFX, ghost-gated damage, a napalm trail, cluster bomb sub-projectiles, and Scorched Earth persistent zones. Wall of Fire is a drag-to-place line AoE with 9 talent permutations, a preview entity lifecycle, pathfinding obstacle registration/deregistration, and five subsidiary systems. Magic Missile is an instant-cast homing volley with a rich retargeting algorithm and six talent combinations including a concentration mode (Arcane Barrage).

MP ghost gating is handled carefully in fireball (`GhostSpellEffect` filters on damage and cleanup systems), but wall_of_fire's damage and cleanup systems lack the equivalent filter even though ghost `WallOfFireEffect` entities are spawned on the guest. Magic missile sidesteps the issue by using a separate `GhostMagicMissile` marker component that does not carry the `MagicMissile` simulation component.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F-01 | ArchitecturalDecay | `wall_of_fire/casting.rs:537` | Medium | S | `apply_wall_of_fire_damage`, `cleanup_wall_of_fire`, and allied talent systems live in `damage.rs`, but `handle_wall_of_fire_cancel` (a casting-lifecycle function) also lives there, mixing two concerns in one file. | Move `handle_wall_of_fire_cancel` to `casting.rs` where the rest of the casting lifecycle lives. |
| F-02 | Multiplayer (ErrorObservability) | `wall_of_fire/damage.rs:68-167` | High | S | `apply_wall_of_fire_damage` queries `WallOfFireEffect` without a `Without<GhostSpellEffect>` filter. Ghost walls spawned on the guest have `WallOfFireEffect` with 0.0 damage but non-zero `half_width`; the system still ticks `time_alive`, writes zero-radius `TerrainDamageMessage`s at Vec3::ZERO, evaluates `InsideWallOfFire` insertion, and may apply `SearingHeatDebuff` / `InsideWallOfFire` / Firestorm markers from the ghost wall. The fireball module correctly gates both `apply_explosion_damage` and `cleanup_finished_explosions` with `Without<GhostSpellEffect>`. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the `effects` query in `apply_wall_of_fire_damage`. Mirror the pattern from `fireball/projectile.rs:513`. |
| F-03 | Multiplayer (ErrorObservability) | `wall_of_fire/damage.rs:171-226` | High | S | `cleanup_wall_of_fire` has no `GhostSpellEffect` guard. It will despawn ghost `WallOfFireEffect` entities when their local `time_alive` reaches `duration`, bypassing snapshot reconciliation and leaving a stale entry in `SpellEffectEntityMap`. | Add `Without<GhostSpellEffect>` to the effects query in `cleanup_wall_of_fire`, matching `cleanup_finished_explosions` in fireball. |
| F-04 | DocDrift | `wall_of_fire/damage.rs:32-36` | Low | S | The four-line doc comment "Computes the axis-aligned bounding box of a rotated wall…" describes `wall_obstacle_bounds` from `casting.rs`, but it is directly attached to `handle_wall_of_fire_cancel`. This is a copy-paste residue from when the file was reorganised. | Replace the stale doc comment with one that correctly describes `handle_wall_of_fire_cancel`. |
| F-05 | ArchitecturalDecay | `wall_of_fire/damage.rs:287` | Low | S | `WALL_SMOKE_INTERVAL` is a file-private constant (`const WALL_SMOKE_INTERVAL: f32 = 0.25`) defined at line 287 inside `damage.rs`, while all other timing constants for this spell live in `constants.rs`. | Move `WALL_SMOKE_INTERVAL` to `constants.rs` as `pub(super)` to keep all WoF tuning values in one place. |
| F-06 | ConsistencyRot | `wall_of_fire/casting.rs:9-10`, `damage.rs:9-10` | Low | S | Both `casting.rs` and `damage.rs` have the dual import `use super::constants; use super::constants::*;`. The bare `constants` import is then used as `constants::FOO` while `*` also brings everything into scope. This is redundant and obscures which names come from the glob. Other spells (e.g., fireball, magic_missile) use only one form. | Remove the `use super::constants;` line and qualify only the constants that need disambiguation, or use only `use super::constants::*;` consistently. |
| F-07 | TypeContract | `fireball/components.rs:100-101`, `wall_of_fire/components.rs:22-23`, `magic_missile/components.rs:84-85` | Low | S | Three `damage_type` fields across `FireballExplosion`, `WallOfFireEffect`, and `MagicMissile` carry `#[allow(dead_code)]`. `FireballExplosion.damage_type` is actually read at `projectile.rs:550` and `projectile.rs:577`, making the suppression a false negative. `Fireball.damage_type` is genuinely stored-but-not-read. | Remove `#[allow(dead_code)]` from `FireballExplosion.damage_type` (the field is live). For `Fireball.damage_type` and `WallOfFireEffect.damage_type`, either use the field consistently (e.g., pass it through rather than hard-coding `DamageType::Fire`) or remove the field and its constructor parameter. |
| F-08 | ArchitecturalDecay | `fireball/projectile.rs:319-342` | Low | S | `spawn_cluster_bombs` contains three magic numbers inlined into the function body (`100.0`, `300.0`, `0.4`, `15.0`) with no named constants. These are tuning values for the Cluster Bomb talent but are absent from `constants.rs`. | Extract `CLUSTER_BOMB_MIN_DISTANCE`, `CLUSTER_BOMB_MAX_DISTANCE`, `CLUSTER_BOMB_FLIGHT_TIME`, and `CLUSTER_BOMB_VISUAL_RADIUS` into `fireball/constants.rs`. |
| F-09 | DocDrift | `fireball/projectile.rs:24` | Low | S | `move_fireballs` carries the doc comment "Local wizard fireball casting — reads mouse input." This is copied from `casting.rs`; `move_fireballs` does not read mouse input at all. | Replace with "Advances all fireball projectiles by their velocity each frame." |

---

### Oversized Files

| File | LOC | Exempt | Reason / Split Into |
|------|-----|--------|---------------------|
| `fireball/projectile.rs` | 745 | No | Split into `fireball/explosion.rs` (FireballExplosion systems: `update_explosions`, `spawn_explosion_bubbles`, `fade_explosion_spheres`, `apply_explosion_damage`, `cleanup_finished_explosions`, `spawn_explosion_with_talents`) and `fireball/trail_effects.rs` (`update_napalm_trails`, `spawn_fireball_smoke_trail`, `spawn_scorched_earth_fire_smoke`). Keep `move_fireballs`, `check_fireball_collisions`, `despawn_distant_fireballs`, `spawn_cluster_bombs` in `projectile.rs` (~250 LOC). |
| `wall_of_fire/casting.rs` | 537 | No | Split into `wall_of_fire/placement.rs` (the drag-to-place logic: `wall_of_fire_casting_logic`, `compute_talent_params`, `wall_transform`, `wall_obstacle_bounds`) and keep the system entry point + preview management in `casting.rs`. ~280 LOC each. |
| `wall_of_fire/damage.rs` | 480 | No | Split into `wall_of_fire/talent_effects.rs` (the five talent systems: `track_wall_of_fire_exit`, `apply_spreading_flames_dot`, `apply_scorched_earth_slow`, `firestorm_death_explosion`, `cleanup_wall_of_fire_sfx`) leaving core tick/damage/cleanup in `damage.rs`. ~240 LOC each. |
| `magic_missile/casting.rs` | 492 | No | Split into `magic_missile/targeting.rs` (`spawn_magic_missile_with_talents`, the target-selection block, crystal targeting) leaving `handle_magic_missile_casting` and `update_arcane_barrage` in `casting.rs`. ~260 LOC each. |
| `magic_missile/missile.rs` | 435 | No | Borderline — all content is genuinely cohesive (movement + collision for one entity type). At 435 LOC with two well-named public functions it is acceptable now but will need splitting when the next talent variant is added. |

---

### Looks Bad But Is Actually Fine

- **`Fireball` component has many boolean talent flags**: `cluster_bomb`, `napalm`, `scorched_earth`, `chain_ignition` could look like they violate the "small focused components" rule. However these are bake-at-spawn parameters, not independently queried markers — no system uses `With<ClusterBomb>` — so they are correctly modelled as fields rather than components.
- **`check_fireball_collisions` closure `explode_at`**: The inner closure captures by mutable reference but is called in separate arms. This looks like it might borrow-split, but each arm calls it once with `&mut commands` and `&mut sphere_materials` captured, then breaks/continues. The borrow checker accepts it and the pattern avoids code duplication.
- **`spawn_explosion_with_talents` has `_time_secs: f32`**: The underscore prefix explicitly signals intentional non-use; this is a clean API that was prepared for time-seeded VFX and may be wired up later. Not dead code — it is part of the function signature contract.
- **Dual `use super::constants; use super::constants::*;`**: While flagged as a consistency issue (F-06), it compiles correctly and there is no ambiguity at runtime.
- **`apply_wall_of_fire_damage` ghost wall with `damage_per_tick = 0.0`**: The zero damage prevents actual HP deduction. However the system still mutates `time_alive` and writes `TerrainDamageMessage`s, so this is not completely harmless (F-02 is real).
- **`MagicMissile.update_magic_missiles` retargeting `Vec<Entity>` allocation**: Only runs when a missile's target despawns, not every frame. Acceptable as-is.
- **`fireball/systems.rs` and `wall_of_fire/systems.rs` are trivial re-export hubs**: These exist to maintain a stable `systems::*` import surface after Phase 14 splitting. This is an intentional pattern, not a violation.

---

### Open Questions

1. **WoF ghost wall lifecycle**: With F-02/F-03 fixed, `time_alive` will no longer tick on ghost walls via `apply_wall_of_fire_damage`. Does `spawn_wall_of_fire_smoke` also need a ghost guard, or does guest reconciliation despawn the ghost before the duration timer would fire?
2. **`Fireball.damage_type` field**: After F-07, if `Fireball.damage_type` is removed and `spawn_explosion_with_talents` hard-codes `constants::DAMAGE_TYPE`, does this close off any future path to mixed-damage-type fireball variants (e.g., from arcane crystal absorption)?
3. **`firestorm_death_explosion` on ghost units**: The Firestorm talent marks real unit entities (not spell effects) with `FirestormMarked`. If a guest-side wizard fires WoF, the host marks those units. When the units die on the guest's simulation, does `firestorm_death_explosion` fire on the guest as well, producing a duplicate explosion? The system has no `Without<GhostSpellEffect>` guard but unit entities are not ghosts — confirm this is safe in the MP handshake.
