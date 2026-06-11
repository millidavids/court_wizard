## spells-batch2

**Scope:** `black_hole/`, `raise_the_dead/`, `plague_wind/`, `spike_growth/`

---

### Mental Model

These are four of the most complex spells in the codebase. Each follows the same architectural skeleton: a `plugin.rs` wiring `run_if` guards, a `casting.rs` (or equivalent) for input → state-machine → spawn logic, and one or more effect files for per-frame gameplay systems. Talent parameters are always pre-computed at cast time into a `*TalentParams` struct that rides on the spell component for zero per-frame lookups.

**Black Hole** is the most physics-heavy: a gravity system pulls all live units and corpses each frame, separate damage/CC/visual systems run inside one `.chain()`, and the `GhostSpellEffect` model is correctly handled with extensive comments explaining why the gravitational force systems intentionally omit the ghost filter.

**Raise The Dead** is the most stateful: a cast-then-channel loop raises corpses at accelerating frequency; five distinct talent components (`PlagueBearerAura`, `CorpseMagnetActive`, `UndeadDetonation`, `PerpetualUnrest`, `RevenantLord`) each drive their own system. The guest path delegates raises via `RaiseCorpse` network messages.

**Plague Wind** uses a click-drag vector input mechanic to launch a moving toxic cloud. Three host-authoritative systems are correctly gated `Without<GhostSpellEffect>`; however `spawn_pandemic_clouds` is not, creating a potential guest-side phantom cloud spawn.

