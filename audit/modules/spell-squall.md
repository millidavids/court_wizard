## spell-squall

**Scope:** `src/game/units/wizard/spells/squall/`

---

### Mental Model

Squall is a concentration AoE spell that rains ice projectiles onto a targeted ground circle. It has three talent tiers: Tier-1 modifies damage/radius/spawn-rate numerically; Tier-2 adds Permafrost (stronger frost buildup), Hailstones (large high-damage shards), or Sleet Storm (evasion debuff inside the zone); Tier-3 converts the spell to a channeled Absolute Zero (stacking slow + mana drain), a moving Blizzard, or an Ice Age that leaves frozen-ground slow patches on every projectile impact.

The module is split into six files: `plugin.rs` (pure registration), `components.rs` (all component types), `constants.rs` (all numeric tuning), `casting.rs` (local-wizard input + storm spawning + shared helpers), `shards.rs` (projectile spawn/physics/collision, explosion update, talent systems, VFX), and `systems.rs` (a thin re-export hub forwarding all public symbols from `casting` + `shards` into the namespace `plugin.rs` imports via `use super::systems::*`).

Ghost-gating for multiplayer is applied correctly on all SquallStorm queries (always `Without<GhostSpellEffect>`), but the `IceExplosion` damage-path query in `update_ice_explosions` is unguarded — ghost IceExplosions on the guest peer will double-apply frost accumulation. Additionally, `ABSOLUTE_ZERO_SLOW_PER_FRAME` is applied once per frame rather than scaled by delta time, making the Absolute Zero slow stack at different rates depending on framerate.

---

### Findings Table

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| SQ-01 | Multiplayer | `shards.rs:272–436` | High | M | `update_ice_explosions` iterates ALL `IceExplosion` entities with no `Without<GhostSpellEffect>` guard on the `explosions` query. Ghost `IceExplosion` entities spawned on the co-op guest have `damage = 0.0`, so health damage is a no-op, but `apply_frost_accumulation` is called with the full `frost_per_hit` (0.3 or 0.6 with Permafrost). This double-applies frost status on the guest, causing incorrect slow/freeze buildup on enemies. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the `explosions` query in `update_ice_explosions`. Ghost explosions handle only visuals via their mesh components; the gameplay damage path must be host-authoritative only. |
| SQ-02 | Performance | `shards.rs:529,542,549` | Medium | S | `ABSOLUTE_ZERO_SLOW_PER_FRAME` is added **once per frame** regardless of frame duration (no `* delta`). At 30 FPS the slow stacks at half the rate of 60 FPS. Other per-second quantities in the same function (`ABSOLUTE_ZERO_MANA_PER_SEC * delta`, `ABSOLUTE_ZERO_DPS * delta`) are correctly delta-scaled. | Rename to `ABSOLUTE_ZERO_SLOW_PER_SEC` and multiply by `delta` at the call site: `az.accumulated_slow -= ABSOLUTE_ZERO_SLOW_PER_SEC * delta`. Adjust the constant value to keep the same feel at 60 FPS (`0.005 * 60 = 0.3/s`). |
| SQ-03 | ArchitecturalDecay | `shards.rs:1` | Medium | M | `shards.rs` is 871 lines — nearly 3× the project's 300-line threshold. It contains at least four independent concerns: projectile spawn/physics/collision (~245 LOC), explosion update + damage (~165 LOC), talent gameplay systems (Sleet Storm, Absolute Zero, Blizzard, Ice Age / ~300 LOC), and snow VFX (~100 LOC). This is not a match/registry monolith. | Split into: `projectile.rs` (spawn, physics, collision, `spawn_ice_explosion`), `explosion.rs` (`update_ice_explosions`), `talents.rs` (`apply_sleet_storm_evasion`, `update_absolute_zero`, `decay_absolute_zero_slow`, `end_absolute_zero_on_release`, `update_blizzard_position`, `update_frozen_ground`, `spawn_frozen_ground_patch`), `snow.rs` (`spawn_snow_particles`, `update_snow_particles`, `update_storm_ring`). |
| SQ-04 | ArchitecturalDecay | `shards.rs:158,190,230` | Medium | S | The `sfx_scale` computation + `audio::play_impact_sfx_scaled` call is copy-pasted identically in all three collision branches (wall, rock, ground) inside `check_ice_projectile_collisions`. Three identical blocks within one function. | Extract a private `fn play_squall_impact_sfx(commands, sfx, game_config, pos, is_hailstone)` helper called from all three branches. |
| SQ-05 | TypeContract | `components.rs:92,100` | Low | S | `IceProjectile` has `#[allow(dead_code)]` at struct level and on the `damage_type` field. The `damage_type` is always `DamageType::Frost` (hardcoded in the constructor) and never read by any system. Dead field silently bloats every spawned entity. | Remove `damage_type` from `IceProjectile` and hard-code `DamageType::Frost` directly where `IceExplosion` is spawned. Drop both `#[allow(dead_code)]` suppressions. |
| SQ-06 | DocDrift | `shards.rs:38–40` | Low | S | The doc-comment on `spawn_ice_projectiles` reads `"Applies or inserts a [SlowMovementModifier] on an entity."` — verbatim copy-paste from the `apply_or_insert_slow` helper above it. The function actually spawns ice projectiles. | Fix doc-comment to: `"Spawns ice projectiles into the storm area on a timed interval."` |
| SQ-07 | Consistency | `shards.rs:562` | Low | S | `FROST_PER_HIT * delta * 5.0` uses an unexplained magic multiplier `5.0` for the Absolute Zero continuous frost accumulation rate. Its intended meaning (e.g. "equivalent to 5 hits/sec") is opaque. | Extract to `ABSOLUTE_ZERO_FROST_RATE_MULT: f32 = 5.0` in `constants.rs` with a doc comment explaining the equivalence. |
| SQ-08 | DocDrift | `systems.rs:1` | Low | S | `systems.rs` carries the comment `"Re-export hub for squall systems split (Phase 14)."` — the "Phase 14" label is stale development scaffolding with no value for readers. | Replace with `"Re-export hub for squall systems."` |
| SQ-09 | ArchitecturalDecay | `shards.rs:337` | Low | S | `use rand::Rng;` is re-imported inside the body of `update_ice_explosions` at line 337, even though it is already present at the top-level imports (line 5). The inner `use` is redundant and suggests the top-level import was added later without removing the scoped one. | Remove the inner `use rand::Rng;` at line 337. |
| SQ-10 | ArchitecturalDecay | `shards.rs:781` | Low | S | `let interval = SNOW_SPAWN_INTERVAL;` is a trivial rename of the constant that is used exactly once; it adds an indirection layer with no value. | Use `SNOW_SPAWN_INTERVAL` directly in the subsequent division expressions. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `shards.rs` | 871 | No | Four independent concerns bundled together. Proposed split: `projectile.rs` (spawn, physics, collision), `explosion.rs` (damage + visual update), `talents.rs` (Sleet Storm, Absolute Zero, Blizzard, Ice Age, Frozen Ground), `snow.rs` (snow VFX + storm ring). |

