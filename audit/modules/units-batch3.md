## units-batch3

**Scope:** `src/game/units/brute/`, `src/game/units/assassin/`, `src/game/units/commander/`, `src/game/units/elite/`, `src/game/units/undead/`

---

### Mental Model

This batch covers five narrow unit modules. **Brute** is a scaled-up infantry attacker with a rock-throw ability; it reuses infantry sprites and the shared weighted-movement helpers. **Assassin** is a fast flanking unit that routes around infantry via a custom flow-field and directly charges archers; its movement is handled by shared helpers with a "never slow in melee" override. **Commander** is a pure aura-buff system — no unit-specific AI — that iterates all alive units each frame and inserts/removes `DamageMultiplier` and `CommanderAuraSpeedModifier` via proximity. **Elite** provides four orthogonal bonus components (`EliteHealthBonus`, `EliteDamageBonus`, `EliteSpeedBonus`, `EliteAttackSpeedBonus`) with two trivial systems for the health ones. **Undead** is an asset-holder only; the actual undead unit behavior is handled entirely by the shared infantry/pathfinding systems with `Team::Undead`.

Overall quality is solid. The five modules respect the plugin-purity and mod.rs conventions. All Update systems have `run_if` guards. The main concerns are a dead stub resource, a stale doc contradiction, a per-frame allocation in the hot commander system, and a stale "not implemented in MVP" comment that is now wrong.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B3-01 | DocDrift | `brute/systems.rs:28` vs `:36` | Low | S | Doc comment says "Brutes spawn in the archer row alongside archers," but the inline comment directly below says "Brute spawns at the front with infantry." The code calls `attacker_spawn_position(0, 0.0)` — no depth offset — matching the infantry row, not the archer row. | Remove the misleading doc comment at line 28 (or correct it to match the inline comment and code). |
| B3-02 | ArchitecturalDecay | `brute/resources.rs:5` + `brute/plugin.rs:14` | Low | S | `BruteAssets` is an empty stub resource (`pub struct BruteAssets;`). It is registered at startup via `preload_brute_assets` (which only calls `commands.insert_resource(BruteAssets)`) but is never consumed by any system — `spawn_brute` takes `Res<InfantryAssets>`, not `Res<BruteAssets>`. This adds a pointless startup system and a registered resource with zero value. | Delete `BruteAssets`, `preload_brute_assets`, and the `Startup` system registration in `BrutePlugin`. If brute ever gets its own sprites, add the resource then. |
| B3-03 | Performance | `commander/systems.rs:41,45` | Medium | S | `apply_commander_auras` allocates a `Vec` (`.collect()` on commander iterator) and a `HashMap<Entity, …>` from scratch every frame the system runs. With up to 9 commanders and hundreds of units on-field, `units_in_aura` can hold hundreds of entries, causing repeated heap allocation each frame. The system is gated on `any_with_component::<Commander>` so it always runs once commanders appear. | Pre-allocate both collections as `Local<Vec<_>>` and `Local<HashMap<_, _>>` resources on the system, clearing them at the top of each invocation instead of reallocating. Alternatively, keep a `CommanderAuraState` resource. |
| B3-04 | DocDrift | `elite/constants.rs:16` | Low | S | `ELITE_ATTACK_SPEED_BONUS` has the doc comment "Elite attack speed bonus as percentage (FUTURE - not implemented in MVP)." This is false: the bonus IS applied in `combat_systems/melee.rs:256` and IS inserted on upgraded units in `loading/upgrade_systems.rs:85`. | Remove or replace the "FUTURE - not implemented in MVP" note with an accurate description. Also delete the same stale note from `elite/components.rs:38–41`. |
| B3-05 | ArchitecturalDecay | `commander/components.rs:27` | Low | S | `TeamFilter::Both` is marked `#[allow(dead_code)]` and has zero call sites anywhere in the codebase. It has existed since the commander module was introduced. | Either remove `Both` or add a `// Reserved for future allied-commander cross-buffing` comment to make intent explicit and suppress the suppressor. |
| B3-06 | ArchitecturalDecay | `brute/systems.rs` (326 LOC) | Low | M | `brute/systems.rs` has four concerns at 326 lines: spawn (`spawn_brute` ~75 LOC), targeting (`update_brute_targeting` ~40 LOC), movement (`brute_movement` ~90 LOC), and rock-throw (`brute_rock_throw` ~85 LOC). The project convention is to split files exceeding ~300 LOC when they are not single-concern monoliths. This file has four distinct behaviors. | Split into `spawn.rs`, `targeting.rs`, `movement.rs`, `rock_throw.rs`. Plugin registration stays in `plugin.rs` unchanged. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `brute/systems.rs` | 326 | false | Four distinct concerns: split into `spawn.rs`, `targeting.rs`, `movement.rs`, `rock_throw.rs` |

