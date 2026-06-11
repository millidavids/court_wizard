## spells-batch1

**Scope:** `src/game/units/wizard/spells/fireball/`, `src/game/units/wizard/spells/wall_of_fire/`, `src/game/units/wizard/spells/magic_missile/`

---

### Mental model

These three spells form the "fire + force burst" core of the wizard's kit. All three follow the same broad casting lifecycle (primed → input → state machine → spawn), rely on `spells/utils.rs` helpers and `SpellVisualAssets`, and have talent trees that bolt extra behaviour onto shared component types at spawn time. Fireball is the most elaborate: a projectile that collides and spawns a growing sphere explosion; talent flags (cluster bomb, napalm, scorched earth, chain ignition, meteor) are stored directly as `bool` fields on the `Fireball` component and transferred to `FireballExplosion` at collision time. Wall of Fire uses a drag-to-draw interaction, stores all talent params in an embedded `WallOfFireTalentParams` struct inside `WallOfFireEffect`, and has the most multiplayer surface area (ghost walls replicated as `WallOfFireEffect` with `damage_per_tick: 0.0`). Magic Missile is an instant-cast homing volley with the most complex movement system; talents are pre-computed into a `MissileParams` struct at cast time and baked into each `MagicMissile` component.

All three modules are well-structured by the project's feature-slice conventions. Most systems carry the correct `run_if` guards and ghost filters. The main issues are: a leaked sub-explosion bubble spawn in the MP path, ungated ghost wall processing in the damage+cleanup systems, a genuine dead-code field on `Fireball`, an oversized `missile.rs` mixing movement and collision, and a handful of magic-number/doc-drift nits.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | TypeContract | `fireball/components.rs:14-16` | Medium | S | `Fireball::damage_type` is suppressed with `#[allow(dead_code)]` and never read back. The explosion spawn always passes `constants::DAMAGE_TYPE` directly rather than forwarding `fireball.damage_type`. The field is written at construction but has no downstream effect — silent divergence if someone changes the field expecting it to take effect. | Either remove the field and the allow-attribute, or pass `fireball.damage_type` into `spawn_explosion_with_talents` so the field is actually honoured. |
| F02 | DocDrift | `fireball/projectile/movement.rs:11` | Low | S | Doc comment on `move_fireballs` reads "Local wizard fireball casting — reads mouse input" — copy-pasted from `casting.rs`. The function just integrates velocity; it does not read input. | Fix to "Advances fireball positions by their velocity each frame." |
| F03 | ArchitecturalDecay | `magic_missile/missile.rs:1-434` | Medium | M | `missile.rs` is 434 lines containing two unrelated systems: `move_magic_missiles` (guided steering + homing, ~190 lines) and `check_magic_missile_collisions` (obstacle + unit hit detection + damage + seeker-swarm split, ~180 lines), plus a private spawn helper. By project convention files exceeding ~300 lines that are not a single cohesive concern must be split. | Split into `movement.rs` (move + despawn) and `collision.rs` (collision detection + detonation + split spawn). |
| F04 | ConsistencyRot | `magic_missile/missile.rs:129` | Low | S | `partial_cmp(...).unwrap_or(std::cmp::Ordering::Equal)` in the "closest enemy fallback" branch. Project convention says no `.unwrap()` in production; `.unwrap_or` on float comparison is the same pattern. | Replace with `a.1.translation.distance(spawn_pos).total_cmp(&b.1.translation.distance(spawn_pos))` which is NaN-safe and needs no unwrap. |
| F05 | Performance | `fireball/projectile/explosion_spawn.rs:23` | Low | S | `_time_secs: f32` parameter is unused (leading `_`) in `spawn_explosion_with_talents`. Dead parameter adds noise to every call site and wastes stack space. | Remove the parameter and update all call sites. |
| F06 | ArchitecturalDecay | `wall_of_fire/casting/spell_casting.rs:112,149` + `placement.rs:50` | Low | S | `preview_height = 10.0` is a local `let` repeated 3 times in `spell_casting.rs` and once in `placement.rs::wall_transform`. If the wall render height changes, all four sites must be updated in sync. | Extract `pub(super) const WALL_RENDER_HEIGHT: f32 = 10.0;` in `constants.rs` and reference it at all four sites. |
| F07 | ConsistencyRot | `fireball/components.rs:66` vs `wall_of_fire/components.rs:172` | Low | S | Two different "scorched earth" effects share near-identical names: `ScorchedEarthFire` (fireball talent — persistent burning ground) and `ScorchedEarthZone` (wall-of-fire talent — slow debuff zone). They appear adjacently in `dispel/bolt.rs` and the names already confused its author's comments. | Rename `ScorchedEarthFire` to `FireballGroundFire` (or `NapalmGroundFire`) to make the owning spell explicit. Update all import sites. |
| F08 | ErrorObservability | `fireball/projectile/explosion_lifecycle.rs:248-249` + `trail_effects.rs:74-75` | Low | S | Two sites call `sphere_materials.get(...).expect("sphere material template").clone()` but `explosion_spawn.rs:45` already uses the shared helper `clone_sphere_material(...)` for the exact same operation. Inconsistent usage: two sites bypass the helper and repeat the `.expect` string independently. | Replace both raw `.get(...).expect(...).clone()` calls with `clone_sphere_material(sphere_materials, &visual_assets.fireball_explosion_sphere)`. |
| F09 | ArchitecturalDecay | `fireball/projectile/movement.rs:65-100` + `magic_missile/missile.rs:241-280` | Medium | M | Wall / rock / tree obstacle-collision is a copy-paste pattern across at least fireball, magic missile, and squall shards — three separate spell modules. Each independently iterates walls (`contains_point_xz`), rocks (`blocks_projectile`), and (in missile) trees. Any new terrain obstacle type must be added to every spell collision system separately. | Extract `fn check_terrain_obstacle_hit(pos: Vec3, walls: ..., rocks: ..., trees: Option<...>) -> bool` into `spells/utils.rs` and call it from all three collision systems. |
| F10 | Security | `wall_of_fire/damage/core.rs:29-130,134-189` | High | M | `apply_wall_of_fire_damage` and `cleanup_wall_of_fire` have no `Without<GhostSpellEffect>` filter. On the guest, the host's walls are replicated as `WallOfFireEffect` with `GhostSpellEffect`. The zero-`damage_per_tick` mitigates numeric damage, but these systems still: (a) advance `time_alive` via local delta-time, racing CRDT-synced values; (b) write `TerrainDamageMessage` with `origin=Vec3::ZERO` (ghost walls have `start=ZERO, end=ZERO`); (c) insert `InsideWallOfFire` / `FirestormMarked` on local units at origin; (d) `cleanup_wall_of_fire` calls `try_despawn` on ghost walls when their local timer fires, potentially racing snapshot reconciliation and emitting a spurious `ObstacleChanged::Removed` event. | Add `Without<GhostSpellEffect>` to the query parameters of both `apply_wall_of_fire_damage` and `cleanup_wall_of_fire` (mirroring `apply_explosion_damage`/`cleanup_finished_explosions` in the fireball module). |
| F11 | ArchitecturalDecay | `fireball/projectile/explosion_lifecycle.rs:228-274` | Medium | S | `spawn_explosion_bubbles` has no `Without<GhostSpellEffect>` filter on its `spawners` query. On the guest, ghost fireball explosions have `GhostSpellEffect` and an `ExplosionBubbleSpawner`. When the ghost explosion grows, this system fires and spawns sub-bubble entities tagged `OnGameplayScreen` (line 270) — the wrong screen lifetime marker for MP. These bubble entities will not be cleaned up when the multiplayer game screen is torn down. | Add `Without<GhostSpellEffect>` to the `spawners` query, or propagate the correct screen-lifetime marker from the parent explosion entity (the existing `spawn_fireball_visuals` generic `M: Component + Clone` pattern shows how). |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|-------------------------|
| `magic_missile/missile.rs` | 434 | No | Two distinct concerns (movement vs. collision). Proposed split: `movement.rs` (`move_magic_missiles` + `despawn_distant_magic_missiles`) and `collision.rs` (`check_magic_missile_collisions` + `spawn_missile_detonation`). |
| `fireball/casting.rs` | 344 | Yes | Single cohesive casting+spawn pipeline: input handler, state machine, talent-modified spawn, raw entity spawn, visual spawn. All five functions share the same data context and are tightly coupled; no split is warranted. |
| `magic_missile/casting/cast.rs` | 324 | Yes | Three functions around a single casting interaction: param computation, casting system, arcane barrage tick. Cohesive; borderline but acceptable. |

