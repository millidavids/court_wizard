# spells-batch4

**Scope:** `src/game/units/wizard/spells/{entangle,polymorph,telekinesis,berserker_rage}/`

---

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
