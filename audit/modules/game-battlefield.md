## game-battlefield

**Scope:** `src/game/battlefield/` — battlefield setup, ground tiles, environmental VFX, terrain hazards, and the trampling overlay system.

---

### Mental Model

The battlefield module is responsible for two distinct phases of work: *spawn-time setup* (ground tiles, walls, castle, underlays, lava/water markers) and *per-frame runtime systems* (ripple/mote/lava VFX, terrain hazards, trampling overlay). Setup is called imperatively from the loading spawn queue and from the multiplayer loading path, not from an `OnEnter` hook. Runtime systems are gated behind `is_gameplay_running`, which correctly covers both SP and MP host roles, so the guest never runs hazard or mote logic.

The trampling subsystem is a well-factored sub-module: `TramplingGrid` is a sparse-saved resource that persists across levels, syncs to a runtime-generated texture at a throttled interval, and handles time-travel replay isolation correctly.

Overall the module is clean. The primary issues are an oversized `systems.rs` that mixes three distinct concerns (spawn/setup helpers, environmental VFX, terrain hazards), a handful of undocumented magic number offsets, a trivial dead alias, and the absence of any unit tests for the non-trivial coordinate math in `TramplingGrid::world_to_index`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| BF-01 | ArchitecturalDecay | `systems.rs:1` | Medium | M | `systems.rs` (640 LOC) mixes three unrelated concerns: spawn/setup helpers (lines 1–396), environmental VFX emitters (lines 398–640), and terrain hazard gameplay systems. The project convention requires splitting any file >300 LOC that contains multiple concerns. | Split into `spawn.rs` (setup_battlefield, spawn helpers, ground tiles), `vfx.rs` (lava smoke/sparks, water ripples, ambient motes), and `hazards.rs` (apply_lava_damage, apply_water_slow). Update `mod.rs` and `plugin.rs` imports accordingly. |
| BF-02 | Performance | `systems.rs:489` | Medium | M | `emit_water_ripples` allocates a brand-new `StandardMaterial` asset on every ripple spawn (every `WATER_RIPPLE_INTERVAL` = 1.2 s). Because Bevy's asset store is reference-counted and despawning the ripple entity drops the handle, the material is eventually freed—but this causes repeated GPU-side material registration/deallocation cycles. A fixed pool of pre-allocated ripple materials would eliminate churn. | Pre-allocate a small `Vec<Handle<StandardMaterial>>` of blank ripple materials in `WaterRippleAssets` at startup; recycle them by updating alpha via `materials.get_mut` instead of creating new assets per ripple. |
| BF-03 | Performance | `trampling/systems.rs:144` | Low | S | `sync_trampling_texture` allocates a fresh `Vec<u8>` (up to ~256 KB for a 256x256 grid) every sync interval (~0.25 s) even when only a few cells changed. The vec is discarded after one use. | Pre-allocate the pixel buffer once as a `Local<Vec<u8>>` in the system and reuse it each sync. |
| BF-04 | DocDrift | `systems.rs:468` | Low | S | `let assets = ripple_assets;` inside `emit_water_ripples` is a stale no-op alias. It appears to be a leftover rename artefact; `ripple_assets` is used directly elsewhere. | Remove the alias; replace the downstream use (`assets.mesh.clone()`) with `ripple_assets.mesh.clone()`. |
| BF-05 | TypeContract | `systems.rs:113` | Low | S | Three magic-number offsets (`+ 100.0`, `- 100.0`, `-1500.0`) are used inline in `setup_battlefield` when positioning the sand and stone noise underlay meshes. They relate to visual alignment but are completely undocumented. | Extract named constants (e.g. `SAND_UNDERLAY_OFFSET_X`, `SAND_UNDERLAY_OFFSET_Z`, `STONE_UNDERLAY_Z`) to `constants.rs` with short comments explaining what they align to. |
| BF-06 | TestDebt | `trampling/resources.rs:34` | Low | S | `TramplingGrid::world_to_index` is non-trivial coordinate math (boundary clamping, row-major index calculation) with no unit tests. The analogous pathfinding grid coordinate math has historically been a source of off-by-one bugs. | Add an inline `#[cfg(test)]` module testing boundary cases: coords at -half, +half, 0, and outside-bounds inputs. |
| BF-07 | Performance | `trampling/systems.rs:103` | Low | S | The 120-frame entity-cleanup path in `track_unit_trampling` allocates a fresh `HashSet<Entity>` every 120 frames unconditionally, even in stable battles where no entity has recently despawned. | Guard the HashSet allocation behind a cheap check (e.g. compare `last_cells.len()` against `units.iter().count()`) or use a despawn-event driven path. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|--------------------------|
| `systems.rs` | 640 | No | Three distinct concern groups. Split into: `spawn.rs` (setup + wall/tile helpers), `vfx.rs` (lava smoke, water ripples, ambient motes), `hazards.rs` (lava damage, water slow) |
| `trampling/systems.rs` | 221 | Yes | All systems share the `TramplingGrid` resource and form a tight pipeline (track → sync → clear → save/restore). Cohesive single concern. |
| `trampling/resources.rs` | 108 | Yes | Single resource with its methods. Well within limit. |

