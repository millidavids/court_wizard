## spell-vfx

**Scope**: `src/game/units/wizard/spells/vfx/` — shared particle VFX library for all spells (fire, explosion, missile, plague, cast flares, motes, dust, aura bubbles, channeling).

---

### Mental model

This module is the central particle system for Court Wizard. It was originally a monolithic 1568-line `systems.rs` that was split in Phase 17 into concern-focused files. The current structure is:

- **`components.rs`** — 12 particle component structs (FireSmoke, FireSpark, MissileGlow, etc.)
- **`constants.rs`** — ~50 named tuning constants, well-organized by effect type
- **`fire_effects.rs`** — Fire glow tracking, smoke wisps, spark spawners/updaters
- **`explosion_effects.rs`** — Explosion smoke, missile glow/sparkles
- **`area_effects.rs`** — Heat shimmer, plague/fog cloud smoke, fire orange particles, embers, rising smoke
- **`cast_effects.rs`** — Cast flares, floating motes, smoke poofs, dust, aura bubbles, **and** all MP-synced wrapper functions + `emit_cast_event`
- **`channel.rs`** — Shared inward-implosion channeling VFX for non-wizard units (healer, dispeller, etc.)
- **`systems.rs`** — Pure re-export hub (`pub use` from the four effect files)
- **`fire_material.rs`** — `FireParticleMaterial` and `SmokeParticleMaterial` GPU shader types
- **`plugin.rs`** — Plugin registration + two private time-uniform update functions

All Update systems are gated behind either `any_exist::<T>()` per-component conditions or the outer `is_spell_effects_active` guard. No `.unwrap()` calls exist. The MP sync wrappers in `cast_effects.rs` follow a clean pattern: call bare local spawn, then push a `CastEventSnapshot`. The split is clean overall but a few architectural issues remain.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `plugin.rs:62-80` | Medium | S | `plugin.rs` defines two system body functions (`update_fire_particle_time`, `update_aura_sphere_time`) instead of keeping registration-only. CLAUDE.md is explicit: "plugin.rs does Bevy plugin registration ONLY. Move system bodies and helpers to sibling files." | Move both functions to a new `time_uniforms.rs` (or into `fire_material.rs` / `cast_effects.rs` respectively), and reference them by path from plugin.rs. |
| F2 | ArchitecturalDecay | `area_effects.rs:251-315` | Medium | S | `spawn_fire_smoke_puff` and `spawn_fire_particle_puff` are near-identical helpers (same signature minus the `material` arg, same spawn bundle, same time-offset logic). Both are `pub(super)` so neither is dead, but this is duplication between 2 private helpers in the same file. | Merge into one function taking `material: Handle<StandardMaterial>` as a parameter. `spawn_fire_particle_puff` becomes a one-liner: `spawn_fire_smoke_puff(commands, assets, assets.fire_particle.clone(), ...)`. |
| F3 | ArchitecturalDecay | `fire_effects.rs:11-31` / `explosion_effects.rs:110-130` | Low | M | `update_fire_glow` and `update_missile_glow` are structurally identical (track source position, pulse scale with different constants). Same pattern for `cleanup_orphaned_glows` / `cleanup_orphaned_missile_glows`. Four near-duplicate functions totalling ~60 lines. | Introduce generic helpers `update_tracking_glow::<G>` and `cleanup_orphaned::<G>` parameterized by a trait carrying the glow multiplier/frequency/amplitude constants. Reduces 4 functions to 2 shared + 2 small call sites. |
| F4 | TypeContract | `components.rs:83-86` / `area_effects.rs:612-613` | Medium | S | `PlagueSmoke.spawn_y` is used as a boolean-disguised-as-float: `spawn_y > 0.0` means "this is a fire puff, not a plague/fog puff". The comment at `components.rs:83` confirms this is a sentinel. This is a type contract violation — a fire puff spawned at world y=0 (legitimate terrain scenario) would be silently misclassified as non-fire, losing all height-based billowing behaviour. | Add an `is_fire: bool` field to `PlagueSmoke`, or introduce a zero-cost `FireSmokePuff` marker component so `update_plague_smoke` can use `Has<FireSmokePuff>` instead of the sentinel check. |
| F5 | ArchitecturalDecay | `cast_effects.rs:474-691` | Low | M | `cast_effects.rs` has grown to 690 lines by absorbing the entire MP-sync wrapper layer (`emit_cast_event`, `spawn_school_flare_synced`, `spawn_aura_bubble_synced`, `spawn_smoke_poof_synced`, `spawn_floating_motes_synced`, `spawn_sparks_with_material_synced`, `spawn_dust_smoke_synced`, `emit_banishment_lens_event`). These ~220 lines of MP-sync glue are a distinct concern from local cast-effect VFX. The file exceeds 300 LOC. | Split into `cast_effects.rs` (local spawn logic + updaters) and `mp_sync.rs` (all `*_synced` wrappers + `emit_cast_event`). Matches CLAUDE.md "group by concern" rule. |
| F6 | ConsistencyRot | `fire_effects.rs:194` / `explosion_effects.rs:105,176` / `area_effects.rs:66,242,277,311,401,536` / `cast_effects.rs:37,66,210,293,429` | High | M | Most VFX spawn helpers hardcode `OnGameplayScreen`. Only `spawn_fire_glow` (`fire_effects.rs:111`) takes a generic `screen_marker: M` to handle the MP ghost path — its own doc comment explains the risk of not doing this. All other spawners (`spawn_smoke_wisps_with_material`, `spawn_missile_glow`, `spawn_fire_smoke_puff`, `spawn_fire_sparks`, etc.) hardcode `OnGameplayScreen`. If a ghost spell effect calls any of these, spawned particles carry `OnGameplayScreen` and survive `cleanup_mp_game`. | Audit each caller in the multiplayer code paths. For any helper reachable from ghost contexts, apply the same `screen_marker: M` pattern. For SP-only helpers, add a `// SP-only callsite` comment to prevent silent regression during future MP work. |
| F7 | TypeContract | `cast_effects.rs:73-82` | High | S | `SpellSchool` lacks `#[repr(u8)]`. `spawn_school_flare_synced` manually maps each variant to a `u8` ordinal (0-7), and `snapshot.rs:690` has a separate `SpellSchoolWire` with a comment "keep ordinals in sync". If a variant is inserted between existing ones on either side, the wire encoding silently mismatches in live MP games. | Add `#[repr(u8)]` to `SpellSchool` with explicit discriminants (`Fire = 0`, …, `Transmutation = 7`), replace the manual match with `school as u8`. Eliminates the dual-enum synchronization risk. |
| F8 | Performance | `plugin.rs:62-67` | Low | S | `update_fire_particle_time` iterates all `FireParticleMaterial` assets every frame (under `is_spell_effects_active`) even when no fire particles are alive. `update_aura_sphere_time` does the same for aura sphere materials. These are constant cost regardless of whether any relevant entities exist. | Guard each with `any_exist::<FireOrangeSmokePuff>()` / `any_exist::<AuraBubbleVfx>()`, or consolidate to a single time-only uniform update gated appropriately. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|-------------------------|
| `area_effects.rs` | 665 | No | Contains 5+ distinct spawn families (heat shimmer, plague cloud, fire orange smoke, fire embers, rising smoke) plus their updaters. Propose: `heat_shimmer.rs`, `plague_smoke.rs`, `fire_smoke.rs` (orange/rising/black), `embers.rs`. |
| `cast_effects.rs` | 690 | No | Local VFX spawn logic mixed with MP-sync wrapper layer. Propose: `cast_effects.rs` (local spawners + updaters) and `mp_sync.rs` (all `*_synced` wrappers + `emit_cast_event`). |
| `explosion_effects.rs` | 209 | Yes | Under 300 LOC; coherent set of explosion-phase VFX. |
| `fire_effects.rs` | 256 | Yes | Just under 300 LOC; all fire glow/smoke/spark for the fire projectile family. Cohesive. |
| `constants.rs` | 236 | Yes | Pure constants, well-sectioned by comment blocks. CLAUDE.md exempts cohesive constant files. |
| `components.rs` | 179 | Yes | All VFX particle components. Cohesive registry. |

