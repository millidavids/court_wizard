## units-batch2

**Scope:** `src/game/units/dispeller/`, `src/game/units/teleporter/`, `src/game/units/shielder/`, `src/game/units/aerialist/`

---

### Mental model

These four specialist attacker/defender units each wrap the same skeleton: a targeting system that sets `TargetingVelocity`, a movement system that calls the shared `calculate_weighted_movement` helper, and a specialty system (dispel channel, teleport channel, shield channel, or drop-bomb attack). Each module is self-contained with `plugin.rs`, `systems.rs`, `constants.rs`, `resources.rs`, `components.rs`, `mod.rs`.

The **dispeller** moves toward dispellable `NetworkedSpellEffect` entities and fires a bolt otherwise. The **teleporter** beelines toward the King, channels for 10 s, then bulk-teleports up to 20 attacker allies onto the King with temp-HP. The **shielder** finds the nearest unshielded same-team ally, channels, then inserts `SpellShield + ShielderDamageReduction`. The **aerialist** is a flying unit that uses momentum-based movement and fires archer arrows from altitude.

The dominant debt theme is **duplicated movement boilerplate** — the body of `dispeller_movement`, `shielder_movement`, `healer_movement`, and (partially) `teleporter_movement` are structurally identical, differing only in the `With<>` marker. A second theme is the **run_if guard mismatch** on the ChannelingCast-gated system groups: both dispeller and shielder guard with `any_with_component::<ChannelingCast>` (which fires whenever *any* unit in the world is channeling — healer, wizard, other units) rather than scoping to their own unit type.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | dispeller/systems.rs:185, shielder/systems.rs:152, healer/systems.rs:165, (brute/systems.rs:147) | High | M | `dispeller_movement` and `shielder_movement` are byte-for-byte identical except for the `With<Dispeller>` / `With<Shielder>` marker. The same shape appears in `healer_movement` and `brute_movement`. This is 4+ sites of duplicated ~120-line movement boilerplate that should live in `units/movement.rs` as a generic system. | Extract a macro or generic-system approach `fn standard_movement<M: Component>()` (using a `With<M>` query parameter or a Bevy generic system) in `units/movement.rs`, and replace all per-unit copies. |
| F2 | Performance | dispeller/plugin.rs:31-33, shielder/plugin.rs:31-33 | Medium | S | Both dispeller and shielder guard their ChannelingCast-dependent system group with `any_with_component::<ChannelingCast>`, a global check that returns `true` whenever *any* entity in the world has that component (healers, wizard channels, etc.). When a healer is healing, all of `dispeller_tick_dispel_channel`, `dispeller_refresh_casting_animation`, `dispeller_spawn_channel_particles`, and the shielder equivalents wake up every frame, iterate over empty queries, and return immediately. | Change to `any_exist::<Dispeller>().and(any_with_component::<ChannelingCast>)` for the dispeller group and the analogous compound guard for shielder. |
| F3 | Performance | dispeller/systems.rs:86-121 | Medium | S | In `update_dispeller_targeting`, `spell_edge_distance` is called twice per candidate pair inside `min_by` (once for `dist_a`, once for `dist_b`), then called a *third* time after the minimum is found to get `distance`. With N dispellable effects and D dispellers, worst case is O(D × N log N × 3) calls. Pre-compute distances into a `Vec<(Entity, Vec3, f32)>` before the sort to avoid the redundant work. | Compute `(entity, pos, dist)` triples up front with `filter_map`, store in a sorted vec, pick minimum by the cached distance. |
| F4 | ArchitecturalDecay | dispeller/systems.rs:593-610, teleporter/systems.rs:580-598 | Medium | S | Both `dispeller_ranged_combat` and `teleporter_ranged_combat` compute `.distance()` twice per candidate: once in the `.filter()` pass and again in `.min_by()`. The shared `find_closest_enemy_in_range` in `units/systems.rs:1849` exists precisely for this purpose but is never called here. | Replace the inline filter+min_by chain with `crate::game::units::systems::find_closest_enemy_in_range(...)`. |
| F5 | ConsistencyRot | dispeller/systems.rs:80,138,179,195,212-213, shielder/systems.rs:71,107,146,162,179-180 | Low | S | Both modules frequently use fully-qualified inline paths like `crate::game::units::components::InMelee`, `::Stunned`, `::Petrified`, `::CombatAnimation` in the middle of function bodies and query tuples, while the same symbols are already partially imported via `use crate::game::units::components::{...}` at the top of the file. The pattern is inconsistent within the same file. | Add `InMelee`, `Stunned`, `Petrified`, `CombatAnimation` to the existing `use crate::game::units::components::{ ... }` block so inline full paths are not needed. |
| F6 | ConsistencyRot | teleporter/systems.rs:619 | Low | S | `teleporter_ranged_combat` inserts `CombatAnimation::new_casting(...)` when firing a bolt attack. Every other ranged unit (`dispeller_ranged_combat:633`, archers) uses `CombatAnimation::new_attack(...)` for the shoot animation. The teleporter has no `attacking_texture` field in `TeleporterAssets`, so it reuses `casting_texture` — but the semantic is wrong. | Either add an `attacking_texture` to `TeleporterAssets` and use `new_attack`, or document clearly why the cast animation is intentionally used for bolts. |
| F7 | ArchitecturalDecay | aerialist/systems.rs:85,144 | Low | S | `aerialist_combat` takes `Res<crate::game::units::archer::resources::ArcherAssets>` and calls `crate::game::units::archer::systems::spawn_arrow(...)`. This creates a hard runtime dependency on the `archer` module's internal resources and spawn function. If archer arrow VFX is ever decoupled, aerialists break silently. | Extract `spawn_arrow` and its asset references into a shared `units/projectiles.rs` or a more general `spawn_ranged_arrow` helper, or add the arrow mesh/texture to `AerialistAssets` directly. |
| F8 | TypeContract | aerialist/systems.rs:384 | Low | S | Health is initialized as `Health::new(crate::game::constants::UNIT_HEALTH * 1.2)` with the `1.2` magic number inline. All other units define an explicit named constant (e.g., `TELEPORTER_HEALTH`, `DISPELLER_HEALTH`). | Add `pub const AERIALIST_HEALTH: f32 = crate::game::constants::UNIT_HEALTH * 1.2;` to `aerialist/constants.rs`. |
| F9 | ArchitecturalDecay | teleporter/systems.rs:354-403 | Low | S | The `update_channel_state` system defines a `push_sorted` closure inline inside a `match` arm, then calls it 3 times to build a priority-ordered pick list. The 3-block candidate-gather pattern (infantry → brute → commander) is repetitive and would benefit from a small private helper. | Extract `push_sorted` as a module-level `fn push_sorted_candidates(...)` in `systems.rs` and consider a `collect_ally_candidates(query_iter, pos) -> Vec<(Entity, f32, f32)>` helper to collapse the three identical `.filter().map().collect()` blocks. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `dispeller/systems.rs` | 640 | No | Contains 6 distinct concerns: targeting, movement, channel-start, channel-tick, animation refresh, particle VFX, ranged combat. Split into `targeting.rs`, `movement.rs`, `channel.rs`, `ranged_combat.rs`. |
| `teleporter/systems.rs` | 627 | No | Contains spawn, targeting, movement, channel state machine, animation refresh, particles, ranged combat, cleanup. Split into `spawn.rs`, `targeting.rs`, `movement.rs`, `channel.rs`, `ranged_combat.rs`. |
| `shielder/systems.rs` | 488 | No | Contains targeting, movement, channel-start, channel-tick, animation refresh, particle VFX. Split into `targeting.rs`, `movement.rs`, `channel.rs`. |
| `aerialist/systems.rs` | 404 | No | Contains spawn, targeting, movement (custom momentum-based), combat, height clamp. Split into `spawn.rs`, `targeting.rs`, `movement.rs`, `combat.rs`. |

