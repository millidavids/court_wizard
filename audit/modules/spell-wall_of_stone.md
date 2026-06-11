## spell-wall_of_stone

**Scope:** `src/game/units/wizard/spells/wall_of_stone/`

---

### Mental model

Wall of Stone is a drag-to-place spell that spawns oriented bounding box (OBB) walls on the battlefield. Walls block unit pathfinding (via `ObstacleChanged` messages to the flow-field grid), rise from the ground with an animation, accept melee damage from units who have no valid path around them, and decay on a lifetime timer (currently `f32::MAX`, swept at level end). A rich talent tree (Quarry Master, Reinforced Stone, Quick Foundations, Jagged Stone, Permafrost Aura, Living Stone, Collapsing Wall, Terraformer, Maze Architect) gates optional behavioural systems behind `WallTalents` / `LivingStoneTracker` component run_if guards. In multiplayer, ghost copies of the host's walls are spawned on the guest by `guest_visuals.rs`; the guest sees the same `WallOfStone` + `WallRising` components but no `WallHealth` / `WallTalents`, keeping the gameplay systems safely inert on the guest.

The module is cleanly split: `casting.rs` owns placement logic, `lifecycle.rs` owns all running systems (cancel, tick, animate, talent effects, VFX), `wall_material.rs` owns the custom shader material, and `systems.rs` is a thin re-export hub. `plugin.rs` is registration-only.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| W01 | DocDrift | `lifecycle.rs:25` | Low | S | Doc comment `/// Computes talent parameters from active talent selections.` is copy-pasted from `casting.rs:29` and precedes `handle_wall_of_stone_cancel` — completely wrong function description. | Replace with `/// Cancels an in-progress Wall of Stone drag on right-click.` |
| W02 | ArchitecturalDecay | `systems.rs:1` | Low | S | Module-file comment `//! Re-export hub for wall_of_stone systems split (Phase 14).` is a stale internal refactoring note. Phase numbers are meaningless to future readers. | Change to `//! Re-exports all wall_of_stone systems from casting and lifecycle submodules.` |
| W03 | ArchitecturalDecay | `lifecycle.rs:70-78` | Low | S | `animate_sinking_walls` queries all `WallOfStone` entities and filters `if wall.sinking` in the loop body, iterating all standing walls every frame for no purpose. The caller is already gated by `any_exist::<WallOfStone>()`, not `any_with_component::<SinkingWall>`. | Either add a marker component `SinkingWall` (consistent with `WallRising`) and narrow the query, or at minimum add an inline comment explaining why a full scan is acceptable. |
| W04 | ArchitecturalDecay | `lifecycle.rs:588-629` | Medium | S | `spawn_wall_dust` accepts a `sinking_walls: Query<&WallOfStone, Without<WallRising>>` and then filters `if wall.sinking` in the body. This iterates every non-rising wall (which includes the majority of standing walls at rest) every dust-interval tick. All standing, non-sinking walls are visited for zero effect. | Add a dedicated `SinkingWall` marker component (parallel to `WallRising`) so the query can be narrowed to `Query<&WallOfStone, With<SinkingWall>>`, eliminating the dead scan. Alternatively inline a note that the population is bounded (≤10 walls) and this is intentional. |
| W05 | Performance | `lifecycle.rs:469-500` | Medium | S | `maze_architect_bonus` iterates all `WallOfStone` entities **twice per frame** (first pass to count + detect `has_maze`, second pass to apply HP adjustment). It runs under `any_with_component::<WallTalents>`, which fires whenever any wall exists with talents — including every frame of normal play. The double-scan is avoidable and the HP adjustment logic inside (`if (health.max - expected_max).abs() > 0.1`) fires every frame when active. | Use a single iteration with a two-phase split: collect `(entity, expected_max, hp_fraction)` tuples, then update. Or cache the bonus-active state in a `Resource` that is recomputed only when wall count changes (on `ObstacleChanged`). |
| W06 | ArchitecturalDecay | `lifecycle.rs:425-465` | Low | M | `collapsing_wall_explosion` performs a full `O(walls × enemies)` nested loop scan on every `PostCombatSet` tick while any `WallTalents` entity exists, even though walls die infrequently. The `CollapseExploded` marker prevents double-fire but does not prevent the outer loop scanning all live walls on every frame. | Move the outer guard into the query: add `Without<CollapseExploded>` AND gate on `WallHealth::is_dead()`. The outer loop is already filtered, but since this runs every frame, pulling the dead check earlier (`continue if !health.is_dead()`) removes the inner enemy scan for healthy walls. This is effectively done in the current code but a documentation comment would clarify intent. The real fix is an observer/trigger on `WallHealth` reaching 0, but that is a larger architectural change. |
| W07 | TypeContract | `components.rs:239-255` | Low | S | `WallOfStoneTalentParams` derives only `Clone` — no `Debug`, no `Default` declared explicitly (it has a manual `Default` impl). Because it is embedded in `WallTalents(pub WallOfStoneTalentParams)` which is `#[derive(Component, Clone)]`, the absence of `Debug` means `WallTalents` cannot be `Debug`-printed in Bevy inspector tools or log macros. | Add `#[derive(Debug)]` to `WallOfStoneTalentParams` and `WallTalents`. |
| W08 | ArchitecturalDecay | `casting.rs:400-403` | Low | S | In the Quick Foundations loop, `result.obstacle_bounds` is overwritten on every iteration — only the **last** wall's bounds are stored and synced to the network. In the 2-wall case only the second wall triggers pathfinding update on the remote peer; the first wall's bounds are silently lost. | Collect both bounds and send two `WallPlaced` messages (one per wall), or send a combined AABB. The comment "Use the last wall's bounds for network sync" acknowledges this but does not fix the partial sync. |
| W09 | Multiplayer | `lifecycle.rs:82-119` | High | M | `cleanup_expired_walls` sends `NetworkMessage::WallPlaced{placed:false}` to the remote peer when a wall expires. Since walls use `duration = f32::MAX`, this path is currently unreachable in practice. However the code path survives and will silently fire if duration ever changes. On the **guest**, ghost walls have `duration = f32::MAX` received from the host (transmitted in the snapshot), so the path is safe today — but the system contains no `Without<GhostSpellEffect>` guard to document the invariant and prevent future regressions. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the `walls` query filter in `cleanup_expired_walls`. Add a comment explaining why this guard is needed. Same applies to `destroy_dead_walls` (though ghost walls lack `WallHealth`, making it safe today). |
| W10 | ArchitecturalDecay | `casting.rs:140-147` | Low | S | `WallOfStoneCaster` is lazily inserted on the wizard entity on the first frame the spell is cast, causing the system to `return` early on that one frame. This means the very first mouse click to start casting silently drops the input for one frame. Other spells that need similar caster state typically insert the component at wizard-spawn time (or on `OnEnter` for the spell). | Insert `WallOfStoneCaster` in the wizard spawn bundle or in an `OnEnter(AppState::InGame)` system, removing the lazy-insert fallback. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `lifecycle.rs` | 657 | No | Mixes cancel, tick, sink animation, VFX dust, wall-attack, dispel, HP-destroy, damage tint, and four talent effect systems. Proposed split: `cancel.rs` (cancel + 1 fn), `tick.rs` (tick_wall_lifetime + animate_sinking_walls + cleanup_expired_walls), `combat.rs` (units_attack_blocking_walls + destroy_dead_walls + collapsing_wall_explosion), `vfx.rs` (animate_rising_walls + apply_wall_trampling + spawn_wall_dust + update_wall_damage_tint), `talents.rs` (apply_permafrost_aura + regenerate_living_stone + maze_architect_bonus). |
| `casting.rs` | 431 | No | Contains both the public system (`handle_wall_of_stone_casting`) and a substantial private helper (`wall_of_stone_casting_logic` ~150 lines), plus `compute_talent_params` and the `CastResult` struct. Proposed split: `casting.rs` (public system + CastResult + wall_of_stone_casting_logic), `talents.rs` (compute_talent_params + WallOfStoneTalentParams logic — though this overlaps with the lifecycle split proposal). |
| `components.rs` | 332 | Yes | Just above the 300-line threshold but contains the substantial `WallOfStone` impl block (push_out, line_segment_intersects, obstacle_bounds, closest_point_on_surface, distance_to_surface, slab_intersect — all genuinely cohesive geometry methods). The impl is a single-concern geometry block; the remainder is component declarations. Borderline but defensible as cohesive. |

