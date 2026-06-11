## spell-meteor_fall

**Scope:** `src/game/units/wizard/spells/meteor_fall/` — all `.rs` files (12 files, 1 644 LOC)

---

### Mental model

Meteor Fall is a concentration spell that creates a `MeteorFallStorm` marker entity at a
targeted position. Each frame the storm ticks its `time_since_spawn` timer; when it fires,
`spawn_meteor_projectiles` creates `MeteorProjectile` entities that fall under gravity and
(optionally) track enemies. On ground collision, `check_meteor_collisions` spawns a
`MeteorExplosion` (visual + one-shot AoE damage) and a `MeteorGroundFire` (persistent DoT
zone that also marks the pathfinding grid as costly terrain).

Three Tier-3 talents substantially change the spell: *Extinction Event* overrides the storm
with a fixed-duration self-destructing path that fires a single massive meteor; *Volcanic
Eruption* detonates nearby ground fires when a meteor lands; *Meteor Shower* triples spawn
rate at lower per-meteor power.

All talent parameters are computed once at cast time into `MeteorTalentConfig`, then copied
onto the `MeteorFallStorm` component, and again into each `MeteorProjectile` at spawn time
via `MeteorProjectileTalentFlags`. `MeteorExplosion` and `MeteorGroundFire` carry
`NetworkedSpellEffect` (ghost-replicated on the guest); the storm and projectiles are
host-authoritative only.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | ArchitecturalDecay | `meteor/projectile.rs:1-389` | Medium | M | File holds five unrelated concerns: projectile physics (`update_meteor_projectiles`), an enemy-search helper (`find_nearest_non_defender_xz`), smoke-trail VFX (`spawn_meteor_smoke_trail`), ground-collision/damage/pathfinding (`check_meteor_collisions`), and Extinction Event logic (`process_extinction_event`). At 389 LOC it exceeds the 300-line threshold and is not a single match/registry monolith. | Split into `movement.rs` (physics + find_nearest helper), `trail.rs` (smoke/glow VFX), and `collision.rs` (collision + pathfinding + extinction). |
| F02 | ArchitecturalDecay | `casting/input_casting.rs:1-343` | Low | S | File contains two distinct concerns: talent-config resolution (`MeteorTalentConfig` + `compute_meteor_talent_config`, ~125 LOC) and the casting state machine (`handle_meteor_fall_casting` + `meteor_fall_casting_logic`, ~218 LOC). | Extract `MeteorTalentConfig` and `compute_meteor_talent_config` to a sibling `casting/talents.rs`, leaving `input_casting.rs` purely for casting input. |
| F03 | ConsistencyRot | `meteor/projectile.rs:230,264` vs `meteor/explosion.rs:94` vs `meteor/ground_fire.rs:88` | Low | S | Three different idioms for XZ-plane distance within the same module: `(dx*dx + dz*dz).sqrt()` (projectile.rs:230, 264), `Vec3::new(..., 0.0, ...).length()` (ground_fire.rs:88), and the shared `xz_distance()` helper from `spells::utils` (explosion.rs:94). | Normalise all distance calculations to use the existing `xz_distance()` helper. |
| F04 | TypeContract | `casting/input_casting.rs:23` and `casting/projectile_spawn.rs:16` | Low | S | `MeteorTalentConfig` (private, casting) and `MeteorProjectileTalentFlags` (pub(crate)) are parallel structs with seven overlapping fields (`aftershock`, `volcanic_eruption`, `ground_fire_*_mult`, `tracking`, `is_extinction`). Talent values are manually transcribed config → storm → projectile at two call sites. | Unify to a single `MeteorTalentParams` type, or add a `From<&MeteorTalentConfig>` impl for `MeteorProjectileTalentFlags`, so field additions only require one edit. |
| F05 | ArchitecturalDecay | `casting/input_casting.rs:290-300` | Low | S | Ten manual field assignments from `MeteorTalentConfig` → `MeteorFallStorm`. If a new talent field is added to the config struct it must also be added in this copy block with no compile-time enforcement. | Add an `apply_talents(&mut self, cfg: &MeteorTalentConfig)` method to `MeteorFallStorm`, or resolve via the F04 type consolidation. |
| F06 | ArchitecturalDecay | `casting/projectile_spawn.rs:105-127` | Low | S | When tracking bias is active, a second `commands.entity(entity).insert(Transform {...})` overwrites the `Transform` from the spawn bundle. Both commands are deferred and apply correctly today, but the pattern is fragile: the spawn-bundle `Transform` is wasted and a future refactor could silently drop the override. | Compute the biased position before calling `spawn_meteor_projectile_entity` and pass the final position directly. |
| F07 | DocDrift | `systems.rs:1` | Low | S | Module-doc reads `"Re-export hub for meteor_fall systems split (Phase 14)."` — the internal refactor phase number is a stale artefact with no reader value. | Replace with a plain description: `"Re-export hub for meteor_fall sub-module symbols."` |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `meteor/projectile.rs` | 389 | false | Five separate concerns. Propose: `meteor/movement.rs`, `meteor/trail.rs`, `meteor/collision.rs` |
| `casting/input_casting.rs` | 343 | false | Two concerns (talent config + casting state machine). Propose: `casting/talents.rs` + slimmed `casting/input_casting.rs` |

---

### Looks bad but is actually fine

- **Missing `Without<GhostEntity>` on unit query in `check_meteor_collisions`** — Ghost entities have `GhostEntity` but no `Health` component. The unit query at line 161 requires `&mut Health`, so ghost entities are structurally excluded from the result set. No damage is accidentally applied to cosmetic ghost units.
- **`MeteorProjectile` and `MeteorFallStorm` lack `NetworkedSpellEffect`** — These are intentionally host-authoritative. Only `MeteorExplosion` and `MeteorGroundFire` are networked, which is the correct authority split.
- **`spawn_explosion_entity` is `pub(crate)` from `casting/mod.rs` but only used internally** — `meteor/projectile.rs` imports it from the `casting/` sub-module so the wider visibility is required by the intra-module import path.
- **`update_meteor_projectiles` tracking query includes ghost entities (which carry `Team`)** — Ghost unit positions are authoritative replicas of host positions; using them as tracking targets on the guest is functionally equivalent to targeting real units.
- **`t = pos.x * 0.01 + pos.z * 0.01` pseudo-time (projectile.rs:179)** — A deterministic per-position seed passed to particle VFX helpers that accept an elapsed-time argument for animation offset. Avoids requiring `Res<Time>` at that call site and prevents all impact particles spawning with identical animation phase.
- **`systems.rs` asymmetric visibility** — `pub(crate)` for `spawn_meteor_projectile_entity` (consumed by `arcane_crystal`) and `pub(super)` for everything else is the correct, minimal-exposure design.

---

### Open questions

1. `cleanup_ground_fire` (ground_fire.rs:121) resets pathfinding terrain cost to `1.0` at expiry. If two overlapping fire zones expire at different times, the first cleanup could reset cells still covered by the second fire. Is overlapping fire handled at the pathfinding layer?
2. The Aftershock damage loop (projectile.rs:219–257) iterates all units inside the outer `for ... in projectiles.iter()` loop. At high meteor density this is O(meteors × units). Worth profiling under Meteor Shower talent.
3. The Volcanic Eruption `break` (projectile.rs:299) means a meteor near multiple overlapping fire zones only erupts the first one found. Is single-eruption-per-impact intentional, or should all fires within radius erupt?
