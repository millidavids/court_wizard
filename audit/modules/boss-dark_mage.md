## boss-dark_mage

**Scope:** `src/game/units/boss/dark_mage/`

---

### Mental model

The Dark Mage is a tier-3 boss with a three-spell kit (Dark Meteor, Shadow Lightning, Plague Cloud) governed by a state-machine component (`DarkMageState`). On each frame the module ticks spell cooldowns and enqueues ready spells (`dark_mage_spell_queue`), then the AI pops the queue, transitions through Idle → Telegraphing → Casting (`dark_mage_ai`), and fires the appropriate spawn helper. A separate teleport system repositions the boss when enemies enter melee range. The module is cleanly feature-sliced: `ai/` holds movement, spell-queue logic, spawn, and teleport; `spells/` holds spawn helpers, per-spell update systems, VFX, and targeting. All Update systems carry `run_if(is_gameplay_running).run_if(any_with_component::<DarkMage>)` guards. No files exceed 300 LOC.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| DM-01 | ConsistencyRot | `ai/ai_core.rs:48-49`, `ai/teleport.rs:44-45` | Low | S | `Stunned` and `Petrified` are referenced inline as `crate::game::units::components::Stunned` / `::Petrified` inside query tuple types rather than being added to the existing `use crate::game::units::components::{...}` import block. Every other status-effect component in the same file is imported at the top. | Add `Stunned, Petrified` to the grouped `use crate::game::units::components::{...}` import. |
| DM-02 | ConsistencyRot | `spells/spell_spawn.rs:39,123,187,240` | Low | S | Spawn-helper functions (`spawn_telegraph_indicators`, `spawn_meteor_explosion`, `spawn_lightning_strike`, `spawn_plague_cloud`) are declared `pub(crate)` but are only consumed within the `dark_mage` module tree and re-exported as `pub(super)` by `spells/mod.rs`. The function-level visibility is broader than necessary. | Change the four spawn-helper declarations from `pub(crate)` to `pub(super)`. The re-exports in `spells/mod.rs` already enforce the intended visibility boundary. |
| DM-03 | ArchitecturalDecay | `systems.rs:1` | Low | S | The stale comment `//! Re-export hub for dark_mage systems split (Phase 15).` references an internal refactoring phase number that is meaningless to future readers. | Remove the `(Phase 15)` label. Rename to `//! Re-exports all dark_mage gameplay systems.` |
| DM-04 | ArchitecturalDecay | `spells/vfx_updates.rs:73,80,86,88` and `spells/spell_spawn.rs:180,270,276` | Low | M | VFX sizing and timing values are scattered inline as magic literals: plague particle interval (`0.4` s), puff Y range (`15.0–45.0`), puff scale range (`30.0–55.0`), puff lifetime range (`2.0–3.5`), meteor fall time (`0.3` s), plague initial puff Y range (`20.0–60.0`), initial puff scale (`40.0–70.0`). No matching constants exist in `constants.rs`, making them invisible to tuning passes. | Add named constants (`PLAGUE_PARTICLE_SPAWN_INTERVAL`, `PLAGUE_PUFF_Y_MIN/MAX`, `PLAGUE_PUFF_SCALE_MIN/MAX`, `PLAGUE_PUFF_LIFETIME_MIN/MAX`, `METEOR_FALL_DURATION`) to `constants.rs` and replace the inline literals. |
| DM-05 | ArchitecturalDecay | `spells/spell_spawn.rs:220` | Low | S | `let bolt_count = 3` is a local magic number with no corresponding constant. The bolt-spacing formula depends on it. | Extract to `pub(super) const LIGHTNING_BOLT_COUNT: usize = 3;` in `constants.rs`. |
| DM-06 | TypeContract | `components.rs:62-68`, `components.rs:139-145` | Low | S | `DarkMageSpellQueue` and `DarkMageEnrage` both provide `new()` constructors for trivially-constructible zero-state values but do not implement `Default`. Bevy convention is to use `Default` for no-arg constructors and `new()` for constructors with arguments. | Add `impl Default for DarkMageSpellQueue` and `impl Default for DarkMageEnrage` (delegating to `new()`), or replace `new()` with `#[derive(Default)]`. |

---

### Oversized files

No files in scope exceed 300 LOC. The two largest approach the threshold but are cohesive.

| File | LOC | Exempt | Reason |
|------|-----|--------|--------|
| `spells/spell_updates.rs` | 289 | true | Three spell-update systems with related query types; splitting would create three trivial single-function files. |
| `spells/spell_spawn.rs` | 283 | true | Three spawn helpers and one broadcast system sharing the same indicator infrastructure; cohesive. |

---

### Looks bad but is actually fine

- **`super::super::` import paths** throughout `ai/` and `spells/` — standard Rust for sub-module reaching two levels up; `crate::game::units::boss::dark_mage::*` would be noisier.
- **`VecDeque::contains()` in `dark_mage_spell_queue` (movement.rs:107)** — O(n) but the queue has at most 3 elements.
- **`find_spell_target` allocates a `Vec` (targeting.rs:54)** — called only on the single frame when `DarkMageState::Idle` pops a spell (an infrequent state transition), not every frame.
- **No `GhostEntity`/`GhostSpellEffect` gating** — `GhostEntity` is only added to opponent-army snapshots (infantry/archer/king) in co-op mode. The Dark Mage and its spell effect entities are never ghost entities. No gating needed.
- **Three `.insert()` calls with ~30 components on spawn (spawn.rs:36–92)** — idiomatic Bevy for a complex boss entity; works around tuple size limits in component bundles.
- **`dark_mage_enrage` has no additional run_if** — it is in its own `add_systems` block with `run_if(is_gameplay_running).run_if(any_with_component::<DarkMage>)` at plugin.rs:40–43.
- **`preload_dark_mage_assets` on `Startup` without `run_if`** — `Startup` runs exactly once at app start; a `run_if` guard would be meaningless, and unconditional asset preload is intentional.
- **`pub(in crate::game) mod constants`** — constants are consumed by `loading/init/world_setup.rs` (outside `dark_mage` but inside `game`), so the wider visibility is justified.

---

### Open questions

1. **Plague hazard removal on premature despawn:** if the boss is killed while a plague cloud is active, is `ObstacleChanged::Removed` still guaranteed to fire (i.e., does cleanup always run `update_plague_clouds` to `lifetime <= 0.0`)? If `try_despawn` is called externally first, the hazard cost leaks into the flow field permanently for the rest of the fight.
2. **Silent teleport failure:** when all 30 random candidates are rejected, `chosen_dest` is `None` and the teleport is skipped silently with no log message and no timer reset. Should the timer be reset to a short retry interval rather than waiting the full `TELEPORT_COOLDOWN`?
3. **CC cancels telegraph but not the cooldown:** when CC interrupts a `Telegraphing` state, the cooldown was already reset at `ai_core.rs:133`. The cancelled spell must wait the full cooldown before re-queuing. Is this the intended balance behaviour?
