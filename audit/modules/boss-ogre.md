## boss-ogre

**Scope:** `src/game/units/boss/ogre/` — all 8 `.rs` files (1,607 total LOC)

---

### Mental model

The ogre is a melee boss with three interlocking behaviors: a charge ability (telegraph → dash → recovery state machine), a rock-throw ranged attack (shared with the `brute` via `RockThrowCooldown`), and an enrage system (3 HP-threshold phases that buff speed, damage, and sprite tint). Logic is split across `charge.rs` (732 LOC, charge + rock-throw) and `combat.rs` (548 LOC, spawn + facing + targeting + melee + movement + enrage). `systems.rs` is a 4-line re-export shim; `plugin.rs` is clean registration-only. Core components are well-modelled (small, ECS-idiomatic). All Update systems are gated with `run_if(is_gameplay_running)`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| O-01 | ArchitecturalDecay | `combat.rs:182` | High | S | `update_ogre_targeting` queries `(With<Boss>, Without<Lich>)` — it accidentally matches Hag, DarkMage, and Ray entities which also carry `Boss + TargetingVelocity`. In BossParade mode multiple boss types coexist, so this system clobbers their dedicated targeting systems' results (last write in `VelocitySystemSet` wins non-deterministically). | Narrow the query to `With<OgreEnrageState>` (an ogre-exclusive component) instead of `Without<Lich>`. |
| O-02 | TypeContract | `combat.rs:273,333` | High | S | `ogre_combat` uses two inconsistent melee-range checks: the first pass triggers an attack when any enemy is within `(radius_sum) * ATTACK_RANGE_MULTIPLIER` (1.5×), but the second pass only deals damage within bare `radius_sum` (1.0×). The ogre can swing, play SFX, and reset its cooldown for zero damage when enemies are in the 1.0–1.5× band. | Apply the same range formula (`* ATTACK_RANGE_MULTIPLIER`) in both passes, or unify into a single pass that also applies damage when `distance <= attack_range`. |
| O-03 | TypeContract | `combat.rs:112` | Medium | S | `MeleeRangeBonus(OGRE_MELEE_RANGE_BONUS)` is inserted on the ogre at spawn but is never consumed. The shared `melee.rs` system excludes `Boss` entities, and `ogre_combat` computes its own range from raw `hitbox.radius` values. The constant `OGRE_MELEE_RANGE_BONUS = 80.0` is dead weight. | Either remove `MeleeRangeBonus` from the ogre spawn bundle (and delete `OGRE_MELEE_RANGE_BONUS`) or incorporate it into the `ogre_combat` range calculation (`boss_hitbox.radius + target_hitbox.radius + melee_range_bonus`). |
| O-04 | ArchitecturalDecay | `charge.rs:732` | Medium | M | `charge.rs` is 732 LOC and handles three distinct concerns: the charge state machine (`ogre_charge_system`), charge visuals (`update_ogre_charge_visuals`), and the rock-throw system (`ogre_rock_throw` + `ogre_throw_release`). Several private helpers are interleaved throughout. | Split into `charge.rs` (state machine only), `charge_visuals.rs` (visual updates), and `rock_throw.rs` (throw targeting + release). Helper functions (`ogre_frame_uv_transform`, `facing_from_world_direction`, `ogre_combat_animation`) move to a sibling `utils.rs` or `animation.rs`. |
| O-05 | ArchitecturalDecay | `combat.rs:548` | Medium | M | `combat.rs` is 548 LOC covering five distinct concerns: spawning, facing override, targeting, melee combat, movement, and enrage. The module comment says "spawn, facing, targeting, combat, movement, enrage" — six concerns. | Split into `spawn.rs`, `movement.rs`, `melee.rs` (or reuse name `combat.rs` for just the combat part), and keep `enrage.rs` as its own file. |
| O-06 | DocDrift | `charge.rs:26` | Low | S | `ogre_charge_system` has the doc comment `/// Spawns the ogre at one of the tunnel spawn points.` — copied from `spawn_ogre` and never updated. The function is actually the charge state machine driver. | Replace with an accurate description of the charge state machine. |
| O-07 | DocDrift | `systems.rs:1` | Low | S | The module-level comment reads `//! Re-export hub for ogre systems split (Phase 15).` "Phase 15" is an internal planning artifact with no meaning to readers of the codebase. | Remove the phase reference: `//! Re-export hub — re-exports systems from charge and combat submodules.` |
| O-08 | TypeContract | `combat.rs:447` | Low | S | `let combined_haste = Some(haste_modifier.map(...).unwrap_or(0.0) + enrage_state.speed_bonus)` always produces `Some(f32)` — the `Option` wrapper adds no information and the `unwrap_or` inside `Some` is redundant. `calculate_weighted_movement` treats `None` and `Some(0.0)` identically (both resolve to `unwrap_or(0.0)`), so the only functional difference is that passing `Some(0.0)` prevents a short-circuit that doesn't exist anyway. | Simplify to `let combined_haste = haste_modifier.map(|m| m.modifier).unwrap_or(0.0) + enrage_state.speed_bonus;` and pass it as `Some(combined_haste)` to make the intent explicit, or change the shared function signature to accept `f32` for this parameter. |

