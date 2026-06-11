## units-batch3

**Scope:** `src/game/units/brute/`, `src/game/units/assassin/`, `src/game/units/commander/`, `src/game/units/elite/`, `src/game/units/undead/`

---

### Mental Model

These five modules collectively handle three distinct concerns:

1. **Special attacker units** (brute, assassin): Heavyweight melee attacker with a rock-throw ability, and a fast flanking unit that targets archers over infantry. Both follow the standard movement-targeting split. Brute is already well-structured (`systems/movement.rs`, `systems/combat.rs`, `systems/spawn.rs`). Assassin fuses targeting, movement, and spawn into a single 230-line `systems.rs`.

2. **Cross-cutting buff framework** (commander, elite): `commander` provides a proximity-aura system that grants speed/damage bonuses to nearby allied units each frame; `elite` provides one-shot components (`EliteHealthBonus`, `EliteDamageBonus`, etc.) applied by external upgrade systems. Commander's `apply_commander_auras` system allocates a `Vec` and a `HashMap` every gameplay frame.

3. **Undead asset pre-loader** (undead): A minimal stub with no plugin — only a single `resources.rs` that is registered directly from `src/game/units/plugin.rs`. Correct design since undead are raised infantry, not an independent unit type.

All modules respect the `plugin.rs = registration only` and `mod.rs = re-exports only` conventions. All Update systems have `run_if` guards. No `.unwrap()` calls in production paths.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B3-01 | ArchitecturalDecay | `assassin/systems.rs:88` and `brute/systems/movement.rs:57` | Medium | M | `assassin_movement` and `brute_movement` both lack `Without<Corpse>` in their primary query filter. Corpse entities retain `Velocity`, `Acceleration`, and `MovementSpeed` after death. These systems will iterate and process them, running full CC checks and `calculate_weighted_movement` every frame on dead units. Every other movement system in scope (`update_assassin_targeting` line 29, infantry, healer) explicitly excludes corpses. | Add `Without<Corpse>` to the `With<Assassin>` and `With<Brute>` query filters in both movement systems. |
| B3-02 | Performance | `commander/systems.rs:41,45` | Medium | S | `apply_commander_auras` allocates a `Vec` (commander snapshot via `.collect()`) and a `HashMap<Entity, (Option<f32>, Option<f32>)>` from scratch on every frame the condition passes. The King is always present so the system runs every gameplay frame. With hundreds of units the HashMap grows proportionally. | Use `Local<Vec<_>>` and `Local<HashMap<_, _>>` parameters on the system, clearing them at the top of each call rather than reallocating. Alternatively store them in a dedicated `CommanderAuraState` resource. |
| B3-03 | DocDrift | `assassin/constants.rs:20–21` | Low | S | `ASSASSIN_ATTACK_SPEED_BONUS` comment says `"Assassin attack speed bonus (2.0 = 3x attack speed)"` but the actual value is `0.0`. The comment describes a value that was never set. | Rewrite to `"Assassin attack speed bonus. Currently 0.0 (no bonus)."` |
| B3-04 | DocDrift | `elite/constants.rs:16–19` | Low | S | `ELITE_ATTACK_SPEED_BONUS` comment reads `"(FUTURE - not implemented in MVP)"`. This is wrong: the bonus is queried in `combat_systems/melee/combat.rs:252`, inserted by `loading/upgrade_systems.rs:85`, and attached in `infantry/systems/spawn.rs:209`. | Remove the stale "FUTURE - not implemented in MVP" note. Update `elite/components.rs:38–41` similarly. |
| B3-05 | ArchitecturalDecay | `assassin/resources.rs:8,20–23` and `brute/resources.rs:5` | Low | S | `AssassinAssets` carries `attacker_corpse_materials` and `undead_corpse_materials` fields suppressed by `#[allow(dead_code)]`; they are never read outside `resources.rs` — cleanup always falls back to infantry materials. `BruteAssets` is an empty stub (`pub struct BruteAssets;`) inserted at startup but never consumed by any system. | Remove the two dead corpse-material fields from `AssassinAssets` and their construction. Remove `BruteAssets` entirely along with the `preload_brute_assets` startup system and its registration in `BrutePlugin`. |
| B3-06 | ConsistencyRot | `assassin/plugin.rs:21` vs `brute/plugin.rs:23` | Low | S | Assassin plugin uses `any_exist::<Assassin>()` (custom wrapper from `run_conditions`) while brute and commander use `any_with_component::<T>` (Bevy built-in). Both are functionally identical; the inconsistency adds confusion about which form to use. | Standardise on `any_with_component` across all unit plugins. |
| B3-07 | ConsistencyRot | `commander/systems.rs:66` vs `assassin/systems.rs:46` | Low | S | Commander aura distance uses full 3D `Vec3::distance()` while assassin targeting uses XZ-only squared distance (`diff.x*diff.x + diff.z*diff.z`). Units are on a flat plane; every other targeting system uses the planar form. | Replace `unit_transform.translation.distance(*commander_pos)` with the XZ-planar distance to match the rest of the codebase. |
| B3-08 | ArchitecturalDecay | `commander/components.rs:27–28` | Low | S | `TeamFilter::Both` is marked `#[allow(dead_code)]` and has zero call sites anywhere in the codebase. | Remove the variant or add a concrete comment linking it to a planned feature so the suppressor is intentional. |
| B3-09 | DocDrift | `brute/systems/spawn.rs:21–22` | Low | S | `spawn_brute` doc comment says "Brutes spawn in the archer row alongside archers" but the inline comment immediately below reads "Brute spawns at the front with infantry" and the code calls `attacker_spawn_position(0, 0.0)` (no archer depth offset). | Correct the doc comment to match the code. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `assassin/systems.rs` | 230 | No | Below 300-line cap but fuses three concerns. Proposed: `systems/targeting.rs`, `systems/movement.rs`, `systems/spawn.rs` mirroring the brute layout. |

