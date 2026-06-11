## game-root

**Scope:** `src/game/*.rs` — root-level files only (14 files, 2251 LOC total)

---

### Mental Model

`src/game/` is the orchestration layer for the entire game loop. `plugin.rs` wires all sub-plugins together and registers shared systems into their schedule slots. `shared_systems.rs` is the single largest file and serves as a genuine cross-cutting module: it holds lifecycle cleanup, the global attack-cycle tick, defender activation, unit effectiveness, audio ambience, and the shadow rendering pipeline. `wave_systems.rs` owns the wave spawner and deferred upgrade application. `run_conditions.rs` centralises all gameplay run-condition functions (`is_gameplay_running`, `is_spell_effects_active`, archetype predicates). Supporting files (`resources.rs`, `components.rs`, `messages.rs`, `sets.rs`, `attack_cycle.rs`, `insight_bonuses.rs`, `debug_ui.rs`, `systems.rs`) are focused and well-sized. The constants module is already split into `tuning.rs`, `positions.rs`, `spawn_math.rs`, `wave_tiers.rs`, `colors.rs`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| G-01 | ArchitecturalDecay | `shared_systems.rs:1` | High | M | 536-line file mixes five distinct concerns: (1) lifecycle cleanup (`cleanup_game`, `cleanup_for_replay`, `reset_resources_for_replay`, `init_level_from_config`, `apply_game_speed`, `auto_pause_on_focus_loss`), (2) gameplay ticks (`tick_attack_cycle`, `tick_elapsed_time`, `calculate_effectiveness`, `activate_defenders_on_proximity`), (3) audio ambience (`load_battle_ambience_assets`, `update_battle_ambience`, `update_crowd_ambience`, `stop_all_sfx`), (4) shadow rendering (`ShadowMaterial`, `ShadowAssets`, `preload_shadow_assets`, `spawn_terrain_shadow`, `spawn_unit_shadows`, `update_unit_shadows`), (5) achievement tracking (`track_wizard_enemy_damage`, `wizard_has_not_damaged_enemies`). Project conventions require files >300 LOC to be split unless cohesive. | Split into `ambience.rs` (~130 LOC: audio resources + systems), `shadows.rs` (~120 LOC: `ShadowMaterial` + shadow systems), `lifecycle.rs` (~90 LOC: cleanup + reset + `apply_game_speed` + `auto_pause_on_focus_loss`), keeping `shared_systems.rs` for the remaining gameplay cross-cuts (~180 LOC). Update `plugin.rs` imports. |
| G-02 | ConsistencyRot | `run_conditions.rs:88,138,172` | Medium | S | The `matches!` arm `MultiplayerGameState::Running \| Paused \| Settings \| SpellBook \| CauldronMenu` is copy-pasted verbatim in three separate functions: `is_gameplay_running` (line 88), `is_local_wizard_active` (line 138), and `is_spell_effects_active` (line 172). Adding a new `MultiplayerGameState` variant requires updating all three independently. | Extract a private `fn mp_game_state_is_active(state: &MultiplayerGameState) -> bool { matches!(state, Running \| Paused \| ...) }` and call it from all three. |
| G-03 | ArchitecturalDecay | `messages.rs:27–31` | Medium | S | `WaveSpawnedMessage` is registered in `plugin.rs:63`, written in `wave_systems.rs:221`, but has **zero** `MessageReader` subscribers anywhere in the codebase. The `wave_number` field carries `#[allow(dead_code)]` confirming it is never consumed. This is a dead message channel that adds unnecessary noise and a per-frame message buffer. | Either add a UI consumer (wave-number HUD indicator) to justify the channel, or remove the message type, its `add_message` registration, and the `MessageWriter` parameter from `tick_wave_timer`. |
| G-04 | ConsistencyRot | `shared_systems.rs:134–146` | Low | S | `cleanup_game` (line 125) and `cleanup_for_replay` (line 139) have identical bodies: both despawn all `OnGameplayScreen` entities. They differ only in their doc-comment and query variable name. Two identical fns registered to different schedule hooks will silently diverge on future changes. | Keep both public fns (they serve different schedule hooks), but have each delegate to a single private `fn despawn_all_gameplay_entities(commands, query)` helper. |
| G-05 | ConsistencyRot | `resources.rs:53–64` | Low | S | `KillStats::reset()` manually zeros all 10 fields, duplicating the logic already implied by `#[derive(Default)]` on the same struct. If a new field is added to the struct but not to `reset()`, the reset silently leaves stale data across replays. | Replace the body of `reset()` with `*self = Default::default();` so the two code paths are structurally equivalent. |
| G-06 | Performance | `shared_systems.rs:74–78` | Medium | M | `calculate_effectiveness` builds a `Vec<_>` snapshot of all unit data on **every gameplay frame** and then runs an O(n²) comparison loop. With ~200+ units at higher levels (100 defenders + 60+ infantry + archers + bosses) this is ~40 000 distance checks and one 200-element heap allocation per frame at 60 fps. No comment documents this as an accepted trade-off. | Consider: (a) throttling to every 4–8 frames via a frame-counter resource (effectiveness changes slowly), or (b) adding a spatial hash using the existing pathfinding grid to reduce comparisons to nearby-cell units only. At minimum, add a doc comment noting the O(n²) cost and acknowledged unit-cap ceiling. |
| G-07 | DependencyConfig | `shared_systems.rs:253` | Low | S | `BATTLE_AMBIENCE_MAX_DISTANCE = 10000.0` is a file-local constant whose value is duplicated by `MAX_SFX_DISTANCE = 10000.0` in `units/wizard/spells/audio.rs:12`. Two independent constants with the same value and the same physical meaning will diverge on tuning. | Define a single `pub const MAX_WORLD_SFX_DISTANCE: f32 = 10000.0;` in `constants/tuning.rs` and use it in both files. |
| G-08 | ArchitecturalDecay | `debug_ui.rs:144` | Low | S | `toggle_debug_ui_visible` is registered with `.add_systems(Update, toggle_debug_ui_visible)` with no `run_if` state guard. Per project conventions every `Update` system must have a `run_if` guard. It currently fires on all keyboard events in all app states (main menu, loading screen, etc.). | Add `.run_if(not(in_state(AppState::Loading)))` or a broader `run_if(in_state(AppState::InGame).or(in_state(AppState::MultiplayerGame)))` to match the F2-debug-UI convention documented in project memory. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|-------------------------|
| `shared_systems.rs` | 536 | No | Mixed concerns (lifecycle, gameplay, audio, shadows, achievement tracking). Propose split into `ambience.rs`, `shadows.rs`, `lifecycle.rs`, keeping `shared_systems.rs` for core gameplay cross-cuts (~180 LOC). |
| `wave_systems.rs` | 345 | Yes | Both functions (`tick_wave_timer` + `apply_wave_upgrades`) are tightly coupled by the `PendingWaveUpgrades` data flow; splitting would create two files that still import all the same unit-spawn helpers. The file contains no unrelated logic. |

