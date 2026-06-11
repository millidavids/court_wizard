## spells-batch4

**Scope:** `src/game/units/wizard/spells/{entangle,polymorph,telekinesis,berserker_rage}/`

---

### Mental model

Four thematically distinct spells, each using a similar feature-sliced layout (constants, components, plugin, systems subdir). **Entangle** has the most complex structure: a nested `casting/` subdir with three files plus `vines.rs` and a forwarding `systems.rs` proxy. **Polymorph** is well-layered across `core` / `livestock` / `behaviors` / `shared` / `casting`. **Telekinesis** is SP-only (explicitly early-returns in MP), uses a `TelekinesisConfig` struct to avoid re-reading talent state, and handles VFX/behavior all in one `vfx_systems.rs`. **BerserkerRage** is a buff-AoE with five distinct talent components; its `undying_fury_trigger` is intentionally registered in the game-root plugin for PostCombatSet ordering rather than in its own plugin.

All four spells make correct use of the `spells/utils.rs` shared casting helpers. Multiplayer ghost-gating is generally correct: gameplay systems are behind `is_gameplay_running` (host-only), visual systems behind `is_spell_effects_active` (both peers). One medium gap exists in the entangle zone overgrowth pathway.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `entangle/systems.rs:1-4` | Medium | S | `systems.rs` is a pure forwarding proxy that re-exports from both `casting::*` and `vines::*`. This adds an indirection layer — the plugin reads `systems::tick_entangle_ground_effect` but that function physically lives in `casting/ground_effect.rs`. The file's own comment admits it is a "re-export hub (Phase 14)", a temporary migration artifact never cleaned up. | Remove `systems.rs` and have `plugin.rs` import directly from `super::casting::*` and `super::vines::*`, mirroring the pattern used by telekinesis and berserker. |
| F2 | ArchitecturalDecay | `entangle/vines.rs:1` | Low | S | `vines.rs` contains both the pure VFX helper `spawn_vine_toruses` and the gameplay-affecting `apply_entangle` (which roots units, spawns ground effect entity, sends pathfinding messages). The file docstring ("vine effects") understates its scope. | Rename to `entangle_effect.rs` or extract `apply_entangle` into a separate `apply.rs` alongside `root_effects.rs`. |
| F3 | Performance | `entangle/casting/root_effects.rs:135-154` | Medium | S | `nourishing_roots_mana_regen` counts rooted enemies by iterating all `EntangleRooted` entities and filtering on `talent_params.nourishing_roots` each frame. All `EntangleRooted` entities store identical `talent_params` (they come from a single cast), so the filter will be all-or-nothing. The system fires whenever `any_exist::<EntangleRooted>()` — even if the active talent is not Nourishing Roots, the query still runs. | Gate with a `has_entangle_talent(1, 2)` run condition (matching the pattern in telekinesis `run_conditions.rs`) so the system only runs when that talent tier/choice is selected. |
| F4 | ConsistencyRot | `entangle/casting/cast_input.rs:30` / `polymorph/systems/casting.rs:25` / `berserker_rage/systems/buff_application.rs:11` | Low | S | All three spells define a function named `compute_talent_params` with the same signature shape `(Option<&ActiveTalents>) -> TalentParams`. There is no shared abstraction but the naming convention is consistent — flag for awareness in case a future trait or macro is desired. Telekinesis uses `compute_telekinesis_config` which returns a named struct with `pub(super)` fields, a slightly better approach. | No immediate change needed, but consider a `ComputeTalentParams` trait in `spells/utils.rs` for future spells. Low priority. |
| F5 | ErrorObservability | `telekinesis/systems/drop_ops.rs:43` | Low | S | `nearest.as_ref().expect("checked")` is technically safe (short-circuit guards it), but violates the project convention that `.expect()` should be reserved for invariants expressed with a descriptive message, not logic proofs. | Replace with `nearest.as_ref().is_some_and(\|n\| distance < n.3)` to make the short-circuit intent explicit without the expect. |
| F6 | Performance | `telekinesis/systems/vfx_systems.rs:84,89,111` | Low | S | Three numeric literals are hardcoded in spawn functions: flash Y position `2.0`, flash scale `20.0`, and shockwave Y `1.0`. These tuning values live next to named constants for all other tunable parameters in `constants.rs`. | Extract `HARVEST_FLASH_Y`, `HARVEST_FLASH_SCALE`, and `SHOCKWAVE_Y` constants into `telekinesis/constants.rs`. |
| F7 | ArchitecturalDecay | `berserker_rage/systems/casting.rs:215` | Low | S | `berserker_rage_casting_logic` receives `_clamped_cursor: Option<Vec3>` as a parameter but never uses it (leading underscore suppresses the warning). It was presumably added for potential future use or was previously used in the Resting transition. | Remove the dead parameter. The caller already has `clamped_cursor` in scope and can pass it at the call site if needed later. |
| F8 | ConsistencyRot | `berserker_rage/systems/mod.rs:1,5-8` | Low | S | `systems/mod.rs` uses `pub(crate) mod` for submodules and `pub use` for re-exports, making several items reachable from outside the module. Telekinesis uses tighter `pub(super)` for the same pattern. Only `undying_fury_trigger` legitimately needs wider visibility (accessed from `game/plugin.rs`). | Restrict most `pub use` items in `berserker_rage/systems/mod.rs` to `pub(super)` except `undying_fury_trigger` which needs `pub`. |
| F9 | TypeContract | `polymorph/components.rs:62-71` | Low | S | `DireSheep` has a `new()` constructor but does not implement `Default`, while the project convention for small marker structs is to use `Default` or `new()` consistently. The `new()` only sets `attack_timer` to a constant — the same as what `Default` would return. | Implement `Default` for `DireSheep` (trivially via `attack_timer: DIRE_SHEEP_ATTACK_INTERVAL`) and use `DireSheep::default()` at the call site. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `entangle/casting/cast_input.rs` | 262 | true | Single system plus one helper — long but cohesive; all lines serve the casting pipeline. |
| `telekinesis/systems/casting.rs` | 269 | true | Single casting system plus its private inner helper (`telekinesis_casting_logic`) — cohesive, no split needed. |
| `telekinesis/systems/vfx_systems.rs` | 251 | true | All VFX/visual helpers for telekinesis — five tightly coupled functions, none large enough to split independently. |
| `polymorph/systems/core.rs` | 233 | true | Two functions: `apply_polymorph_to_target` + `polymorph_casting_logic`. Genuinely cohesive (core polymorph application logic). |
| `polymorph/systems/livestock.rs` | 235 | true | Two functions: `tick_polymorphed_units` + `check_explosive_sheep_deaths` — sequential lifecycle, cohesive. |
| `berserker_rage/systems/talent_effects.rs` | 286 | true | Six distinct but all berserker-talent-effect functions — at 286 lines it is near the 300-line limit but each function is small and focused; the grouping is intentional and coherent. |
| `berserker_rage/systems/casting.rs` | 245 | true | Single casting system + private core logic helper — cohesive, same pattern as other spells. |

