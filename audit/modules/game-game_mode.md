## game-game_mode

**Scope:** `src/game/game_mode/` — game mode lifecycle, toggle/slider modifier system, roguelite run tracking.

---

### Mental model

The module owns two game modes (Endless, Roguelite) and an 11-toggle modifier layer that sits on top of Roguelite runs. It manages resource lifecycle (insert on game-start, remove on main-menu return), applies one-shot stat patches to units at `OnEnter(InGame)`, and drives two ongoing per-frame features: the Fortified Horde pulsing shield glow and the WizardCycle archetype rotation timer. `components.rs` is the central type hub for the whole feature — it holds ECS Resources, Components, pure-data structs, utility functions, and constants all in one file, making it the module's main architectural tension. `systems.rs` is a clean collection of system bodies with no logic leakage into `plugin.rs`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| GM-01 | ArchitecturalDecay | `components.rs:1–444` | Medium | M | `components.rs` is 444 LOC mixing ECS Resources, ECS Components, plain data structs (`RunAggregateStats`, `LevelRunStats`), utility functions (`format_time`, `scorched_earth_mult`, `is_roguelite_mode`, `is_endless_mode`), and module-wide constants. Project convention reserves the name `components.rs` for cross-cutting ECS types and says utility fns belong in feature files. | Split into: `resources.rs` (all `#[derive(Resource)]` types), `modifiers.rs` (`ToggleModifier` + `ActiveToggles` + `RogueliteModifiers`), `run_stats.rs` (`LevelRunStats`, `RogueliteRunState`, `RunAggregateStats`, `format_time`), and keep `components.rs` for the three actual `#[derive(Component)]` types (`FortifiedHordeShield`, `WizardCycleFlash`, `ArchetypeUI`). |
| GM-02 | Performance | `systems.rs:52–54` | Medium | S | `init_toggle_resources` calls `crate::config::save_data::load_unified_save()` — a synchronous filesystem read — inside a Bevy system (on `OnEnter(InGame)`). The unlocked wizard-type list is already reconstructable from in-memory resources rather than from disk. | Pass the already-in-memory `GameConfig` (or a dedicated `UnlockedWizardTypes` resource populated at startup) instead of re-reading the save file. This is an `OnEnter` system so it only fires once, but the pattern is fragile: it will silently return stale data if called on a platform or build that delays disk I/O. |
| GM-03 | TypeContract | `systems.rs:58–63` | Medium | S | `WizardType` identity is compared against strings via `format!("{:?}", wt)` — i.e., the Debug representation is the canonical persistence key. If a variant is renamed, all existing save data silently loses that unlock. The same `format!("{:?}", wizard_type)` pattern exists in `save_data.rs:186` for writing, so the serialization is at least symmetric, but it has no compile-time guard. | Add a `WizardType::to_save_id(&self) -> &'static str` method (parallel to `ToggleModifier::id()` which already does this correctly) and use it both when persisting and when deserializing. The `ToggleModifier` enum in the same file already demonstrates the right pattern. |
| GM-04 | DocDrift | `components.rs:436–441` | Low | S | `SCORCHED_EARTH_DURATION_MULT` at line 436 has the comment `"Last level of a roguelite run (tier 4 boss, level 25)."` — which is the doc for `ROGUELITE_MAX_LEVEL` at line 441, not for the duration multiplier. The SCORCHED_EARTH constant's actual meaning (duration multiplier) is buried in the second sentence of the same comment. | Fix the doc on `SCORCHED_EARTH_DURATION_MULT` to read `"Duration multiplier applied by the Scorched Earth toggle."` |
| GM-05 | ArchitecturalDecay | `systems.rs:341–375` | Medium | M | `tick_wizard_cycle` contains a long `match new_type { ... }` that directly calls into concrete archetype UI spawn functions (`spawn_rune_display`, `spawn_roulette_display`, `spawn_arcanorouter_display`, `spawn_enter_fray_button`) and initializes archetype-specific resources (`GunState`, `FlamethrowerSfx`). This makes `game_mode` an implicit dependency on every archetype's internal modules. | Add a `fn spawn_archetype_ui(commands, &AssetServer, ...)` associated function (or trait) to each archetype so `tick_wizard_cycle` calls a single dispatch point. Alternatively, drive this through a `WizardTypeChangedMessage` that the archetype plugins listen to, keeping game_mode decoupled from archetype internals. |
| GM-06 | ArchitecturalDecay | `plugin.rs:47–49` | Low | S | `refresh_spell_visuals_for_wizard` (defined in `units/wizard/spells/visual_assets.rs`) is registered in three different plugins: `spells/plugin.rs` (OnEnter Loading), `multiplayer/plugin.rs`, and `game_mode/plugin.rs` (run_if config changed). The `game_mode` registration is the right place for the wizard-cycle use case, but the comment at the call site gives no hint why this particular plugin owns the "config changed" trigger. | Add an inline comment explaining that this run-on-`config.is_changed()` registration exists to recolor Excremage materials when `tick_wizard_cycle` changes `config.wizard_type` mid-game. |
| GM-07 | ArchitecturalDecay | `systems.rs:216–233` | Low | S | `animate_fortified_horde_glow` contains an early-out `if shielded.is_empty() { return; }` guard (line 221) that is already redundant because the system is registered with `.run_if(any_exist::<FortifiedHordeShield>())` in the plugin. The inner check cannot fire unless Bevy somehow scheduled the system despite the run condition failing. | Remove the `if shielded.is_empty()` guard inside the system body; rely solely on the `run_if` condition. |
| GM-08 | ConsistencyRot | `components.rs:128,223,292,325,331,347` | Low | S | Six methods in `ActiveToggles` and `ToggleModifier` carry `#[allow(dead_code)]`. In a library these would need the attribute, but in a binary crate the compiler already warns only when items are truly unreachable. If these methods are reachable from UI or save-data code, the attribute is misleading; if they are genuinely unused they should be removed. | Audit each `#[allow(dead_code)]` site. Remove the attribute and let `cargo check` confirm reachability; delete any truly unused methods. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `components.rs` | 444 | No | Contains mixed concerns: ECS Resources, ECS Components, data structs, utility fns, constants. Propose splitting into `resources.rs`, `modifiers.rs`, `run_stats.rs`, `components.rs` (ECS components only). |
| `systems.rs` | 419 | Yes | All 10 system bodies are genuinely cohesive (they all service the game-mode lifecycle and its toggles). No logic belongs elsewhere. Borderline but exempt. |

