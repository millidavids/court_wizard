## wizard-root

**Scope:** `src/game/units/wizard/*.rs` (root-level), `src/game/units/wizard/aim_line/`, `src/game/units/wizard/spell_range_indicator/`, `src/game/units/wizard/systems/`, `src/game/units/wizard/wizard_state/`

---

### Mental model

The wizard-root module is the structural spine of the player-controlled unit: it defines the `Spell` enum (with all 31 spells and every piece of per-spell metadata inlined as const match arms), the wizard's component types (`Wizard`, `Mana`, `ManaRegen`, `CastingState`, `PrimedSpell`, `WizardAnimation`), the wizard asset resource, and the runtime systems that run every frame (mana regen, animation, spell priming via messages, empowerment reset, mana-cost sync). Sub-modules `aim_line/` and `spell_range_indicator/` are compact visual systems—gamepad crosshair line and range circle respectively. The module is well-factored into concern-named files; the only structural outlier is `spell_enum.rs` at 764 LOC, which is a canonical match-on-enum registry and therefore exempt from the 300-line split rule.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| W1 | DocDrift | `spell_enum.rs:718–722` | Medium | S | `Grease` is grouped in the `Burning` arm of `status_effects()`, meaning the spell book shows "Burning" as its primary effect. The spell's own description (line 329) and its actual implementation (`spells/grease/casting/slow.rs`) make clear that slowing is the primary effect; ignition (burning) is a secondary interaction triggered only by fire spells. Players reading the spell book will be misled. | Move `Grease` out of the `Burning` arm and add `&[StatusEffectKind::Slowed]` to it. Add `Burning` only in a comment noting it appears on ignition, or leave ignition-only effects undocumented per the existing convention. |
| W2 | DocDrift | `spell_enum.rs:576` | Low | S | `WallOfStone` is listed under the comment `// Root spells (immediately researchable)` with `research_cost = 30`, but its `prerequisite()` (line 623) returns `Some(Spell::SpikeGrowth)`. `SpikeGrowth` costs 60 and is itself second-tier. WallOfStone is not researchable until SpikeGrowth is unlocked, so the comment is wrong and the cost tier is arguably wrong too. | Fix the comment to reflect WallOfStone's second-tier status, and consider raising its cost to the second-tier range (50–60) so `research_cost` matches its tree depth. |
| W3 | ArchitecturalDecay | `spell_enum.rs:647–650` | Low | S | `required_total_spells()` always returns `0` with the comment "All spells are now in prerequisite chains; no gate requirements needed." The function is called in two UI files (`study_tab/panels/helpers.rs:47`, `spell_detail.rs:229`). These call sites compute and display a gate value that is permanently zero—dead work and dead UI branches. | Either delete `required_total_spells()` and the two call sites (if the feature is permanently retired), or leave a `#[deprecated]` note so future authors understand the gap. |
| W4 | ConsistencyRot | `systems/runtime.rs:207–218` | Low | S | `talent_cast_time_multiplier()` is a private helper inside `runtime.rs` that only handles `Spell::BlackHole`. It references `black_hole_constants::QUICK_COLLAPSE_CAST_TIME_MULT` via a long cross-module path. As the talent count grows, each new talent will need another arm here, coupling the wizard runtime to individual spell constant modules. | Consider moving per-spell talent cast-time overrides into each spell's own constants file (e.g. as a method on `PrimedSpell` or a constant consumed by `primed_config()`), so the wizard runtime doesn't accumulate a growing match arm per spell. |
| W5 | Performance | `spell_range_indicator/systems.rs:68–86` | Low | S | `pulse_spell_range_indicator` runs every Update frame and unconditionally writes `base_color` and `alpha_mode` to the material via `get_mut`, which marks the asset dirty every frame even when the delta is sub-pixel. The `alpha_mode` field never changes after setup but is set on every frame. | Write only `base_color` (drop the unconditional `alpha_mode` assignment); or use `set_if_neq` semantics by comparing the current alpha before calling `get_mut`. |
| W6 | DocDrift | `spell_range_indicator/systems.rs:110` | Low | S | Inline comment `// Thin ring, 5 units wide (half of previous 10)` is a developer changelog note embedded in source. It reveals refactoring history but will become stale if the value changes again. | Replace with a purpose comment: `// ring cross-section thickness in world units`. |
| W7 | TypeContract | `spell_range_indicator/systems.rs:96–98` | Low | S | `spawn_range_circle` computes `initial_scale = actual_ground_radius / base_ground_radius`. If the wizard spawns at `y >= BASE_HEIGHT (100.0)`, `base_ground_radius` can be zero or negative, producing `NaN`/`inf` scale. No guard is present. | Add `if base_ground_radius <= 0.0 { return; }` before the division. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `spell_enum.rs` | 764 | true | Single enum with all 31 spells; every public function is a `match self { ... }` covering all variants. Splitting by concern (metadata vs config vs tree) would scatter a single enum's arms across files with no real benefit. Canonical registry monolith — exempt. |
| `wizard_state/casting_state.rs` | 235 | true | All content is the `CastingState` state-machine enum plus directly associated types (`PrimedSpell`, `GlobalCastCooldown`, `SpellCaster`). Every method is a small accessor on the same state machine. Genuinely cohesive; no split warranted. |
| `systems/runtime.rs` | 292 | true | Borderline at 292 lines. Every function is a distinct Bevy system; no single function exceeds ~50 lines. Consistent system-per-function structure; acceptable without a split. |

---

### Looks bad but is actually fine

- **`regenerate_mana` / `mana_on_kill` running on `With<Wizard>` (not `LocalWizard`).** In multiplayer the host spawns a `GuestWizard` proxy that carries `Wizard` + `Mana`. These systems correctly regenerate the proxy's mana on the host so it can validate the guest's spell casts. Intentional.
- **`cancel_active_casts` / `reset_empowerment_after_cast` running on all `With<Wizard>`.** Same reasoning—the host must cancel/reset the guest wizard's cast state on round-end transitions.
- **`mana_on_kill` inline closure run condition in `plugin.rs:41–50`.** This looks noisy but is valid: the `ManaDrought` toggle is a game-mode-specific resource path that does not warrant a dedicated named run condition. Acceptable as-is.
- **`update_wizard_animation` running on all `With<Wizard>`.** GuestWizard has a separate sprite texture and its animation should tick on the host so the guest's wizard animates for both players. Correct.
- **`AimLinePlugin` running in both `InGameState::Running` and `MultiplayerGameState::Running`.** The aim line is a local-input visual; each MP peer runs it independently. The `or` is intentional, not a copy-paste error.
- **`components.rs` being a pure re-export hub.** The file re-exports from `spell_enum.rs`, `spell_status_effects.rs`, and `wizard_state/*`. It provides a stable public import path. Consistent with project conventions.

---

### Open questions

1. **`Grease` status_effects correctness (W1):** Was listing `Burning` for Grease intentional to advertise ignition potential, or is it a placement bug? If intentional, the `status_effects` doc comment should clarify that ignition-path effects are included.
2. **`WallOfStone` research cost vs tree depth (W2):** Is cost 30 for a non-root spell intentional ("powerful but cheap deep in the tree"), or did the cost not get updated when the prerequisite was changed from root to `SpikeGrowth`?
3. **`required_total_spells` retirement (W3):** Is the function permanently dead, or is there a design intent to re-introduce gate-by-count spells in a future update? If the former, clean it up; if the latter, document it with a `// TODO` comment.