---

### Looks bad but is actually fine

- **`contagious_rage_spread` has no `Without<GhostEntity>`** — safe because the system is gated `is_gameplay_running`, which returns `false` for the MP guest. GhostEntity units only exist on the guest peer.
- **`handle_pig_movement` / `tick_dire_sheep` in `behaviors.rs` have no `Without<GhostEntity>`** — same reasoning: both gated `is_gameplay_running`, host-only.
- **`nourishing_roots_mana_regen` queries all `EntangleRooted` without ghost guard** — the mana target is locked to `With<LocalWizard>` so there is no cross-peer contamination; the count inflation concern is moot because roots are applied by the casting wizard's system.
- **`unwrap_or_default()` in `vines.rs:133`** — this is `unwrap_or_default()` on a material lookup, returning an empty `StandardMaterial` as a safe fallback. Not the same as `.unwrap()`.
- **`undying_fury_trigger` registered in `game/plugin.rs` not `berserker_rage/plugin.rs`** — intentional for PostCombatSet ordering; the berserker plugin cannot safely express the `before(convert_dead_to_corpses)` constraint from inside the spell module.
- **`KEEN_SENSES_DROP_CHANCE_MULT` and `TRANSMUTATION_POTENCY_PER_STACK` exported from telekinesis constants to drops and cauldron** — intentional cross-cutting configs owned by the telekinesis feature, legitimately used by other modules that implement the talent effect.
- **`compute_talent_params` pattern repeated in entangle, polymorph, berserker** — each operates on a different `TalentParams` type; no generic abstraction is possible without a trait. This is a naming convention, not duplicated logic.
- **Polymorph casting gated `is_spell_effects_active` not `is_gameplay_running`** — intentional: the guest's local wizard can cast polymorph in coop. The target query excludes `PolymorphedModifier` and `Corpse` but not `GhostEntity`, however the guest's real units (not ghosts) are the intended targets in a coop session.

