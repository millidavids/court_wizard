## wizard-archetypes

**Scope:** `src/game/units/wizard/archetypes/` — all wizard archetype plugins: Arcanorouter, Gunslinger, Meteorologist, Psychopath, Randomancer (Roulette), RuneCaster (Runes), Shepherd, Swordcerer.

---

### Mental Model

Each wizard archetype is a self-contained sub-module with its own plugin, components, resources, and systems. All archetypes are gated at runtime via `run_if` conditions (`is_warglock`, `is_meteorologist`, `is_swordcerer`, etc.), so inactive archetypes are zero-cost. The Swordcerer is the most complex: it has a field-avatar mechanic with full host-authoritative multiplayer networking, a health bar UI, and shared physics helpers. The Meteorologist manages dual-slot weather state (local + remote) replicated via a single wire message per choice change, with per-peer intensity ramp for smooth visual transitions without per-frame packets. The Gunslinger has 5 weapons with hitscan + projectile systems and a dedicated replication layer for opponent-visible gun visuals. The Arcanorouter features a 4-slider resource pool with normalization and MP setup-stage range pinning. Roulette/Runes are simpler state machines. Psychopath is deliberately disabled in MP. Shepherd has zero runtime systems.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| W-A-01 | Performance | `meteorologist/visuals.rs:175,206` | High | M | `spawn_weather_particles` allocates new mesh and material assets via `meshes.add()`/`materials.add()` on every frame it runs. Rain spawns a new `Rectangle` mesh + `StandardMaterial` per batch call, snow similarly. On weather-active frames this is fresh allocations into the Bevy asset store each frame, triggering change detection unnecessarily. The project's standard pattern is to pre-cache shapes in `SpellVisualAssets` (see `particle_quad` at `spells/visual_assets.rs:865`). | Extract rain and snow mesh + material handles into the `MeteorologistPlugin`'s `OnEnter` setup (or a dedicated `WeatherVisualAssets` resource), cache them at game start, and clone the handles for each particle spawn. This reduces frame-time allocation to zero for the common case. |
| W-A-02 | DocDrift | `meteorologist/visuals.rs:11-12` | Medium | S | `spawn_weather_overlays` carries the doc comment "Applies the Drought healing reduction to a heal amount. Returns the (possibly reduced) heal amount." — copied verbatim from `state.rs::apply_dry_healing_reduction`. Completely wrong. | Replace with a correct doc: "Spawns the fullscreen sky-tint UI overlay node used by all weather conditions." |
| W-A-03 | DocDrift | `meteorologist/effects.rs:20-21` | Medium | S | `storm_lightning` carries the same wrong doc comment "Applies the Drought healing reduction to a heal amount." — another copy/paste artifact. | Replace with a correct doc summarizing the function's actual behavior (random lightning strike on a living unit, AoE splash, visual beam). |
| W-A-04 | Performance | `meteorologist/effects.rs:83` | Low | S | `let target_count = targets.iter().len();` iterates all entities to count them (O(n)), then `targets.iter_mut().nth(target_index)` iterates again up to the selected index (O(n) worst case). Two full scans per strike frame. | Collect target entities into a `Vec` once, then index randomly. This also makes the intent (random uniform selection) obvious and avoids the double iteration. |
| W-A-05 | ConsistencyRot | `arcanorouter/systems.rs:81` | Low | S | `BASE_SPELL_RANGE` is defined as a function-local const with value `3000.0` — identical to `wizard::constants::DEFAULT_SPELL_RANGE` (also 3000.0, `src/game/units/wizard/constants.rs:21`). Silently diverges if tuning changes one. | Replace with `use crate::game::units::wizard::constants::DEFAULT_SPELL_RANGE;` and reference that constant. |
| W-A-06 | ArchitecturalDecay | `meteorologist/state.rs:233` | Low | S | `spread_shock_to_wet` (shock propagation logic) lives in `state.rs` alongside weather-input handling and timer ticking. `state.rs` is named as state management but mixes damage-propagation concerns. | Move `spread_shock_to_wet` to `effects.rs` where the other damage-dealing weather effects live (`storm_lightning`, `update_burning_patches`). Low priority — file is under 300 lines. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `swordcerer/networking.rs` | 408 | No | Mixed concerns: spawn receive (`receive_swordcerer_spawn`), guest input stream (`send_swordcerer_avatar_input`, `apply_guest_avatar_input`), and death lifecycle (`check_guest_avatar_death`, `receive_swordcerer_death`). Proposed: `networking/spawn.rs`, `networking/input.rs`, `networking/death.rs`. |
| `meteorologist/effects.rs` | 389 | No | Contains: lightning strike simulation, burning patch tick + visuals, lightning visual cleanup, weather status cleanup, weather SFX management — 6 distinct concerns. Proposed: `lightning.rs` (strike + visual), `patches.rs` (burning ground), `cleanup.rs` (teardown + sfx). |
| `swordcerer/ui.rs` | 388 | No | Mixed concerns: cooldown ticking, avatar death detection, health bar spawn/update/despawn, Enter-the-Fray button + handlers, avatar spawn helper. Proposed: `health_bar.rs`, `enter_fray.rs`, `avatar_spawn.rs`. |
| `arcanorouter/resources.rs` | 311 | Yes | Single cohesive struct `ArcanoRouterState` with math methods and unit-test block. All lines serve one concern. |

---

### Looks Bad But Is Actually Fine

- **Drain-and-re-push pattern** in `swordcerer/networking.rs` and `meteorologist/networking.rs`: Verbose but is the established codebase convention for discriminated network message dispatch. Safe.
- **`spawn_weather_particles` using `rand::rng()` (non-seeded)**: Intentional and documented — visual-only particles must not consume `GameRng` to avoid MP desync.
- **`apply_weather_status` querying `Health` without `Without<GhostEntity>`**: Gated by `is_gameplay_running` which is host-only in MP. The host does not spawn `GhostEntity` units (those only exist on the guest side). Safe.
- **`check_hitscan_collisions` missing `Without<GhostEntity>`**: Gated on `any_exist::<HitscanRay>()`. Rays are only spawned by the local Warglock's firing systems; the opponent's peer never spawns them, so no cross-peer ghost damage occurs.
- **`PsychopathPlugin` only registering `AppState::InGame`**: Psychopath is explicitly blocked from multiplayer selection in the UI. The missing `AppState::MultiplayerGame` is intentional, not an oversight.
- **`send_swordcerer_avatar_input` draining fire/swing messages before the early-return**: Correct by design — prevents a pre-deploy click from leaking into the first on-field frame.

---

### Open Questions

1. **Weather particle asset pooling**: Should rain/snow mesh handles go into a new `WeatherVisualAssets` resource or be added to the existing `SpellVisualAssets`?
2. **`swordcerer/networking.rs` co-location preference**: The file mixes 4 networking concerns at 408 LOC. Is splitting into spawn/input/death subfiles preferred, or is co-location intentional given all fns share the `SwordcererAvatar` context?
3. **`apply_weather_status` per-frame iteration**: Status application iterates ALL living entities every frame regardless of whether weather changed. Could this be throttled to only when `WeatherState::is_changed()` fires? The wet-timer refresh in `update_weather_intensity` may require it to run each frame regardless.
