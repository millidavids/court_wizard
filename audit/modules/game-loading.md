## game-loading

**Scope:** `src/game/loading/` — queue-based progressive spawn system that builds the SP battlefield in ~4 frames.

---

### Mental model

The loading module converts "start a level" into a sequence of granular `SpawnTask` items processed in a tight `while` loop inside `process_spawn_queue` (Update, gated by `run_if(in_state(AppState::Loading))`). `init_loading_progress` (OnEnter Loading) builds a `VecDeque<SpawnTask>` covering battlefield geometry, terrain obstacles, all defender and attacker units, wizard, cauldron, and post-spawn upgrade passes. The queue respects Bevy's command-flush boundary via `SpawnTask::needs_command_flush` / `creates_deferred_state`. On queue completion, the system triggers `AppState::InGame` with an optional co-op "both peers loaded" handshake inline. Constants and helpers for elite/commander upgrade selection live in `constants.rs`, `upgrade_selection.rs`, and `upgrade_systems.rs` — shared with `wave_systems.rs`. `terrain_generation.rs` is a standalone pure-Rust placer reused by `game/multiplayer/loading.rs`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| L01 | ArchitecturalDecay | `queue.rs:304–306` | Medium | S | `SpawnTask::Castle` is a no-op — its match arm contains only a comment. The variant is enqueued at `init/world_setup.rs:79` but performs zero work because castle geometry is already spawned inside `setup_battlefield`. | Remove the `Castle` variant from `SpawnTask`, its `push_back` in `world_setup.rs:79`, and its empty match arm. Add the ordering comment before the `Battlefield` arm instead. |
| L02 | ArchitecturalDecay | `init/world_setup.rs:220–226` | Medium | S | The `_ =>` wildcard arm of `match tier % BOSS_CYCLE_LENGTH` (which silently spawns an Ogre) is unreachable. The outer guard `!is_lich_level(level)` already excludes residue 1, so all five residues (0–4) are covered by named arms or excluded. The wildcard hides future boss-cycle expansion errors. | Replace `_ =>` with `1 => unreachable!("lich levels are excluded by the outer guard")` to make the exhaustive intent explicit, matching the canonical `boss_name_for_level` match. |
| L03 | ConsistencyRot | `queue.rs:371,394,417,439,461,483` | Medium | S | Each of the six `Select*Upgrades` match arms re-declares `let level = current_level.0; let seed_base = level as u64;` identically — 12 duplicate let-bindings inside the while loop. `current_level` is immutable during loading. | Hoist both bindings above the `while !spawn_queue.is_complete()` loop at line 132. |
| L04 | ArchitecturalDecay | `upgrade_systems.rs:103–106` | Medium | S | `apply_commander_upgrade` declares `_materials: &mut Assets<StandardMaterial>` and `_meshes: &mut Assets<Mesh>` (underscore-prefixed, never read). Callers in both `queue.rs:558–559` and `wave_systems.rs` must pass these assets solely to satisfy the signature. | Remove the two dead parameters and update both call sites. |
| L05 | ArchitecturalDecay | `queue.rs:513–526` and `551–565` | Low | S | `UpgradeToElite` and `UpgradeToCommander` arms both manually extract `transform` and `hitbox` via sequential ParamSet access with identical boilerplate comments. The shared helper `upgrade_systems::get_transform_and_hitbox` exists for exactly this purpose and is already used in `wave_systems.rs` but not here. | Call `upgrade_systems::get_transform_and_hitbox` in both arms, eliminating the duplicated copy pattern. |
| L06 | DocDrift | `init/world_setup.rs:343,373,376` | Low | S | Step-numbering comments are inconsistent: "// 8. Defender Archers" appears at line 343 after step 8 was already used at line 263 for attacker archers; both cauldron steps are labelled "// 13."; steps 7, 9, 10 are skipped. | Renumber step comments to match actual queue order, or drop numbering in favor of descriptive inline comments. |
| L07 | ArchitecturalDecay | `systems.rs:1` | Low | S | `systems.rs` is a single-line stale comment with no symbols. It mentions "Phase 18" — an internal refactor tag that is now noise. The module is declared in `mod.rs` but exports nothing. | Delete `systems.rs` and remove `mod systems;` from `mod.rs`. |
| L08 | DocDrift | `queue.rs:28` | Low | S | The doc-comment on `process_spawn_queue` reads "Initializes the loading progress tracker and spawn queue" — copied from the old monolith before `init.rs` was split out. It describes `init_loading_progress`, not this function. | Update the doc-comment to describe per-frame queue draining. |
| L09 | DocDrift | `init/world_setup.rs:198–199` | Low | S | Lines 198–199 contain inner `use crate::game::constants::{BOSS_CYCLE_LENGTH, get_tier}` and `use crate::game::constants::{is_boss_level, is_lich_level}` inside the function body. These symbols are already in scope from the module-level glob `use crate::game::constants::*` at line 7. The re-imports imply they are not covered by the glob, confusing readers. | Remove the two inner `use` statements. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| `queue.rs` | 706 | Yes | Single large match-on-enum dispatching every `SpawnTask` variant to a thin per-unit spawn helper. Every line is part of that one dispatch operation. Qualifies for the match-on-enum exemption. The co-op handshake (lines 638–684) is a distinct concern but is too small to warrant extraction on its own. |
| `init/world_setup.rs` | 389 | No | Mixes three concerns: (1) co-op session promotion/handshake setup, (2) queue population for world/terrain/units, (3) resource initialization (WaveState, RetreatState, InitialDefenderCount). Proposed split: `coop_session.rs` for lines 43–65; `wave_init.rs` for WaveState + wave-count bookkeeping; keep terrain+unit enqueue in `world_setup.rs`. |
| `terrain_generation.rs` | 428 | Yes | Single cohesive terrain generator function (`generate_terrain`) plus one private placement helper (`try_place`). All 428 lines serve one algorithm. |

