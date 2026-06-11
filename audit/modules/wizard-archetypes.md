## wizard-archetypes

**Scope:** `src/game/units/wizard/archetypes/` — all eight wizard archetype modules.

---

### Mental model

Eight wizard archetypes (RuneCaster, Randomancer, Arcanorouter, Gunslinger/Warglock, Swordcerer, Meteorologist, Shepherd, Psychopath) each live in their own sub-directory. The module structure is genuinely feature-sliced: each archetype has dedicated `plugin.rs`, `systems.rs`/`combat.rs`/`ui.rs`, `components.rs`, `resources.rs`, `constants.rs`, and optional `networking.rs`/`replication.rs` files. All `Update` systems are properly guarded with `run_if`. Multiplayer ghost-gating is largely correct. The largest files are `gunslinger/fire.rs` (739 LOC), `swordcerer/combat.rs` (619 LOC), and `swordcerer/networking.rs` (408 LOC); the first two are cohesive single-concern files but may benefit from splitting. The main recurring issues are: a copy-paste doc comment in two meteorologist files, an unused system parameter, a per-frame mesh/material allocation hot path in the particle system, and a duplicated drain-and-requeue pattern across three swordcerer networking functions.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| 1 | DocDrift | `meteorologist/effects.rs:21`<br>`meteorologist/visuals.rs:11` | Medium | S | Both `storm_lightning` in `effects.rs` and `spawn_weather_overlays` in `visuals.rs` carry the identical stale doc comment `/// Applies the Drought healing reduction to a heal amount. Returns the (possibly reduced) heal amount.` — clearly a copy-paste leftover that describes neither function. | Replace with accurate doc comments: `effects.rs:21` → describe `storm_lightning`; `visuals.rs:11` → describe `spawn_weather_overlays`. |
| 2 | TypeContract | `meteorologist/state.rs:134` | Medium | S | `apply_weather_status` declares `_units_with_wet: Query<Entity, With<WetModifier>>` but never reads it (prefixed `_` to suppress the warning). The comment above the system explains that wet is intentionally NOT removed immediately, but the query does no work at all — it adds Bevy query overhead every frame for zero gain. | Remove the `_units_with_wet` parameter entirely. |
| 3 | Performance | `meteorologist/visuals.rs:175–212` | High | M | `spawn_weather_particles` runs every frame and calls `meshes.add(Rectangle::new(...))` and `materials.add(StandardMaterial {...})` once per weather slot per frame that has active weather (rain: 8 per frame × 2 slots max; snow: 6 per frame × 2 slots max). Each `meshes.add` / `materials.add` inserts a new asset into the asset server, growing `Assets<Mesh>` and `Assets<StandardMaterial>` unboundedly over a match. | Cache the particle mesh and material handles in a resource (similar to how `SpellVisualAssets` pre-builds all spell meshes). Spawn the resource on `OnEnter(Running)` and clone the handles each frame instead of re-allocating. Alternatively, store the handles as `Local<Option<Handle<…>>>` and create on first use. |
| 4 | ArchitecturalDecay | `swordcerer/networking.rs:56–78`<br>`swordcerer/networking.rs:257–280`<br>`swordcerer/networking.rs:395–407` | Medium | M | The "drain + match + collect unhandled + extend back" message-routing pattern is copy-pasted three times in the same file (`receive_swordcerer_spawn`, `apply_guest_avatar_input`, `receive_swordcerer_death`). The same pattern also appears in `meteorologist/networking.rs:85–110`. Across the codebase this is at least four identical sites. | Extract a generic helper function (e.g., in `networking/resources.rs` or `game/shared_systems.rs`): `fn drain_matching<T, F: FnMut(NetworkMessage) -> Option<T>>(conn: &mut NetworkConnection, mut handler: F) -> Vec<T>` — or at minimum extract it as a free function local to `swordcerer/networking.rs`. |
| 5 | ArchitecturalDecay | `gunslinger/fire.rs` (739 LOC) | Low | M | `fire.rs` is a well-scoped file (all gun firing + projectile lifecycle) but at 739 LOC it is large enough that the hitscan, tracer, and flame sections could be separated. Not a hard violation since every line is cohesive with firing, but it is past the 300 LOC guideline. | Consider splitting into `hitscan.rs` (hitscan ray spawn + collision), `tracer.rs` (tracer spawn + movement + cleanup), and `flamethrower.rs` (flame particle spawn + collision + vfx + ground fire). Keep `fire.rs` as the per-gun entry points that delegate to these. |
| 6 | ArchitecturalDecay | `swordcerer/combat.rs` (619 LOC) | Low | M | `combat.rs` mixes avatar physics helpers (`apply_avatar_physics`, `spawn_avatar_missile`), input systems (`player_movement`, `fire_missile`, `sword_swing`), sword arc VFX (`build_arc_strip_mesh`, `spawn_sword_arc`, `update_sword_arcs`), and retreat logic. Over 300 LOC and has at least three distinct concerns. | Split into `avatar_movement.rs` (physics + input), `sword_arc.rs` (mesh building + spawn + update), and keep `combat.rs` for the remaining retreat/block logic or merge it with `ui.rs` retreat. |
| 7 | ArchitecturalDecay | `arcanorouter/systems.rs:81` | Low | S | `BASE_SPELL_RANGE: f32 = 3000.0` is defined as an inline `const` inside `apply_bonuses_to_wizard_stats`. This value is the same base spell range used by the wizard system at large. It should live in a shared constants location so it can't silently diverge. | Move `BASE_SPELL_RANGE` to `game/units/wizard/constants.rs` (or the arcanorouter's own `constants.rs`) so both `apply_bonuses_to_wizard_stats` and any other system that may need the wizard's base range use the same source of truth. |
| 8 | ErrorObservability | `gunslinger/components.rs:50,62`<br>`gunslinger/resources.rs:85,91` | Low | S | Four methods on `GunType` (`fire_interval`, `is_hold_to_fire`) and two helpers in `resources.rs` are marked `#[allow(dead_code)]`. Dead code allowed on `pub(super)` items suggests they are either genuinely unused or are exposed purely for potential future use. | Audit whether these methods are actually called. If not, remove them. If they are intended as a public API for external callers (e.g., UI displaying fire interval), make that explicit with a comment; otherwise remove the suppression and the method. |
| 9 | ArchitecturalDecay | `psychopath/plugin.rs:12` | Low | S | `PsychopathPlugin` registers `apply_defender_spell_vulnerability` only on `OnEnter(AppState::InGame)`, not on `OnEnter(AppState::MultiplayerGame)`. Per the project memory notes, the Psychopath is disabled in multiplayer, but if that changes — or if the run condition is the only guard — a multiplayer Psychopath would silently miss the vulnerability application. | Add `OnEnter(AppState::MultiplayerGame)` with the same `run_if(is_psychopath)` guard, or add a comment documenting that Psychopath is explicitly MP-disabled (so the missing hook is intentional). |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `gunslinger/fire.rs` | 739 | false | Single logical concern (gun firing) but four distinct subsystems. Propose: `hitscan.rs`, `tracer.rs`, `flamethrower.rs`, keep `fire.rs` for per-gun entry-point fns. |
| `swordcerer/combat.rs` | 619 | false | Mixes avatar physics, input systems, sword arc VFX, and retreat. Propose: `avatar_movement.rs`, `sword_arc.rs`, slim down `combat.rs`. |
| `swordcerer/networking.rs` | 408 | true | All content is multiplayer network I/O for the single Swordcerer avatar; no mixed concerns. Long but cohesive. |
| `meteorologist/effects.rs` | 389 | true | Contiguous weather simulation logic (lightning, burning patches, cleanup). Slightly large but a single concern. |
| `swordcerer/ui.rs` | 388 | true | All swordcerer UI: health bar, enter-fray button, location click, avatar spawn. Cohesive single-screen concern. |
| `arcanorouter/resources.rs` | 311 | true | Single `ArcanoRouterState` resource with deep business logic + embedded tests; every line is part of the slider redistribution algorithm. Exempt as a single-concern data type + its unit tests. |

