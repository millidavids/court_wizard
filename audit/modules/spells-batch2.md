## spells-batch2

**Scope:** `src/game/units/wizard/spells/black_hole/`, `raise_the_dead/`, `plague_wind/`, `spike_growth/`

---

### Mental Model

Four complex multi-talent spells sharing a common structure: a `casting.rs` (input + indicator + CastingState machine), a `components.rs` (zone/effect component + talent-params struct), a `constants.rs`, and sub-modules for effects. Each uses `compute_talent_params` to materialise talent flags at cast time into a `*TalentParams` struct stored on the effect entity and read each frame. Multiplayer paths diverge: damage/lifecycle systems guarded by `is_spell_effects_active` run on **both peers**; only systems guarded by `is_gameplay_running` are host-exclusive. Ghost-spell filtering (`Without<GhostSpellEffect>`) is applied inconsistently across the four spells.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B2-01 | ErrorObservability | `raise_the_dead/effects.rs:164` | Low | S | `let _t = time.elapsed_secs();` is fetched but never used in `handle_undead_detonation`. Dead code left from an earlier draft. | Remove the `let _t = ...` line. |
| B2-02 | DocDrift | `black_hole/gravity/accretion.rs:35-40` | Medium | S | The comment on `apply_black_hole_damage` states the system is "gated to the host in MP (it lives under `MovementCalculationSet` via the plugin's chain)." In the plugin, only `apply_gravitational_forces` and `apply_corpse_gravity_and_despawn` are added to `MovementCalculationSet`; `apply_black_hole_damage` is not. The system runs on both peers via `is_spell_effects_active`. The premise of the ghost-filter justification is therefore factually wrong. | Correct the comment: the system runs on both peers; it is intentionally unfiltered because `Health` is CRDT-merged and damage is idempotent per peer. Remove the false `MovementCalculationSet` claim. |
| B2-03 | ConsistencyRot | `plague_wind/cloud/spawn.rs:11` and `plague_wind/cloud/damage.rs:16` | Medium | S | `fn horizontal_distance` is defined twice with identical bodies in two sibling files in the same `cloud/` sub-module. The shared utility `xz_distance` already exists at `spells/utils/spell_math.rs` and is already used by `spike_growth`. | Remove both local definitions and use `crate::game::units::wizard::spells::utils::xz_distance` in both files. |
| B2-04 | Security | `plague_wind/cloud/spawn.rs:60-106` | High | M | `spawn_pandemic_clouds` is guarded only by `is_spell_effects_active` (runs on **both** host and guest). Its `clouds` query has no `Without<GhostSpellEffect>` filter. In MP, the guest holds a `GhostSpellEffect`-tagged copy of every host-cast cloud. When a dying unit is near that ghost cloud, the guest spawns a local child cloud (not a ghost, not authoritative), creating duplicate effects that fire `ObstacleChanged` events on the guest and deal damage from a non-authoritative source. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the `clouds` query, mirroring the pattern in `move_plague_wind_cloud` and `apply_plague_wind_damage`. |
| B2-05 | Security | `spike_growth/systems/damage.rs:22` | High | S | `apply_spike_growth_damage` is guarded by `is_spell_effects_active` (runs on both peers). Its `zones` query has no `Without<GhostSpellEffect>` filter. In MP, the guest has ghost copies of all host-cast `SpikeGrowthZone` entities (each tagged with `NetworkedSpellEffect { kind: SpikeGrowthZone }`). The guest would apply zone damage, insert `ZonePresenceTracker`/`SpikeGrowthLingeringPoison` on ghost entities, and emit spurious talent progress increments from the wrong peer. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the `zones` query, matching the pattern in `apply_plague_wind_damage`. |
| B2-06 | Security | `spike_growth/systems/zone_vfx.rs:44` | High | S | `spike_storm_volley` runs on both peers (`is_spell_effects_active`) and queries `zones` without a `GhostSpellEffect` filter. On the guest, every ghost zone with `spike_storm = true` periodically spawns local `SpikeStormProjectile` entities. These projectiles are not `NetworkedSpellEffect` entities so they exist only on the guest and deal redundant damage to ghost units via `update_spike_storm_projectiles`. | Add `Without<GhostSpellEffect>` to the `zones` query in `spike_storm_volley`; also consider gating `update_spike_storm_projectiles` to `is_gameplay_running` (host-only) if Spike Storm projectiles are intentionally not networked. |
| B2-07 | ArchitecturalDecay | `plague_wind/casting.rs` (392 LOC) | Medium | M | File exceeds the 300-line limit. It bundles the outer system, the inner `plague_wind_casting_logic` state machine, `arrow_transform`, and the `cleanup_indicator` helper. | Split into `casting.rs` (outer system + casting logic, ~230 lines) and `indicator.rs` (`arrow_transform` + `cleanup_indicator`). |
| B2-08 | ArchitecturalDecay | `raise_the_dead/casting/input_casting.rs` (340 LOC) | Medium | M | File exceeds the 300-line limit. Contains `compute_talent_params`, the outer system, and the 140-line `raise_the_dead_casting_logic` inner function. | Move `raise_the_dead_casting_logic` to the existing `raising.rs`; keep `input_casting.rs` as the thin outer system + `compute_talent_params`. |
| B2-09 | ArchitecturalDecay | `black_hole/casting.rs` (328 LOC) | Low | M | Just over the limit. Contains `compute_talent_params`, `spawn_black_hole`, the outer system, and `black_hole_casting_logic`. | Move `compute_talent_params` and `black_hole_casting_logic` to a new `casting_logic.rs`; keep `spawn_black_hole` and the outer system in `casting.rs`. |
| B2-10 | ConsistencyRot | `black_hole/constants.rs:134` | Low | S | `TWIN_STARS_MANA_MULT = 1.0` is a constant whose value is a no-op (multiplying by 1.0 changes nothing). The casting code computes `total_mana_cost = MANA_COST * mana_mult` where `mana_mult` is either `TWIN_STARS_MANA_MULT` or `1.0`. Both branches are identical. | Either set this to a non-trivial multiplier if the design intent is for Twin Stars to cost more, or remove the constant and the `mana_mult` indirection. |
| B2-11 | Performance | `raise_the_dead/effects.rs:243` and `raise_the_dead/effects.rs:295` | Medium | S | Both `handle_perpetual_unrest` and `tick_revenant_raise` call `compute_talent_params(active_talents.as_deref())` on every frame they execute (i.e., every frame a corpse exists or a Revenant is alive). This reconstructs the talent param struct from scratch via `ActiveTalents` lookups each frame for persistent ambient effects. | Store `talent_params` on the `PerpetualUnrest` and `RevenantLord` components at insert time (same way `SpikeGrowthTalentParams` is embedded in `SpikeGrowthZone`). |
| B2-12 | TypeContract | `raise_the_dead/casting/raising.rs:67-78` | Low | S | `find_nearest_corpse` calls `target_pos.distance(a.1.translation)` twice per candidate: once in the `filter` closure and again inside `min_by`. The distance is computed twice for every candidate in range. | Compute distance once with `filter_map` returning `(entity, translation, dist)`, then `min_by` on the cached value. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Split Proposal |
|------|-----|--------|--------------------------|
| `plague_wind/casting.rs` | 392 | No | `casting.rs` (outer system + state machine) + `indicator.rs` (`arrow_transform` + `cleanup_indicator`) |
| `raise_the_dead/casting/input_casting.rs` | 340 | No | Move `raise_the_dead_casting_logic` to `raising.rs`; keep outer system + `compute_talent_params` here |
| `black_hole/casting.rs` | 328 | No | `casting.rs` (outer system + `spawn_black_hole`) + `casting_logic.rs` (`compute_talent_params` + `black_hole_casting_logic`) |
| `raise_the_dead/effects.rs` | 327 | No | Optional split: `plague_bearer.rs`, `detonation.rs`, `revenant.rs` — low urgency, only if the file grows |
| `spike_growth/systems/damage.rs` | 278 | Yes | Under 300; three cohesive damage systems |
| `black_hole/gravity/accretion.rs` | 245 | Yes | Under 300 |

