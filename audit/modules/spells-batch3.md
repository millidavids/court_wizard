## spells-batch3

**Scope:** `lightning_rod/`, `chain_lightning/`, `mind_control/`, `healing_plume/`  
28 files, 4 196 LOC.

---

### Mental model

All four spells follow the same outer shape: a `casting.rs` that owns input handling and the cast-time state machine, a `plugin.rs` that does registration only, a `components.rs` for data types, and a `constants.rs` for tuning values. Lightning Rod and Healing Plume additionally split the ongoing zone/lifecycle work into a second file (`lifecycle.rs` / `aura.rs`). Chain Lightning carries its bounce-propagation logic in `chain.rs`. Mind Control carries talent side-effects in `effects.rs`.

The shared `spells/utils.rs` helpers (`build_wizard_input`, `handle_spell_release`, `cleanup_spell_caster`, etc.) are used by most but not all of these spells — Chain Lightning and Mind Control inline their release-handling instead. All four spells are correctly run-if-gated at the plugin level.

The main structural risk is a recurring multiplayer ghost-targeting gap: the unit-mutation queries in Healing Plume, Chain Lightning, and Mind Control lack `Without<GhostEntity>` filters, matching the project's documented "#1 recurring MP bug class." Lightning Rod already has the correct `Without<GhostSpellEffect>` guard on its authoritative path.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B3-01 | Multiplayer | `healing_plume/casting.rs:287` | High | S | `apply_healing_plume_heal` targets query (`mut targets: Query<..., Without<Corpse>>`) lacks `Without<GhostEntity>`. On the host side, ghost units (visual representations of the guest's army) have `Health` and `Transform`, so they would receive healing ticks, get `SpellHealTally` inserted, and potentially have `TemporaryHitPoints` mutated. | Add `Without<crate::game::multiplayer::components::GhostEntity>` to the targets query filter, matching the pattern in `entangle/casting.rs:323` and `polymorph/systems.rs:470`. |
| B3-02 | Multiplayer | `mind_control/casting.rs:113` | High | S | `handle_mind_control_casting` enemies query (`(Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>)`) does not exclude `GhostEntity`. Ghost units carry `Team`, `Transform`, and `MeshMaterial3d`, so the find-nearest-enemy / highlight logic can select and insert `MindControlled`, `TraitorsMarkAura`, `AmnesiaOnExpiry`, etc. onto the host's visual mirror of the guest's units. | Add `Without<GhostEntity>` to the enemies query filter in `handle_mind_control_casting`. Same fix needed for the `existing_controlled` count query if it ever queries ghost entities by component shape. |
| B3-03 | Multiplayer | `mind_control/effects.rs:241` | High | S | `update_mass_hysteria_targeting` iterates `all_units: Query<(Entity, &Transform), Without<Corpse>>` which includes ghost entities. A hysteria-afflicted unit on the host will steer toward ghost mirror-images of the guest's army rather than real combatants. | Add `Without<GhostEntity>` to the `all_units` query. |
| B3-04 | ArchitecturalDecay | `healing_plume/casting.rs:280,387` | Medium | M | `apply_healing_plume_heal` and `apply_cleansing_plume` are zone-effect runtime systems, not casting logic, yet they live in `casting.rs` (427 LOC total). The file's declared concern is "casting and heal application" but it also owns ongoing zone behavior. This inflates the file and blurs the casting/zone boundary. | Move `apply_healing_plume_heal` and `apply_cleansing_plume` to a new `healing_plume/zone.rs` (or rename `aura.rs` to cover both spawn helpers and zone tick systems). Update `systems.rs` re-exports and `plugin.rs` imports. |
| B3-05 | ArchitecturalDecay | `mind_control/casting.rs:76` | Medium | M | `HighlightState`, `update_highlight`, and `clear_highlight` — 100 lines of target-highlighting logic — live inside `casting.rs` (486 LOC). Highlight management is a distinct visual concern that is unrelated to spell-casting state transitions. | Extract to `mind_control/highlight.rs`. `handle_mind_control_casting` can take `highlight: Local<HighlightState>` from the new module. |
| B3-06 | ConsistencyRot | `chain_lightning/casting.rs:247` | Medium | S | `chain_lightning_casting_logic` (and `mind_control/casting.rs:194`) inline the release-cancel logic (`if input.just_released { casting_state.cancel(); return false; }`) instead of calling the shared `handle_spell_release` helper from `spells/utils.rs`. Lightning Rod and Healing Plume already use the helper. The inline version also skips the `cleanup_spell_caster` call, which is load-bearing for indicator cleanup. | Replace the inline release checks with `handle_spell_release(&input, commands, wizard_entity, &mut casting_state, &caster_query)`. For Chain Lightning, thread `Commands` and `caster_query` into `chain_lightning_casting_logic`, or move the guard to the outer `handle_chain_lightning_casting` (which already has them). |
| B3-07 | ConsistencyRot | `chain_lightning/casting.rs:387` | Low | S | `find_target_near_position_excluding` constructs `Vec3::new(pos.x, 0.0, pos.z)` manually three times to compute XZ distance (lines 387, 406, 410–413) instead of calling the `xz_distance` helper from `spells/utils.rs`, which is used elsewhere in the same codebase. | Replace the four manual `Vec3::new(pos.x, 0.0, pos.z).distance(…)` expressions with `xz_distance`. |
| B3-08 | ConsistencyRot | `mind_control/casting.rs:397` | Low | S | `find_nearest_enemy` uses `(dx*dx + dz*dz).sqrt()` for the spell-range check (line 399) but `xz_distance(…)` for the target-proximity check (line 402). Both measure 2-D ground-plane distance; the inconsistency is confusing and the first form is slower. | Replace the manual sqrt form with `xz_distance(transform.translation, wizard_pos) <= spell_range`. |
| B3-09 | DocDrift | `mind_control/effects.rs:13` | Low | S | The doc comment on `update_traitors_mark_aura` reads `/// Computes talent parameters from active talent selections.` — clearly a copy-paste from the adjacent `compute_talent_params` function in `casting.rs`. | Replace with a correct summary, e.g. `/// Applies / removes the Demoralized debuff on enemies near a Traitor's-Mark unit.` |
| B3-10 | DocDrift | `healing_plume/aura.rs:18` | Low | S | The doc comment on `font_of_life_detect_deaths` reads `/// Computes talent parameters from active talent selections.` — the same wrong boilerplate. | Replace with `/// Scans new corpses inside Font-of-Life zones and tags them for resurrection.` |
| B3-11 | TypeContract | `chain_lightning/components.rs:24` | Low | S | `ChainLightningBolt.damage_type` is annotated `#[allow(dead_code)]` but the field is actively read: `bolt.damage_type` is copied into `ChainLightningBoltSnapshot.damage_type` and used by every `apply_spell_damage_with_team` call in `chain.rs`. The suppress annotation is stale and misleads future readers into thinking the field is vestigial. | Remove `#[allow(dead_code)]` from `ChainLightningBolt.damage_type`. |
| B3-12 | ArchitecturalDecay | `lightning_rod/casting.rs:247` | Low | S | `_wizard: &Wizard` parameter in `lightning_rod_casting_logic` is prefixed with `_` to suppress "unused" warnings, indicating it is never read inside the function. It is forwarded from the outer handler solely as dead weight. | Remove the parameter from `lightning_rod_casting_logic` and its call site (line 215). Similarly `_wizard_entity: Entity` in `chain_lightning_casting_logic` (casting.rs:226) is unused. |