**Spike Growth** places a persistent hazard zone with the widest talent surface (slow, root, lingering poison, death-garden growth, spike storm volleys). None of its gameplay systems filter `Without<GhostSpellEffect>`, so the ghost zone on the guest independently ticks time, applies zero-damage slows, and eventually fires an `ObstacleChanged::Removed` event into the guest's pathfinding grid before the host has actually expired the zone.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B2-01 | Security/Multiplayer | `spike_growth/systems.rs:669` | High | S | `cleanup_spike_growth_zone` has no `Without<GhostSpellEffect>` filter. On the guest the ghost `SpikeGrowthZone` entity has its own `time_alive` incremented by `apply_spike_growth_damage` (line 287). When `time_alive >= duration`, the guest fires `ObstacleChanged::Removed` into its pathfinding grid and `try_despawn`s the ghost entity ahead of the host snapshot's authoritative cleanup. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the `zones` query on `cleanup_spike_growth_zone` — matching the pattern in `cleanup_plague_wind_cloud`. |
| B2-02 | Security/Multiplayer | `spike_growth/systems.rs:266` | High | S | `apply_spike_growth_damage` has no `Without<GhostSpellEffect>` filter. Ghost zone has `damage_per_tick=0` so no numeric damage is applied, but the system still mutates `zone.time_alive` (line 287) and inserts `SlowMovementModifier` / `ZonePresenceTracker` (from default-false talent params, so those branches skip) — and, critically, drives `time_alive` forward enabling the premature cleanup in B2-01. | Add `Without<GhostSpellEffect>` filter to the `zones` query, same as `move_plague_wind_cloud` and `apply_plague_wind_damage`. |
| B2-03 | Security/Multiplayer | `plague_wind/cloud.rs:329` | High | M | `spawn_pandemic_clouds` queries all `PlagueWindCloud` entities without filtering `Without<GhostSpellEffect>`. On the guest, ghost clouds have `pandemic=true` when the host cast with that talent. If a guest ghost unit's `Health` reaches zero inside a ghost cloud, the system spawns a new real (non-ghost) local cloud with `commands.spawn`, which then runs `apply_plague_wind_damage` / `cleanup_plague_wind_cloud` independently on the guest, applying unsynchronized damage and pathfinding updates. | Add `Without<GhostSpellEffect>` to the `clouds: Query<&PlagueWindCloud>` in `spawn_pandemic_clouds`, matching the approach used by `move_plague_wind_cloud` and `cleanup_plague_wind_cloud`. |
| B2-04 | ArchitecturalDecay | `spike_growth/systems.rs:1` (whole file) | Medium | M | `systems.rs` is 790 lines and mixes six distinct concerns: input/casting, zone damage + CC, lingering poison tick, death-garden growth, spike storm volley, spike storm projectile update, VFX ring emission, and spawn helpers. All other complex spells in scope split their casting from their effects. | Extract into `casting.rs` (lines 1–260), `damage.rs` (apply_spike_growth_damage, tick_lingering_poison), `effects.rs` (update_death_garden, spike_storm_volley, update_spike_storm_projectiles), `vfx.rs` (emit_spike_growth_rings), and `spawn.rs` (spawn_spike_growth_zone, spawn_minefield_zones). Keep `systems.rs` as a re-export hub like black_hole and raise_the_dead. |
| B2-05 | ArchitecturalDecay | `raise_the_dead/casting.rs:1` (whole file) | Medium | S | `casting.rs` is 577 lines. The bottom half (`find_nearest_corpse`, `raise_corpse_as_undead`, `resurrect_nearest_corpse`, `try_raise_or_forward`) is the "raising mechanics" layer, not casting input logic. `effects.rs` already imports from here (`use super::casting::{...}`). | Move `find_nearest_corpse`, `raise_corpse_as_undead`, `resurrect_nearest_corpse`, `try_raise_or_forward` to a new `raising.rs` sibling. `casting.rs` becomes ~200 lines (compute_talent_params + apply_talent_components + handle/logic). |
| B2-06 | ArchitecturalDecay | `black_hole/gravity.rs:1` (whole file) | Medium | S | `gravity.rs` is 446 lines and holds gravitational physics, damage, crushing pressure, dimensional rift, visuals update, and despawn/singularity. The name implies only gravity but the file owns all live-effect systems. | Split into `gravity.rs` (apply_gravitational_forces, apply_corpse_gravity_and_despawn, ~120 lines), `damage.rs` (apply_black_hole_damage, apply_crushing_pressure, apply_dimensional_rift, remove_units_from_black_hole), and `lifecycle.rs` (update_black_hole_visuals, despawn_expired_black_holes, cleanup_black_hole_sfx). |
| B2-07 | DocDrift | `raise_the_dead/effects.rs:29` | Low | S | Doc comment `/// Computes talent parameters from active talent selections.` is attached to `tick_plague_bearer_aura` — clearly copied from `casting.rs` when the file was split. | Replace with `/// Ticks the Plague Bearer aura: spawns smoke VFX and applies periodic poison damage to nearby living units.` |
| B2-08 | DocDrift | `plague_wind/cloud.rs:25` | Low | S | Doc comment `/// Computes talent parameters from the player's active talent selections.` is attached to `spawn_plague_cloud`. | Replace with `/// Spawns a plague wind cloud entity and registers it as a pathfinding obstacle.` |
| B2-09 | DocDrift | `raise_the_dead/casting.rs:259` | Low | S | Function-level doc says "With Revenant Lord talent, only one corpse is raised (no channeling phase)" but the code calls `casting_state.start_channeling()` unconditionally at the end of the cast completion branch — channeling still begins. Revenant Lord raises the first corpse normally, then `tick_revenant_raise` in effects handles passive auto-raise. | Update the doc: "With Revenant Lord, the first corpse is raised at cast completion; the Revenant Lord component then drives passive auto-raise from `tick_revenant_raise`. Channeling still proceeds normally." |
| B2-10 | ArchitecturalDecay | `black_hole/components.rs:57` | Low | S | `BlackHole.damage_type` is annotated `#[allow(dead_code)]` — the field is never read; `gravity.rs` hard-codes `DamageType::Force` at every damage call site. | Remove the field and its `DAMAGE_TYPE` constant from `constants.rs` (or move it to a `use`-only comment). |
| B2-11 | ArchitecturalDecay | `black_hole/constants.rs:134` | Low | S | `TWIN_STARS_MANA_MULT = 1.0` is a no-op multiplier (multiplying by 1.0 has no effect). It is used in casting but the const name misleads readers into thinking it adjusts cost. | Either tune it to a real value (e.g. 1.5 to match the "two black holes" theme) or remove the constant and inline the branch in `casting.rs` with a comment explaining it is intentionally unchanged. |
| B2-12 | ConsistencyRot | `raise_the_dead/effects.rs:87,211` | Low | S | `tick_plague_bearer_aura` and `handle_undead_detonation` use `apply_damage_to_unit` which bypasses `SpellShield` and team-vulnerability, while all other spell damage in this batch uses `apply_spell_damage_with_team`. The king's `SpellShield` therefore does not protect against plague aura or detonation damage. | Decide if this is intentional "plague is indiscriminate" design and add an explicit comment, or switch to `apply_spell_damage_with_team` with a `caster_team` parameter for consistency. |
| B2-13 | ArchitecturalDecay | `raise_the_dead/effects.rs:164` | Low | S | `handle_undead_detonation` injects `Res<Time>` and immediately assigns `let _t = time.elapsed_secs()` which is never used. This is dead code leftover from an earlier VFX pass. | Remove the `time: Res<Time>` parameter and the `let _t` assignment. |
| B2-14 | ArchitecturalDecay | `spike_growth/systems.rs:124` | Low | S | `handle_spike_growth_casting` destructures `talent_resources` as `(_talent_progress, ...)` — `BattleTalentProgress` is injected as `ResMut` but discarded. Talent progress tracking happens in `apply_spike_growth_damage` which injects it separately. | Remove `Option<ResMut<BattleTalentProgress>>` from the `talent_resources` tuple in `handle_spike_growth_casting` to avoid the unnecessary ResMut borrow. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|--------------------------|
| `spike_growth/systems.rs` | 790 | No | Six distinct concerns in one file. Split into: `casting.rs`, `damage.rs`, `effects.rs` (death garden + spike storm), `vfx.rs`, `spawn.rs`, `systems.rs` (re-export hub). |
| `raise_the_dead/casting.rs` | 577 | No | Casting input logic mixed with raising mechanics helpers. Split into `casting.rs` (~200 LOC) + `raising.rs` (~300 LOC). |
| `plague_wind/cloud.rs` | 435 | No | Cloud spawn + movement + 6 effect systems. Consider splitting: `movement.rs` (move + cleanup), `damage.rs` (apply_plague_wind_damage, cleanup_toxic_weakness, track_plague_carrier, apply_plague_carrier_dot), `pandemic.rs` (spawn_pandemic_clouds), `vfx.rs` (emit_plague_cloud_particles). Current size is tolerable but trending large. |
| `black_hole/gravity.rs` | 446 | No | Named "gravity" but owns all live-effect systems. Split into `gravity.rs`, `damage.rs`, `lifecycle.rs` as described in B2-06. |
| `plague_wind/casting.rs` | 392 | No | Borderline; all content is cohesive casting-flow logic. Acceptable at current size but watch for growth. |

