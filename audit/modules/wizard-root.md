## wizard-root

**Scope:** `src/game/units/wizard/*.rs` (root-level), `aim_line/`, `spell_range_indicator/`

---

### Mental model

The wizard root module is the hub for the player-controlled wizard entity. It owns the wizard's component types (`Wizard`, `Mana`, `CastingState`, `PrimedSpell`, etc.), the `Spell` enum registry with all 31 spells' data/metadata methods, runtime systems for mana regeneration / spell priming / animation, two visual sub-modules (`aim_line` for gamepad aim visualization, `spell_range_indicator` for the pulsing range ring), and UI-facing status-effect helpers. `WizardPlugin` delegates most logic to `SpellsPlugin`, `ArchetypesPlugin`, `AimLinePlugin`, and `SpellRangeIndicatorPlugin`.

The module is well-guarded by `is_spell_effects_active` / `is_local_wizard_active` run conditions and has no `.unwrap()` in production code. The main structural debt is `wizard_state.rs` (440 LOC, 12 distinct types) and `systems.rs` (414 LOC, mixes spawn/setup helpers with runtime systems), plus two small but concrete issues: duplicated `BASE_RADIUS`/`BASE_HEIGHT` constants between two functions in `spell_range_indicator/systems.rs`, and a stale `#[allow(dead_code)]` on `Spell::category()` which is actively used.

---

### Findings table

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| W1 | ArchitecturalDecay | `wizard_state.rs:1` | High | M | 440-line file holds 12 unrelated types (PrimedSpell, Wizard, Mana, ManaRegen, CastingState, GlobalCastCooldown, SpellCaster, LocalWizard, WizardAnimation, WizardAssets, GuestWizard, WizardInput). The 300-line exemption applies only to a single match-on-enum or asset registry; this is neither. | Split into at least `mana.rs` (Mana, ManaRegen), `casting_state.rs` (CastingState, GlobalCastCooldown, SpellCaster), `wizard.rs` (Wizard, LocalWizard, GuestWizard, WizardAssets, WizardInput, WizardAnimation). Keep `wizard_state.rs` as the transitional name or use the new split directly. |
| W2 | ArchitecturalDecay | `systems.rs:18-132` | Medium | M | `systems.rs` (414 LOC) mixes three concerns: asset loading (`load_wizard_assets`), entity spawning (`setup_wizard`, `apply_archetype_stat_bonuses`), and runtime gameplay systems (mana regen, priming, animation, etc.). The project convention is concern-focused feature files. | Extract `load_wizard_assets`, `setup_wizard`, and `apply_archetype_stat_bonuses` into a sibling `spawn.rs`. Keep runtime systems in `systems.rs`. Update `plugin.rs` and `mod.rs` accordingly. |
| W3 | ConsistencyRot | `spell_range_indicator/systems.rs:19-21,57-59` | Medium | S | `BASE_RADIUS = 3000.0` and `BASE_HEIGHT = 100.0` are duplicated as local `const` blocks inside both `setup_spell_range_indicator` and `update_spell_range_indicator`. `BASE_RADIUS` is exactly `DEFAULT_SPELL_RANGE` from `wizard/constants.rs` but the name is not reused, so a future change to the default range would silently desync the indicator. | Pull the two constants up to the module level in `spell_range_indicator/constants.rs` (or reuse `constants::DEFAULT_SPELL_RANGE` for `BASE_RADIUS`) and reference them from both functions. |
| W4 | ConsistencyRot | `spell_enum.rs:159` | Low | S | `#[allow(dead_code)]` on `Spell::category()` is stale. The method is actively called from `src/ui/cauldron_menu/setup.rs:426`, `src/ui/compendium/rows.rs:361,370`, and `src/ui/wizard_tower/layout/decorations.rs:45`. The annotation suppresses a warning that no longer fires and may mask a future real dead-code regression. | Remove the `#[allow(dead_code)]` attribute. |
| W5 | ConsistencyRot | `spell_range_indicator/systems.rs:50,53` | Low | S | `update_spell_range_indicator` uses `.iter().next()` on a `With<LocalWizard>` query (which is a single-entity query) and on the `SpellRangeCircle` query (also single). Every other wizard system in this codebase uses `.single()` / `.single_mut()`. The `.iter().next()` pattern silently succeeds when the entity is missing, potentially hiding a missing-entity bug. | Replace both `.iter().next()` guards with `.single()` / `.single_mut()` to be consistent with the rest of the codebase. |
| W6 | ArchitecturalDecay | `spell_status_effects.rs:128` | Low | S | `spawn_status_effects_section` is a UI spawning function (it builds Bevy UI nodes via `ChildSpawnerCommands`) living inside the game layer (`game/units/wizard/`). Its two callers are both in `src/ui/`. Per the feature-sliced architecture, UI code belongs in `src/ui/`. | Move `spawn_status_effects_section` and `effective_status_effects` (and `StatusEffectKind` if it has no other game-layer consumers) to a shared UI helper module, e.g., `src/ui/spell_book/status_effects.rs`, and update `src/ui/` callers to import from there. |
| W7 | Performance | `spell_range_indicator/systems.rs:128` | Low | S | `pulse_spell_range_indicator` runs every frame and calls `materials.get_mut()` on the range circle material, dirtying it unconditionally. This drives a GPU material re-upload every frame even when the alpha value change is imperceptibly small. | Apply the material change only when the computed `alpha` differs from the stored value by more than a small epsilon (e.g., 0.005), using `set_if_neq` on the material color. |
| W8 | ConsistencyRot | `wizard_state.rs:378` | Low | S | `WizardAnimation` has a `new()` method that returns all-zero fields but does not `#[derive(Default)]`. This is inconsistent with Bevy's idiomatic `Default` usage (and Clippy `new_without_default` lint). `CastingState` correctly derives `Default`. | Add `#[derive(Default)]` to `WizardAnimation` (default fields are zero/0) and replace `WizardAnimation::new()` calls with `WizardAnimation::default()`, or keep `new()` and add an explicit `impl Default`. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `spell_enum.rs` | 765 | Yes | Pure match-on-enum registry: every method is a `match self { ... }` data lookup. No logic, no system bodies. |
| `wizard_state.rs` | 440 | No | 12 distinct component/marker/resource types with no unifying concern. Proposed split: `mana.rs` (Mana, ManaRegen), `casting_state.rs` (CastingState, GlobalCastCooldown, SpellCaster), `wizard.rs` (Wizard, LocalWizard, GuestWizard, WizardAssets, WizardInput, WizardAnimation, PrimedSpell). |
| `systems.rs` | 414 | No | Mixes spawn/setup helpers (`load_wizard_assets`, `setup_wizard`, `apply_archetype_stat_bonuses`) with runtime systems. Proposed split: `spawn.rs` (asset loading + entity spawning helpers), `systems.rs` (runtime systems only). |

