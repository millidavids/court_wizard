## spell-squall

**Scope:** `src/game/units/wizard/spells/squall/`

---

### Mental Model

Squall is a concentration AoE spell that rains ice projectiles onto a targeted ground circle. It has three talent tiers: Tier-1 modifies damage/radius/spawn-rate numerically; Tier-2 adds Permafrost (stronger frost buildup), Hailstones (large high-damage shards), or Sleet Storm (evasion debuff inside the zone); Tier-3 converts the spell to a channeled Absolute Zero (stacking slow + mana drain), a moving Blizzard, or an Ice Age that leaves frozen-ground slow patches on every projectile impact.

The module is split into six files: `plugin.rs` (pure registration), `components.rs` (all component types), `constants.rs` (all numeric tuning), `casting.rs` (local-wizard input + storm spawning + shared helpers), `shards.rs` (projectile spawn/physics/collision, explosion update, talent systems, VFX), and `systems.rs` (a thin re-export hub forwarding all public symbols from `casting` + `shards` into the namespace `plugin.rs` imports via `use super::systems::*`).

Ghost-gating for multiplayer is applied correctly on all SquallStorm queries (always `Without<GhostSpellEffect>`), but the `IceExplosion` damage-path query in `update_ice_explosions` is unguarded — ghost IceExplosions on the guest peer will double-apply frost accumulation.

---

### Findings Table

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| SQ-01 | TypeContract | `components.rs:99,104,107,149,156` | Medium | S | Five fields on `IceProjectile` and `IceExplosion` are annotated `#[allow(dead_code)]`: `damage_type`, `radius`, `empowerment` (×2). If they are genuinely never read, they bloat every spawned component. If they are intended for future use or networking introspection, that intent is undocumented. | Audit actual reads. Remove unused fields or replace `#[allow(dead_code)]` with a brief `// used by: ...` comment naming the consumer. |
| SQ-02 | Performance | `shards.rs:529,542,549` | Medium | S | `ABSOLUTE_ZERO_SLOW_PER_FRAME` is added **once per frame** regardless of frame duration (no `* delta`). At 30 FPS the slow stacks at half the rate of 60 FPS. Other per-second quantities in the same function (`ABSOLUTE_ZERO_MANA_PER_SEC * delta`, `ABSOLUTE_ZERO_DPS * delta`) are correctly delta-scaled. | Rename to `ABSOLUTE_ZERO_SLOW_PER_SEC` and multiply by `delta` at the call site: `az.accumulated_slow -= ABSOLUTE_ZERO_SLOW_PER_SEC * delta`. Adjust the constant value to keep the same feel at 60 FPS (`0.005 * 60 = 0.3 /s`). |
| SQ-03 | ArchitecturalDecay | `shards.rs:1` | Medium | M | `shards.rs` is 868 lines — nearly 3× the project's 300-line threshold. It contains at least four independent concerns: projectile spawn + physics, explosion update + damage, talent effect systems (Sleet Storm, Absolute Zero, Blizzard, Ice Age), and VFX (snow particles). This is not a match/registry monolith; it is multiple systems bundled together. | Split into: `projectiles.rs` (spawn, physics, collision, `spawn_ice_explosion`), `explosions.rs` (`update_ice_explosions`), `talents.rs` (`apply_sleet_storm_evasion`, `update_absolute_zero`, `decay_absolute_zero_slow`, `end_absolute_zero_on_release`, `update_blizzard_position`, `update_frozen_ground`, `spawn_frozen_ground_patch`), `snow.rs` (`spawn_snow_particles`, `update_snow_particles`, `update_storm_ring`). |
| SQ-04 | Multiplayer | `shards.rs:273–432` | High | M | `update_ice_explosions` iterates ALL `IceExplosion` entities with no `Without<GhostSpellEffect>` guard on the `explosions` query. Ghost `IceExplosion` entities spawned on the co-op guest have `damage = 0.0`, so health damage is a no-op, but `apply_frost_accumulation` is called with the full `frost_per_hit` (0.3 or 0.6 with Permafrost). This double-applies frost status on the guest, causing incorrect slow/freeze buildup on enemies the guest never truly hit. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the `explosions` query in `update_ice_explosions`. Ghost explosions handle only visuals via their `Mesh3d`/`MeshMaterial3d` components; the gameplay damage path must be host-authoritative only. |
| SQ-05 | ArchitecturalDecay | `shards.rs:39–40` | Low | S | The doc-comment on `spawn_ice_projectiles` reads `"Applies or inserts a [`SlowMovementModifier`] on an entity."` — this was copied from the `apply_or_insert_slow` helper above it and is entirely wrong for a function that spawns ice projectiles. | Fix doc-comment to describe the actual function: `"Spawns ice projectiles into the storm area on a timed interval."` |
| SQ-06 | ArchitecturalDecay | `systems.rs:1` | Low | S | `systems.rs` is a pure re-export hub with the comment `"Phase 14"`. All other re-export hubs in the project say `"Phase 16"`. The phase number is stale development scaffolding that adds no value for readers. | Remove the phase annotation from the doc-comment or replace it with a one-line description: `"Re-export hub for squall systems."` |
| SQ-07 | DocDrift | `shards.rs:786–807` | Low | S | `let seed = time_secs * 7.1 + i as f32 * 1.618_034;` is computed but `phase` is the only derived value that uses it. The variable name `seed` implies it seeds the RNG, but the actual `rng.random_range` calls above it are independent of `seed`. This creates a misleading "seed" impression when `seed` is just a phase-offset. | Rename `seed` to `phase_seed` and add a brief comment: `// deterministic phase offset — does not seed the RNG`. |
| SQ-08 | Performance | `shards.rs:576–601` | Low | S | `decay_absolute_zero_slow` performs a full O(units × storms) double-loop: outer loop over all affected units, inner loop over all storms to check if each unit is inside a zone. With Absolute Zero there is at most one storm, so this is benign in practice, but the `has_active_az` guard only skips the inner loop when there are zero AZ storms — it still allocates the `storms` query result per unit when a storm exists. | Pre-collect the single active AZ storm's position and radius into an `Option<(Vec3, f32)>` before the unit loop, eliminating repeated query iteration. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `shards.rs` | 868 | No | Four independent concerns bundled together. Proposed split: `projectiles.rs` (spawn, physics, collision), `explosions.rs` (damage + visual update), `talents.rs` (Sleet Storm, Absolute Zero, Blizzard, Ice Age, Frozen Ground), `snow.rs` (snow VFX + storm ring). |

