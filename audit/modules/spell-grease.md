## spell-grease

**Scope:** `src/game/units/wizard/spells/grease/`

---

### Mental model

The Grease spell drops a persistent zone (GreaseZone) that slows enemies. It can be ignited by any of six fire sources (FireDoT units, FireballExplosion, WallOfFireEffect, MeteorGroundFire, DisintegrateBeam, or chain-combustion from an adjacent ignited zone), after which it transitions to a burn-damage state (GreaseIgnited). Three talent tiers add numeric modifiers (radius, slow, burn damage), behavioral overlays (Slip and Fall stun on entry, Oil Slick vulnerability debuff, Lingering Flames duration extension), and transformative modes (Chain Combustion cross-zone ignition, Grease Geyser airborne launch, Endless Oil post-fire regeneration). The module is feature-sliced into casting/ignite/components/constants, with a thin `systems.rs` re-export shim consumed by the plugin. The biggest structural issue is that the gameplay systems (slow application, ignition detection, burn application, zone cleanup, fade, VFX, regeneration) never filter out `GhostSpellEffect`, meaning on the guest peer in multiplayer every host-replicated grease zone would run all host-authoritative gameplay logic a second time.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| G-01 | Security / Multiplayer | `ignite.rs:35-252`, `ignite.rs:307-376`, `casting.rs:309-433`, `ignite.rs:436-470`, `ignite.rs:378-433`, `ignite.rs:473-501`, `ignite.rs:504-529` | High | M | All GreaseZone gameplay systems (`check_grease_ignition`, `apply_grease_burn`, `apply_grease_slow`, `cleanup_grease_zone`, `fade_grease_zone`, `update_grease_regeneration`, `cleanup_grease_debuffs`) query `GreaseZone` without `Without<GhostSpellEffect>`. On the guest client, the host's replicated GreaseZone entities carry both `GreaseZone` and `GhostSpellEffect`. Every one of these systems will run against ghost zones — causing double-application of slow, burn damage, ignition, talent stuns, Oil Slick vulnerability, and potentially incorrect cleanup of debuffs tied to zones the guest doesn't own. Other spells (plague_wind, arcane_crystal, meteor_fall, lightning_rod, dispel) all gate their zone queries with `Without<GhostSpellEffect>`. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the primary GreaseZone queries in all seven gameplay systems listed. `fade_grease_zone` and `update_grease_fire_spread` are visual-only and may intentionally run on ghosts — audit each individually for intent before gating. |
| G-02 | ArchitecturalDecay | `ignite.rs:1-784` | Medium | M | `ignite.rs` is 784 lines and mixes four distinct concerns: (1) fire ignition logic (`check_grease_ignition`), (2) fire VFX and smoke (`spawn_grease_fire_smoke`, `update_grease_fire_spread`), (3) zone lifecycle (`fade_grease_zone`, `cleanup_grease_zone`, `update_grease_regeneration`, `cleanup_grease_debuffs`), (4) zone spawn helper and VFX particles (`spawn_grease_zone`, `spawn_grease_zone_vfx`, `update_grease_bubbles`, `update_grease_splatters`). This violates the 300-line cohesion rule for non-match/non-registry files. | Split into `ignite.rs` (fire ignition logic only), `lifecycle.rs` (fade, cleanup, regeneration, debuff cleanup), `spawn.rs` (spawn_grease_zone constructor), `vfx.rs` (fume, bubble, splatter, smoke). Update `systems.rs` re-exports. |
| G-03 | ArchitecturalDecay | `casting.rs:309-433` | Medium | S | `apply_grease_slow` — including Slip and Fall stun application and Oil Slick vulnerability management — lives in `casting.rs`. This is a zone-effect system, not casting logic. It has no dependency on casting inputs and is conceptually part of lifecycle/slow application. Naming drift: `casting.rs` contains both input-driven casting and a passive tick system. | Move `apply_grease_slow` to a new `slow.rs` (or into `lifecycle.rs` when G-02 is addressed). |
| G-04 | DocDrift | `ignite.rs:33` | Low | S | The doc comment immediately above `check_grease_ignition` reads `/// Helper to write an obstacle event for a grease zone.` — this is the doc comment for `write_grease_obstacle` in `casting.rs`, copy-pasted and left on the wrong function. `check_grease_ignition` has no doc comment of its own. | Replace line 33's comment with a correct description of `check_grease_ignition`'s purpose (e.g., `/// Checks all active fire sources and ignites any overlapping non-ignited grease zones.`). |
| G-05 | Performance | `ignite.rs:378-432` | Low | S | `fade_grease_zone` calls `materials.get_mut(material_handle)` unconditionally every frame for every active grease zone, even during the non-fading period (when `remaining >= FADE_DURATION` and `fade == 1.0`). Bevy's `Assets::get_mut` marks the asset as changed on every call, forcing a GPU re-upload of the material every frame regardless of whether any value changed. The iridescent sheen does genuinely change every frame, but the alpha/base_color write during the non-fading window still dirtying the material is wasteful. | Guard the base_color write behind `if fade < 1.0`. For the iridescent sheen, accept the per-frame update (it's animated) but avoid the redundant alpha write when fade is exactly 1.0. |
| G-06 | ConsistencyRot | `components.rs:157-162`, `components.rs:173-177` | Low | S | `GreaseOilSlickDebuff::new()` and `GreaseRegenerating::new()` are zero-argument constructors whose body is trivially `Self { field: constant }` or `Self { field: 0.0 }`. Rust convention for zero-arg constructors is `#[derive(Default)]`. Other components in the codebase use `Default`. | Replace `fn new() -> Self` on both structs with `#[derive(Default)]`. Callers switch from `::new()` to `::default()` (or just `GreaseRegenerating {}` directly). |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|-------------------------|
| `ignite.rs` | 784 | false | Split into: `ignite.rs` (check_grease_ignition, update_grease_fire_spread), `lifecycle.rs` (fade_grease_zone, cleanup_grease_zone, update_grease_regeneration, cleanup_grease_debuffs), `spawn.rs` (spawn_grease_zone), `vfx.rs` (spawn_grease_fire_smoke, spawn_grease_zone_vfx, update_grease_bubbles, update_grease_splatters) |
| `casting.rs` | 433 | false | Split into: `casting.rs` (handle_grease_casting, grease_casting_logic, write_grease_obstacle, compute_talent_params), `slow.rs` (apply_grease_slow) |

