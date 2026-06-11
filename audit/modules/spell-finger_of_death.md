## spell-finger_of_death

**Scope:** `src/game/units/wizard/spells/finger_of_death/` — all 13 `.rs` files (casting/, effects/, components.rs, constants.rs, mod.rs, plugin.rs, systems.rs).

---

### Mental model

Finger of Death is a charged single-shot necrotic beam. The wizard holds the mouse to charge over 2 seconds (talent-modifiable), then releases or completes the charge to fire; mouse release before completion cancels. The module is split into: `casting/` (input handling, talent-param computation, beam spawn factory, primary damage system), `effects/` (deathmark chain beams, Reaper's Scythe sweep, necrotic explosion AoE, undead raises, all per-frame visual systems), `components.rs` (components + `FodTalentParams` struct), `constants.rs` (all tuning), and a 4-line `systems.rs` re-export hub. `plugin.rs` is pure Bevy registration. The module architecture is well-decomposed at the file level but three files exceed 300 LOC; the critical risk is four damage systems that mutate `Health` on world entities without `Without<GhostEntity>` filters.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | Security | `casting/damage.rs:82` | High | S | `apply_finger_of_death_damage` targets query is `Without<Wizard>` only — no `Without<GhostEntity>`. On the guest peer, if a ghost unit's CRDT-replicated `Health` causes the beam to perceive it as alive, the guest's FoD mutates ghost `Health` locally, diverging from host-authoritative state. Additionally, `DeathmarkDebuff` can be inserted on a ghost unit (line 217). | Add `Without<crate::game::multiplayer::components::GhostEntity>` to the `targets` query, matching `entangle/casting/root_effects.rs:66`. |
| F2 | Security | `effects/beam_effects.rs:100` | High | S | `update_reapers_scythe` targets query `Without<Wizard>` has no `Without<GhostEntity>`. The sweep's per-frame damage loop can mutate ghost unit `Health` on the guest for the full 1 s sweep duration. | Add `Without<GhostEntity>` to the `targets` query. |
| F3 | Security | `effects/necrotic_explosion.rs:63` | High | S | `apply_necrotic_explosion_damage` targets query `Without<Wizard>` has no `Without<GhostEntity>`. Every `NecroticExplosionBurst` applies AoE damage on its first tick to all non-Wizard entities in radius, including ghost units. | Add `Without<GhostEntity>` to the `targets` query. |
| F4 | Security | `effects/beam_effects.rs:26` | High | S | `update_deathmark_debuffs` `all_enemies` query `Without<Wizard>` has no `Without<GhostEntity>`. A ghost unit with `DeathmarkDebuff` (implanted by F1's unfixed path) triggers chain-beam emission when its locally-visible HP ≤ 0, causing a spurious beam that then runs through F1. Prerequisite: fix F1 to prevent `DeathmarkDebuff` insertion on ghosts. | Add `Without<GhostEntity>` to both `marked_targets` and `all_enemies` queries. |
| F5 | ArchitecturalDecay | `casting/casting_logic.rs:267–282` and `325–338` | Low | S | The 8-line "beam origin + range-clamp + direction + length" block is copy-pasted verbatim inside `CastingState::Casting` (line 267) and `CastingState::Resting` (line 325). | Extract a private `fn compute_beam_direction_and_length(cursor_pos: Vec3, beam_origin: Vec3, spell_range: f32, max_len: f32) -> (Vec3, f32)` and call from both branches. |
| F6 | DocDrift | `effects/undead.rs:7` | Low | S | The doc comment on `process_pending_undead_raises` reads `/// Computes talent-modified parameters for Finger of Death.` — a copy-paste error from `compute_fod_params`. | Replace with `/// Raises killed units as undead when PendingUndeadRaise is present (deferred by one frame so corpses exist).` |
| F7 | ConsistencyRot | `effects/undead.rs:9` | Low | S | `process_pending_undead_raises` takes `Option<Res<PendingUndeadRaise>>` despite the plugin gating it with `resource_exists::<PendingUndeadRaise>` (plugin.rs:45). The inner `let Some(pending) = pending else { return; }` guard is always `Some`, making it dead code. | Change parameter to `Res<PendingUndeadRaise>` and remove the `Option`-unwrap guard. |
| F8 | ErrorObservability | `casting/damage.rs:38–40`, `339–341`, `364–366`; `effects/necrotic_explosion.rs:28–30`; `effects/beam_effects.rs:224–226` | Low | S | Five `materials.get(&handle).cloned().unwrap_or_default()` calls silently substitute a blank `StandardMaterial` if the asset hasn't loaded. Invisible geometry yields no log. | Add `warn!("FoD: material asset not ready — falling back to default")` inside an `unwrap_or_else` closure, or guard with `let Some(mat) = ... else { warn!(...); return; }`. |
| F9 | ArchitecturalDecay | `casting/damage.rs:1–385` | Low | M | `damage.rs` is 385 LOC and mixes two distinct concerns: `spawn_beam` (mesh/material factory) and `apply_finger_of_death_damage` (beam hit-scan, occlusion, all talent post-effects, audio, VFX, mana drain, particle spawn). The factory could be tested and read independently. | Extract `spawn_beam` into `casting/spawn.rs` and update `casting/mod.rs` exports. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `casting/damage.rs` | 385 | No | Two distinct concerns: `spawn_beam` (factory) + `apply_finger_of_death_damage` (damage system with talent post-effects). Propose: `casting/spawn.rs` (spawn_beam) + `casting/damage.rs` (apply_finger_of_death_damage). |
| `casting/casting_logic.rs` | 353 | Yes | Three tightly-coupled functions all serving the casting state machine (`compute_fod_params`, `handle_finger_of_death_casting`, `finger_of_death_casting_logic`). Splitting `compute_fod_params` into `casting/talents.rs` is reasonable but not urgent — cohesion is high. |
| `effects/visual.rs` | 326 | Yes | Eight small visual animation systems (beam, glow, flare, veins, pulse, necrotic explosion burst). Each function is ≤50 LOC; they share the same concern (animate and despawn visual entities). Splitting would add file-navigation noise without clarity gain. |

---

### Looks bad but is actually fine

- **`plugin.rs` line 75 — `use systems::handle_finger_of_death_casting` after the `impl Plugin` block.** This is a use-import alias scoped to the file, not an exported symbol. No system body is defined in plugin.rs; the convention is respected.
- **`systems.rs` 4-line wildcard re-export hub.** Consistent with `disintegrate`, `entangle`, and other spell modules. Exists so `plugin.rs` uses a single `use super::systems::*` instead of enumerating each function.
- **`FodTalentParams` not deriving `Component`.** It is stored as a field on `FingerOfDeathBeam` and `DeathmarkDebuff`. No system queries on it independently, so it correctly remains a plain struct.
- **`BEAM_WIDTH` and `BEAM_WIDTH_FIRED` both equal `30.0`.** The comment "Same width after firing" and the `DEATHS_REACH_WIDTH_MULT` talent that scales them independently confirm this is deliberate — two names for future independent tuning.
- **Per-frame `materials.get_mut()` in visual update systems.** Standard Bevy pattern for independently animated spell effects; each beam has its own instanced material. Not a per-frame allocation; the mutation is in-place on an existing GPU resource.
- **`materials.get().cloned().unwrap_or_default()` + `materials.add()` in spawn functions.** These allocate on cast-fire events, not on every frame. Acceptable.
- **`msg_ctx` tuple parameter grouping in `apply_finger_of_death_damage` (line 104).** The comment at line 100 explains the Bevy 16-parameter system limit. Idiomatic workaround.
- **Trail beams from Reaper's Scythe spawn real `FingerOfDeathBeam` entities with `has_fired=true`.** `apply_finger_of_death_damage` correctly skips them (`beam.has_fired` guard at line 127). They are cleaned up within `POST_FIRE_DURATION = 0.3 s`.

---

### Open questions

1. **Ghost gating scope in versus mode**: Does `is_spell_effects_active` already return `false` for the versus guest's local SP systems? If so, F1–F4 are co-op-only risks. The fix is still correct in both modes.
2. **Deathmark infinite-chain risk**: `DeathmarkDebuff` is applied to surviving targets (line 216: `health.current > 0.0`). If chain-beam kills trigger another chain-beam for units that were previously debuffed, infinite chains are possible. Chain beams are marked `is_chain_beam=true` which skips wizard-state changes, but nothing prevents inserting `DeathmarkDebuff` inside the chain beam's damage pass (F1 path). Intentional cap or latent bug?
3. **`compute_fod_params` is `pub(crate)` but only used within the module.** Was it once called by a multiplayer sync path? If not, reduce to `pub(super)`.
4. **`clear_awaiting_fod_release` runs outside `is_spell_effects_active`** (plugin.rs:41). This is intentional ("runs independently of casting run conditions"), but should it be gated to not clear the flag during a paused game where the mouse is physically released?
