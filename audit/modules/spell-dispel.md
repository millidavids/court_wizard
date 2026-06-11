## spell-dispel

**Scope:** `src/game/units/wizard/spells/dispel/`

---

### Mental Model

The Dispel spell fires a straight-line projectile that detonates on ground contact, spawning an expanding DispelImpact sphere that suppresses nearby persistent spell effects and strips enemy buffs. It has a nine-talent tree (3 tiers × 3 choices) modifying cooldown/mana cost, adding per-effect mana refunds/explosive damage/spell reflection, and introducing an Antimagic Pulse (cursor-targeted instant sphere) or a persistent NullZone field. In multiplayer, the host runs all authoritative gameplay logic; the guest displays ghost visuals for host-cast Dispel impacts (`tick_ghost_dispel_impacts`) and is supposed to forward its own impacts to the host via `forward_dispel_impacts_to_host`, but that forwarding path contains a critical query bug that silently drops all guest-cast dispels.

The module was refactored in "Phase 14" into `casting/` (input handling + projectile spawn + `move_dispel_projectiles`) and `bolt.rs` (impacts, null zones, ghost tick, MP forwarding, and all shared dispel helpers also used by the `dispeller` unit). `systems.rs` is now just a glob re-export hub for both.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| D-01 | TypeContract | `bolt.rs:606-613` | Critical | M | `forward_dispel_impacts_to_host` queries `spell_effects` with `&NetworkedSpellEffect`, but ghost spell effects on the guest are tagged with `GhostSpellEffect` + `NetworkEntityId` only — they never receive `NetworkedSpellEffect`. The query returns zero results on the guest, so guest-cast Dispel never forwards to the host and never suppresses any of the host's persistent spell effects. | Add `NetworkedSpellEffect { kind }` to ghost spell effects when spawning them in `guest_visuals/ghost_effect_spawn.rs`, or change the forwarding query to join on `NetworkEntityId` + `GhostSpellEffect` directly. |
| D-02 | Performance | `bolt.rs:128-132` | Medium | S | `collect_dispellable_effects` allocates a fresh `Vec<_>` inside the per-impact loop. While there is usually only 1 active impact, the Vec is rebuilt every frame for each active impact, iterating all networked spell effects each time. The same pattern repeats in `update_null_zones` (bolt.rs:408). | Hoist the Vec allocation outside the impacts loop (collect once before the `for … in &mut impacts` block, then pass `&all_dispellable` in). Same fix for `update_null_zones`. |
| D-03 | ArchitecturalDecay | `bolt.rs:1` | Medium | M | `bolt.rs` is 872 LOC and mixes five distinct concerns: `update_dispel_impacts` (gameplay), `update_null_zones` (gameplay), `tick_ghost_dispel_impacts` (MP visual), `forward_dispel_impacts_to_host` (MP network), plus six shared helpers (`suppress_spell_effects_in_radius`, `collect_dispellable_effects`, `remove_mind_control_in_radius`, `strip_spell_shields_in_radius`, `spell_edge_distance`, `despawn_spell_effect`). The project convention caps non-cohesive files at ~300 LOC. | Split into `impacts.rs` (update_dispel_impacts + update_null_zones), `multiplayer.rs` (tick_ghost_dispel_impacts + forward_dispel_impacts_to_host + DispelForwarded), and `suppress.rs` (the six shared helpers). The glob re-export in `systems.rs` continues to work with no downstream breakage. |
| D-04 | TypeContract | `bolt.rs:401` | Low | S | `update_null_zones` fades the NullZone alpha using `life_frac = time_remaining / constants::NULL_ZONE_DURATION`, but `time_remaining` is initialized to `NULL_ZONE_DURATION * scorched_mult` (bolt.rs:348). With Scorched Earth active (mult = 3.0), the zone starts with 30s remaining but the denominator is fixed at 10s, giving an initial `life_frac` of 3.0 and an alpha of 0.45 — 3× the intended maximum of 0.15. The zone stays over-bright for the first 20 of its 30 seconds. | Store the total duration at spawn time in `NullZone` (add `total_duration: f32` field) and use that as the denominator: `life_frac = time_remaining / zone.total_duration`. |
| D-05 | ConsistencyRot | `bolt.rs:81,287` | Low | S | `Petrified` is referenced via full inline crate path (`crate::game::units::components::Petrified`) in both the query tuple (line 81) and the removal call (line 287), while all other `units::components` types are cleanly imported at the top of the file (lines 9–13). | Add `Petrified` to the existing `use crate::game::units::components::{…}` import block. |
| D-06 | ArchitecturalDecay | `systems.rs:1-4` | Low | S | `systems.rs` is a glob re-export hub (`pub use super::bolt::*; pub use super::casting::*`) left over from the Phase 14 split. External callers like `dispeller/systems/channel.rs` import via `systems::` while `host_systems/dispel_receive.rs` imports directly from `bolt::`, creating two access paths to the same symbols. | Decide on one canonical access path. Either remove `systems.rs` and have all callers import from `bolt` and `casting` directly, or enforce that all external callers go through `systems` and fix the `bolt::` direct access in `dispel_receive.rs`. |
| D-07 | ConsistencyRot | `plugin.rs:31-33` | Low | S | `update_null_zones` is guarded by `is_gameplay_running` while every other system in the plugin uses `is_spell_effects_active`. The asymmetry is correct but unexplained — a future maintainer may "fix" it to match the others. | Add a one-line comment: `// NullZone is host-only gameplay simulation, not a guest visual — use is_gameplay_running, not is_spell_effects_active`. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Split Into |
|------|-----|--------|---------------------|
| `bolt.rs` | 872 | false | Five separate concerns (gameplay impacts, null zone, ghost visual tick, MP forwarding, shared helpers). Propose: `impacts.rs`, `null_zone.rs`, `multiplayer.rs`, `suppress.rs`. |

