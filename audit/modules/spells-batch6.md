## spells-batch6

**Scope:** `spells/haste/`, `spells/sleep/`, `spells/battle_hymn/`, `spells/lightning_bolt.rs`, `spells/run_conditions.rs`, `spells/spell_materials.rs`, `spells/audio.rs`, `spells/utils.rs`, `spells/plugin.rs`

---

### Mental Model

This batch covers three AoE buff/debuff spells (Haste, Sleep, BattleHymn), the shared jagged-lightning-bolt visual (`lightning_bolt.rs`), and the spine of the entire spell system: `utils.rs`, `audio.rs`, `run_conditions.rs`, `spell_materials.rs`, and the top-level `plugin.rs`.

`utils.rs` (~804 LOC) is the most important file: it provides the `LocalSpellOrigin` resource (with a lock-free atomic snapshot for non-system callers), shared casting helpers (`build_wizard_input`, `try_start_cast_with_indicator`, `cleanup_spell_caster`, `handle_spell_release`, `update_indicator_position`), the `SpellCircleIndicator` / `AnimatedRingParticle` shared components, range-clamping math, and the global-cooldown tick systems. The helpers exist precisely to prevent per-spell boilerplate.

`audio.rs` (~432 LOC) manages a monolithic `SpellSfxAssets` handle bag, distance-attenuation helpers, and the Excremage override system. It is a single-concern asset registry plus companion helpers and is effectively exempt from the 300-LOC split rule.

`lightning_bolt.rs` (~394 LOC) is a self-contained crackling visual: it despawns/re-spawns child quad segments every frame to produce a jittery animated bolt. Used by chain_lightning, lightning_rod, and the hag boss.

The three spell modules (Haste/Sleep/BattleHymn) each follow the project's granular pattern well. All Update systems carry `run_if` guards; ghost-entity exposure is not a risk for these buff spells because the guest has no `GhostEntity` units with `HasteModifier`/`SleepModifier`/`BattleHymnModifier`.

