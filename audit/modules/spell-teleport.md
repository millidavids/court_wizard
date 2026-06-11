## spell-teleport

**Scope:** `src/game/units/wizard/spells/teleport/` — all `.rs` files (15 files, ~1770 LOC).

---

### Mental model

Teleport is the most complex spell in the codebase: a two-phase channelled cast (destination circle → source circle → execute), extended by nine talent variants (Wide Aperture, Hasty Translocation, Lingering Gate, Disorienting Arrival, Swap, Emergency Recall, Dimensional Rift, Teleport Up, Scatterport).

The module is split into four concern groups:

- **`casting/`** — local input state-machine (`cast_input.rs`) delegating to a pure logic function (`finalize.rs`).
- **`arrival/`** — execution engines for the five teleport variants (`teleport_logic.rs`), post-effect application (`cleanup.rs`), and persistent portal tick (`rift.rs`).
- **`vfx_components.rs` / `vfx_constants.rs` / `vfx_systems.rs`** — screen-space distortion ripple effects.
- **`components.rs` / `constants.rs`** — shared data and tuning.

`systems.rs` is a thin re-export hub used by `plugin.rs` to access systems through one namespace. Ghost-gating is implicitly safe: ghost entities spawned on the guest never receive `Teleportable`, so every units query in this module is naturally filtered. All Update systems carry `run_if` guards.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| T01 | ArchitecturalDecay | `casting/finalize.rs:132–205` and `:237–298` | High | M | The second-phase teleport completion logic is duplicated verbatim between the "early release" path and the "timer complete" path. Both paths call `execute_teleport`, populate the same result fields, and handle Lingering Gate — ~60 lines of near-identical code each. The only real difference is that early release uses `spawn_aura_bubble_contracting_synced` for the source bubble while timer-complete uses `spawn_aura_bubble_synced`. | Extract a `fn complete_second_phase(…, contracting: bool)` helper in `finalize.rs` that takes source/dest positions, radius, and the contraction flag, and call it from both sites. |
| T02 | TypeContract | `casting/finalize.rs:248` | High | S | The timer-complete Phase 2 path calls `mana.consume(effective_mana_cost)` at line 248 without re-checking `can_afford`. The guard at line 230 only runs when entering `CastingState::Resting`; if mana is drained between pressing and cast completion the game over-consumes into negative values. The early-release path (line 145) correctly re-checks `can_afford` before consuming. | Add `if !mana.can_afford(effective_mana_cost) { casting_state.cancel(); return result; }` before line 248, matching the early-release guard. |
| T03 | ArchitecturalDecay | `arrival/cleanup.rs:16,57,91` | Medium | S | `cleanup.rs` is misnamed: it contains `apply_post_teleport_effects` (post-cast talent application) and `update_circle_animations` (animation tick), neither of which are cleanup. Only `cleanup_teleport_on_spell_switch` belongs here. The wrong file name misleads navigation. | Rename `cleanup.rs` to `post_effects.rs` (housing `apply_post_teleport_effects` + `update_circle_animations`). Move `cleanup_teleport_on_spell_switch` to a short `cleanup.rs` or inline into `cast_input.rs`. |
| T04 | ConsistencyRot | `components.rs:60–64` and `:89–93` | Medium | S | `TeleportDestinationCircle::pulse_scale` and `TeleportSourceCircle::pulse_scale` are byte-for-byte identical (same formula, same inline magic numbers `pulse_freq = 2.0`, `pulse_amplitude = 0.05`). | Extract a free function `fn pulse_scale(time_alive: f32) -> f32` in `components.rs` and delegate both `impl` blocks to it. Promote the two magic numbers to named constants in `constants.rs`. |
| T05 | DocDrift | `constants.rs:42–46` | Medium | S | Constant docs say "stun enemies on arrival" (line 42) and "attack speed bonus for allies on arrival" (line 44), but `arrival/cleanup.rs:24–33` applies both `Stunned` and `DisorientingHaste` to **all** teleported entities regardless of team. | Clarify design intent: if all-entities is correct (consistent with friendly-fire philosophy), update the constant docs to say "all teleported units." If selective stun is the target, add team-based filtering in `apply_post_teleport_effects`. |
| T06 | ConsistencyRot | `arrival/rift.rs:25–30` | Low | S | `LingeringGateMarker` and `DimensionalRift` manually decrement `time_remaining` in their own systems, while `DisorientingHaste` and `RiftCooldown` (same module) correctly implement `TimedModifier` and use the shared `update_timed_modifier` generic. Two patterns for the same job in the same module. | Implement `TimedModifier` for `LingeringGateMarker` (the rift expiry side-effect — resetting `TeleportCaster` — can stay in `tick_lingering_gate` with a check). For `DimensionalRift` the custom logic is larger so the manual tick is acceptable, but consider a comment explaining why it diverges. |
| T07 | ArchitecturalDecay | `casting/finalize.rs:52` | Low | S | `teleport_casting_logic` accepts `_primed_spell: &PrimedSpell` (underscore-suppressed, never read). Empowerment is accessed from `source_circle.empowerment` instead. | Remove the dead parameter from the function signature. |
| T08 | DocDrift | `systems.rs:1` | Low | S | The module-level doc comment reads "Re-export hub for teleport systems split (Phase 14)". "Phase 14" is a stale internal refactoring ticket reference with no meaning to a future reader. The same comment pattern exists across many spell `systems.rs` files. | Replace with a plain description, e.g. `//! Re-export hub: arrival and casting system functions.` |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|-------------------------|
| `casting/cast_input.rs` | 413 | No | Contains three separate concerns: `compute_talent_params` (talent parsing), `handle_teleport_cancel`, and `handle_teleport_casting` (the main system, ~200 lines). Split into: `talents.rs` (compute_talent_params), `cancel.rs` (handle_teleport_cancel), `cast_input.rs` (handle_teleport_casting only). |
| `casting/finalize.rs` | 305 | Yes | Single function `teleport_casting_logic` + result struct. Genuinely cohesive state machine. After T01 extraction the file drops under 250 LOC. Exempt as-is. |
| `arrival/teleport_logic.rs` | 278 | Yes | Five teleport execution variants plus one helper. Each function is small and they form a logical match-on-variant group. Under 300 LOC; no split needed. |

