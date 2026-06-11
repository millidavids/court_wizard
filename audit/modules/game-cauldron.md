## game-cauldron

**Scope:** `src/game/cauldron/` — brewing system, buff application, multiplayer replication.

---

### Mental Model

The cauldron module is a self-contained brewing pipeline. Players pick ingredients into a `Recipe`, which is queued via `StartBrewMessage`. `CauldronState` on the cauldron entity ticks the brew timer; on completion a `BrewCompleteMessage` triggers `CauldronBuffs` (a `Vec<ActiveBuff>`) to be updated and a visual bubble spawned. Each frame a set of host-authoritative systems reads the aggregated scalars from `CauldronBuffs` and inserts/removes per-unit buff components (`CauldronDamageBonus`, `CauldronDamageResistance`, `CauldronSpeedModifier`) on army entities, while writing scalar fields directly (`Effectiveness.cauldron_spell_bonus`, `Health.heal`, `TemporaryHitPoints`).

Multiplayer adds a second peer path: the guest Alchemist's `CauldronBuffs` snapshots into `CauldronArmyScalars`, sent via `NetworkMessage::CauldronBuffsSync`, received into `RemoteCauldronBuffs` on the host, then applied to the guest's army by `apply_guest_army_buffs`. The separation between local brew loop (`is_spell_effects_active`, both peers) and host-authoritative army buffs (`is_gameplay_running`, host only) is sound, and all Update systems are gated appropriately.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| C1 | ArchitecturalDecay | `systems.rs:892` | High | L | `systems.rs` is 892 lines and mixes fundamentally different concerns: spawn/animation, brew lifecycle, army buff application, and MP replication. Per project convention files exceeding ~300 LOC must be split unless every line is cohesive. | Split into `spawn.rs` (load_cauldron_assets, spawn_cauldron ~70 LOC), `brew_lifecycle.rs` (handle_start_brew, update_brew_timer, handle_brew_complete, handle_cancel_brew, block_spells_during_brewing ~145 LOC), `visuals.rs` (update_brew_bubble, update_cauldron_animation, start_brewing_effects, update_brewing_effects, update_brewing_timer ~130 LOC), `army_buffs.rs` (heal_defenders, buff_defender_damage, buff_defender_resistance, apply_cauldron_speed_modifiers, shield_defenders, apply_max_mana_buff, buff_defender_effectiveness, cleanup_cauldron_buff_components ~290 LOC), `multiplayer.rs` (send_cauldron_buffs_to_host, receive_cauldron_buffs, reset_remote_cauldron_buffs, apply_guest_army_buffs ~190 LOC). |
| C2 | ConsistencyRot | `systems.rs:623,812` | Medium | S | `MAX_SHIELD: f32 = 20.0` is declared as a local `const` in two separate functions: `shield_defenders` (line 623) and `apply_guest_army_buffs` (line 812). Similarly, the "keep alive" duration `5.0` appears four times (lines 631, 635, 849, 854). | Promote both to named constants in `constants.rs`: `pub const SHIELD_MAX_HP: f32 = 20.0;` and `pub const SHIELD_KEEPALIVE_SECS: f32 = 5.0;` and reference them from both systems. |
| C3 | ArchitecturalDecay | `constants.rs:28-32` | Low | S | `CAULDRON_COOP_POSITION` is byte-for-byte identical to `CAULDRON_POSITION` (both compute `WIZARD_POSITION + CAULDRON_OFFSET` with the same signs). The constant exists as documentation but creates a maintenance hazard — if the SP cauldron moves, the co-op constant should also move, but the link is not enforced. | Replace the co-op constant with `pub const CAULDRON_COOP_POSITION: Vec3 = CAULDRON_POSITION;` to make the aliasing explicit and automatically correct when the SP position changes. |
| C4 | ArchitecturalDecay | `components.rs:30-31` | Low | S | `CauldronState::Cooldown { remaining }` is marked `#[allow(dead_code)]` and is never constructed anywhere in the codebase (only matched in `start_brewing_effects` at `systems.rs:344` and `components.rs:87`). The `Cooldown` timer logic in `tick()` is unreachable in practice. | Remove the `Cooldown` variant and its `tick()` branch, or document it as a planned future feature with a `// TODO:` comment explaining the design intent. |
| C5 | ArchitecturalDecay | `components.rs:46,99` | Low | S | `CauldronState::active_recipe()` and `CauldronState::progress()` are marked `#[allow(dead_code)]`. `progress()` IS called externally from `src/ui/in_game/bars.rs:129`, so it should have its annotation removed. `active_recipe()` appears to be genuinely unused. | Remove `#[allow(dead_code)]` from `progress()`. Investigate whether `active_recipe()` can be deleted. |
| C6 | ConsistencyRot | `systems.rs:450-489` | Medium | M | `buff_defender_damage` and `buff_defender_resistance` are structurally identical: get scalar → iterate units → insert-if-positive-and-absent / remove-if-zero-and-present. The same pattern is repeated a third time inside `apply_guest_army_buffs` (lines 872-890). At 3+ sites this meets the shared-helper threshold. | Extract a generic helper (in a future `army_buffs.rs`): `fn sync_buff_component<C: Component + Clone>(commands, value: f32, existing: Has<C>, entity: Entity, constructor: impl Fn(f32) -> C)`. The three call sites collapse to single lines. |
| C7 | Performance | `systems.rs:742-768` | Medium | M | `receive_cauldron_buffs` drains `connection.incoming_messages` into a temporary `Vec`, processes only `CauldronBuffsSync` messages, and puts unhandled messages back. This is the same drain-and-re-extend pattern repeated across many multiplayer systems and causes two allocations per frame when there are unhandled messages. It also means later systems in the frame see an empty `incoming_messages` until `extend` runs. | This is a codebase-wide architectural concern (not just cauldron), but for cauldron specifically: since this is gated on `is_remote_alchemist`, the performance hit is minor. The real risk is message ordering — if `receive_cauldron_buffs` runs before other MP systems, it consumes messages those systems need. Audit the scheduling order of all `incoming_messages.drain` callers. |
| C8 | Performance | `systems.rs:124-144` | Medium | S | `handle_brew_complete` calls `load_unified_save()` (disk I/O) inside a message-handler system, inside the game loop. Although gated on `on_message::<BrewCompleteMessage>` (fires at most once per brew), the sync disk read can stall the frame for tens of milliseconds on some platforms. | This pattern (disk I/O on a rare event) is also present in achievements, drops, and talents — it's consistent project behaviour. For cauldron specifically, consider dispatching combo unlock via the existing `ComboDiscoveredMessage` path, writing the save file in a dedicated end-of-game or achievement system that already has disk I/O. |
| C9 | TypeContract | `brews/constants.rs:155` | Low | S | `PHILOSOPHERS_STONE_CONFIG` has `effect: BrewEffect::BuffDurationMultiplier(1.0)`, which is a no-op (multiplier of exactly 1.0). The `is_noop()` method confirms this returns `true`. The stone's actual function (removing dilution) is implemented entirely in `Recipe::dilution_factor()`, not through `BrewEffect`. This creates a misleading contract: a recipe containing the Stone appears to have a real `BuffDurationMultiplier` effect when its effects are enumerated. | Give `PhilosophersStone` a dedicated `BrewEffect::RemovesDilution` variant (or document explicitly in the config that the effect field is intentionally a no-op sentinel) to prevent future confusion when iterating effects. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Split Proposal |
|------|-----|--------|--------------------------|
| `src/game/cauldron/systems.rs` | 892 | false | Not a single large match or registry — contains 5 distinct concern groups. Split into: `spawn.rs`, `brew_lifecycle.rs`, `visuals.rs`, `army_buffs.rs`, `multiplayer.rs`. |

