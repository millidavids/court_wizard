# Consolidated Tech Debt Backlog — Court Wizard

Generated from Stage 2 (59 module auditors). Findings ranked by `severity × effort(inverse) × churn`.
Machine-readable source: `audit/findings.json`. Per-module detail: `audit/modules/*.md`.

**Totals:** 481 findings — 2 Critical · 66 High · 151 Medium · 262 Low. 123 non-exempt oversized files (>300 LOC); 64 correctly exempt (registries/match-monoliths).

**By category:** ArchitecturalDecay 194 · ConsistencyRot 85 · Performance 65 · DocDrift 59 · TypeContract 43 · ErrorObservability 21 · Security 5 · TestDebt 5 · Multiplayer 4

**By wave:** A (cross-cutting foundations) 32 · B (file-splits + intra-module dedup) 107 · C (dead code + deps) 39 · D (Medium/Low quick-wins) 303

---

## Priority 0 — correctness bugs (do FIRST, before any refactor)

These are genuine gameplay/multiplayer bugs the audit surfaced, not structural debt. They are
low-effort, high-value, and independent of the file-split work. Fix as a pre-wave.

| ID | Sev | File:Line | Bug | Fix |
|----|-----|-----------|-----|-----|
| F446 | Critical | src/game/battlefield/systems.rs:544 | apply_lava_damage queries (&Transform, &mut Health) without Without<GhostEntity>. Ghost units intentionally carry Health for CRDT damage propagation (guest_snap | Add Without<crate::game::multiplayer::components::GhostEntity> to the query filter tuple alongside the existing Without< |
| F133 | Critical | src/game/units/wizard/spells/dispel/bolt.rs:606 | forward_dispel_impacts_to_host queries spell_effects with &NetworkedSpellEffect, but ghost spell effects on the guest are tagged with GhostSpellEffect + Network | Add NetworkedSpellEffect { kind } to ghost spell effects when spawning them in guest_visuals.rs, or change the forwardin |
| F162 | High | src/game/units/wizard/spells/healing_plume/casting.rs:287 | apply_healing_plume_heal targets query lacks Without<GhostEntity>. Ghost units on the host carry Health, Transform, and Team; they receive healing ticks and get | Add Without<crate::game::multiplayer::components::GhostEntity> to the targets query filter, matching the pattern in enta |
| F163 | High | src/game/units/wizard/spells/mind_control/casting.rs:113 | handle_mind_control_casting enemies_query has no Without<GhostEntity> filter. Ghost units carry Team, Transform, and MeshMaterial3d<StandardMaterial>, so the ho | Add Without<GhostEntity> to the enemies query filter in handle_mind_control_casting. |
| F164 | High | src/game/units/wizard/spells/mind_control/effects.rs:241 | update_mass_hysteria_targeting iterates all_units: Query<(Entity, &Transform), Without<Corpse>> which includes ghost entities. Hysteria-afflicted units on the h | Add Without<GhostEntity> to the all_units query. |
| F109 | High | src/game/units/wizard/spells/squall/shards.rs:273 | update_ice_explosions iterates ALL IceExplosion entities with no Without<GhostSpellEffect> guard on the explosions query. Ghost IceExplosion entities on the co- | Add Without<crate::game::multiplayer::components::GhostSpellEffect> to the explosions query in update_ice_explosions. Gh |

---

## Wave A — Cross-cutting foundations (32 findings)

Shared-helper extraction + plugin/mod purity in the cross-cutting root files, done before
file-splits so the shared API stabilizes first. `save_data.rs` is **file-split only — no renames**.
Plus the Stage 1 dependency CVEs (steamworks→0.13.1, hickory-dns→0.26.1).

| ID | Sev | Eff | File:Line | Issue | Recommendation |
|----|-----|-----|-----------|-------|----------------|
| F010 | High | S | src/game/plugin.rs:50 | GlobalAttackCycle (struct + Default + tick() method) is defined inside plugin.rs. Convention requires plugin.rs to conta | Move GlobalAttackCycle to shared_systems.rs or a new attack_cycle.rs. Plugin.rs keeps only the init_resource c |
| F011 | High | S | src/game/plugin.rs:350 | apply_game_speed, auto_pause_on_focus_loss, DebugHitboxes, DebugHitboxMarker, sync_debug_hitboxes_resource, and update_d | Move debug hitbox types and systems to debug_ui.rs. Move apply_game_speed and auto_pause_on_focus_loss to shar |
| F001 | High | M | src/game/units/systems.rs:1 | God file (1878 LOC) mixing four distinct unrelated concerns: (1) shared movement/targeting helper functions (lines 1–310 | Split into concern-named siblings: dots.rs (FireDoT/Electric/Frost/Poison/Sickened/Smelly tick systems + proce |
| F020 | High | M | src/game/units/wizard/wizard_state.rs:1 | 440-line file holds 12 unrelated component/marker/resource types: PrimedSpell, Wizard, Mana, ManaRegen, CastingState, Gl | Split into at least three sibling files: mana.rs (Mana, ManaRegen), casting_state.rs (CastingState, GlobalCast |
| F012 | Medium | S | src/game/shared_systems.rs:182 | ENGAGEMENT_RANGE: f32 = 800.0 is defined as an inline const inside activate_defenders_on_proximity while DEFENDER_ACTIVA | Remove the inline const; replace with super::constants::DEFENDER_ACTIVATION_RANGE. |
| F003 | Medium | S | src/game/units/systems.rs:1179 | Hardcoded inline LinearRgba literals for Fear (purple: 0.5, 0.1, 0.6) and Petrified (gray: 0.4, 0.4, 0.4) inside update_ | Add FEAR_EFFECT_COLOR, FEAR_EFFECT_INTENSITY, PETRIFIED_EFFECT_COLOR, and PETRIFIED_EFFECT_INTENSITY to consta |
| F044 | Medium | M | src/config/save_data.rs:1 | save_data.rs is 1603 LOC combining five distinct concerns: save structs/serde types (lines 34–535), cache + disk I/O (53 | Split into save_structs.rs (all Serialize/Deserialize types + default fns), save_cache.rs (SAVE_CACHE, SAVE_DI |
| F004 | Low | S | src/game/units/components.rs:12 | Blanket pub use re-exports of three entire sub-modules (animation.rs, status_effects.rs, unit_type.rs) from components.r | Import from the owning module directly at each call site (super::animation::WalkingAnimation, super::status_ef |
| F006 | Low | S | src/game/units/components.rs:395 | The static SETUP_IMMUNITY_ACTIVE AtomicBool and its free-function accessors (set_setup_immunity, is_setup_immune) live i | Move the immunity static and its accessors to damage.rs (alongside apply_damage_to_unit which reads it) or to  |
| F005 | Low | S | src/game/units/systems.rs:688 | ElectricArcVisual component is defined inside systems.rs rather than in status_effects.rs or components.rs where all oth | Move the ElectricArcVisual struct to status_effects.rs (or to the new dots.rs when the U-01 split is done). |
| F008 | Low | S | src/game/units/systems.rs:509 | FIRE_MELTS_FROST_RATE (line 509) and FROST_QUENCHES_FIRE_RATE (line 586) are declared as local const values inside funct | Hoist both to constants.rs with appropriate doc comments; all other shared tuning constants already live there |
| F021 | Medium | M | src/game/units/wizard/systems.rs:18 | 414-line systems.rs mixes three concerns: asset loading (load_wizard_assets), entity spawning (setup_wizard, apply_arche | Extract load_wizard_assets, setup_wizard, and apply_archetype_stat_bonuses into a sibling spawn.rs. Keep runti |
| F015 | Low | S | src/game/constants.rs:664 | calculate_defender_grid_position carries two stacked doc-comment blocks (lines 664-677 describe a generic attacker-style | Remove the first (wrong) doc block; keep only the correct defender-specific comment starting at line 673. |
| F016 | Low | S | src/game/constants.rs:518 | is_ray_level() is annotated #[allow(dead_code)] and has no callers anywhere in the codebase. The Ray boss exists but thi | Either wire it into the boss spawn/selection logic or delete it to keep the public API clean. |
| F272 | Medium | S | src/ui/constants.rs:34 | GOLD_ACCENT is named 'gold' but its value is hsla(270, 0.65, 0.55) — pure purple/violet. The inline comment even says 'P | Rename to PURPLE_ACCENT (or ARCANE_ACCENT) across all six usages: constants.rs, markdown.rs, spell_book/consta |
| F022 | Medium | S | src/game/units/wizard/spell_range_indicator/systems.rs:19 | BASE_RADIUS = 3000.0 and BASE_HEIGHT = 100.0 are duplicated as local const blocks inside both setup_spell_range_indicato | Hoist the two constants to module level in spell_range_indicator/constants.rs (or import DEFAULT_SPELL_RANGE f |
| F275 | Medium | M | src/ui/layout_helpers.rs:924 | spawn_slider_row hard-codes the value display as format!("{}%", (current_value * 100.0) as u32). The roguelite tab uses  | Add an optional value_format: Option<fn(f32) -> String> (or show_as_percent: bool) field to SliderRowConfig an |
| F277 | Low | S | src/ui/systems.rs:1 | systems.rs is a four-line glob re-export hub (pub use super::button_systems::*; pub use super::layout_helpers::*). This  | Either delete systems.rs and migrate all callers to import directly from button_systems or layout_helpers, or  |
| F051 | Low | M | src/config/systems.rs:66 | load_and_apply_config manually copies every field from ConfigFile.game into GameConfig (30+ field assignments, lines 66– | Extract a GameConfig::from_config_file(config_file: &ConfigFile) -> Self constructor. Document which fields ar |
| F017 | Low | S | src/game/messages.rs:27 | WaveSpawnedMessage (and its wave_number field, suppressed with #[allow(dead_code)]) is written by tick_wave_timer but ha | Either add a subscriber (e.g., a UI wave-number notification) or remove the message type and the MessageWriter |
| F023 | Low | S | src/game/units/wizard/spell_enum.rs:159 | #[allow(dead_code)] on Spell::category() is stale. The method is actively called from src/ui/cauldron_menu/setup.rs:426, | Remove the #[allow(dead_code)] attribute from Spell::category(). |
| F024 | Low | S | src/game/units/wizard/spell_range_indicator/systems.rs:50 | update_spell_range_indicator uses .iter().next() on a With<LocalWizard> query (single-entity) and on the SpellRangeCircl | Replace both .iter().next() guards with .single()/.single_mut() to be consistent with the rest of the codebase |
| F027 | Low | S | src/game/units/wizard/wizard_state.rs:378 | WizardAnimation has a new() method returning all-zero fields but does not #[derive(Default)]. This is inconsistent with  | Add #[derive(Default)] to WizardAnimation (both fields zero-initialize naturally) and replace WizardAnimation: |
| F046 | Low | S | src/config/input_bindings.rs:120 | #[allow(dead_code)] appears on BindingContext (line 120), BindingAction (line 132), the InputBindings impl block (line 3 | Remove all four #[allow(dead_code)] attributes in input_bindings.rs. Run cargo check to confirm nothing is gen |
| F202 | Low | S | src/game/units/boss/utils.rs:7 | EYE_SHEET_WIDTH and EYE_SHEET_HEIGHT are exported pub(in crate::game) but are never imported outside utils.rs. Only EYE_ | Demote to private module-level constants since they are only used to define EYE_FRAME_UV in the same file. Rem |
| F203 | Low | S | src/game/units/boss/utils.rs:84 | is_on_screen is a general camera-NDC utility in the shared boss helpers but has exactly one call-site (dark_mage/ai.rs:5 | Inline into dark_mage/ai.rs where it is the sole consumer, or add a doc comment explicitly noting it is intend |
| F276 | Low | S | src/ui/layout_helpers.rs:269 | Three inline Color::hsla(25.0, …) literals for the contact, medium, and ambient shadow layers in spawn_page_container's  | Extract as FRAME_SHADOW_CONTACT, FRAME_SHADOW_MEDIUM, and FRAME_SHADOW_AMBIENT constants in constants.rs and r |
| F279 | Low | S | src/ui/layout_helpers.rs:734 | spawn_dot_leader uses an inline TextColor(Color::hsla(0.0, 0.0, 0.3, 1.0)) with no named constant. This specific shade i | Extract as DOT_LEADER_COLOR in constants.rs. |
| F280 | Low | S | src/ui/layout_helpers.rs:850 | The slider track background uses an inline BackgroundColor(Color::srgb(0.2, 0.2, 0.2)) with no named constant, inconsist | Extract as SLIDER_TRACK_BG in constants.rs alongside the other SLIDER_* constants. |
| F025 | Low | S | src/game/units/wizard/spell_status_effects.rs:128 | spawn_status_effects_section is a UI spawning function (builds Bevy UI nodes via ChildSpawnerCommands) living inside the | Move spawn_status_effects_section and effective_status_effects to a shared UI helper module (e.g., src/ui/spel |
| F018 | Low | S | src/game/debug_ui.rs:52 | toggle_debug_ui_visible is registered in Update with no run_if guard. Every other Update system in the project carries a | Add a lightweight guard such as .run_if(not(in_state(AppState::Loading))) to match project convention. |
| F278 | Low | S | src/ui/button_systems.rs:37 | ParchmentPanel and FrostedGlassOverlay marker components are defined in button_systems.rs but are semantically owned by  | Move ParchmentPanel and FrostedGlassOverlay definitions to layout_helpers.rs alongside the material types they |

## Wave B — Per-module file-splits + intra-module dedup

**123 files over 300 LOC** to split into concern-named siblings (largest first). Each handled
by one coordinator-per-parent-mod.rs to avoid registration races. Split + dedup atomic per module.

| LOC | File | Proposed split |
|-----|------|----------------|
| 1969 | src/ui/wizard_tower/study_tab/interaction.rs | detail_panel.rs, slider_interaction.rs, cursor.rs, talent_interaction.rs, allocation.rs |
| 1878 | src/game/units/systems.rs | dots.rs, vfx_tinting.rs, animation_systems.rs, systems.rs (movement/targeting helpers only) |
| 1682 | src/ui/wizard_tower/roguelite_tab.rs | constants.rs, components.rs, panel_no_run.rs, panel_active_run.rs, init.rs, slider.rs, seed_input.rs, toggle.rs, run_summary.rs |
| 1650 | src/ui/main_menu/settings/builders.rs | settings/builders.rs, settings/keybind_systems.rs, settings/tab_systems.rs |
| 1603 | src/config/save_data.rs | save_structs.rs, save_cache.rs, save_ops.rs, save_migration.rs |
| 1496 | src/game/multiplayer/spell_sync.rs | spell_sync_collect.rs, spell_sync_render.rs |
| 1435 | src/ui/compendium/setup.rs | layout.rs, handlers.rs, item_spawners.rs, rebuild.rs |
| 1403 | src/ui/wizard_tower/study_tab/panels.rs | helpers.rs, spawn.rs, commit.rs, graph_nav.rs, positions.rs |
| 1364 | src/game/multiplayer/guest_snapshot.rs | guest_status_forward.rs, guest_snapshot.rs |
| 1203 | src/game/multiplayer/systems.rs | run_conditions.rs, lifecycle.rs, score_ui.rs, pause_ui.rs, disconnect.rs |
| 1120 | src/game/units/boss/hags/core.rs | spawn.rs, movement.rs, combat.rs, eye_transfer.rs, death.rs, animation.rs |
| 1102 | src/game/units/boss/ray/movement.rs | spawn.rs, lifecycle.rs, particles.rs, fear_movement.rs, beams.rs (receives disintegration + petrification) |
| 1096 | src/game/units/components.rs | components.rs (pure component structs and traits), damage_helpers.rs (apply_spell_damage, apply_damage_to_unit, immunity static) |
| 1067 | src/game/multiplayer/host_systems.rs | host_snapshot.rs, host_game_over.rs, host_message_handlers.rs |
| 1062 | src/game/achievements/checks.rs | victory_checks.rs, defeat_checks.rs, midbattle_checks.rs, encounter_checks.rs, boss_checks.rs, spell_unlock_checks.rs, completionist_checks. |
| 1039 | src/game/units/wizard/spells/arcane_crystal/auto.rs | auto_cast.rs, turret.rs, network.rs, talents.rs, spawn_helpers.rs |
| 1024 | src/game/units/boss/hags/abilities.rs | justina.rs, josephina.rs, martina.rs |
| 965 | src/game/multiplayer/guest_visuals.rs | guest_spell_spawn.rs, guest_visuals.rs |
| 936 | src/ui/layout_helpers.rs | materials.rs, panels.rs, escape.rs, slider.rs, button_spawn.rs |
| 923 | src/game/units/boss/ray/beams.rs | beams/disintegration.rs, beams/petrification.rs, beams/fear.rs, beams/mind_control.rs, beams/teleport.rs, beams/helpers.rs |
| 892 | src/game/cauldron/systems.rs | spawn.rs, brew_lifecycle.rs, visuals.rs, army_buffs.rs, multiplayer.rs |
| 871 | src/game/units/wizard/spells/dispel/bolt.rs | impacts.rs, null_zone.rs, mp_dispel.rs, suppress.rs |
| 868 | src/game/units/wizard/spells/squall/shards.rs | projectiles.rs, explosions.rs, talents.rs, snow.rs |
| 852 | src/ui/wizard_tower/layout/setup.rs | layout/resources.rs, layout/panel_rebuild.rs, layout/tab_state.rs |
| 811 | src/game/units/boss/dark_mage/spells.rs | spell_updates.rs, spell_spawn.rs, targeting.rs |
| 804 | src/game/units/wizard/spells/utils.rs | spell_origin.rs, spell_math.rs, spell_heal.rs, casting_helpers.rs, ring_vfx.rs |
| 792 | src/game/crt_effect/pipeline.rs | pipeline.rs, sync.rs |
| 790 | src/game/units/wizard/spells/spike_growth/systems.rs | casting.rs, damage.rs, effects.rs, vfx.rs, spawn.rs, systems.rs (re-export hub) |
| 784 | src/game/units/wizard/spells/grease/ignite.rs | ignite.rs, lifecycle.rs, spawn.rs, vfx.rs |
| 782 | src/ui/tutorial/lifecycle.rs | triggers.rs, overlay.rs, highlight.rs, handlers.rs, persistence.rs |
| 780 | src/ui/in_game/bars.rs | bars.rs, boss_bar.rs |
| 757 | src/game/units/wizard/spells/polymorph/systems.rs | polymorph/casting.rs, polymorph/livestock.rs, polymorph/behaviors.rs |
| 753 | src/game/units/wizard/spells/finger_of_death/effects.rs | effects.rs, visual.rs |
| 750 | src/game/units/wizard/spells/arcane_crystal/setup.rs | casting.rs, spawn.rs, visuals.rs, helpers.rs |
| 745 | src/game/units/wizard/spells/fireball/projectile.rs | fireball/explosion.rs, fireball/trail_effects.rs |
| 742 | src/game/units/wizard/spells/disintegrate/beam.rs | spawn.rs, visuals.rs, particles.rs, searing_finale.rs |
| 739 | src/game/units/wizard/archetypes/gunslinger/fire.rs | hitscan.rs, tracer.rs, flamethrower.rs, fire.rs |
| 734 | src/game/units/wizard/spells/finger_of_death/casting.rs | casting.rs, damage.rs |
| 732 | src/game/units/boss/ogre/charge.rs | charge.rs, charge_visuals.rs, rock_throw.rs, ogre_animation.rs |
| 726 | src/ui/in_game/spawn.rs | spawn.rs, input.rs, widgets.rs |
| 722 | src/game/multiplayer/loading.rs | spawn_tasks.rs, loading.rs |
| 703 | src/game/units/wizard/spells/teleport/casting.rs | casting.rs, finalize.rs |
| 697 | src/game/constants.rs | colors.rs, positions.rs, spawn_math.rs, tuning.rs |
| 691 | src/game/units/healer/systems.rs | healer/targeting.rs, healer/channel.rs, healer/bolt.rs, healer/targeting_helpers.rs |
| 690 | src/game/units/wizard/spells/vfx/cast_effects.rs | cast_effects.rs, mp_sync.rs |
| 688 | src/game/units/wizard/spells/telekinesis/systems.rs | telekinesis/casting.rs, telekinesis/talents.rs, telekinesis/vfx.rs |
| 677 | src/game/units/wizard/spells/disintegrate/casting.rs | talent_config.rs, damage.rs, cleanup.rs |
| 665 | src/game/units/wizard/spells/vfx/area_effects.rs | heat_shimmer.rs, plague_smoke.rs, fire_smoke.rs, embers.rs |
| 657 | src/game/units/wizard/spells/wall_of_stone/lifecycle.rs | cancel.rs, tick.rs, combat.rs, vfx.rs, talents.rs |
| 656 | src/game/units/wizard/spells/berserker_rage/systems.rs | berserker_rage/casting.rs, berserker_rage/talent_effects.rs |
| 640 | src/game/units/dispeller/systems.rs | dispeller/targeting.rs, dispeller/movement.rs, dispeller/channel.rs, dispeller/ranged_combat.rs |
| 640 | src/ui/button_systems.rs | click.rs, structure.rs, animation.rs, active.rs, focus_tint.rs, sync.rs |
| 636 | src/game/units/wizard/spells/meteor_fall/meteor.rs | projectile.rs, explosion.rs, ground_fire.rs |
| 635 | src/game/units/wizard/spells/mark_of_death/systems.rs | casting.rs, talent_effects.rs, deaths_ledger.rs, indicators.rs |
| 627 | src/game/units/teleporter/systems.rs | teleporter/spawn.rs, teleporter/targeting.rs, teleporter/movement.rs, teleporter/channel.rs, teleporter/ranged_combat.rs |
| 626 | src/game/battlefield/systems.rs | src/game/battlefield/spawn.rs, src/game/battlefield/effects.rs |
| 625 | src/ui/compendium/rows.rs | stat_rows.rs, detail_panel.rs |
| 622 | src/config/resources.rs | wizard_type.rs, config_file.rs, game_config.rs |
| 619 | src/game/units/wizard/archetypes/swordcerer/combat.rs | avatar_movement.rs, sword_arc.rs, combat.rs |
| 613 | src/game/units/wizard/spells/banishment/systems.rs | casting.rs, vfx.rs, tick.rs |
| 603 | src/ui/action_bar/systems.rs | spawn.rs, input.rs, keyboard_highlight.rs |
| 587 | src/game/units/boss/dark_mage/ai.rs | spawn.rs, movement.rs, teleport.rs |
| 587 | src/game/units/king/systems.rs | king/spawn.rs, king/movement.rs, king/cohesion.rs, king/spell_shield.rs, king/aura_vfx.rs |
| 584 | src/game/units/wizard/spells/fog_cloud/systems.rs | casting.rs, effects.rs, phantoms.rs, utils.rs |
| 581 | src/game/movement_systems.rs | flocking.rs, wall_collision.rs, rough_terrain.rs |
| 580 | src/ui/cauldron_menu/setup.rs | build_menu.rs, detail_panel.rs, ingredient_list.rs, stone_selector.rs |
| 577 | src/game/units/wizard/spells/raise_the_dead/casting.rs | casting.rs, raising.rs |
| 564 | src/game/units/boss/lich/combat.rs | soul_power.rs, movement.rs, combat.rs, animation.rs |
| 557 | src/game/multiplayer/spawning.rs | spawn_units.rs, spawn_world.rs |
| 548 | src/game/units/boss/ogre/combat.rs | spawn.rs, facing.rs, melee.rs, movement.rs, enrage.rs |
| 548 | src/ui/game_over/screen.rs | src/ui/game_over/layout.rs, src/ui/game_over/actions.rs |
| 544 | src/game/crt_effect/components.rs | shader_settings.rs, timers.rs |
| 540 | src/game/units/wizard/spells/guardian_circle/systems.rs | casting.rs, buff.rs, talent_reactions.rs |
| 537 | src/game/units/wizard/spells/wall_of_fire/casting.rs | wall_of_fire/placement.rs |
| 532 | src/game/units/wizard/spells/meteor_fall/casting.rs | casting.rs, projectile.rs |
| 532 | src/game/units/infantry/systems.rs | infantry/activation.rs, infantry/targeting.rs, infantry/spawn.rs |
| 531 | src/game/units/wizard/spells/entangle/casting.rs | entangle/casting.rs, entangle/ground_effect.rs, entangle/root_effects.rs |
| 524 | src/game/terrain/boulder/systems.rs | boulder/projectile.rs, boulder/lifetime.rs, boulder/combat.rs, boulder/tint.rs, boulder/spawn.rs |
| 523 | src/game/combat_systems/melee.rs | targeting.rs (snapshot building, range check, fog redirect logic), damage_calc.rs (modifier accumulation and apply_damage_to_unit call), att |
| 521 | src/game/shared_systems.rs | ambience.rs, shadows.rs, shared_systems.rs (helpers only) |
| 518 | src/game/units/wizard/spells/sleep/systems.rs | casting.rs, effects.rs, sleepwalking.rs |
| 499 | src/game/pathfinding/runtime.rs | rebuild.rs, sampling.rs |
| 492 | src/game/units/wizard/spells/magic_missile/casting.rs | magic_missile/targeting.rs |
| 488 | src/game/units/shielder/systems.rs | shielder/targeting.rs, shielder/movement.rs, shielder/channel.rs |
| 486 | src/game/units/wizard/spells/mind_control/casting.rs | casting.rs, highlight.rs |
| 480 | src/game/units/wizard/spells/wall_of_fire/damage.rs | wall_of_fire/talent_effects.rs |
| 475 | src/game/units/wizard/spells/teleport/arrival.rs | arrival.rs, indicators.rs |
| 472 | src/game/pathfinding/setup.rs | setup.rs, wave_activation.rs, stuck_detection.rs |
| 471 | src/game/units/wizard/spells/chain_lightning/casting.rs | casting.rs, targeting.rs, arc.rs |
| 462 | src/ui/wizard_tower/plugin.rs | layout/panel_rebuild.rs |
| 454 | src/game/terrain/pond/systems.rs | pond/wet.rs, pond/freeze.rs, pond/evaporation.rs, pond/shock.rs, pond/ripples.rs, pond/spawn.rs |
| 453 | src/game/units/archer/movement.rs | archer/targeting.rs, archer/physics.rs, archer/spawn.rs |
| 449 | src/game/plugin.rs | attack_cycle.rs (or shared_systems.rs), debug_ui.rs (hitbox debug), shared_systems.rs (auto_pause, apply_game_speed) |
| 447 | src/ui/rune_display/systems.rs | spawn.rs, visuals.rs, input.rs, sequence.rs, glyphs.rs |
| 446 | src/game/units/wizard/spells/black_hole/gravity.rs | gravity.rs, damage.rs, lifecycle.rs |
| 444 | src/game/game_mode/components.rs | resources.rs, modifiers.rs, run_stats.rs, components.rs |
| 441 | src/game/units/wizard/spells/haste/systems.rs | casting.rs, expiry.rs |
| 440 | src/game/units/wizard/wizard_state.rs | mana.rs (Mana, ManaRegen), casting_state.rs (CastingState, GlobalCastCooldown, SpellCaster), wizard.rs (Wizard, LocalWizard, GuestWizard, Wi |
| 440 | src/game/units/wizard/spells/battle_hymn/systems.rs | casting.rs, buff.rs |
| 438 | src/game/multiplayer/coop_pause.rs | coop_pause_state.rs, coop_pause.rs |
| 437 | src/game/crt_effect/systems.rs | cursor_correction.rs, animations.rs |
| 435 | src/game/units/wizard/spells/plague_wind/cloud.rs | spawn.rs, movement.rs, damage.rs, pandemic.rs, vfx.rs |
| 433 | src/game/units/wizard/spells/grease/casting.rs | casting.rs, slow.rs |
| 431 | src/game/units/wizard/spells/wall_of_stone/casting.rs | casting.rs, talent_params.rs |
| 427 | src/game/units/wizard/spells/healing_plume/casting.rs | casting.rs, zone.rs |
| 418 | src/game/units/archer/combat.rs | archer/melee.rs, archer/ranged.rs |
| 414 | src/game/units/wizard/systems.rs | spawn.rs (load_wizard_assets, setup_wizard, apply_archetype_stat_bonuses), systems.rs (regenerate_mana, mana_on_kill, handle_prime_spell_mes |
| 408 | src/ui/pause_menu/main/systems.rs | src/ui/pause_menu/main/layout.rs, src/ui/pause_menu/main/actions.rs |
| 404 | src/game/units/aerialist/systems.rs | aerialist/spawn.rs, aerialist/targeting.rs, aerialist/movement.rs, aerialist/combat.rs |
| 404 | src/steam/multiplayer/lobby_systems.rs | create_lobby.rs, join_lobby.rs, lobby_systems.rs |
| 393 | src/game/input/gamepad/systems.rs | device_detection.rs, virtual_cursor.rs, trigger_translation.rs, radial_slot.rs |
| 391 | src/game/loading/init.rs | coop_session.rs, wave_init.rs, init.rs |
| 369 | src/game/achievements/resources.rs | resources.rs, lifecycle.rs |
| 363 | src/ui/wizard_tower/components.rs | study_tab/allocation.rs |
| 360 | src/game/multiplayer/coop.rs | coop_resources.rs, coop_lifecycle.rs |
| 357 | src/ui/cauldron_menu/interaction.rs | button_action.rs, detail_panel_update.rs |
| 356 | src/game/pathfinding/debug.rs | debug_flow_field.rs, debug_ball.rs |
| 353 | src/ui/main_menu/settings/interaction.rs | settings/option_interaction.rs, settings/slider_interaction.rs |
| 345 | src/game/wave_systems.rs | wave_spawn.rs, wave_upgrades.rs |
| 340 | src/game/units/wizard/spells/dispel/casting.rs | casting.rs, spawn.rs |
| 326 | src/game/units/brute/systems.rs | spawn.rs, targeting.rs, movement.rs, rock_throw.rs |
| 321 | src/game/combat_systems/post_combat.rs | invulnerability.rs (enforce_invulnerability system), death_conversion.rs (convert_dead_to_corpses system) |
| 313 | src/ui/wizard_tower/constants.rs | coop_button.rs |

Plus 107 architectural-decay findings tied to these splits (see `audit/findings.json`, wave=B).

## Wave C — Dead code + dependency/config hygiene (39 findings)

| ID | Sev | Eff | File:Line | Issue |
|----|-----|-----|-----------|-------|
| F347 | High | S | src/ui/action_bar/systems.rs:117 | ActionBarSlotText component is defined (components.rs:19) and queried in update_action_bar_slots (lines 348-440) but is never spawned anywhe |
| F425 | High | S | src/game/input/messages.rs:13 | cursor_position: Option<Vec2> fields on MouseLeftPressed, MouseLeftHeld, MouseRightPressed, and MouseRightHeld are all marked #[allow(dead_c |
| F082 | Medium | S | src/game/units/wizard/spells/disintegrate/components.rs:57 | DisintegrateBeam.damage_type is annotated #[allow(dead_code)]. It is set once in new() but never read back; every damage system uses constan |
| F105 | Medium | S | src/game/units/wizard/spells/squall/components.rs:99 | Five fields on IceProjectile and IceExplosion are annotated #[allow(dead_code)]: damage_type, radius, empowerment (×2). If genuinely never r |
| F417 | Medium | S | src/game/loading/init.rs:221 | The wildcard _ arm of the match on tier % BOSS_CYCLE_LENGTH (which spawns an Ogre) is unreachable dead code. The enclosing guard on line 203 |
| F152 | Medium | S | src/game/units/wizard/spells/raise_the_dead/casting.rs:1 | casting.rs is 577 lines. The bottom half (find_nearest_corpse, raise_corpse_as_undead, resurrect_nearest_corpse, try_raise_or_forward) is ra |
| F238 | Medium | S | src/game/units/boss/lich/combat.rs:32 | track_soul_power carries a copy-pasted doc comment ('Checks if it's time to spawn the Lich mid-game…') that belongs to check_lich_spawn in s |
| F243 | Medium | S | src/game/units/boss/lich/combat.rs:409 | resolve_finger_of_death inserts PendingUndeadRaise as a singleton Resource. The wizard's FoD spell (casting.rs line 653) also inserts this s |
| F476 | Low | S | src/state/states.rs:19 | #[allow(dead_code)] on AppState and MenuState with the comment 'Variants will be used as game features are implemented' is stale — both enum |
| F193 | Low | S | src/game/units/wizard/spells/guardian_circle/systems.rs:210 | _clamped_cursor: Option<Vec3> is a dead parameter in guardian_circle_casting_logic — it is never read and prefixed with underscore, indicati |
| F287 | Low | S | src/ui/wizard_tower/components.rs:239 | TalentProgressBarFill.spell is marked #[allow(dead_code)]. The field is written at interaction.rs:338 but no system ever reads fill.spell —  |
| F306 | Low | S | src/ui/wizard_tower/components.rs:240 | TalentProgressBarFill.spell is annotated #[allow(dead_code)], meaning the field is defined but never read. Suppressed dead-code is either un |
| F401 | Low | S | src/game/pathfinding/resources.rs:24 | PathfindingGrid::world_max carries #[allow(dead_code)] and is never read anywhere outside its constructor. It is stored in the resource but  |
| F157 | Low | S | src/game/units/wizard/spells/black_hole/components.rs:57 | BlackHole.damage_type field is annotated #[allow(dead_code)] and never read. All damage call sites in gravity.rs hard-code DamageType::Force |
| F145 | Low | S | src/game/units/wizard/spells/fireball/components.rs:100 | #[allow(dead_code)] on FireballExplosion.damage_type is a false negative: the field is read at projectile.rs:550 (TerrainDamageMessage) and  |
| F173 | Low | S | src/game/units/wizard/spells/lightning_rod/casting.rs:247 | _wizard: &Wizard parameter in lightning_rod_casting_logic is never read (prefixed underscore to suppress warning). It is passed from the out |
| F400 | Low | S | src/game/pathfinding/messages.rs:15 | The rebuild: bool field of ObstacleChanged is decorated #[allow(dead_code)] and its doc-comment states it 'is no longer needed'. It is set a |
| F468 | Low | S | src/networking/session.rs:67 | MultiplayerSession.host_spells and guest_spells are both #[allow(dead_code)]. They are populated at session creation but never read during g |
| F035 | Low | S | src/game/units/wizard/archetypes/gunslinger/components.rs:50 | Four methods (fire_interval, is_hold_to_fire on GunType; two helpers in resources.rs) are tagged #[allow(dead_code)], indicating they are de |
| F172 | Low | S | src/game/units/wizard/spells/chain_lightning/components.rs:24 | #[allow(dead_code)] on ChainLightningBolt.damage_type is stale. The field is actively read: bolt.damage_type is copied into ChainLightningBo |
| F384 | Low | S | src/game/cauldron/components.rs:30 | CauldronState::Cooldown { remaining } is marked #[allow(dead_code)] and is never constructed anywhere in the codebase. The tick() branch for |
| F385 | Low | S | src/game/cauldron/components.rs:46 | CauldronState::active_recipe() is annotated #[allow(dead_code)] and is genuinely unused. CauldronState::progress() is also annotated #[allow |
| F343 | Low | S | src/ui/cauldron_menu/interaction.rs:99 | update_detail_panel_on_selection_change is already gated by .run_if(resource_changed::<IngredientSelection>) in plugin.rs:47. The manual if  |
| F437 | Low | S | src/game/game_mode/components.rs:128 | Six methods in ActiveToggles and ToggleModifier carry #[allow(dead_code)] attributes. In a binary crate the compiler already warns only for  |
| F042 | Low | S | src/game/units/wizard/talents/resources.rs:76 | set_selection and has_talent on ActiveTalents are both marked #[allow(dead_code)] and are never called anywhere in the codebase outside the  |
| F102 | Low | S | src/game/units/wizard/spells/teleport/casting.rs:454 | teleport_casting_logic accepts _primed_spell: &PrimedSpell (underscore prefix indicating unused) as its 5th parameter. The outer system pass |
| F269 | Low | S | src/game/units/commander/components.rs:27 | TeamFilter::Both is marked #[allow(dead_code)] and has zero call sites anywhere in the codebase. It has been suppressed since the commander  |
| F375 | Low | S | src/game/achievements/checks.rs:24 | Some BattleEndedMessage handlers call grant_achievement_insight after do_unlock (check_first_victory, check_the_king_is_dead, check_chain_re |
| F378 | Low | S | src/game/achievements/checks.rs:425 | detect_out_of_range iterates all non-wizard, non-corpse entities every gameplay frame and filters by Team::Defenders inside the loop body. A |
| F394 | Low | S | src/game/terrain/pond/components.rs:16 | Pond::obstacle_bounds and Tree::obstacle_bounds (tree/components.rs:65) are annotated #[allow(dead_code)]. No callers exist anywhere in the  |
| F092 | Low | S | src/game/units/wizard/spells/finger_of_death/effects.rs:21 | The doc comment on process_pending_undead_raises reads '/// Computes talent-modified parameters for Finger of Death.' — a copy-paste from th |
| F097 | Low | S | src/game/units/wizard/spells/finger_of_death/effects.rs:47 | The nearest-corpse search uses best.as_ref().map(\|(_, d)\| *d).unwrap_or(f32::MAX) — a verbose pattern. More critically, if a primary beam  |
| F159 | Low | S | src/game/units/wizard/spells/raise_the_dead/effects.rs:87 | tick_plague_bearer_aura and handle_undead_detonation use apply_damage_to_unit which bypasses SpellShield and spell_vulnerability, while all  |
| F160 | Low | S | src/game/units/wizard/spells/raise_the_dead/effects.rs:164 | handle_undead_detonation injects Res<Time> and immediately assigns let _t = time.elapsed_secs() which is never used. Dead code leftover from |
| F412 | Low | S | src/game/crt_effect/distortion.rs:49 | let count = 0u32; in update_lensing_positions is a dead variable. Slots 0-1 were previously used for black-hole lensing and are now permanen |
| F461 | Low | S | src/steam/multiplayer/lobby_state.rs:32 | The peer field in SteamLobbyState::Joined is suppressed with #[allow(dead_code)], but sync_coop_peer_name in lobby_systems.rs already reads  |
| F470 | Low | S | src/networking/transport/runtime.rs:44 | TransportEvent::PingUpdate(f32) is #[allow(dead_code)] — the iroh transport never emits it. The bridge handler at bridge.rs:162-164 processe |
| F041 | Low | S | src/game/units/wizard/talents/definitions/mod.rs:20 | TalentDefinition.implemented: bool is marked #[allow(dead_code)], is never set to false anywhere in the codebase, and is never read. It resu |
| F118 | Low | M | src/game/units/wizard/spells/wall_of_stone/lifecycle.rs:425 | `collapsing_wall_explosion` performs a full `O(walls × enemies)` nested loop scan every `PostCombatSet` tick while any `WallTalents` entity  |

## Wave D — Medium/Low quick-wins (303 findings)

Full list in `audit/findings.json` (wave=D). Top 30 by churn-weighted score:

| ID | Sev | Eff | File:Line | Issue |
|----|-----|-----|-----------|-------|
| F014 | Medium | M | src/game/shared_systems.rs:68 | calculate_effectiveness runs an O(N²) double-pass over all non-boss non-corpse units every frame with no early-out for units alrea |
| F002 | Medium | S | src/game/units/plugin.rs:173 | process_pending_damage_effects, update_fire_dot, update_electric_charge, update_electric_arc_visuals, and update_persistent_effect |
| F308 | Medium | S | src/ui/main_menu/settings/systems.rs:1 | systems.rs is solely a glob re-export hub (pub use super::builders::*; pub use super::interaction::*;) with a stale migration note |
| F273 | Medium | S | src/ui/plugin.rs:96 | update_ui_scale runs unconditionally every Update frame with no run_if guard or Changed<Window> query filter. It reads two resourc |
| F349 | Medium | S | src/ui/action_bar/systems.rs:322 | handle_slot_click checks exclusive casting via config.wizard_type.uses_exclusive_casting() (line 292), while handle_keyboard_input |
| F045 | Medium | S | src/config/systems.rs:128 | apply_vsync_config maps both VsyncMode::On and VsyncMode::Adaptive to PresentMode::AutoVsync. The UI exposes all three modes as di |
| F054 | Medium | S | src/game/multiplayer/host_systems.rs:543 | apply_status_to_entity doc still references 'Phase 1' and 'Phase 3' as future work, but Phase 3 appears to have shipped. Separatel |
| F057 | Medium | S | src/game/multiplayer/host_systems.rs:551 | apply_status_to_entity takes polymorph_capture: Option<(Team, f32, f32, Handle<StandardMaterial>, Handle<Mesh>, f32)> — a 6-elemen |
| F194 | Medium | S | src/game/units/wizard/spells/sleep/systems.rs:192 | sleep_casting_logic hand-rolls the same spell-range clamp (Pythagorean ground-radius, center-distance clamping) that clamp_cursor_ |
| F249 | Medium | M | src/game/units/infantry/systems.rs:481 | spawn_single_kings_guard computes its spawn position from a hard-coded centroid of raw castle corner coordinates (centroid_x = -15 |
| F053 | Medium | S | src/game/multiplayer/loading.rs:132 | MpSpawnQueue::pop_next calls Vec::remove(0), which shifts all remaining elements on every call. During loading this is called in a |
| F178 | Medium | S | src/game/units/wizard/spells/polymorph/systems.rs:362 | Entangle (casting.rs:351, vines.rs:35) and Polymorph (systems.rs:362, 404, 513, 637, 670, 717) use Vec3::distance() (3D Euclidean) |
| F197 | Medium | S | src/game/units/wizard/spells/battle_hymn/systems.rs:311 | Nine talent balance literals are inlined as bare floats in apply_battle_hymn_buff: 1.5 (Inspiring Words duration), 1.5 (War Drums  |
| F381 | Medium | S | src/game/cauldron/systems.rs:623 | MAX_SHIELD: f32 = 20.0 is declared as a local const inside both shield_defenders (line 623) and apply_guest_army_buffs (line 812). |
| F387 | Medium | S | src/game/cauldron/systems.rs:124 | handle_brew_complete calls load_unified_save() (synchronous disk I/O) inside a game-loop system. Although gated on on_message::<Br |
| F416 | Medium | S | src/game/loading/spawn_queue.rs:54 | SpawnTask::Castle is enqueued in init.rs:81 but the match arm in queue.rs:304-306 is a complete no-op. The castle is spawned impli |
| F258 | Medium | S | src/game/units/dispeller/systems.rs:86 | In update_dispeller_targeting, spell_edge_distance is called twice per candidate pair inside min_by (dist_a and dist_b) and then a |
| F259 | Medium | S | src/game/units/dispeller/systems.rs:593 | Both dispeller_ranged_combat (line 593) and teleporter_ranged_combat (teleporter/systems.rs:580) compute .distance() twice per can |
| F297 | Medium | S | src/ui/wizard_tower/constants.rs:219 | spawn_coop_gated_button is a UI builder function with branching logic living in constants.rs, violating the 'constants only' contr |
| F409 | Medium | S | src/game/crt_effect/plugin.rs:205 | plugin.rs defines LensingLabel, HeatDistortionLabel, CrtEffectLabel, CrtEffectNode, and a full ViewNode implementation (lines 205- |
| F447 | Medium | S | src/game/battlefield/plugin.rs:45 | load_battlefield_assets has its full system body defined in plugin.rs. Per project convention plugin.rs must contain Bevy registra |
| F348 | Medium | S | src/ui/action_bar/plugin.rs:140 | plugin.rs defines two system bodies — action_bar_enabled (line 140) and reset_layout_progress (line 149) — directly in the plugin  |
| F248 | Medium | S | src/game/units/king/plugin.rs:56 | despawn_king_aura_on_death is registered with no run_if guard of any kind. While the Added<Corpse> filter makes it cheap at steady |
| F418 | Medium | S | src/game/loading/upgrade_systems.rs:104 | apply_commander_upgrade accepts _materials: &mut Assets<StandardMaterial> and _meshes: &mut Assets<Mesh> (underscore-prefixed, nev |
| F151 | Medium | M | src/game/units/wizard/spells/spike_growth/systems.rs:1 | systems.rs is 790 lines mixing six distinct concerns: casting input, zone damage+CC, lingering poison tick, death-garden growth, s |
| F009 | Low | M | src/game/units/components.rs:219 | AttackTiming::can_attack and can_attack_with_speed_bonus — a timer-wrapping algorithm annotated with a comment about a 60-hit-per- |
| F019 | Low | S | src/game/constants.rs:122 | The doc-comment on the SPELL_OFFSET constant says 'Offset from wizard position to place the cauldron beside the wizard' — but SPEL |
| F068 | Medium | S | src/game/units/wizard/spells/arcane_crystal/components.rs:66 | ArcaneCrystal.fod_beams_processed and explosions_processed are Vec<Entity> that grow unboundedly for the crystal's full lifetime ( |
| F291 | Medium | S | src/ui/wizard_tower/roguelite_tab.rs:1211 | slider_interaction, update_sliders, and update_slider_text all take ResMut/Res<RogueliteModifiers> as required (non-Optional) para |
| F299 | Medium | S | src/ui/wizard_tower/multiplayer_tab/plugin.rs:92 | multiplayer_tab/plugin.rs contains four non-trivial system function bodies (route_pending_rematch_from_menu, handle_pending_rematc |

---

## Open questions for the maintainer

- **[units-root]** process_pending_damage_effects (systems.rs:332) spawns BurningPatch entities inline for the Drought fire synergy. Should that spawning belong to a meteorologist-owned system, or is the central DoT processor the correct owner for cross-archetype synergies?
- **[units-root]** ElectricArcVisual update (systems.rs:908) has no Without<GhostEntity> filter. Arc visuals are cosmetic — is it intentional that ghost entities on the guest can also spawn arc visuals, or should arc spawning be host-only like the Shocked damage itself?
- **[units-root]** apply_unit_movement (movement.rs:19) has no Without<Corpse> filter; instead clear_corpse_velocity runs as a separate trailing system. Is the two-system approach preferred for clarity, or would a filter in apply_unit_movement be cleaner?
- **[game-root]** WaveSpawnedMessage subscribers: Was there a UI wave-number overlay removed without also removing the emit? Or is a subscriber planned?
- **[game-root]** is_ray_level dead code: The Ray boss presumably spawns via the boss-cycle modulo logic in wave_systems.rs — is is_ray_level intentionally bypassed, or is it a gap in the boss spawn path?
- **[game-root]** constants.rs scope creep: calculate_total_aerialists already imports from units::aerialist::constants. As more unit types add tier functions here, will this create a dependency cycle? Should constants.rs become a module?
- **[game-root]** apply_separation COLLISION_ITERATIONS re-collect: Would switching the hard-collision pass to Bevy 0.18 parallel queries eliminate the need for the snapshot vec and reduce allocation pressure?
- **[wizard-root]** mana_on_kill and regenerate_mana run on the GuestWizard proxy on the co-op host. Is there any current or planned path where the host reads GuestWizard mana for casting decisions (e.g., mana-gated spell execution)? If so, With<LocalWizard> should be the filter for all mana-mutation systems.
- **[wizard-root]** BASE_HEIGHT = 100.0 in spell_range_indicator/systems.rs represents the wizard's Y coordinate used in the range-projection calculation. It is undocumented and does not match any constant in wizard/constants.rs. Is this the actual wizard spawn height, a camera plane value, or an approximation? If the wizard's Y-coordinate ever changes this constant will silently produce a wrong indicator radius.
- **[wizard-root]** apply_wizard_stats_to_primed_spell and reset_empowerment_after_cast both query With<Wizard> and would match GuestWizard. Does the GuestWizard entity have PrimedSpell inserted on the host? If so, is the empowerment reset lifecycle correct for the host-driven (message-based) guest cast flow?
- **[wizard-archetypes]** Is the Psychopath archetype intentionally disabled for multiplayer permanently, or just deferred? The plugin only hooks AppState::InGame; a comment or explicit run_if(!is_multiplayer) guard would make intent clear.
- **[wizard-archetypes]** meteorologist/constants.rs exports weather bar UI colors (STORM_COLOR, BLIZZARD_COLOR, DROUGHT_COLOR) consumed by the UI layer. Should these live in the UI module that uses them rather than the simulation constants file?
- **[wizard-archetypes]** The drain-and-requeue pattern in swordcerer/networking.rs and meteorologist/networking.rs is informal protocol multiplexing. Is there a plan to centralize message routing, or is ad-hoc drain acceptable long-term as the network protocol grows?
- **[wizard-talents]** Is Spell::Fireball => [1, 2, 3] intentional? It unlocks all three talent tiers on the very first battle while every other damage spell requires tens to thousands of uses.
- **[wizard-talents]** Is the TalentDefinition.implemented field still serving any undocumented purpose (e.g. driving a UI lock state that was removed), or is it fully orphaned?
- **[wizard-talents]** Are set_selection and has_talent on ActiveTalents placeholders for a planned talent-selection UI flow, or can they be removed?
- **[wizard-talents]** Should duplicate talent names (Chain Reaction, Dimensional Rift, Scorched Earth, etc.) be globally unique to support future features like achievement text or analytics keyed on talent names?
- **[config]** VsyncMode::Adaptive intent: was this planned to map to FifoRelaxed/Mailbox and never finished, or are On and Adaptive intentionally identical on this Bevy version? If the latter, the variant should be removed with a serde alias migration.
- **[config]** saved_trees / saved_ponds / saved_bushes / saved_boulders in GameConfig: these are #[serde(skip)] and absent from WizardSave's top-level fields — they are only populated via load_level_terrain_into_config. Are they always transient in-memory, reloaded from terrain_per_level on each session start? If so, are they properly populated in non-Endless mode?
- **[config]** progress.rs / ProgressData / SignedProgress: this infrastructure now appears only used by migrate_very_old_progress (the very-old single-save path). Is load_verified_progress called from anywhere else, or is progress.rs purely a migration artifact that could be folded into save_migration.rs?
- **[config]** resources.rs DTO/resource conflation: was there an intention to move WindowConfig, AudioConfig, and ConfigFile into a dedicated config_file.rs to separate serialization DTOs from the live Bevy resource GameConfig?
- **[multiplayer]** apply_status_to_entity line 691: the Polymorph failure path inserts SleepModifier as a fallback when polymorph_capture is None (unit already polymorphed). Is silently sleeping an already-sheep unit the intended behavior, or should this be a no-op?
- **[multiplayer]** receive_spell_visual_snapshot and receive_crdt_snapshot both drain incoming_unreliable and re-queue by type prefix. Both run in the same Update frame. Is there a scheduling guarantee preventing them from both draining the same packet? (They filter on different prefix bytes so it should be safe, but the ordering is implicit.)
- **[multiplayer]** coop_pause_heartbeat runs with run_if(coop_sync_pause_enabled) which checks for a live MultiplayerSession. Is there a risk of one extra heartbeat firing after cleanup_mp_game removes the session but before the run condition is re-evaluated?
- **[spell-arcane_crystal]** crystal_black_hole_interaction ghost intent (F3): Should a local black hole visually pull the ghost of the remote peer's crystal? If yes, move this system to the visual block. If no, add the Without<GhostSpellEffect> filter.

---

## DEFERRED: F133 dispel forwarding bug — full diagnosis (needs live co-op smoke-test)

**Why deferred from the P0 batch:** unlike the other 5 ghost-gating fixes (pure query-filter
additions), this one touches the netcode broadcast path and cannot be applied as a one-liner.
Shipping it blind risks a wire feedback-loop. It needs a 2-client co-op smoke-test to validate.

**Root cause:** ghost spell effects on the guest are spawned in `spell_sync.rs:824` with only
`GhostSpellEffect` + `NetworkEntityId` — NOT `NetworkedSpellEffect` (which carries `kind`).
`forward_dispel_impacts_to_host` (`dispel/bolt.rs:606`) queries `&NetworkedSpellEffect`, so it
sees zero ghost effects → guest-cast Dispel never forwards to the host.

**The correct fix is TWO coordinated changes (not one):**
1. `spell_sync.rs:824` — also insert `NetworkedSpellEffect { kind }` on the ghost, converting
   `effect.kind` (u8) → `SpellEffectKind`. (Confirm a u8→enum conversion exists/derive one.)
2. `spell_sync.rs:114` `collect_spell_effect_snapshots` — add `Without<GhostSpellEffect>` to the
   `effects` query. **This is mandatory**: that system runs on BOTH peers (`run_if(both_peers)`)
   and currently has no ghost filter, so step 1 alone would make the guest re-broadcast the
   host's mirrored effects back to the host (feedback loop / duplicate effects).

**Validation:** host casts a dispellable zone (e.g. Wall of Fire); guest casts Dispel over it;
confirm the zone is stripped AND no effect duplication/wire spam occurs on either peer.

---

## P0 remediation log (2026-06-10) — RESOLVED

Fixed and gated (cargo check/clippy/test/native-build all green; **needs your 2-client co-op smoke-test**):

**Audited P0 (5 of 6):**
- ✅ F446 `battlefield/systems.rs` `apply_lava_damage` — `Without<GhostEntity>`
- ✅ F162 `healing_plume/casting.rs` `apply_healing_plume_heal` — `Without<GhostEntity>`
- ✅ F163 `mind_control/casting.rs` `handle_mind_control_casting` (+ 2 helper sigs) — `Without<GhostEntity>`
- ✅ F164 `mind_control/effects.rs` `update_mass_hysteria_targeting` (both queries) — `Without<GhostEntity>`
- ✅ F109 `squall/shards.rs` `update_ice_explosions` — `Without<GhostSpellEffect>`
- ⏸️ F133 `dispel/bolt.rs` — DEFERRED (see diagnosis above; 2-part netcode change, needs smoke-test)

**Cold-review completeness additions (same bug class, same files, found by the gate's Staff Engineer review):**
- ✅ `mind_control/effects.rs` `update_traitors_mark_aura` `enemies` query (inserts `Demoralized`) — `Without<GhostEntity>`
- ✅ `mind_control/effects.rs` `confused_combat_attack` + `tick_amnesia_effect` `potential_targets` (mutate `Health`) — `Without<GhostEntity>`
- ✅ `mind_control/effects.rs` `tick_sleeper_agent` `potential_targets` (mutates `Health`) — `Without<GhostEntity>`

**OPEN QUESTION for smoke-test (deliberately NOT changed — would alter effect-tick semantics):**
The *driver* queries that tick the effect timers (`amnesia_query`, `agent_query`/`SleeperAgentActive`,
`aura_query`/`TraitorsMarkAura`, `tick_mass_hysteria`'s `hysteria_query`) were left ungated. If the host
replicates these status components onto ghost units, a ghost could locally tick down / act. Recipient
queries are now gated (no ghost gets damaged/demoralized), but confirm in co-op whether a ghost ever
*carries* these driver components. Pre-existing minor `apply_water_slow` (`battlefield/systems.rs`,
`RoughTerrainModifier` on ghosts) is logged for a later wave.

---

## Wave A progress log (2026-06-10) — partial

Cold-reviewed *ship as-is*. Gate green (check/clippy/test/native build). Needs your smoke-test.

**F133 dispel fix — RESOLVED** (host-side authority approach, not the 2-part option originally
diagnosed — cleaner, zero blast radius): guest forwards in-radius `GhostSpellEffect` ghosts by
`NetworkEntityId`; host's `receive_dispel_messages` validates `is_dispellable(kind)` before despawn.

**Wave A foundation — DONE:**
- ✅ F010 `game/plugin.rs` — `GlobalAttackCycle` extracted to new `game/attack_cycle.rs`; 10 refs updated.
- ✅ F011 `game/plugin.rs` — `apply_game_speed`/`auto_pause_on_focus_loss` → `shared_systems.rs`;
  debug-hitbox types+systems → `debug_ui.rs` (now self-registered in `DebugUiPlugin`).
  `game/plugin.rs` is now registration-only (449→~285 LOC).
- ✅ F012 `shared_systems.rs` — inline `ENGAGEMENT_RANGE` → `constants::DEFENDER_ACTIVATION_RANGE`.
- ✅ F022 `spell_range_indicator/systems.rs` — duplicate `BASE_RADIUS`/`BASE_HEIGHT` hoisted to module consts.
- ✅ F023 `spell_enum.rs` — removed stale `#[allow(dead_code)]` on `Spell::category` (used 4×).
- ✅ F018 (partial) — debug-hitbox systems gained their `run_if`; `toggle_debug_ui_visible` still unguarded (Low).

**Wave A remaining (~26):** mostly Low/S UI const-naming in `ui/layout_helpers.rs` & `ui/constants.rs`
(F272/F275/F276/F279/F280), re-export/marker tidy (F277/F278), small wizard/config items
(F024/F025/F027/F046/F051), boss-utils dead/single-use (F202/F203). **Deferred to Wave B** (so files
aren't edited twice): F001 (`units/systems.rs` split), F020/F021 (wizard root splits), F044
(`save_data.rs` split). **Deferred to Wave C:** intentional `#[allow(dead_code)]` reviews incl. F016
`is_ray_level` (looks deliberate — maintainer open-question on dead-code intent).

---

## Wave A completion log (2026-06-10) — batch 2 (cold-reviewed: ship)

Additional Wave A items done (gate green: check/clippy/test/native build):
- ✅ F272 `ui/constants.rs` — renamed misnamed `GOLD_ACCENT` (value is purple) → `ACTIVE_ACCENT` (10 refs/5 files).
- ✅ F276/F279/F280 `ui/layout_helpers.rs` — inline shadow/dot-leader/slider-track colors → named consts in `ui/constants.rs`.
- ✅ F278 — moved `ParchmentPanel`/`FrostedGlassOverlay` markers `button_systems.rs` → `layout_helpers.rs` (where they're used).
- ✅ F202 `boss/utils.rs` — narrowed `EYE_SHEET_WIDTH/HEIGHT` visibility to private (only used locally).
- ✅ F015 `game/constants.rs` — removed stale stacked doc block on `calculate_defender_grid_position`.
- ✅ F024 `spell_range_indicator/systems.rs` — `.iter().next()` → `.single()`/`.single_mut()`.

### Wave A final tally: 14 of 32 resolved. Remainder deliberately routed (not skipped):

**→ Wave B (would edit a file Wave B splits — avoid double-touch):**
F001/F003/F005/F008 (`units/systems.rs`), F020/F021 (wizard root splits), F044 (`save_data.rs`),
F004/F006 (`units/components.rs`), F027 (`wizard_state.rs`).

**→ Wave C (intentional / dead-code-intent — needs maintainer call, not unilateral removal):**
F016 `is_ray_level` (deliberate `#[allow]`, symmetry with other boss predicates),
F017 `WaveSpawnedMessage` (written-not-read; possible telemetry hook),
F046 `input_bindings` allows (dormant rebinding feature, partially wired),
F277 `ui/systems.rs` re-export hub (deliberate "Phase 16 split" hub used by 10+ files),
F203 `is_on_screen` (single-use but a legit reusable camera util).

**→ Maintainer design questions (behavior/visual change — should not be done blind):**
F275 slider `%` display (a 3.0× multiplier shown as "300%" is a legitimate representation, not
clearly a bug — needs a design call + visual check),
F025 `spawn_status_effects_section` (UI-spawning fn in the game layer — cross-layer move, risky),
F051 `load_and_apply_config` field-copy (behavior-sensitive config refactor).

---

## Wave B progress (2026-06-11)

**Batch 1 — DONE (8 files split, gate green, v0.10.15).** Path-preserving `X.rs`→`X/` pattern.
LOC-integrity verified (all deltas positive = boilerplate only, no dropped code):
interaction.rs(1969), panels.rs(1403), roguelite_tab.rs(1682), compendium/setup.rs(1435),
settings/builders.rs(1650), achievements/checks.rs(1062), arcane_crystal/auto.rs(1039),
cauldron/systems.rs(892). Post-split fixups: promoted re-exported items to `pub(crate)`,
added `rand::Rng`/`default_graph_offset` imports, `cargo fix` cleaned unused imports.

**Batch 2 — DONE (8 files split, gate green).** hags/core(1120), hags/abilities(1024),
dark_mage/spells(811), grease/ignite(784), tutorial/lifecycle(781), in_game/bars(780),
finger_of_death/effects(753), fireball/projectile(745). LOC-integrity all positive.
Fixups: promoted re-exports + component types to pub(crate); fixed wrong Wizard import path
(units::components -> units::wizard::components) + missing Team imports + grease super-depth +
rand::Rng imports; renamed 3 module_inception siblings (projectile/movement, ignite/burn,
bars/resource_bars). cargo fix cleaned unused imports. Note: gameplay files need more import
fixups than UI files (heavier cross-module type deps).

**Batch 3 — DONE (8 files split, gate green).** layout/setup(852), spike_growth/systems(790),
polymorph/systems(757), arcane_crystal/setup(750), disintegrate/beam(742), gunslinger/fire(739),
ogre/charge(732), in_game/spawn(726). LOC-integrity all positive. Hardened agent instructions
cut fixups from ~90 errors (batch 2) to 11. Fixups: promoter + narrow 2 over-wide re-exports +
Ordering/get_unlocked_spells imports.

**Batch 4 — DONE (8 files split, gate green).** finger_of_death/casting(734), teleport/casting(703),
healer/systems(691), vfx/cast_effects(690), telekinesis/systems(688), disintegrate/casting(677),
vfx/area_effects(665), wall_of_stone/lifecycle(657). LOC-integrity positive. Fixups: promoter,
super-depth fix (finger_of_death/casting siblings), narrowed over-wide re-exports, renamed
teleport/casting inception module (casting->cast_input).

**Batch 5 — DONE (8 files split, gate green).** berserker_rage/systems(656), dispeller/systems(640),
meteor_fall/meteor(636), mark_of_death/systems(635), teleporter/systems(627), compendium/rows(625),
swordcerer/combat(619), button_systems(640). LOC-integrity positive. Fixups: promoter + narrowed
over-wide re-exports + promoted glob-reexported compendium/rows fns to pub(crate). **40/110 files done.**

**Batch 6 — DONE (8 files, gate green).** banishment/systems(613), action_bar/systems(603),
dark_mage/ai(587), king/systems(587), fog_cloud/systems(584), cauldron_menu/setup(580),
raise_the_dead/casting(577), lich/combat(564). Fixups: promoter + lich glob-fns promoted +
king game-constants/Corpse imports + ButtonColors import + banishment vfx super-depth. **48/110 done.**

**Batch 7 — DONE (8 files, gate green).** ogre/combat(548), game_over/screen(548),
guardian_circle/systems(540), wall_of_fire/casting(537), meteor_fall/casting(532),
infantry/systems(532), entangle/casting(531), terrain/boulder/systems(524). Fixups: promoter +
infantry game-constants/FlowFieldVelocity + ogre CombatAnimation/Health/FlowFieldVelocity-path +
renamed entangle/casting inception. **56/110 done.**

**Batch 8 — DONE (8 files, gate green).** sleep/systems(518), pathfinding/runtime(499),
magic_missile/casting(492), shielder/systems(488), wall_of_fire/damage(480), teleport/arrival(475),
pathfinding/setup(472), chain_lightning/casting(471). Cleanest batch yet (0 re-export errors;
only 1 private-type promotion: WallOfFireSfx). **64/110 done.**

**Batch 9 — DONE (8 files, gate green).** pond/systems(454), archer/movement(453),
rune_display/systems(447), black_hole/gravity(446), game_mode/components(444), haste/systems(441),
wizard_state(440, was deferred F020), battle_hymn/systems(440). Fixup: 1 missing constant import.
**72/110 done.**

**Batch 10 — DONE (8 files, gate green).** plague_wind/cloud(435), grease/casting(433),
wall_of_stone/casting(431), archer/combat(418), wizard/systems(414, was deferred F021),
pause_menu/main/systems(408), aerialist/systems(404), gamepad/systems(393). Near-clean
(1 re-export fix). **80/110 done.**

**Batch 11 — DONE (10 files, gate green).** loading/init(391), achievements/resources(369),
wizard_tower/components(363), cauldron_menu/interaction(357), pathfinding/debug(356),
settings/interaction(353), dispel/casting(340), brute/systems(326), combat_systems/post_combat(321),
wizard_tower/constants(313). Fixups: promoted wizard_tower constants, narrowed pathfinding/debug
re-exports, promoted StudyAllocAdjustButton.target field. **90/110 done — all SAFE splits complete.**

### Wave B status: 90/110 oversized files split. Remaining ~20 are the DEFERRED high-risk set
(NOT bulk-split — need individual careful handling): MP wire-format (spell_sync, guest_snapshot,
host_systems, multiplayer/systems, multiplayer/plugin), save_data.rs (back-compat), units/systems.rs
+ units/components.rs (dedup-coupled), boss/ray/*, dispel/bolt.rs + squall/shards + healing_plume +
mind_control/casting (edited for P0/F133), spells/utils.rs + shared_systems.rs + game/constants.rs +
movement_systems.rs + config/resources.rs + combat_systems/melee.rs (cross-cutting/load-bearing),
crt_effect/* (shader, load-bearing per memory), game/plugin.rs + wizard_tower/plugin.rs (registration).

---

## Wave C progress (2026-06-11)

**DONE:** Removed unused direct deps `anyhow` + `serde_json` (0 usages in src/; cargo machete clean).

**Dependency CVEs — diagnosed, deferred (require breaking direct-dep upgrades, must be tested):**
- `steamworks 0.12.2` (RUSTSEC-2026-0121, P2P-auth DoS → ≥0.13.1): transitively pinned by
  `bevy-steamworks 0.16`. Fix needs a newer bevy-steamworks (Bevy-version-coupled) — not a safe
  `cargo update`. Deferred to a deliberate, tested dep upgrade.
- `hickory-proto`/`hickory-net 0.26.0-beta.4` (RUSTSEC-2026-0119/closest-encloser → ≥0.26.1):
  transitively pinned by `iroh 0.98` (the MP netcode backbone). Bumping risks MP wire-compat; needs
  an iroh upgrade + co-op smoke-test. Deferred.
- Unmaintained (informational, no fix needed now): `bincode 1.x` (save-format risk — see memory),
  `paste`, `atomic-polyfill` (both transitive).

**Dead-code (`#[allow(dead_code)]` ×65) — needs maintainer intent, NOT swept unilaterally:**
Many are deliberate (future API surface, debug-only, boss-predicate symmetry like `is_ray_level`).
The safe sub-action — removing *stale* allows where the item is actually used — was done where found
(e.g. F023 Spell::category). A full sweep is an open question for the maintainer (do you want the
genuinely-unused items deleted, or are they intentional API-for-later?).

### Wave C dependency CVE bump — INVESTIGATED, blocked upstream (2026-06-11)
Checked crate versions (authoritative source). Neither CVE is safely bumpable:
- **steamworks 0.12.2 → 0.13.1:** BLOCKED. `bevy-steamworks 0.16.0` is the latest published release
  and hard-depends on steamworks 0.12.2. No bevy-steamworks version uses 0.13. A [patch] override
  would break bevy-steamworks's 0.12-era API usage. Needs upstream bevy-steamworks release.
- **hickory 0.26.0-beta.4 → 0.26.1:** BLOCKED within range. `iroh 0.98.1` hard-pins
  `hickory-resolver = "=0.26.0-beta.4"` (exact). Only `iroh 1.0.0-rc.1` (major breaking RC) changes
  it — bumping the P2P netcode backbone to an RC risks wire-compat and can't be verified without a
  2-client smoke-test. Deferred to a deliberate, tested iroh 1.0 upgrade.
Both remain documented for the maintainer; not force-bumped to honor "make sure nothing breaks".

### Wave C dead-code sweep — DONE (2026-06-11)
Compiler-driven (stripped the 62 `#[allow(dead_code)]`, let rustc report genuine dead code):
- **Removed 12 genuinely-dead methods/functions** (active_recipe, is_ray_level, iter, obstacle_bounds×2,
  fire_interval, is_hold_to_fire, ammo_for, ammo_for_mut, set_selection, has_talent + a few).
- **Removed ~7 stale allows** (items that were actually used — the attribute was a lie).
- **Suppressed 46 intentional/cascade-risky dead items** with `#[allow(dead_code)]`: the 18 wire-format
  flag constants in `networking/protocol.rs` (module-level allow — protocol surface), set-for-completeness
  data fields (`damage_type`, `*_corpse_materials`, `cursor_position`, etc. — removing cascades into
  constructors), and never-constructed enum variants.
- **Restored `get_multiplier`** (test-only API) after the cfg(test) gotcha: the default `cargo check`
  dead-scan misses test usages, so it looked dead but a unit test uses it.
- Gate: bin `clippy -D warnings` clean + test 52/52 + native build. (Note: a pre-existing
  `--all-targets`-only clippy lint in `damage.rs` test code is unrelated and left as-is.)
