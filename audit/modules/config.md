## config

**Scope:** `src/config/` — configuration loading/saving, save-data CRUD, input bindings, achievement IDs.

---

### Mental model

The `config` module is the game's single I/O boundary for all persistent player state. It separates
concerns cleanly into three layers: (1) an in-memory Bevy `GameConfig` resource that is the runtime
source of truth; (2) a write-behind save cache (`SAVE_CACHE` static `Mutex`) that batches all
unified-save mutations and flushes to disk on a 2-second periodic timer; and (3) a thin filesystem
abstraction in `storage.rs` that does write-temp-then-rename for crash safety and automatic .bak
fallback. A separate `ConfigPlugin` in `plugin.rs` wires change-detection systems, a debounce
timer, and an `AppExit` flush. The module recently split a monolithic `save_data.rs` into four
focused subfiles (`save_structs`, `save_cache`, `wizard_crud`, `migration`), which is an
improvement. Remaining tech debt is primarily: stale WASM-era doc comments, a handful of
unconditional `Update` systems, a misleading debounce-duration comment, `VsyncMode::Adaptive`
silently mapping to the same Bevy mode as `On`, a double-load pattern in
`load_wizard_type_into_config`, and the file name `saves_v2.json` whose content is obfuscated TOML
not JSON. `wizard_crud.rs` at 909 LOC is oversized and should be split.

---

### Findings table

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| C-01 | DocDrift | `resources/config_file.rs:12` | Medium | S | `VsyncMode::Adaptive` doc says "falls back to off if frame rate drops" but `apply_vsync_config` (systems.rs:131) maps both `Adaptive` and `On` to `PresentMode::AutoVsync`. The two variants behave identically. | Either map `Adaptive` to `PresentMode::Immediate` or update the doc to "same as On on this backend". |
| C-02 | DocDrift | `plugin.rs:12` | Low | S | Plugin doc says "Persists changes to disk after 0.5s of inactivity" but `SaveDebounceTimer::default()` sets a 2.0-second timer (config_file.rs:129). | Fix the comment to "2.0s". |
| C-03 | DocDrift | `resources/game_config.rs:151` | Low | S | Field doc says "restored on game start after page reload" — stale WASM/localStorage era comment. | Update to "reset at startup; populated when a wizard is loaded". |
| C-04 | DocDrift | `save_data/save_structs.rs:14`, `messages.rs:3,13`, `progress.rs:149` | Low | S | Four doc comments still reference "localStorage" (WASM era). The game is native-only since March 2026. | Replace every "localStorage" occurrence with "disk" or "save file". |
| C-05 | ArchitecturalDecay | `save_data/wizard_crud.rs:170–208` | Medium | S | `validate_action_bar_slots` (line 171) calls `load_unified_save()` independently, but its only caller `load_wizard_type_into_config` already loaded the save via `get_wizard_by_type`. This is a redundant second cache-clone per wizard load. | Accept the already-loaded `unlocked_spells: &[String]` or `&UnlockedContent` as a parameter so no second load is needed. |
| C-06 | Performance | `save_data/wizard_crud.rs:646–672` | Medium | M | `increment_levels_completed`, `increment_games_played`, and `accumulate_kill_stats` each perform their own `load_unified_save` → `save_unified` cycle. At battle-end in `achievements/helpers.rs:64–72` all three are called sequentially, producing 3 separate cache-clone+store cycles for one logical event. | Introduce `record_battle_ended_counters(victory, defenders, attackers, undead)` that does one load + all mutations + one save. |
| C-07 | ConsistencyRot | `plugin.rs:35–47` | Medium | S | Five systems (`detect_window_resize`, `detect_window_move`, `detect_game_config_changes`, `detect_input_bindings_changes`, `mark_save_on_config_changed`) run unconditionally every `Update` frame with no `run_if` guard. The project convention requires every `Update` system to have one. | Gate the whole group with `.run_if(not(in_state(AppState::Splash)).and(not(in_state(AppState::Loading))))` or equivalent. The systems are internally cheap but the convention gap is a maintenance signal. |
| C-08 | DocDrift | `storage.rs:7` | Low | S | `UNIFIED_SAVE_FILENAME` is `"saves_v2.json"` but the content is obfuscated TOML serialised to base64 — not JSON. Misleads anyone inspecting the file on disk. | Rename to `"saves_v2.dat"` (with a one-time migration shim at startup to move the old file) or add a prominent comment at the constant explaining the naming is historical. |
| C-09 | ArchitecturalDecay | `resources/config_file.rs:118–132` | Low | S | `SaveDebounceTimer` and `SavedWindowedGeometry` are pure runtime resources with zero serialisation concern, yet they live in `config_file.rs` whose stated purpose is "only used for serialisation". | Move them to a new `resources/runtime.rs` or inline them into the files that own them. |
| C-10 | TypeContract | `save_data/save_structs.rs:27` | Medium | M | `SaveMetadata::version: u32` is written as `2` on every new save (save_cache.rs:35) but is never read to gate or drive any migration path. Migrations use file-existence checks instead. The field implies a versioning contract that does not exist. | Either wire version-gated migration logic (e.g. `if metadata.version < 3 { migrate_v2_to_v3(...) }`) and document the increment protocol, or remove the field. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `save_data/wizard_crud.rs` | 909 | No | Mixes unlock ops, CRUD, roguelite run ops, terrain ops, meta-counters, and insight/spell-talent ops. Propose: `unlocks.rs` (unlock_achievement/wizard_type/ingredient/unit + validate_action_bar), `wizard_ops.rs` (create/load/save wizard + terrain), `roguelite_ops.rs` (save/load/clear/toggle runs), `meta_counters.rs` (games_played/levels_completed/kill_stats), `insight.rs` (grant/spend/research/talent ops). |
| `systems.rs` | 571 | No | Mixes startup config loading, window geometry tracking, and config persistence. Propose: `startup.rs` (load_and_apply_config), `window_tracking.rs` (detect_resize/move, apply_display_mode, apply_deferred_mode_change), `persistence.rs` (persist_config, save_on_exit, build_config_from_game_config, debounce save). |
| `achievement_id.rs` | 522 | Yes | Single large match-on-enum data registry. Every method is a match over the same enum. Splitting would produce partial tables with no cohesion benefit. |
| `input_bindings.rs` | 667 | Yes | Single concern (KeyCode binding CRUD). Body is dominated by exhaustive match arms over context+action tuples. Splitting by archetype would just scatter the same pattern into 6 files. |
| `save_data/save_structs.rs` | 372 | No | Just above the 300-LOC limit. Currently clean data structs. Monitor: add saved-terrain fields directly to `WizardSave` and it would grow substantially. No split needed now. |
| `resources/game_config.rs` | 327 | No | Contains `GameConfig`, serde helpers, and two small enums. Could extract `ColorblindType`/`ControllerGlyphStyle` + serde helpers to `accessibility.rs` but not urgent since the file is cohesive. |

