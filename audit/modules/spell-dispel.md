## spell-dispel

**Scope:** `src/game/units/wizard/spells/dispel/`

---

### Mental Model

The Dispel spell fires a straight-line projectile that detonates on ground contact, spawning an expanding DispelImpact sphere that suppresses nearby persistent spell effects and strips enemy buffs. It has a nine-talent tree (3 tiers × 3 choices) modifying cooldown/mana cost, adding per-effect mana refunds/explosive damage/spell reflection, and introducing an Antimagic Pulse (cursor-targeted instant sphere) or a persistent NullZone field. In multiplayer, the host runs all authoritative gameplay logic; the guest displays ghost visuals for host-cast Dispel impacts (`tick_ghost_dispel_impacts`) and is supposed to forward its own impacts to the host via `forward_dispel_impacts_to_host`, but that forwarding path contains a critical query bug that silently drops all guest-cast dispels.

The module was refactored in "Phase 14" into `casting.rs` (input handling + projectile spawn + `move_dispel_projectiles`) and `bolt.rs` (impacts, null zones, ghost tick, MP forwarding, and all shared dispel helpers also used by the `dispeller` unit). `systems.rs` is now just a glob re-export hub for both.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| D-01 | TypeContract | `bolt.rs:606-613` | Critical | M | `forward_dispel_impacts_to_host` queries `spell_effects` with `&NetworkedSpellEffect`, but ghost spell effects on the guest are tagged with `GhostSpellEffect` + `NetworkEntityId` only — they never receive `NetworkedSpellEffect`. The query returns zero results on the guest, so guest-cast Dispel never forwards to the host and never suppresses any of the host's persistent spell effects. | Add `NetworkedSpellEffect { kind }` to ghost spell effects when spawning them in `guest_visuals.rs`, or change the forwarding query to join on `NetworkEntityId` + `GhostSpellEffect` directly. |
| D-02 | Performance | `bolt.rs:128-132` | Medium | S | `collect_dispellable_effects` allocates a fresh `Vec<_>` inside the per-impact loop. While there is usually only 1 active impact, this Vec is rebuild every frame for each active impact, iterating all networked spell effects each time. | Hoist the Vec allocation outside the impacts loop (collect once before the `for … in &mut impacts` block, then pass `&all_dispellable` in). |
| D-03 | ArchitecturalDecay | `bolt.rs:1` | Medium | M | `bolt.rs` is 871 LOC and mixes five distinct concerns: `update_dispel_impacts` (gameplay), `update_null_zones` (gameplay), `tick_ghost_dispel_impacts` (MP visual), `forward_dispel_impacts_to_host` (MP network), plus six shared helpers (`suppress_spell_effects_in_radius`, `collect_dispellable_effects`, `remove_mind_control_in_radius`, `strip_spell_shields_in_radius`, `spell_edge_distance`, `despawn_spell_effect`). The project convention caps non-cohesive files at 300 LOC. | Split into `impacts.rs` (update_dispel_impacts + update_null_zones), `multiplayer.rs` (tick_ghost_dispel_impacts + forward_dispel_impacts_to_host + DispelForwarded), and `suppress.rs` (the six shared helpers). The glob re-export in `systems.rs` continues to work with no downstream breakage. |
| D-04 | ConsistencyRot | `bolt.rs:81,287` | Low | S | `Petrified` is referenced via full inline crate path (`crate::game::units::components::Petrified`) in both the query tuple (line 81) and the removal call (line 287), while all other `units::components` types are cleanly imported at the top of the file (lines 9–13). | Add `Petrified` to the existing `use crate::game::units::components::{…}` block. |
| D-05 | ArchitecturalDecay | `systems.rs:1-4` | Low | S | `systems.rs` is a glob re-export hub (`pub use super::bolt::*; pub use super::casting::*`) left over from the Phase 14 split. This glob-re-export pattern means external callers (e.g. `dispeller/systems.rs`) import from a module whose internals change silently. It also makes `bolt` and `casting` `pub(crate)` even though they are consumed only through this hub. | Either make `bolt` and `casting` plain `mod` (private) and keep the hub, or remove `systems.rs` and have the plugin import from `bolt` and `casting` directly. Keep the public API surface explicit. |
| D-06 | ConsistencyRot | `plugin.rs:31-33` | Low | S | `update_null_zones` is guarded by `is_gameplay_running` while every other system in the plugin uses `is_spell_effects_active`. Both are intentional (null zone is host-only gameplay simulation, not a guest-side visual), but there is no comment explaining the deliberate asymmetry. | Add a one-line comment like `// host-only: NullZone is a gameplay simulation, not a ghost visual` to prevent future maintainers from "fixing" the condition. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Split Into |
|------|-----|--------|---------------------|
| `bolt.rs` | 871 | false | Five separate concerns (gameplay impacts, null zone, ghost visual tick, MP forwarding, shared helpers). Propose: `impacts.rs`, `null_zone.rs`, `mp_dispel.rs`, `suppress.rs`. |
| `casting.rs` | 340 | false | Slightly over 300; contains talent param computation, wizard cast handler, projectile spawn, and projectile movement. Borderline — could extract `spawn.rs` for the two spawn helpers if desired. |

