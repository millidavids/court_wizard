## spell-teleport

**Scope:** `src/game/units/wizard/spells/teleport/` (10 files, 1730 LOC total)

---

### Mental model

The Teleport spell is a two-phase cast: Phase 1 places a destination marker (crosshair), Phase 2 grows a source circle that captures all `Teleportable` units and sends them to the destination. A rich talent tree adds nine variants: Wide Aperture, Hasty Translocation, Lingering Gate (Tier 1), Disorienting Arrival, Swap, Emergency Recall (Tier 2), Dimensional Rift, Gravitational Surge (Up), and Scatterport (Tier 3).

The module is well-factored into concern files (`casting.rs`, `arrival.rs`, `vfx_systems.rs`, `vfx_components.rs`, `vfx_constants.rs`, `components.rs`, `constants.rs`). The `systems.rs` is purely a re-export hub. Ghost-entity safety is achieved implicitly: ghost units never receive the `Teleportable` component, so all gameplay queries (`With<Teleportable>`) naturally exclude them. All Update systems carry `run_if` guards.

The main tech-debt centres are: `arrival.rs` hosts three functions (`update_circle_animations`, `cleanup_teleport_on_spell_switch`, `random_position_in_circle`) that are not about *arrival* at all, making it a mixed-concern file; `casting.rs` has two near-identical teleport-completion branches that diverge in one VFX call; and two `pulse_scale` methods are copy-pasted between `TeleportDestinationCircle` and `TeleportSourceCircle`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| T-01 | ArchitecturalDecay | `arrival.rs:389–475` | High | M | `arrival.rs` owns three functions unrelated to teleport arrival: `update_circle_animations` (animation), `cleanup_teleport_on_spell_switch` (cleanup), and `random_position_in_circle` (geometry helper). The 475-line file violates "group by concern." | Move `update_circle_animations` and `cleanup_teleport_on_spell_switch` to a new `indicators.rs`; move `random_position_in_circle` to a `geometry.rs` or inline it into `arrival.rs` at the call sites. |
| T-02 | ArchitecturalDecay | `casting.rs:530–603` vs `casting.rs:635–695` | Medium | S | Two teleport-completion branches (early-release and timer-complete) duplicate ~40 lines each: `execute_teleport`, result field assignment, and Lingering Gate bookkeeping. They differ in only two ways: which radius expression is used and whether the source gets a contracting vs non-contracting aura bubble. | Extract a `fn finalize_teleport(...)` helper that takes `source_pos`, `dest_pos`, `radius`, `contracting: bool` and fills the `TeleportCastResult`. The 4-parameter difference is small enough to unify. |
| T-03 | ConsistencyRot | `components.rs:60–64` and `components.rs:89–93` | Medium | S | `TeleportDestinationCircle::pulse_scale` and `TeleportSourceCircle::pulse_scale` are byte-for-byte identical (same constants, same formula). | Extract `fn pulse_scale_for(time_alive: f32) -> f32` as a module-private helper or a shared free function, and call it from both `impl` blocks. |
| T-04 | DocDrift | `components.rs:108` | Medium | S | `TeleportTalentParams.disorienting_arrival` is documented as "stun enemies, haste allies on arrival." The implementation in `arrival.rs:279–288` inserts `Stunned` on ALL teleported entities — allies and enemies alike — with no team filter. The `constants.rs:42` doc also says "stun enemies." | Either fix the doc to say "stun all teleported units (friendly-fire applies)" to match the actual behaviour, or add a team filter to the `Stunned` insertion if selective stun was intended. Given the project's friendly-fire design, fixing the doc is likely the right call. |
| T-05 | ArchitecturalDecay | `casting.rs:454` | Low | S | `teleport_casting_logic` accepts `_primed_spell: &PrimedSpell` (prefixed with underscore indicating unused) as its 5th argument. The outer system passes `primed_spell` but the inner function never reads it — `empowerment` is accessed only in the outer wrapper. | Remove the dead parameter from `teleport_casting_logic`. The caller already has `primed_spell` in scope; it does not need forwarding. |
| T-06 | ConsistencyRot | `casting.rs:649–668` | Low | S | The timer-complete branch spawns `spawn_aura_bubble_synced` for the **source** position (non-contracting), while the early-release branch (line 548) spawns `spawn_aura_bubble_contracting_synced` for the source. This means the visual effect differs depending on whether the player releases early or waits for the timer — an unintentional inconsistency or an undocumented design choice. | If intentional, add a comment explaining why; if not, make both paths use `spawn_aura_bubble_contracting_synced` for the source to match the "circle shrinks away" metaphor. |
| T-07 | ConsistencyRot | `arrival.rs:312` | Low | S | `tick_dimensional_rift` is a full Bevy system defined in `arrival.rs`, but it has no arrival-phase responsibility — it ticks an ongoing portal. The plugin (correctly) guards it with `any_with_component::<DimensionalRift>` but it still lives in the wrong file. | Move to a new `rift.rs` or to `systems.rs` as a re-exported concern. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|------------------------|
| `casting.rs` | 703 | No | Split into: `casting.rs` (phase-1/2 state machine + outer system, ~400 LOC), `finalize.rs` (extraction of the two completion branches + helper, ~150 LOC) |
| `arrival.rs` | 475 | No | Split into: `arrival.rs` (execute_teleport + per-mode helpers + apply_post_teleport_effects + tick_dimensional_rift, ~350 LOC), `indicators.rs` (update_circle_animations + cleanup_teleport_on_spell_switch + random_position_in_circle, ~125 LOC) |

