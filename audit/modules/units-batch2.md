## units-batch2

**Scope:** `src/game/units/dispeller/`, `src/game/units/teleporter/`, `src/game/units/shielder/`, `src/game/units/aerialist/`

---

### Mental model

Four specialist units that layer bespoke abilities on top of the shared movement/targeting pipeline:

- **Dispeller** (defender upgrade from archer): channels 5 s via `ChannelingCast`, fires a dispel projectile at the nearest `NetworkedSpellEffect`; falls back to magic bolts when no spell zones exist. Cooldown tracked by a `DispellerDispelCooldown` component.
- **Teleporter** (attacker): owns a three-state `TeleporterState` machine (Approaching → Channeling → Cooldown). After a 10 s channel it bulk-teleports up to 20 allies (infantry > brute > commander) onto the King position with massive temp HP.
- **Shielder** (defender upgrade from archer): channels 5 s via `ChannelingCast`, inserts `SpellShield + ShielderDamageReduction` on the nearest same-team ally without a shield.
- **Aerialist** (attacker): flies at Y=144, uses a custom momentum arc system (no `calculate_weighted_movement`), fires archer arrows at ground targets.

All modules are well-structured: `plugin.rs` is registration-only, `mod.rs` is pure re-exports, no `styles.rs`, no `unwrap()`. The dominant debt is **duplicated movement boilerplate** across dispeller/shielder, a **too-broad run-if guard** on channeling particle/refresh systems, **spell_edge_distance triple-call** in dispeller targeting, and several cross-cutting consistency nits. No multiplayer ghost-gating gaps (all four units are SP/host-only via `is_gameplay_running`).

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B2-01 | ArchitecturalDecay | `dispeller/systems/movement.rs:1`<br>`shielder/systems/movement.rs:1` | High | S | The two movement files are functionally identical — same CC-immobilize guard, same polymorph-wander block, same `calculate_weighted_movement` call, same stop-when-still guard. Only the marker component (`With<Dispeller>` vs `With<Shielder>`) differs across 137/138 lines. Any future modifier addition (e.g. new CC type) must be applied in both files. | Introduce a generic `standard_ground_movement<M: Component>()` in `units/movement.rs` and reduce each per-unit file to a one-line call. Bevy 0.18 supports generic systems. |
| B2-02 | ConsistencyRot | `dispeller/systems/movement.rs:97`<br>`shielder/systems/movement.rs:98`<br>`aerialist/systems/movement.rs:94`<br>+ 6 other unit files | Medium | M | The polymorphed-wander block (`velocity.x.to_bits() as f32` angle, speed `20.0`) is copy-pasted verbatim across 9+ unit movement files codebase-wide. No shared helper exists. | Extract `apply_polymorph_wander(velocity: &mut Velocity, time: &Time)` into `units/movement.rs` and call it from every unit's movement system. |
| B2-03 | Performance | `dispeller/plugin.rs:31-33`<br>`shielder/plugin.rs:31-33` | Medium | S | The secondary system groups (tick/refresh/particles) are guarded by `any_with_component::<ChannelingCast>`. `ChannelingCast` is also used by healers and by wizard channeling spells. These groups therefore pass their run-if whenever any healer or wizard channels — even with no dispellers/shielders alive — and pay scheduler dispatch cost unnecessarily. | Use a compound guard: `any_exist::<Dispeller>().and(any_with_component::<ChannelingCast>)` (and analogously for shielder). |
| B2-04 | Performance | `dispeller/systems/targeting.rs:69-105` | Medium | S | `spell_edge_distance` is called three times for the winning spell target: twice inside the `min_by` comparator (once per pair), then once more after `min_by` to read the distance. | Pre-compute `(entity, pos, dist)` in a single pass before `min_by`, cache the triple, and read `dist` from the returned element — no extra calls. |
| B2-05 | TypeContract | `aerialist/systems/movement.rs:50`<br>`aerialist/systems/spawn.rs:55-81` | Low | S | Aerialists receive no `CommanderAuraSpeedModifier` component at spawn, and their custom movement system never queries it. The commander aura system (which scans all `Without<Corpse>` units) will insert the component onto aerialists that walk into aura range, but it is silently ignored — wasting the insert/remove churn and making aerialists immune to commander speed buffs with no documentation. | Either add `CommanderAuraSpeedModifier(0.0)` at spawn and apply it in `aerialist_movement`, or explicitly document and enforce the intentional immunity by filtering aerialists out of the commander-aura affected query. |
| B2-06 | TypeContract | `aerialist/systems/spawn.rs:61` | Low | S | Aerialist health is expressed as `crate::game::constants::UNIT_HEALTH * 1.2` inline. Every other unit defines a named constant for its health value (e.g. `TELEPORTER_HEALTH`, `DISPELLER_HEALTH`). | Add `pub const AERIALIST_HEALTH: f32 = crate::game::constants::UNIT_HEALTH * 1.2;` to `aerialist/constants.rs` and use it in spawn. |
| B2-07 | ArchitecturalDecay | `aerialist/systems/spawn.rs:26` | Low | S | `spawn_single_attacker_aerialist` accepts `_level: u32` but never uses it (underscore prefix). It occupies the calling convention without purpose. | Remove the parameter and update the single call site in `loading/queue.rs`, or use it to scale health/count if tier scaling is planned. |
| B2-08 | ConsistencyRot | `dispeller/plugin.rs:22`<br>`shielder/plugin.rs:22` vs `teleporter/plugin.rs:26` | Low | S | Dispeller and Shielder guard primary systems with the project-local `any_exist::<T>()` wrapper; Teleporter uses Bevy's built-in `any_with_component::<Teleporter>`; Aerialist uses `any_exist`. Two equivalent mechanisms for the same check. | Standardise on one — prefer Bevy's `any_with_component` (avoids a custom wrapper) across all units. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `teleporter/systems/channel.rs` | 308 | No | Four loosely-coupled concerns: state-machine transitions + ally-collect + teleport execution in `update_channel_state`; animation refresh in `refresh_teleporter_casting_animation`; particle VFX in `spawn_channel_particles`; dead-channel cleanup in `cleanup_dead_teleporter_channels`. Proposed split: `channel_state.rs` (state machine + teleport execution), `channel_vfx.rs` (particles + animation refresh), `channel_cleanup.rs` (dead-channel indicator despawn). |

