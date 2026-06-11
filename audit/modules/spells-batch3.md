## spells-batch3

**Scope:** `lightning_rod/`, `chain_lightning/`, `mind_control/`, `healing_plume/`

---

### Mental Model

All four spells are feature-sliced with their own casting, components, constants, plugin, and systems files (4251 LOC total). The modules are generally well-structured following the project conventions. Every `Update` system has `run_if` guards. No `.unwrap()` calls in production paths. Ghost gating is present in `mind_control/` and `healing_plume/`, partially applied in `lightning_rod/` (rod entity correctly excluded, but arc target units are not), and completely absent in `chain_lightning/` — the highest-priority issue. There is a consistent per-spell `compute_talent_params` pattern which is intentional per-spell locality. Several files exceed 300 LOC and merit splitting. A per-bounce Vec allocation in chain lightning is a minor performance concern.

---

### Findings Table

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | Security | `chain_lightning/chain.rs:35-45` | High | S | `process_chain_lightning_bounces` queries `enemies` without `Without<GhostEntity>`. In multiplayer the guest's ghost army receives arc damage from chain lightning bounces on the host, corrupting ghost simulation state. | Add `Without<crate::game::multiplayer::components::GhostEntity>` to the `enemies` query filter at line 44. Mirror the pattern in `healing_plume/casting.rs:298`. |
| F2 | Security | `chain_lightning/casting/spell_cast.rs:54` | High | S | `enemies_query` and `health_query` in `handle_chain_lightning_casting` / `chain_lightning_casting_logic` lack `Without<GhostEntity>`. The initial damage hit on cast can strike ghost units. | Add `Without<crate::game::multiplayer::components::GhostEntity>` to `enemies_query` (line 54) and health query (lines 57–61). |
| F3 | Security | `lightning_rod/lifecycle.rs:39` | High | S | `ArcTargetFilter` type alias excludes `LightningStrike` entities but not `GhostEntity`. The `update_lightning_strikes` system arc-damages ghost units, running damage twice per peer. | Add `Without<crate::game::multiplayer::components::GhostEntity>` to `ArcTargetFilter`. |
| F4 | DocDrift | `healing_plume/aura.rs:18` | Low | S | Doc comment above `font_of_life_detect_deaths` reads "Computes talent parameters from active talent selections." — a copy-paste leftover from the talent-params pattern. | Replace with an accurate description: `/// Tier 3: Font of Life — detects corpses inside the zone and queues them for resurrection.` |
| F5 | ConsistencyRot | `healing_plume/constants.rs:54` and `aura.rs:68` | Low | S | `FIELD_MEDIC_COLOR` is typed as a raw `(f32, f32, f32, f32)` tuple instead of `Color`, forcing destructuring at the call site. The Font of Life resurrect tint (`Color::srgba(0.3, 0.9, 0.3, 1.0)` at `aura.rs:68`) is a separate nearly-identical green with no named constant. | Change `FIELD_MEDIC_COLOR` to type `Color`. Add a `FONT_OF_LIFE_RESURRECT_COLOR: Color` constant so both green tints are named and verifiable. |
| F6 | Performance | `chain_lightning/chain.rs:106-108` | Low | S | Inside the `for (bolt_entity, snapshot) in bolts_to_process` loop, walls/rocks/trees queries are `.collect()`-ed into `Vec<_>` on every active bolt tick. With Thunderstorm talent this can mean many short-lived Vec allocations per frame. | Hoist the three snapshot collections outside the loop (before line 94). |
| F7 | ConsistencyRot | `chain_lightning/systems.rs:3-4` | Low | S | Uses bare `pub use super::casting::*` and `pub use super::chain::*`, exporting everything wider than necessary. Sibling spell modules (`mind_control/systems.rs`, `lightning_rod/systems.rs`) use `pub(super)`. | Tighten to `pub(super) use` to match the project visibility convention. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|--------------------------|
| `mind_control/casting.rs` | 492 | false | Mixes `HighlightState` type + highlight management helpers + `find_nearest_enemy` + the main casting system. Split into: `casting/input.rs`, `casting/highlight.rs`, `casting/targeting.rs`. |
| `healing_plume/casting.rs` | 431 | false | Five distinct concerns: talent params, cast system, cast logic, heal application, and cleanse application. Split into: `casting/input.rs`, `casting/heal.rs`, `casting/cleanse.rs`. |
| `chain_lightning/chain.rs` | 419 | false | Bounce processing + on-hit effects + snapshot type + child-bolt spawn + target-finding + group cleanup. Split into: `chain/bounce.rs`, `chain/targeting.rs`, `chain/cleanup.rs`. |
| `lightning_rod/lifecycle.rs` | 399 | false | Rod tick + strike descent + arc-to-units spawn + arc-hit apply + arc visual spawn — five distinct concerns. Split into: `lifecycle/rod_tick.rs`, `lifecycle/strikes.rs`, `lifecycle/arcs.rs`. |
| `lightning_rod/casting.rs` | 383 | false | Talent config + descending-strike spawn + cast system + cast logic + rod entity spawn. Split into: `casting/input.rs`, `casting/spawn.rs`, `casting/talents.rs`. |
| `chain_lightning/casting/spell_cast.rs` | 325 | false | Cast system entry + inner logic + target-find helper. Extract `find_target_near_position_excluding` into `casting/targeting.rs`. |
| `healing_plume/aura.rs` | 317 | false | Font of Life, Healing Rain, Field Medic, zone fade, zone despawn — five lifecycle concerns. Split into: `aura/font_of_life.rs`, `aura/healing_rain.rs`, `aura/field_medic.rs`, `aura/zone_lifecycle.rs`. |
| `mind_control/effects.rs` | 313 | false | Traitor's mark, shared confused-combat helper, mass hysteria tick, amnesia tick, sleeper agent tick, hysteria targeting, strip helper. Split into: `effects/traitors_mark.rs`, `effects/confusion.rs`, `effects/sleeper.rs`. |

