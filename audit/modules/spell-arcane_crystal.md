## spell-arcane_crystal

**Scope:** `src/game/units/wizard/spells/arcane_crystal/` — 8 files, 2,910 LOC total.

---

### Mental Model

The Arcane Crystal is the most complex spell in the codebase. It places a stationary entity that: (1) absorbs incoming spell projectiles/beams and re-emits scaled-down versions at random enemies; (2) auto-casts the last-absorbed spell on a timer while alive; (3) supports five mutually-exclusive tier-1/2/3 talent branches (Refined Facets, Wider Prism, Enduring Crystal, Overcharged Matrix, Resonance Cascade, Spell Echo, Crystal Network, Prismatic Explosion, Auto-Crystal turret).

The code was split from a monolith into `setup.rs`, `hits.rs`, `auto.rs`, `components.rs`, and `constants.rs` in a migration labelled "Phase 14". `systems.rs` is a thin re-export hub that lets `plugin.rs` call everything as `systems::fn_name`. Ghost gating (multiplayer) is applied consistently across `hits.rs` and most of `auto.rs`, but one gameplay system (`crystal_black_hole_interaction`) slipped through without the guard.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `setup.rs:1` | High | M | `setup.rs` is 750 LOC containing 7 distinct concerns: talent-param computation, frame-reset, casting helpers, spawn helpers, visual/lifetime, black-hole interaction, and range-limited despawn. The >300-LOC rule requires splitting unless a single cohesive concern. | Split into `casting.rs` (handle_arcane_crystal_casting + arcane_crystal_casting_logic, ~160 LOC), `spawn.rs` (spawn_crystal, spawn_permanent_crystal, spawn_crystal_entity + helpers, ~130 LOC), `visuals.rs` (update_crystal_visuals, cleanup_expired_*, clear_absorption_flags, ~200 LOC), and `helpers.rs` (talent params, shared math helpers ~80 LOC). |
| F2 | ArchitecturalDecay | `auto.rs:1` | High | L | `auto.rs` is 1039 LOC containing five distinct concerns: timer-based auto-cast dispatch, persistent auto-disintegrate beam management, the Auto-Crystal turret firing loop, Resonance Cascade burst, Crystal Network chaining, plus spawner helpers. Not a single match-on-enum or registry monolith. | Split into `auto_cast.rs` (~380 LOC), `turret.rs` (~70 LOC), `network.rs` (~190 LOC), `talents.rs` (~120 LOC), `spawn_helpers.rs` (~110 LOC). |
| F3 | Security | `setup.rs:639` | High | S | `crystal_black_hole_interaction` is registered in the **gameplay** system block (plugin.rs:59), whose comment explicitly guarantees "every crystal query is gated `Without<GhostSpellEffect>`". The function's query (`mut crystals: Query<..., &mut Transform>`) has no such filter. A guest's ghost copy of the host's crystal would be pulled toward a local black hole and potentially despawned locally, desynchronising crystal positions across peers. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the crystal query, mirroring the pattern on every other system in that plugin block. |
| F4 | ConsistencyRot | `auto.rs:45` `hits.rs:47` | Medium | S | `GhostSpellEffect` is referenced as a full inline path (`crate::game::multiplayer::components::GhostSpellEffect`) 10 times across `auto.rs` and `hits.rs` but is never imported at the top of either file. `setup.rs` already uses a proper `use` import for its multiplayer types. | Add `use crate::game::multiplayer::components::GhostSpellEffect;` to both files and replace the 10 inline paths. |
| F5 | ArchitecturalDecay | `hits.rs:220` `hits.rs:276` | Medium | S | `detect_beam_hits` inlines two Fisher-Yates shuffle blocks (lines 220-234 and 276-291) instead of calling the existing shared helpers `find_random_targets_in_range` / `find_random_enemies_in_range`. This creates four total Fisher-Yates shuffle implementations across the module (setup.rs:138, setup.rs:183, hits.rs:231, hits.rs:287). | Replace the two inline shuffle blocks in `detect_beam_hits` with calls to the existing `find_random_targets_in_range` helper. |
| F6 | ArchitecturalDecay | `auto.rs:333` `auto.rs:576` `hits.rs:342` | Medium | S | The forked-fan beam emission pattern (select offsets `[-angle, 0, +angle]` or `[0]`, compute rotated directions, spawn beams) is duplicated across three sites. The FoD absorption case (`hits.rs:342`) and the FoD auto-cast helper (`auto.rs:576`) are structurally identical. | Extract a shared `spawn_fod_beams_for_targets` helper that accepts `origin, targets, range, empowerment, damage_per_tick, talent_cfg, lifetime` and handles the fan pattern internally. |
| F7 | Performance | `components.rs:66` `hits.rs:69` `hits.rs:318` | Medium | S | `ArcaneCrystal.fod_beams_processed` and `explosions_processed` are `Vec<Entity>` that grow unboundedly for the crystal's lifetime. They are never pruned after tracked entities are despawned. The `.contains()` calls at hits.rs:69 and hits.rs:318 are O(n) linear scans. During a heavy fight with many fireballs, `explosions_processed` could accumulate many stale entries over a 25-second crystal lifetime. | Replace the Vecs with a frame-counter approach: store the last-seen `Entity` generation/id and a frame index. Or prune entries each frame by retaining only IDs that still match living entities. An O(1) `HashSet<Entity>` would also eliminate the linear scan. |
| F8 | ArchitecturalDecay | `systems.rs:1` | Low | S | `systems.rs` is a migration-artifact re-export hub labelled "Phase 14" in its doc comment. It glues `auto`, `hits`, and `setup` into one `systems::` namespace for `plugin.rs`. This indirection layer adds no semantics; the comment leaks internal history into the module surface. | Strip the "Phase 14" comment and, when F1/F2 splits are done, remove the file entirely and have `plugin.rs` import from the split files directly (or re-route through `mod.rs`). |
| F9 | ConsistencyRot | `plugin.rs:56` | Low | S | The gameplay system block (plugin.rs:56-71) is a flat tuple without `.chain()`. Six systems all take `&mut ArcaneCrystal` — Bevy serializes them via conflict detection but the execution order is non-deterministic. If two spell types hit the crystal in the same frame, `crystal.remembered_spell` is set by whichever system runs last. Currently benign but undocumented. | Either add `.chain()` to make the order deterministic and explicit, or add a comment noting that last-writer-wins is intentional. |