---

### Looks Bad But Is Actually Fine

- **`update_dispel_impacts` is `Without<GhostSpellEffect>` while running under `is_spell_effects_active` (which runs on the guest).** Looks like the guest runs gameplay logic — it does not. The ghost filter means the system's impacts query is empty on the guest (guest has no local non-ghost DispelImpact), so it's a no-op there.
- **`move_dispel_projectiles` has no ghost filter.** The ghost dispel projectile on the guest uses `GhostSpellProjectile` (not `DispelProjectile`), so the system's query never matches ghost projectiles. Safe.
- **`update_null_zones` lacks `Without<GhostSpellEffect>`.** NullZone is never added to the snapshot and never ghosted; there are no guest-side NullZone entities, so the filter is unnecessary.
- **`despawn_spell_effect` is `pub(crate)` but only called internally.** The broader visibility is a forward-looking choice for potential future callers from host_systems; that is fine.
- **`receive_dispel_messages` in `host_systems.rs` does a raw `try_despawn()` instead of calling `despawn_spell_effect`.** For the host-authoritative path (guest-forwarded dispels), Wall of Stone sinking and obstacle removal are not needed because the host's own `update_dispel_impacts` handles those correctly when the host processes its own impacts. Once D-01 is fixed, this path will need upgrading.
- **Large 12-field unit_query tuple in `update_dispel_impacts`.** Necessary to avoid a second query for each of the many conditional talent branches in a single system pass. The `#[allow(clippy::type_complexity)]` annotation is present and justified.
- **`DispelTalentParams` reconstructed from `Has<T>` booleans when projectile detonates.** Looks roundabout but is correct — the booleans are transferred into `DispelTalentParams` just to call `insert_talent_markers`. The shared helper keeps spawning logic DRY between the projectile-detonation and direct-cast (Antimagic Pulse) paths.
- **`HealingPlumeZone` has a mana cost entry in `spell_effect_mana_cost` (constants.rs:90) but is excluded from `is_dispellable`.** The mana cost entry is dead code for now but harmless — it documents the cost in case the policy changes, and the `is_dispellable` exclusion prevents it from ever being reached.
- **`damage_targets: Vec<(Entity, f32, bool)>` looks like a per-frame allocation inside the impact loop.** It is declared once outside the outer `for … in &mut impacts` loop (bolt.rs:90) and only `.clear()`ed inside the inner per-effect loop. No repeated allocation.

---

### Open Questions

1. **D-01 fix scope:** Should `forward_dispel_impacts_to_host` use the `GhostSpellEffect` + `NetworkEntityId` query (requiring no change to guest_visuals), or should `NetworkedSpellEffect` be added to ghost entities in `guest_visuals/ghost_effect_spawn.rs`? The latter is more consistent but has broader blast radius.
2. `HealingPlumeZone` is excluded from `is_dispellable` (bolt.rs:693). Is this intentional gameplay design (friendly healing fields should not be dispellable by the wizard) or an oversight?
3. `receive_dispel_messages` (host_systems) raw-despawns spell effects without the Wall of Stone sinking animation or obstacle removal. Once D-01 is fixed and guest dispels reach the host, this will silently leave pathfinding stale for Wall of Stone and obstacle-tagged effects. Should `despawn_spell_effect` be plumbed into the message handler as part of the D-01 fix?
