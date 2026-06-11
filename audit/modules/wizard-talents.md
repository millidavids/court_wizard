## wizard-talents

**Scope:** `src/game/units/wizard/talents/` — resource management, progression logic, tier thresholds, and talent content definitions for all 31 spells.

---

### Mental model

The talents module is a self-contained progression subsystem. `BattleTalentProgress` accumulates per-spell usage counts during a battle and is flushed to persistent save data at battle end (`flush_to_save`). `ActiveTalents` loads the player's chosen talent selections from save on game entry, validates them against unlocked tiers, and exposes `get_selection(spell, tier)` for spell systems to branch on. The content layer (`definitions/`) is a pure data registry: one file per elemental school, each returning a `[[TalentDefinition; 3]; 3]` (3 tiers × 3 choices) for each spell it owns. `constants.rs` centralises tier unlock thresholds and display helpers. The plugin registers two lifecycle systems (init / cleanup) on state transitions — there are no `Update` systems in this module.

The module is small, clean, and well-partitioned. The most notable issues are (a) `plugin.rs` containing system bodies, (b) `definitions/mod.rs` acting as a logic file rather than a pure re-exporter, (c) duplicate talent names across spells that risk UI confusion, (d) the `implemented` field on `TalentDefinition` being entirely dead, and (e) the `Fireball` tier thresholds `[1, 2, 3]` that look like a debug/placeholder value left in production.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| T1 | ArchitecturalDecay | `talents/plugin.rs:33–50` | Medium | S | `plugin.rs` defines two system bodies (`init_talent_resources`, `cleanup_talent_resources`) in violation of the project rule that `plugin.rs` does Bevy registration only. | Move both functions to `resources.rs` (they are resource lifecycle helpers) and keep `plugin.rs` as pure registration. |
| T2 | ArchitecturalDecay | `talents/definitions/mod.rs:12–59` | Medium | S | `definitions/mod.rs` defines the `TalentDefinition` struct and the `talent_definitions()` dispatch function, making it a logic file rather than a pure `mod`/`pub use` re-exporter as required by project convention. | Move `TalentDefinition` and `talent_definitions()` into a sibling `definitions/registry.rs` (or `definitions/dispatcher.rs`), then `mod.rs` does only `mod` declarations and `pub(crate) use`. |
| T3 | ConsistencyRot | `talents/definitions/electric.rs:56,97` | Medium | S | The talent name `"Chain Reaction"` appears twice in `electric.rs` — once for Chain Lightning (tier 3, choice 1) and once for Lightning Rod (tier 2, choice 0). Same name collision also: `"Magnetic Pull"` appears in both `electric.rs:41` and `force.rs:97`; `"Scorched Earth"` appears in both `fire.rs:56` and `fire.rs:171`; `"Lingering Flames"` in `fire.rs:14` and `nature.rs:313`; `"Dimensional Rift"` in `force.rs:198` and `utility.rs:322`. Duplicate names across different spells are not a functional bug but will confuse players in the UI and create ambiguity in any future code that keys on talent names. | Disambiguate the colliding names with spell-contextual prefixes or unique names (e.g. "Lightning Chain Reaction" vs "Rod Chain Reaction", or choose distinct names). |
| T4 | TypeContract | `talents/constants.rs:8` | High | S | `Spell::Fireball => [1, 2, 3]` — tier thresholds of 1/2/3 are three orders of magnitude smaller than every other spell (e.g. MagicMissile: 50/400/1000, BattleHymn: 20/75/200). This makes all three Fireball talent tiers unlock on the very first use. Every other damage-dealing spell has meaningful thresholds. This is almost certainly a debug/placeholder value that was never updated for release. | Set Fireball thresholds to values comparable to other offense spells (e.g. `[20, 75, 200]`). Verify intentionality with the designer before changing. |
| T5 | TypeContract | `talents/definitions/mod.rs:20–21` | Low | S | `TalentDefinition.implemented: bool` is marked `#[allow(dead_code)]` and is never set to `false` anywhere in the codebase, nor read anywhere. It was presumably useful during initial development to track unimplemented talent effects, but has since become a no-op field with 31 × 9 = 279 `implemented: true` literals. | Remove the field and all `implemented: true` literals to reduce noise and eliminate the dead_code suppression. If the field is wanted for future use, convert it to a marker type or use a TODO comment instead. |
| T6 | TypeContract | `talents/resources.rs:76–89` | Low | S | `set_selection` and `has_talent` on `ActiveTalents` are both `#[allow(dead_code)]`. Neither is called anywhere outside the struct definition. | Remove if there are no plans to use them, or remove the `dead_code` suppression and let the compiler enforce usage. Keeping suppressed dead public API is misleading about what callers should use. |
| T7 | ConsistencyRot | multiple spell files (e.g. `guardian_circle/systems.rs:53,260`) | Low | S | Some call-sites use inline full-path `crate::game::units::wizard::talents::resources::BattleTalentProgress` as a system parameter type directly in a tuple (instead of importing at the top of the file like the majority of other call-sites do). This is a minor inconsistency in import style. Note: these files are OUTSIDE this scope boundary — this finding is noted for the cross-cutting auditor. No finding raised against in-scope files for this item. | (Cross-cutting; deferred to spell auditors.) |
| T8 | DocDrift | `talents/plugin.rs:29–32` | Low | S | The doc-comment on `init_talent_resources` says "Only creates resources if they don't already exist, since `InGameState::Running` can be re-entered (e.g., after spell book or pause menu)." The comment is accurate but the "spell book or pause menu" examples are more specific than what the code actually guards against. Minor but slightly misleading. | Either broaden the comment to say "any re-entry of `InGameState::Running`" or leave as-is; low priority. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `definitions/utility.rs` | 545 | true | Single-concern asset registry: 9 functions each returning a `[[TalentDefinition; 3]; 3]` struct literal for 9 utility spells. Every line is cohesive data. No logic. Splitting would not reduce comprehension. |
| `definitions/force.rs` | 341 | true | Same pattern: 6 functions, all pure struct-literal data for force spells. |
| `definitions/nature.rs` | 341 | true | Same pattern: 6 functions for nature spells. |
| `definitions/necrotic.rs` | 341 | true | Same pattern: 6 functions for necrotic spells. |

