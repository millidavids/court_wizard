## game-pathfinding

**Scope:** `src/game/pathfinding/` — flow-field pathfinding, wave staging, stuck detection, obstacle handling, and debug visualization.

---

### Mental Model

The pathfinding module implements a continuous-rebuild flow-field system. A `PathfindingGrid` resource holds base terrain costs, and three main fields (attacker, defender, assassin) plus a set of static staging fields are maintained as `Option<FlowField>`. On every frame where a field has no pending async task, a new `bevy::tasks::AsyncComputeTaskPool` task is spawned to regenerate it via Dijkstra + bilinear gradient + LIC smoothing. Completed tasks are polled the following frame and swapped in. Units query `FlowFieldVelocity` (sampled each frame in `VelocitySystemSet`) and `StuckDetection` (nudge recovery). Wave staging drives a pre-activation phase where attacker units march to staging points at 3x game speed before activating. The module is well-factored into sub-crates: `runtime/`, `setup/`, `debug/`, with `systems.rs` acting as a re-export shim.

The system is sound and the MP guard concern (ghost entities) is not a real problem here: ghost units never receive `FlowFieldInfluence`/`FlowFieldVelocity`, and the rebuild systems are filtered to host-only via `is_gameplay_running`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| PF-01 | ArchitecturalDecay | `pathfinding/systems.rs:1-4` | Low | S | `systems.rs` is a four-line re-export shim (`pub use super::setup::*`). It exists solely so external callers (`plugin.rs`, `loading/queue.rs`, `multiplayer/loading.rs`) can spell `pathfinding::systems::initialize_pathfinding` and `pathfinding::systems::suppress_staging_targeting`. This is non-idiomatic: `mod.rs` is the conventional re-export hub, and the indirection obscures where symbols actually live. | Remove `systems.rs`. Add the re-exports directly to `mod.rs`. Update the three call-sites to use `pathfinding::setup::initialize_pathfinding` (or via the `mod.rs` re-export). |
| PF-02 | TypeContract | `pathfinding/messages.rs:16-18` | Medium | S | `ObstacleChanged.rebuild: bool` is `#[allow(dead_code)]` and its comment says it is "no longer needed but kept for API compatibility." The field is hardcoded to `false` at 15+ call sites and `true` at 3 (wall-of-fire), but `handle_obstacle_events` never reads it. Carrying dead fields in a frequently-constructed message struct creates maintenance confusion and silent contract drift. | Remove the `rebuild` field. Update all ~18 construction sites (trivial `rebuild: false` drops and three `rebuild: true` in wall_of_fire — no functional replacement needed since continuous rebuilds already handle it). |
| PF-03 | ArchitecturalDecay | `pathfinding/resources.rs:24-25` | Low | S | `PathfindingGrid.world_max: Vec2` has `#[allow(dead_code)]`. It is used during construction to compute `grid_width`/`grid_height`, but is never read after that. | Remove the field. The computed grid dimensions are already stored separately and are what callers need. |
| PF-04 | Performance | `pathfinding/setup/wave_staging.rs:116-118` | Medium | S | `check_wave_activation` allocates a `HashMap<u32, (u32, u32)>` and a `HashSet<u32>` on every frame while any staging attackers are alive (potentially hundreds of frames per wave). The early-exit at line 97 skips work post-activation, but the hot path during staging always allocates. | Declare both collections as `Local<HashMap<...>>` and `Local<HashSet<...>>` system parameters and call `.clear()` at the top of each invocation. Reuses heap allocation across frames. |
| PF-05 | ArchitecturalDecay | `pathfinding/flow_field.rs:331-396` | Low | M | `smooth_with_lic` has a guard at line 335 that performs a full O(N) scan of the `costs` array (`self.costs.iter().any(|&c| c > 1.0 && !c.is_infinite())`) on every call to decide whether to skip smoothing. For a field with no elevated terrain, this scans every cell to immediately return early — paying full cost to confirm nothing needs doing. | Track whether elevated terrain exists as a flag on `FlowField` (set in `create_field_with_base_costs` or when costs are applied). Pass it to `smooth_with_lic` or store it on the struct. Eliminates the linear scan for the common case. |
| PF-06 | ConsistencyRot | `pathfinding/staging.rs:60-62` | Medium | S | `WaveStagingPlan::next_staging_point` panics with `expect(...)` if called before `compute_wave_staging`. The caller `tag_new_attackers` already lazily initializes the plan before calling this, so the invariant holds in practice. But the method is `pub`, and any future caller that forgets the precondition will get a runtime panic in production. | Return `Option<u8>` or document with a `debug_assert!` + return a safe default. The `expect` in production code is a violation of the project convention. |
| PF-07 | DocDrift | `pathfinding/setup/grid_init.rs:111-115` | Low | S | The doc-comment on `generate_initial_fields` reads "Initializes the pathfinding grid resource and registers static terrain obstacles" — which describes `initialize_pathfinding`, not `generate_initial_fields`. Copy-paste doc drift. | Fix the doc-comment: "Spawns the initial attacker flow field rebuild task when the Defender King first appears." |
| PF-08 | TestDebt | `pathfinding/setup/wave_staging.rs` | Low | M | `compute_wave_staging`, `WaveStagingPlan::next_staging_point`, and `check_wave_activation` contain the core wave distribution and activation logic — high-churn gameplay code. `flow_field.rs` has tests; staging has none. | Add unit tests covering: correct staging point counts per tunnel, round-robin cycling, timeout-based activation, and swordcerer aggro activation. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `flow_field.rs` | 517 | true | Single cohesive algorithm: Dijkstra integration, bilinear gradient direction generation, LIC smoothing, and sampling. All methods operate on the single `FlowField` struct. Test suite at line 450 adds ~67 lines. Genuinely cohesive. |
| `runtime/flow_field_rebuild.rs` | 344 | false | Mixes three concerns: polling/scheduling (`continuous_flow_field_rebuild`), king target tracking (`update_king_target` + `tick_defender_rally_delay`), and three async spawn-task helpers. Proposed split: `rebuild_poll.rs` (polling macro + continuous rebuild orchestrator), `king_target.rs` (king target tracking + rally delay), `rebuild_spawn.rs` (spawn_attacker/defender/assassin helpers). |
| `resources.rs` | 303 | true | Single struct `PathfindingGrid` with constructor and methods. All 303 lines belong to the type: constructor, bounds conversion, obstacle marking, cost sampling, field factory, shape filtering. Borderline but exempt. |

