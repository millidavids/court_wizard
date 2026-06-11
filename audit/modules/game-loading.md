## game-loading

**Scope:** `src/game/loading/` — queue-based progressive spawn system that builds and drains a `SpawnQueue` over a handful of frames to transition from `AppState::Loading` to `AppState::InGame`.

---

### Mental model

The loading module converts "start a level" into a sequence of granular `SpawnTask` items that are processed in a tight `while` loop inside `process_spawn_queue` (Update, gated by `run_if(in_state(AppState::Loading))`). The queue respects Bevy's command-flush boundary: tasks that read deferred World state set a `needs_command_flush` flag so the loop breaks and continues on the next frame. `init_loading_progress` (OnEnter Loading) builds the queue in spawn order; `cleanup_loading_progress` (OnExit Loading) removes it. Constants and helper functions for elite/commander upgrade selection live in `constants.rs`, `upgrade_selection.rs`, and `upgrade_systems.rs` — all shared with `wave_systems.rs` in the parent module. Terrain generation is isolated in `terrain_generation.rs` and called from both the SP loading path and the MP loading path (`game/multiplayer/loading.rs`).

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| L01 | ArchitecturalDecay | `queue.rs:304-306` | Medium | S | `SpawnTask::Castle` is enqueued in `init.rs:81` but the match arm in `queue.rs` is a no-op (`// Castle is spawned as part of battlefield setup`). The task variant exists solely to document an ordering intent that is no longer enforced by actual work. | Remove the `Castle` variant from `SpawnTask` and its enqueue call in `init.rs:81`. If the ordering comment is meaningful, add it as a comment before the `Battlefield` arm instead. |
| L02 | ArchitecturalDecay | `init.rs:221-226` | Medium | S | The `_` wildcard arm of the `match tier % BOSS_CYCLE_LENGTH` block (which spawns an Ogre) is unreachable dead code. The enclosing guard on line 203 is `is_boss_level(level) && !is_lich_level(level)`. `is_lich_level` is exactly `tier % 5 == 1`, so tier residue 1 is excluded before the match is ever reached. The wildcard therefore silently hides any future boss-cycle expansion error. | Replace `_ =>` with an explicit `1 =>` arm that `unreachable!("lich levels are excluded by the outer guard")`, and add a comment. This makes the exhaustive coverage intent clear and will surface bugs if `BOSS_CYCLE_LENGTH` changes. |
| L03 | DocDrift | `init.rs:375,378` | Low | S | Both `// 13. Load cauldron assets` and `// 13. Cauldron` use the same step number "13". The step numbering sequence also skips 7, 9, 10 and jumps from 6 to 8, then from 8 to 11. | Renumber the step comments to match the actual enqueue order, or drop the numbering entirely in favor of descriptive inline comments (numbering already drifts every time a step is added). |
| L04 | DocDrift | `init.rs:198-199` | Low | S | Lines 198-199 contain inner `use crate::game::constants::{BOSS_CYCLE_LENGTH, get_tier}` and `use crate::game::constants::{is_boss_level, is_lich_level}` statements inside the function body. These symbols are already in scope from the module-level glob `use crate::game::constants::*` at line 7. The re-imports are redundant and imply those items were not covered by the glob, which confuses readers. | Remove the two inner `use` statements. |
| L05 | ArchitecturalDecay | `upgrade_systems.rs:101-142` | Medium | S | `apply_commander_upgrade` accepts `_materials: &mut Assets<StandardMaterial>` and `_meshes: &mut Assets<Mesh>` (prefixed with underscore, never read). These were likely placeholders for a material-swap path that was replaced by the `UnitTypeGlow` component. The parameters are still threaded through from the call site in `queue.rs:555-566`, forcing `queue.rs` to keep `meshes` and `materials` in mutable scope for this one call. | Remove `_materials` and `_meshes` from `apply_commander_upgrade` and update the call site in `queue.rs`. |
| L06 | ArchitecturalDecay | `queue.rs:513-526` and `queue.rs:550-566` | Low | S | Both `UpgradeToElite` and `UpgradeToCommander` arms in `queue.rs` manually copy `transform` and `hitbox` via sequential ParamSet access with identical boilerplate comments (`"ParamSet requires sequential access — copy transform before querying hitbox"`). The helper `upgrade_systems::get_transform_and_hitbox` exists precisely for this purpose and is used from `wave_systems.rs` but not here. | Call `upgrade_systems::get_transform_and_hitbox` in both arms, passing the ParamSet projections as individual query references. This removes the duplication and keeps the shared helper consistent. |
| L07 | ArchitecturalDecay | `systems.rs` | Low | S | `systems.rs` is a single-line comment file (`//! Re-export hub for loading systems split into init.rs + queue.rs (Phase 18).`) with no actual content. It is a module declaration stub that serves no runtime purpose and mentions an internal refactor phase tag ("Phase 18") that is now stale noise. | Remove `systems.rs` and its `mod systems;` declaration from `mod.rs`. The file is empty of any exported symbols; the module declaration in `mod.rs` is the only reason it compiles. |
| L08 | TypeContract | `queue.rs:29` | Low | S | `process_spawn_queue` doc-comment (line 28) still reads "Initializes the loading progress tracker and spawn queue" — copied verbatim from the old monolith before `init.rs` was split out. It describes the wrong function's responsibility. | Update the doc-comment to describe the actual purpose: draining the spawn queue in per-frame batches. |
| L09 | ArchitecturalDecay | `init.rs:7` | Low | S | `use crate::game::constants::*` is a glob import. Combined with the stale inner re-imports (L04) this makes it non-obvious which symbols come from constants vs local scope. In a file that already cherry-picks specific items below the glob, the glob is the root cause of the L04 redundancy. | Replace the glob with explicit named imports from `crate::game::constants`. This is mechanical but makes the dependency surface visible and removes the redundancy. |

