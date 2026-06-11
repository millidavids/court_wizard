## spell-wall_of_stone

**Scope:** `src/game/units/wizard/spells/wall_of_stone/`

---

### Mental model

Wall of Stone is a click-drag placement spell. The player presses the mouse button to anchor one end, drags to set direction and length, and releases to commit. The placed wall is an OBB (oriented bounding box) collision entity that physically blocks unit pathfinding and can be attacked by stuck units until destroyed. Walls now have an effectively infinite lifetime (`f32::MAX`) and only expire via HP loss, dispel, or level-end sweep.

The module is split into six concern files plus a systems re-export shim. Casting logic lives in `casting/{wizard_system, placement, talents}.rs`. Wall lifecycle (ticking, sinking, despawn, combat, VFX) lives in `lifecycle/{tick, cancel, combat, wall_vfx, talents, permanent}.rs`. The `components.rs` holds all component/resource types plus the rich `WallOfStone` OBB geometry struct. `constants.rs` and `wall_material.rs` are self-contained. `plugin.rs` is registration-only. `systems.rs` is a pure re-export hub.

The module is generally well-structured. The one notable correctness issue is a **Quick Foundations network sync gap** where only the last wall segment's bounds are sent to the remote peer. The other findings are minor hygiene items.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| W1 | TypeContract | `casting/placement.rs:154` | High | S | When `quick_foundations` places two wall segments, `result.obstacle_bounds` is overwritten each loop iteration so only the last segment's bounds reach the caller. `handle_wall_of_stone_casting` then sends a single `WallPlaced` message to the remote peer with only those bounds. The guest's pathfinding grid ends up with only one of the two segments registered as an obstacle. | Send one `WallPlaced` message per segment (or accumulate and send both), matching what the local `ObstacleChanged` path already does. |
| W2 | ArchitecturalDecay | `components.rs:1-332` | Medium | M | At 332 lines, `components.rs` mixes three distinct concerns: (1) the `WallOfStone` OBB geometry struct with six non-trivial geometry methods; (2) caster/preview state types; (3) all talent and status components plus the `PermafrostAuraTimer` resource. Exceeds the 300-line guideline with mixed concerns. | Split into `wall.rs` (the `WallOfStone` struct and OBB methods), `caster.rs` (`WallOfStoneCaster`, `WallOfStonePreview`), and `talent_components.rs` (talent params, `WallTalents`, `LivingStoneTracker`, `PermafrostAuraTimer`, `WallRising`, `CollapseExploded`, `DispelledWall`, `WallHealth` alias). |
| W3 | DocDrift | `lifecycle/cancel.rs:7` | Low | S | The doc comment on `handle_wall_of_stone_cancel` reads "Computes talent parameters from active talent selections." — a stale copy-paste from `casting/talents.rs`. The function actually cancels an in-progress wall drag. | Replace with an accurate doc comment describing the cancel behavior. |
| W4 | Performance | `lifecycle/talents.rs:127-159` | Low | S | `maze_architect_bonus` does two full wall-query passes every frame — first to count and detect the talent, then to adjust HP. It has no change-detection guard: it runs even when wall state has not changed. For typical wall counts this is negligible, but the redundant iteration is avoidable. | Collapse into one pass and add a `Local<(usize, bool)>` tracking last (wall_count, bonus_active) to short-circuit frames where nothing has changed. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| `components.rs` | 332 | No | Mixed OBB geometry + caster state + talent components. Proposed split: `wall.rs`, `caster.rs`, `talent_components.rs`. |

---

### Looks bad but is actually fine

- **`systems.rs` is 4 lines of pure re-exports.** Correct per-project pattern for the Phase 14 split hub. Not a violation.
- **`super::super::super::super::components` import chains** — verbose but consistent with every other spell module at this nesting depth. Not a local defect.
- **Ghost walls being processed by `tick_wall_lifetime`, `animate_sinking_walls`, `spawn_wall_dust`, `animate_rising_walls`** — intentional. Ghost walls use `duration = f32::MAX`, making tick/cleanup no-ops. `animate_rising_walls` and `spawn_wall_dust` are intentionally shared to display the rise animation on the remote peer. `destroy_dead_walls`, `units_attack_blocking_walls`, `update_wall_damage_tint`, `maze_architect_bonus`, `apply_permafrost_aura`, `regenerate_living_stone`, and `collapsing_wall_explosion` all require additional components (`WallHealth`, `WallTalents`, `LivingStoneTracker`) that ghost walls do not carry, so they are safely excluded without explicit `Without<GhostSpellEffect>` filters.
- **`PermafrostAuraTimer` doing a `Vec::collect()` per tick** — ticks only every 0.5 s, bounded by wall count. Not a hot path.
- **`cleanup_expired_walls` sends a `WallPlaced { placed: false }` network message** — walls currently never naturally expire (`duration = f32::MAX`) so this path is dead in normal play, but the code is correct for future use and for dispelled walls.
- **`WallOfStoneTalentParams` is `Clone` but not `Component`** — intentional; it is embedded inside the `WallTalents(WallOfStoneTalentParams)` component wrapper.
- **`maze_architect_bonus` running every frame without `Without<GhostSpellEffect>`** — ghost walls lack `WallTalents`, so they are excluded by the query filter and do not inflate the wall count.

---

### Open questions

1. **Quick Foundations network sync (W1)**: Was the single-bounds `WallPlaced` message for two-segment walls a known limitation? Does playtesting confirm the guest's grid correctly blocks both segments, or is one segment passable on the guest side?
2. **Ghost wall `time_alive` ticking**: Ghost walls accumulate `time_alive` independently of the host. If the host snapshot later carries a different `time_alive`, is there reconciliation, or does the guest rely solely on its local tick?
3. **Permanent walls and `register_permanent_wall_obstacles`**: This `OnEnter(AppState::InGame)` system runs on both host and guest. Does the guest run pathfinding at all (and is the `ObstacleChanged` message ignored), or could this double-register obstacles on the guest's grid?
