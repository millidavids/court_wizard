<!-- UPDATED AUDIT: 2026-06-11 — reflects post-split module structure (casting/, ignite/) -->
## spell-grease

**Scope:** `src/game/units/wizard/spells/grease/`

---

### Mental model

Grease is a placement spell that drops a persistent oil-slick zone (`GreaseZone`) on the battlefield. It has three phases:

1. **Casting** — the local wizard aims with a circle indicator and releases to spawn the zone (`casting/input.rs`). Talent parameters are computed at cast time and baked into the zone (`casting/talents.rs`).
2. **Active (slippery)** — `apply_grease_slow` (in `casting/slow.rs`) ticks every `TICK_INTERVAL`, applying a `SlowMovementModifier` to overlapping units. Slip and Fall talent stuns on entry; Oil Slick applies a spell-vulnerability debuff. The zone fades and is cleaned up when `time_alive >= duration` (`ignite/lifecycle.rs`).
3. **Ignited** — `check_grease_ignition` (`ignite/burn.rs`) scans six fire-source query types and inserts `GreaseIgnited` on overlap. Burn damage then spreads radially from the ignition point. The Endless Oil talent regenerates the zone back to slippery state after fire burns out.

The module was recently split (Phase 14) from a monolithic `ignite.rs` + `casting.rs` into well-granulated sub-packages (`casting/`, `ignite/`). No files exceed 300 LOC except `ignite/burn.rs` (325 lines), which still blends three distinct fire concerns. The most significant remaining problem is a complete absence of `GhostSpellEffect` filters across all gameplay systems, meaning the guest peer in multiplayer runs host-authoritative damage/slow/cleanup logic twice.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| G-01 | Multiplayer | `ignite/burn.rs:24` | High | M | `check_grease_ignition` queries `GreaseZone` without `Without<GhostSpellEffect>`. Ghost grease zones are full `GreaseZone` entities on the guest. The fire-source queries (`fireball_explosions` at line 41, `wall_of_fires` at line 43, etc.) also lack `Without<GhostSpellEffect>`, so a ghost fireball explosion can trigger ignition on a ghost zone. Once ignited, `apply_grease_burn` runs on that ghost zone with no `Without<GhostEntity>` guard on targets (line 269), causing duplicate burn damage to real units. | Add `Without<GhostSpellEffect>` to the `zones` query. Add `Without<GhostSpellEffect>` to all six fire-source queries. |
| G-02 | Multiplayer | `ignite/burn.rs:256`, `casting/slow.rs:15` | High | M | `apply_grease_burn` and `apply_grease_slow` query `GreaseZone` without `Without<GhostSpellEffect>`. On the guest, `apply_grease_burn` will process ghost zones and damage real units (targets query at line 269 has no `Without<GhostEntity>` filter). `apply_grease_slow` inserts `SlowMovementModifier` on units near ghost zones (slow_modifier is zeroed on ghost zones so this is low-impact, but the talent stun/Oil Slick paths in `apply_grease_slow` could still fire). | Add `Without<GhostSpellEffect>` to the zone queries in both systems. Add `Without<GhostEntity>` to the targets queries. |
| G-03 | Multiplayer | `ignite/lifecycle.rs:68` | High | M | `cleanup_grease_zone` calls `commands.entity(entity).try_despawn()` when `time_alive >= duration`. Ghost zone `time_alive` is incremented locally by `apply_grease_slow` (which also has no ghost gate). The guest will despawn ghost zones on its local clock instead of waiting for the host snapshot to drop them, causing ghost zones to disappear out of sync with the host's authoritative state. | Add `Without<GhostSpellEffect>` to `cleanup_grease_zone`'s zone query. Ghost zones are already managed by `ghost_spawn.rs` when removed from the host snapshot. |
| G-04 | Multiplayer | `ignite/lifecycle.rs:105`, `ignite/lifecycle.rs:136` | Medium | S | `update_grease_regeneration` and `cleanup_grease_debuffs` also lack `Without<GhostSpellEffect>`. Regeneration on ghost zones would reset `time_alive` and remove `GreaseIgnited` on the guest. `cleanup_grease_debuffs` removes `GreaseZonePresenceTracker` and `GreaseOilSlickDebuff` — since ghost zones have no trackers this is low-impact, but the omission is inconsistent. | Add `Without<GhostSpellEffect>` to both zone queries. |
| G-05 | ArchitecturalDecay | `ignite/burn.rs:1` | Medium | M | `burn.rs` is 325 lines mixing three distinct concerns: ignition detection (`check_grease_ignition`, ~218 lines), fire-spread timing (`update_grease_fire_spread`), and burn-tick damage (`apply_grease_burn`). Per project convention files over 300 lines that are not a single match-on-enum or asset registry must be split. | Extract `check_grease_ignition` (with its fire-source scanning logic and talent burst damage) into `ignite/detection.rs`. Keep `update_grease_fire_spread` and `apply_grease_burn` in `burn.rs`. |
| G-06 | DocDrift | `ignite/burn.rs:22` | Low | S | The doc comment `/// Helper to write an obstacle event for a grease zone.` on `check_grease_ignition` is a stale copy-paste from `write_grease_obstacle` in `casting/obstacle.rs`. `check_grease_ignition` scans fire sources, inserts `GreaseIgnited`, applies burst damage, and handles three talent effects. | Replace with an accurate doc comment. |
| G-07 | Performance | `ignite/burn.rs:59` | Low | S | `check_grease_ignition` allocates `Vec<(Vec3, f32)>` via `.collect()` every frame for `ignited_zones`. This happens unconditionally even when there are no non-ignited zones to check against. | Consider scanning `ignited_zone_query` inline inside the outer loop (avoiding the collect entirely) or gating the entire collection on `any_with_component::<GreaseIgnited>` already available. |
| G-08 | Performance | `ignite/lifecycle.rs:43` | Low | S | `fade_grease_zone` calls `materials.get_mut(material_handle)` every frame for every active zone — even during the non-fading period (`fade == 1.0`, emissive constant). Bevy marks the material dirty on every `get_mut`, forcing GPU re-upload each frame. The iridescent sheen is genuinely animated, but the alpha/base_color write during the non-fade window is wasteful. | Guard the base_color write with `if fade < 1.0`. Accept the per-frame sheen update as intentional but avoid the redundant alpha mutation. |
| G-09 | TypeContract | `components.rs:157`, `components.rs:173` | Low | S | `GreaseOilSlickDebuff::new()` and `GreaseRegenerating::new()` are zero-argument constructors with trivial bodies. Rust convention for such types is `#[derive(Default)]`; `clippy::new_without_default` would flag these. | Add `#[derive(Default)]` to both structs. Replace `::new()` call sites with `::default()`. |
| G-10 | ConsistencyRot | `systems.rs:1` | Low | S | `systems.rs` is a pure wildcard re-export hub (`pub use super::casting::*; pub use super::ignite::*;`). Any new `pub fn` added to `casting/` or `ignite/` automatically enters the `systems` namespace without explicit review. Other spell modules (entangle, plague_wind) use explicit named re-exports in their sub-package `mod.rs` files. | Switch to explicit named re-exports, or add a comment documenting that all public items from both sub-packages are intentionally surfaced through this hub. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `ignite/burn.rs` | 325 | false | Three distinct concerns in 325 lines. Proposed split: `ignite/detection.rs` (check_grease_ignition — ignition source scanning + burst damage + talent geyser/lingering-flames logic), `ignite/burn.rs` (update_grease_fire_spread + apply_grease_burn). |