---

### Open questions

1. **F3 nourishing roots performance**: Is iterating `EntangleRooted` every frame (when rooted units exist but Nourishing Roots talent is not selected) a visible cost at high enemy counts, or profiled as negligible?
2. **entangle/systems.rs longevity**: Was the re-export proxy left intentionally post-Phase 14 split, or is it recognized as cleanup debt?
3. **Polymorph guest casting**: Is there a design intent to allow the guest wizard to polymorph enemy units in coop, and if so does the current lack of `Without<GhostEntity>` on the `targets_query` need guarding?

## Mental model

These four spells form the "control + support + utility" tier of the wizard arsenal. **Entangle** lays a persistent ground zone that roots units and drives multiple talent-gated secondary effects (overgrowth expansion, thorny DPS, mana regen, Stranglehold burst). **Polymorph** converts enemies into livestock with highly branching per-form behaviors (pig, dire sheep, explosive, contagious, permanent). **Telekinesis** is a SP-only ingredient-pull utility that gains passive pull radius, storm pickup, and knockback ring from talents. **Berserker Rage** buffs nearby units with a damage/vulnerability aura and layers up to three tier-2 and tier-2 per-unit component markers that fire independently (Frenzy, Undying Fury, Contagious Rage, Final Stand).

All four follow the established spell skeleton (build_wizard_input → indicator → CastingState machine → consume mana → apply effect → school flare VFX). Ghost-entity / MP gating is present where health mutations occur, and every Update system carries at least one `run_if` guard. The main technical debt is **oversized `systems.rs` files** (three of the four exceed 500 LOC) with multiple distinct concerns that haven't yet been split following the project's granular-file convention.

---

## Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `entangle/casting.rs:1` (531 LOC) | High | M | `casting.rs` was meant to hold casting input only but accumulated ground-effect tick (`tick_entangle_ground_effect`), overgrowth re-root (`overgrowth_root_new_units`), despawn cleanup (`cleanup_entangle_ground_effect`), Thorny Vines DPS (`thorny_vines_tick`), root-expire effects (`handle_entangle_root_expire`), and mana regen (`nourishing_roots_mana_regen`) — at least three distinct concerns beyond casting. The project convention says files exceeding ~300 LOC must be split unless genuinely cohesive. | Split into `casting.rs` (input + mana + indicator, ~170 LOC), `ground_effect.rs` (tick + overgrowth + despawn, ~120 LOC), and `root_effects.rs` (thorny, expire, regen, apply helper, ~200 LOC). Update the `systems.rs` re-export hub accordingly. |
| F2 | ArchitecturalDecay | `polymorph/systems.rs:1` (757 LOC) | High | M | Mixes casting logic, per-livestock-type tick systems (pig flee, dire sheep attack), and sheep death handling into a single file. Concerns are distinct: input/cast, polymorphed-unit lifecycle, individual livestock behaviors. | Split into `casting.rs` (cast input + `apply_polymorph_to_target` + visual helper), `livestock.rs` (tick, explosive deaths), and `behaviors.rs` (pig movement, dire sheep). |
| F3 | ArchitecturalDecay | `telekinesis/systems.rs:1` (688 LOC) | High | M | Casts, storm pickup, Harvest damage+flash, Psychic Shockwave ring, Magnetic Pull drift, Transmutation stack tracking, and indicator animation all live in one file. | Split into `casting.rs` (indicator, casting state, normal/storm pickup), `talents.rs` (magnetic pull, transmutation tracking), and `vfx.rs` (harvest flash, shockwave ring). |
| F4 | ArchitecturalDecay | `berserker_rage/systems.rs:1` (656 LOC) | Medium | M | Casting, buff application, and five independent talent tick systems (Undying Fury, Frenzy, Contagious Rage, Final Stand, cleanup) coexist. The casting + buff logic is ~300 LOC and talent effects are ~350 LOC. | Split into `casting.rs` (input, cast state, `apply_berserker_rage_buff`) and `talent_effects.rs` (all tier-2/3 tick and event-response systems). |
| F5 | ConsistencyRot | `polymorph/systems.rs:362,404`, `entangle/casting.rs:351`, `entangle/vines.rs:35` vs `berserker_rage/systems.rs:333,530` | Medium | S | Entangle and Polymorph use `Vec3::distance()` (3D Euclidean) for unit hitbox checks, while Berserker Rage and the shared util correctly use `xz_distance()` (XZ-plane only). Units live on the XZ plane; an enemy at a different Y (e.g., mid-jump animation) could fail the distance check when it should hit, or vice versa. Telekinesis uses raw `dx*dx + dz*dz` (correct XZ but inconsistent style). | Replace all `transform.translation.distance(pos)` in area-of-effect hit-detection with `spells::utils::xz_distance()`. Use the existing helper consistently. |
| F6 | ArchitecturalDecay | `telekinesis/plugin.rs:51–68` | Low | S | `plugin.rs` defines three non-registration functions: `has_telekinesis_talent` (a run-condition factory), `init_transmutation_stacks`, and `cleanup_transmutation_stacks`. The project rule is plugin.rs = Bevy registration only. | Move `has_telekinesis_talent` and the two lifecycle fns into `systems.rs` (or into a new `telekinesis/resources.rs` alongside `TransmutationStacks`). |
| F7 | ConsistencyRot | `entangle/vines.rs:130–134` | Low | S | Material clone for vine rings uses `materials.get(&handle).cloned().unwrap_or_default()`. If the asset handle is invalid (e.g., asset not yet loaded), `unwrap_or_default()` silently produces a default white material instead of logging a warning. The rest of the codebase avoids `.unwrap_or_default()` on asset lookups by either `if let Some` or `.expect("asset must be loaded")`. | Use `if let Some(base_mat) = materials.get(&assets.entangle_vine).cloned()` and log a `warn!` in the else branch, consistent with other material cloning patterns. |
| F8 | ConsistencyRot | `telekinesis/systems.rs:535` | Low | S | `nearest.as_ref().expect("checked").3` in `find_nearest_drop` — the `.expect()` documents the invariant ("nearest is Some because we just set it"), but the message "checked" is not self-explanatory. The project convention says `.expect()` requires a descriptive invariant message. | Change to `.expect("nearest is Some: condition above guarantees it was set")` or refactor to `if let Some(ref n) = nearest && distance < n.3`. |
| F9 | ConsistencyRot | `telekinesis/systems.rs:546` | Low | S | `clone_material_if_needed` is a private helper that duplicates the same deferred-clone pattern found in `lightning_bolt.rs` (lines 128–135). Two sites is not three, but this pattern is spreading; extracting it to `spells/utils.rs` would prevent a third copy. | Move `clone_material_if_needed` to `src/game/units/wizard/spells/utils.rs` as a `pub(crate)` helper used by any spell VFX that needs per-entity material cloning. |