---

### Looks Bad But Is Actually Fine

- **`apply_water_slow` has no `Without<GhostEntity>` guard (systems.rs:587).** Ghost entities are spawned without a `RoughTerrainModifier` component (confirmed in `apply_state_snapshot.rs`), so they will never appear in the `Query<(&Transform, &mut RoughTerrainModifier), Without<Corpse>>`. The system is safe as-is.
- **`apply_lava_damage` calls `is_setup_immune()` inside the system body instead of as a `run_if` condition (systems.rs:570).** The global `AtomicBool` pattern is intentionally used here (and in two other spell systems) to handle a narrow MP loading-phase race. It is not a `run_if` guard because the immunity window is very short and checked inside the hot path by design.
- **`emit_lava_fire_smoke`, `emit_lava_sparks`, `emit_ambient_motes` depend on `Res<SpellVisualAssets>` without a `resource_exists::<SpellVisualAssets>` guard.** `SpellVisualAssets` is inserted at `Startup` (before any game state runs), and all three systems are additionally gated by `is_gameplay_running`. The resource is always present when these systems execute.
- **`sync_trampling_texture` removes the old image handle before adding the new one (trampling/systems.rs:138).** This is the correct explicit cleanup to avoid leaking orphaned GPU textures; Bevy does not auto-drop replaced material texture handles.
- **Two wildcard imports (`use super::constants::*` and `use crate::game::constants::*`) in `systems.rs`.** Consistent with the rest of the codebase and acceptable for well-named constants files.
- **`spawn_castle_wall` has inline local constants `IMAGE_WIDTH` and `IMAGE_HEIGHT` (systems.rs:281–282).** These are used only inside this one function body and refer to a specific texture's pixel dimensions. Inlining is correct per project convention ("constants used by exactly one feature file should be inlined").

---

### Open Questions

1. Water ripple material pool: at `WATER_RIPPLE_LIFETIME = 4.0 s` and `WATER_RIPPLE_INTERVAL = 1.2 s`, there are at most ~3–4 ripples alive at once. Would a pool of 4 pre-allocated materials be a correct upper bound, or can multiple ripples be spawned in a burst?
2. The sand underlay offsets `+100.0` and `-100.0` (relative to `WATER_POOL_POSITION`) appear to center the sand quad on the visually-correct center of the pond art. Are these derived from a pixel-to-world calculation that should be documented, or are they empirically tuned?
3. `TramplingGrid::world_to_index` shares the same cell-size constant (`TRAMPLING_CELL_SIZE = 10.0`) as the pathfinding grid. Are these intentionally kept in sync? If the pathfinding grid cell size changes, the trampling overlay will silently misalign.
