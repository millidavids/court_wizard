## boss-lich

**Scope:** `src/game/units/boss/lich/` — 13 files, ~1 040 unique LOC (deduplicated).

---

### Mental model

The Lich is a three-phase boss that spawns after all normal attacker waves are cleared. Phase 0 (Approaching) drives it from a tunnel spawn toward the staging zone via a bespoke steering loop; Phase 1 (Summoning) parks it stationary and periodically raises corpses / spawns fresh undead while soul power accumulates from undead kills; Phase 2 (Combat) releases it to chase defenders with `FingerOfDeath` beams. The module is well feature-sliced: `spawn.rs`, `combat/{movement,targeting,soul_power,beam,animation}.rs`, `components.rs`, `constants.rs`, `resources.rs`. A thin `systems.rs` re-export hub feeds the single `plugin.rs`. Most mechanics reuse shared helpers (`boss/utils.rs`, `units/systems/resurrect_corpse_as_infantry`, `finger_of_death` spell visuals). The primary concerns are: a wrong-direction comment on a constant, in-function steering duplication that skips the project-wide movement helper, and visibility markers that are broader than the access pattern requires.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| L-01 | DocDrift | `constants.rs:78` | High | S | `LICH_DAMAGE_MULTIPLIER` comment says "negative = takes less damage from non-spell sources" but `DamageMultiplier` is applied as the attacker's outgoing modifier (`damage * (1 + d.0)` in `combat_systems/melee/combat.rs:351`). A value of `-0.5` means the Lich deals 50% less melee damage — not that it takes less damage. The separate `LICH_MELEE_DAMAGE_REDUCTION` already handles incoming damage resistance. | Correct the comment to: `/// Outgoing melee damage penalty — the Lich deals 50% less melee damage.` |
| L-02 | ArchitecturalDecay | `combat/movement.rs:39–83` | Medium | S | The `Approaching` and `Combat` arms of `lich_movement` each inline the same steering + damping idiom (target velocity, `STEERING_FORCE` clamp, `VELOCITY_DAMPING.powf` damping). The project-wide equivalent lives in `units/systems/movement_helpers.rs:224–264`. The lich version also skips the speed-scale correction present in the shared helper (line 235–239 there), which means at high `GLOBAL_SPEED_MULTIPLIER` the lich under-accelerates compared to normal units. | Extract a small private `fn apply_steering(...)` helper within `movement.rs` for both arms, or adapt the lich to call the shared movement helper if it accepts compatible params. |
| L-03 | ConsistencyRot | `combat/targeting.rs:110` | Medium | S | Beam target selection uses `(kill_stats.elapsed_time * 1000.0) as usize % eligible.len()` — a floating-point time cast — instead of `GameRng`. The index changes every frame until `fod.target` is set, so the target is deterministic only by accident (whichever frame the cooldown happens to expire). If a defender dies the same frame the cast resolves, `eligible` shrinks and a different entity is chosen. All other randomised boss behaviours use `GameRng`. | Replace with a single `game_rng.0.gen_range(0..eligible.len())` call (add `ResMut<GameRng>` param). |
| L-04 | ConsistencyRot | `combat/animation.rs:11,23,35,73`; `combat/beam.rs:29,188`; `combat/soul_power.rs:12,33`; `combat/targeting.rs:12,55`; `combat/movement.rs:13` | Low | S | All public combat system functions are marked `pub(crate)`. They are only ever used within the `lich::` subtree (re-exported by `systems.rs` with `pub(super)`, consumed by `plugin.rs`). `pub(crate)` widens visibility crate-wide unnecessarily. | Change all `pub(crate) fn` in `combat/*.rs` to `pub(super)`, matching the re-export visibility in `systems.rs`. |
| L-05 | DocDrift | `combat/soul_power.rs:8–9` | Low | S | The `track_soul_power` function carries the doc comment verbatim copied from `check_lich_spawn` ("Checks if it's time to spawn the Lich mid-game. The Lich spawns as an extra wave…"). | Replace with an accurate comment: `/// Accumulates soul power from undead kills during Phase 1 (Summoning).` |
| L-06 | TypeContract | `components.rs:112–114` | Low | S | `LichFingerOfDeath::new()` is a free constructor but the type has no `Default` impl. Any future reflection, inspector, or `..Default::default()` usage would require a separate addition. | Add `impl Default for LichFingerOfDeath { fn default() -> Self { Self::new() } }` |
| L-07 | ArchitecturalDecay | `spawn.rs:250–293` | Low | M | `spawn_fresh_undead` is a local private helper that manually assembles a unit bundle. It diverges from `resurrect_corpse_as_infantry` in component set (missing `FlockingModifier`, `OnGameplayScreen`), and from the normal infantry spawn helper. A future change to the base undead bundle will silently miss this path. | Extract a shared `spawn_undead_infantry` function in `units/undead/` that both the lich and the `raise_the_dead` spell can call, eliminating the divergence. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|--------------------------|
| `spawn.rs` | 294 | true | Just under the 300-line limit; every line belongs to spawn-related concerns (check, spawn, approach, summoning, raise-dead resolution). No split needed. |
| `combat/beam.rs` | 254 | true | Single cohesive concern: FoD beam cast → resolve → damage loop. Well under 300 LOC. |

