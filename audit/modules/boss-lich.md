## boss-lich

**Scope:** `src/game/units/boss/lich/` (8 files, ~1 240 LOC)

---

### Mental model

The Lich is a three-phase mid-game boss that spawns after all normal waves are cleared. Phase 1 (Approaching): it navigates from the tunnel to the staging point using hand-rolled steering physics — the normal staging flow field targets only Team::Attackers, so the Lich drives its own. Phase 1b (Summoning): it stands still and periodically raises undead corpses or spawns fresh undead infantry, accumulating Soul Power for each undead kill by the player's army. Phase 2 (Combat): Soul Power full, it wades into the defenders and fires Finger of Death beams (recycling the wizard's FoD beam system) with king-immunity logic. State is encoded as the `LichPhase` enum component; active casts are represented by a `LichCasting` component that lets the sprite swap to the casting sheet for the wind-up duration. The plugin is clean registration-only. The split into `spawn.rs` and `combat.rs` via a `systems.rs` re-export hub is mostly sound, but `combat.rs` has grown to 564 LOC mixing movement, soul-power bookkeeping, targeting, combat, and visual/animation concerns.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| L-01 | DocDrift | `combat.rs:32-35` | Medium | S | `track_soul_power` carries a copy-pasted doc comment ("Checks if it's time to spawn the Lich mid-game…") that belongs to `check_lich_spawn` in `spawn.rs`. The function actually tracks soul-power accumulation. A spurious `#[allow(clippy::too_many_arguments)]` is also present on a 2-parameter function. | Replace doc comment with an accurate description of soul-power accumulation; remove the `#[allow]`. |
| L-02 | ArchitecturalDecay | `combat.rs:1-564` | Medium | M | At 564 LOC, `combat.rs` is well above the 300-line project limit and bundles at least four distinct concerns: soul-power/phase transition, movement, combat targeting + FoD dispatch, and visual animation (facing, float, material swap). The project rule states files >300 LOC must be split unless they are a single match-on-enum or asset registry — this is neither. | Split into `soul_power.rs` (track + phase transition), `movement.rs` (lich_movement + update_lich_targeting), `combat.rs` (lich_combat_targeting + lich_fire_beam + tick_lich_casting + resolve_finger_of_death), and `animation.rs` (update_lich_facing + update_lich_float + on_lich_cast_started + on_lich_cast_ended). Update `systems.rs` re-export hub accordingly. |
| L-03 | ArchitecturalDecay | `combat.rs:146-190` | Low | S | `lich_movement` contains two nearly identical steering-physics blocks (Approaching and Combat branches). Both compute `max_speed`, apply `STEERING_FORCE`-clamped acceleration, set `velocity.max_speed`, and apply `VELOCITY_DAMPING`. The only difference is the target direction vector. | Extract a private `apply_steering(velocity, acceleration, target_dir, max_speed, delta)` helper and call it from both branches. |
| L-04 | Performance | `combat.rs:94-97` | Low | S | `update_lich_targeting` unconditionally collects `all_units` into a `Vec<_>` heap allocation every frame, even when the Lich is in Approaching or Summoning phase (where the snapshot is immediately discarded). | Add an early-return guard: if none of the lich entities is in `LichPhase::Combat`, skip the snapshot entirely. |
| L-05 | Performance | `combat.rs:224-240` | Low | S | `lich_combat_targeting` builds `non_king: Vec<Entity>` and `eligible: Vec<Entity>` every frame even when `fod.target.is_some()` or `!fod.is_ready()` (the dominant path during the cooldown window). The defender collection loop runs for all defenders regardless. | Check `fod.target.is_some() || !fod.is_ready()` before building the eligible list; `continue` early to avoid per-frame allocation during cooldown. |
| L-06 | TypeContract | `combat.rs:409-413` | Medium | S | `resolve_finger_of_death` inserts `PendingUndeadRaise` as a singleton `Resource`. The wizard's FoD spell in `casting.rs` also inserts this same resource (when the `finger_of_undeath` talent is active). If both fire in the same frame, the second `insert_resource` silently overwrites the first, losing one set of kill positions. | Change `PendingUndeadRaise.kill_positions` to accumulate across writers, or consume the resource before inserting, or switch to a `Message` (broadcast event) so both sources can enqueue independently and `process_pending_undead_raises` processes all of them. |
| L-07 | ConsistencyRot | `mod.rs:5,7` | Low | S | `resources` and `systems` are both declared `pub(in crate::game)`. No consumer outside `src/game/units/boss/lich/` imports from `lich::resources` or `lich::systems`. (`LichAssets` is only used internally; the `systems` module is a re-export hub consumed only via `plugin.rs`.) | Tighten both to `pub(super)` to match their actual visibility scope and prevent accidental coupling. |
| L-08 | TypeContract | `components.rs:115` | Low | S | `LichFingerOfDeath::new()` hard-codes the initial cooldown as the literal `1.0` with only an inline comment. All other timing values in this module are named constants in `constants.rs`. | Add `const LICH_FOD_INITIAL_COOLDOWN: f32 = 1.0;` to `constants.rs` and reference it from `::new()`. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|--------------------------|
| `combat.rs` | 564 | false | Four distinct concerns; split into `soul_power.rs`, `movement.rs`, `combat.rs`, `animation.rs` |
| `spawn.rs` | 294 | true | Just under limit; coherent spawn + summoning concern |

