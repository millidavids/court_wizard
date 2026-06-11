## units-root

**Scope:** `src/game/units/*.rs` (root-level files only — no subdirectory recursion)

---

### Mental model

The `units/` root layer is the **shared infrastructure layer** for all units in the game. It owns:

- **Cross-cutting components** (`components.rs`, `status_effects.rs`, `animation.rs`): the canonical component types (`Health`, `Team`, `Effectiveness`, `FireDoT`, `Shocked`, `FrostAccumulation`, timed-modifier status effects, sprite animation state). These are the "vocabulary" that both defenders and attackers share.
- **Shared helper functions** (`systems.rs`): `calculate_weighted_movement`, `update_melee_unit_targeting`, `is_cc_immobilized` are pure free functions called by ~16 sites across infantry, archer, boss modules. The same file also holds the implementations of several global per-frame *systems* (DoT ticking, persistent-effect visual tinting, animation drivers, knockback/airborne physics).
- **Movement integration** (`movement.rs`): the terminal `apply_unit_movement` system that integrates acceleration into velocity and velocity into position.
- **Plugin orchestration** (`plugin.rs`): purely registration — clean.
- **Misc utilities**: `damage.rs` (DamageType enum + wire-format helpers), `constants.rs` (all shared visual and gameplay tuning values), `spawning.rs` (cell jitter), `spell_stats.rs` (one-frame score-screen tallies), `sets.rs` (ECS system-set markers), `hit_flash.rs` (VFX), `ranged_bolt.rs` (shared bolt for dispeller/teleporter), `unit_type.rs` (enum + compendium specs).

The layer is reasonably well-factored but `systems.rs` (1 878 LOC) is the one clear over-size problem: it mixes DoT logic, VFX spawning, sprite animation drivers, and physics helpers into one file that is hard to navigate at-a-glance.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| U-01 | ArchitecturalDecay | `systems.rs:1–1879` | High | M | **God file (1 878 LOC) mixing four distinct concerns.** The file contains: (1) shared movement/targeting *helper functions* (1–310), (2) DoT simulation *systems* (311–920), (3) a large material-tinting *visual system* (939–1202), (4) sprite-animation *systems* (1204–1845), and (5) two small physics systems (1657–1878). These concerns are entirely independent and the file is too large to navigate. | Split into `dots.rs` (FireDoT/Electric/Frost/Poison/Sickened/Smelly tick systems + process_pending_damage_effects), `vfx_tinting.rs` (update_persistent_effect_visuals + helpers), `animation_systems.rs` (walking/combat/dying/rising animation drivers), and keep the movement/physics helpers in the existing `systems.rs` or rename to `movement_helpers.rs`. |
| U-02 | ArchitecturalDecay | `systems.rs:509,586` | Low | S | **Local `const` values for fire↔frost cross-effect rates** (`FIRE_MELTS_FROST_RATE = 0.3`, `FROST_QUENCHES_FIRE_RATE = 5.0`) are declared inside function bodies. These are tuning constants that belong in `constants.rs` alongside the other frost/fire constants. | Move to `constants.rs` with the rest of the elemental interaction constants. |
| U-03 | ConsistencyRot | `systems.rs:1179,1184` | Medium | S | **Hardcoded inline color literals** for Fear (`LinearRgba::new(0.5, 0.1, 0.6, 1.0)`) and Petrified (`LinearRgba::new(0.4, 0.4, 0.4, 1.0)`) in `update_persistent_effect_visuals`. Every other effect color is a named constant in `constants.rs`; these two are the lone exceptions. | Add `FEAR_EFFECT_COLOR` and `PETRIFIED_EFFECT_COLOR` (and corresponding intensity constants) to `constants.rs` and reference them from `update_persistent_effect_visuals`. |
| U-04 | Performance | `plugin.rs:80–85` | Low | S | **Four `update_timed_modifier` instances lack `any_with_component` guards** (`TemporaryHitPoints`, `SlowMovementModifier`, `RootedModifier`, `HasteModifier`). They run every frame during gameplay even when no unit has these components. `Stunned` on the same block already has the guard (line 86). | Add `.run_if(any_with_component::<X>)` to the four ungated calls, matching the pattern used for `Stunned`, `FrostAccumulation`, etc. |
| U-05 | Performance | `plugin.rs:173–181` | Medium | S | **`process_pending_damage_effects`, `update_fire_dot`, `update_electric_charge`, `update_electric_arc_visuals`, and `update_persistent_effect_visuals` have no `any_with_component` fast-exit guard.** They are covered only by the `is_spell_effects_active` outer run_if, which is true any time gameplay is running. All five iterate potentially large queries every frame during normal gameplay, even when no fire/electric/pending-damage components exist. | Add `any_with_component` guards matching the component they iterate: `PendingDamageEffect`, `FireDoT`, `Shocked`, `ElectricArcVisual`, and `OriginalMaterial.or(any_with_component::<FireDoT>)...` respectively. |
| U-06 | ArchitecturalDecay | `components.rs:12–15` | Low | S | **Blanket `pub use` re-exports of three entire sub-modules** (`animation.rs`, `status_effects.rs`, `unit_type.rs`) from `components.rs`. This means `use super::components::*` (as in `systems.rs` lines 6–18) imports types from three conceptually distinct modules. Callers cannot tell which module actually owns a type. | Import from the owning module directly (e.g. `use super::animation::WalkingAnimation`, `use super::status_effects::FrozenSolidModifier`). Remove the blanket `pub use` re-exports from `components.rs`, or at minimum scope them to only the types that are genuinely cross-cutting. |
| U-07 | ConsistencyRot | `systems.rs:697–1202` | Low | S | **`ElectricArcVisual` component is defined inside `systems.rs`** (line 688–692), not in `components.rs` or `status_effects.rs` where all other effect components live. The pattern for every other effect component is to define it in a data file and implement the tick/visual system separately. | Move `ElectricArcVisual` struct to `status_effects.rs` (or a new `dots.rs` when U-01 is addressed). |
| U-08 | TypeContract | `components.rs:395–400` | Low | S | **`static SETUP_IMMUNITY_ACTIVE: AtomicBool` with free-function accessors** (`set_setup_immunity`, `is_setup_immune`) lives in `components.rs`, a data-only file. Process-global hidden state that is invisible to Bevy's ECS makes testability harder and violates the convention that `components.rs` contains only component/trait definitions. | Move to a dedicated `immunity.rs` or into `damage.rs` alongside the damage helpers that read it (`apply_damage_to_unit`, `apply_spell_damage_inner`). |
| U-09 | TestDebt | `components.rs` / `status_effects.rs` | Low | M | **No tests for core game-mechanic logic in status effects and DoT components.** `Effectiveness::recalculate` has 8 tests (good). But `AttackTiming::can_attack` and `can_attack_with_speed_bonus` — a subtle timer-wrapping algorithm annotated with a comment about a 60-hit-per-second instakill — have zero tests. Similarly `FireDoT::update`, `Shocked::update/can_arc`, and `SlowMovementModifier::apply` (strongest-wins semantics) are all untested. | Add unit tests for `AttackTiming::can_attack` (wrap-around, edge of window), `FireDoT::update` (tick timing, expiry), `Shocked::stack` (cap), and `SlowMovementModifier::apply` (strongest-wins). |