---

### Looks Bad But Is Actually Fine

- **`SpellSchool::Force` for an ice spell** (`casting.rs:184`): No Ice/Frost school exists in the `SpellSchool` enum. `Force` is the closest available color palette for the cast flare. Known gap, not a squall bug.
- **`IceExplosion::current_radius` called with `max(explosion.max_radius)`** (`shards.rs:385–387`): `explosion.current_radius(EXPLOSION_GROWTH_TIME).max(explosion.max_radius)` — during `damage_applied = false` the growth factor can be < 1.0, so this ensures full radius is used for hit-detection even before the visual has expanded. Intentional.
- **`SquallTalentParams` not `#[derive(Component)]`** (`components.rs:10`): Embedded as a plain `Copy` struct inside `SquallStorm`; it doesn't need to be queried independently. Fine per project component design guidelines.
- **`apply_or_insert_slow` and `apply_frost_accumulation` in `casting.rs`** (`casting.rs:26–56`): These helpers are `pub(super)` and shared between `casting.rs` and `shards.rs` within the module. While other spells duplicate the pattern locally, these helpers are not exported to the rest of the codebase, so the coupling is module-local and controlled. Extracting to `spells/utils.rs` would be a cross-cutting improvement but is not a squall-internal violation.
- **`seed` variable in snow particles** (`shards.rs:789`): Despite its name, `seed` is used only as a deterministic phase-offset for snow sway animation (line 810), not to seed the RNG. The RNG calls above it are independent. The naming is misleading but the math is correct.
- **Nested unit loop inside `update_absolute_zero`** (`shards.rs:512–563`): The outer loop is over storms (at most 1 for AZ), so this is effectively O(1 × units). No real performance concern.
- **`update_frozen_ground` lacks `Without<GhostSpellEffect>`** (`shards.rs:733`): `FrozenGround` is not tagged with `NetworkedSpellEffect` and is not a ghost entity — it is spawned exclusively on the authoritative peer. The query is correct.

---

### Open Questions

1. **Ghost `IceExplosion` terrain damage**: Ghost explosions (damage=0.0) still call `terrain_damage.write(TerrainDamageMessage { damage: 0.0, ... })` which runs the terrain system on the guest with a no-op. Should ghost explosions skip the terrain message entirely?
2. **`decay_absolute_zero_slow` post-storm behaviour**: When a unit with `AbsoluteZeroSlow` outlives the storm, the `decay_timer` counts down and the component is removed after `ABSOLUTE_ZERO_SLOW_DECAY_TIME`. Is this the intended post-zone decay behaviour, or should the slow be removed immediately on storm despawn?
3. **`SquallStorm` concentration-spell vs. channeled dual-mode**: `ConcentrationSpell` is conditionally inserted only on non-AZ storms. Is the AZ storm ever accidentally treated as a concentration spell elsewhere in the codebase (e.g. by generic concentration-end cleanup systems)?
4. **`IceProjectile` entities in MP**: `IceProjectile` entities are spawned host-only (correctly gated via `spawn_ice_projectiles`), but `update_ice_projectiles` and `check_ice_projectile_collisions` query all `IceProjectile` with no ghost guard. Since `IceProjectile` is never tagged `NetworkedSpellEffect` and is never ghosted to the guest, is this safe to leave ungated, or should explicit guards be added for defensive correctness?