---

### Oversized files (> 300 LOC)

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `mind_control/casting.rs` | 486 | No | Split into: `casting.rs` (talent params + input handler + state machine), `highlight.rs` (HighlightState + update_highlight + clear_highlight + find_nearest_enemy) |
| `chain_lightning/casting.rs` | 471 | No | Split into: `casting.rs` (talent config + outer handler + core logic), `targeting.rs` (find_target_near_position_excluding), `arc.rs` (spawn_arc) |
| `healing_plume/casting.rs` | 427 | No | Split into: `casting.rs` (talent params + input handler + core logic), `zone.rs` (apply_healing_plume_heal + apply_cleansing_plume) |
| `chain_lightning/chain.rs` | 419 | Yes | Single concern: bounce propagation logic (process_bounces, on-hit effects, child bolt spawning, target finding, group cleanup). Coherent enough at 419 LOC to remain. |
| `lightning_rod/lifecycle.rs` | 399 | Yes | Single concern: the full strike lifecycle (rod ticking, strike descent, arc spawning, arc damage). All functions are tightly coupled through shared type aliases and helpers. |
| `lightning_rod/casting.rs` | 383 | Yes | Contains: talent-param computation, descending-strike spawn, outer casting handler, core casting logic, rod entity spawn. Each helper is called by exactly one other function in the same file; no cross-cutting reuse. Borderline but coherent. |
| `healing_plume/aura.rs` | 317 | Yes | Contains: Font-of-Life detect/resurrect, Healing Rain move, Field Medic convert/revert, fade + cleanup, and zone spawn helper. All zone-passive behavior; no single concern dominates enough to justify another split. |

