## spell-disintegrate

**Scope:** `src/game/units/wizard/spells/disintegrate/`

---

### Mental model

Disintegrate is a channeled ray-beam spell with a full talent tree (3 tiers × 3 options). The wizard aims with the mouse during a cast phase, then channels a continuous beam that grows to full length and deals periodic damage to anything in its path. Talents unlock narrow/wide variants, a split forked-beam, escalating damage ramp, an auto-sweep, sky-dropping annihilation, a searing detonation on release, and mini-fireball pulsing along the beam.

The module is split into six files post "Phase 14" refactor: `plugin.rs` (registration), `constants.rs` (all tuning), `components.rs` (the `DisintegrateBeam` struct and its geometry helpers, plus light visual component markers), `casting.rs` (talent config resolution, the casting state machine, damage application, and cleanup), `beam.rs` (spawn factories, all Update visual systems, particle/smoke emitters, sweep and searing finale systems), and `systems.rs` (a two-line glob re-export hub for the plugin to reference without deep paths).

The multiplayer story is solid: ghost beams are pure mesh entities without a `DisintegrateBeam` component, intentionally bypassing all SP beam systems; per-frame impact VFX are reproduced in the guest via raw snapshot geometry helpers (`emit_impact_particles` / `emit_beam_smoke`) exported from `beam.rs`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| D-01 | ArchitecturalDecay | `beam.rs:1–742` | High | M | `beam.rs` at 742 LOC mixes five distinct concerns: beam spawn factories, continuous visual animation systems (`update_beam_visuals`, `update_beam_glow`, `update_beam_origin_flare`, `update_beam_eclipse`), particle/smoke emitters, the sweep talent system, and the searing finale system. Project convention requires splitting files exceeding ~300 lines unless every line is a single large match. | Split into: `spawn.rs` (spawn factories + despawn helper), `visuals.rs` (the four update-visual systems + `update_sweep_beams`), `particles.rs` (`spawn_impact_particles`, `emit_impact_particles`, `update_impact_particles`, `spawn_beam_smoke`, `emit_beam_smoke`, `update_beam_smoke`), `searing_finale.rs` (`update_searing_finale_detonations`, `spawn_searing_finale`). |
| D-02 | ArchitecturalDecay | `casting.rs:1–677` | High | M | `casting.rs` at 677 LOC mixes `TalentConfig` resolution, `BeamAction` enum, the core casting state machine, `apply_disintegrate_damage`, and `cleanup_beams_on_cancel`. Five distinct concerns in one file. | Split into: `talent_config.rs` (`TalentConfig`, `compute_talent_config`), `casting.rs` (state machine only, ~200 lines), `damage.rs` (`apply_disintegrate_damage`), `cleanup.rs` (`cleanup_beams_on_cancel` + `despawn_all_beam_visuals` helper). |
| D-03 | ArchitecturalDecay | `casting.rs:414–430, 458–476` | Medium | S | The beam-origin / range-clamped-target / direction / length computation is copy-pasted verbatim across two arms of the `CastingState` match (`Channeling` and `Casting`). The only difference is the resulting `BeamAction` variant. | Extract a `compute_beam_geometry(wizard_pos, target_pos, spell_range) -> (Vec3, Vec3, f32)` helper and call it from both arms. |
| D-04 | TypeContract | `components.rs:57–59` | Medium | S | `DisintegrateBeam.damage_type` is annotated `#[allow(dead_code)]`. It is set once in `DisintegrateBeam::new()` but never read; all damage systems use `constants::DAMAGE_TYPE` directly. The field provides no value and adds noise to every `::new()` call. | Remove the `damage_type` field from `DisintegrateBeam` and its `#[allow(dead_code)]` annotation. Update `new()` and any callers. |
| D-05 | Performance | `beam.rs:392–410, 496–522` | Medium | S | `emit_impact_particles` (lines 396–401) and `emit_beam_smoke` (lines 498–503) independently compute an identical perpendicular basis (`up = if direction.y.abs() > 0.9 { Vec3::X } else { Vec3::Y }; right = direction.cross(up).normalize(); forward = right.cross(direction).normalize()`). This 5-line pattern is duplicated inside the two most frequently called VFX helpers in the module. | Extract `fn perp_basis(direction: Vec3) -> (Vec3, Vec3)` returning `(right, forward)` and call it from both sites. Belongs in `beam.rs` or, once split, in a shared `particles.rs`. |
| D-06 | DocDrift | `components.rs:210–211` | Low | S | The doc comment on `contains_point_with_radius` states "The beam has uniform width along its entire length," but the implementation (line 248: `distance <= self.beam_width() * cone_t + unit_radius`) computes a cone that widens from zero at the origin to full width at the tip. `intersects_hitbox_cylinder` correctly documents "Cone radius widens linearly." | Fix the doc comment to say "cone that widens from 0 at the origin to full width at the tip." |
| D-07 | DocDrift | `casting.rs:121` | Low | S | The in-code comment for tier-3 selection 2 reads `// Unstable Resonance`, but the player-visible talent name in `talents/definitions/frost.rs:62` is `"Beam Fireballs"`. The field name `TalentConfig::resonance` and all internal constants also use the "resonance" terminology. | Rename internally to `beam_fireballs` / `beam_fireballs: bool` (field, comment, constant section header in `constants.rs`) to match the player-visible name and avoid confusion when reading talent tree descriptions. |
| D-08 | ErrorObservability | `casting.rs:553` | Low | S | The "pseudo-random angle" for annihilation resonance fireballs uses `beam.resonance_timer * 137.5`. After `resonance_timer -= MINI_FIREBALL_INTERVAL` the timer resets to near-zero (≈0..delta_secs ≈ 0..0.016 s), so `angle ≈ 0..2.2 radians`. The fireballs cluster in a narrow direction rather than spreading around the full circle as the comment implies. | Replace with `time.elapsed_secs() * 137.5` (same approach used in `emit_impact_particles` for rotating spread), or seed from the seeded-RNG resource for deterministic spread. |
| D-09 | TypeContract | `beam.rs:302` | Low | S | `update_beam_eclipse` falls back to a magic number `500.0` for `spell_range` when no `Wizard` entity is found. This value is not a named constant. | Extract to `constants::DEFAULT_SPELL_RANGE_FALLBACK` with a doc comment, or gate the system on `any_with_component::<Wizard>()` to avoid the fallback entirely. |
| D-10 | TestDebt | `components.rs:204–352` | Low | L | The pure-math geometry methods (`contains_point_with_radius`, `intersects_hitbox_cylinder`, `eclipse_geometry`) implement non-trivial spatial math with cone projections, XZ flattening, ellipse clipping, and edge cases. None have unit tests. These methods are high-churn collision primitives. | Add `#[cfg(test)]` unit tests for boundary cases: point at origin, point at tip, lateral miss, cone growth at `t=0.5`, `ground_collision` flattening, and `eclipse_geometry` range clipping. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|------------------------|
| `beam.rs` | 742 | false | Mixes 5 concerns. Split into `spawn.rs`, `visuals.rs`, `particles.rs`, `searing_finale.rs`. |
| `casting.rs` | 677 | false | Mixes 4 concerns. Split into `talent_config.rs`, `casting.rs` (narrowed), `damage.rs`, `cleanup.rs`. |
| `components.rs` | 372 | true | Single large component (`DisintegrateBeam`) with cohesive geometry helper methods and a few small marker structs. All lines are directly related to the beam data contract. |

