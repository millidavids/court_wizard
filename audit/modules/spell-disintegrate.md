## spell-disintegrate

**Scope:** `src/game/units/wizard/spells/disintegrate/` — 16 files, 2 103 LOC

---

### Mental model

Disintegrate is a channeled beam spell. The local-wizard casting system (`handle_disintegrate_casting`) drives a state machine (`Resting → Casting → Channeling`) and issues one of three beam actions (spawn / update / despawn-all) per frame. Beams live as `DisintegrateBeam` entities carrying all geometry and talent state; their companion visuals (glow cylinder, origin flare, ground eclipse, impact particles, smoke wisps) are separate sibling entities linked by entity reference. A shared `apply_disintegrate_damage` system ticks through every `DisintegrateBeam` — wizard-cast AND crystal-spawned — each frame. Multiplayer is handled by deliberately not attaching `DisintegrateBeam` to ghost visuals; the MP layer calls the beam's `emit_*` helpers directly. Talents are resolved once per frame into a `TalentConfig` struct. Searing Finale spawns a one-shot `SearingFinaleDetonation` on channeling-end. The module is cleanly decomposed across `casting/` and `beam/` sub-trees.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| D-01 | ArchitecturalDecay | `casting/beam_actions.rs:74-100` and `118-140` | Medium | S | The beam-origin / range-clamped-target / direction / length computation is copy-pasted verbatim across the `Channeling` arm and the `Casting` arm. The blocks are byte-for-byte identical except for the `BeamAction` variant returned at the end. | Extract `fn compute_beam_geometry(wizard_pos: Vec3, target_pos: Vec3, spell_range: f32) -> (Vec3, Vec3, f32)` returning `(origin, direction, length)` and call it from both arms. |
| D-02 | ArchitecturalDecay | `beam/particles.rs:62-68` and `164-170` | Low | S | The perpendicular-basis construction (`up` fallback + `right = direction.cross(up).normalize()` + `forward = right.cross(direction).normalize()`) is duplicated between `emit_impact_particles` and `emit_beam_smoke` in the same file. | Extract `fn beam_perp_basis(direction: Vec3) -> (Vec3, Vec3)` at the bottom of `particles.rs` and call it from both functions. |
| D-03 | TypeContract | `components.rs:259` | Low | S | `intersects_hitbox_cylinder` accepts `_hitbox_height: f32` (leading underscore = intentionally unused). The body performs full 3D cone-cylinder intersection but does not cull units whose vertical extent falls outside the beam's Y range. Callers believe height is factored in. | Either add a vertical extent check using `hitbox_height`, or remove the parameter from the signature and all call-sites so the contract matches the implementation. |
| D-04 | DocDrift | `components.rs:207-211` | Low | S | The doc comment on `contains_point_with_radius` states "The beam has uniform width along its entire length," but line 247 (`distance <= self.beam_width() * cone_t + unit_radius`) implements a cone that widens from 0 at the origin to full width at the tip. The sibling method `intersects_hitbox_cylinder` correctly documents "Cone radius widens linearly." | Fix the doc comment to say "cone that widens from 0 at the origin to full width at the tip." |
| D-05 | ErrorObservability | `casting/damage.rs:69-70` | Low | S | The pseudo-random angle for annihilation resonance mini-fireballs uses `beam.resonance_timer * 137.5` (line 69). After `resonance_timer -= MINI_FIREBALL_INTERVAL` the timer resets to near-zero (~0..0.016 s), so the angle barely advances from 0 each interval — fireballs cluster in a narrow arc rather than spreading around the full circle as the comment implies. | Replace with `time.elapsed_secs() * 137.5` (the same rotating-spread approach used in `emit_impact_particles`) so each pulse fires at a meaningfully different angle. |
| D-06 | TypeContract | `beam/visuals.rs:130-134` | Low | S | `update_beam_eclipse` falls back to the magic literal `500.0` for `spell_range` when no Wizard is found. This value is unnamed. | Define `constants::FALLBACK_SPELL_RANGE: f32 = 500.0` with a doc comment, or gate the system on `any_with_component::<Wizard>()`. |
| D-07 | ConsistencyRot | `casting/talent_config.rs:93-95` and `constants.rs:168-173` | Low | S | The tier-3 option 2 talent is called `resonance` in `TalentConfig` and all constant names (`MINI_FIREBALL_INTERVAL`, `MINI_FIREBALL_DAMAGE_FRACTION` etc.), but the constant section header says `// Talent: Beam Fireballs (T3-2)` and the comment in `talent_config.rs` says `// Unstable Resonance`. Two different names for the same talent in the same file. | Standardise: rename the `TalentConfig::resonance` bool to `beam_fireballs` and the `casting/damage.rs` field read to match. Keep constant section header aligned. |
| D-08 | TestDebt | `components.rs:204-351` | Low | L | The pure-math geometry methods (`contains_point_with_radius`, `intersects_hitbox_cylinder`, `eclipse_geometry`) implement non-trivial spatial math: cone projections, XZ flattening, ellipse clipping. No unit tests exist. These are the highest-churn collision primitives in the spell. | Add `#[cfg(test)]` tests for: point at origin (should miss cone), point at tip (should hit), lateral miss, `ground_collision` flattening, cone growth at `t=0.5`, `eclipse_geometry` range-clipping. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|------------------------|
| `components.rs` | 371 | true | All 371 lines are methods on two types (`DisintegrateBeam` geometry/collision helpers + `SearingFinaleDetonation`). Single-type cohesive method monolith — the explicit exemption case. |

