## game-combat_systems

**Scope:** `src/game/combat_systems/` (6 files, 862 LOC)

---

### Mental model

`combat_systems` is a two-file module that owns the two most critical per-frame gameplay loops: the melee hit-resolution system (`combat`) and the post-combat state-machine transitions (`convert_dead_to_corpses`, `enforce_invulnerability`). Both systems are registered in the game-wide `PostCombatSet` which is gated by `is_gameplay_running` — a host-only run condition — so ghost entities are never processed (MP safety is architecturally guaranteed by the set gate, not by per-query `Without<GhostEntity>` guards). The module is a pure system module: it contains no Bevy plugin, no resource definitions, and re-exports everything through a minimal `mod.rs`. The main pain points are query complexity and per-frame heap allocations in `melee.rs`, and a large but largely mechanical component-removal chain in `post_combat.rs` that is kept in sync by hand.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| C01 | Performance | melee.rs:127–149 | High | S | Three `Vec`/`HashSet` allocations every frame: `units_snapshot` (all unit positions), `flying_set` (HashSet), and `disorienting_snapshot` (fog zones). On a large wave these can be hundreds of elements each call. | Cache snapshots in a `Local<Vec<_>>` / `Local<HashSet<_>>` and `clear()` + `extend()` each frame to reuse the allocation, eliminating repeated heap pressure in the hottest system. |
| C02 | Performance | melee.rs:229–230 | Medium | S | Per-target `sqrt()` for distance comparison is called in the innermost filter_map on every unit×target pair. Since only the minimum is needed and range check is a scalar, compare squared distances directly (avoid `sqrt` on the filter side; keep it only for the final accepted target if the absolute value is needed). | Replace `distance = (dx*dx+dz*dz).sqrt()` with squared comparison; promote `attack_range` to squared too. Use `sqrt` only for the single chosen target if the raw distance is ever needed downstream. |
| C03 | ArchitecturalDecay | melee.rs:18–112 | Medium | M | The `combat` system signature is deeply nested with two levels of 16-tuple fields packed inside parenthesized sub-tuples. The Bevy 16-element WorldQuery limit forces nesting via `(A, B, (C, D, ...))` which is understandable, but there is no named type alias or doc comment explaining the nesting structure. Readability suffers — each destructuring level is anonymous. | Introduce type aliases (e.g. `type AttackerExtras<'w> = (Option<&'w RetaliationTarget>, ...)`) for the inner tuple groups. This does not change behaviour but makes the system signature self-documenting. |
| C04 | ArchitecturalDecay | melee.rs:280–282 | Low | S | Disorienting Vapors chance constant is referenced inline via a deep path (`super::super::units::wizard::spells::fog_cloud::constants::DISORIENTING_VAPORS_CHANCE`) without a `use` import. The same constant path could be imported at the top of the file like the other fog cloud types. | Add a top-level `use` import for `DISORIENTING_VAPORS_CHANCE` for consistency with the rest of the file. |
| C05 | ArchitecturalDecay | melee.rs:256–258 | Low | S | `ASSASSIN_ATTACK_SPEED_BONUS` is accessed via its full module path inline (`crate::game::units::assassin::constants::ASSASSIN_ATTACK_SPEED_BONUS`) without a use import, while other per-unit constants from assassin/shielder appear with full paths elsewhere. | Add top-level `use` imports for assassin and shielder constants used in the function body, consistent with how cauldron and unit component types are imported at the top. |
| C06 | ArchitecturalDecay | post_combat.rs:258–318 | Medium | M | The corpse-conversion system manually removes ~40 components in a single chained `.remove::<T>()` block. This list is maintained by hand; when a new status effect component is added to the game, it is easy to miss adding it here, leaving dead units with stale components. | Extract a marker bundle or a helper function `remove_all_status_effects(entity_cmds: &mut EntityCommands)` in `units/components.rs` (or `units/status_effects.rs`) so every status-effect cleanup path is co-located. The combat system calls the helper once. |
| C07 | ArchitecturalDecay | post_combat.rs:83–107 | Low | S | The death-asset resources are split into `death_assets` and `death_assets_2` as a workaround for Bevy's 16-tuple WorldQuery limit. This is technically sound but the naming is opaque and the split boundary is arbitrary. | Name the split semantically: e.g. `basic_death_assets` (infantry/archer/assassin/dispeller/undead/king) and `specialist_death_assets` (shielder/healer/aerialist/teleporter), with a comment explaining the limit. |
| C08 | TypeContract | post_combat.rs:126–145 | Low | S | `CLOSE_CALL_DISTANCE` is imported from `swordcerer::constants` inside a `use` statement within the function body (`use super::super::...`). This constant measures proximity of any kill to the wizard, not a swordcerer-exclusive mechanic. The constant living in the swordcerer archetype's `constants.rs` creates a misleading conceptual ownership. | Move `CLOSE_CALL_DISTANCE` to an achievements constants file or inline its value with a comment, so post_combat doesn't depend on a wizard-archetype module for a generic achievement trigger. |
| C09 | ConsistencyRot | melee.rs:1 / post_combat.rs:1 | Low | S | Both files use `super::super::` relative paths throughout (61 and 92 occurrences respectively) for all cross-cutting types rather than `crate::game::` absolute paths. This is inconsistent with some imports in the same files that already use `crate::game::...`. | Standardize on `use crate::game::...` absolute imports for all cross-cutting types; reserve `super::` for direct siblings only. Reduces visual noise when files are read in isolation. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| `melee.rs` | 523 | No | Single function that must be split for readability; proposed split: `targeting.rs` (target-selection helpers: snapshot building, range check, fog redirect), `damage_calc.rs` (modifier accumulation and apply_damage call), `attack_animation.rs` (combat animation dispatch), keeping `melee.rs` as the orchestrating system entry point. |
| `post_combat.rs` | 321 | No | Slightly over threshold; proposed split: `invulnerability.rs` (the 4-line `enforce_invulnerability`), `death_conversion.rs` (the large `convert_dead_to_corpses`). Or exempt if considered a single cohesive "death pipeline" — borderline. |

