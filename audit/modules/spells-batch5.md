## spells-batch5

**Scope:** `fog_cloud/`, `mark_of_death/`, `banishment/`, `guardian_circle/`

---

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