---

### Looks Bad But Is Actually Fine

- **`compute_talent_params` present in every spell module** — each function maps spell-specific talent indices to a spell-specific params struct. The per-spell locality is intentional; no shared abstraction would reduce duplication without losing clarity.
- **`lightning_rod/lifecycle.rs:235` uses inline `Vec3::new(x,0,z).distance(...)` instead of `xz_distance`** — functionally equivalent, slightly more explicit about which axes matter.
- **`mind_control/effects.rs:278` uses raw `(dx*dx+dz*dz).sqrt()`** in `update_mass_hysteria_targeting` — hot path over all hysteria units × all units; inline avoids an extra function call and import from a distant module. No bug.
- **`cleanup_chain_lightning_groups` is O(groups × bolts)** — bolt count is bounded tightly (~18 maximum at Thunderstorm peak). Not a real perf concern.
- **`handle_spell_release` absent from chain lightning casting** — chain lightning has no `SpellCaster` indicator, so the stripped-down inline `casting_state.cancel()` on `just_released` is correct and simpler.
- **`GhostSpellEffect` filter on `update_lightning_rod`** is intentionally different from `GhostEntity`: the rod itself carries `NetworkedSpellEffect`, and the guest-side ghost rod is tagged `GhostSpellEffect`. This correctly prevents double strike-spawning. The missing gating is only on the unit *targets* of those strikes (F3).
- **`mind_control/plugin.rs` at 93 LOC references `crate::game::units::boss::hags::systems`** — the MC wear-off and combat systems are genuinely shared between the wizard spell and the Hag boss; cross-module reuse is correct.

---

### Open Questions

1. Does the Thunderstorm talent (3 simultaneous bolts) interact correctly with `ChainLightningGroup` hit deduplication? Each bolt cast spawns its own group, so a single unit can be hit by all three initial strikes. Is intentional triple-damage, or should there be a per-frame shared exclusion set?
2. `healing_plume/aura.rs:font_of_life_resurrect` hardcodes `Team::Defenders` for all resurrections regardless of which team's unit died. Should attacker corpses that fall inside a Healing Plume zone also resurrect as defenders?
3. `chain_lightning/chain.rs:find_next_bounce_targets` collects candidates, filters LoS, sorts by distance, and takes N — but it already excludes `hit_entities` before the LoS check, meaning a LoS-blocked entity that was added to `hit_entities` on a prior bounce could open a slot for a farther valid target. Is the `max_targets` cap intended as "closest N valid targets" or "closest N in radius regardless of prior hits"?