---

### Looks Bad But Is Actually Fine

- **`Ingredient::PhilosophersStone` absent from `Ingredient::all()`** — intentional; the stone is a special unlock not shown in the regular ingredient grid.
- **`apply_max_mana_buff` has no per-system `run_if` guard** — intentional and documented in `plugin.rs:66-67`; it must run every frame to self-reset the guest's mana cap when a max-mana buff expires, because the host-only cleanup path never runs on the guest.
- **`buff_defender_effectiveness` has no per-system `run_if` guard** — same intentional design: it writes `cauldron_spell_bonus = 0.0` when no buff is active, serving as its own cleanup. Internal change-guard (`abs() > f32::EPSILON`) prevents spam writes.
- **`receive_cauldron_buffs` runs under `is_remote_alchemist` only** — correctly limits the drain to when the guest is an Alchemist; no drain in single-player.
- **No `Without<GhostEntity>` on army buff queries** — ghost entities only exist on the guest's side, and all army-buff systems that mutate unit components are gated to `is_gameplay_running` which returns `true` only for the host. No cross-contamination.
- **`update_cauldron_animation` runs without a fine-grained run_if (no `cauldron_is_brewing` guard)** — correct; the cauldron should animate (idle bubbling) even when not brewing.
- **`BREWING_PULSE_SCALE_MAX` and `BREWING_COLOR_ALPHA_MAX` exported but not used directly** — only used to compute `_RANGE` constants at compile time; technically could be private, but exporting them for possible UI inspection is harmless.

---

### Open Questions

1. **Co-op dual-Alchemist double-brew**: The comment at `systems.rs:826-830` acknowledges that when both wizards brew `DefenderDamageBonus`, only the first writer wins (insert-if-none). Is this a known limitation or a tracked deferred item?
2. **`CauldronState::Cooldown` future plans**: The variant exists with dead_code but the tick logic is implemented. Is a cooldown between brews planned? If not, should it be removed before the state machine grows more match arms?
3. **Multiplayer message drain ordering**: `receive_cauldron_buffs` drains and re-extends `incoming_messages`. If a future multiplayer system scheduled earlier in the same frame also drains, cauldron messages could be swallowed. Is there a defined ordering contract for `incoming_messages` consumers?
