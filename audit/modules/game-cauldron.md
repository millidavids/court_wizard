## game-cauldron

**Scope:** `src/game/cauldron/` — brewing system, active buff tracking, army buff application, cauldron visuals, and multiplayer sync.

---

### Mental model

The cauldron module owns the full lifecycle of the Alchemist's brewing system. A player opens a UI menu (out of scope), assembles a `Recipe` from `Ingredient` enums, and fires a `StartBrewMessage`. `CauldronState` (a component on the cauldron entity) drives the brew timer; when it completes a `BrewCompleteMessage` pushes an `ActiveBuff` into `CauldronBuffs` (a global `Resource`). Per-frame army-buff systems read `CauldronBuffs` scalars and insert/remove marker components (`CauldronDamageBonus`, `CauldronDamageResistance`, `CauldronSpeedModifier`) on unit entities, which the combat layer reads. In multiplayer the guest Alchemist serialises its `CauldronArmyScalars` over the wire; the host receives them into `RemoteCauldronBuffs` and applies them to the guest's army. The module is well-decomposed with concern-sliced files and thorough run_if guards throughout the plugin.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| C-01 | TypeContract | `systems/army_buffs.rs:44-49` | High | S | `buff_defender_damage` and `buff_defender_resistance` only insert the component when it is absent (`existing.is_none()`), never updating it. If two Mistletoe brews are active simultaneously (stacking `ActiveBuff` entries), the summed `defender_damage_bonus()` grows but the already-inserted component retains the first brew's value. The same issue exists in `buff_defender_resistance` (line 65). The combat system reads the component value directly, so the stale value means later stacks silently under-buff. | Change the guard from `if existing.is_none()` to `if existing.map(\|m\| m.0) != Some(bonus)`, matching the already-correct pattern used in `apply_cauldron_speed_modifiers` (line 139). |
| C-02 | ArchitecturalDecay | `constants.rs:28-33` | Low | S | `CAULDRON_COOP_POSITION` is byte-for-byte identical to `CAULDRON_POSITION` (both are `WIZARD_POSITION + CAULDRON_OFFSET`). It exists only as a semantic alias. Any future tweak to the co-op position must remember to change this constant and not the base one, which is a maintenance trap. | Replace all call sites that choose between `CAULDRON_POSITION` and `CAULDRON_COOP_POSITION` with just `CAULDRON_POSITION`, and delete `CAULDRON_COOP_POSITION`. Add a comment at `CAULDRON_POSITION` noting it is also the co-op shared position. |
| C-03 | ArchitecturalDecay | `components.rs:30-31` | Low | S | `CauldronState::Cooldown` is suppressed with `#[allow(dead_code)]` and its `tick` arm at line 78 transitions to `Idle` immediately — the variant is never constructed anywhere in the codebase. The dead arm adds noise and the `#[allow]` attribute signals intentional abandonment. | Remove the `Cooldown` variant and its `tick` arm. If a cooldown feature is planned, track it in the issue tracker rather than leaving a dead branch in production code. |
| C-04 | ConsistencyRot | `systems/army_buffs.rs:209` and `systems/multiplayer.rs:121` | Low | S | `const MAX_SHIELD: f32 = 20.0` is defined twice (once as a local in `shield_defenders`, once as a local in `apply_guest_army_buffs`) and the `time_remaining = 5.0` magic literal appears four times across those two files. Both values belong in `constants.rs`. | Move `MAX_SHIELD` and the shield-keepalive duration to `cauldron/constants.rs` as named constants, and reference them from both systems. |
| C-05 | DocDrift | `brews/constants.rs:85` | Low | S | Section divider comment `// ===== New Ingredient Configs =====` is stale; the ingredients below it are now fully shipped and indistinguishable from the originals. The label implies they are tentative or recently added, which misleads future contributors. | Remove or rename the divider to reflect current content (e.g., `// ===== Extended Ingredients =====`). |
| C-06 | ErrorObservability | `systems/brew_lifecycle.rs:85` | Low | S | `save_unified(&save_file)` silently swallows any underlying I/O error (the inner `save_unified_save` returns `ConfigResult<()>` but the public wrapper logs and discards failures). The combo-unlock write is fire-and-forget with no call-site acknowledgement. | Add a `warn!` in the storage layer's error path, or at minimum add a comment at this call site acknowledging that the swallowed error is intentional. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|------------------------|
| `systems/army_buffs.rs` | 285 | true | All 8 functions are tightly coupled army-buff systems sharing the same components and resources. No natural split boundary exists; the file is just under the 300-line limit and is genuinely cohesive. |

No other file in scope exceeds 300 LOC.

---

### Looks bad but is actually fine

- **`receive_cauldron_buffs` drains `incoming_messages` and re-queues unhandled messages** (`multiplayer.rs:51-76`): This is the established codebase-wide pattern for cooperative message dispatch across independent receiver systems. It does not lose messages.
- **`apply_max_mana_buff` runs every frame without a `has_active_buffs` guard** (`plugin.rs:67`): Intentional — it must also run when no buffs are active to reset the guest's mana cap after buff expiry. Has an internal `f32::EPSILON` change-guard making it idempotent.
- **`buff_defender_effectiveness` runs every frame without a `has_active_buffs` guard** (`plugin.rs:95`): Same intentional self-resetting pattern; zeroes `cauldron_spell_bonus` when no effectiveness brew is active.
- **`PhilosophersStone` absent from `Ingredient::all()`** (`brews/ingredient.rs:131-152`): Confirmed intentional by `wizard_crud.rs:114` comment. It is a special once-per-battle slot handled via a separate UI selector.
- **`CAULDRON_POSITION` used as fallback in `handle_brew_complete`** (`brew_lifecycle.rs:131`): Defensive fallback for a `Query::single()` on an entity that is always present in gameplay; the fallback path cannot be reached.
- **`CauldronState::tick` calls `std::mem::take(self)`** (`components.rs:68`): Necessary because Rust's borrow checker won't allow moving out of `self` inside a mutable-reference match arm. The pattern is correct.
- **`handle_brew_complete` has 9 system parameters with `#[allow(clippy::too_many_arguments)]`**: Idiomatic Bevy; the allow attribute is appropriate per project conventions.

---

### Open questions

1. **Stacking damage/resistance brews (C-01)**: After fixing the insert-only guard, should `buff_defender_damage` write the new summed value into an already-present component on each frame, or is the intended design that a single brew is active at a time? The `active_buffs: Vec<ActiveBuff>` model clearly supports stacking; combat reads a single `Option<&CauldronDamageBonus>` per entity, so the fix should keep single-component-per-unit and update its value each frame.
2. **Co-op damage/resistance first-writer-wins**: `multiplayer.rs:135-139` acknowledges that when both co-op wizards brew damage or resistance, the first-writer wins (no summing). Is this accepted scope for now or should it be tracked as a known limitation?
3. **`CauldronState::Cooldown` (C-03)**: Was this removed from the roadmap? If a brew cooldown is still planned, an issue ticket is cleaner than a dead variant with `#[allow(dead_code)]`.