---

### Looks Bad But Is Actually Fine

- **`update_dispel_impacts` is `Without<GhostSpellEffect>` while running under `is_spell_effects_active` (which runs on the guest).** Looks like the guest runs gameplay logic — it does not. The ghost filter means the system's impacts query is empty on the guest (guest has no local non-ghost DispelImpact), so it's effectively a no-op there.
- **`move_dispel_projectiles` has no ghost filter.** The ghost dispel projectile on the guest uses `GhostSpellProjectile` (not `DispelProjectile`), so the system's query never matches ghost projectiles. Safe.
- **`update_null_zones` lacks `Without<GhostSpellEffect>`.** NullZone is never added to the snapshot and never ghosted; there are no guest-side NullZone entities, so the filter is unnecessary.
- **`despawn_spell_effect` is `pub(crate)` but only called internally.** It is used inside `suppress_spell_effects_in_radius` (bolt.rs:489) and not currently called from outside the module. The visibility is a forward-looking choice for future callers (e.g. `host_systems` could use it); that's fine.
- **`receive_dispel_messages` in `host_systems.rs` does a raw `try_despawn()` instead of calling `despawn_spell_effect`.** For the host-authoritative path (guest-forwarded dispels), Wall of Stone sinking and obstacle removal are not needed because the host's own `update_dispel_impacts` handles those correctly when the host processes its own impacts. The raw despawn in `receive_dispel_messages` only fires for guest-issued messages, which currently never arrive (D-01); when D-01 is fixed, this will need to be upgraded to use `despawn_spell_effect`.
- **Large 12-field unit_query tuple in `update_dispel_impacts`.** Necessary to avoid a second query for each of the many conditional talent branches in a single system pass. The `#[allow(clippy::type_complexity)]` annotation is present and justified.
- **`DispelTalentParams` reconstructed from `Has<T>` booleans when projectile detonates.** Looks roundabout but is correct — `Has<T>` booleans transferred into `DispelTalentParams` just to call `insert_talent_markers`. The shared helper keeps spawning logic DRY between the projectile-detonation and direct-cast (Antimagic Pulse) paths.

---

### Open Questions

1. **D-01 fix scope:** Should `forward_dispel_impacts_to_host` use the `GhostSpellEffect` + `NetworkEntityId` query (requiring no change to guest_visuals), or should `NetworkedSpellEffect` be added to ghost entities in `guest_visuals.rs`? The latter is more consistent but has broader blast radius.
2. `HealingPlumeZone` is excluded from `is_dispellable` (bolt.rs:692). Is this intentional gameplay design (friendly healing fields should not be dispellable by the wizard) or an oversight?
3. `receive_dispel_messages` (host_systems.rs:945) raw-despawns spell effects without the Wall of Stone sinking animation or obstacle removal. Once D-01 is fixed and guest dispels reach the host, this will silently leave pathfinding stale for Wall of Stone + obstacle-tagged effects. Should `despawn_spell_effect` be plumbed into the message handler?
