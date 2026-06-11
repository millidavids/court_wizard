## game-terrain

**Scope:** `src/game/terrain/` — all .rs files (50 files, 3 222 LOC)

---

### Mental Model

The terrain module is a self-contained cluster of destructible and interactive battlefield objects: boulders (thrown by brutes/ogres, explodable by fire), ponds (freeze/electrify/evaporate reactively), trees and bushes (flammable, provide rough terrain), and cosmetic flora (trampleable). A cross-cutting `TerrainDamageMessage` bus lets spell systems broadcast damage events that the terrain then reacts to, keeping spell code decoupled from terrain internals.

The module is well-structured: every sub-feature gets its own folder with the project's prescribed `plugin.rs` / `components.rs` / `constants.rs` / `systems.rs` decomposition. All Update systems carry `run_if` guards. No `.unwrap()` calls. No `println!` / debug prints. All messages use `#[derive(Message)]` with `Message` suffixes. The multiplayer host-vs-guest split is clearly documented in comments. This is one of the cleaner modules in the codebase.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| T-01 | ConsistencyRot | `boulder/systems/combat.rs:78` | Medium | S | `xz_distance` is re-defined as a local closure in `apply_spell_damage_to_rocks`, duplicating the free function of the same name already imported via `crate::game::units::wizard::spells::utils::xz_distance` (used in `terrain/utils.rs`). | Remove the local closure and `use crate::game::units::wizard::spells::utils::xz_distance;` at the top of the file. |
| T-02 | ConsistencyRot | `bush/systems.rs:129` + `tree/systems.rs:105` | Medium | M | `apply_burning_bush_damage` and `apply_burning_tree_damage` are structural clones — identical loop shape, timer tick, heat-radius and `apply_heat_zone_fire_dot` call. Only component type and three constants differ. | Extract a `apply_burning_vegetation_damage` helper into `terrain/utils.rs` that takes `center: Vec3`, `radius: f32`, `tick_interval: f32`, `heat_radius_mult: f32`, `spell_damage: f32` and the unit query. Both functions reduce to a 3-line wrapper. |
| T-03 | ConsistencyRot | `bush/systems.rs:164` + `tree/systems.rs:140` | Medium | M | `emit_burning_bush_vfx` and `emit_burning_tree_vfx` are near-identical. The only substantive differences are `smoke_count`/`spark_count` (2 vs 3) and using `BUSH_SPRITE_HEIGHT` as height (bush is fixed) vs `tree.height` (per-entity). | Extract `emit_burning_vegetation_vfx` into `terrain/utils.rs` accepting `center_x`, `center_z`, `height`, `radius`, `smoke_count`, `spark_count`, and smoke/spark interval constants. The two systems become thin wrappers. |
| T-04 | ArchitecturalDecay | `pond/systems/fish.rs` + `pond/systems/lily.rs` | Low | S | File names are misleading. `fish.rs` contains `tick_pond_shocked` (electrical arc damage — nothing to do with fish). `lily.rs` contains `apply_pond_wet`, `tick_wet_timer`, and `apply_frozen_pond_slow` (wet/slow logic — nothing to do with lily pads). New readers have to open both files to understand pond behavior. | Rename `fish.rs` → `shocked.rs` and `lily.rs` → `wet_slow.rs`. Update `pond/systems/mod.rs` accordingly. |
| T-05 | ArchitecturalDecay | `boulder/components.rs`, `tree/components.rs` | Low | L | Circle-geometry methods (`contains_point_xz`, `line_segment_intersects`, `push_out`, `any_blocks_los`) are fully duplicated between `Boulder` and `Tree`. A third copy exists in `wall_of_stone/components.rs` (outside this scope). Three independent implementations of the same circle-intersection algorithm. | Introduce a `CircleObstacle { center: Vec3, radius: f32 }` newtype (or trait) in `terrain/utils.rs` with these methods. `Boulder` and `Tree` can embed or `Deref` to it. This would also allow `wall_of_stone` to reuse the same code. |
| T-06 | TypeContract | `boulder/systems/projectile.rs:20` | Low | S | `let start_y = 20.0;` is a magic number representing the vertical launch height from a thrower. There is no named constant and no explanation of where 20.0 comes from relative to unit sprite heights. | Extract `pub(super) const BOULDER_LAUNCH_Y: f32 = 20.0;` into `boulder/constants.rs` with a comment explaining the value (e.g., approximate brute/ogre sprite center height). |
| T-07 | ArchitecturalDecay | `pond/components.rs:10` | Low | S | `Pond` carries `ripple_timer: f32`, a purely visual emission timer, inside the same component that holds gameplay-authoritative data (`center`, `radius`). This means the `Pond` component is marked `Changed` every ripple-emission interval, potentially triggering unnecessary change-detection in any system querying `Pond` without filtering. | Move `ripple_timer` to a separate `PondRippleTimer` component, or store it in a `Local<HashMap<Entity, f32>>` in the `emit_pond_ripples` system, keeping `Pond` clean for change-detection purposes. |