---

### Looks bad but is actually fine

- **`check_lich_spawn` with `Local<f32>` debounce** — using a `Local` accumulator instead of a timer component looks ad-hoc, but is correct because the Lich does not yet exist when the debounce runs; there is no entity to attach a component to.
- **`king_query.iter().next()` in `targeting.rs:76` and `beam.rs:45`** — using `.iter().next()` instead of `.single()` is intentional: the King can be dead (no entity) or absent in edge cases, and `.single()` would log a warning. The optional handling is correct.
- **`FlockingModifier::new(0.0, 0.0, 0.0)` on Lich spawn** — all bosses (hags, ogre, ray, dark_mage) follow this pattern to opt out of flocking while keeping the component present for query compatibility.
- **`spawn_fresh_undead` missing `FlockingModifier`** — `resurrect_corpse_as_infantry` also omits `FlockingModifier`, so freshly-spawned and resurrected undead are consistent. The flocking system skips units without the component.
- **`pub(crate)` on `Lich`, `LichSpawnPending`, `SoulPower`, `LichPhase`** — these are read by `ui/in_game/bars/`, `win_lose_systems.rs`, and `achievements/`, so crate-wide visibility is genuinely needed.
- **Three separate `.insert(...)` calls in `spawn_lich`** — Bevy has a bundle component limit; splitting across multiple `.insert()` calls is idiomatic when exceeding ~15 components in a single bundle.
- **`INITIAL_DEFENDER_COUNT` used in `beam.rs:230`** — compile-time game constant; correct to read directly rather than from a runtime resource.
- **`systems.rs` as a pure re-export hub** — the two-line file looks vestigial but cleanly separates the `plugin.rs` import graph from the internal submodule layout.

---

### Open questions

1. Is the Lich intended to be authoritative on the multiplayer host only, or can it appear in co-op sessions? `is_gameplay_running` already gates it to host-only in MP, but neither the Lich's spawn state nor its components carry `GhostEntity`/`GhostSpellEffect` guards — if a guest somehow ends up with a Lich (via future changes), gameplay systems would run incorrectly.
2. `LICH_DAMAGE_MULTIPLIER = -0.5` — if this is intended to reduce the Lich's outgoing melee damage, should it be removed entirely? The Lich is a spell-caster boss; its melee DPS is never the primary threat. A value of `0.0` (brute spawn pattern) may be cleaner.
3. `spawn_fresh_undead` does not insert `OnGameplayScreen`. Is cleanup of these entities handled by the team-based despawn path rather than the marker-based path?
