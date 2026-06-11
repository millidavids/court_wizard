## spell-meteor_fall

**Scope:** `src/game/units/wizard/spells/meteor_fall/`

---

### Mental model

Meteor Fall is a concentration spell (or fixed-duration Extinction Event) that rains falling meteors on a targeted area. The module follows a three-stage pipeline: (1) `MeteorFallStorm` marker entity periodically spawns `MeteorProjectile` entities, (2) projectiles fall under gravity and optional tracking force until they hit Y≤0 where they explode and leave `MeteorGroundFire` hazards, (3) ground fires tick periodic damage and fade out. All three stages are gated on `any_exist` run conditions in the plugin. Ghost (MP guest-side) separation is correct: explosion damage and ground-fire damage/cleanup are `Without<GhostSpellEffect>`; visual systems run on both real and ghost entities. Ghost meteors are bare `GhostSpellProjectile` entities without a `MeteorProjectile` component, so the physics/collision systems naturally skip them.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | TypeContract | casting.rs:320 | High | S | `ConcentrationSpell.mana_cost` is hardcoded to `MANA_COST` (50.0) instead of the talent-adjusted `effective_mana_cost`. The Meteor Shower talent halves the cast cost to 25.0, but the reservation field stays at 50.0, so the mana bar over-reserves 25 mana for the lifetime of the storm — reducing the wizard's effective mana pool by 25 more than intended. | Change `mana_cost: MANA_COST` to `mana_cost: effective_mana_cost` (already computed on line 240). |
| F2 | ArchitecturalDecay | meteor.rs:1–636 | Medium | M | `meteor.rs` is 636 LOC and hosts four distinct concerns: projectile physics + smoke trail (lines 33–147), collision/impact including Aftershock and Volcanic Eruption (148–352), explosion animation + damage (396–505), and ground fire lifecycle—particles, damage, fade, cleanup (507–636). The project rule requires splitting files >300 LOC unless they are a single match-on-enum or asset registry; this file is neither. | Split into `projectile.rs` (movement, smoke trail, collision/impact), `explosion.rs` (animate + damage systems), and `ground_fire.rs` (particles, damage, fade, cleanup). Update `systems.rs` re-exports and `mod.rs` declarations accordingly. |
| F3 | ConsistencyRot | meteor.rs:235,269,471,573 | Low | S | XZ-plane distance is computed four different ways in the same file: manual `(dx*dx+dz*dz).sqrt()` twice (Aftershock line 235, Volcanic Eruption line 269), `xz_distance()` utility once (explosion damage line 471), and `Vec3::new(x,0,z).length()` once (ground fire damage line 573). All four are semantically identical; having four representations is inconsistent and makes future refactors more error-prone. | Standardize to `xz_distance(a, b)` from `crate::game::units::wizard::spells::utils`. The utility is already imported in meteor.rs at line 19 via `crate::game::units::wizard::spells::utils`. |
| F4 | ArchitecturalDecay | casting.rs:1–532 | Low | M | `casting.rs` is 532 LOC and bundles four concerns: talent config computation (lines 32–134), local wizard casting input system (135–202), storm spawning logic (203–352), projectile factory `spawn_meteor_projectile_entity` (476–506), and explosion entity factory `spawn_explosion_entity` (508–532). The projectile/explosion factory functions belong more naturally alongside the entities they create than alongside casting input. | Extract `spawn_meteor_projectile_entity`, `MeteorProjectileTalentFlags`, and `spawn_explosion_entity` into `projectile.rs` (created as part of F2 split). This reduces `casting.rs` to ~350 LOC and places factory functions next to the projectile systems. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| meteor.rs | 636 | No | Four distinct concerns; split into `projectile.rs`, `explosion.rs`, `ground_fire.rs` |
| casting.rs | 532 | No | Factory functions (projectile/explosion entity spawners) should move to `projectile.rs` once created; remaining casting logic is ~350 LOC |

---

### Looks bad but is actually fine

- **`spawn_meteor_smoke_trail` iterates `projectiles` twice (lines 99–111 and 122–146).** The first pass mutates `has_glow` flag on each projectile; the second is an immutable read for smoke VFX. Rust forces the first loop to end before a shared borrow can start, so this is a valid two-pass pattern. No data races, and the overhead is negligible (typically <10 in-flight meteors).
- **`spawn_meteor_projectiles` and `check_meteor_collisions` don't filter out ghost entities.** Ghost meteors are spawned as bare `GhostSpellProjectile` entities without a `MeteorProjectile` component; `spawn_meteor_projectiles` queries `MeteorFallStorm` (host-only, never ghost-synced); `check_meteor_collisions` queries `MeteorProjectile`. Ghost meteors are therefore correctly excluded from both systems with no explicit `Without` filter needed.
- **`spawn_ground_fire_particles` and `fade_ground_fire` run on ghost `MeteorGroundFire` entities.** These are purely visual systems. Ghost fires have `time_alive=0` (only `apply_ground_fire_damage` ticks it, and that excludes ghosts), so `fade_ground_fire` will never scale them down (remaining = duration > GROUND_FIRE_FADE_DURATION always), and `spawn_ground_fire_particles` will emit particles during the full virtual duration — correct behaviour since the fire is still alive from the guest's perspective.
- **`systems.rs` is a re-export hub, not a systems module.** This pattern ("Phase 14") is used consistently across 6+ spells (fireball, entangle, chain_lightning, finger_of_death, black_hole). It predates the current style guide and is codified project-wide; not a per-module violation.
- **`ConcentrationSpell` is not spawned for Extinction Event** (line 312–324). The Extinction Event path uses `duration: Some(EXTINCTION_STORM_DURATION)` and self-despawns after a fixed time, bypassing the concentration system entirely. This is intentional design, not a missing component.
- **`meteor_fall_casting_logic` is a large private helper (lines 206–351).** Its 145 lines are a single linear state machine (Resting→Casting→complete). Splitting a single match on `CastingState` into smaller functions would add indirection without clarity benefit.

---

### Open questions

1. **Volcanic Eruption `break` on first fire zone hit (line 301):** The comment says "Only trigger eruption on the first matching fire zone." Is this a game design choice (one eruption per meteor) or an oversight? If the intent is to trigger eruptions on *all* overlapping fire zones, the `break` needs to be removed and the query order should be made deterministic (sorted by distance).
2. **Ghost ground fires emitting double particles:** On the MP guest, *both* a ghost `MeteorGroundFire` entity (spawned by `guest_visuals`) and the local fire effects from the snapshot would coexist. Are ghost ground fires intended to also emit VFX particles via `spawn_ground_fire_particles`, or does this cause visual doubling compared to singleplayer?
3. **`find_nearest_non_defender_xz` is `pub(super)` in `meteor.rs` but imported into `casting.rs` via a direct `use super::meteor::` path.** This works today, but if `meteor.rs` is split (F2), this helper needs a new home (`projectile.rs` or `utils.rs`).
