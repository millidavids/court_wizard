## spell-arcane_crystal

**Scope:** `src/game/units/wizard/spells/arcane_crystal/` — 18 `.rs` files across `setup/`, `auto/`, and root.

---

### Mental Model

The Arcane Crystal spell places a stationary gem that absorbs incoming spell projectiles/beams and re-emits scaled-down versions at random enemies in range, auto-casts the last-absorbed spell on a timer, and supports five mutually-exclusive talent branches across three tiers (Resonance Cascade, Spell Echo, Crystal Network, Prismatic Explosion, Auto-Crystal turret). The module has been recently split from a monolith (`systems.rs` still carries a "Phase 14" migration comment) into:

- **`setup/`** — `casting.rs` (input/state), `spawn.rs` (entity spawn), `visuals.rs` (lifetime/visual/cleanup/black-hole), `helpers.rs` (shared math/target-finding/talent-param helpers).
- **`auto/`** — `auto_cast.rs` (timer driver + `handle_auto_disintegrate`), `spell_variants.rs` (per-spell auto-cast helpers), `spawn_helpers.rs` (`spawn_crystal_mini_missile`, `spawn_crystal_disintegrate_beam`, `spawn_fod_beams_at`), `talents.rs` (`resonance_cascade_burst`, `crystal_aoe_burst`), `network.rs` (`crystal_network_chain`), `turret.rs` (`auto_crystal_fire`).
- **`hits.rs`** — five per-spell hit-detection systems (fireball, beam/FoD, meteor, magic missile, chain lightning).
- **`components.rs`**, **`constants.rs`**, **`plugin.rs`**, thin **`systems.rs`** re-export hub.

All gameplay systems carry `Without<GhostSpellEffect>` for MP correctness. All Update systems have `run_if` guards via either component-presence conditions or the outer `is_spell_effects_active`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `setup/casting.rs:130-134` | High | S | Auto-Crystal turret placement limit is broken. Comment says "only 1 non-permanent crystal allowed per level" but the code filters `existing_crystals` by `!c.permanent`. Since newly cast turrets set `crystal.permanent = true`, the filter always returns 0, and the one-turret limit never fires. | Change filter to `.filter(\|(_, c)\| c.permanent)` to count existing permanent turrets. Update the comment to match. |
| F2 | ArchitecturalDecay | `plugin.rs:56-71` | High | S | The gameplay `add_systems` group is not `.chain()`-ed. `crystal_network_chain` reads `just_absorbed` set by the `detect_*` systems, and `resonance_cascade_burst` reads `resonance.absorptions` incremented inside `detect_*`. Bevy serialises these due to shared `&mut ArcaneCrystal` access, but execution order is non-deterministic. `crystal_network_chain` and `resonance_cascade_burst` will frequently observe stale state from the previous frame, causing missed Crystal Network triggers and delayed Resonance Cascade bursts. | Add `.chain()` to the gameplay group, or add explicit `.after(systems::detect_fireball_hits).after(systems::detect_beam_hits)...` ordering for `crystal_network_chain` and `resonance_cascade_burst`. |
| F3 | Performance | `components.rs:66-68`, `hits.rs:69,76,318,323` | Medium | S | `fod_beams_processed` and `explosions_processed` are `Vec<Entity>` that grow unboundedly over the crystal's 25-second lifetime. Every absorbed entity ID is pushed but stale dead-entity IDs are never pruned. The `.contains()` calls at hits.rs:69 and hits.rs:318 are O(n) scans. Under heavy fireball/FoD activity, these vecs accumulate hundreds of stale IDs. | Switch to `HashSet<Entity>` to make `.contains()` O(1), or add a `retain` pass at the start of each hit-detection loop to prune IDs whose entities are gone. |
| F4 | DocDrift | `setup/casting.rs:128-129` | Low | S | Comment says "Count crystals placed this level (non-permanent ones)" but the spawned crystal IS permanent (`auto_crystal` → `crystal.permanent = true`). The comment is the opposite of the correct intent. | Correct to "Count permanent turrets placed this level". |
| F5 | DocDrift | `setup/casting.rs:153-158` | Low | S | Comment says "despawn oldest if at limit" but the code uses `existing_crystals.iter().next()` which returns an arbitrary entity, not the chronologically oldest (no `time_alive` comparison). | Either sort by `time_alive` to really despawn oldest, or change comment to "despawn an arbitrary crystal at limit". |
| F6 | DocDrift | `systems.rs:1` | Low | S | `systems.rs` carries the comment "Re-export hub for arcane_crystal systems split (Phase 14)". The "Phase 14" label is a migration artifact that leaks internal history into the module surface. | Drop the phase reference; update to a simple description of purpose. |
| F7 | DocDrift | `hits.rs:134-135` | Low | S | Doc comment before `detect_beam_hits` says "All crystal beams are now real DisintegrateBeam entities with CrystalSpawn marker." The word "now" is a migration note, not a stable description. | Rewrite as permanent description: "Crystal beams are DisintegrateBeam entities tagged with CrystalSpawn for range limiting and lifetime tracking." |
| F8 | DocDrift | `hits.rs:166-176` | Low | S | The disintegrate absorption path sets `hit_by_disintegrate = true` and marks absorption but does NOT call `increment_resonance()` or `progress.increment()`, unlike every other spell absorption path. There is no comment explaining this intentional asymmetry. | Add a comment: "// Resonance/progress not incremented: disintegrate hits every frame while channelling — would spam the counter." |
| F9 | ArchitecturalDecay | `auto/network.rs:52-64` | Low | S | `crystal_network_chain` collects crystal data as a 6-tuple where positions 2 and 3 (`_source_range`, `_source_emp`) and the inner loop's positions 4 and 5 (`_target_range`, `_target_emp`, `_target_spell`) are all underscore-prefixed and never used. The data is still fetched and iterated unnecessarily. | Narrow the tuple to only the fields actually used: `(Entity, Vec3, Option<RememberedSpell>, bool)`. |

