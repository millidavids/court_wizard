## game-terrain

**Scope:** `src/game/terrain/` — static/dynamic terrain objects: boulders, ponds, bushes, trees, flora, wind-sway shader.

---

### Mental model

The terrain module is a well-structured feature-sliced hierarchy. Each terrain type (boulder, bush, flora, pond, tree, wind\_sway) lives in its own sub-module with a plugin, components, constants, resources, and systems file. Cross-cutting concerns are cleanly hoisted: `TerrainDamageMessage` is the unified fan-out bus for reactive effects (fire→evaporation, frost→freeze, electric→shock), `utils.rs` holds shared ignition detection (`should_ignite_from_fire`) and VFX emission (`emit_burning_vfx`) helpers, and `systems.rs` owns the cross-cutting dam-application fan-out to ponds and boulders.

All Update systems carry `run_if` guards (either `is_gameplay_running`, `is_spell_effects_active`, or a cheap `any_with_component` short-circuit), so idle frames incur minimal cost. The module is multiplayer-aware: host-authoritative gameplay systems are gated with `is_gameplay_running` (which returns false for the guest), while pure-visual systems run under `is_spell_effects_active` on both peers.

Main pain points: (1) `boulder/systems.rs` and `pond/systems.rs` both exceed 300 LOC with multiple distinct concerns; (2) `Tree::line_segment_intersects` / `push_out` and `Boulder::line_segment_intersects` / `push_out` are byte-for-byte copies; (3) `apply_spell_damage_to_rocks` in boulder uses a local closure for `xz_distance` instead of the shared helper from `spells/utils.rs`; (4) `ignite_trees_from_fire` uses full `crate::…` paths inline rather than `use` imports, unlike `ignite_bushes_from_fire`; (5) `Pond::obstacle_bounds` and `Tree::obstacle_bounds` are suppressed with `#[allow(dead_code)]` indicating unused methods.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| T-01 | ArchitecturalDecay | `boulder/systems.rs:1–524` | High | M | File has 524 LOC and mixes 6 distinct concerns: projectile spawn, projectile animation & landing, lifetime/sinking, attack interaction, damage-tint, spell-damage application. The CLAUDE.md limit is ~300 LOC for non-cohesive files. | Split into `projectile.rs` (spawn + animate + land), `lifetime.rs` (tick, sink, cleanup), `combat.rs` (units attack, spell damage), `tint.rs` (damage tint), keeping only the thin `spawn_terrain_boulder` helper in `systems.rs` or a new `spawn.rs`. |
| T-02 | ArchitecturalDecay | `pond/systems.rs:1–454` | High | M | File has 454 LOC across 9 systems covering 4 independent state machines (wet, evaporation/fog, freeze, shock). Pond module has no splitting precedent for systems. | Split into `wet.rs` (apply_pond_wet, tick_wet_timer), `freeze.rs` (apply_frozen_pond_slow, tick_pond_frozen, update_frozen_pond_tint, restore_pond_material_on_thaw), `evaporation.rs` (tick_pond_evaporation, emit_pond_fog_particles), `shock.rs` (tick_pond_shocked), `ripples.rs` (emit_pond_ripples, spawn_single_pond). |
| T-03 | ConsistencyRot | `tree/components.rs:34–62` / `boulder/components.rs:46–74` | Medium | S | `line_segment_intersects` implementation is byte-for-byte identical in `Tree` and `Boulder`. The only difference is the comment on the "entirely inside" branch in boulder. Same duplication for `push_out` (tree:76–96, boulder:79–100). | Extract a free function `circle_segment_intersect(center: Vec2, radius: f32, start: Vec3, end: Vec3) -> Option<f32>` and `circle_push_out(center: Vec2, radius: f32, point: Vec3, unit_radius: f32) -> Option<Vec3>` into `terrain/utils.rs`, then delegate from both impls. |
| T-04 | ConsistencyRot | `boulder/systems.rs:362–365` | Low | S | `apply_spell_damage_to_rocks` defines a local closure `let xz_distance = |a, b| …` instead of importing the shared `xz_distance` from `crate::game::units::wizard::spells::utils` that `terrain/utils.rs` already imports and uses. | Replace the closure with `use crate::game::units::wizard::spells::utils::xz_distance;` at the top of the file. |
| T-05 | ConsistencyRot | `tree/systems.rs:65–72` | Low | S | `ignite_trees_from_fire` uses raw `crate::game::units::wizard::spells::…::ComponentName` paths inline in the Query type parameters. The sibling `ignite_bushes_from_fire` in `bush/systems.rs` does the same, but this inconsistency with files that use top-level `use` imports makes the signatures hard to read. | Add `use` imports for the five spell-component types to the top of both files (or simply pull them in via the existing `utils.rs` re-export). |
| T-06 | TypeContract | `pond/components.rs:16–18` / `tree/components.rs:65–66` | Low | S | `Pond::obstacle_bounds` and `Tree::obstacle_bounds` are annotated `#[allow(dead_code)]`. The code compiles them but no site inside the terrain module (or project-wide, confirmed by grep) calls them. Dead methods with suppressed warnings are a maintenance trap. | Remove the methods if nothing needs them. If they are intended as public geometry helpers, make them `pub` and remove the `#[allow(dead_code)]`. |
| T-07 | Performance | `pond/systems.rs:409–427` | Low | S | `tick_pond_shocked` does a heap `Vec::collect()` to gather and sort targets on each arc pulse (up to every 0.8 s). Given `POND_SHOCK_MAX_TARGETS = 4` and the infrequent interval this is borderline acceptable, but the collect + sort could be replaced with an in-place partial sort (e.g. `select_nth_unstable`) over a stack `SmallVec<[_; 8]>`. | Either leave as-is (runs ~1/0.8 s = 1.25/s and the world has at most a handful of shocked ponds) or replace with a `SmallVec` + `select_nth_unstable` to avoid heap allocation. |
| T-08 | Performance | `flora/systems.rs:111–128` | Low | S | `trample_flora` allocates `Vec<u32>` every frame when flora exists (up to 80 items). The inner double-loop is O(flora × units) — up to 80 × 200 = 16 000 distance checks per frame. | Change the outer loop to iterate flora, inner to iterate units (same complexity), but collect trampled `Entity` values into a `SmallVec` or use `Commands::entity().try_despawn()` directly inside the inner loop with an early-break, removing the post-loop retain loop and the allocation. |
| T-09 | ArchitecturalDecay | `boulder/systems.rs:51–55` | Low | S | `crate::game::multiplayer::components::NetworkedSpellEffect` and `crate::networking::snapshot::SpellEffectKind::BoulderProjectileEffect` are referenced three times in the file with fully qualified paths (lines 51–55, 137–140, 507–510). This breaks the "imports at top" convention visible everywhere else. | Add `use` aliases for `NetworkedSpellEffect` and `SpellEffectKind` at the top of `boulder/systems.rs`. |
| T-10 | Performance | `bush/systems.rs:59–73` | Low | S | `apply_bush_slow` runs every gameplay frame and is O(units × bushes). In a worst-case battle with ~200 units and 12 bushes that is 2 400 point-in-circle tests per frame. No spatial indexing — acceptable for this scale but worth noting if unit counts rise. | Acceptable at current scale. If future unit counts grow, pre-collect bush positions/radii into a stack array once per frame, rather than iterating the query repeatedly with `.iter().any()`. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `boulder/systems.rs` | 524 | false | Six concerns. Proposed split: `projectile.rs`, `lifetime.rs`, `combat.rs`, `tint.rs`, keep `spawn.rs` + `sync.rs` for terrain-spawner and teleport sync. |
| `pond/systems.rs` | 454 | false | Four independent state machines. Proposed split: `wet.rs`, `freeze.rs`, `evaporation.rs`, `shock.rs`, `ripples.rs`, `spawn.rs`. |

