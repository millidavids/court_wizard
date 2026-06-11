## spells-batch6

**Scope:** `src/game/units/wizard/spells/{haste,sleep,battle_hymn}/` + shared spell files (`lightning_bolt.rs`, `run_conditions.rs`, `spell_materials.rs`, `audio.rs`, `utils/`, `plugin.rs`)

---

### Mental Model

This batch covers two AoE buff spells (Haste, Battle Hymn), one AoE crowd-control spell (Sleep), the shared jagged lightning-bolt visual (`lightning_bolt.rs`), shared casting utilities (`utils/`), spell audio helpers (`audio.rs`), and the top-level spells `plugin.rs`. All three spells follow the same casting skeleton: `build_wizard_input` → `clamp_cursor_to_spell_range_with_origin` → `try_start_cast_with_indicator` → state-machine advance → mana consume → AoE apply → VFX/SFX. Sleep and Haste expose a `compute_talent_params` helper local to their `casting.rs`; Battle Hymn reads talent indices inline.

The shared utilities are cleanly modular (split into `casting_helpers.rs`, `spell_math.rs`, `reticle.rs`, etc.). Run conditions are thorough — all Update systems have `run_if` guards.

The most serious structural issue is that status-effect tick systems (`update_sleep_modifiers`, `handle_haste_expiry`, `tick_haste_slow_zone`, `update_night_terrors`, `update_narcoleptic_wave`, `update_battle_hymn_modifier`) all operate without `Without<GhostEntity>` guards. Per the project's documented ghost-gating pattern (MEMORY: project_mp_ghost_gating), these are gameplay/lifecycle systems that MUST exclude ghost entities. On the guest side, ghost units intentionally receive status components for local visual feedback (via `status_forwarding`), so these tick systems will fire expiry/damage/chain logic on ghost entities — diverging from host-authoritative state.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | Multiplayer | `sleep/systems/effects.rs:90` | High | S | `update_sleep_modifiers` queries all `SleepModifier` entities without `Without<GhostEntity>`. On the guest, ghost units intentionally receive `SleepModifier` for local visual feedback via `status_forwarding`. The tick will expire and remove `SleepModifier` (plus `NightTerrors`, `Comatose`, `NarcolepticWave`, `Sleepwalking`) on ghost units, ahead of host-authoritative timing, diverging guest visual state from host truth. | Add `Without<GhostEntity>` to the query filter (same fix needed on `update_night_terrors` line 107 and `update_narcoleptic_wave` line 132). |
| F02 | Multiplayer | `sleep/systems/wake.rs:8` | High | S | `update_sleepwalkers` overrides `TargetingVelocity` for all `Sleepwalking` entities without `Without<GhostEntity>`. On the guest, ghost units that receive `Sleepwalking` for visual feedback would have their targeting velocity overridden locally by the guest's `LocalSpellOrigin` rather than being driven by the host snapshot. | Add `Without<GhostEntity>` to the query. |
| F03 | Multiplayer | `haste/systems/effects.rs:15` | High | S | `handle_haste_expiry` (chain-haste propagation + momentum buff insertion) and `tick_haste_slow_zone` (applies `SlowMovementModifier`) lack `Without<GhostEntity>` guards. Chain-haste propagation inserts `HasteModifier`/`ChainHasteSource` on additional ghost entities — components not forwarded to the host — causing silent divergence. | Add `Without<GhostEntity>` to all queries in both systems. |
| F04 | Multiplayer | `battle_hymn/systems/effects.rs:6` | High | S | `update_battle_hymn_modifier` ticks all `BattleHymnModifier` entities without `Without<GhostEntity>`. On the guest, ghost units with `BattleHymnModifier` (forwarded to host on apply) will have it expired and removed locally ahead of host-authoritative timing. The `EchoingSong` re-apply path also fires on ghost entities. | Add `Without<GhostEntity>` to the query. |
| F05 | ArchitecturalDecay | `sleep/systems/casting.rs:193-208` | Medium | S | `sleep_casting_logic` manually re-implements the Pythagorean ground-projected range clamp (lines 193–208) that already exists as `clamp_cursor_to_spell_range_with_origin` in `utils/spell_math.rs`. Haste uses the shared helper correctly at `haste/systems/casting.rs:106`. The duplicated math (18 lines) will silently drift if the formula ever changes. | Replace lines 191–208 with a call to `clamp_cursor_to_spell_range_with_origin(input.cursor_pos, local_origin, wizard.spell_range, effective_radius)` (returns `Option<Vec3>`) and remove the manual `wizard_height` / `max_ground_radius` / `max_center_distance` locals. |
| F06 | ConsistencyRot | `battle_hymn/systems/casting.rs:81` | Low | S | Magic number `1.4` for Wide Anthem radius multiplier is inlined in a match arm. The analogous values in Haste (`ALACRITY_SPEED_MULT`) and Sleep (`LULLABY_RADIUS_MULT`) are named constants. A comment documents the intent, but the value is not searchable in `constants.rs`. | Add `pub const WIDE_ANTHEM_RADIUS_MULT: f32 = 1.4;` to `battle_hymn/constants.rs` and reference it. |
| F07 | ConsistencyRot | `battle_hymn/systems/casting.rs:174` | Low | S | Chorus of Valor VFX aura is spawned with hardcoded `200.0`. `constants::CIRCLE_RADIUS` is already `200.0` and is used on lines 97, 127, and 198 in the same file. The VFX radius would silently diverge from spell radius if `CIRCLE_RADIUS` were changed. | Replace `200.0` with `constants::CIRCLE_RADIUS * primed_spell.empowerment`. |
| F08 | ConsistencyRot | `battle_hymn/systems/aura.rs:41-124` | Low | S | Five gameplay tuning values are inlined: `1.5` (Inspiring Words duration), `1.5` (War Drums damage), `2.0` (Hymn of Legends double), `0.3` (Anthem Resilience reduction), `20.0` (Fortifying Hymn temp HP), `0.25` (Swift March speed). Compare `haste/constants.rs` and `sleep/constants.rs` which name every tunable. Makes balance reviews and changelog notes harder. | Move to `battle_hymn/constants.rs` as `INSPIRING_WORDS_DURATION_MULT`, `WAR_DRUMS_DAMAGE_MULT`, `HYMN_LEGENDS_BONUS_MULT`, `ANTHEM_RESILIENCE_REDUCTION`, `FORTIFYING_HYMN_TEMP_HP`, `SWIFT_MARCH_SPEED_BONUS`. |
| F09 | DocDrift | `CLAUDE.md:project instructions` | Low | S | The shared-utilities reference in project CLAUDE.md lists `commit_spell_cast` as an available helper ("use `build_wizard_input`, `cleanup_spell_caster`, `handle_spell_release`, `update_indicator_position`, `try_start_cast_with_indicator`, and `commit_spell_cast`"). No function by that name exists anywhere in `utils/casting_helpers.rs` or elsewhere in the codebase. | Remove `commit_spell_cast` from the CLAUDE.md utils list. It was presumably renamed or removed during a refactor. |
| F10 | ArchitecturalDecay | `audio.rs` | Low | M | `audio.rs` (432 LOC) mixes three separable concerns: `SpellSfxAssets` resource + `load_spell_sfx_assets` startup system (~120 LOC), `lookup_sfx_handle` / `sound_id_kind` dispatch (~45 LOC), and all playback helpers (`play_sfx`, `play_looping_sfx`, `play_sfx_synced`, `play_remote_sfx`, `play_looping_sfx_at`, `play_sfx_scaled`, etc. ~265 LOC). Above the 300-line guidance for non-cohesive files. | Split into `audio/assets.rs` (resource + loading), `audio/lookup.rs` (handle-map + kind dispatch), `audio/playback.rs` (all `play_*` helpers). Keep `audio.rs` as a re-export shim or rename to `audio/mod.rs`. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `audio.rs` | 432 | No | Three separable concerns: asset registry, handle lookup, playback helpers. Split into `audio/assets.rs`, `audio/lookup.rs`, `audio/playback.rs`. |
| `lightning_bolt.rs` | 394 | Yes | Single cohesive concern: jagged-bolt visual component, crackle update, afterimage, path generation, segment spawn helpers. Every line serves one feature. |
| `haste/systems/casting.rs` | 311 | Yes | 311 LOC; just over threshold. Cohesive: one casting system + `apply_haste_buff` helper that is tightly coupled to casting parameters. The helper is called only from casting. |