---

### Looks bad but is actually fine

- **`regenerate_mana` with `With<Wizard>` matching GuestWizard on host** — In co-op the host runs `regenerate_mana` on the guest wizard proxy entity. This is wasted CPU cycles, but the guest's mana on the host side is never read for casting decisions (the host drives `GuestWizard` spells from incoming `SpellCommand` messages, not by checking mana). It produces no gameplay correctness issue; it is merely redundant computation. Filed as a non-finding because the impact is negligible and the intent is to keep `Wizard` queries uniform.
- **Inline lambda `run_if` for `mana_on_kill` in `plugin.rs:41-50`** — The closure is verbose but it correctly gates the system on `ManaDrought` being active. An extracted `is_mana_drought` run-condition function would be cleaner but this is a style preference, not a correctness or performance issue.
- **`update_wizard_animation` using `With<Wizard>` (animates both wizards in co-op)** — Intentional: both the host wizard and guest wizard sprites animate independently, which is correct visual behavior.
- **`spell_status_effects.rs` mixing `StatusEffectKind` enum data with UI spawning** — While the caller sites are UI-only, the file sits in the game layer by design: `StatusEffectKind` is a game concept that the `Spell` enum references (`status_effects()` method on `Spell`). The `spawn_status_effects_section` UI function is a co-habitant of convenience. This is an architectural smell (finding W6) but not severe enough to call a bug.
- **`SpellCaster::new()` and `SpellCaster::with_indicator()` without `Default` derive** — `SpellCaster` intentionally has no `Default` because the two constructors encode distinct intent (`no indicator` vs `with indicator`). The absence of `Default` is defensible here.
- **`required_total_spells()` always returns 0** — The comment says all spells are now in prerequisite chains; the function exists to keep call sites at `interaction.rs:246` and `panels.rs:86` from needing conditional guards. This is intentional scaffolding, not dead logic.
- **`cancel_active_casts` using `With<Wizard>` (including GuestWizard on host side)** — Intentional. The multiplayer plugin also registers `cancel_active_casts` on `OnExit(MultiplayerGameState::Running)` specifically to cancel both wizards on pause/exit. This is documented in `src/game/multiplayer/plugin.rs:176`.

---

### Open questions

1. **`mana_on_kill` guest wizard mana** — The `mana_on_kill` system uses `With<Wizard>` and will restore mana on the GuestWizard proxy on the host. Is there a path where the host reads GuestWizard mana for anything (e.g., a future mana-gated guest spell check)? If so, the double-regen + on-kill restore will cause incorrect behavior. Clarify whether `With<LocalWizard>` is the right filter for mana-mutation systems.
2. **`BASE_HEIGHT = 100.0` in spell_range_indicator** — This constant represents the wizard's height above the ground plane for the range-projection calculation. It is not defined anywhere in `wizard/constants.rs` and its meaning is not documented. Is it the wizard's spawn Y-coordinate, a camera-related value, or an approximation? If the wizard's Y-coordinate changes, this will silently produce a wrong range circle.
3. **`GuestWizard` not gated from `apply_wizard_stats_to_primed_spell` and `reset_empowerment_after_cast`** — Both use `With<Wizard>`. The guest wizard entity has `CastingState` and `PrimedSpell` components on it (host drives the guest's cast via messages). Does it also get a `PrimedSpell` inserted, and is the empowerment-reset cycle correct for the host-driven flow?