---

### Looks Bad But Is Actually Fine

- **`flow_field.rs` line 40 — `partial_cmp(...).unwrap_or(Ordering::Equal)`**: Standard pattern for floating-point `BinaryHeap` ordering. NaN costs would break pathfinding well before this, and the fallback is safe.
- **`systems.rs` as a re-export hub**: Not technically a `plugin.rs` purity violation (the rule targets `plugin.rs`), but the indirection is still confusing. Flagged as PF-01 but lower severity.
- **`create_field_with_base_costs` cloning `base_costs`** (resources.rs:199): Fires only when `pending_*_rebuild.is_none()`, i.e., approximately once per completed async task, not every frame. Acceptable.
- **Per-frame `Vec` allocation in `continuous_flow_field_rebuild` lines 131-143**: The `atk`/`def` Vecs are only allocated when `pending_assassin_rebuild.is_none()`, gated behind the prior task completing. Not a steady per-frame allocation.
- **No `GhostEntity` filters on `sample_flow_fields` and `detect_and_recover_stuck_units`**: Ghost units never receive `FlowFieldVelocity` or `FlowFieldInfluence` (confirmed in `apply_state_snapshot.rs`), so these queries return no ghost rows. Explicit `Without<GhostEntity>` filters would be noise.
- **`manage_staging_speedup` not gated by `is_gameplay_running`**: Intentional. The plugin comment explains it must run outside `Running` state to drop 3x speed when menus open.
- **`index()` defined on both `FlowField` and `PathfindingGrid`**: Both are `z * width + x` but on different types referencing different field names. Not worth extracting — a free function would be less clear.

---

### Open Questions

1. **`flow_field_rebuild.rs:125`** — `if target_pos != Vec2::ZERO` skips spawning the assassin field when the target equals the world origin. Is `Vec2::ZERO` a valid battlefield position? If the lava pool or a staging point lands at origin, the assassin field silently stops rebuilding.
2. **Staging satisfaction radius** — `STAGING_SATISFACTION_RADIUS` is uniform across all 7 staging points. Staging point 3 (shared by both tunnels, `CENTER_STAGING_INDEX`) sees double the traffic. Should it have a wider radius to avoid units from both streams jamming the same cell?
3. **`continuous_flow_field_rebuild:89`** — When all enemies die and `king_current_target` is cleared, the rally rebuild fires immediately on the next frame (when `pending_defender_rebuild.is_none()`). The `DEFENDER_RALLY_DELAY_SECS` timer in `tick_defender_rally_delay` only delays the *clearing* of `king_current_target`. But `continuous_flow_field_rebuild` checks `king_current_target.is_none()` and `defender_field.is_some()` directly — so the moment the delay expires and `king_current_target` is set to `None`, the rally field rebuild begins immediately. Is this the intended timing, or should the delay also suppress the rally rebuild for its full duration?