---

### Looks bad but is actually fine

- **`systems.rs` as a pure re-export shim** — the 4-line file re-exporting from `casting` and `ignite` looks like an empty wrapper. This is the established pattern across ~10 other spells (dispel, entangle, healing_plume, fireball, magic_missile, etc.) so the plugin can consistently use `systems::` paths regardless of how many backing files exist.
- **`#[allow(clippy::too_many_arguments)]` on multiple functions** — `grease_casting_logic` has 20 parameters and `check_grease_ignition` has 12 injected query parameters. These are Bevy systems and spell helper functions with mapped struct fields; this is explicitly sanctioned by project conventions.
- **`spawn_grease_zone` called from both `casting.rs` and `guest_visuals.rs`** — the `pub(crate)` visibility on the spawn helper is intentional; multiplayer guest reconstruction calls it directly to replicate the host's zone, which is the correct sharing pattern.
- **`queue_silenced` closure for `SlowMovementModifier` insertion** (`casting.rs:368`) — this pattern is used in `units/systems.rs` for the same reason: inserting a new component on an entity already borrowed in a Query requires deferred world access. It is idiomatic for this codebase.
- **Ignition source queries using separate Query parameters** (`check_grease_ignition` lines 46-62) — six separate Query params for different fire-source component types looks verbose but avoids a union query that would either require enum-dispatch or unsafe. The alternative (one query with many `Option<>` fields) would be worse for performance and readability.
- **`GreaseTalentParams` as a plain struct, not a Component** — talent params are cast-time constants stamped into `GreaseZone`, not independently queried by any system. Keeping them as a field on `GreaseZone` rather than a separate component is correct per the project's component granularity rule.

---

### Open questions

1. Should `fade_grease_zone` and `update_grease_fire_spread` intentionally run on `GhostSpellEffect` zones on the guest (for visual correctness) while all damage/slow/debuff systems are gated out? If so, this should be explicitly documented in those functions.
2. The guest-side `GreaseZone` spawned in `guest_visuals.rs` (line 292) has `slow_modifier: 0.0`, `slow_duration: 0.0`, and all damage values zeroed — this means `apply_grease_slow` would produce no effect even without the `GhostSpellEffect` gate. Is this zeroing intentional as a belt-and-suspenders safety measure, or an implicit reliance on it that makes the missing ghost gate less dangerous than it appears?
3. `update_grease_regeneration` (`ignite.rs:473`) queries `zone_materials: Query<&MeshMaterial3d<StandardMaterial>, With<GreaseZone>>` inside an outer loop over `zones: Query<..., With<GreaseRegenerating>>` — these are separate queries on the same entity. Could this be unified into a single query to avoid the inner `zone_materials.get(entity)` call?
