## boss-hags

**Scope:** `src/game/units/boss/hags/` — spawn, movement, combat, eye-transfer mechanic, and all three hag ability systems (Justina chain-lightning/fireball, Josephina leap/mauling/consume, Martina teleport-pull/mind-control).

---

### Mental model

The hag boss fight is a three-unit encounter governed by a rotating "eye" mechanic. Two magical eyes (invulnerability and ability) cycle between the three hags on a timer, making each hag temporarily unkillable or able to use her special abilities. A hag permanently dies only when blind (holding no eyes) and reduced to zero HP. Each permanent death removes an eye from the pool, and the last survivor enrages.

The module is feature-sliced into `core.rs` (spawn, movement, combat, eye-system), `abilities.rs` (per-hag ability systems), and small supporting files. `systems.rs` is a thin re-export shim. `plugin.rs` is registration-only.

The five mind-control gameplay systems (`update_mind_controlled_targeting`, `mind_controlled_pursue_allies`, `update_mind_control_wear_off`, `mind_controlled_combat`, `cleanup_retaliation_targets`) live in `abilities.rs` but are *registered* inside the wizard spell's `mind_control/plugin.rs`. This ownership split is architecturally awkward—these systems are general MC behavior shared between the wizard spell and Martina, yet they live in the hag module and are public.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| H01 | ArchitecturalDecay | `abilities.rs:790–1024` | High | M | Five general mind-control systems (`update_mind_controlled_targeting`, `mind_controlled_pursue_allies`, `update_mind_control_wear_off`, `mind_controlled_combat`, `cleanup_retaliation_targets`) live in the hag module but are registered in `mind_control/plugin.rs` and implement shared MC behavior not specific to hags. Their registration via `crate::game::units::boss::hags::systems::*` creates a hard dependency from the wizard spell module into the boss module. | Move these five systems into `src/game/units/wizard/spells/mind_control/systems.rs` (or a shared `mind_control/mc_systems.rs`) and update the `pub use` chain. The hag module should only call `MindControlled` component inserts, not own the MC runtime. |
| H02 | ArchitecturalDecay | `abilities.rs:662–671` | Medium | S | `guard_entities` is populated in the pre-pass loop in `martina_teleport_pull` but is never read afterward. The comment at line 728 says "Guards will snap to king via their existing system" — so the logic was deliberately deferred, but the dead collection allocates every cast. | Remove the `guard_entities` `Vec`, the `Option<&KingsGuard>` query component, and the else-if branch that populates it, or implement the missing guard-teleport logic. |
| H03 | TypeContract | `components.rs:96–116` | Medium | S | `ChainLightningCooldown::new()` initializes `time_remaining` to `2.0` and `FireballCooldown::new()` to `5.0`, but the constants `CHAIN_LIGHTNING_COOLDOWN = 1.0` and `FIREBALL_COOLDOWN = 2.0` are defined in `constants.rs`. These are different values—the `new()` constructors use startup-delay offsets not tied to any constant, so the initial cooldowns silently diverge from the gameplay cooldowns with no documentation explaining the intent. | Add named constants `CHAIN_LIGHTNING_INITIAL_COOLDOWN` and `FIREBALL_INITIAL_COOLDOWN` (or reuse the same constants) and replace the magic literals. At minimum add a doc comment explaining that the initial values are intentionally different from the cycle cooldowns. `TeleportPullCooldown::new()` similarly initializes to `6.0` while `TELEPORT_PULL_COOLDOWN = 2.0`. |
| H04 | ArchitecturalDecay | `core.rs:418–489` | Medium | S | `hag_combat` does two O(n) scans over the full target set per hag per frame: first a has-target check (lines 420–435), then a nearest-target scan (lines 452–476). The first pass can be eliminated by combining both into the second scan — if `nearest_target` is still `None` after the pass, skip the reset. | Collapse into a single pass: track `nearest_target` directly, and only `reset` and apply the attack if `nearest_target` is `Some`. |
| H05 | TypeContract | `abilities.rs:778` | Medium | S | `MindControlled.wear_off_duration` is hardcoded to `300.0` (5 minutes) in `martina_mind_control`. This magic number has no constant, no comment explaining why Martina's duration differs from any wizard-cast value, and is indistinguishable from "permanent". If the value ever needs balancing it must be hunted down manually. | Extract to a named constant `MARTINA_MIND_CONTROL_DURATION` in `constants.rs`. |
| H06 | ErrorObservability | `core.rs:801,854` | Low | S | Two `.expect("checked above")` calls in `tick_eye_transfer` rely on the logical invariant that `needs_flight` is only true when the source `Option` is `Some`. The invariant holds at the call site, but the pattern would silently panic if the control flow were ever refactored. | Restructure to use an inner `if let` binding instead, eliminating the need for the `.expect` calls. |
| H07 | DocDrift | `abilities.rs:51–52` | Low | S | The doc comment on `justina_chain_lightning` reads "Builds a `WalkingAnimation` configured for the hag sprite sheet" — this is a copy-paste artifact from `hag_walking_animation` in `core.rs` and has nothing to do with chain lightning. | Replace with the correct doc string describing the chain lightning ability. |
| H08 | Performance | `core.rs:605` | Low | S | `hag_separation` allocates a `Vec<(Entity, Vec3)>` of all hag positions every frame it runs (even when no hags are close enough to need separation). With only 3 hags this cost is minimal, but the pattern is inconsistent with how the rest of the engine avoids per-frame allocation. | Use `hags.iter()` directly in the inner loop with an early-out on the same entity, removing the snapshot allocation. |
| H09 | ConsistencyRot | `abilities.rs:43–48` | Low | S | `MindControlPursuitFilter` type alias (line 41) is defined in-file but the similar `MindControlTargetData` and `MindControlTargetFilter` aliases (lines 26–38) are also in-file with no pattern for where these belong. Since the five MC systems are already out-of-place in this module (see H01), these type aliases should move with them. | Addressed by H01 — move to `mind_control` module. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `core.rs` | 1120 | No | Mixes spawn logic, animation helpers, movement, combat, eye-system. Proposed split: `spawn.rs` (spawn_hags, spawn_eye_visual), `movement.rs` (hag_movement, hag_separation, justina_kite_distance, update_hag_targeting), `combat.rs` (hag_combat), `eye_transfer.rs` (tick_eye_transfer, update_eye_flight), `death.rs` (intercept_blind_hag_death, apply_enrage_to_last_hag, resurrect_eyed_hags), `animation.rs` (hag_walking_animation, hag_combat_animation, hag_attack_animation, hag_casting_animation, set_hag_attack_pose_frame, restore_hag_walking_pose, eye_pulsing_animation). |
| `abilities.rs` | 1024 | No | Contains three distinct hag ability sets plus five unrelated general MC systems. Proposed split: `justina.rs` (chain lightning + fireball), `josephina.rs` (leap, mauling, corpse consume), `martina.rs` (teleport pull + aura mind control), with the five MC systems moved to `mind_control/` (see H01). |

