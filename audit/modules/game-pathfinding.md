## game-pathfinding

**Scope:** `src/game/pathfinding/` — 11 files, 2,570 LOC total.

---

### Mental model

The module implements a continuous flow-field pathfinding system using Dijkstra's algorithm on an 800×600 cell grid (10-unit cells, 480k cells). Three dynamic fields are kept perpetually fresh by spawning background async rebuilds immediately after each one finishes: an *attacker* field (toward the Defender King), a *defender* field (toward King's nearest enemy or spawn center), and an *assassin* field (toward the archer centre-of-mass, with infantry marked as high-cost terrain). Seven static *staging* fields guide unactivated attackers to pre-assigned rallying points before each wave activates. `FlowFieldVelocity` is sampled every frame in `VelocitySystemSet` and blended with flocking/targeting velocities in the movement layer. A stuck-detection system applies perpendicular nudge forces to units that stop moving for >1.5 s. The wave-staging subsystem handles attacker tagging, 90%-threshold activation, timeout force-activation, and 3× staging speedup. Multiplayer is handled correctly: staging systems are gated `in_state(AppState::InGame)` (SP-only), ghost entities carry no `FlowFieldInfluence` component so the sampling query skips them, and `manage_staging_speedup` hard-resets speed when an MP state is active.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `setup.rs:150–472` | High | M | `setup.rs` conflates four unrelated concerns: grid initialisation (lines 30–116), flow-field kick-off (124–141), wave-staging lifecycle (144–312), and stuck-unit recovery (390–472). The file is 472 LOC and fails the project's "split by concern" rule. `WaveStagingTimers` (a staging concern) is defined here instead of `staging.rs`. `suppress_staging_targeting`, `init_stuck_detection`, and `detect_and_recover_stuck_units` live here even though they have no setup dependency. | Extract: `wave_activation.rs` (`tag_new_attackers`, `check_wave_activation`, `manage_staging_speedup`, `suppress_staging_targeting`, `WaveStagingTimers`) and `stuck_detection.rs` (`init_stuck_detection`, `detect_and_recover_stuck_units`). Keep `setup.rs` for grid init, terrain registration, staging field construction, and `generate_initial_fields`. |
| F2 | ArchitecturalDecay | `runtime.rs:1–499` | Medium | M | `runtime.rs` (499 LOC) merges three distinct responsibilities: continuous rebuild management (`continuous_flow_field_rebuild`, the three `spawn_*` helpers), obstacle event handling (`handle_obstacle_events`), and field sampling (`sample_flow_fields`, `rally_to_spawn`, `sample_field_or_zero`). The rebuild-vs-sampling split is conceptually clean and worth making explicit. | Split into `rebuild.rs` (rebuild orchestration + `spawn_*` helpers) and `sampling.rs` (`sample_flow_fields`, `sample_field_or_zero`, `rally_to_spawn`). Move `handle_obstacle_events` to `rebuild.rs` since it updates the cost state that drives rebuilds. |
| F3 | ArchitecturalDecay | `messages.rs:15–18` | Low | S | The `rebuild: bool` field of `ObstacleChanged` is decorated `#[allow(dead_code)]` and the doc-comment explicitly states it "is no longer needed." It is set at ~30 call-sites across the codebase but never read. This is dead state that every call-site must populate. | Remove the field and all `rebuild: …` struct literal entries. If a future explicit-trigger mode is desired, add it back then. |
| F4 | ArchitecturalDecay | `resources.rs:24–25` | Low | S | `PathfindingGrid::world_max` carries `#[allow(dead_code)]` and is not read anywhere outside its constructor. It is stored in the resource purely so it can be computed with `world_min` during construction. | Either remove the field and inline the `grid_width`/`grid_height` calculation during `new()`, or keep it without the `pub` visibility since it is never consumed externally. |
| F5 | ArchitecturalDecay | `systems.rs:1–4` | Low | S | `systems.rs` is a two-line re-export shim from `setup.rs`. Its doc comment calls itself a "re-export hub for pathfinding systems split into feature files (Phase 18)", implying it was a migration artefact. The `pub use super::setup::*` wildcard re-export makes every public symbol from `setup.rs` reachable as `systems::*`, which is the API surface `mod.rs` and `game/plugin.rs` actually use. Once the F1 split lands, this file should be re-evaluated for removal or proper curation. | After splitting `setup.rs`, enumerate the systems that need to be accessible at the module boundary in `systems.rs` explicitly (no wildcard), or expose them directly from `mod.rs`. |
| F6 | ConsistencyRot | `runtime.rs:348` | Low | S | `DEFENDER_RALLY_DELAY_SECS` is defined at line 348, well after its only use at line 212. All other constants in this codebase sit near the top of the file or alongside the code that owns them. | Move the constant to the top of `runtime.rs` (or to a future `rebuild.rs`) alongside the other staging-related constants. |
| F7 | ConsistencyRot | `staging.rs:52–53` | Low | S | `WaveStagingPlan::has_wave` iterates all `HashMap` keys with `.keys().any(|(w, _)| *w == wave)` instead of two `O(1)` `contains_key` probes. `remove_wave` uses `retain` (also O(n)). With at most ~20 entries this is harmless, but it is inconsistent with the rest of the codebase's preference for direct key lookup. | Replace `has_wave` with `self.wave_points.contains_key(&(wave, SpawnTunnel::Left)) \|\| self.wave_points.contains_key(&(wave, SpawnTunnel::Right))`. |
| F8 | TestDebt | `staging.rs` | Medium | M | `compute_wave_staging` and `WaveStagingPlan::next_staging_point` are deterministic, pure functions with branching staging-point selection logic that is exercised at runtime every wave. No unit tests exist. The seed-based RNG makes the output fully predictable for a given `(seed, level, wave)` tuple, which is ideal for table-driven tests. | Add tests: verify `compute_wave_staging` for both tunnels selects only from the allowed ranges; verify round-robin cycling in `next_staging_point`; verify `remove_wave` clears state. |
| F9 | TestDebt | `setup.rs:214–311` | Low | M | `check_wave_activation` contains the 90%-threshold and timeout activation logic — the most game-play-critical logic in the module. No tests cover the threshold boundary, the timeout path, or the swordcerer-aggro early activation branch. | Add integration-light unit tests with manually populated `Query` data (or extract the threshold calculation into a pure function that can be tested without ECS). |
| F10 | Performance | `flow_field.rs:339` | Low | S | `smooth_with_lic` clones `self.directions` (a `Vec<Vec3>` of 480k elements → ~5.76 MB) on every field rebuild to read originals while writing smoothed values. Rebuilds happen on background threads, so this does not stall the main thread, but it means each rebuild allocates and then drops ~5.76 MB of heap. | Use a second pre-allocated scratch buffer stored on `FlowField` (or passed in) and swap, avoiding the per-rebuild heap churn. |
| F11 | ErrorObservability | `setup.rs:390–399` | Low | S | `suppress_staging_targeting` reads `targeting.velocity` and `targeting.distance_to_target` every frame for every staging attacker, even when both are already zeroed. The fast-path guard `if targeting.velocity != Vec3::ZERO \|\| targeting.distance_to_target != f32::MAX` is correct but the whole system runs unconditionally inside `VelocitySystemSet` (which is guarded by `is_gameplay_running`). The system does have correct gating through VelocitySystemSet, but is not itself gated on there being any staging attackers — add an early-return on `query.is_empty()` to match the pattern used in `check_wave_activation`. | Add `if query.is_empty() { return; }` at the top of `suppress_staging_targeting`. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|-------------------------|
| `flow_field.rs` | 517 | Yes | Single cohesive type (`FlowField`) with Dijkstra, LIC smoothing, and unit tests — every line belongs to the same algorithm. |
| `runtime.rs` | 499 | No | Three distinct concerns: rebuild orchestration, obstacle handling, field sampling. Propose `rebuild.rs` + `sampling.rs`. |
| `setup.rs` | 472 | No | Four distinct concerns: grid init, flow-field kick-off, wave-staging lifecycle, stuck-unit recovery. Propose `setup.rs` (grid+terrain), `wave_activation.rs` (staging lifecycle), `stuck_detection.rs` (stuck recovery). |
| `debug.rs` | 356 | No | Two separate debug tools (flow-field arrows and the F4 debug ball + staging markers). Propose `debug_flow_field.rs` + `debug_ball.rs`. |
| `resources.rs` | 303 | Yes | Single type (`PathfindingGrid`) — all methods operate on the same grid resource. Borderline at 303 lines but every line belongs to the one resource type. |