---

### Looks bad but is actually fine

- **No explicit `Without<GhostEntity>` guards on teleport unit queries** — ghost entities on the guest never receive the `Teleportable` component (confirmed in `guest_snapshot/apply_state_snapshot.rs:607`). The `With<Teleportable>` filter implicitly excludes all ghost units. Safe and intentional.
- **`systems.rs` is only 4 lines** — this is the project's established re-export hub pattern for Phase 14 spell splits. Not a violation.
- **`tick_lingering_gate` implements its own tick loop** rather than using `update_timed_modifier` — on expiry, it must also reset `TeleportCaster` state on the wizard entity. The generic helper only removes the component; the custom system is justified for this side effect.
- **`handle_teleport_cancel` and `handle_teleport_casting` are two separate systems** — intentional. Right-click must always cancel even when the `run_if` conditions on the main casting system would block it. The plugin registers them with different guard sets on purpose.
- **`vfx_constants.rs` is imported by `crt_effect/distortion.rs` via full path** — the CRT shader needs the same wave-shape constants as the spawned VFX to keep visual parameters in sync. Not a layering violation.
- **`RIPPLE_STRENGTH`, `RIPPLE_FREQUENCY`, `RIPPLE_SPEED`, `RIFT_LENSING_RADIUS`, `RIPPLE_INFLUENCE_MULT` appear unreferenced within the teleport module** — all are used externally by `crt_effect/components/effect_settings.rs` and `crt_effect/distortion.rs`. Not dead code.
- **Large `handle_teleport_casting` parameter list** — `#[allow(clippy::too_many_arguments)]` is present; idiomatic per project convention for Bevy systems.

---

### Open questions

1. **Disorienting Arrival team filtering (T05):** Is the current all-entities behaviour (stun+haste applied to every teleported unit) intentional gameplay, or is the original "stun enemies / haste allies" design still the target? The friendly-fire philosophy makes all-entities plausible, but the mismatched constants doc is a trap.
2. **Early-release vs timer-complete VFX difference (T01):** Early release uses `spawn_aura_bubble_contracting_synced` for the source bubble, but timer-complete uses the expanding variant. Is this a deliberate "snap vs bloom" feel difference, or an accidental divergence?
3. **Multiplayer replication of Dimensional Rift and DisorientingHaste:** `apply_post_teleport_effects` is only called from `handle_teleport_casting` (gated to `LocalWizard`). On the guest, rift entities and haste components are never spawned when the host fires Teleport. Is there a network path that replicates these effects to the guest, or is the guest expected to not see them?