---

### Looks Bad But Is Actually Fine

- **`SpellSchool::Force` for an ice spell** (`casting.rs:184`): There is no Ice/Frost school in the `SpellSchool` enum. `Force` is the closest available color palette for the cast flare. This is a known enum-coverage gap, not a squall bug.
- **`IceExplosion::current_radius` called with `max(explosion.max_radius)`** (`shards.rs:385–387`): `explosion.current_radius(EXPLOSION_GROWTH_TIME).max(explosion.max_radius)` — during `damage_applied = false` the growth factor can be < 1.0, so this ensures full radius is used for hit-detection even if the visual hasn't expanded yet. Intentional.
- **`SquallTalentParams` not `#[derive(Component)]`** (`components.rs:10`): Embedded as a plain `Copy` struct inside `SquallStorm`; it doesn't need to be queried independently. Fine per project component design guidelines.
- **Nested unit loop inside `update_absolute_zero` with mutable borrow** (`shards.rs:512–563`): The outer loop is over storms (at most 1 for AZ), so this is effectively O(1 × units). No real performance concern.
- **`apply_or_insert_slow` and `apply_frost_accumulation` in `casting.rs`** (`casting.rs:26–56`): These helpers are private to the module (`pub(super)`) and called from both `casting.rs` and `shards.rs` — sharing is correct and they are not duplicated elsewhere in the codebase.
- **`mana.consume` called twice for non-AZ path** (`casting.rs:244–274`): Once at `try_start_cast_with_indicator` (subtracts `MANA_COST` to reserve) and once on cast completion. This is the standard two-phase pattern used by other spells; not double-charged.

---

### Open Questions

1. **Ghost `IceExplosion` terrain freeze**: Ghost explosions (damage=0.0) call `terrain_damage.write(TerrainDamageMessage { damage: 0.0, ... })` which will insert `PondFrozen { freeze_level: 0.0 }` on in-range ponds on the guest. The component gets inserted with zero effect but runs the terrain system unnecessarily. Should ghost explosions skip the terrain message entirely, or is this acceptable?
2. **`decay_absolute_zero_slow` without a storm**: This system runs whenever `any_exist::<AbsoluteZeroSlow>()` — correct. But if a unit with `AbsoluteZeroSlow` outlives the storm and there are no active storms, `has_active_az` is false, the `decay_timer` counts down, and the component is removed after `ABSOLUTE_ZERO_SLOW_DECAY_TIME`. Is this the intended post-zone decay behaviour, or should the slow be removed immediately on storm despawn?
3. **`SquallStorm` concentration-spell vs. channeled dual-mode**: `ConcentrationSpell` is conditionally inserted only on non-AZ storms. Is the AZ storm ever accidentally treated as a concentration spell elsewhere in the codebase (e.g. by generic concentration-end cleanup systems)?