---

### Looks bad but is actually fine

- **`swordcerer/ui.rs:59`** — `avatar_query.iter().next()` instead of `single()` in `check_avatar_death` looks like sloppy query use, but the comment explains it avoids a panic in Swordcerer-vs-Swordcerer matches where two avatars can match the query.
- **`swordcerer/combat.rs:131`** — `gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0)` — looks like an `.unwrap()` but is actually `.unwrap_or`, a safe default. Not a violation.
- **`meteorologist/state.rs:127–194`** — `apply_weather_status` takes 8 queries and iterates all living units. Looks like a performance problem, but this system is gated behind `is_gameplay_running` + `is_meteorologist_participating` (not unconditional Update), so it only runs in the relevant configuration and not on every frame in menus.
- **`gunslinger/fire.rs:186`** — using `spawn_fireball_entity` from the `fireball` spell module to implement rockets. Looks like a leaky cross-concern dependency, but reuse of the fireball explosion for rocket detonation is an intentional design decision documented in the comment.
- **`swordcerer/networking.rs:55–77`** — `already_deployed` mutable guard for duplicate spawn messages in the same batch (before Commands flush) looks fragile, but the comment explains the deferred-commands constraint precisely. This is the correct approach given Bevy's deferred entity visibility.
- **`meteorologist/effects.rs:83`** — `targets.iter().len()` before the strike loop. Iterating once to count looks wasteful, but this is only called when a lightning strike fires (not every frame) and `iter().len()` on a Bevy query is O(n) over archetypes, not a full scan of all entities.
- **`arcanorouter/resources.rs:152–207`** — `normalize_excluding_range` uses a manual 4-pass residual loop instead of a solver. This looks overly complex, but the in-code comments and embedded unit tests demonstrate correctness for edge cases where naive proportional scaling undershoots the pool.

---

### Open questions

1. Is the `Psychopath` archetype intentionally disabled for multiplayer permanently, or is it just deferred? The plugin only hooks into `AppState::InGame`; a comment or a `run_if(!is_multiplayer)` guard would make intent explicit.
2. `meteorologist/constants.rs` contains weather bar UI colors (`STORM_COLOR`, `BLIZZARD_COLOR`, `DROUGHT_COLOR`). These are used by the UI layer, not the meteorologist simulation. Should they live in the UI module that consumes them?
3. The "drain + unhandled requeue" pattern in `swordcerer/networking.rs` and `meteorologist/networking.rs` is informal protocol multiplexing. Is there a plan to centralize message routing (e.g., a typed dispatch table per message kind) or is the ad-hoc drain pattern acceptable long-term?