---

### Looks bad but is actually fine

- **`dispellable_effects: Option<Vec<...>>` lazy init in `dispeller_tick_dispel_channel` (dispeller/systems.rs:413):** Looks like awkward control flow but is a correct one-time-per-frame query optimization when multiple dispellers complete simultaneously.
- **`Without<GhostEntity>` absent from all 4 modules:** Ghost units are spawned by the multiplayer guest-snapshot system with only generic components (`GhostEntity`, `Team`, `Health`, `Hitbox`, `WalkingAnimation`). None of them ever receive `Dispeller`, `Shielder`, `Teleporter`, or `Aerialist` markers. Therefore the absence of `Without<GhostEntity>` on these queries does not cause the documented MP ghost-gating bug.
- **Hardcoded `Team::Attackers` filters in `teleporter/systems.rs:374,385,396`:** Teleporters are exclusively attacker units (`spawn_single_teleporter` inserts `Team::Attackers`). The hardcoded filter is correct.
- **`push_sorted` closure captures no environment:** Correct. It takes all inputs as arguments to avoid lifetime issues with the outer query borrows. Not a closure anti-pattern here.
- **Three separate ally-type queries in `update_channel_state`:** Bevy forbids querying the same entity mutably through two `Query` params; the three separate queries (`infantry_allies`, `brute_allies`, `commander_allies`) are required by the borrow checker since they all need `&mut Transform` and the unit types overlap (a unit can't be both Infantry and Brute, but Bevy can't prove that statically).
- **`Local<f32>` timer in `dispeller_spawn_channel_particles` and equivalents:** Correct Bevy idiom for lightweight per-system state without a `Resource`.

---

### Open questions

1. Should `dispeller_movement` and `shielder_movement` be unified now that the Bevy 0.18 generic system machinery supports `With<M>` queries? The query tuple is large enough that a macro might be cleaner than a generic fn.
2. `dispeller/plugin.rs` and `shielder/plugin.rs` pass `any_with_component::<ChannelingCast>` — is this a copy-paste oversight from the healer plugin, or was it an intentional "run if the channeling VFX system is active" shortcut that predates the per-unit guards being added?
3. Does the teleporter's lack of `attacking_texture` in `TeleporterAssets` mean there is no distinct shoot animation on disk, or was it simply never wired up?