---

### Oversized files

| File | LOC | Exempt? | Reason / Proposed split |
|------|-----|---------|--------------------------|
| `systems.rs` | 1 878 | No | Four unrelated concerns: shared movement helpers + DoT systems + visual tinting + animation drivers. Propose split into: `dots.rs`, `vfx_tinting.rs`, `animation_systems.rs`, keep small `systems.rs` for movement helpers. |
| `components.rs` | 1 096 | No | Contains pure component data PLUS free functions (`apply_spell_damage`, `apply_damage_to_unit`), a static global, a macro, and tests. Propose: extract free damage helpers + immunity static to `damage_helpers.rs`, keep components/traits in `components.rs`. |
| `status_effects.rs` | 570 | Yes (exempt) | Long list of tightly-cohesive CC/buff/debuff component structs. Each struct is small; they genuinely belong together. No logic entanglement. |
| `animation.rs` | 426 | Yes (exempt) | All entries are `WalkingAnimation`, `CombatAnimation`, `DyingAnimation`, `RisingAnimation`, `PulsingAnimation` — clearly cohesive animation component set. No logic beyond tick/UV methods on each type. |
| `unit_type.rs` | 295 | Yes (exempt) | Single enum with match arms for display strings per variant — classic match-on-enum monolith. |

---

### Looks bad but is actually fine

- **`update_persistent_effect_visuals` query with 20+ tuple fields (systems.rs:949–1007)**: Looks alarming, but this is a well-known Bevy pattern for tinting logic that needs to read many independent Has<> markers in a single pass to avoid per-component material clones. The alternative (multiple passes) would produce many more material mutations per frame.
- **`calculate_weighted_movement` with 14 arguments (systems.rs:159)**: The `#[allow(clippy::too_many_arguments)]` is justified here. All 14 are genuinely independent modifier inputs; grouping them into a struct would just move the verbosity without reducing it.
- **`resurrect_corpse_as_infantry` stripping 12 component types (systems.rs:1353–1396)**: The long chain of `.remove::<X>()` calls is intentional defensive cleanup to ensure a re-raised corpse never carries stale unit-type markers from its previous life.
- **`process_pending_damage_effects` allocating `burning_patch_mesh` / `burning_patch_material` lazily (systems.rs:364–365)**: Looks like a per-frame allocation, but the `Option` variables are initialized to `None` and only filled on the first DRY synergy hit in a given frame. In practice, the Drought synergy is uncommon enough that this path is cold.
- **`impl_timed_modifier!` macro in `components.rs` (line 27)**: An in-file macro instead of a derive is unusual but is correct here — it auto-implements the `TimedModifier` trait (which delegates to the existing `update(delta)` method) without needing a proc-macro crate.
- **`SmellyModifier` implements `impl_timed_modifier!` in `status_effects.rs` instead of `components.rs` (line 525)**: Appears inconsistent with the other types listed in `components.rs:38–52`, but is necessary because the macro requires `update()` on the type and all other SmellyModifier code lives in `status_effects.rs`.

---

### Open questions

1. `process_pending_damage_effects` (systems.rs:332) spawns `BurningPatch` entities inline. Should that spawning logic be in a meteorologist-owned system, or is the central DoT processor the correct owner?
2. `ElectricArcVisual` component (systems.rs:688) has no Ghost/MP filtering in its update system (`update_electric_arc_visuals`, line 908). Arc visuals are cosmetic-only — is it intentional that ghosts can spawn these on the guest, or should they be owned by the host only?
3. `apply_unit_movement` (movement.rs:19) has no `Without<Corpse>` filter; instead `clear_corpse_velocity` runs afterward. Is the chained two-system approach preferred over filtering in `apply_unit_movement` itself?
