## spell-vfx

**Scope**: `src/game/units/wizard/spells/vfx/` — shared particle VFX library for all spells (fire, explosion, missile, plague, cast flares, motes, dust, aura bubbles, channeling).

---

### Mental model

This module is the central particle system for Court Wizard. It was originally a monolithic 1568-line `systems.rs` that was split in Phase 17 into concern-focused files. The current structure is:

- **`components.rs`** — 12 particle component structs (FireSmoke, FireSpark, MissileGlow, etc.)
- **`constants.rs`** — ~50 named tuning constants, well-organized by effect type
- **`fire_effects.rs`** — Fire glow tracking, smoke wisps, spark spawners/updaters
- **`explosion_effects.rs`** — Explosion smoke, missile glow/sparkles
- **`area_effects/`** — Sub-module: heat shimmer, plague/fog cloud smoke, fire orange particles, embers, rising smoke
- **`cast_effects/`** — Sub-module: `cast_vfx.rs` (cast flares, motes, poofs, dust, aura bubbles) + `mp_sync.rs` (all MP-synced wrappers)
- **`channel.rs`** — Shared inward-implosion channeling VFX for non-wizard units (healer, dispeller, etc.)
- **`systems.rs`** — Pure re-export hub (`pub use` from the four effect files + two time-uniform update systems)
- **`fire_material.rs`** — `FireParticleMaterial` and `SmokeParticleMaterial` GPU shader types
- **`plugin.rs`** — Plugin registration only (clean)

All Update systems are gated behind either `any_exist::<T>()` per-component conditions or the outer `is_spell_effects_active` guard. No `.unwrap()` calls exist. The Phase 17 split was largely successful; the area_effects sub-split is good. Key remaining issues are: (a) `cast_effects/cast_vfx.rs` at 468 LOC mixes four unrelated concerns; (b) the `SpellSchool` → `u8` encode in `mp_sync.rs` is maintained separately from the canonical `SpellSchoolWire` enum, a silent wire-format risk; (c) two spawn helper functions (`spawn_fire_smoke_puff` and `spawn_fire_particle_puff`) are near-identical and should be merged; (d) two inline gravity literals are absent from `constants.rs`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `cast_effects/cast_vfx.rs:1` (468 LOC) | High | M | `cast_vfx.rs` mixes four distinct VFX concerns: cast flares + `SpellSchool` enum (lines 1–127), floating motes (160–249), smoke poofs (252–325), dust smoke (327–375), and aura bubbles (377–468). Exceeds the 300-LOC ceiling and violates the "group by concern" rule. | Split into `flares.rs` (cast flare + SpellSchool enum), `motes.rs` (floating motes), `poofs.rs` (smoke poof + dust smoke), and `aura_bubbles.rs`. Update `cast_effects/mod.rs` re-exports. |
| F2 | TypeContract | `cast_effects/mp_sync.rs:57-65` | High | S | `spawn_school_flare_synced` encodes `SpellSchool` as raw `u8` literals (`Fire => 0`, …`Transmutation => 7`). There is no compile-time link to `SpellSchoolWire` in `snapshot.rs:690`. Adding a new variant to either enum without mirroring the other silently routes the wrong flare color to the remote peer. | Add `#[repr(u8)]` with explicit discriminants to `SpellSchool`, replace the manual match with `school as u8`. Or unify `SpellSchool` and `SpellSchoolWire` into one type. |
| F3 | TypeContract | `components.rs:83-86` | Medium | S | `PlagueSmoke.spawn_y` doubles as a boolean sentinel: `spawn_y > 0.0` means "is a fire puff." A fire puff legitimately spawned at world-y=0 (e.g. on flat terrain) would silently lose all height-based billowing animation with no error. | Add an `is_fire: bool` field to `PlagueSmoke`, or a zero-cost `FirePuffMarker` component so `update_plague_smoke` can use `Has<FirePuffMarker>` instead of the y-sentinel. |
| F4 | ArchitecturalDecay | `area_effects/fire_smoke.rs:14-79` | Medium | S | `spawn_fire_smoke_puff` (lines 14–45) and `spawn_fire_particle_puff` (lines 50–79) are nearly identical. The only difference is the material argument: the latter hardcodes `assets.fire_particle.clone()`. This is ~35 lines of duplication. | Remove `spawn_fire_particle_puff`. Its three call sites call `spawn_fire_smoke_puff` with `assets.fire_particle.clone()` as the material argument. |
| F5 | ConsistencyRot | `fire_effects.rs:246`, `cast_effects/cast_vfx.rs:55` | Medium | S | Two gravity values (`200.0` for explosion sparks; `150.0` for cast-flare sparks) are inlined as literals. Every other per-particle tuning value lives in `constants.rs`. | Add `pub(crate) const SPARK_GRAVITY: f32 = 200.0;` and `pub(crate) const CAST_FLARE_SPARK_GRAVITY: f32 = 150.0;` to `constants.rs`. |
| F6 | ConsistencyRot | `area_effects/fire_smoke.rs:82,116`, `area_effects/heat_shimmer.rs:27` | Low | S | `spawn_fire_black_smoke`, `spawn_fire_rising_smoke`, and `spawn_heat_shimmer_sized` are declared `pub` but are only called within the `vfx` module. They do not appear in the `area_effects/mod.rs` re-exports. Leaking these as crate-public widens the API surface unnecessarily. | Change visibility to `pub(super)` (or `pub(crate)` if cross-module use is planned). |
| F7 | Performance | `systems.rs:19-40` | Low | S | `update_fire_particle_time` and `update_aura_sphere_time` both call `Assets::iter_mut()` every frame (under `is_spell_effects_active`) regardless of whether any relevant particle entities are alive. | Guard `update_fire_particle_time` with `any_exist::<FireOrangeSmokePuff>()` and `update_aura_sphere_time` with `any_exist::<AuraBubbleVfx>()` to skip the material iteration when those effects are not in use. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|-------------------------|
| `cast_effects/cast_vfx.rs` | 468 | No | Four distinct VFX concerns in one file. Propose: `flares.rs`, `motes.rs`, `poofs.rs`, `aura_bubbles.rs` |
| `area_effects/fire_smoke.rs` | 282 | Yes | Under 300 LOC; all entries are variants of the same fire-smoke particle family (black smoke, orange fire particles, rising smoke, apex puff emission). Cohesive enough to exempt, but borderline. |
| `cast_effects/mp_sync.rs` | 227 | Yes | Under 300 LOC; single concern (MP-sync wrappers). Clean. |
| `area_effects/plague_smoke.rs` | 220 | Yes | Under 300 LOC; plague + fog cloud smoke variants plus shared `SmokePuffParams` config struct. Cohesive. |
| `explosion_effects.rs` | 209 | Yes | Under 300 LOC; coherent explosion-phase VFX. |
| `fire_effects.rs` | 256 | Yes | Under 300 LOC; fire glow/smoke/spark family. Cohesive. |
| `constants.rs` | 236 | Yes | Pure constants, well-sectioned. CLAUDE.md exempts cohesive constant files. |
| `components.rs` | 179 | Yes | All VFX particle components. Cohesive registry. |