---

### Looks bad but is actually fine

- **`slab_intersect` returning `NEG_INFINITY/INFINITY`** for the parallel-inside case: this is correct SAT (Separating Axis Theorem) interval arithmetic; the caller's `t_enter.max(0.0)` bounds it properly.
- **`WallOfStone.empowerment` field comment says "stored for potential future use"**: this is intentional forward storage for save-data round-tripping (`SavedWall.empowerment`). Not dead code.
- **`wall_material.rs` multiple `#[uniform(0)]` bindings**: Bevy's `AsBindGroup` packs sequential same-slot uniforms into a single buffer struct; this is idiomatic, not a collision.
- **`maze_architect_bonus` setting `health.max` and `health.current` every frame**: the `abs() > 0.1` early-out means it is a no-op for the common case of healthy walls. Acceptable given the small wall population cap.
- **`handle_wall_of_stone_casting` having 15+ parameters**: `#[allow(clippy::too_many_arguments)]` is present and this is idiomatic Bevy system injection. Not a violation.
- **`apply_permafrost_aura` allocating `frost_walls: Vec<_>`** each tick: the tick interval is 0.5s (not per-frame), and the wall count is tiny. This Vec allocation is negligible.
- **`update_wall_damage_tint` cloning a material handle on first damage**: this is an explicit lazy-clone pattern documented in the function comment; it is intentional to avoid N per-frame material hash lookups on undamaged walls.

---

### Open questions

1. **Quick Foundations + network sync (W08)**: Was the decision to send only the last wall's bounds to the remote peer intentional (the remote peer rebuilds both walls from the full snapshot anyway)? If so, a comment should document this. If not, it is a genuine partial sync bug in the `WallPlaced` removal path.
2. **Ghost walls and `cleanup_expired_walls` (W09)**: Is the invariant "ghost walls always have `duration = f32::MAX`" guaranteed to hold as the network protocol evolves? If Terraformer walls are ever given a finite duration, ghost walls would start sending spurious removal messages to the host.
3. **`animate_sinking_walls` runs on ghost walls** on the guest: ghost walls have `sinking = false` permanently (no `tick_wall_lifetime` is driving them since they never expire). Is the guest visual intended to show sinking at all, or is the host expected to send a dispel/remove message before the ghost wall disappears?
4. **`WallOfStoneCaster` one-frame drop (W10)**: Has this ever been noticed as a gameplay issue (first click sometimes not registering the anchor point)? The lazy-insert is clean defensively but introduces a subtle UX bug.