---

### Looks bad but is actually fine

- **`Option<Res<GameMode>>` / `Option<Res<RogueliteModifiers>>` throughout systems** — looks like unnecessary Optional wrapping, but these resources are deliberately absent in Endless mode or when no run is active. Using `Option<Res<T>>` is the correct Bevy pattern for optional resources.
- **`tick_wizard_cycle` has `#[allow(clippy::too_many_arguments)]`** — 9 injected parameters for a Bevy system that drives archetype-switching UI is expected per project conventions.
- **`cleanup_game_mode` runs `OnEnter(MainMenu)`** — looks like a state-transition risk if main menu is ever entered without a prior run, but `commands.remove_resource` is a no-op when the resource doesn't exist, so this is safe.
- **`apply_endless_scaling` / `apply_roguelite_effectiveness` mutating unit `Effectiveness.base` at `OnEnter(InGame)` without compensating on exit** — looks like a data leak, but these systems only run once per battle (OnEnter) and the entire unit set is respawned between battles. No rollback needed.
- **Three registrations of `refresh_spell_visuals_for_wizard`** — looks like duplication but each registration has a distinct trigger and schedule context. The game_mode one is the only one handling mid-game config changes.
- **`WizardCycleTimer` uses a raw `f32` timer instead of Bevy's `Timer`** — project-consistent pattern, not a bug.
- **`ActiveToggles.toggles` is a `Vec` rather than `HashSet`** — correct given that the list is small (≤11 items), serialized to JSON, and order-stable. The `contains` O(n) scan is negligible.

---

### Open questions

1. **WizardCycle + Psychopath**: `Psychopath` is included in `WizardType::all()` and is NOT filtered out of the `unlocked_types` list built in `init_toggle_resources`. The match in `tick_wizard_cycle` falls through to `_ => {}` for Psychopath, which is probably fine (it has no special UI), but Psychopath is disabled in multiplayer. Could WizardCycle inadvertently re-enable it in a single-player roguelite run with Psychopath unlocked? Confirm whether Psychopath should be excluded from the WizardCycle rotation.
2. **Attrition + King's Guard query comment** (line 283): the comment says "Infantry query includes KingsGuard (they also have Infantry component)". Is that invariant actually guaranteed, or could a future refactor silently break the subtraction logic in `update_attrition_survivors`?
3. **`RunAggregateStats` visibility**: it is `pub(crate)` but carries no serde derives, relying on being computed at read time from `Vec<LevelRunStats>`. Confirm this is intentional and no persistence of aggregate stats is planned.
