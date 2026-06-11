## spells-batch5

**Scope:** `src/game/units/wizard/spells/fog_cloud/`, `mark_of_death/`, `banishment/`, `guardian_circle/`

---

## Mental model

Four self-contained area/targeting spells that each follow the project's casting boilerplate pattern:

- **fog_cloud** — places a lingering zone that grants evasion; six talent markers (BlindingMist, ConcealingVeil, DisorientingVapors, PhantomFog, ChokingFog, RollingFog) each add a component and a matching system gated by `any_exist`. Zone entity is replicated to the guest via `NetworkedSpellEffect`/`SpellEffectKind::FogCloudZone`. Ghost zone on guest carries the same talent markers with stub values (dps=0, speed=0).
- **mark_of_death** — single-target or AoE (Mass Marking) debuff that amplifies damage taken. Death triggers talent effects (Spreading Blight, Swift Hex mana refund, Death's Ledger AoE explosion, Doom escalating amp). Uses `Without<GhostEntity>` consistently on every gameplay system; guest only runs visual indicator systems.
- **banishment** — temporarily removes a unit from the battlefield (hidden + `BanishedModifier`). Talent flags are stored as optional components (`PainfulReturn`, `Displacement`, `DimensionalShunt`, `OneWayTrip`) on the banished entity and processed on return. `tick_banished_units` runs under `is_gameplay_running` (host-only). **Does not call `local_player_team()`**.
- **guardian_circle** — grants temporary HP to all units in radius; talent reaction systems fire after `PostCombatSet` under `is_gameplay_running`. Uses full 3D `Vec3::distance()` for range checks rather than the `xz_distance` used by the other three spells.

---

## Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B5-01 | TypeContract | `banishment/systems/cast_logic.rs:106,185` | High | S | `Team::Defenders.is_enemy(team)` is hardcoded as the caster's perspective. In **versus-guest** mode the local wizard is `Team::Attackers`, so their enemies are `Team::Defenders`. The current filter treats Defenders as non-enemies from the Defenders POV, meaning the guest finds *zero* valid targets and can never land a banishment. `local_player_team(session)` is the correct call as used by `mark_of_death`, `disintegrate`, and every other session-aware spell. | Inject `Option<Res<MultiplayerSession>>` into `handle_banishment_casting`, call `local_player_team(session.as_deref())`, pass the result into `cast_single_banishment` and `cast_mass_banishment`, and replace the two hardcoded calls. |
| B5-02 | Multiplayer | `fog_cloud/systems/phantoms.rs:10-70` | High | S | `spawn_phantom_units` iterates all `PhantomFogZone` entities without excluding ghost zones. Ghost `FogCloudZone` entities on the **guest** are spawned with `PhantomFogZone { spawn_timer: 0.0 }` (see `ghost_effect_spawn.rs:180`). The plugin runs under `is_spell_effects_active` (guest-inclusive), so after ~3 s the guest's ghost zone triggers a second wave of local phantom units — real `Team::Defenders` entities that exist only on the guest machine, are not networked, and pollute all-unit queries. | Add `Without<crate::game::multiplayer::components::GhostSpellEffect>` to the `zones` query in `spawn_phantom_units`, matching the "host-only" intent stated in the ghost spawn comment. |
| B5-03 | Multiplayer | `fog_cloud/systems/effects.rs:15,44` | Medium | S | `apply_fog_cloud_evasion` and `apply_blinding_mist` both iterate all non-corpse units without excluding `GhostEntity`. Ghost units on the guest have `Health` and `Team` but their state is managed by the host-side CRDT. Inserting `FogEvasionModifier` or `BlindingMistDebuff` onto ghosts is harmless for combat (runs host-only) but the `update_timed_modifier::<FogEvasionModifier>` system in `units/plugin.rs` runs on all units on the guest — creating per-frame ECS churn as it ticks and removes ghost evasion modifiers. | Add `Without<crate::game::multiplayer::components::GhostEntity>` to the `targets` queries in both systems. |
| B5-04 | ConsistencyRot | `guardian_circle/systems/buff.rs:95,156` | Medium | S | `apply_guardian_circle_buff` and `deal_aoe_force_damage` both use `transform.translation.distance(origin)` (3D Euclidean). The other three spells in this batch and most of the codebase use `xz_distance` which ignores the Y axis — correct for a top-down battlefield where units stand at varied heights. The 3D distance makes Guardian Circle's effective radius slightly smaller for taller units (behemoths, king) and can cause buff misses for units visually inside the circle. | Replace `.distance()` calls on lines 95 and 156 with `crate::game::units::wizard::spells::utils::xz_distance(transform.translation, origin)`. Same fix in `talent_reactions.rs:120`. |
| B5-05 | ErrorObservability | `mark_of_death/systems/deaths_ledger.rs:27-29` | Low | S | `materials.get(&visual_assets.necrotic_pulse).cloned().unwrap_or_default()` silently falls back to a white `StandardMaterial` if the asset handle is invalid. The explosion still spawns but renders incorrectly with no log entry. `SpellVisualAssets` is always loaded before gameplay so this should never fail in practice — but the silent fallback hides future asset-pipeline regressions. | Replace with `let Some(pulse_material) = materials.get(&visual_assets.necrotic_pulse).cloned() else { warn!("necrotic_pulse material not loaded"); return; };` |
| B5-06 | ConsistencyRot | `mark_of_death/systems/talent_effects.rs:115-116` | Low | S | `handle_marked_corpses` uses 3D `distance_squared` for Spreading Blight nearest-enemy search, while `focal_point_retarget` in the same file uses manual XZ-only distance (line 193). Inconsistency within the same file. For correctness the spreading blight nearest-enemy search should also ignore Y. | Replace `distance_squared` on lines 115-116 with an XZ-only comparison: `(a.1.translation.x - t.x).powi(2) + (a.1.translation.z - t.z).powi(2)`. |
| B5-07 | ArchitecturalDecay | `guardian_circle/components.rs:8-43` | Low | M | `GuardianCircleShielded` is a monolithic struct bundling nine fields for four distinct talent effects (Retaliating Wards, Fortified Resolve, Sanctuary, Martyrdom, Chain Ward). The project convention prefers small focused components queryable with `With<T>`. All four talent-reaction systems run an `if field > 0` check to determine applicability rather than a clean component presence query. | Consider splitting into `RetaliatingWardsShield`, `FortifiedResolveShield`, `SanctuaryShield`, `MartyrdomShield`, `ChainWardShield` components. Not a blocker, but worth tracking. |

---

## Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|--------------------------|
| `fog_cloud/systems/casting.rs` | 253 | true | Cohesive: single casting function + inner `fog_cloud_casting_logic` + `compute_talent_params`. All three serve the same cast flow. |
| `mark_of_death/systems/casting.rs` | 266 | true | Cohesive: single casting function wrapping `mark_of_death_casting_logic`. |
| `banishment/systems/casting.rs` | 245 | true | Cohesive: single casting function + `banishment_casting_logic` + `compute_talent_params`. |
| `guardian_circle/systems/casting.rs` | 241 | true | Cohesive: single casting function + inner `guardian_circle_casting_logic`. |
| `banishment/systems/cast_logic.rs` | 215 | true | Cohesive: two closely related cast helpers (`cast_single_banishment`, `cast_mass_banishment`) plus shared `banish_target` and `is_in_spell_range`. |
| `mark_of_death/systems/talent_effects.rs` | 195 | true | Cohesive: four talent-effect systems sharing the same query types and death-cleanup loop. |
| `guardian_circle/systems/buff.rs` | 168 | true | Cohesive: `apply_guardian_circle_buff` + `deal_aoe_force_damage` + `cleanup_guardian_circle_shielded`. |

No files exceed 300 LOC.

---

## Looks bad but is actually fine

- **`fog_cloud/systems/casting.rs:168-183` — manual sphere-circle intersection for cursor clamping.** Bespoke but correct: wizard hovers above ground level, so a true 3D sphere radius must be projected onto the XZ plane. Well commented.
- **`banishment/systems/tick.rs` uses `is_gameplay_running` while VFX uses `is_spell_effects_active`.** Intentional: damage/expiry is host-authoritative; VFX should play on both peers.
- **`mark_of_death/systems/casting.rs:127-131` — manual `if input.just_released` instead of `handle_spell_release`.** `handle_spell_release` also cleans up `SpellCaster`/indicator entities, which Mark of Death does not use. The manual cancel is correct.
- **`fog_cloud/systems/effects.rs:83` — `is_setup_immune()` in `apply_choking_fog_damage`.** Consistent with how `health.rs:183,268` uses the same global atomic. Not a one-off hack.
- **`spawn_phantom_units` uses non-random seeded noise from `time.elapsed_secs()`.** The pseudo-random positioning is deterministic and based on time, which is fine for cosmetic scatter of decoy units.
- **`deaths_ledger.rs` spawns explosion with `damage_applied: false` and a separate system applies damage.** This is the correct two-system pattern for deferred AoE (spawn → damage next frame avoids mutable-query conflicts).
- **`phantom_units` spawned with `Stunned { time_remaining: f32::MAX }`.** Intentional: prevents phantoms from ever attacking, matching their role as harmless decoys.
- **`guardian_circle/plugin.rs` docblock listing all registered systems.** Accurate and helpful; no rule against per-plugin documentation.

---

## Open questions

1. **Versus-mode coverage for fog_cloud:** Fog Cloud does not call `local_player_team()`. `apply_fog_cloud_evasion` and `apply_blinding_mist` grant evasion/blindness to *all* non-corpse units in range, including the opposing team. Is this intentional friendly-fire-style gameplay in versus mode, or should evasion only apply to the caster's own army?
2. **Ghost zone self-ticking:** `apply_fog_cloud_evasion` increments `zone.time_since_last_tick` and `zone.time_alive` on every zone including ghost zones. Ghost zone lifetime is thus controlled by the guest's own clock rather than the host snapshot. If the clocks diverge, the ghost zone could expire at a different time than the host's. Is independent self-ticking for ghost zones intentional?
3. **PhantomFogZone on ghost zones:** The comment in `ghost_effect_spawn.rs:169` says phantom spawning is "host-only," but `PhantomFogZone` is still inserted on guest ghost zones (line 180). Was this intentional (to keep `any_exist::<PhantomFogZone>()` true on the guest for some other reason), or is the flag insertion a mistake that B5-02 covers?

### Mental model

Four area-control/debuff spells that share the wizard casting pipeline (mouse input → cast-time → zone/modifier effect). Each module follows the same five-file layout: `mod.rs`, `plugin.rs`, `components.rs`, `constants.rs`, `systems.rs`. Plugin files are registration-only. `mod.rs` files are declaration/re-export only. All `Update` systems carry `run_if` guards.

The main structural problem is that every `systems.rs` exceeds 540 LOC — each is a mixed bag of casting logic, talent effects, VFX helpers, and cleanup. The project convention mandates splitting at ~300 LOC per concern. Beyond size, two multiplayer correctness issues were found: `spawn_phantom_units` runs on the VS guest because `PhantomFogZone` is mirrored and the system has no ghost-gate, and `banishment` hardcodes `Team::Defenders.is_enemy()` instead of calling `local_player_team()`, making the VS guest unable to banish their enemies.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ArchitecturalDecay | `fog_cloud/systems.rs:1` | High | M | `systems.rs` is 584 LOC mixing casting logic, evasion application, talent effects (blinding mist, choking fog, rolling fog, phantom spawning), zone cleanup, and the public `is_in_fog_zone` helper. Violates the ~300-LOC feature-slice rule. | Split into `casting.rs` (handle_fog_cloud_casting + fog_cloud_casting_logic + spawn_fog_cloud_zone + compute_talent_params), `effects.rs` (apply_fog_cloud_evasion + apply_blinding_mist + tick_blinding_mist_debuff + apply_choking_fog_damage + move_rolling_fog + emit_fog_cloud_particles + cleanup_fog_cloud_zone), `phantoms.rs` (spawn_phantom_units + cleanup_phantom_units), and `utils.rs` (is_in_fog_zone). |
| F2 | ArchitecturalDecay | `mark_of_death/systems.rs:1` | High | M | 635 LOC mixing casting logic, talent ticking (doom, executioner), death handling (spreading blight, swift hex, death's ledger), visual indicator management, and AoE explosion update. | Split into `casting.rs` (handle_mark_of_death_casting + mark_of_death_casting_logic + compute mark flags), `talent_effects.rs` (tick_doom_marks + executioner_brand_check + handle_marked_corpses), `deaths_ledger.rs` (spawn_deaths_ledger_explosion + apply_deaths_ledger_damage + update_deaths_ledger_bursts), `indicators.rs` (spawn_mark_indicators + update_mark_indicators + focal_point_retarget). |
| F3 | ArchitecturalDecay | `banishment/systems.rs:1` | High | M | 613 LOC mixing talent param computation, the `banish_target` helper, VFX spawn, casting logic, single-target and mass-target sub-functions, VFX animation update, and banishment tick. | Split into `casting.rs` (handle_banishment_casting + banishment_casting_logic + cast_single_banishment + cast_mass_banishment + compute_talent_params + is_in_spell_range), `vfx.rs` (banish_target, spawn_banishment_vfx, update_banishment_vfx), `tick.rs` (tick_banished_units). |
| F4 | ArchitecturalDecay | `guardian_circle/systems.rs:1` | High | M | 540 LOC mixing casting, buff application, AoE damage helper, and three distinct talent-reaction systems (retaliating wards, martyrdom, chain ward). | Split into `casting.rs` (handle_guardian_circle_casting + guardian_circle_casting_logic), `buff.rs` (apply_guardian_circle_buff + cleanup_guardian_circle_shielded), `talent_reactions.rs` (retaliating_wards_check + martyrdom_on_death + chain_ward_on_death + deal_aoe_force_damage). |
| F5 | Security | `fog_cloud/systems.rs:434` | High | S | `spawn_phantom_units` runs on both host and VS guest because `PhantomFogZone` is mirrored to the guest (confirmed in `guest_visuals.rs:267`) and the system has no `Without<GhostSpellEffect>` gate or `is_gameplay_running` (host-only) guard. The guest spawns local phantom units that the host never knows about — unsynced gameplay entities violating the ghost-gating rule. | Add `.run_if(is_gameplay_running)` to `spawn_phantom_units` in `plugin.rs` so it only executes on the authoritative host/SP peer. |
| F6 | TypeContract | `banishment/systems.rs:390` | High | S | `cast_single_banishment` and `cast_mass_banishment` filter enemies with hardcoded `Team::Defenders.is_enemy(team)`. In VS multiplayer the guest wizard is `Team::Attackers`, so their enemies are `Team::Defenders`. `Team::Defenders.is_enemy(&Team::Defenders)` returns `false`, making banishment a no-op for the VS guest. Compare to `mark_of_death` which correctly calls `local_player_team(session.as_deref())`. | Thread a `MultiplayerSession` resource through `handle_banishment_casting` and `banishment_casting_logic`, compute `caster_team = local_player_team(session.as_deref())`, and replace the hardcoded `Team::Defenders.is_enemy(team)` with `caster_team.is_enemy(team)`. |
| F7 | Performance | `mark_of_death/systems.rs:585` | Low | S | `spawn_mark_indicators` allocates a `HashSet<Entity>` every frame while marks exist (any_exist::<ActiveMarkOfDeath>). In typical play this is a small set, but it is an unconditional heap allocation on an every-frame hot path. | Replace the HashSet lookup with a second query: `Query<(), With<MarkVisualIndicator>>` combined with `existing_indicators.contains(entity)` using a local entity set, or query `MarkVisualIndicator` directly on the marked entity. |
| F8 | Performance | `fog_cloud/systems.rs:366` | Low | S | `move_rolling_fog` collects all attacker positions into a `Vec<Vec3>` every frame while `RollingFogZone` exists. | Rewrite using an in-loop min-distance scan without allocation (iterate `units` once inside the zone loop). |
| F9 | TypeContract | `mark_of_death/systems.rs:430` | Low | S | `spawn_deaths_ledger_explosion` uses `materials.get(&visual_assets.necrotic_pulse).cloned().unwrap_or_default()` to clone the base material. If the handle is invalid (e.g., hot-reload or race), the explosion silently uses a default (black) material with no warning. | Log a `warn!` when the material is not found rather than silently using a default. |
| F10 | ConsistencyRot | `guardian_circle/systems.rs:328` | Low | S | `apply_guardian_circle_buff` and `deal_aoe_force_damage` use `Vec3::distance()` (3D) for radius checks, while every other spell in scope uses `xz_distance()` (2D XZ plane). For units on flat terrain this is negligible, but it's an inconsistency that could matter if a unit's Y position is elevated. | Replace `transform.translation.distance(pos)` with `xz_distance(transform.translation, pos)` for consistency with the rest of the spell suite. |
| F11 | DocDrift | `guardian_circle/systems.rs:210` | Low | S | `guardian_circle_casting_logic` takes `_clamped_cursor: Option<Vec3>` as a parameter that is never read (prefixed with `_`). The function signature misleads callers about its purpose. | Remove the dead parameter from `guardian_circle_casting_logic` and all call sites. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|--------------------------|
| `fog_cloud/systems.rs` | 584 | false | Split into: `casting.rs`, `effects.rs`, `phantoms.rs`, `utils.rs` |
| `mark_of_death/systems.rs` | 635 | false | Split into: `casting.rs`, `talent_effects.rs`, `deaths_ledger.rs`, `indicators.rs` |
| `banishment/systems.rs` | 613 | false | Split into: `casting.rs`, `vfx.rs`, `tick.rs` |
| `guardian_circle/systems.rs` | 540 | false | Split into: `casting.rs`, `buff.rs`, `talent_reactions.rs` |

---

### Looks bad but is actually fine

- **`apply_choking_fog_damage` targets all teams (no caster filter):** The game's CLAUDE.md states "friendly fire is fundamental to the game" — magic is indiscriminate by design. Not a bug.
- **`ConcealingVeilZone` and `DisorientingVaporsZone` are marker components with no system in `fog_cloud/systems.rs`:** They are intentionally read by cross-cutting systems in `combat_systems/melee.rs` and `units/archer/combat.rs`. The fog cloud module is correctly providing marker data that combat systems consume.
- **`banishment` and `mark_of_death` use raw `if input.just_released { casting_state.cancel(); }` instead of `handle_spell_release`:** `handle_spell_release` also calls `cleanup_spell_caster` for indicator cleanup. Since neither banishment nor mark_of_death use `SpellCaster`/indicators, bypassing the helper is correct.
- **`fog_cloud/systems.rs:460` pseudo-random angle for phantom placement** (`t * 7.1 + zone.origin.x * 3.3`): This is a deterministic hash-based scatter pattern to avoid using `GameRng` (which would require extra plumbing). Consistent with how other particle-scatter patterns work in the codebase.
- **`spawn_phantom_units` counter laziness** (`phantom_count.get_or_insert_with`): The lazy evaluation is intentional to avoid counting phantoms unless a zone is actually ready to spawn. This is a minor optimization, not a bug.
- **`BanishedModifier` on ghost entities being iterated by `tick_banished_units`:** `tick_banished_units` has `.run_if(is_gameplay_running)` which returns `true` only for the SP player or MP host. Ghost entities live on the guest side. The host only has real units with `BanishedModifier`, so no ghost entities are ever processed.
- **`guardian_circle/systems.rs` plugin comment block above the `Plugin` impl:** The doc comment lists all registered behaviors — this is not architectural decay but useful navigation documentation for a complex plugin.

---

### Open questions

1. **`spawn_phantom_units` on the guest in VS mode:** Given that `PhantomFogZone` is mirrored to the guest via `guest_visuals.rs` (confirmed), and the system runs under `is_spell_effects_active` (which includes the guest), does the current live version actually spawn duplicate phantom units on the guest in VS matches with the Phantom Fog talent? If so, these local phantoms are unsynced and could cause the guest's combat to diverge (ghost entities fighting unreal phantoms).
2. **Banishment in VS co-op (guest wizard):** The guest casting Banishment in a VS match effectively targets no one due to the hardcoded `Team::Defenders.is_enemy()` filter. Is Banishment reachable by the guest in the current archetype/spell assignment logic, or is it only assigned to host-side archetypes in VS?
3. **`apply_fog_cloud_evasion` on guest ghost units:** `FogEvasionModifier` is forwarded by `guest_snapshot.rs` when added to ghost entities. If the guest's fog-cloud evasion system inserts `FogEvasionModifier` on ghost units, `guest_snapshot.rs` forwards these to the host. Is this double-application (host applies via its own gameplay systems, guest also forwards) intentional or does it cause duplicate modifiers?