---

### Oversized files

| File | LOC | Exempt | Reason | Proposed split |
|------|-----|--------|--------|----------------|
| `charge.rs` | 732 | No | Three distinct concerns (charge state machine, charge visuals, rock throw) plus helpers | `charge.rs` (state machine), `charge_visuals.rs`, `rock_throw.rs`, `ogre_animation.rs` (helpers) |
| `combat.rs` | 548 | No | Six distinct concerns (spawn, facing, targeting, melee, movement, enrage) | `spawn.rs`, `facing.rs`, `targeting.rs`, `melee.rs`, `movement.rs`, `enrage.rs` |

---

### Looks bad but is actually fine

- **`camera_query.single().ok().unwrap_or(Vec3::NEG_Z)` (`charge.rs:404–408`)** — uses `.ok()` to convert `Result` to `Option` then falls back to a default; not a misuse of `.unwrap()`. A missing camera is gracefully handled.
- **`_boss_entity` unused binding (`charge.rs:85`)** — intentional suppression of the unused-variable lint; the entity is destructured but the charge logic uses other fields. Correct Rust idiom.
- **`HashSet` inside `OgreChargeState::Charging` (`components.rs:75`)** — a per-frame allocation concern at first glance, but the `HashSet` is created once when entering the `Charging` state and reused for the duration of the charge. Not a hot allocation.
- **Two-level `.insert()` chain in `spawn_ogre` (`combat.rs:65–114`)** — works around Bevy's tuple-size limit (max 15 items per bundle). Idiomatic workaround, not a design smell.
- **`RockThrowCooldown` borrowed from `brute::components` (`charge.rs:18`, `combat.rs:17`)** — reusing brute's cooldown type is intentional code sharing; the ogre's rock throw uses the same boulder infrastructure as the brute.
- **Long query parameter lists in `ogre_charge_system` and `ogre_rock_throw`** — CC-check tuples are necessarily long; `#[allow(clippy::too_many_arguments)]` is already applied and matches project conventions for Bevy systems.
- **`ogre_movement` runs on `With<Boss>` (no ogre-specific filter)** — safe because the query also requires `OgreEnrageState` and `OgreChargeState`, which only the ogre carries. The query will match nothing when the ogre is absent.

---

### Open questions

1. Should `update_ogre_targeting` be narrowed to `With<OgreEnrageState>` consistently, or is there a plan to replace it with a shared boss-targeting utility that each boss type opts into?
2. Is the intentionally wider melee trigger range (ATTACK_RANGE_MULTIPLIER) in `ogre_combat`'s first pass a design choice (swing-but-miss behaviour), or an unnoticed inconsistency from the two-pass implementation?
3. Is BossParade mode considered a supported/tested configuration for multi-boss coexistence, or a debug/experimental toggle where cross-boss system interference is accepted?