---

### Looks Bad But Is Actually Fine

- **`partial_cmp(...).unwrap_or(Ordering::Equal)` in `handle_haste_expiry` line 78.** Looks like a silent NaN swallow, but distances from `Vec3::distance` are NaN only if a position is NaN — which would break dozens of other systems before this one.
- **`sleep/systems/effects.rs:37` — `commands.entity(entity).insert(Health{current:0.0,...})`.** Looks like a raw health override bypassing the damage pipeline. The intent is to trigger the existing death/corpse-conversion systems downstream. A legitimate concern (missing `TemporaryHitPoints` / setup-immune bypass) would be a separate finding but is not raised here because it affects game-feel balance rather than correctness given current gameplay.
- **`compute_talent_params` present in both `haste/casting.rs` and `sleep/casting.rs`.** Not a DRY violation — each returns a different strongly-typed params struct (`HasteTalentParams` vs `SleepTalentParams`) with different fields. No useful common abstraction exists.
- **`battle_hymn/plugin.rs` lacks `mouse_held_or_wizard_casting`** compared to Haste/Sleep plugins. Intentional: Battle Hymn's indicator is spawned inside the match arm, so the guard is redundant; `spell_input_not_blocked` + `mouse_left_not_consumed` already cover the case.
- **`apply_battle_hymn_buff(..., radius=0.0)` for Chorus of Valor.** Looks like a bug but the `ignore_radius` flag bypasses the radius check, and line 181 has an explanatory comment.
- **`handle_haste_expiry` re-calls `compute_talent_params` every frame.** Acceptable: the system is gated with `any_with_component::<ChainHasteSource>.or(any_with_component::<MomentumPending>)` so it only runs when needed, and `ActiveTalents` is a tiny struct read.
- **`lightning_bolt.rs` spawns/despawns 20-30 child quads every frame per bolt.** Intentional crackling design. Flagged as a potential future optimization (F note in findings) but acceptable for current bolt counts.

---

### Open Questions

1. **Ghost-gating and visual correctness trade-off (F01–F04).** If `Without<GhostEntity>` guards are added to the tick systems, ghost visual state (sleep animation, haste speed tint, battle hymn glow) will freeze until the host snapshot overwrites it. Is frozen-visual-until-host-update acceptable, or should a separate visual-only marker component (e.g. `RemoteSleepEffect`) be introduced as the project memory suggests for the cleaner long-term pattern?

2. **Chain Haste talent not forwarded to host.** `ChainHasteSource`/`MomentumPending` are not in `status_forwarding.rs`. When the guest casts Haste with Chain Haste, chain propagation runs on ghost units locally but never reaches the host. Is this a known deferred item, or is chain haste intentionally guest-view-only?

3. **Narcoleptic Wave on host-side ghost enemies.** On the host, ghost enemies (guest's army) can receive `NarcolepticWave` via `status_receive`. `update_narcoleptic_wave` then spreads sleep to nearby real (non-ghost) attackers. Is that the desired behavior, or does it create double-application because the guest also spreads from its own non-ghost units?
