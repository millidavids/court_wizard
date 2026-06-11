## boss-ogre

**Scope:** `src/game/units/boss/ogre/` — all `.rs` files (16 files, ~1 327 LOC total across charge/, combat/, and top-level files).

---

### Mental model

The ogre is a melee boss with three interlocking abilities: a charge attack (telegraph → dash → recovery state machine), a rock throw (borrowed from the brute via `RockThrowCooldown`), and a progressive three-phase enrage that scales speed and damage as HP drops. The module is well-sliced into `combat/` (spawn, movement, melee, facing, enrage) and `charge/` (charge state machine, visuals, rock throw). The top-level `systems.rs` is a pure re-export hub. Component and constant files are appropriately sized. The largest single file is `charge_attack.rs` (370 LOC), which is a single large state-machine match — exempt under the convention rule.

Overall health is **good**: no `.unwrap()`, no `println!`, granular file layout, run_if guards on all Update systems. Three concrete issues stand out: a silent ghost-entity gap in multiplayer, a MeleeRangeBonus that is registered on the entity but silently ignored by the ogre's own melee system, and a cross-cutting audio path leak.

---

### Findings table

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| OG-01 | TypeContract | `combat/melee.rs:120-122` | High | S | `ogre_combat` first-pass range check computes `attack_range = (boss_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER` without adding `OGRE_MELEE_RANGE_BONUS`. The ogre has `MeleeRangeBonus(OGRE_MELEE_RANGE_BONUS)` on its entity (spawn.rs:104) and the global melee system in `combat_systems/melee/combat.rs:229` does add the bonus — but `ogre_combat` bypasses the global system and silently drops the bonus. The ogre effectively has a shorter first-hit trigger range than intended. | Read `melee_range_bonus: Option<&MeleeRangeBonus>` in the `bosses` query and add `melee_range_bonus.map_or(0.0, |b| b.0)` to the range calculation at line 121. |
| OG-02 | ArchitecturalDecay | `combat/melee.rs:1` | Medium | S | `use crate::game::units::animation::CombatAnimation;` bypasses the canonical re-export `crate::game::units::components::CombatAnimation` (which exists via `pub use super::animation::*` in `components/mod.rs`). All other ogre files import from `components`. | Change to `use crate::game::units::components::CombatAnimation;` for consistency. |
| OG-03 | ArchitecturalDecay | `charge/charge_attack.rs:265`, `charge/rock_throw.rs:113`, `combat/melee.rs:136` | Medium | S | All three ogre action sites call `crate::game::units::wizard::spells::audio::play_sfx_scaled` via the full absolute wizard-spell path. `play_sfx_scaled` is a shared audio utility used by bosses, cauldron, etc. The coupling to the wizard/spells path is an architectural smell. | Move `play_sfx_scaled` to a shared audio module (e.g. `src/game/audio.rs`) and re-export from there. In the meantime, at minimum add `use crate::game::units::wizard::spells::audio::play_sfx_scaled;` at the import block of each file instead of inline fully-qualified calls. |
| OG-04 | ConsistencyRot | `charge/rock_throw.rs:11` | Low | S | `RockThrowCooldown` is imported from `crate::game::units::brute::components`. The ogre shares the brute's component to avoid duplication, which is intentional, but a future refactor of `brute::components` could inadvertently break the ogre with no documentation of the dependency. | Move `RockThrowCooldown` to `crate::game::units::components` (the cross-cutting module) so both brute and ogre import from the canonical location. |
| OG-05 | ArchitecturalDecay | `charge/charge_visuals.rs:206-220` | Low | S | `facing_from_world_direction` is a 4-way camera-relative direction helper defined with `pub(crate)` in the ogre module. The XZ cam dot-product logic is conceptually identical to `back_facing_for_velocity` in `boss/utils.rs`. | Move `facing_from_world_direction` to `boss/utils.rs` alongside the other camera-relative facing helpers. |
| OG-06 | Performance | `combat/melee.rs:109-126` | Low | S | The first-pass loop (has_target check) and the second-pass loop (damage application, lines 152-186) both iterate the full `targets` query, doubling iteration. The ogre is a singleton so the real cost is low, but the pattern is easy to consolidate. | Combine into a single pass that collects hit candidates and applies damage only if the range check passed. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `charge/charge_attack.rs` | 370 | Yes | Single large `match charge_state.as_mut()` across all five enum arms — exactly the exempt "single large match-on-enum" case. |

All other files are well under 300 LOC.

---

### Looks bad but is actually fine

- **`systems.rs` is 4 lines of re-exports** — looks like an empty stub but is correct: it is a pure re-export hub for a split module, intentional per project convention.
- **`OgreChargeVisuals.elapsed` reset to `0.0` in `charge_visuals.rs:131`** — looks like a state mutation bug (zeroing elapsed mid-charge) but is intentional: it serves as a one-shot "restore position" flag for vibration offset.
- **`RockThrowCooldown` hard-coded to `8.0` in spawn.rs vs. `ROCK_THROW_COOLDOWN = 15.0` in boulder constants** — looks inconsistent. The ogre intentionally uses a shorter cooldown than the brute; the spawn comment just doesn't explain it.
- **`camera_query.single().ok().unwrap_or(Vec3::NEG_Z)` in `charge_visuals.rs:34-41`** — looks like a `.ok()` swallowing an error, but the fallback is a safe default; the camera is always present during gameplay.
- **`update_ogre_targeting` queries `Without<Lich>`** — looks like a leaky filter but is intentional: both ogre and lich are `Boss` entities and the ogre's targeting must not fire on the lich.
- **`OgreChargeState::Targeting` fallback uses hardcoded `cooldown: 2.0`** — looks like a magic number but is a single-use retry wait, well within the "inline is fine" threshold.
- **No `GhostEntity` filter on gameplay systems** — investigated; bosses are SP-only content and never spawn in the multiplayer wave set. The `is_gameplay_running` guard is sufficient. Not a bug.

---

### Open questions

1. Should `facing_from_world_direction` be promoted to `boss/utils.rs` to serve lich/dark_mage 4-way facing if those bosses ever need it?
2. Is the ogre intended to have the full `OGRE_MELEE_RANGE_BONUS` effect on its first-pass trigger range, or should the bonus only affect the second-pass damage sweep? (OG-01 assumes the former since the component is inserted.)
3. `RockThrowCooldown` is owned by `brute::components` — is there a roadmap to move it to the units cross-cutting module?
