## boss-ray

Scope: `src/game/units/boss/ray/` — the Ray boss (beholder-type), its five beam eyes, lifecycle, movement, particles, and per-beam sweep systems.

---

### Mental model

Ray is a beholder-style boss with a floating body and five independent eyes (Petrification, Disintegration, Fear, MindControl, Teleportation). Each eye has its own sweep component on the body entity that drives a cooldown→channel→fire loop. Damage dealt to the body is redistributed to the eyes; victory is triggered when all eyes reach zero HP. The module is well-sliced into concern-focused files below 300 LOC, with shared targeting helpers extracted into `beams/disintegration.rs` and `beams/beam_helpers.rs`. The primary technical debts are: a duplicated beam-steering pattern replicated across three files, a missing `Without<GhostEntity>` guard on every SP gameplay system, an inconsistent bare `.despawn()` call alongside `try_despawn`, and structural duplication between `RayPetrificationBeam` and `RayMindControlBeam` components.

---

### Findings table

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| R01 | ArchitecturalDecay | movement/petrification.rs:140–152, beams/mind_control_beam.rs:151–162 | Medium | M | Beam-steering while-channeling logic is copy-pasted verbatim across two files. Both clamp dot, compute angle, apply `max_turn / angle` lerp, and normalize. The disintegration sweep (movement/disintegration.rs:182–199) uses a slightly different 2D rotation variant but addresses the same concern. | Extract `steer_beam_toward(current: Vec3, desired: Vec3, turn_rate: f32, delta: f32) -> Vec3` into `beams/beam_helpers.rs` and call it from all three sites. |
| R02 | ArchitecturalDecay | movement/petrification.rs:68–79, movement/disintegration.rs:92–103, beams/mind_control_beam.rs:75–86 | Medium | S | Three identical `has_targets` range-check closures: iterate defenders, re-run `team_query.get(entity)` to filter non-Defenders, then check horizontal distance ≤ MAX_BEAM_RANGE. | Extract `has_defenders_in_range(boss_pos, defenders, team_query, max_range) -> bool` into `beams/beam_helpers.rs`. |
| R03 | TypeContract | components.rs:116–168 | Low | S | `RayPetrificationBeam` and `RayMindControlBeam` are structurally identical (origin, direction, length, channel_progress, has_fired). Two separate types force duplicated visual systems that are nearly identical except for beam width constants. | Consider a single generic `RayChannelBeam { origin, direction, length, channel_progress, has_fired }` component shared by both. The separate glow/sweep components already differentiate them for system dispatch. |
| R04 | ErrorObservability | movement/disintegration.rs:42 | Low | S | `commands.entity(entity).despawn()` is used for `RayBeamVisual` lifetime expiry while every other despawn in the module uses `try_despawn()`. A double-despawn from two racing systems would panic. | Change to `try_despawn()` for consistency and safety. |
| R05 | ArchitecturalDecay | movement/eye_movement.rs:59–62 | Low | S | Fear eye orbit angle is stored in `eye.heading.x` (repurposed as a float angle accumulator) rather than a dedicated field. A reader unfamiliar with the special-case path would expect `heading` to be a 2D unit direction. | Add an `orbit_angle: f32` field to `RayEye`, use it only for Fear eyes, and remove the dual-use of `heading.x`. |
| R06 | Performance | beams/teleportation.rs:140–148 | Medium | S | A fresh `StandardMaterial` is allocated via `materials.add(...)` every time the teleport eye fires (up to once every 15 s but still a heap allocation in a gameplay system). The bubble material parameters are fixed. | Pre-allocate the bubble material in `preload_ray_assets` (resources.rs), store it in `RayAssets`, and reuse the handle. |
| R07 | Performance | movement/particles.rs:103–107 | Low | S | `update_ray_stalk_particles` accesses `game_rng` (a `ResMut`) every frame for each particle to produce random shudder. This forces the entire particle tick to be single-threaded. | Pre-bake shudder seed into `RayStalkParticle` at spawn time, or derive pseudo-noise from `time.elapsed_secs()` + entity index. |
| R08 | ConsistencyRot | beams/mind_control_beam.rs:19–21 | Low | S | The doc-comment on line 19 (`/// Attenuated volume for Ray's sound effects...`) was copy-pasted from `ray_sfx_volume` in spawn.rs and left on the `ray_mind_control_beam` function signature. | Remove the stale comment from `ray_mind_control_beam`. |
| R09 | DocDrift | movement/eye_movement.rs:86 | Low | S | `to_body` is computed as `Vec2::new(body_pos.x - my_pos.x, body_pos.z - my_pos.y)` — `.y` on `my_pos` (a Vec2) refers to world-Z, not world-Y. This is correct by convention but reads misleadingly against the `.z` subscript used elsewhere. | Rename the intermediate to `my_xz` to match `body_xz` on line 63. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| movement/disintegration.rs | 256 | true | Single-concern: disintegration sweep system + shared helpers; all lines cohesive. |
| beams/mind_control_beam.rs | 254 | true | Single-concern: mind control sweep + visual update; under 300 LOC. |
| movement/petrification.rs | 228 | true | Single-concern: petrification sweep + visual update; under 300 LOC. |
| components.rs | 220 | true | All component definitions for one module; genuinely cohesive registry. |
| movement/spawn.rs | 216 | true | Spawn + movement tightly coupled by shared component set; under 300 LOC. |

No file exceeds 300 LOC.

---

### Looks bad but is actually fine

- **`pub use super::movement::*` and `pub(crate) use super::beams::*` in systems.rs** — looks like a wildcard anti-pattern, but `systems.rs` is explicitly a "re-export hub" (documented Phase 15 split). Intentional.
- **`unwrap_or(boss_pos)` for eye_pos lookups** — appears lossy, but the boss entity is always present when these systems run (guarded by `any_with_component::<Ray>`). Eye-not-found falling back to boss position is graceful degradation, not a hidden panic.
- **`RayEyeState` without `#[derive(Default)]`** — uses a hand-written `new()` returning all-true; the default state is intentionally all eyes alive, which is not what `Default` would express.
- **`partial_cmp().unwrap_or(Ordering::Equal)` in teleportation.rs:122** — standard Rust idiom for f32 sort; the fallback is safe because NaN distances would mean the unit is at the same position as Ray.
- **Separate `despawn_fear_beam`, `despawn_mind_control_beam`, `despawn_ray_beam`, `despawn_petrify_beam`** — look like duplication but each sweep type is structurally distinct (disintegration adds `sfx_entity`); a shared trait abstraction would add more complexity than it removes.
- **Multiple `#[allow(clippy::too_many_arguments)]`** — all on Bevy systems; expected per project conventions.

---

### Open questions

1. Is Ray intended to appear in co-op multiplayer? No `Without<GhostEntity>` guard exists on any Ray system. If the boss can appear in a co-op session, gameplay systems (body damage redistribution, petrification apply, mind control apply, teleportation scatter) would incorrectly run on ghost-synced units on the guest side.
2. The `has_targets` check filters `team_query.get(entity) != Team::Defenders` but the query already uses `With<Team>`. Could the team secondary lookup be eliminated by a tighter primary query filter?
3. `PETRIFY_DURATION: f32 = f32::MAX` and `MIND_CONTROL_DURATION: f32 = f32::MAX` — are these truly permanent? `ray_death_cleanup` does not remove `Petrified` or `MindControlled` from defender entities on Ray's death, so those debuffs would persist indefinitely into post-encounter gameplay.