---

### Looks bad but is actually fine

- **`systems.rs` is only 4 lines** — a pure `pub use` re-export hub referencing `arrival::*` and `casting::*`. This matches the project convention for re-export hubs. Not a violation.
- **Ghost-entity safety has no explicit `Without<GhostEntity>` guards** on teleport queries. This is safe because ghost units spawned via `guest_snapshot.rs` never receive `Teleportable`. The `With<Teleportable>` filter on all gameplay queries implicitly excludes ghosts. The pattern is consistent with multiplayer spawning code.
- **`tick_lingering_gate` implements its own tick loop** instead of using `update_timed_modifier<LingeringGateMarker>`. This is justified because expiry must also reset `TeleportCaster` state on the wizard entity — the generic `update_timed_modifier` only removes the component. The custom system is the right tool here.
- **`handle_teleport_cancel` and `handle_teleport_casting` are split into two separate systems** despite sharing caster-state logic. This is intentional: right-click must always cancel even when other `run_if` conditions on the main casting system would block it. The plugin registers them independently with different guard sets.
- **`vfx_constants.rs` is accessed from `crt_effect/distortion.rs` and `crt_effect/components.rs` directly** via full module path. This cross-module coupling is necessary because the CRT distortion shader needs the same wave-shape constants that drive the spawned VFX, ensuring visual consistency. It is not a layering violation.
- **`TeleportCastResult` is a crate-private struct** only used within `casting.rs`. It does not need to be a public type.

---

### Open questions

1. Was Disorienting Arrival's stun intentionally applied to all teleported units (including allies), or should it stun only enemies and haste only allies as the doc suggests? The "friendly fire is fundamental" project philosophy makes the current behaviour plausible, but the mismatched doc string is a trap for future maintainers.
2. The early-release path uses a contracting source-bubble VFX while the timer-complete path uses a non-contracting one. Is this a deliberate feel difference ("snap" vs "bloom") or an accidental divergence?
3. `casting.rs` is 703 LOC. The project's 300-LOC guideline allows exemptions for "single large match-on-enum." The two-phase state machine here is close but not a single match — it contains multiple conditional branches. Is a `finalize.rs` extraction in scope for the next refactor pass?
