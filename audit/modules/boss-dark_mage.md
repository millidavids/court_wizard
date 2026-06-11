## boss-dark_mage

**Scope:** `src/game/units/boss/dark_mage/` — 9 files, ~2 054 LOC total.

---

### Mental model

The Dark Mage is a telegraphed-AoE spellcaster boss with three spells (Dark Meteor, Shadow Lightning, Plague Cloud) and a reactive teleport. It uses an explicit state machine (`DarkMageState`) rather than the timer-driven approach of the Lich. Spell cooldowns, enrage multipliers, and telegraph indicators are all component-driven. Animation is a simple sprite-sheet that swaps material between a floating sheet and a casting sheet when state changes. The module is logically structured — `ai.rs` holds spawn + movement + AI state machine, `spells.rs` holds all spell systems and helper spawn functions, `animation.rs` handles visuals, `components.rs` holds all ECS types, `constants.rs` holds tuning values, and `resources.rs` loads assets at startup. A thin `systems.rs` re-export hub keeps the public surface clean.

Two architectural quirks stand out: the `spawn_dark_mage` function lives in `ai.rs` (other bosses isolate spawn in `spawn.rs`), and `spells.rs` is 811 LOC with genuinely mixed concerns (update systems + helper spawn functions + targeting logic + `PlagueHazardBroadcast` component).

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| DM-01 | ArchitecturalDecay | `spells.rs:1` | High | M | `spells.rs` (811 LOC) mixes four distinct concerns: update systems (`update_meteor_explosions`, `update_lightning_strikes`, `update_plague_clouds`, `update_plague_particles`, `update_meteor_projectiles`, `update_lightning_bolts`, `update_visual_effects`), helper spawn functions (`spawn_meteor_explosion`, `spawn_lightning_strike`, `spawn_plague_cloud`, `spawn_telegraph_indicators`), targeting logic (`find_spell_target`), and a component definition (`PlagueHazardBroadcast`). The project convention mandates files ≤300 LOC unless a single cohesive match/registry. This file is none of those. | Split into: `spell_updates.rs` (the 7 update systems), `spell_spawn.rs` (spawn helpers + `spawn_telegraph_indicators`), `targeting.rs` (`find_spell_target`). Move `PlagueHazardBroadcast` to `components.rs`. Keep `telegraph_duration`/`spell_cooldown` helpers alongside their callers in `spell_spawn.rs`. |
| DM-02 | ArchitecturalDecay | `ai.rs:36` | Medium | S | `spawn_dark_mage` lives in `ai.rs`. Every other boss (lich, ogre) isolates spawn logic in a dedicated `spawn.rs`. This makes `ai.rs` 587 LOC and violates the consistent pattern the project has established. | Extract `spawn_dark_mage` and its direct imports into a new `spawn.rs` sibling. `ai.rs` drops to ~470 LOC. |
| DM-03 | ConsistencyRot | `ai.rs:156, 274, 478` | Medium | S | `crate::game::units::systems::calculate_weighted_movement` and `crate::game::units::systems::is_cc_immobilized` are called with full crate-path qualifiers three times each rather than being imported at the top of `ai.rs`. Other boss files (e.g. `lich/combat.rs`) prefer `use` imports. The two inline paths for `Stunned` and `Petrified` at lines 245–246 and 444–445 are the same issue. | Add `use crate::game::units::systems::{calculate_weighted_movement, is_cc_immobilized};` and `use crate::game::units::components::{MeleeDamageReduction, Stunned, Petrified};` to `ai.rs`'s import block. |
| DM-04 | ConsistencyRot | `spells.rs:811` | Low | S | The `use crate::game::units::boss::utils::indicator_rotation;` import appears at the very bottom of `spells.rs` (line 811), after all function bodies. Rust permits this, but it violates the universal convention of grouping all `use` statements at the top of the file. | Move the import to the top-of-file `use` block alongside the other imports. |
| DM-05 | ConsistencyRot | `spells.rs:474–533` | Low | S | The circle indicator spawning code for `DarkMeteor` (lines 476–487) and `PlagueCloud` (lines 515–527) are near-identical: same mesh, same rotation, same component bundle — differing only in the radius scale. A 12-line duplication inside the same `match` arm. | Extract `spawn_circle_indicator(commands, assets, fill_material, center, radius)` returning `Entity`. Call it for both arms. |
| DM-06 | ConsistencyRot | `components.rs:63` | Low | S | `DarkMageSpellQueue::new()` and `DarkMageEnrage::new()` implement zero-argument constructors whose bodies are trivially `Self { field: default_value }`. The idiomatic Bevy/Rust pattern for this is `#[derive(Default)]`; `new()` is only meaningful when it accepts arguments (as `DarkMageSpellCooldowns::new` does). | Replace the no-arg `new()` impls with `#[derive(Default)]` on `DarkMageSpellQueue` and `DarkMageEnrage`, or keep `new()` for callsite clarity but also derive `Default`. |
| DM-07 | ArchitecturalDecay | `spells.rs:721` | Low | S | `PlagueHazardBroadcast` (a marker `Component`) is defined at the bottom of `spells.rs` (line 721) rather than in `components.rs` where all other Dark Mage components live. | Move `PlagueHazardBroadcast` to `components.rs`. |
| DM-08 | TypeContract | `constants.rs:51` | Low | S | `DARK_MAGE_DAMAGE_MULTIPLIER` is `-0.3` with the comment "Negative damage multiplier = takes less melee damage (like ogre)." The boss also has `DARK_MAGE_MELEE_DAMAGE_REDUCTION = 0.4` applied via `MeleeDamageReduction`. Two separate mechanisms both affecting the same damage pathway are active simultaneously with no comment explaining their interaction or whether both are intentional. | Add an explicit comment documenting that `DamageMultiplier` affects spell damage and `MeleeDamageReduction` specifically reduces melee damage, and both are intentional. Alternatively, verify in the damage pipeline that they do not stack unintentionally, and remove whichever is redundant. |
| DM-09 | Performance | `spells.rs:334–345` | Low | S | `find_spell_target` calls `.collect()` into a `Vec<Vec3>` (line 334) on every call — which happens every frame the mage is in `Idle` state while a spell is queued. The collected vec is then iterated again for O(n²) cluster scoring. | Pre-filter within the scoring loop without the intermediate allocation, or keep `collect` only when the set needs multiple passes. Given the low enemy count the actual impact is minor, but the allocation is avoidable. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `spells.rs` | 811 | No | Mixed update systems, spawn helpers, targeting logic, and a component. Split into: `spell_updates.rs`, `spell_spawn.rs`, `targeting.rs`. |
| `ai.rs` | 587 | No | Mixes spawn, movement, spell-queue, AI state machine, and teleport. Extract `spawn_dark_mage` → `spawn.rs`; consider extracting `dark_mage_teleport` → `teleport.rs` to reach ~300 LOC each. |
| `components.rs` | 210 | Yes | All entries are ECS component types — genuinely cohesive, no logic. No split needed. |
| `constants.rs` | 174 | Yes | All entries are named constants. Below 200-line split threshold and no mixed concerns. |

