## boss-ray

**Scope:** `src/game/units/boss/ray/` — 8 files, 2,571 total LOC.

---

### Mental Model

Ray is a floating multi-eyed boss. Its body is a damage-redirect sentinel (all damage dealt to the body is divided evenly among its 5 eyes). Each eye has a unique beam attack: disintegration (sweeping burn), petrification (channel + cone), fear (orbit drop-beam), mind control (channel + cone), and teleportation (scatter defenders when Ray is in melee). Eyes die independently; Ray dies when all 5 eyes are dead. Stalk particles fly from the body to each live eye for visual flavor.

The module is split into `movement.rs` (spawn, body/eye lifecycle, particle, fear movement, disintegration/petrification beams — 1102 LOC) and `beams.rs` (mind control, fear, teleport beams, cone helpers — 923 LOC). Both files are well above 300 LOC. `components.rs` holds 13 components (220 LOC). `resources.rs` holds asset preloading (108 LOC). `plugin.rs` is pure registration (102 LOC). `constants.rs` is pure constants (99 LOC). `systems.rs` is a one-liner re-export hub (4 LOC).

The module is guarded by `is_gameplay_running` (which is host-only in multiplayer) so ghost-entity contamination is not a concern for gameplay systems.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `movement.rs:1` | High | M | `movement.rs` (1102 LOC) conflates five distinct concerns: spawn, body/eye lifecycle, stalk particles, fear movement, and two full beam attacks (disintegration + petrification). The scope note calls this out explicitly. | Split into `spawn.rs` (spawn_ray), `lifecycle.rs` (body_damage_to_eyes, eye_death_check, all_eyes_dead_check, death_cleanup, dying_eyes), `particles.rs` (stalk particle spawn/update, beam_visual cleanup), `fear_movement.rs` (cleanse_fear_with_rage, update_fear_movement), and move disintegration + petrification beams into `beams.rs`. |
| F2 | ArchitecturalDecay | `beams.rs:1` | Medium | M | `beams.rs` (923 LOC) holds four unrelated beam attacks plus three shared helper functions. The module name implies it is already the dedicated beam file, but it is still over-sized. | Move disintegration and petrification beams out of `movement.rs` into `beams.rs`, then split `beams.rs` by attack: `beams/disintegration.rs`, `beams/petrification.rs`, `beams/fear.rs`, `beams/mind_control.rs`, `beams/teleport.rs`, `beams/helpers.rs`. |
| F3 | ConsistencyRot | `components.rs:116-168` | Medium | S | `RayPetrificationBeam` and `RayMindControlBeam` are structurally identical (`origin: Vec3`, `direction: Vec3`, `length: f32`, `channel_progress: f32`, `has_fired: bool`). The corresponding `*Sweep`, `*Glow` component triples are also near-clones. The two beam systems that drive them (petrification in `movement.rs:901`, mind control in `beams.rs:28`) duplicate 80+ lines of channel/steer/fire logic. | Extract a generic `ChanneledBeam { origin, direction, length, channel_progress, has_fired }` component (or a typed enum variant), plus a shared `channel_beam_logic` helper, reducing duplication to per-beam overrides (damage type, cone width, effect applied). |
| F4 | ArchitecturalDecay | `movement.rs:627` / `movement.rs:640` | Medium | S | `cleanse_fear_with_rage` and `update_fear_movement` are cross-cutting systems (FearModifier is a units-level concept, registered in `units/plugin.rs`). They live inside `movement.rs` only because Ray is the only current fear source, but they are not Ray-specific and create an invisible ownership oddity. | Move both systems to `src/game/units/status_effects.rs` or a dedicated `fear.rs` under `units/` alongside the `FearModifier` definition, and register them in `units/plugin.rs` instead. |
| F5 | ConsistencyRot | `movement.rs:709-723` / `movement.rs:942-955` / `beams.rs:82-95` | Medium | S | The "does any defender exist within MAX_BEAM_RANGE?" check (`has_targets`) is inlined identically in at least three beam attack functions (disintegration, petrification in `movement.rs`, mind control in `beams.rs`). | Extract `any_defender_in_range(boss_pos, defenders, team_query, max_range)` into `beams.rs` helpers or `boss/utils.rs`. |
| F6 | DocDrift | `beams.rs:26-27` | Low | S | The doc comment `/// Attenuated volume for Ray's sound effects — slight falloff from wizard/camera position.` is placed directly above `ray_mind_control_beam` (line 28) but describes `ray_sfx_volume`, which lives in `movement.rs`. The comment was clearly left behind after `ray_sfx_volume` was moved. | Delete the orphaned comment from `beams.rs:26-27`. |
| F7 | Performance | `movement.rs:389-392` | Low | S | `update_ray_eye_movement` allocates a `Vec<(Entity, Vec2)>` every frame for the 5-eye separation pass. With only 5 eyes this is trivially small, but it is a needless heap allocation in a hot system. | Replace with a `[Option<(Entity, Vec2)>; 5]` stack array populated in-place. |
| F8 | Performance | `beams.rs:682-689` | Low | S | `ray_teleport_eye` calls `materials.add(StandardMaterial { ... })` each time the teleport fires (once every 15 seconds). While infrequent, the teleport bubble material is stateless and identical every time. | Add a `bubble_material: Handle<StandardMaterial>` field to `RayAssets` and create it in `preload_ray_assets`, eliminating the runtime allocation. |
| F9 | TypeContract | `components.rs:42-47` | Low | S | `RayEyeState::new()` is a manual constructor that returns `active: [true; COUNT]`. The type does not derive `Default`. Since `new()` is trivially equivalent to `Default::default()`, callers must know to call `new()` rather than using any derive-based initialization path. | `#[derive(Default)]` on `RayEyeState` with `active: [true; COUNT]` using a custom `Default` impl, and replace the `new()` call sites. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|-------------------------|
| `movement.rs` | 1102 | No | Split into: `spawn.rs`, `lifecycle.rs`, `particles.rs`, `fear_movement.rs`; move beam systems to `beams.rs`. |
| `beams.rs` | 923 | No | Split into: `beams/disintegration.rs`, `beams/petrification.rs`, `beams/fear.rs`, `beams/mind_control.rs`, `beams/teleport.rs`, `beams/helpers.rs`. |
| `components.rs` | 220 | Yes | Under 300 LOC; all entries are Ray-specific components — genuinely cohesive. |