---

### Looks bad but is actually fine

- **`super::super::super::super::components::Spell`** — four-deep `super::` chains in `casting/input.rs`, `casting/slow.rs`, `casting/talents.rs`, and `ignite/burn.rs` are the standard pattern for reaching `wizard/components.rs` from a two-level sub-package. The project uses `crate::` for cross-module imports and `super::` for in-module traversal consistently throughout.
- **`GreaseTalentParams` as a field on `GreaseZone`, not a separate `Component`** — talent params are computed at cast time and stored as a `Copy` struct on the zone. No system queries on individual talent booleans via `With<T>`. This is correct per the project component-granularity rule.
- **Ghost `GreaseZone` has zeroed gameplay params** (`slow_modifier=0.0`, all damage=0.0) — so `apply_grease_slow` would produce zero-effect slow insertions on nearby units. This partially mitigates G-02, but the missing ghost gate is still a real bug because the talent paths (Slip and Fall stun, Oil Slick) and `apply_grease_burn` can still fire incorrectly.
- **`#[allow(clippy::too_many_arguments)]`** on `handle_grease_casting`, `grease_casting_logic`, `check_grease_ignition`, and `spawn_grease_zone` — all are Bevy systems or constructors with parameters mapping 1:1 to fields/injections. Explicitly sanctioned by project convention.
- **`queue_silenced` closure in `casting/slow.rs:74`** — used in exactly 4 places codebase-wide for deferred component insertion on already-borrowed entities. Consistent with usage in `terrain/utils.rs` and `units/systems/vfx_tinting.rs`.
- **Six separate fire-source query params in `check_grease_ignition`** — looks verbose but avoids a union query with `Option<>` sprawl. The sequential scanning approach is intentional and matches the priority of ignition sources (chain-ignition before unit FireDoT before fireballs etc.).
- **`update_grease_fire_spread` as a separate system from `apply_grease_burn`** — even though it only increments `fire_spread_time`, keeping it separate preserves the `.chain()` ordering guarantee and keeps `apply_grease_burn` a pure damage system.
- **`fade_grease_zone` and VFX systems running on ghost zones** — visual-only systems running on ghost zones produce correct visuals on the guest at no gameplay cost. The fade/sheen/VFX are purely cosmetic.

---

### Open questions

1. Should `fade_grease_zone` and `update_grease_fire_spread` intentionally run on `GhostSpellEffect` zones (for visual correctness) while all damage/slow/debuff systems are gated out? If so, this should be explicitly documented in those functions with a comment mirroring the pattern in `arcane_crystal/plugin.rs:49`.
2. The ghost `GreaseZone` does not snap `time_alive` from the host snapshot — the guest tracks it locally via delta accumulation. Is this acceptable for a visual-only ghost, or should the host include `time_alive` in `extra[]` to keep fade/cleanup in sync?
3. `update_grease_regeneration` queries `zone_materials: Query<&MeshMaterial3d<StandardMaterial>, With<GreaseZone>>` separately inside an outer loop over `zones: Query<..., With<GreaseRegenerating>>`. These could be unified into a single query since the material is on the same entity, eliminating the inner `zone_materials.get(entity)` call.