---

### Looks bad but is actually fine

- **`systems.rs` as a pure `pub use` hub with two small system functions**: The two time-uniform update functions (`update_fire_particle_time`, `update_aura_sphere_time`) live here because they don't belong in any particular feature file. They are few lines and clearly named. The file is not a plugin.rs violation.
- **`plugin.rs` is clean registration-only**: All system bodies are in sibling files. The large `run_if` chain looks verbose but is the project-standard pattern.
- **12 golden-ratio seed lines across files** (`let seed = i as f32 * 1.618_034 + time_secs * k`): Each uses a different `k` scramble multiplier to produce visually distinct distributions. Not extractable into a meaningful shared helper without adding a parameter with no simplification gain.
- **`spawn_fire_glow` is generic over `screen_marker: M`** while other spawners hardcode `OnGameplayScreen`: This is intentional. `spawn_fire_glow` is documented as being called from both SP and MP ghost paths; the generics comment in the doc-block explains the reason. Other spawners are only called from systems already gated to SP or host-side contexts.
- **`SpellSchool` defined in `cast_effects/cast_vfx.rs`** rather than a dedicated types file: Only consumed by `spawn_school_flare` and the mp_sync wrapper in that same sub-module. Correct co-location.
- **`channel.rs` inside `spells/vfx/`**: Used exclusively by non-wizard units (healer, dispeller, shielder, teleporter). Placement in `vfx/` is appropriate as a shared VFX primitive; `pub(in crate::game)` visibility and registration in `units/plugin.rs` make the dependency explicit and correct.
- **Double `run_if` nesting in `plugin.rs`**: Outer `is_spell_effects_active` prevents all systems from running during menu/loading; inner `any_exist::<T>()` skips per-system query overhead when zero entities of that type exist. Both are needed and complementary.
- **Ghost fireball explosions emit VFX with `OnGameplayScreen`**: Ghost explosions run on the guest inside `MultiplayerGameState`. The VFX particles they spawn correctly use `OnGameplayScreen` because the game-state cleanup for the guest uses `OnMultiplayerGameScreen` for MP-only entities but `OnGameplayScreen` for shared simulation entities. This pattern is intentional (owned by fireball, outside vfx scope).

---

### Open questions

1. **SpellSchool + SpellSchoolWire unification**: Should these two enums be merged? The decode path in `ghost_update.rs` already has an exhaustive match from `SpellSchoolWire` → `SpellSchool`; eliminating one removes a whole class of wire-format drift risk.
2. **F3 y=0 edge case**: Can a fire puff legitimately spawn at world-space y ≤ 0 (e.g. on flat terrain or in a pit)? If so the `spawn_y > 0.0` sentinel already misfires silently in production.
3. **`update_aura_sphere_time` ownership**: `AuraSphereMaterial` is defined in `visual_assets` (outside the vfx scope) yet its time-uniform update lives in `vfx/systems.rs`. Is this the intended long-term home, or should it move to the parent `spells/plugin.rs` alongside the `MaterialPlugin::<AuraSphereMaterial>` registration?