---

### Oversized Files

| File | LOC | Exempt | Reason | Split Into |
|------|-----|--------|--------|------------|
| `auto.rs` | 1039 | false | Contains 5+ distinct systems (auto-cast dispatch, auto-disintegrate management, turret firing, resonance burst, network chaining) plus two spawn helpers. Not a single-concern monolith. | `auto_cast.rs`, `turret.rs`, `network.rs`, `talents.rs`, `spawn_helpers.rs` |
| `setup.rs` | 750 | false | Contains casting, spawn, visual/lifetime, black-hole interaction, and range despawn — at least 5 distinct concerns. | `casting.rs`, `spawn.rs`, `visuals.rs`, `helpers.rs` |
| `hits.rs` | 664 | true | One system per absorbed spell type (fireball, disintegrate/FoD beam, meteor, magic missile, chain lightning). Repetitive match-per-spell-type structure is cohesive; each function is structurally isomorphic. Splitting by spell type yields 5 tiny files with no shared logic. | — |

---

### Looks Bad But Is Actually Fine

- **`crystal_data` Vec pre-collected every frame in `auto_cast_remembered_spell` (auto.rs:66):** Required to break the Bevy borrow conflict between the immutable `.iter()` pass and subsequent `get_mut()` calls. At most 3 elements. The `auto_disintegrate_beam.clone()` is `Option<(Vec<Entity>, Entity)>` — tiny.
- **`#[allow(clippy::too_many_arguments)]` on 13 functions:** Every one is a Bevy system or a constructor mapping 1:1 to struct fields. Idiomatic per CLAUDE.md.
- **`detect_beam_hits` complexity (~225 LOC, hits.rs:141):** The disintegrate case tracks live beam groups, replaces dead targets, and spawns new groups every frame. The complexity is inherent to the persistent-beam feature, not accidental.
- **`crystal_network_chain` early-returning when `just_absorbed` is false (auto.rs:871):** Correctly skips the O(n²) crystal-pair loop on the vast majority of frames. Not premature optimization.
- **`ArcaneCrystal` carrying 15 fields (components.rs:50):** All fields are tightly coupled to the main crystal systems. No independent system queries on a subset; splitting into sub-components would create artificial joins with no query benefit.
- **`cleanup_expired_crystal_beams` allocating `despawned: Vec<Entity>` (setup.rs:721):** Only populated when beams expire, and guarded by `if despawned.is_empty() { return; }`. Harmless.

