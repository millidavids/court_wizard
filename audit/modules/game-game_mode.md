## game-game_mode

**Scope:** `src/game/game_mode/` — game-mode lifecycle, roguelite run state, toggle modifiers, and the wizard-cycle mechanic.

---

### Mental Model

The `game_mode` module owns two orthogonal concepts that coexist during a level:
- **Mode resources** — `GameMode` (Endless vs Roguelite), `RogueliteRunState`, `RogueliteModifiers`. Inserted when a run starts, removed on main-menu return.
- **Toggle modifiers** — `ActiveToggles` (a thin `Vec<ToggleModifier>` wrapper). Each toggle is handled either by a dedicated `OnEnter(InGame)` system (FortifiedHorde, GlassCannon, VeteranDefenders, WizardCycle, Attrition, SpellRotation) or by external systems in other modules (ManaDrought in wizard/runtime, BossParade/RisingTide in wave_systems, Urgent in run_conditions).

The most complex system is `tick_wizard_cycle`: it hot-swaps `config.wizard_type`, despawns old archetype UI, and spawns new archetype UI — essentially replicating the per-archetype setup that each archetype plugin normally handles at `OnEnter(InGame)`.

File layout is clean: `plugin.rs` is registration-only, `systems.rs` owns all system bodies, and `components/` is correctly split into `modifiers.rs`, `run_state.rs`, and `toggles.rs`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| GM-01 | ArchitecturalDecay | `systems.rs:340-374` | High | M | `tick_wizard_cycle` handles per-archetype UI spawning in a match arm, but is missing Meteorologist. The meteorologist plugin spawns weather overlays and a ground overlay via `spawn_weather_overlays` / `spawn_ground_overlay` on `OnEnter(InGameState::Running)`. Those systems don't re-fire when `config.wizard_type` changes mid-game. Cycling TO Meteorologist leaves weather visuals absent; cycling AWAY from Meteorologist leaves overlays orphaned. | Add `WizardType::Meteorologist` arm to call the meteorologist spawn helpers, and add cleanup of weather overlay entities when departing from Meteorologist. Consider extracting per-archetype setup/teardown behind a `fn swap_archetype_setup(old: WizardType, new: WizardType, ...)` helper in a `wizard_cycle_ui.rs` file. |
| GM-02 | ArchitecturalDecay | `systems.rs:293-398` | Medium | M | `tick_wizard_cycle` is 106 lines and encodes archetype-specific UI spawning inline. As archetypes are added, every author must remember to add a branch here AND keep it in sync with each archetype's own plugin. The comment `// BoringOleMage, Excremage, etc. have no special UI` (line 374) is the only guard against omissions. | Extract wizard-cycle UI management into a `wizard_cycle_ui.rs` sibling file. This keeps `tick_wizard_cycle` focused on the timer logic and makes the per-archetype dispatch table easy to audit. |
| GM-03 | Performance | `systems.rs:221-223` | Low | S | `animate_fortified_horde_glow` calls `shielded.is_empty()` and returns early (line 221), but the system is already gated with `.run_if(any_exist::<FortifiedHordeShield>())` in the plugin (line 38). The guard can never be true when the system fires — it is dead code. | Remove the `if shielded.is_empty() { return; }` guard. |
| GM-04 | TypeContract | `systems.rs:58-62` | Low | S | Unlocked wizard types are compared using `format!("{:?}", wt)` (line 60). This is project-wide convention, but ties the save-data format to Rust's `Debug` output. A rename of any `WizardType` variant silently breaks unlock detection with no compiler warning. | Add an `fn id(&self) -> &'static str` method on `WizardType` (mirroring `ToggleModifier::id()`) and use it in `wizard_crud.rs`, `save_structs.rs`, and `systems.rs:60` for all wizard-type string keys. |
| GM-05 | ErrorObservability | `systems.rs:52-54` | Low | S | When `load_unified_save()` fails in `init_toggle_resources`, the WizardCycle feature silently treats all wizard types as locked (empty `unlocked_types`), resulting in the toggle appearing to do nothing without any diagnostic. | Add `warn!("WizardCycle: failed to load save data; treating all wizard types as locked")` on the failure path before `unwrap_or_default()`. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Split Proposal |
|------|-----|--------|--------------------------|
| `systems.rs` | 419 | No | Split into: `lifecycle.rs` (cleanup_game_mode, init_roguelite_run, init_toggle_resources, update_attrition_survivors), `scaling.rs` (apply_endless_scaling, apply_roguelite_effectiveness, apply_defender_toggles, apply_fortified_horde, animate_fortified_horde_glow, cleanup_fortified_horde_glow), `wizard_cycle.rs` (tick_wizard_cycle, update_wizard_cycle_flash) |
| `components/toggles.rs` | 204 | Yes | Single concern: the `ToggleModifier` enum and all its match arms. Every line is cohesive. |

---

### Looks Bad But Is Actually Fine

- **`format!("{:?}", wt)` at line 60:** Looks fragile, but is the project-wide convention for wizard-type persistence (see `wizard_crud.rs:37`, `save_structs.rs:125`). Flagged as a low-severity contract risk (GM-04) but not a current bug.
- **`init_resource::<GunState>()` / `init_resource::<FlamethrowerSfx>()` inline in `tick_wizard_cycle` (lines 371-372) instead of calling `init_gun_state`:** `init_gun_state` does exactly these two `init_resource` calls and nothing else (`state.rs:21-24`). Equivalent.
- **`wizard_entity_query.single()` called twice (lines 333, 358):** Both are inside different non-overlapping `match` arms. No double-borrow or redundancy concern.
- **`update_attrition_survivors` queries without `Without<GhostEntity>`:** Ghost entities carry `Team` but not `Infantry`, `Archer`, or `KingsGuard` components. Queries exclude ghosts naturally.
- **`apply_endless_scaling` and `apply_roguelite_effectiveness` as `OnEnter(InGame)` without ghost gating:** Ghost entities don't carry `Effectiveness`, so these systems correctly skip them.
- **`SHIELD_AMOUNT` / `SHIELD_DURATION` as inline `const` inside `apply_fortified_horde` (lines 202-203):** Project convention says constants used by exactly one feature file should be inlined. Correct placement.
- **`WizardCycleTimer` uses a manual `f32` countdown instead of Bevy's `Timer`:** Consistent with `WizardCycleFlash.timer` and other timers in the codebase. Not a violation.

---

### Open Questions

1. When WizardCycle transitions to or from Meteorologist mid-level, should the weather system reset? The weather plugin's `reset_weather_state` runs at `OnEnter(InGameState::Running)`, not mid-level — leaving potentially stale weather state when cycling.
2. The `VeteranDefenders` toggle description says "half as many defenders." `apply_defender_toggles` only modifies stats on existing entities and does not reduce spawn count. Is the spawn-count halving handled in the loading/spawn queue, or is the toggle description misleading?
3. `AttritionState` is protected by `init_resource` (only inserts if absent) to preserve counts between roguelite levels. But `cleanup_game_mode` unconditionally removes it. If a roguelite run is abandoned mid-level (defeat → main menu → new roguelite run), is `cleanup_game_mode` guaranteed to run before the new `init_resource` call, resetting counts correctly?