---

### Looks Bad But Is Actually Fine

- **`apply_gravitational_forces` and `apply_corpse_gravity_and_despawn` omit `Without<GhostSpellEffect>`** — Both are host-authoritative gravity systems. The comments at lines 33–38 and 87–91 explicitly document that including guest-cast ghost black holes intentionally allows the host to apply gravity for guest spells. This is correct MP architecture.
- **`black_hole/casting.rs`: `audio_ctx` tuple parameter** — The three resources are bundled to free argument slots (Bevy 16-param limit). This is consistent with other casting files.
- **`raise_the_dead/casting.rs` lines 320, 367: `#[allow(clippy::needless_option_as_deref)]`** — The `connection: Option<&mut NetworkConnection>` is destructured from a tuple, making `.as_deref_mut()` genuinely needed to satisfy the function signature. The suppression is correct.
- **`compute_talent_params` duplicated across all 4 spells** — Each function returns a different `TalentParams` struct type. The structural similarity (match t1/t2/t3) is unavoidable given Rust's type system; a macro or trait could unify them but would add complexity with no runtime or safety gain. Not actionable given the 21-spell-wide pattern.
- **`_talent_progress` in raise_the_dead effects.rs `tick_revenant_raise` and `handle_perpetual_unrest`** calling `compute_talent_params(active_talents)` each frame — this re-reads the talent resource per invocation but is cheap (3 pattern matches on small integers). Not a hot-path perf concern.
- **`spawn_pandemic_clouds` using `Without<Corpse>` instead of `With<Health>` to find dying units** — This matches the game's pipeline where `Corpse` is only added by a dedicated post-combat system. Using `Added<Corpse>` would be cleaner but the current approach is correct given the ordering.
- **`PlagueWindCloud.smoke_spawn_timer` stored on the component** — This is a visual-only timer stored on a gameplay component. Not ideal architecturally, but it avoids an extra entity/resource and is consistent with how `SpikeGrowthZone.ring_timer` works.

---

### Open Questions

1. **Spike Growth talent sync gap**: `SpikeGrowthZone` sends only `base_radius` and `duration` to the guest; talent params (death_garden growth, spike_storm presence) are not packed into `flags`. Should the guest visualize death-garden radius growth? Currently it does not. Is this intentional or an oversight relative to `PlagueWindCloud` which does pack talent flags?
2. **Plague Carrier on ghost side**: `track_plague_carrier` reads all `PlagueWindCloud` entities (including ghosts) to decide if a unit is "still inside." On the guest, ghost cloud positions are snapshotted from the host, so the tracking diverges slightly on movement frames. Is there an accepted latency tolerance for this?
3. **`TWIN_STARS_MANA_MULT = 1.0`**: Is this intentionally a no-op (Twin Stars is already balanced by halved `TWIN_STARS_EFFECTIVENESS`) or a forgotten tuning value?
4. **Revenant Lord channeling**: The RaiseTheDead+Revenant Lord combination channels indefinitely (or until mana runs out), raising regular undead on each tick in addition to the Revenant's passive auto-raise. Was this intended, or should the RevEntity cast be a one-shot that skips the channeling phase?