---

### Open Questions

1. **`crystal_black_hole_interaction` ghost intent (F3):** Should a local black hole visually pull the ghost of the remote peer's crystal? If yes, move this system to the visual block. If no, add the `Without<GhostSpellEffect>` filter as described.
2. **`fod_beams_processed` / `explosions_processed` growth (F7):** Are FoD beams and fireball explosions numerous enough per crystal lifetime to make the linear scan observable? Only matters if many FoDs fire against one crystal.
3. **Gameplay block ordering (F9):** Is there a known case where two spell types can both hit the crystal in the same frame, causing `remembered_spell` to be set non-deterministically?


## Mental Model

The Arcane Crystal is the most complex spell in the codebase. The player places a crystal that passively absorbs 6 different spell types (Fireball, Disintegrate, Finger of Death, Meteor, Magic Missile, Chain Lightning) and re-emits scaled-down versions at random enemies. Three tiers of talents expand the crystal into: a burst counter (Resonance Cascade), a chained multi-crystal network (Crystal Network), a detonation on expiry (Prismatic Explosion), or a permanent auto-firing turret (Auto-Crystal).

The module was deliberately split from a monolith ("Phase 14" comment in `systems.rs`): casting and spawn live in `setup.rs` (750 LOC), per-spell hit detection in `hits.rs` (664 LOC), and timer-driven auto-cast + talent burst systems in `auto.rs` (1039 LOC). A 6-line `systems.rs` glob-re-exports all three into a flat namespace for `plugin.rs`. `components.rs` and `constants.rs` are clean and well-sized.

MP gating is thorough — every gameplay query carries `Without<GhostSpellEffect>`. All Update systems have `run_if` guards (either per-component or via the outer `is_spell_effects_active`).

---

## Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `auto.rs:1–1039` | High | M | `auto.rs` at 1039 LOC mixes 5 distinct concerns: timer-based auto-cast driver (`auto_cast_remembered_spell` + `handle_auto_disintegrate`), per-spell auto-cast helper functions (`auto_cast_magic_missiles` through `auto_cast_fod_beams`), the `spawn_crystal_mini_missile` spawn helper, `resonance_cascade_burst` talent system, `crystal_aoe_burst` shared helper, `auto_crystal_fire` turret system, and `crystal_network_chain` talent system. This violates the project's ≤300 LOC granularity rule. | Split into: `auto_cast.rs` (timer driver + per-spell helpers + `spawn_crystal_mini_missile`), `talents.rs` (resonance cascade burst + crystal network chain + `crystal_aoe_burst`), `turret.rs` (`auto_crystal_fire`). `spawn_crystal_disintegrate_beam` and `despawn_beam_group` can move to `beams.rs` alongside `detect_beam_hits` from `hits.rs`. |
| F2 | ArchitecturalDecay | `setup.rs:1–750` | High | M | `setup.rs` at 750 LOC also mixes too many concerns: talent param computation, shared helpers (`scaled_count`, `spell_echo_multiplier`, `increment_resonance`, geometry helpers, target-finding helpers), the casting system, three spawn functions, the visual/lifetime system, black hole interaction, and two range-despawn systems. The file comment (`//! Arcane crystal helpers, casting, and spawn.`) undersells its true scope. | Extract: `casting.rs` (casting system + `arcane_crystal_casting_logic`), `visuals.rs` (`update_crystal_visuals` + `cleanup_expired_crystals` + `cleanup_expired_crystal_beams` + `cleanup_expired_crystal_visuals`), `helpers.rs` (shared helpers that are truly cross-file: `find_random_targets_in_range`, `find_random_enemies_in_range`, `crystal_beam_geometry`, `crystal_target_teams`, `scaled_count`, etc.). Keep `setup.rs` for spawn functions only. |
| F3 | Performance | `auto.rs:65–81` | Medium | S | `auto_cast_remembered_spell` clones `auto_disintegrate_beam` (a `Vec<Entity>`) every frame for every non-permanent crystal. With `AUTO_CRYSTAL_INTERVAL = 0.2s`, this runs every Update tick when any crystal exists, producing a heap allocation per crystal. | Store the beam entity list in the component as a fixed-size `[Option<Entity>; 3]` (max 3 beams when forked), eliminating the per-frame `Vec` clone. Or use a `SmallVec<[Entity; 3]>` to avoid heap allocation for the common non-forked case. |
| F4 | Performance | `components.rs:66–68`, `hits.rs:69,76,318,323` | Medium | S | `fod_beams_processed` and `explosions_processed` are `Vec<Entity>` that grow without bound and are never pruned. FoD beams and fireball explosions are despawned within <1s, so these vecs accumulate stale dead-entity IDs for the full crystal lifetime (25s). `contains()` is O(n) on each. | Add a `retain` call on each vec to remove IDs whose entities no longer exist, or switch to a ring buffer / entity generation counter. A simple fix: call `crystal.fod_beams_processed.retain(|e| fod_beams.contains(*e))` at the start of the FoD absorption check. |
| F5 | ConsistencyRot | `hits.rs:229–234`, `hits.rs:285–291` | Medium | S | The Fisher-Yates shuffle is inlined twice inside `detect_beam_hits` — but `find_random_targets_in_range` in `setup.rs` already encapsulates exactly this pattern. The inlined versions need the `used_targets` exclusion filter that the helper doesn't support, so they can't call the helper as-is. This means the exclusion pattern (build candidates, filter, shuffle, truncate) is duplicated with no shared function. | Extend `find_random_targets_in_range` with an optional `exclude: &[Entity]` parameter (defaults empty), then replace both inlined blocks with calls to the helper. This consolidates the shuffle+filter pattern in one place. |
| F6 | DocDrift | `hits.rs:134–135` | Low | S | The comment before `detect_beam_hits` says "All crystal beams are now real DisintegrateBeam entities with CrystalSpawn marker." The word "now" implies this was a recent change and the comment is a migration note rather than a stable description. | Rewrite as a permanent description: "Crystal beams are DisintegrateBeam entities tagged with CrystalSpawn for range limiting and lifetime tracking." |
| F7 | DocDrift | `hits.rs:173–180`, no counterpart comment | Low | S | The disintegrate absorption path (`hit_by_disintegrate` block) does not call `increment_resonance` or `progress.increment()`, unlike every other spell absorption path. There is no comment explaining why. This is likely intentional (continuous beam would spam the counter every frame) but a reader has no way to tell. | Add a one-line comment: `// Not incremented: disintegrate runs every frame while channeling — would spam the counter.` |
| F8 | ArchitecturalDecay | `auto.rs:877–888` | Low | S | In `crystal_network_chain`, the crystal data is collected as a 6-tuple `(Entity, Vec3, f32, f32, Option<RememberedSpell>, bool)` where positions 2 and 3 (`_source_range`, `_source_emp`) and positions 4 and 5 of the inner loop (`_target_range`, `_target_emp`, `_target_spell`) are silently discarded (underscore-prefixed). The data is still fetched, copied, and iterated. | Replace the tuple with a named mini-struct or only collect the fields that are actually used: `(Entity, Vec3, Option<RememberedSpell>, bool)`. |
| F9 | TestDebt | `auto.rs:709–770` (`crystal_aoe_burst`) | Low | S | `crystal_aoe_burst` is a shared helper used by both `resonance_cascade_burst` and `cleanup_expired_crystals` (Prismatic Explosion). It has non-trivial logic: XZ distance filtering, friendly-fire damage application, and visual spawning. No tests cover it. | Write a unit test with a mock world verifying (a) only units within `radius` are damaged, (b) hit_count is returned correctly, (c) a burst visual entity is spawned. |

---

## Oversized Files