---

### Looks bad but is actually fine

- **`process_spawn_queue` with ~20 injected parameters**: Idiomatic Bevy; the system dispatches across 30+ spawn task variants and needs every asset type. `ParamSet` is used correctly for query conflicts. `#[allow(clippy::too_many_arguments, clippy::type_complexity)]` is appropriate.
- **`resources.rs` is a comment-only file**: Unlike the dead `systems.rs`, `resources.rs` is intentional tombstone documentation explaining why `LoadingProgress` was removed and what to reach for if it returns. It is load-bearing documentation.
- **`SpawnQueue.tasks` is `pub`**: Needed for direct push/iteration by task-selector arms in the same crate. Accessor boilerplate would add no value here.
- **`try_place` retries 80 times**: Fixed retry count is intentional probabilistic placement. Silently returning `None` is correct graceful-degradation behavior.
- **`select_from_pool` seeds RNG as `seed_base.wrapping_mul(seed_multiplier)`**: Looks fragile but is intentional. Each call site uses a distinct prime multiplier (997, 1009, 1013, 1019, 1031) to separate unit-type RNG streams deterministically from a single level seed.
- **`Res::clone(ogre_assets)` etc.**: Required because boss spawn functions take `Res<T>` by value; the clone is a cheap Arc handle clone.
- **Co-op handshake inlined in `process_spawn_queue` (lines 638–684)**: Runs only after `is_complete()`, so it doesn't pollute the hot dispatch loop. Technically a second concern, but too small to justify a separate file.

---

### Open questions

1. **`SpawnTask::Castle` removal safety**: Is Castle geometry fully covered by `setup_battlefield`, or was the variant a placeholder for future roguelite castle upgrades? Confirm before removing.
2. **`apply_commander_upgrade` unused params (L04)**: Were `_materials` and `_meshes` retained for a planned commander color-swap visual that never shipped? Removing forecloses that option — a comment would help.
3. **`init/world_setup.rs` split (L oversized proposal)**: Is the co-op session promotion logic (lines 43–65) better extracted now, or should it stay co-located with the queue-build it feeds? The concern is small but structurally different from queue population.