The main decay items are: an inline range-clamp duplication in `sleep/systems.rs` that ignores the shared `clamp_cursor_to_spell_range_with_origin` utility; a duplicated six-power attenuation formula inside `audio.rs`; a cluster of magic literal talent constants hard-coded inside `battle_hymn/systems.rs` that belong in `constants.rs`; and a contract fragility in the Eternal Slumber kill path that bypasses `apply_damage_to_unit`'s setup-immunity check.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B6-01 | ConsistencyRot | `sleep/systems.rs:192–209` | Medium | S | `sleep_casting_logic` hand-rolls the same spell-range clamp (Pythagorean ground-radius, center-distance clamping) that `clamp_cursor_to_spell_range_with_origin` in `utils.rs` already provides. The same pattern also appears in `fog_cloud` and `grease` (out of scope), making three redundant copies total. | Replace the 18-line inline block with `let cursor_world_pos = crate::game::units::wizard::spells::utils::clamp_cursor_to_spell_range_with_origin(input.cursor_pos, local_origin, wizard.spell_range, effective_radius).unwrap_or(return false);` — identical to the pattern Haste uses correctly at `haste/systems.rs:106`. |
| B6-02 | TypeContract | `sleep/systems.rs:320–326` | High | S | The Eternal Slumber instant-kill branch inserts a raw `Health { current: 0.0, … }` component directly, bypassing `apply_damage_to_unit`. This skips the `is_setup_immune()` immunity window (active during MP spawn phase) and the `TemporaryHitPoints` absorption path. A unit could be killed during the setup immune window, or a unit with TempHP absorbing a hit could be one-shot by this path. | Call `apply_damage_to_unit(&mut health_mut, temp_hp.as_deref_mut(), health.max * 2.0)` (overkill damage) instead of directly inserting a new `Health` component. Requires adding `Option<&mut TemporaryHitPoints>` to the `targets` query. |
| B6-03 | ConsistencyRot | `audio.rs:193–198` and `audio.rs:415–419` | Low | S | The six-power distance attenuation formula (`linear^6`) is written out identically in `play_sfx_scaled` (line 197) and `play_looping_sfx_at` (line 418). | Extract into a private `fn compute_attenuation(effect_pos: Vec3) -> f32` that calls `audio_origin()` internally. Both sites call the helper. |
| B6-04 | ConsistencyRot | `battle_hymn/systems.rs:311–394` | Medium | S | Nine talent balance literals are inlined as bare floats directly in `apply_battle_hymn_buff`: `1.5` (Inspiring Words duration mult), `1.5` (War Drums damage mult), `2.0` (Hymn of Legends double), `0.3` (Anthem Resilience reduction), `20.0` (Fortifying Hymn HP), `0.25` (Swift March speed), `0.5` (echo duration fraction), `2.0` (Chorus of Valor mana cost factor), `200.0` (Chorus VFX radius). Compare `haste/constants.rs` and `sleep/constants.rs` which extract every tunable. | Add named constants to `battle_hymn/constants.rs` (e.g. `INSPIRING_WORDS_DURATION_MULT`, `WAR_DRUMS_DAMAGE_MULT`, `FORTIFYING_HYMN_TEMP_HP`, `SWIFT_MARCH_SPEED_BONUS`, `ANTHEM_RESILIENCE_REDUCTION`, `CHORUS_MANA_COST_MULT`, `CHORUS_VFX_RADIUS`). |
| B6-05 | ArchitecturalDecay | `utils.rs:804 LOC` | Medium | M | `utils.rs` is 804 lines and mixes five distinct concerns: (1) `LocalSpellOrigin` resource + lock-free atomic snapshot (~90 LOC), (2) geometric math helpers (`xz_distance`, `sphere_intersects_cylinder`, `distance_to_line_segment_xz`, range-clamp variants ~120 LOC), (3) heal helpers (`PendingDefenderHeal`, `apply_spell_heal`, `apply_pending_defender_heal` ~80 LOC), (4) shared casting boilerplate (`SpellCircleIndicator`, `build_wizard_input`, `cleanup_spell_caster`, etc. ~350 LOC), (5) ring-particle VFX shared between entangle/spike_growth (~100 LOC), (6) global cooldown systems (~80 LOC). No single group exceeds 300 LOC, but the file lacks cohesion. | Split into: `spell_origin.rs` (LocalSpellOrigin + atomic snapshot), `spell_math.rs` (geometry helpers + range clamp), `spell_heal.rs` (PendingDefenderHeal + apply_spell_heal), `casting_helpers.rs` (SpellCircleIndicator + all casting boilerplate + cooldowns), `ring_vfx.rs` (AnimatedRingParticle + spawner + animator). Keep `utils.rs` as a re-export hub. Effort is M because the split is mechanical (no logic change) but touches many downstream `use` imports. |
| B6-06 | ArchitecturalDecay | `battle_hymn/systems.rs:112–145` | Low | S | The indicator/SpellCaster management in `handle_battle_hymn_casting` is split across the outer `match *casting_state` block (lines 112–145) and `battle_hymn_casting_logic` (lines 239–277). The `CastingState::Channeling` arm in the outer match calls `cleanup_spell_caster` but does not call `casting_state.cancel()`; that happens inside the inner function. This split makes the invariant that "cleanup and cancel always happen together" non-obvious and harder to audit. | Fold the indicator/SpellCaster management into `battle_hymn_casting_logic` or use `try_start_cast_with_indicator` so the casting lifecycle is expressed in one place (matching the Haste pattern). |
| B6-07 | Performance | `utils.rs:542–578` | Low | S | `compute_target_assist` runs every frame during `is_spell_effects_active` and iterates all entities with `Health` — potentially 200+ units at high wave counts — to find the nearest one to the cursor. When targeting assistance is disabled (`snap_radius <= 0.0`) it exits early for free, but when enabled it is an unbounded O(N) scan. The `TargetAssistWorldPos` resource is only useful while the mouse is held and a spell is primed. | Add an additional `run_if` guard such as `wizard_is_casting_or_channeling.or(mouse_held_or_wizard_casting)` in `plugin.rs` so the scan is skipped when no spell input is active. |
| B6-08 | Performance | `lightning_bolt.rs:198,166` | Low | L | `update_lightning_bolts` calls `commands.entity(entity).despawn_related::<Children>()` followed by re-spawning 20–30 child quads every frame for each active bolt. For 10 simultaneous chain-lightning bolts this is 200–300 entity spawn/despawn operations per frame. This is the intentional crackle design but scales poorly if spells are spammed. | Consider a mesh-update approach: pre-allocate a fixed set of `N` child quads per bolt and update their `Transform`s each frame instead of destroying and recreating them. Effort is L because it requires reworking `spawn_segments` and `spawn_forks`. Flag as a future optimization, not blocking. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `utils.rs` | 804 | No | Multi-concern shared utilities — split into `spell_origin.rs`, `spell_math.rs`, `spell_heal.rs`, `casting_helpers.rs`, `ring_vfx.rs` |
| `audio.rs` | 432 | Yes | Single-concern asset registry + companion audio helpers (load, attenuate, play, Excremage override, MP sync). Every line is cohesive. |
| `sleep/systems.rs` | 518 | No | Three independent concerns: (1) casting + indicator (lines 1–290), (2) sleep modifier tick + sub-effect tick systems (291–412), (3) movement override for sleepwalkers (413–519). Split into `casting.rs`, `effects.rs`, `sleepwalking.rs`. |
| `haste/systems.rs` | 441 | No | Two independent concerns: (1) casting + buff application (1–311), (2) expiry-driven talent behavior (312–441). Split into `casting.rs` and `expiry.rs`. |
| `battle_hymn/systems.rs` | 440 | No | Two concerns: (1) casting + SpellCaster/indicator lifecycle (1–233), (2) buff application + tick (234–441). Split into `casting.rs` and `buff.rs`. |
| `lightning_bolt.rs` | 394 | Yes | Single cohesive concern: the jagged-bolt visual component, update system, and geometry helpers. All 394 lines serve one feature. |