---

### Looks Bad But Is Actually Fine

- **`#[allow(clippy::too_many_arguments)]` on beam systems** — Bevy systems with many injected `Query`/`Res` parameters are idiomatic; the project CLAUDE.md explicitly calls this out.
- **`unwrap_or(boss_pos)` at eye-position lookups** (`movement.rs:732`, `movement.rs:964`, `beams.rs:104`, `beams.rs:438`, `beams.rs:631`)** — These fall back to `boss_pos` when the expected eye entity is not found; this is a safe, semantically correct fallback for a transient race during eye death, not a panic path.
- **`partial_cmp(...).unwrap_or(Ordering::Equal)` at `beams.rs:664`** — This is the standard Rust pattern for NaN-safe `f32` comparison in a sort; acceptable here since the values are distances (non-NaN in practice).
- **`RayEyeState` holding `[bool; 5]` indexed by `RayEyeType::index()`** — Looks fragile, but the `ALL` const array and `index()` method are defined in the same `components.rs` and form a closed enum-indexed array; no out-of-bounds risk.
- **`is_gameplay_running` as the sole multiplayer gate (no `Without<GhostEntity>`)** — `is_gameplay_running` returns `true` only for the multiplayer host, so Ray's gameplay systems never run on the guest client. The ghost-entity gating pattern seen in `hags/abilities.rs` is not needed here.
- **`systems.rs` being a 4-line re-export hub** — This is explicitly the project's "Phase 15" split residue. The file is intentional; not a mod.rs purity violation since it is `systems.rs` not `mod.rs`.
- **Duplicate `find_units_in_cone` and `find_units_in_cone_filtered`** — These are not duplicates in the strict sense: the unfiltered version takes a `&mut Health` query (needed for damage reads), while the filtered version takes a `&Hitbox` query (excludes King/KingsGuard/Petrified). They serve genuinely different callers.

---

### Open Questions

1. Are Ray's beam attacks ever expected to run on the multiplayer guest (for visual mirroring)? If so, the ghost-entity gating omission is a real gap; if not (boss is host-only), the current design is correct.
2. Should `cleanse_fear_with_rage` / `update_fear_movement` be promoted to `units/` now, or deferred until a second fear source is added?
3. `PETRIFY_DURATION` and `MIND_CONTROL_DURATION` are both `f32::MAX`. Is this intentional (permanent effects)? If so, a named constant like `PERMANENT_DURATION` would communicate intent better.