---

### Oversized files

| File | LOC | Exempt | Reason | Proposed split |
|------|-----|--------|--------|----------------|
| `queue.rs` | 706 | Yes | The file is a single large `match` dispatch over every `SpawnTask` variant. Every arm is a thin delegating call to a per-unit spawn helper with no logic of its own. This fits the "single large match-on-enum" exemption exactly. | N/A |
| `init.rs` | 391 | No | `init_loading_progress` mixes three distinct concerns: (1) co-op session promotion and handshake setup, (2) queue population for world/terrain/units, and (3) resource initialization (WaveState, RetreatState, InitialDefenderCount). | `coop_session.rs` — co-op session promotion logic (lines 43-67); `wave_init.rs` — WaveState and wave-count bookkeeping; keep terrain + unit enqueue in `init.rs` |
| `terrain_generation.rs` | 428 | Yes | The file is a single cohesive terrain generator function (`generate_terrain`) plus one private placement helper (`try_place`). All 428 lines serve a single algorithm. | N/A |

---

### Looks bad but is actually fine

- **`queue.rs` system has ~20 injected parameters** — This is an idiomatic Bevy system that needs every asset type available to dispatch across 30+ spawn task variants in one function. The `ParamSet` is used correctly to work around query conflicts. The `#[allow(clippy::too_many_arguments, clippy::type_complexity)]` suppression is appropriate.
- **`resources.rs` is a comment-only file** — Unlike `systems.rs`, `resources.rs` is intentional: it documents why the `LoadingProgress` resource was removed and what to reach for if it's needed again. The comment is load-bearing documentation, not dead code.
- **`SpawnQueue.tasks` is `pub`** — The field is accessed from `queue.rs` (same crate, `pub(crate)` module) and needs direct push/iteration by the task-selector arms. Keeping it pub avoids boilerplate accessor methods while the type is only constructed in this module.
- **`try_place` in `terrain_generation.rs` retries 80 times** — The fixed retry count is intentional: placement is probabilistic and 80 attempts is enough to fill realistic densities without an unbounded loop. Silently returning `None` on failure is the right graceful-degradation behavior.
- **Two `// 13.` comments + skipped numbering in `init.rs`** — Flagged as DocDrift (L03), but the underlying spawn logic is correct; this is purely a maintenance cosmetic issue.
- **`select_from_pool` seeds RNG as `seed_base.wrapping_mul(seed_multiplier)`** — The multiplication-based seed mixing looks fragile, but it is intentional and consistent across all upgrade selection calls. Each call site uses a distinct prime multiplier (997, 1009, 1013, 1019, 1031) to separate unit-type RNG streams deterministically from a single level-based seed.

---

### Open questions

1. **Should `SpawnTask::Castle` be removed or converted to a real task?** The castle is currently spawned implicitly inside `setup_battlefield`. If in future the castle needs independent spawning (e.g., for roguelite castle upgrades), the variant would need a real handler. The current no-op silently hides that the step does nothing.
2. **Is the `init.rs` splitting (L proposal for `coop_session.rs` / `wave_init.rs`) worth the churn?** The file is 391 lines, slightly above the 300-line guideline, but the three concerns are not deeply interleaved — extraction would be mechanical and low-risk.
3. **`apply_commander_upgrade` unused params (L05)** — Were `_materials` and `_meshes` kept for a planned commander color-swap visual that never shipped? Removing them simplifies the signature but forecloses that option without comment.
