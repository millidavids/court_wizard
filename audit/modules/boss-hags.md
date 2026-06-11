## boss-hags

**Scope:** `src/game/units/boss/hags/` (all `.rs` files, including `core/` and `abilities/` subdirectories)

---

### Mental model

The Hags are a trio boss (Justina, Martina, Josephina) built around a shared "eye" mechanic: two floating eyes (invulnerability and ability) rotate among the three hags on a timer. A hag holding the invulnerability eye cannot be permanently killed; a hag holding the ability eye may use her special attacks. A blind hag (no eyes) that drops to zero health becomes permanently dead, and after two permanent deaths the last survivor enrages.

The module is well-decomposed overall: `core/` (animation, combat, death, eye_transfer, movement, spawn) handles the shared boss lifecycle, while `abilities/` (justina, josephina_leap, josephina_maul, martina_pull, martina_mc) handles per-hag abilities. `components.rs`, `constants.rs`, and `resources.rs` carry supporting types. Plugin and system set registration are clean.

The most significant structural problem is that `abilities/martina_mc.rs` contains five **general** `MindControlled` gameplay systems that have nothing to do with the hags. They are registered by `MindControlPlugin` (in the wizard spell tree) and operate on any `MindControlled` unit in the game. The misleading file name implies Martina ownership, but the code is a cross-cutting concern that belongs in `wizard/spells/mind_control/`. This misplacement causes a follow-on: three of the four MC gameplay systems lack `Without<GhostEntity>` exclusion (one — `mind_controlled_pursue_allies` — was correctly patched via `MindControlPursuitFilter`), creating potential ghost-gating risk in multiplayer.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| H-01 | ArchitecturalDecay | `abilities/martina_mc.rs:1-261` | High | M | `martina_mc.rs` contains five `MindControlled` behavior systems (`update_mind_controlled_targeting`, `mind_controlled_pursue_allies`, `update_mind_control_wear_off`, `mind_controlled_combat`, `cleanup_retaliation_targets`) with zero Martina-specific code. They are registered by `MindControlPlugin`, operate on any `MindControlled` unit, and create a hard reverse-dependency from `wizard/spells/mind_control/plugin.rs` back into `boss/hags/`. | Move these five functions to `wizard/spells/mind_control/systems/mc_behavior.rs` (or add them to the existing `effects.rs`). Update `MindControlPlugin` imports. Repurpose `martina_mc.rs` for Martina's aura (currently in `martina_pull.rs`). |
| H-02 | TypeContract | `abilities/martina_mc.rs:27` | High | S | `update_mind_controlled_targeting` (line 27), `update_mind_control_wear_off` (line 109), and `mind_controlled_combat` (line 174) query `MindControlled` units without `Without<GhostEntity>`. In multiplayer, `status_receive.rs:303` inserts `MindControlled` onto units by network ID. If ghost units on the host receive MC status (forwarded via the status pipeline), these systems would mutate their `TargetingVelocity`, tick their `time_elapsed`, and apply damage from ghost MC attackers — the exact ghost-gating bug class described in project memory. `mind_controlled_pursue_allies` correctly uses `Without<GhostEntity>` via `MindControlPursuitFilter:17`. | Add `Without<crate::game::multiplayer::components::GhostEntity>` to the `controlled` query in `update_mind_controlled_targeting`, `update_mind_control_wear_off`, and `mind_controlled_combat`. |
| H-03 | ArchitecturalDecay | `abilities/martina_pull.rs:193` | Medium | S | `MindControlled.wear_off_duration` is hardcoded to `300.0` (5 minutes) with only an inline comment. No named constant exists anywhere in the hags module for this value, and it differs from all wizard-spell MC durations, making it invisible during balance passes. | Add `pub const MARTINA_MC_DURATION: f32 = 300.0; // effectively permanent for fight duration` to `constants.rs`. |
| H-04 | ArchitecturalDecay | `abilities/josephina_leap.rs:145` | Low | S | Landing pause timer `0.3` is a bare float literal. Every other Josephina timing (`LEAP_COOLDOWN`, `LEAP_FLIGHT_DURATION`, `MAULING_DURATION`) lives in `constants.rs`; this value was missed. | Add `pub const LEAP_LANDING_DURATION: f32 = 0.3;` to `constants.rs` and reference it. |
| H-05 | ArchitecturalDecay | `abilities/martina_pull.rs:148` | Medium | S | `martina_mind_control` (Martina's passive aura) and `martina_teleport_pull` (her cooldown ability) live in the same file named `martina_pull.rs`. The name implies pull only. Since H-01 will free up `martina_mc.rs`, the aura should live there. | After H-01 is resolved, move `martina_mind_control` into `martina_mc.rs` so each file has a clear single concern. |
| H-06 | TypeContract | `components.rs:98,111` | Low | S | `ChainLightningCooldown::new()` initialises `time_remaining = 2.0` and `FireballCooldown::new()` to `5.0`, but `CHAIN_LIGHTNING_COOLDOWN = 1.0` and `FIREBALL_COOLDOWN = 2.0` in `constants.rs`. The constructors use different values (likely intentional startup delays), with no documentation explaining why. `TeleportPullCooldown::new()` similarly sets `6.0` vs constant `2.0`. | Add named constants `CHAIN_LIGHTNING_INITIAL_COOLDOWN`, `FIREBALL_INITIAL_COOLDOWN`, `TELEPORT_PULL_INITIAL_COOLDOWN` (or add doc comments explaining the offset is a startup-delay stagger). |
| H-07 | TypeContract | `components.rs:27` | Low | S | `HagEyeState::new()` manually constructs `{has_invulnerability_eye: false, has_ability_eye: false}`. The type should `#[derive(Default)]` since both fields are `false` which is Rust's `bool` default, allowing callers to use `HagEyeState::default()` and struct-update syntax. | Add `#[derive(Default)]` to `HagEyeState`; remove or delegate `new()`. |
| H-08 | ArchitecturalDecay | `core/eye_transfer.rs:104` | Low | M | The invulnerability-eye and ability-eye processing blocks in `tick_eye_transfer` (lines 103–154 and 157–204) are structurally identical: determine new holder, check if flight needed, despawn old visual, spawn `EyeInFlight`. A divergence between the two blocks (e.g. one getting a bug fix the other misses) has already happened in subtle form with the `spawn_eye_visual` re-offset logic. | Extract a private `fn initiate_eye_transfer(commands, hags, eye_visuals, hag_assets, eye_type, current_holder, new_holder)` helper to remove the duplication. |
| H-09 | DependencyConfig | `core/spawn.rs:9` | Low | S | `use crate::game::constants::*;` imports the full game constants namespace. Only `WIZARD_POSITION` and `attacker_spawn_position` appear to be used from it. | Replace with explicit imports: `use crate::game::constants::{WIZARD_POSITION, attacker_spawn_position};`. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `core/eye_transfer.rs` | 329 | No | Two near-identical parallel flight-launch blocks (invuln eye + ability eye) plus a separate "fix offsets" section. Proposed split: `eye_toss.rs` (`tick_eye_transfer` — launch logic) and `eye_flight.rs` (`update_eye_flight` — delivery logic). If H-08 helper extraction is done first, `tick_eye_transfer` will shrink considerably and the file may drop under 300. |
| `abilities/martina_mc.rs` | 261 | No | Contains general MC systems that belong in the wizard spell module (H-01). Once resolved this file shrinks to zero or is repurposed for Martina's aura (H-05). |
| `core/spawn.rs` | 233 | Yes | Single cohesive spawn function plus one eye-visual helper. All 233 lines directly assemble the hag entities and initialise resources. Exempt. |
| `core/combat.rs` | 212 | Yes | Exactly two systems (`update_hag_targeting` + `hag_combat`) that are tightly coupled by shared data shapes. Borderline but cohesive; exempt. |
| `abilities/martina_pull.rs` | 202 | Yes | Two Martina ability systems (`martina_teleport_pull` + `martina_mind_control` aura) plus type aliases. Below threshold; exempt pending H-05. |

---

### Looks bad but is actually fine

- **`pub use super::abilities::*` and `pub use super::core::*` in `systems.rs`** — looks like a wildcard export blob, but `systems.rs` is an explicitly documented "re-export hub for Phase 15 split." All public symbols resolve to specific named functions in their respective files. Intentional.
- **`hag_combat` two-pass loop** — first pass checks whether ANY target is in range (to decide whether to reset the cooldown), second pass finds the NEAREST target. The cooldown reset (including frenzy-speed computation) is deliberate between the two passes. Collapsing them would require restructuring cooldown logic.
- **`resurrect_eyed_hags` registered in `game/plugin.rs` not `HagsPlugin`** — intentional: it must run before the generic corpse-conversion system in the global `PostCombatSet` chain, which is easier to guarantee from the top-level plugin.
- **`martina_mind_control` has no ability-eye guard** — the aura is an always-on Martina passive (intentional design). Only teleport pull is eye-gated.
- **Double `hags.get(source)` calls in `tick_eye_transfer` (borrow-then-borrow pattern)** — correct Bevy borrow-split usage. First call borrows `&mut HagEyeState`, second borrows `&Transform`; separate calls are required because they target the same entity differently.
- **`justina_kite_distance` iterates all hags but filters identity inside loop** — HagIdentity is an enum value, not filterable at the query level. With ≤3 hags this is two skipped iterations per frame; negligible.
- **`hag_separation` O(n²)** — 3 hags × 3 hags = 9 iterations. Not a performance concern.
- **Stale `HagDeathTracker` / `EyeTransferTimer` after a non-hag game** — `spawn_hags()` unconditionally calls `commands.insert_resource(HagDeathTracker::new())` which overwrites stale state. When no hags are present, `any_with_component::<Hag>` gates silence all consumer systems. Not a live bug.

---

### Open questions

1. **H-02 real-world trigger:** Does `host_systems/status_receive.rs:303` insert `MindControlled` onto ghost entities (i.e. by network ID that maps to a GhostEntity on the host), or only onto real authoritative units? If ghosts never receive MC on the host, H-02 is theoretical risk rather than an active bug.
2. **`guard_entities` future intent (martina_pull.rs:86):** The KingsGuard collection is populated in the pre-pass but never used. Was the "guards teleport with the king" mechanic deliberately deferred, or dropped? The dead code should either be implemented or removed.
3. **Initial vs cycle cooldown offsets (H-06):** Are the startup-delay values in `::new()` intentionally different from the cycle constants to stagger hag abilities at fight start, or are they accidental drift? A code comment would prevent future confusion.