---

## Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| `polymorph/systems.rs` | 757 | false | Split into `casting.rs`, `livestock.rs` (tick + death), `behaviors.rs` (pig, dire sheep) |
| `telekinesis/systems.rs` | 688 | false | Split into `casting.rs`, `talents.rs` (magnetic pull, transmutation), `vfx.rs` (harvest flash, shockwave) |
| `berserker_rage/systems.rs` | 656 | false | Split into `casting.rs`, `talent_effects.rs` (undying fury, frenzy, contagious rage, final stand, cleanup) |
| `entangle/casting.rs` | 531 | false | Split into `casting.rs`, `ground_effect.rs`, `root_effects.rs` |

---

## Looks bad but is actually fine

- **`Without<GhostEntity>` in `tick_polymorphed_units` / `bounce_sheep_units` while also gated by `is_gameplay_running`**: `is_gameplay_running` already restricts MP execution to the host, and `GhostEntity` components are only spawned on the guest. The guard is redundant but not wrong — it is explicitly defensive, with comments explaining the intent.
- **Entangle `systems.rs` re-export hub**: `systems.rs` containing only `pub use super::casting::*; pub use super::vines::*;` looks like a `mod.rs` violation, but this exact pattern (systems.rs as a re-export gateway for split sub-files) is established project-wide in chain_lightning, grease, dispel, finger_of_death, etc.
- **`polymorph_casting_logic` double-query for Mass Polymorph (iter then get)**: The `target_entities: Vec<Entity>` collect-then-get pattern avoids holding multiple mutable borrows on the same query simultaneously, which Bevy's borrow checker does not allow in a single `iter_mut()`. This is the correct Bevy idiom for applying mutations inside a query loop that also reads from the same query.
- **Telekinesis early return on `mp_session.is_some()`**: The spell silently becomes a no-op in multiplayer rather than showing an error. This is intentional — IngredientDrop entities don't exist in MP, and the comment documents the reason.
- **Berserker Rage `undying_fury_trigger` is registered in `src/game/plugin.rs` rather than `berserker_rage/plugin.rs`**: This is because it must run in a specific post-combat ordering (`PostCombatSet`) that is defined at the game plugin level. Moving it into the spell plugin could cause ordering issues.
- **`partial_cmp(...).unwrap_or(Ordering::Equal)` in polymorph and berserker_rage**: These are comparisons of `f32` distances that could theoretically produce `NaN`. The `unwrap_or(Ordering::Equal)` fallback on `partial_cmp` is a standard Rust idiom for total-order sorting of floats and is not a production error risk.

---

## Open questions

1. **Entangle `apply_entangle_to_unit` visibility**: It is `pub(super)` in `casting.rs` and called from `vines.rs`. After the proposed split into `ground_effect.rs` + `root_effects.rs`, does it belong in the new `root_effects.rs` or stay in `casting.rs`? The caller in `vines.rs` is outside the casting concern — consider promoting to `pub(crate)` or moving it to a shared position.
2. **Telekinesis in multiplayer**: The spell is completely disabled in MP with a silent early return. Is there a design intention to eventually support it (e.g., via drop-sync), or should this be surfaced to the player as "spell unavailable in co-op"?
3. **Berserker Rage Bloodlust**: The `Bloodlust` component (lifesteal on damage) is inserted on cast, but there is no visible system in this module that reads it and heals the unit on damage. Where does bloodlust lifesteal actually run? If it is in a cross-cutting combat system, that dependency should be documented in `berserker_rage/components.rs`.