---

### Looks bad but is actually fine

- **`Fireball` boolean talent fields** — looks like candidate ECS components, but they are set once at spawn and only ever read by a single collision system. Making them separate components would add query overhead with no queryability benefit (project convention: flags that don't drive their own system stay as fields).
- **`WallOfFireTalentParams` not deriving `Component`** — intentionally embedded as a field inside `WallOfFireEffect`, not a separate component. Correct per project convention.
- **`apply_wall_of_fire_damage` writing `TerrainDamageMessage` unconditionally** — damage is zero on ghost walls so terrain receives a zero-damage event. This is wasteful but harmless; the real bug is the `time_alive` advancement and the `cleanup` despawn (F10).
- **`update_napalm_trails` has no ghost filter** — napalm trail explosions are themselves `NetworkedSpellEffect { kind: NapalmTrail }` and replicated to the guest as ghost entities; the `apply_explosion_damage` system correctly filters `Without<GhostSpellEffect>`. No double-damage.
- **`handle_wall_of_fire_casting` at 294 lines** — a single complex system managing three tightly-coupled phases (drag-start, drag-update, release+place) that share local mutable state. Extraction would require threading many variables; the 294 lines are genuinely cohesive.
- **`ArcaneBarrage` duplicating `MissileParams` fields** — intentional: the barrage entity is self-contained and must not re-read `ActiveTalents` on each periodic fire. Snapshot of talent state at cast time.

---

### Open questions

1. Was the zero-`damage_per_tick` sentinel on ghost `WallOfFireEffect` intended as a deliberate mitigation (accepting the timer-advance and terrain-message side effects), or is it a leftover from before the `GhostSpellEffect` filter pattern was established? If the former, a comment explaining the tradeoff would prevent well-intentioned future "fixes" from re-introducing the bug.
2. Were ghost-explosion sub-bubbles (F11) intentionally omitted from the MP path (to save bandwidth / entities), or is this an oversight from the SP-only prototype?
3. Should `ScorchedEarthFire` (fireball) be renamed to avoid confusion with `ScorchedEarthZone` (wall of fire)? Both appear side-by-side in `dispel/bolt.rs` and the similar names cause comment drift there already.