| File | LOC | Exempt? | Reason / Proposed Split |
|------|-----|---------|--------------------------|
| `auto.rs` | 1039 | No | Contains 8 distinct behavioral concerns. Split into: `auto_cast.rs` (timer driver + per-spell emit helpers + `spawn_crystal_mini_missile`), `talents.rs` (`resonance_cascade_burst`, `crystal_network_chain`, `crystal_aoe_burst`), `turret.rs` (`auto_crystal_fire`). |
| `setup.rs` | 750 | No | Contains casting, spawn, visuals, lifetime, and black-hole interaction. Split into: `casting.rs`, `visuals.rs`, `helpers.rs` (shared targeting/geometry helpers). Keep `setup.rs` for spawn functions or rename to `spawn.rs`. |
| `hits.rs` | 664 | No | Contains 5 independent hit-detection systems, each ~100–140 LOC. Could be split into `hits_beam.rs` (disintegrate + FoD) and `hits_projectile.rs` (fireball, meteor, magic missile, chain lightning), but at 664 LOC it is the least urgent split — each function is internally cohesive. |

---

## Looks Bad but is Actually Fine

- **`systems.rs` wildcard re-exports (`pub(super) use super::auto::*`):** Looks like a visibility anti-pattern, but `pub(super)` ensures nothing leaks outside the `arcane_crystal` module. The flat namespace in `plugin.rs` is intentional for readability. The `pub(crate)` explicit re-exports for `compute_talent_params` and `spawn_permanent_crystal` are correct.
- **`clear_absorption_flags` in a `.chain()` before the casting system (plugin.rs:27–43):** The ordering guarantee that `clear_absorption_flags` runs before `handle_arcane_crystal_casting` looks like over-engineering, but it ensures the `just_absorbed` flag (read by `crystal_network_chain`) is reset before any new absorption this frame. Correct.
- **`GhostSpellEffect` on every gameplay query instead of a system-level filter:** Each query has an individual `Without<GhostSpellEffect>` rather than a shared filter set, which looks verbose. But the project convention (per the MP ghost-gating memory) is per-query gating to keep system intent explicit. Intentional.
- **`detect_beam_hits` mut-borrows `crystal_beams` separately from `crystals`:** The two-query pattern (`crystals` mut + `crystal_beams` mut) looks like it risks aliasing, but they query different component sets (`ArcaneCrystal` vs `DisintegrateBeam + CrystalSpawn`), so Bevy allows both mutable at once. Correct.
- **`auto_cast_remembered_spell` pre-collects crystal data into a `Vec` (auto.rs:65–81):** This looks like unnecessary data copying, but the comment explains it: collecting entity IDs first allows subsequent `crystals.get_mut(entity)` calls (O(1)) instead of re-iterating (O(n)). The pattern is sound given Bevy's borrow checker constraints.
- **`arcane_crystal_casting_logic` returns `bool` (setup.rs:360–395):** Returning a bare `bool` for "cast completed" looks like a weak type contract. In practice the function is private and has exactly one call site with a clear variable name (`completed`). Acceptable.
- **`spawn_crystal` sets `crystal.position = Vec3::ZERO` as a placeholder (setup.rs:412):** The `new()` call uses `Vec3::ZERO` then `spawn_crystal_entity` overwrites it. Looks like a bug but the comment says "placeholder — set by spawn_crystal_entity", and the entity builder immediately computes and assigns the real position. Correct but fragile; the `ArcaneCrystal::new` signature taking `position` then having it be overwritten by the caller is a mild leaky abstraction.

---

## Open Questions

1. **Disintegrate + Resonance Cascade interaction** — is it intentional that a crystal being hit by Disintegrate never contributes to the Resonance Cascade threshold, even on the *first* frame contact? A player using Disintegrate + Resonance Cascade talent combo currently gets zero synergy. If unintentional, the fix is a frame-debounce (e.g., only increment once per 0.5s while `hit_by_disintegrate` is true).
2. **`fod_beams_processed` and `explosions_processed` growth** — are there scenarios (e.g., long-duration Enduring Crystal + Overcharged Matrix + many FoD casts) where these vecs grow large enough to cause measurable frame time on the O(n) `contains()` scans?
3. **`setup.rs` module name** — now that the file also owns runtime systems (visuals, black hole, cleanup), the name `setup.rs` is misleading. Should it be renamed `spawn.rs` to match the project's feature-sliced naming convention?
4. **`CrystalSpawn` lifetime field** — `lifetime: None` means "use range as the despawn trigger", while `lifetime: Some(t)` means "use time". Mixing two despawn policies in one component can be confusing. Is there a case where both should apply simultaneously?
