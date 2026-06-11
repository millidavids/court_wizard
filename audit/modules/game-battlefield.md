## game-battlefield

**Scope:** `src/game/battlefield/` — battlefield ground, castle wall, environmental effects (lava, water, motes), trampling mud-overlay system.

---

### Mental model

The module owns two concerns: (1) **one-shot visual setup** (`setup_battlefield`, `spawn_castle_wall`) called from the loading queue and multiplayer spawning code, which builds the ground-tile grid, wall backdrops, floor plane, and stone/sand noise underlays; and (2) **per-frame environmental systems** — lava damage, water slow, ambient particle effects (fire smoke, sparks, ripples, floating motes), and the trampling mud overlay. The trampling sub-module (`trampling/`) is well-isolated: it maintains a sparse f32 grid, serialises it into save data (base-64 packed binary), and periodically rebuilds a runtime GPU texture. Custom materials (`GroundMaterial`, `StoneNoiseMaterial`) are thin `AsBindGroup` wrappers that delegate work to WGSL shaders. The module is small and mostly clean; the one critical bug is a missing `Without<GhostEntity>` guard on `apply_lava_damage`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| BF-01 | TypeContract | `systems.rs:544` | Critical | S | `apply_lava_damage` queries `(&Transform, &mut Health)` without `Without<GhostEntity>`. Ghost units intentionally carry `Health` for CRDT damage propagation (see `guest_snapshot.rs:1151`). A ghost standing in the lava pool on the guest side will have `health.take_damage()` called, which pushes through the CRDT channel and silently reduces the host's authoritative unit HP. | Add `Without<crate::game::multiplayer::components::GhostEntity>` to the query filter tuple alongside the existing `Without<Corpse>` exclusions. |
| BF-02 | ArchitecturalDecay | `plugin.rs:45` | Medium | S | `load_battlefield_assets` has its full system body defined in `plugin.rs`. Per project convention, `plugin.rs` must contain Bevy registration only; system bodies belong in sibling files. | Move `load_battlefield_assets` to `systems.rs` (or a new `assets.rs` if preferred) and reference it from `plugin.rs` via `systems::load_battlefield_assets`. The `pub(crate) use plugin::load_battlefield_assets` re-export in `mod.rs` should then become `pub(crate) use systems::load_battlefield_assets`. |
| BF-03 | Performance | `systems.rs:478` | Medium | M | `emit_water_ripples` calls `materials.add(StandardMaterial { … })` every time a ripple is spawned (every `WATER_RIPPLE_INTERVAL = 1.2s`). Each call allocates a new asset slot and registers it with the GPU. Ripples are then individually mutated in `update_water_ripples` to fade alpha. This pattern leaks a new material handle per live ripple (typically 3–4 alive simultaneously). | Encode alpha as part of a custom thin material or drive alpha via a `WaterRipple`-specific uniform rather than per-entity `StandardMaterial` clone. A simpler fix: pre-allocate a small pool of `StandardMaterial` handles in `WaterRippleAssets` (e.g. 8 slots) and recycle them — avoids continuous GPU registration pressure. |
| BF-04 | Performance | `trampling/systems.rs:145` | Low | M | `sync_trampling_texture` rebuilds a fresh 360,000-cell (1.44 MB) pixel buffer on every dirty sync (up to 4× per second). The `Vec::with_capacity(pixel_count * 4)` allocation + per-cell push loop + `Image::new` + `images.remove` + `images.add` is an avoidable hot path given that the grid is sparsely dirty in practice. | Maintain a persistent reusable `Vec<u8>` (e.g. stored in `TramplingGrid` or `TramplingGrid`'s texture resources) and write into it in-place, then update the existing `Image`'s data directly. Pair with a dirty-rectangle tracking approach so only changed cells need repacking. If in-place `Image` mutation still doesn't reliably upload to GPU in Bevy 0.17, the comment on line 109 explains the workaround; that's acceptable, but the buffer should at least be pre-allocated and reused. |
| BF-05 | DocDrift | `systems.rs:467` | Low | S | Comment says "random position within the pool" but the position is deterministically derived from `time.elapsed_secs()`. The sequence is not random — it is a deterministic pseudo-random walk driven by elapsed time. It appears visually varied, but restarts with the same pattern each session start. | Change comment to "pseudo-random position derived from elapsed time" to avoid misleading future readers. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| `systems.rs` | 626 | No | Two distinct concerns: (1) one-shot battlefield setup/spawn helpers (lines 1–385), (2) per-frame environmental effect systems (lines 387–626). Proposed split: `spawn.rs` (setup_battlefield, spawn_castle_wall, spawn_wall_backdrop, spawn_right_wall_backdrop, spawn_ground_tiles, create_tile_mesh, pick_weighted_tile) and `effects.rs` (emit_lava_fire_smoke, emit_lava_sparks, emit_water_ripples, update_water_ripples, apply_lava_damage, apply_water_slow, emit_ambient_motes). |

---

### Looks bad but is actually fine

- **`StoneNoiseMaterial` with four `#[uniform(0)]` fields** (`ground_material.rs:36–51`): looks like a binding collision but Bevy's `AsBindGroup` macro packs all fields with the same binding number into a single packed uniform struct. The shader at `assets/shaders/stone_noise.wgsl` confirms a matching `StoneUniforms` struct with four `vec4<f32>` fields in order. This is the same pattern used across many other materials in this codebase and is correct.

- **`spawn_castle_wall` being `pub`** (`systems.rs:258`): called from `src/game/multiplayer/spawning.rs` and `src/game/loading/queue.rs` — two different callers need the generic `<M: Component + Clone>` version. The public API is intentional.

- **`trampling/systems.rs:103` allocating a `HashSet` every 120 frames** for entity cleanup: this is a periodic O(n) dead-entity purge on the `last_cells` `Local<HashMap>`. The interval is explicit (120 frames ≈ 2 seconds), the allocation is proportional to live unit count (not per-frame), and the pattern prevents unbounded HashMap growth. Reasonable trade-off.

- **`emit_ambient_motes` using `vfx::systems::spawn_floating_motes`** with tight coupling to `SpellVisualAssets`: the battlefield reuses spell VFX assets for motes rather than owning its own asset registry. This is intentional shared-asset design, not a hidden coupling violation.

- **`systems.rs:95–150` hardcoded world-space constants** (sand underlay at `WATER_POOL_POSITION.x + 100.0`, stone underlay at Z=-1500): these offsets are render-layer Z-sorting tricks specific to the single piece of setup code that reads them. Inlining them here rather than in `constants.rs` is correct per the project rule that constants used by exactly one site should be inlined.

---

### Open questions

1. Does `is_setup_immune()` (called in `apply_lava_damage`) cover the multiplayer guest scenario where ghost entities are live but the game has not yet fully started? If so, BF-01 might be latent rather than immediately exploitable — but the guard should still be added as a safety net because `is_setup_immune` is a global flag, not a per-entity filter.

2. The `sync_trampling_texture` comment (line 109) says "Direct mutation of `image.data` doesn't reliably trigger GPU re-upload in Bevy 0.17." Is this still true after any recent Bevy patch, or has `image.mark_as_changed()` / `ImagePlugin` configuration changed this? If in-place mutation now works, the full asset-replace-and-remove cycle can be simplified significantly.

3. `spawn_castle_wall` takes a generic `<M: Component + Clone>` marker. The `Clone` bound is unused by the body (only one entity is spawned). Consider removing the `Clone` bound if nothing else requires it — it currently restricts callers to `Clone` marker types unnecessarily.