---

### Looks bad but is actually fine

- **`partial_cmp(...).unwrap_or(std::cmp::Ordering::Equal)` at `spells.rs:441`** — looks like a `.unwrap()` smell, but this is the standard Rust idiom for sorting floats where NaN cannot arise in practice (both are length-squared values of finite positions). The `unwrap_or` fallback makes it safe regardless.
- **`run_if(any_with_component::<DarkMage>)` double-guard on every system set** — looks redundant with `run_if(is_gameplay_running)`, but `any_with_component` additionally short-circuits when no DarkMage is alive (i.e. non-dark-mage waves), which is a meaningful performance guard.
- **`Dark_mage_enrage` using `PostCombatSet` ordering** — `dark_mage_enrage` runs after `PostCombatSet` so HP changes from the combat tick are visible before the enrage threshold is checked. Looks like an ordering quirk but is intentional.
- **`dark_mage_movement` calling `calculate_weighted_movement` with 7 `None` parameters** — the helper has many optional modifier slots that don't apply to the Dark Mage. This is idiomatic Bevy "pass None for features this unit doesn't use" — not a bug.
- **`spawn_dark_mage` doing two `.insert()` calls** after `.spawn()`** — Bevy allows split inserts; this is a deliberate pattern used across the codebase when the bundle exceeds tuple-size limits.
- **`systems.rs` being a 4-line re-export hub** — matches the pattern from `lich/systems.rs` and `hags/systems.rs`. Pure convention, not a violation.

---

### Open questions

1. Does the Dark Mage appear in multiplayer (co-op) sessions? None of the AI/damage/movement systems are gated `Without<GhostEntity>`. If it is possible for a ghost snapshot of a DarkMage to exist on the guest side, systems like `dark_mage_movement` and `dark_mage_spell_queue` would incorrectly run on ghost entities.
2. The `DARK_MAGE_DAMAGE_MULTIPLIER = -0.3` and `DARK_MAGE_MELEE_DAMAGE_REDUCTION = 0.4` (DM-08) — are these two mechanisms stacking, or does the pipeline apply only one? The lich uses only `DamageMultiplier(-0.5)`, not both.
3. `update_meteor_projectiles` (line 725) spawns with no `DarkMageFloatBase` or owner reference; is there a scenario where a pending projectile survives a wave reset and hits on the next wave?