---

### Looks Bad But Is Actually Fine

- **`sleep/systems.rs:320` — `commands.entity(entity).insert(Health{…})`** looks like it bypasses the death system, but the comment explains the intent: setting `current: 0.0` triggers the existing death/corpse-conversion systems that run downstream. The real concern (B6-02) is the bypass of `is_setup_immune` and `TemporaryHitPoints`, not the death routing itself.

- **`utils.rs:65–67` — `AtomicU32` storing float bits for `LocalSpellOrigin`** looks like premature micro-optimization, but the comment explains a genuine `Once`-based race condition this replaces. The Relaxed ordering is correct here because the write (`update_local_spell_origin_snapshot`) always runs before any read that depends on the guest value, and there is no cross-thread synchronization contract beyond "eventually consistent within a frame."

- **`audio.rs` — `SpellSfxAssets` struct with ~40 fields** looks like a god struct, but it is a typed handle bag (purely data, no logic). Adding a new SFX just requires adding one field and one `asset_server.load(...)` line — the pattern is correct.

- **`haste/systems.rs:379` — `.partial_cmp(…).unwrap_or(Equal)`** looks like a silent NaN swallow, but distances are computed via `Vec3::distance` which returns NaN only if a position is NaN — a pre-existing data corruption that would manifest in dozens of other systems first.

- **`battle_hymn/plugin.rs:21` — `handle_battle_hymn_casting` missing `mouse_held_or_wizard_casting`** compared to Haste/Sleep plugins, looks inconsistent. It is intentional: Battle Hymn's indicator is spawned inside the casting match arm (frame it becomes primed), so the system only needs to run while the mouse is held; the `spell_input_not_blocked` + `mouse_left_not_consumed` guards are sufficient.

- **`battle_hymn/systems.rs:180–181` — `apply_battle_hymn_buff(..., 0.0)` passing radius 0.0** for Chorus of Valor looks like a bug, but the `ignore_radius` flag explicitly bypasses the radius check, and the comment at line 181 documents this.

- **`utils.rs:541–542` — `#[allow(clippy::too_many_arguments)]` on `compute_target_assist`** looks like a suppression, but the system legitimately needs `ResMut`, `Res<GameConfig>`, camera query, cursor resource, and unit query — all required Bevy system params per CLAUDE.md conventions.

---

### Open Questions

1. `sleep/systems.rs:317` — The Eternal Slumber threshold (`health.current <= health.max * 0.25`) is evaluated against the raw health value, not accounting for `TemporaryHitPoints`. Should a unit with 30% real HP + TempHP that brings total above 25% trigger Eternal Slumber? Intended behavior needs clarification to determine if B6-02's fix should also adjust the threshold check.

2. `compute_target_assist` (utils.rs:542) iterates all `With<Health>` entities regardless of team. Should target assist snap to enemies only (hostile targets) or all units? Currently it could snap to a friendly unit the player is trying to avoid hitting with e.g. Sleep. If it should only snap to enemy units, the query needs a team filter.

3. `utils.rs` is 804 LOC. Is the split proposed in B6-05 worth doing as a standalone refactor task, or should it be deferred until a new utility is added that makes the file obviously too large?

4. The `haste/systems.rs:341` (`handle_haste_expiry`) re-calls `compute_talent_params` every frame when chain haste or momentum components are present. Should talent params be cached on the wizard entity once selected, rather than re-derived from `ActiveTalents` each frame across multiple systems?