---

### Looks bad but is actually fine

- **`manage_staging_speedup` not gated on `is_gameplay_running`** — intentional. The system must fire when the player opens the spell book (which exits `Running`) so it can *drop* the 3× clock back to 1×. The plugin comment explains this clearly.
- **`handle_obstacle_events` not gated on `is_gameplay_running`** — intentional. It runs during loading so terrain spawned via the spawn queue is registered before the first frame of gameplay.
- **`init_stuck_detection` and `detect_and_recover_stuck_units` having no explicit `run_if`** — they live inside `VelocitySystemSet` which is configured with `run_if(is_gameplay_running)` at the set level in `game/plugin.rs:174-175`. The per-system gate is unnecessary.
- **Wildcard import `use super::runtime::*` in `plugin.rs`** — acceptable; all symbols are `pub` and the module is small enough that the glob doesn't obscure the dependency surface.
- **Duplicate `fn index()` on `FlowField` and `PathfindingGrid`** — both are private `impl` methods on different types with different backing field names (`width` vs `grid_width`). This is idiomatic Rust, not a sharing opportunity.
- **`sample_flow_fields` runs on all MP host units without `Without<GhostEntity>` guard** — ghost entities do not receive `FlowFieldInfluence` components (confirmed in `guest_snapshot.rs`), so the query filter is naturally exclusive of ghosts.
- **`tag_new_attackers` / `check_wave_activation` lack `Without<GhostEntity>`** — both systems run `in_state(AppState::InGame)` which is SP-only (`AppState::MultiplayerGame` is the MP root state). No MP ghost can reach these systems.
- **`next_staging_point` uses `.expect()`** — the panic message is descriptive and the invariant (call `compute_wave_staging` first) is enforced by `tag_new_attackers` which calls `compute_wave_staging` before calling `next_staging_point`. This is a correct use of `.expect()` for an invariant.

---

### Open questions

1. **Staging fields are built synchronously at startup** (`build_staging_fields` in `setup.rs:54`). With 7 fields × ~480k cells each that is roughly 67 ms of Dijkstra on the main thread at game load. Is this inside the loading-screen budget, or should staging fields also be built async?
2. **`DEFENDER_RALLY_DELAY_SECS = 2.0`** is a balance constant. Should it be exposed in `GameConfig` so it can be tuned without a rebuild?
3. **Assassin field rebuild strategy** — the assassin field avoids attacker (friendly) infantry with a wide radius and defender (enemy) infantry with a narrower radius. The radius constants live in `units/assassin/constants.rs`. Is this the right home, or should they live with the field-rebuild logic in pathfinding?
4. After the F1/F2 split, `systems.rs` becomes a pure shim over two or three files. Should it be eliminated entirely and have `mod.rs` re-export from the concern files directly?