(All other files in scope are under 150 lines. Brute is already correctly split.)

---

### Looks Bad But Is Actually Fine

- **`assassin/systems.rs:49,60` — `partial_cmp().unwrap_or(Ordering::Equal)`**: Correct NaN-safe pattern for `min_by` on `f32` distances. Not an `.unwrap()` risk.
- **`brute/systems/spawn.rs:82` — `DamageMultiplier(0.0)` on spawn**: Placeholder for the commander aura system to overwrite. Harmless; not a zero-damage bug.
- **`assassin/constants.rs:21` — `ASSASSIN_ATTACK_SPEED_BONUS = 0.0`**: The constant IS wired to `melee.rs:256`. Zero value is intentional design (assassins kill archers in one hit, not via attack rate).
- **`commander/systems.rs` iterating ghost entities in MP**: `affected_units` lacks `Without<GhostEntity>`, so the system calls `remove::<DamageMultiplier>()` and `remove::<CommanderAuraSpeedModifier>()` on ghost entities. Both removals are no-ops when the component is absent (ghost entities don't carry these). No incorrect state results.
- **`commander/plugin.rs` — `is_gameplay_active` vs `is_gameplay_running`**: Commander correctly uses the broader condition so aura buffs persist during pause overlays rather than being stripped and re-applied on resume.
- **`undead/` has no plugin, no systems**: Correct design; undead are raised infantry (`Team::Undead`) handled by shared systems.
- **`brute/systems/spawn.rs:27` — `_current_level` unused parameter**: Kept for future level-scaling. Harmless forward-planning.
- **Assassin lacks `FlockingModifier` component on spawn**: Intentional — assassins route around infantry via flow-field, not flocking. `apply_separation` defaults to 1.0 multipliers for entities without `FlockingModifier`.

---

### Open Questions

1. **Corpse movement (B3-01)**: Do corpses have `Velocity` zeroed by another system before `assassin_movement`/`brute_movement` run? If so, the missing `Without<Corpse>` is cosmetic rather than a real performance drain.
2. **Assassin corpse materials (B3-05)**: Were `attacker_corpse_materials`/`undead_corpse_materials` on `AssassinAssets` originally used for assassin-specific corpse visuals and later deprecated, or copied from another asset struct by mistake?
3. **Commander aura deferred-command pressure (B3-02)**: `apply_commander_auras` issues `insert`/`remove` commands for every unit every frame regardless of whether the value changed. For large waves this creates significant command-queue churn. Would a "dirty flag" approach (only issue commands when the buff value changes) be worth the complexity?
4. **`ASSASSIN_ATTACK_SPEED_BONUS = 0.0` design intent**: The comment describes a 2.0 value. Was faster attack speed ever the plan for assassins, or should this tuning constant be removed entirely?