---

### Looks Bad But Is Actually Fine

- **`apply_black_hole_damage` has no `Without<GhostSpellEffect>` filter on black holes** — the comment's *rationale* is wrong (not MovementCalculationSet-gated), but the *outcome* is intentional: both peers process damage on all black holes; `Health` is CRDT-merged so double writes are harmless. The ghost filter on `despawn_expired_black_holes` correctly gates the authoritative despawn to the host only.
- **`raise_the_dead` effect systems have no `Without<GhostEntity>` filter** — these systems use `is_gameplay_running` which is host-exclusive in MP. The guest never runs them.
- **Tuple parameter bundling in `handle_raise_the_dead_casting`** (`cursor_resources`, `cast_ctx`, `mp_ctx`, `talents_and_progress`) — idiomatic workaround for Bevy's system parameter limit; `#[allow(clippy::too_many_arguments)]` is present per project convention.
- **`partial_cmp(...).unwrap_or(Ordering::Equal)` in float sorting** (`find_nearest_corpse`, `spike_storm_volley`) — NaN-safe float comparison idiom, not a real panic risk.
- **`input.cursor_pos.unwrap_or(wizard_pos)` in `plague_wind/casting.rs:273`** — called inside a block that only executes once `is_complete(cast_time)` is true, where cursor_pos is expected to be Some; the fallback to `wizard_pos` provides a safe default.
- **`compute_talent_params` defined separately in each spell** — this function is deliberately spell-specific (it reads `Spell::BlackHole`, `Spell::RaiseTheDead`, etc. for tier indexing). It cannot be unified without a generic abstraction that would be more complex than the current repetition. This is by-design.

---

### Open Questions

1. **Spike Storm projectiles in MP (B2-06):** Are `SpikeStormProjectile` entities intentionally local (no `NetworkedSpellEffect`) to avoid bandwidth cost? If yes, gating `spike_storm_volley` and `update_spike_storm_projectiles` to `is_gameplay_running` is the correct fix.
2. **`TWIN_STARS_MANA_MULT = 1.0` (B2-10):** Was this intended to be a cost multiplier (e.g., 1.5×) that was never set, or is the current 1.0 a deliberate design choice to give Twin Stars no extra mana cost?
3. **Pandemic child clouds in MP (B2-04):** After the ghost-filter fix, no child clouds will spawn on the guest. The host spawns them correctly. Are pandemic child clouds expected to be visible on the guest (requiring propagation via snapshot), or is the current behaviour (guest sees no child clouds) acceptable?