---

### Looks bad but is actually fine

- **`talent_definitions()` returns `[[TalentDefinition; 3]; 3]` by value (27 structs of `&'static str` fields):** This looks like a per-call allocation but each `TalentDefinition` is three `&'static str` pointers; the entire array is 27 × 3 × 8 = ~648 bytes on the stack. Call sites are triggered by UI hover / click, not per-frame loops. Acceptable.
- **`ActiveTalents::from_save()` does synchronous file I/O on `OnEnter(InGameState::Running)`:** This runs once on game start, not per-frame. Synchronous save I/O at load time is the established pattern in this codebase.
- **`flush_to_save` called only from `achievements/helpers.rs` via `BattleTalentProgress`:** The indirection (achievements helper owns the flush) looks odd at first but is correct — the achievements system is the natural place to check crossed thresholds and fire `TalentTierUnlockedMessage`.
- **`Telekinesis => [1, 5, 10]` thresholds:** Extremely low relative to damage spells, but Telekinesis picks up ingredients and each ingredient is a single event; low thresholds make sense for a utility spell with low usage frequency. Not a placeholder.
- **The `definitions/mod.rs` contains a `match` dispatch function:** The `talent_definitions()` dispatch is a single match-on-enum, which per project convention is exempt from the 300-LOC file-split rule. However the struct definition (`TalentDefinition`) also lives there, which is the actual violation (T2 above).
- **`Option<Res<ActiveTalents>>` / `Option<ResMut<BattleTalentProgress>>` everywhere:** Resources are optional params because they may not exist in menus or edge states. The pattern is consistent across all spell systems and matches the MP ghost-gating memory note.

---

### Open questions

1. Is `Spell::Fireball => [1, 2, 3]` intentional? If so, why does Fireball unlock all tiers in the first battle while every other spell requires tens-to-thousands of uses?
2. Is the `implemented` field on `TalentDefinition` still serving a purpose (e.g. driving any UI lock state or tooltip that was removed)? If not, cleaning it up removes 279 `implemented: true` literals.
3. `set_selection` and `has_talent` on `ActiveTalents` are dead. Are they placeholders for the talent-selection UI flow (currently handled elsewhere) or genuinely orphaned?
4. Should duplicate talent names (e.g. `"Chain Reaction"`, `"Dimensional Rift"`, `"Scorched Earth"`) be globally unique to support future features like achievement text or analytics keyed on talent names?