---

### Looks Bad But Is Actually Fine

- **`assassin/systems.rs:49,60` — `partial_cmp().unwrap_or(Ordering::Equal)`**: Looks like an unwrap but is the correct NaN-safe pattern for `min_by` on `f32` distances. No float will be NaN here (it is a sqrt result guarded by real positions), but the fallback is defensive and correct.
- **Assassin lacks `FlockingModifier` component**: Brutes, archers, and kings pre-insert a `FlockingModifier`; assassins do not. `apply_separation` in `movement_systems.rs` takes `Option<&FlockingModifier>` and defaults to 1.0 multipliers — correct, intentional, documented by the "assassins pass through non-assassin units" comment.
- **`brute/systems.rs:88` — `DamageMultiplier(0.0)` on spawn**: Looks like a bug (0% bonus = no effect). It is a pre-populated placeholder slot; the commander aura system will `insert()` a real value when the brute enters an aura and `remove()` it when it leaves. Without the slot the first removal-pass of the commander system would call `remove::<DamageMultiplier>()` on an entity that never had it, which is a no-op — so the placeholder is technically unnecessary, but it is harmless and communicates intent.
- **`assassin/constants.rs:21` — `ASSASSIN_ATTACK_SPEED_BONUS = 0.0`**: This constant IS used in `melee.rs:256`, so it is not dead code. A 0.0 value means no bonus, which is intentional (assassins kill archers in one hit rather than being faster attackers).
- **`commander/systems.rs` touching ghost entities**: `apply_commander_auras` iterates `affected_units: Query<(Entity, &Transform, &Team), Without<Corpse>>`, which includes ghost entities in multiplayer. The commander system will attempt to `remove::<DamageMultiplier>()` and `remove::<CommanderAuraSpeedModifier>()` from ghost entities — both are no-ops if the component isn't present (ghost entities spawned via `guest_snapshot.rs` carry neither). Harmless.
- **`commander/plugin.rs` — `is_gameplay_active` vs `is_gameplay_running`**: Commander uses the broader `is_gameplay_active` (runs even while paused) while brute/assassin/elite use `is_gameplay_running`. This is intentional — aura buffs should persist during pause overlays, not be stripped and re-applied on resume.
- **`undead/` has no plugin, no systems**: Undead units are raised by the `raise_the_dead` spell, which spawns them as infantry entities with `Team::Undead`. All gameplay is handled by shared systems. The module is correctly scoped to asset-holding only.
- **`brute/systems.rs:34` — `_current_level: Res<CurrentLevel>` (unused param)**: Common pattern in spawn functions to keep the signature extensible for future scaling. Not harmful.

---

### Open Questions

1. **`ASSASSIN_ATTACK_SPEED_BONUS = 0.0`**: The doc comment says "2.0 = 3x attack speed." Was there a design intent to give assassins faster attacks against archers? The current 0.0 means the field is purely a no-op tuning hook.
2. **Commander aura removal touches ALL units every frame**: For large waves the O(commanders × units) distance check plus iterating every alive unit for removal scales poorly. Are there plans to cap wave sizes or could this system be converted to a proximity event?
3. **`TeamFilter::Both` reserved variant**: Is there a planned use case for cross-team commander auras (e.g., necromancer buffs undead + attacker allies)? If not, should it be removed entirely?
4. **Brute always spawns at index 0 / no depth offset**: Is there a future plan for multiple brutes or variable spawn positions? The `unit_index` parameter concept seen in `spawn_single_attacker_assassin` is absent here.