---

### Looks bad but is actually fine

- **No `Without<GhostEntity>` on target queries in `apply_disintegrate_damage` and `update_searing_finale_detonations`:** These systems are only reachable when a real `DisintegrateBeam` exists. Ghost beams deliberately carry no `DisintegrateBeam` component (documented in `spell_sync/ghost_spawn.rs:359-360`), so on the guest the beam query is empty and the damage loop is an inert no-op. Ghost units lack `Health` by definition, so the target query would also not match them.
- **`apply_disintegrate_damage` beam_query has no `Without<CrystalSpawn>` filter:** Deliberate shared design. Crystal-spawned `DisintegrateBeam` entities intentionally delegate their damage-tick loop to this shared system; `arcane_crystal/hits.rs` only manages beam targeting.
- **`systems.rs` glob re-export hub:** `pub use super::beam::*; pub use super::casting::*` is an intermediate namespace layer. It is the established Phase-14 architectural contract referenced by `arcane_crystal` and the MP spell-sync path.
- **`cleanup_beams_on_cancel` allocates a `Vec<Entity>` per tick:** Maximum beam count is 3 (forked talent) so this is a 3-pointer heap allocation. The system is gated by `any_exist::<DisintegrateBeam>()` so it only runs when beams are alive.
- **`BEAM_LENGTH: f32 = 5000.0` looks like a placeholder:** Intentional. The beam is clamped to `wizard.spell_range` during casting; this constant is effectively "never the active limit."
- **`BEAM_ORIGIN_HEIGHT_OFFSET = 0.0`:** Explicit placeholder for a future height adjustment without code change.
- **`compute_talent_config` called every Update frame in `handle_disintegrate_casting`:** O(1) enum-match, no allocation, called only while primed for Disintegrate. Not a performance concern.

---

### Open questions

1. `cleanup_beams_on_cancel` is gated by `any_exist::<DisintegrateBeam>()`. When the wizard is in the `Casting` state (pre-beam) and gets interrupted (e.g. mana runs out), the state machine calls `casting_state.cancel()` and returns `DespawnAll` — but no beam exists, so the gate is `false` and cleanup never fires. Is there a window where `CastingState` can be left non-Resting without a beam and without cleanup running?
2. `DisintegrateBeam.damage_type` stores a value that is never read back — all damage paths use `constants::DAMAGE_TYPE` directly. If a crystal beam ever needs a different damage type (via `damage_per_tick_override`), it would be silently ignored. Intentional omission?