---

### Looks bad but is actually fine

- **Ghost entity MP safety (`apply_disintegrate_damage`, `update_searing_finale_detonations`):** Neither system gates on `Without<GhostEntity>`. On the guest, `GhostEntity` units have `Hitbox` + `Health` + `Team` and would match the target query. However, these systems iterate `beam_query` first, and ghost beams deliberately carry no `DisintegrateBeam` component (documented in `spell_sync.rs:1093`), making the beam query empty on the guest. The damage loop is an inert no-op.

- **`systems.rs` glob re-exports (`pub use super::beam::*; pub use super::casting::*`):** This pattern leaks implementation helpers to crate scope. It is intentional: `arcane_crystal` and the MP spell-sync path depend on these items by name via the `disintegrate::systems::*` path. The re-export hub is the established Phase 14 architectural contract and is consistent with other spell modules (`entangle`, `finger_of_death`, `plague_wind`, `wall_of_fire`).

- **`cleanup_beams_on_cancel` allocates a `Vec<Entity>` per frame (line 650):** With a max of 3 wizard beams (forked) this is a heap allocation of 3 pointers. The `Vec::contains` lookups are O(3). Negligible in practice; the system is gated by `any_exist::<DisintegrateBeam>()` so it only runs when beams are alive.

- **Chained `.run_if` condition includes `ChannelingSfx` for gameplay systems:** The nine-system chain runs when any of five markers exist, including `ChannelingSfx` (which is alive during the cast phase before a beam spawns). `update_sweep_beams` and `apply_disintegrate_damage` run but do nothing when the beam query is empty. The condition is intentionally conservative to keep the chain simple.

- **`BEAM_LENGTH: f32 = 5000.0`:** Large value that looks like a placeholder. It is intentional — the beam is clamped to `wizard.spell_range` during casting logic; this constant is effectively "unbounded within spell range."

- **`Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0)` where `BEAM_ORIGIN_HEIGHT_OFFSET = 0.0`:** Looks dead but is an explicit design placeholder for a future height adjustment, changing only the constant.

---

### Open questions

1. `cleanup_beams_on_cancel` runs inside the chain gated by `any_exist::<DisintegrateBeam>()`. When the wizard's `CastingState` is `Resting` and no beam exists, the gate is `false` and cleanup never runs. Is there a window during the `Casting` state (before a beam spawns) where an interruption could leave `LocalWizard` in a non-Resting state without a beam, causing cleanup to never trigger?

2. The `damage_type` field is stored on `DisintegrateBeam` but never read back — all callers use `constants::DAMAGE_TYPE`. If a crystal beam ever sets a non-Fire damage-per-tick override, the damage type applied by `apply_disintegrate_damage` would silently remain Fire. Is this gap intentional?