---

### Looks bad but is actually fine

- **`systems.rs` as a pure `pub use` hub**: Exists to preserve backwards-compatible import paths after Phase 17 split. All callers use `vfx::systems::spawn_*`; the re-export hub makes the split transparent. Intentional and clean.
- **12 golden-ratio seed lines across files** (`let seed = i as f32 * 1.618_034 + time_secs * k`): Each call site uses a different `k` scramble multiplier to produce visually distinct distributions across particle types. Extracting to a helper would need that parameter, giving no meaningful simplification.
- **`pub(super)` on `spawn_fire_smoke_puff` and `spawn_fire_particle_puff`**: Only called from sibling files via `pub use super::area_effects::*` in `systems.rs`. Visibility is intentionally limited.
- **Large `spawn_fire_orange_smoke` (area_effects.rs:416-499)**: Looks large but is a single coherent spawner doing three coordinated particle bursts (fire particles + embers + rising smoke). Not a god function.
- **`SpellSchool` defined in `cast_effects.rs`** rather than a dedicated `types.rs`: Only consumed by `spawn_school_flare` and `spawn_school_flare_synced` in that same file. Correct co-location.
- **`channel.rs` inside `spells/vfx/`**: The module is exclusively consumed by non-wizard units (healer, dispeller, shielder, teleporter). Placement in `vfx/` is defensible as a shared VFX primitive; `pub(in crate::game)` visibility and `units/plugin.rs` registration make the dependency explicit.
- **Double `run_if` nesting in `plugin.rs`**: Per-system `any_exist::<T>()` guards under an outer `is_spell_effects_active` look redundant but serve different roles — outer guard prevents any system from waking at all during menu/loading, inner guards skip per-system overhead when that component type has zero entities.

---

### Open questions

1. **F6 ghost-path coverage**: Are there MP scenarios where `spawn_missile_glow`, `spawn_explosion_smoke`, `spawn_plague_smoke_puffs`, or `spawn_floating_motes` are called from a ghost spell effect context? `spawn_fire_glow` was explicitly made generic for this reason. Should each non-generic spawner carry a `// SP-only callsite` annotation to lock in the current assumption?
2. **F4 y=0 edge case**: Can a fire puff legitimately spawn at world-space y=0 (e.g., a unit dying at ground level)? If so, the `spawn_y > 0.0` sentinel already misfires silently today.
3. **`update_aura_sphere_time` ownership**: `AuraSphereMaterial` is defined in `visual_assets` (outside the vfx scope) yet its time-uniform update lives in `vfx/plugin.rs`. Is this the intended long-term home, or should it move to the parent `spells/plugin.rs` alongside the `MaterialPlugin::<AuraSphereMaterial>` registration?