---

### Looks bad but is actually fine

- **`pub use super::abilities::*` and `pub use super::core::*` in `systems.rs`** — looks like a wildcard export blob, but this is a documented Phase 15 re-export shim. The downstream consumer (`plugin.rs`) imports `use super::systems::*` and gets exactly the system functions it needs, preserving the original flat public API while the implementation is split. Intentional.
- **`martina_mind_control` has no ability-eye check** — the aura always fires whenever Martina is alive and not staged. On inspection, the aura *is* an always-on passive for Martina (the `martina_query` doesn't include `HagEyeState`); only the *teleport pull* is eye-gated. The comment in `plugin.rs` labels mind control as a Martina ability correctly; this is a design choice, not a bug.
- **`resurrect_eyed_hags` registered in `game/plugin.rs` rather than `HagsPlugin`** — looks like a stray registration but is intentional: it must run before the generic corpse-conversion system in the global update order, which is easier to guarantee from the top-level plugin scheduling than from within `HagsPlugin`.
- **Double `hags.get(source)` call pattern in `tick_eye_transfer` (lines 803–804 then 814)** — the first borrows mutably (eye state), the second borrows immutably (transform). Bevy's query system requires separate borrow accesses; this is correct Bevy borrow-split usage, not a logic error.
- **`justina_kite_distance` queries all hags then filters by identity inside the loop** — the function runs without StagingAttacker filter but quickly skips non-Justina entries. Given there are only ever 3 hags and the run_if guards the system on `any_with_component::<Hag>`, this is negligible and is consistent with the multi-hag query patterns elsewhere in the module.

---

### Open questions

1. **MC system ownership**: Was `abilities.rs` chosen as the home for the five MC runtime systems because Martina *creates* MC units, making it feel natural? Or was it an incremental convenience decision? If there are future non-hag sources of mind control (other bosses, etc.), these systems will need to stay general — confirming the rationale for H01.
2. **`guard_entities` future intent**: Is the "guards teleport with the king" behavior still planned (the dead collection in `martina_teleport_pull` suggests it was started), or has it been ruled out? The KingsGuard query component should be removed if the behavior is dropped.
3. **Initial cooldown values vs cycle cooldowns (H03)**: Is the offset intentional (e.g., "don't fire abilities the instant the fight starts") or accidental drift? If intentional, a constant-level comment would prevent future confusion.
4. **`resurrect_eyed_hags` placement**: The system is a hag-specific mechanic registered in `game/plugin.rs`. Should it be moved into `HagsPlugin` with an explicit system ordering annotation relative to the corpse-conversion system?