---

### Oversized Files

No .rs files in this scope exceed 300 LOC. The largest file is `pond/systems/ripple.rs` at 243 lines.

| File | LOC | Exempt | Reason |
|------|-----|--------|--------|
| `pond/systems/ripple.rs` | 243 | true | Under 300 LOC; all content is cohesive pond-visual/state logic. |
| `terrain/utils.rs` | 214 | true | Under 300 LOC; shared helpers used across the module. |
| `bush/systems.rs` | 203 | true | Under 300 LOC. |
| `terrain/systems.rs` | 191 | true | Under 300 LOC. |
| `boulder/systems/projectile.rs` | 182 | true | Under 300 LOC. |
| `tree/systems.rs` | 179 | true | Under 300 LOC. |

---

### Looks Bad But Is Actually Fine

- **`update_wind_sway_time` iterates all `Assets<WindSwayMaterial>` without an entity guard**: Wind-sway materials are a tiny fixed set created at startup (one per sprite variant). Iterating ~15 asset entries per frame is negligible, and `is_spell_effects_active` prevents it from running during loading.
- **Per-ripple `StandardMaterial` allocation in `emit_pond_ripples`**: Each ripple spawns a fresh material. This runs at most once per 1.5 s per pond. Bevy's asset GC reclaims handles when the entity despawns — no persistent leak.
- **`tick_pond_shocked` allocates a `Vec<(Entity, Vec3, f32)>` per arc pulse**: Arc pulses are gated by `POND_SHOCK_ARC_COOLDOWN = 0.8s` and the shocked state is rare. The allocation is acceptable.
- **`trample_flora` uses `Vec<u32>::contains` inside `retain`**: With at most 80 flora and rarely more than 1–2 tramples per frame, this O(n) inner call is not a hotspot.
- **`ignite_bushes_from_fire` / `ignite_trees_from_fire` structural similarity**: They appear similar but differ meaningfully — bush ignition upgrades the flow cost via `ObstacleChanged` (`BURNING_BUSH_FLOW_COST`) whereas tree ignition does not, because trees are already full `Blocked` obstacles. Merging them would require special-casing that one difference.
- **`Boulder::line_segment_intersects` vs `Tree::line_segment_intersects` tiny comment difference**: Both implementations are algorithmically identical and correct. The difference is that Boulder has an inline `// Segment entirely inside circle` comment. This is not a functional divergence.

---

### Open Questions

1. Should `Pond::ripple_timer` be moved to a separate `PondRippleTimer` component? Currently no system queries `Changed<Pond>`, so the timer churn is harmless, but future refactors could introduce change-detection sensitivity.
2. Is there an architectural plan to extract circle-geometry methods into a shared `terrain/geometry.rs`? Three copies now exist (boulder, tree, wall_of_stone) — past the project's 3-site threshold. The scope boundary means this must be coordinated with the spell-wall_of_stone auditor.
3. The `start_y = 20.0` boulder launch height — is this intentionally set to approximately the brute/ogre sprite center? At `UNIT_SCALE = 4.0` a 32px-tall sprite has a world height of 128 units, so a center Y of ~64 units. The value 20.0 is much lower and may reflect a deliberate visual choice (launch from near the ground) rather than a calculation error.