All other files in scope are well under 300 LOC.

---

### Looks bad but is actually fine

- **No `Without<GhostEntity>` in any of the four modules** — Ghost entities only receive generic components; none ever carry `Dispeller`, `Teleporter`, `Shielder`, or `Aerialist` markers. `is_gameplay_running` returns false for the guest, so these systems never execute guest-side. No ghost-gating gap.
- **Three separate mutable ally queries in `update_channel_state`** — Required by the Bevy borrow checker: a single `Query` cannot yield the same entity mutably twice, and infantry/brute/commander could theoretically be the same archetype slot. The three-query pattern is the standard Bevy workaround.
- **`dispellable_effects: Option<Vec<...>>` lazy init in `dispeller_tick_dispel_channel`** — Correct one-shot optimisation: only builds the effect list if at least one channel completes this frame.
- **`Local<f32>` particle timer in `*_spawn_channel_particles`** — Idiomatic Bevy per-system state. Not a resource smell.
- **`#[allow(clippy::too_many_arguments)]` throughout** — Project-accepted idiom for Bevy systems.
- **`partial_cmp(...).unwrap_or(Equal)` on positions** — Standard project NaN-safe f32 compare pattern.
- **`aerialist_combat` borrows `ArcherAssets`** — Aerialists fire arrows visually; sharing the archer arrow asset is correct and intentional until a bespoke aerialist projectile is introduced.
- **`teleporter/systems/channel.rs` hardcodes `Team::Attackers` filter** — Teleporters are exclusively attackers; this is a correct guard, not a magic constant.
- **Dispeller/Shielder have no `Teleportable` component** — They are defender upgrades; being teleportable (wizard `Teleport` spell) would be unexpected. Aerialist and Teleporter both include `Teleportable`, which is correct.

---

### Open questions

1. **Aerialist commander-aura immunity** — intentional design (flying units bypass ground buffs) or an omission from when the aerialist was written before the commander aura system existed?
2. **`_level` parameter on `spawn_single_attacker_aerialist`** — was tier-based scaling planned but shelved? If shelved, remove the dead parameter.
3. **Teleporter `attacking_texture` gap** — the ranged-bolt animation at `teleporter/systems/ranged_combat.rs:106` uses `new_casting` because `TeleporterAssets` has no `attacking_texture` field. Is a bespoke bolt-attack sheet planned, or should the casting sheet officially double as the attack animation?
4. **`dispeller_movement` / `shielder_movement` unification** — Bevy 0.18 generic system parameterisation is available; is the team ready to attempt the `fn standard_ground_movement<M: Component>()` approach (B2-01)?