---

### Looks Bad But Is Actually Fine

- **`plugin.rs` is 290 lines of pure `.add_systems`/`.configure_sets`/`.add_plugins`/`.init_resource` chains** — no system bodies, no helper fns. Per project convention a purely-registration `plugin.rs` is not a violation regardless of length. The inline comments explaining run-condition rationale are load-bearing.
- **`is_gameplay_running` returns `true` for the MP host in `Paused`/`SpellBook` states** — this correctly keeps the host simulation alive while co-op/versus overlays are open. The lengthy comment block at line 84 explains the intentional design.
- **`calculate_effectiveness` excludes `Boss` entities via `Without<Boss>`** — looks like a special case but is correct: bosses have their own team/targeting model and should not skew the melee-density effectiveness calculation.
- **`cleanup_game` is registered on both `OnExit(AppState::InGame)` and `OnExit(AppState::MultiplayerGame)`** — identical registration to two different hooks is intentional; the comment at `plugin.rs:113–118` explains that MP has different cleanup paths for the HUD/action bar entities.
- **`apply_wave_upgrades` is gated only by `resource_exists::<PendingWaveUpgrades>`** — the resource is only ever inserted by `tick_wave_timer` which is already guarded to SP+InGame, so the simpler gate is safe and correct.
- **`WaveSpawnedMessage` registered in top-level `GamePlugin`** — messages that cross module boundaries (or are expected to have multi-plugin consumers in future) belong at the orchestrator level even if currently written by only one system.

---

### Open Questions

1. **`WaveSpawnedMessage` (G-03)**: Was there a UI wave-counter overlay that was removed without cleaning up the message emit? Or is a subscriber planned?
2. **`calculate_effectiveness` throttle (G-06)**: Is there an established performance budget for the effectiveness system? The current 200-unit bound is unlikely to be a bottleneck, but higher-difficulty roguelite levels could push it.
3. **`BATTLE_AMBIENCE_MAX_DISTANCE` / `MAX_SFX_DISTANCE` (G-07)**: Are these intentionally the same value? If so, unifying them is a safe no-brainer refactor. If they could diverge for design reasons, they should stay separate but be documented as intentionally different.