---

### Oversized Files

| File | LOC | Exempt? | Reason / Proposed Split |
|------|-----|---------|--------------------------|
| `hits.rs` | 664 | No | Contains 5 independent hit-detection functions (fireball, beam/FoD, meteor, magic missile, chain lightning), each ~100-130 LOC. Could be split into `hits/mod.rs` + `hits/fireball.rs`, `hits/beam.rs`, `hits/meteor.rs`, `hits/magic_missile.rs`, `hits/chain_lightning.rs`. |
| `auto/auto_cast.rs` | 312 | No | Contains the main timer-dispatch system and the `handle_auto_disintegrate` helper (~90 LOC). `handle_auto_disintegrate` is a distinct concern that could move to `auto/disintegrate.rs`. |

---

### Looks Bad but is Actually Fine

- **`auto_cast_remembered_spell` collects crystal data into a `Vec` every frame** — required by Bevy's borrow checker: you can't call `crystals.get_mut(entity)` inside `crystals.iter()`. The pre-collect + O(1) get_mut pattern is idiomatic. At most 3 entries.
- **Inline Fisher-Yates shuffles at hits.rs:231 and hits.rs:288** — these are NOT duplicates of `find_random_targets_in_range`. They operate on an already-collected `Vec` pre-filtered by `used_targets` exclusion. The shared helper cannot accommodate that additional filter without API changes. Legitimately inline.
- **`ArcaneCrystal` carrying 15+ fields** — all fields are tightly coupled to the central crystal systems. No independent system queries on a subset; splitting into sub-components would create costly joins with no query benefit.
- **`crystal_aoe_burst` taking many arguments** — it's a shared helper called from two sites with legitimately different parameters. The `#[allow(clippy::too_many_arguments)]` is appropriate.
- **Two separate `add_systems` groups in plugin.rs** — the split into visual-group (chained) and gameplay-group is intentional to allow visual systems to run on ghost crystals while gameplay systems are ghost-gated. The design rationale is clearly documented in comments.
- **`cleanup_expired_crystal_beams` allocating `despawned: Vec<Entity>`** — only populated when beams actually expire; guarded by `if despawned.is_empty() { return; }`. Not a hot path.

---

### Open Questions

1. **Disintegrate + Resonance Cascade synergy (hits.rs:166-176):** Is it intentional that disintegrate hits never contribute to the Resonance Cascade threshold? A player who picks both talents currently gets zero synergy from Disintegrate → Crystal. If unintentional, a per-second debounce would fix it.
2. **`crystal_black_hole_interaction` and ghosts:** Should the black hole visually pull/destroy the guest's ghost copy of the host's crystal? The system currently has no `Without<GhostSpellEffect>` guard, unlike all other systems in the gameplay block. This might be correct (visual effect on ghosts) or a missed guard (see the pattern in every other system in that block).
3. **`CrystalSpawn` dual despawn policy:** `lifetime: None` means range-based despawn; `lifetime: Some(t)` means time-based. Can both policies ever apply simultaneously to one entity? If not, a comment clarifying mutual exclusivity would help.