---

### Looks bad but is actually fine

- **`emit_pond_ripples` calls `materials.add()` each ripple.** Each pond spawns one ripple every 1.5 s and the ripple entity is despawned after ~3 s. The `MeshMaterial3d` strong handle on the entity is the only reference, so the asset is freed on despawn. No leak. Matches the established pattern in `battlefield/systems.rs` `emit_water_ripples`.
- **Missing `Without<GhostEntity>` in terrain damage systems (`apply_burning_tree_damage`, `apply_burning_bush_damage`, `apply_pond_wet`, etc.).** These all run under `is_gameplay_running` which returns `false` for the guest. `GhostEntity` units only exist on the guest side. The host never spawns `GhostEntity` for guest units, so the missing filter is not a bug.
- **`trample_flora` uses `config.saved_flora.retain(|f| trampled_ids.contains(&f.id))`.** The `Vec::contains` call is O(n) but `trampled_ids` is at most a handful of items per frame (one trample per flora, and flora disappear on contact), so this is negligible.
- **`boulder/systems.rs:265` — `_health` and `_temp_hp` fields unused in `units_attack_blocking_rocks`.** These fields are part of a destructuring pattern to get `AttackTiming`; the compiler would warn if they were actually unused without the underscore prefix. This is idiomatic Bevy partial destructuring.
- **`ignite_trees_from_fire` does not send `ObstacleChanged` when a tree ignites** (unlike `ignite_bushes_from_fire` which bumps flow cost to 15.0). Trees are already `ObstacleType::Blocked` (impassable), so a burning tree doesn't need a flow-cost upgrade — this asymmetry is intentional.
- **`BoulderPlugin` comment says "host-only in MP … no-op on guest"** for `update_rock_damage_tint`. The system is registered unconditionally but gated by `is_gameplay_running`. This means the system body is never scheduled on the guest. The comment is accurate.

---

### Open questions

1. Should `Pond::obstacle_bounds` and `Tree::obstacle_bounds` be kept as future API (e.g., for upcoming dispel/physics interactions) or removed? The `#[allow(dead_code)]` implies the methods were written speculatively.
2. `tick_pond_shocked` calls `chain_lightning::systems::spawn_arc` directly via full path. Is that intended to bypass any future chain-lightning gating logic, or should it go through a shared VFX helper?
3. `apply_burning_bush_damage` inserts `FireDoT` on units, and so does `apply_burning_tree_damage`. Currently there is no cap on how many terrain fire sources can stack DoTs simultaneously. Is this intentional (stackable friendly fire) or should there be a DoT-source cap?