---

### Looks bad but is actually fine

- **`detect_game_config_changes` runs every frame with no guard**: looks expensive but only calls `game_config.is_changed()` which is a bitflag read — near-zero overhead even at 120fps.
- **`GameConfig` implements both `Serialize`/`Deserialize` and `Resource`**: deliberate — it doubles as the `[game]` section of the TOML config file. The `#[serde(skip)]` fields correctly exclude in-memory-only terrain state.
- **`from_string` falls back to `KeyCode::Escape`**: safe because `is_bindable_key` explicitly excludes `Escape` from rebinding. A corrupted binding degrades to unbound rather than panic.
- **Static `Mutex<Option<UnifiedSaveFile>>` for the save cache**: intentional — the cache must outlive Bevy world resets (main-menu returns). A process-lifetime static is the correct tool.
- **`save_unified` writes only to cache, not disk**: the write-behind split is intentional and well-documented. Every mutation sets `SAVE_DIRTY`; only the periodic flush or exit handler writes the file.
- **`compute_save_signature` uses `unwrap_or_default()`**: `toml::to_string` failure on an in-memory struct is effectively impossible. If it somehow fails, the signature becomes empty string, which will trigger a warn on next load (non-blocking) rather than panic. This is the right failure mode for a save-integrity hint.
- **Multiple near-identical `unlock_*` functions** (lines 22–80 of wizard_crud.rs): each modifies a different sub-collection with different containment checks. Genericising over `&mut Vec<String>` + a formatter closure would work but adds indirection for minimal gain.
- **`VsyncMode::Adaptive` variant mapping to `AutoVsync`**: `PresentMode::Adaptive` was removed from Bevy 0.18. `AutoVsync` is the correct replacement. The variant is kept for save-file forwards-compatibility.

---

### Open questions

1. **`saves_v2.json` naming**: is the `.json` extension purely historical, or is there a planned migration to actual JSON? If JSON is the goal, the obfuscation layer would need to be reconsidered since base64-of-XOR-of-TOML is not valid JSON.
2. **`metadata.version` increment protocol**: when is version `3` written? Without a gated migration path the field is ornamental.
3. **Debounce reset vs tick**: `mark_save_on_config_changed` resets the timer every frame that `ConfigChanged` fires. If the player holds a volume slider (continuous `ConfigChanged` fire), the timer never finishes — saves are deferred indefinitely. Is this intended, or should the timer tick unconditionally and only restart when `pending` transitions `false → true`?
4. **`GameConfig` terrain fields trigger TOML config saves**: mutating `saved_walls` (a `#[serde(skip)]` field) marks `GameConfig` as changed, which fires `detect_game_config_changes`, which debounces a TOML config write. That TOML write does not include the walls, so the write is a no-op from a data perspective but still incurs the serialize+write cycle. Worth confirming this is acceptable or whether terrain mutations should bypass the debounce path.