---

### Looks bad but is actually fine

- **`resolve_finger_of_death` as a private helper called from `tick_lich_casting`** — passing `&mut MessageWriter` as a plain `&mut` ref is unusual but valid; the system owns the `MessageWriter` and passes a mutable reference down, avoiding a second system parameter extraction.
- **`lich_combat_targeting` uses `kill_stats.elapsed_time * 1000.0` for pseudo-random target selection** — this looks like it could be jittery, but the selection only fires once when `fod.target` transitions from `None` to `Some`, so frame-to-frame variance doesn't matter; the chosen target persists until the beam fires and `fod.reset()` clears it.
- **`LichBeamTargetData` / `LichBeamTargetFilter` type aliases at the top of `combat.rs`** — these look like over-engineering but they are used in two separate places (`resolve_finger_of_death` signature and `tick_lich_casting` query declaration), making the alias genuinely DRY.
- **Absence of `Without<GhostEntity>` in lich targeting queries** — the lich plugin is gated behind `is_gameplay_running` which requires `PeerRole::Host` in multiplayer. `GhostEntity` units exist only on the guest side; the host never spawns them, so there is no risk of the lich targeting phantom ghost units.
- **`spawn_fresh_undead` in `spawn.rs` spawns units without a health bar or kill-stat contribution marker** — intentional; summoned undead are cheap cannon fodder and their kills intentionally feed the Lich's soul power via `KillStats.undead_killed` which is tracked globally.
- **`systems.rs` "Phase 15" comment** — a historical note from the refactor that produced the split. Harmless, though it could be removed.

---

### Open questions

1. **Lich FoD + wizard FoD same-frame collision (L-06)**: Is there a systemic ordering guarantee that prevents both from inserting `PendingUndeadRaise` in the same frame? If the wizard's `finger_of_undeath` talent and the Lich's FoD resolve in the same frame, which system runs last and silently wins?
2. **`lich_combat_targeting` excludes `With<Boss>` but the filter is on the `defenders` query, not the king query** — the king query does not exclude `Boss`. Is `King` ever tagged `Boss`? If so, the king could appear in both the `king_query` and the general `defenders` query simultaneously.
3. **Phase 15 systems split comment** — is the `systems.rs` re-export hub a permanent architectural choice or a transitional artifact? If the latter, collapsing it and importing directly from `combat` / `spawn` in `plugin.rs` would be cleaner.