---

### Looks bad but is actually fine

- **`update_lightning_rod` does not have a `Without<GhostSpellEffect>` in a query filter (line 49–51)** — it does; `Without<GhostSpellEffect>` is correctly present on the `rods` query in `lifecycle.rs:48–52`. The ghost zone cannot fire its own strikes.
- **`process_chain_lightning_bounces` lacks `Without<GhostEntity>` on its `enemies` query** — `ChainLightningBolt` is only ever spawned by `handle_chain_lightning_casting` (a `LocalWizard`-gated system). On the guest peer, that wizard is also a `LocalWizard` but with a different primed spell; the run-if on `any_exist::<ChainLightningBolt>()` means the bounce system simply never fires when no bolts are present. The guest never spawns bolts from the host's cast, so ghost unit contamination via bolt processing does not occur in practice. Still, a `Without<GhostEntity>` guard would make the invariant explicit.
- **`cleanup_chain_lightning_groups` iterates all groups then all bolts (O(n*m))**  — in practice the max live groups per cast is `THUNDERSTORM_CAST_COUNT = 3`, and bolts decay within a handful of frames. The quadratic scan is negligible.
- **`apply_healing_plume_heal` runs on the guest-spawned ghost `HealingPlumeZone` (heal_per_tick = 0.0)** — the guest zone is spawned with `heal_per_tick: 0.0`, so no actual healing fires. The zone does advance `time_alive` and trigger mote visuals, which is intentional for visual parity. The ghost zone does not carry `FontOfLifeZone`, `CleansingPlumeZone`, or `OverflowZone`, so those sub-systems are naturally skipped.
- **`find_target_near_position_excluding` builds three separate iterator chains and chains them** — this looks wasteful but the candidate lists are small (at most a few hundred units + handful of rods/crystals) and the chain is non-allocating until `.collect()`.
- **`INDICATOR_COLOR` constant in `mind_control/constants.rs` appears unused** — it is referenced from `src/game/units/wizard/spells/visual_assets.rs:367` to create the Mass Hysteria indicator material.
- **Mind Control plugin.rs (93 LOC) registers systems from `hags::systems`** — this is intentional cross-module registration for shared MC behavior owned by the wizard spell but reused by the hag boss. The coupling is load-bearing and documented with inline comments.

---

### Open questions

1. Should the `Without<GhostEntity>` gap in `apply_healing_plume_heal` and `mind_control` be treated as a known-safe limitation (ghost zones always have zero heal, ghost units have no real effect on host simulation) or as an active MP bug risk? The project memory notes this pattern as the "#1 recurring bug class" suggesting the explicit guard is always preferred.
2. `chain_lightning_casting_logic` and `mind_control` inline release handling without calling `cleanup_spell_caster`. Are there indicator-leak issues in those spells if the player releases the mouse mid-cast? Chain Lightning has no indicator, but Mind Control's Mass Hysteria mode does spawn one — this should be verified.
3. `lightning_rod/casting.rs` calls `spawn_lightning_rod` as `pub(crate)`. Is this visibility needed for the multiplayer guest-side spawn path, or can it be tightened to `pub(super)`?
