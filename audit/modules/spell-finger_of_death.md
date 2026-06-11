## spell-finger_of_death

**Scope:** `src/game/units/wizard/spells/finger_of_death/`

---

### Mental model

Finger of Death is a high-investment beam spell that grows during a 2-second cast and fires a one-shot instant-kill beam. The module is split across six non-trivial files: `casting.rs` handles input, beam lifecycle, and the primary damage application; `effects.rs` owns all post-cast talent effects (undead raise, deathmark chain, reaper's scythe sweep, necrotic explosion, and all visuals); `components.rs` defines the data model including the fat `FodTalentParams` struct; `constants.rs` holds all tuning values; `plugin.rs` is pure registration; `systems.rs` is a thin re-export hub. The spell is well-structured at the module level but carries two oversize files, a critical multiplayer ghost-damage bug, two instances of duplicated range-clamping logic that already has a shared helper in `utils.rs`, and a wrong doc comment copied from an adjacent function.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `casting.rs:286-302` & `casting.rs:345-358` | Medium | S | The 8-line "clamp target to spell range, compute direction, compute beam_length" block is copy-pasted verbatim inside `CastingState::Casting` and again inside `CastingState::Resting`. `utils.rs` already exports `clamp_to_spell_range` (line 257) for exactly this purpose. | Extract to a private helper `compute_beam_direction_and_length(cursor_pos, beam_origin, wizard_range) -> (Vec3, f32)` or call `utils::clamp_to_spell_range` and compute the direction from its result; eliminate both duplicate blocks. |
| F2 | ArchitecturalDecay | `casting.rs:424` | High | M | `apply_finger_of_death_damage` is 310 lines and handles beam firing, wall/rock occlusion, talent dispatch (soul harvest, siphon life, undead raise, necrotic explosion, deathmark application), sound, VFX, mana drain, mouse-state mutation, and talent-progress tracking. It is a god function. | Split into `fire_beam_damage`, `apply_fod_talents_on_fire`, and a thin coordinator. Most talent side-effects already have dedicated systems in `effects.rs` — only the hit-scan loop and mana drain belong in this function. |
| F3 | ArchitecturalDecay | `systems.rs:1-4` | Low | S | `systems.rs` is a glob re-export hub (`pub use super::casting::*; pub use super::effects::*;`) with a stale implementation note "Phase 14" in the module doc. Glob re-exports from `systems.rs` pollute the public namespace unnecessarily — every symbol from both files is now visible without the calling module knowing which file owns it. | Either drop `systems.rs` and have `plugin.rs` import directly from `casting` and `effects`, or keep it but use named `pub use` re-exports only for the symbols `plugin.rs` references. Remove the stale "Phase 14" comment. |
| F4 | DocDrift | `effects.rs:21` | Low | S | The doc comment on `process_pending_undead_raises` reads `/// Computes talent-modified parameters for Finger of Death.` — a copy-paste from `compute_fod_params` in `casting.rs`. | Fix the doc comment to describe what `process_pending_undead_raises` actually does: raising corpses near kill positions as undead infantry. |
| F5 | ErrorObservability | `casting.rs:387-388`, `effects.rs:102-103`, `effects.rs:419-420` | Medium | S | `materials.get(&handle).cloned().unwrap_or_default()` silently substitutes a blank `StandardMaterial` when the asset hasn't loaded. This yields invisible geometry with no log. The same pattern appears in `spawn_beam` (casting.rs 387), `spawn_necrotic_explosion` (effects.rs 102), and the sweep visual trail (effects.rs 419). | Use `.unwrap_or_else(|| { warn!("FoD: material asset not loaded"); StandardMaterial::default() })` or guard with `let Some(mat) = ... else { return; }`. At minimum add a `warn!` so invisible-geometry cases are observable. |
| F6 | Performance | `effects.rs:332-338` | Low | S | `update_reapers_scythe` allocates a full `FingerOfDeathBeam` struct on every frame tick (once per active sweep) purely to call `beam.contains_point` and `beam.beam_width()`. `FingerOfDeathBeam::with_talents` clones the 12-field `FodTalentParams` struct every frame. | Inline the two small helpers (`contains_point` logic and `beam_width` formula) directly in the sweep loop using local variables from the `ReapersScytheSweep` fields; avoid the per-frame struct allocation. |
| F7 | Security | `casting.rs:430-440`, `effects.rs:131-141`, `effects.rs:285-294` | High | M | The targets query in `apply_finger_of_death_damage`, `apply_necrotic_explosion_damage`, and `update_reapers_scythe` uses `Without<Wizard>` only. In Versus multiplayer the guest has a `LocalWizard` and the host's ghost units (`GhostEntity`) carry a `Health` component (confirmed in `guest_snapshot.rs:1151-1158`). Since `is_spell_effects_active` returns `true` for MP guests, the guest's local FoD beam will iterate and damage ghost units — applying double-authoritative damage. Other spells with the same concern (entangle, mark_of_death, polymorph) already carry `Without<GhostEntity>` filters with explanatory comments. | Add `Without<crate::game::multiplayer::components::GhostEntity>` to the targets query filter-tuple in all three damage systems, matching the pattern established in `entangle/casting.rs:323`. |
| F8 | ConsistencyRot | `casting.rs:285-300` | Medium | S | FoD inline-clamps cursor to `wizard.spell_range` while `disintegrate/casting.rs:420-425` does the identical manual clamp. The shared helper `utils::clamp_to_spell_range` exists (`utils.rs:257`) but both beam spells bypass it. | Both spells should adopt `utils::clamp_to_spell_range(cursor_pos, beam_origin, wizard.spell_range)` to keep range enforcement consistent. |
| F9 | TypeContract | `effects.rs:47-52` | Low | S | The nearest-corpse search in `process_pending_undead_raises` uses `best.as_ref().map(|(_, d)| *d).unwrap_or(f32::MAX)` — verbose and slightly error-prone pattern. Also, if multiple beams fire on the same frame, `commands.insert_resource(PendingUndeadRaise { ... })` at `casting.rs:653` silently overwrites a prior raise list from the same frame. | Use `best.map_or(f32::MAX, |(_, d)| d)`. For the overwrite issue, consider accumulating into an existing resource (or appending to a `Vec` via a message) if chain beams can trigger in the same frame as the primary beam. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `casting.rs` | 734 | No | Split `apply_finger_of_death_damage` (lines 424–734, 310 lines) into `damage.rs` (beam hit-scan + mana drain + immediate talent effects) and keep casting input + `finger_of_death_casting_logic` + `spawn_beam` in `casting.rs`. Proposed siblings: `casting.rs`, `damage.rs`. |
| `effects.rs` | 753 | No | Mixes undead-raise logic, chain-beam deathmark logic, sweep-combat logic, and all visual update systems. Proposed split: `visual.rs` (update/cleanup for beam, glow, flare, vein, pulse, necrotic explosion burst visuals) + `effects.rs` (undead raise, deathmark chain, reaper's scythe combat only). |

---

### Looks bad but is actually fine

- **`plugin.rs` line 75 — `use systems::handle_finger_of_death_casting;` outside the impl block.** Looks like plugin.rs exports logic, but this is only a use-import alias to shorten the name inside `build()`. No system body or helper is defined here; the convention is respected.
- **`systems.rs` glob re-exports.** Glob re-exports from a re-export module are unusual but not strictly forbidden. The module exists to give `plugin.rs` a single `use super::systems::*;` import; it does not define logic.
- **`FodTalentParams` not deriving `Component`.** It is a plain data struct stored as a field on `FingerOfDeathBeam` and `DeathmarkDebuff`, both of which are Components. No system independently queries on `FodTalentParams`, so it correctly does not need to be a Component.
- **`BEAM_WIDTH` and `BEAM_WIDTH_FIRED` having identical values (`30.0`).** The comment "Same width after firing" confirms this is intentional — the constant exists so the visual can be tuned separately from the charge-up width. The `DEATHS_REACH_WIDTH_MULT` talent modifier scales both independently, which further validates the two-constant design.
- **Per-frame `materials.get_mut()` calls in visual update systems.** Mutating a material's `base_color` and `emissive` every frame is the standard Bevy pattern for animated spell effects; each beam already has its own instanced material handle.
- **`ReapersScytheSweep.hit_entities: HashSet<Entity>` on a Component.** The set grows to at most the number of living battlefield units; it is neither unbounded nor per-frame-allocated.
- **`msg_ctx` tuple parameter grouping in `apply_finger_of_death_damage` (line 451).** The comment at line 449 documents this is required to stay under Bevy's 16-parameter system limit. This is idiomatic.

---

### Open questions

1. Should `FodTalentParams.chain_damage_mult` default to `DEATHMARK_CHAIN_DAMAGE_PERCENT` (0.1) when `deathmark = true`, or is resetting it in `update_deathmark_debuffs` (`effects.rs:255`) the intended pattern? Currently chain beams get the 0.1 multiplier set twice — once in `compute_fod_params` and again in the deathmark fire path.
2. `PendingUndeadRaise` is a `Resource` with `kill_positions: Vec<Vec3>`. If a chain beam and a primary beam both fire in the same frame (possible via the deathmark path), `commands.insert_resource` at `casting.rs:653` silently drops the first raise list. Is this intended, or should it accumulate?
3. The "Phase 14" comment in `systems.rs` references an internal refactoring phase. Should stale phase references be scrubbed from module docs?

### Mental model

Finger of Death is the Necromancer-archetype's signature instant-cast beam that drains significant mana, stuns the wizard for a mouse-release cycle, and tears through anything in a straight line. The module is split across six files (plus a re-export hub `systems.rs`):

- `constants.rs` – all numeric tuning values (visual + gameplay)
- `components.rs` – all components/resources including `FodTalentParams`, the beam, debuffs, and the talent result struct
- `casting.rs` – local wizard input handling, talent-param computation (`compute_fod_params`), the internal `BeamAction` enum/`CastingResult` struct, `spawn_beam`, and the monolithic `apply_finger_of_death_damage`
- `effects.rs` – post-cast effects: undead raises, necrotic explosion AoE, Deathmark chain-beam, Reaper's Scythe sweep, and all per-frame visual update systems
- `plugin.rs` – Bevy system registration only (clean)
- `systems.rs` – wildcard re-export hub (established pattern across the codebase)

The architecture is sound for an SP spell. The dominant risks are (1) `casting.rs` and `effects.rs` are both substantially over 300 LOC and contain multiple distinct concerns each, (2) two identical range-clamping code blocks inside `finger_of_death_casting_logic`, (3) a copy-paste doc comment left on the wrong function, and (4) the talent-effect damage systems (`update_deathmark_debuffs`, `apply_necrotic_explosion_damage`, `update_reapers_scythe`) lack explicit `Without<GhostSpellEffect>` / `Without<GhostEntity>` guards — they're incidentally safe today because those components are only spawned by the local cast path, but carry silent fragility if a future sync change pushes them to the guest.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | casting.rs:425 | High | M | `apply_finger_of_death_damage` is 310 lines (lines 425–734) and handles damage resolution, mana drain, screen FX, audio, soul-harvest mana refund, siphon-life heal queue, undead-raise queue, necrotic-explosion spawning, kill vein particles, and pulse ring spawn — six or seven distinct concerns in one function body. | Extract sub-helpers: `drain_mana_and_state`, `apply_fod_hit_effects` (veins + pulse), `handle_kill_effects` (undead raise, necrotic explosion, soul harvest, siphon life). Each is <60 lines and independently readable. |
| F2 | ArchitecturalDecay | casting.rs:287–302 / 341–356 | Medium | S | The beam origin + range-clamping block (7 lines: `beam_origin`, `to_target`, `distance`, `clamped_target`, `direction`, `beam_length`) is duplicated verbatim inside `finger_of_death_casting_logic` for the `Casting` branch (line 287) and the `Resting` first-spawn branch (line 341). | Extract a private `fn compute_beam_origin_and_dir(wizard_pos: Vec3, cursor_pos: Vec3, spell_range: f32) -> (Vec3, Vec3, f32)` and call it from both branches. |
| F3 | DocDrift | effects.rs:21 | Low | S | The doc comment on `process_pending_undead_raises` reads `"Computes talent-modified parameters for Finger of Death."` — clearly copy-pasted from `compute_fod_params` in casting.rs. | Replace with `"Processes deferred undead raises spawned by Finger of Undeath, matching the nearest corpse to each kill position."` |
| F4 | Performance | effects.rs:40 | Low | S | `raised_entities` in `process_pending_undead_raises` is a `Vec<Entity>` used as a "seen" set with `Vec::contains` on line 44 — O(n²) for multi-kill salvos. | Replace with `HashSet<Entity>` (already imported in `components.rs` for `ReapersScytheSweep`). |
| F5 | TypeContract | effects.rs:49 | Low | S | `best.as_ref().map(|(_, d)| *d).unwrap_or(f32::MAX)` inside the already-`Some`-guarded condition — the outer `best.is_none() ||` short-circuits before `best.as_ref()` could be `None`, making the `.unwrap_or` unreachable. The logic is correct but misleading. | Simplify to `if let Some((_, best_dist)) = best { dist < *best_dist } else { true }` (identical to the pattern used correctly in `update_deathmark_debuffs` at line 239). |
| F6 | Security | effects.rs:216 / casting.rs:425 | Medium | M | `apply_finger_of_death_damage`, `apply_necrotic_explosion_damage`, `update_deathmark_debuffs`, and `update_reapers_scythe` all mutate `Health` on world entities and are gated only by `is_spell_effects_active`, which runs on both host and guest in multiplayer. They are currently safe because the spawning path (`apply_finger_of_death_damage`) requires a `LocalWizard` beam, and `NecroticExplosionBurst` / `ReapersScytheSweep` / `DeathmarkDebuff` are not synced to the guest. However, there are no explicit `Without<GhostSpellEffect>` / `Without<GhostEntity>` guards — a future sync addition could silently introduce double-damage. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the beam query in `apply_finger_of_death_damage` and to the effect-entity queries in `apply_necrotic_explosion_damage` / `update_reapers_scythe` / `update_deathmark_debuffs` as a defensive belt-and-suspenders guard, matching the pattern in `arcane_crystal/hits.rs` and `plague_wind/cloud.rs`. |
| F7 | ArchitecturalDecay | casting.rs:446–460 | Low | S | `msg_ctx` tuple parameter exists solely to pack `MessageWriter<ScreenDesaturateMessage>`, `MessageWriter<VignettePulseMessage>`, and `Option<Res<MultiplayerSession>>` together to stay under Bevy's 16-param system limit. The comment explains this but there are only 14 top-level params already (the tuple counts as 1). The workaround is harmless but adds noise. | Verify actual param count; if under 16 without the tuple, unwrap it. If still needed, document the exact count in the comment so the next reader doesn't have to recount. |

---

### Oversized files

| File | LOC | Exempt | Reason | Proposed split |
|------|-----|--------|--------|----------------|
| casting.rs | 734 | No | Contains `compute_fod_params` (talent computation), `finger_of_death_casting_logic` (core state machine), `BeamAction`/`CastingResult` private types, `spawn_beam` (visual factory), and `apply_finger_of_death_damage` (massive damage + FX system) — five distinct concerns. | `talent_params.rs` (`compute_fod_params`, `FodTalentParams`-related logic), `beam_spawn.rs` (`spawn_beam`, `BeamAction`, `CastingResult`), `damage.rs` (`apply_finger_of_death_damage`); keep `casting.rs` for `handle_finger_of_death_casting` + `finger_of_death_casting_logic`. |
| effects.rs | 753 | No | Contains undead-raise logic, chain-beam/deathmark, Reaper's Scythe sweep (incl. damage), necrotic explosion AoE damage, and five separate visual-update systems. | `raise_undead.rs` (`process_pending_undead_raises`), `talents.rs` (`update_deathmark_debuffs`, `update_reapers_scythe`, `apply_necrotic_explosion_damage`, `spawn_necrotic_explosion`), keep `effects.rs` for the pure visual update systems (`update_finger_of_death_beam_visuals`, `update_necrotic_veins`, `update_finger_of_death_glow`, etc.). |

---

### Looks bad but is actually fine

- **`systems.rs` being only 4 lines of wildcard re-exports** — This is the established pattern across all spell modules (`disintegrate`, `entangle`, `plague_wind`, etc.) used so `plugin.rs` can import via `super::systems::*` without enumerating each function. Not a violation.
- **`#[allow(clippy::too_many_arguments)]` on multiple functions** — `handle_finger_of_death_casting` (15 params) and `apply_finger_of_death_damage` (~14 params) are Bevy systems with injected resources/queries; the project CLAUDE.md explicitly sanctions this attribute for systems.
- **`materials.get(...).cloned().unwrap_or_default()` pattern** (casting.rs:385–388, casting.rs:684–688, etc.) — This clones the base material to give each beam/vein an independent animated instance. It's only called on spawn events (once per cast, not every frame), so it's not a per-frame allocation issue.
- **`FingerOfDeathBeam::with_talents` creating temporaries inside `update_reapers_scythe`** (effects.rs:332–338) — The beam struct is a lightweight value type used only for `contains_point` hit-detection, not spawned as an entity. Acceptable.
- **`std::collections::HashSet::new()` inline in `ReapersScytheSweep` spawn** (casting.rs:498) — One allocation per sweep initiation; fine.
- **`plugin.rs` importing private casting/effects items directly** — `use systems::handle_finger_of_death_casting` at line 75 (after the impl block) is a Rust placement quirk but valid; the system is correctly private to the module and only registered here.
- **`BEAM_ORIGIN_HEIGHT_OFFSET = 0.0`** — Looks dead but is intentional: it names the zero-offset for semantic clarity and future tunability without magic numbers.

---

### Open questions

1. **Reaper's Scythe visual trail spawns real `FingerOfDeathBeam` entities** (effects.rs:423–440) — these trail beams go through the normal `update_finger_of_death_beam_visuals` and `cleanup_finger_of_death_beams` paths. Do they also go through `apply_finger_of_death_damage` on the next frame? They are spawned with `has_fired = true` and `cast_progress = 1.0`, so `apply_finger_of_death_damage` would skip them (`beam.has_fired` is true). Confirmed safe, but worth a comment on the spawn.
2. **`clear_awaiting_fod_release` runs unconditionally on `AwaitingFingerOfDeathRelease`** (plugin.rs:41–42) — it doesn't have `is_spell_effects_active`. This appears intentional (the comment says "runs independently of casting run conditions"), but should it also be gated to prevent clearing on a paused game where the mouse might be released?
3. **Chain beam (Deathmark) spawned in `update_deathmark_debuffs`** inherits `talent_params.is_chain_beam = true`, which skips wizard state changes in `apply_finger_of_death_damage`. But the chain beam will also trigger `AwaitingFingerOfDeathRelease` via... wait, it won't — `is_chain_beam` bypasses that entire block. What prevents a chain-beam kill from spawning another chain beam (infinite chain)? The `DeathmarkDebuff` is only applied by the primary beam to survivors; if the chain beam kills someone who also has a `DeathmarkDebuff`, it would trigger another chain. Is that intended gameplay or an infinite-chain bug?