---

### Looks bad but is actually fine

- **No `Without<GhostEntity>` in combat queries** — looks like an MP bug. But `combat`, `enforce_invulnerability`, and `convert_dead_to_corpses` are all in `PostCombatSet` which is gated by `is_gameplay_running`, a run condition that only returns `true` on the **host** peer. Ghost entities only exist on the guest peer, so they can never reach these systems. The architectural guard is at the set level, not the query level.
- **`partial_cmp(...).unwrap_or(Ordering::Equal)` at melee.rs:243** — this is not a production `unwrap`. It is the standard idiomatic Rust pattern for float comparisons where NaN is impossible (squared distances are always finite), so `unwrap_or` is purely defensive and correct.
- **`wizard_query.single()` wrapped in `let Ok(...)` at post_combat.rs:138** — uses the fallible form, not the panicking one. Safe.
- **`death_assets` / `death_assets_2` tuple split** — awkward but necessary due to Bevy's 16-element system-parameter tuple limit. Not a code smell.
- **`PostCombatAction` enum defined at bottom of melee.rs** — private to the file, cohesive with its only consumer. Appropriate placement.
- **Long `remove::<T>()` chain** — while flagged above as a maintenance risk (C06), it is intentional and exhaustive. All components that must not exist on a corpse are listed explicitly. The exhaustive listing is preferable over silently missing a component.

---

### Open questions

1. Should `CLOSE_CALL_DISTANCE` be renamed / moved to an achievements constants module, or is swordcerer-archetype ownership intentional (only applies when swordcerer is active)?
2. Could `PostCombatAction` grow more variants as the game evolves? If so, is keeping it file-private in `melee.rs` sustainable, or should it move to a shared `post_combat_actions.rs`?
3. Is there an existing mechanism (bundle or helper) for "clear all status effects on death" that could replace the manual 40-component removal chain, or does each death path intentionally differ?
