//! Mind control casting: input handling and target selection.

use std::cmp::Ordering;

use bevy::prelude::*;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard,
};
use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{Corpse, MindControlled, Team};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::networking::snapshot::SpellSoundId;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_cursor_to_spell_range_with_origin, cleanup_spell_caster, ground_projected_range,
    spawn_circle_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> MindControlTalentParams {
    let mut params = MindControlTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::MindControl, 0);
    let t2 = talents.get_selection(Spell::MindControl, 1);
    let t3 = talents.get_selection(Spell::MindControl, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => params.duration_mult = constants::IRON_WILL_DURATION_MULT,
        Some(1) => params.damage_multiplier = constants::DEEP_DOMINATION_DAMAGE_MULT,
        Some(2) => params.cast_time_mult = constants::QUICK_SUBJUGATION_CAST_MULT,
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.puppet_master = true,
        Some(1) => params.traitors_mark = true,
        Some(2) => params.amnesia = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.dominate = true,
        Some(1) => params.mass_hysteria = true,
        Some(2) => params.sleeper_agent = true,
        _ => {}
    }

    params
}

/// Tracked highlight target — stored as system-local state so we have a single
/// source of truth that doesn't depend on deferred command timing.
/// Stores the entity, its original material handle (to restore), and the cloned
/// tinted handle (to remove from the asset store on cleanup).
#[derive(Default)]
pub(super) struct HighlightState {
    target: Option<HighlightedUnit>,
}

struct HighlightedUnit {
    entity: Entity,
    /// The entity's original shared material handle — restored on un-highlight.
    original_handle: Handle<StandardMaterial>,
    /// The cloned tinted material we created — removed from assets on cleanup.
    tinted_handle: Handle<StandardMaterial>,
}

/// Local wizard mind control casting — hold-to-cast with dynamic target highlighting.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_mind_control_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut assets: (ResMut<Assets<StandardMaterial>>, ResMut<Assets<Mesh>>),
    mut wizard_query: Query<
        (
            Entity,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
            Option<&MindControlCooldown>,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    cursor_resources: (
        Res<CorrectedCursorPosition>,
        Res<TargetAssistWorldPos>,
        Res<LocalSpellOrigin>,
    ),
    enemies_query: Query<
        (Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>),
        (
            Without<Corpse>,
            Without<MindControlled>,
            Without<Boss>,
            Without<MassHysteriaTarget>,
        ),
    >,
    existing_controlled: Query<&MindControlled>,
    existing_dominated: Query<Entity, With<DominatedUnit>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    mut highlight: Local<HighlightState>,
    mut loaded_assets: (
        Res<SpellSfxAssets>,
        Res<SpellVisualAssets>,
        Res<GameConfig>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
    talent_resources: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
    ),
) {
    let (ref mut materials, ref mut meshes) = assets;
    let (ref sfx, ref visual_assets, ref game_config, ref mut pending_cast_events) = loaded_assets;
    let (active_talents, mut talent_progress) = talent_resources;
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);
    let raw_cursor_pos = input.cursor_pos;

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell, cooldown)) =
        wizard_query.single_mut()
    else {
        return;
    };
    let spell_range = ground_projected_range(wizard.spell_range, local_origin.0.y);
    let cursor_pos = clamp_cursor_to_spell_range_with_origin(
        raw_cursor_pos,
        local_origin.0,
        wizard.spell_range,
        0.0,
    );
    if primed_spell.spell != Spell::MindControl {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let controlled_count = existing_controlled.iter().count() as u32;

    // Determine max controlled based on talents
    let max_controlled = if talent_params.dominate {
        // Dominate: only 1 permanent unit at a time
        if existing_dominated.iter().next().is_some() {
            0 // Can't dominate another while one exists
        } else {
            1
        }
    } else if talent_params.puppet_master {
        constants::PUPPET_MASTER_MAX
    } else {
        constants::MAX_CONTROLLED
    };

    // Mass Hysteria doesn't use the normal MC flow
    let mana_cost = if talent_params.mass_hysteria {
        constants::MANA_COST * constants::MASS_HYSTERIA_MANA_MULT
    } else {
        constants::MANA_COST
    };

    let cast_time = primed_spell.cast_time * talent_params.cast_time_mult;

    // On release → cancel cast, remove highlight, and despawn indicator
    if input.just_released {
        cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
        casting_state.cancel();
        clear_highlight(&mut commands, materials, &mut highlight);
        return;
    }

    match *casting_state {
        CastingState::Resting => {
            let can_cast = if talent_params.mass_hysteria {
                // Mass Hysteria doesn't check controlled count
                mana.can_afford(mana_cost) && !cooldown.is_some_and(|cd| cd.remaining > 0.0)
            } else {
                mana.can_afford(mana_cost)
                    && !cooldown.is_some_and(|cd| cd.remaining > 0.0)
                    && controlled_count < max_controlled
            };

            if (input.just_pressed || input.pressed) && can_cast {
                if talent_params.mass_hysteria
                    && let Some(pos) = cursor_pos
                {
                    let circle_entity = spawn_circle_indicator(
                        &mut commands,
                        meshes,
                        visual_assets.mind_control_indicator.clone(),
                        pos,
                        constants::MASS_HYSTERIA_RADIUS,
                    )
                    .id();
                    commands
                        .entity(wizard_entity)
                        .insert(SpellCaster::with_indicator(circle_entity));
                }
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if talent_params.mass_hysteria {
                // Update indicator position to follow cursor
                if let Some(pos) = cursor_pos {
                    update_indicator_position(
                        wizard_entity,
                        pos,
                        &caster_query,
                        &mut indicator_query,
                    );
                }
            } else {
                // Find nearest enemy to cursor each frame and update highlight
                let nearest =
                    find_nearest_enemy(&enemies_query, cursor_pos, spell_range, local_origin.0);
                update_highlight(
                    &mut commands,
                    materials,
                    &enemies_query,
                    &mut highlight,
                    nearest,
                );
            }

            if casting_state.is_complete(cast_time) {
                if mana.consume(mana_cost) {
                    vfx::systems::spawn_school_flare_synced(
                        &mut commands,
                        visual_assets,
                        pending_cast_events,
                        local_origin.0,
                        vfx::systems::SpellSchool::Dark,
                        time.elapsed_secs(),
                    );
                    if talent_params.mass_hysteria {
                        // Mass Hysteria: apply chaos to all enemies in radius
                        if let Some(pos) = cursor_pos {
                            let mut count = 0u32;
                            for (entity, transform, team, _) in &enemies_query {
                                if !matches!(*team, Team::Attackers | Team::Undead) {
                                    continue;
                                }
                                if crate::game::units::wizard::spells::utils::xz_distance(
                                    transform.translation,
                                    pos,
                                ) <= constants::MASS_HYSTERIA_RADIUS
                                {
                                    commands.entity(entity).insert(MassHysteriaTarget {
                                        time_remaining: constants::MASS_HYSTERIA_DURATION
                                            * talent_params.duration_mult,
                                    });
                                    count += 1;
                                }
                            }
                            if count > 0 {
                                audio::play_sfx_synced(
                                    &mut commands,
                                    pending_cast_events,
                                    SpellSoundId::MindControlCast,
                                    pos,
                                    game_config,
                                    sfx,
                                );
                                if let Some(ref mut progress) = talent_progress {
                                    progress.increment(Spell::MindControl, count);
                                }
                            }
                        }
                    } else if let Some(ref highlighted) = highlight.target {
                        // Normal single-target MC
                        if let Ok((_, target_transform, _, _)) =
                            enemies_query.get(highlighted.entity)
                        {
                            audio::play_sfx_synced(
                                &mut commands,
                                pending_cast_events,
                                SpellSoundId::MindControlCast,
                                target_transform.translation,
                                game_config,
                                sfx,
                            );
                        }

                        let wear_off = if talent_params.dominate {
                            f32::MAX // Permanent
                        } else {
                            constants::EFFECT_WEAR_OFF_DURATION * talent_params.duration_mult
                        };

                        let mut entity_cmds = commands.entity(highlighted.entity);
                        entity_cmds.insert(MindControlled {
                            time_elapsed: 0.0,
                            wear_off_duration: wear_off,
                            original_spawn_pos: None,
                            damage_multiplier: talent_params.damage_multiplier,
                        });

                        // Insert talent-specific behavioral components
                        if talent_params.traitors_mark {
                            entity_cmds.insert(TraitorsMarkAura);
                        }
                        if talent_params.amnesia {
                            entity_cmds.insert(AmnesiaOnExpiry);
                        }
                        if talent_params.dominate {
                            entity_cmds.insert(DominatedUnit);
                        }
                        if talent_params.sleeper_agent {
                            entity_cmds.insert(SleeperAgentPending);
                        }

                        commands.entity(wizard_entity).insert(MindControlCooldown {
                            remaining: constants::COOLDOWN,
                        });

                        mouse_state.left_consumed = true;

                        if let Some(ref mut progress) = talent_progress {
                            progress.increment(Spell::MindControl, 1);
                        }
                    }
                }

                cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
                clear_highlight(&mut commands, materials, &mut highlight);
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
            casting_state.cancel();
            clear_highlight(&mut commands, materials, &mut highlight);
        }
    }
}

/// Finds the nearest enemy to the cursor within TARGET_SEARCH_RADIUS and spell range.
/// Bosses are excluded from mind control targeting.
fn find_nearest_enemy(
    enemies_query: &Query<
        (Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>),
        (
            Without<Corpse>,
            Without<MindControlled>,
            Without<Boss>,
            Without<MassHysteriaTarget>,
        ),
    >,
    cursor_pos: Option<Vec3>,
    spell_range: f32,
    local_origin: Vec3,
) -> Option<Entity> {
    let wizard_pos = local_origin;
    cursor_pos.and_then(|pos| {
        enemies_query
            .iter()
            .filter(|(_, _, team, _)| **team == Team::Attackers || **team == Team::Undead)
            .filter(|(_, transform, _, _)| {
                let dx = transform.translation.x - wizard_pos.x;
                let dz = transform.translation.z - wizard_pos.z;
                (dx * dx + dz * dz).sqrt() <= spell_range
            })
            .filter_map(|(entity, transform, _, _)| {
                let dist = crate::game::units::wizard::spells::utils::xz_distance(
                    transform.translation,
                    pos,
                );
                if dist <= constants::TARGET_SEARCH_RADIUS {
                    Some((entity, dist))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
            .map(|(entity, _)| entity)
    })
}

/// Updates the highlight to point at a new target (or clears if None).
/// Clones the material for the highlighted entity so only it gets tinted.
fn update_highlight(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    enemies_query: &Query<
        (Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>),
        (
            Without<Corpse>,
            Without<MindControlled>,
            Without<Boss>,
            Without<MassHysteriaTarget>,
        ),
    >,
    highlight: &mut HighlightState,
    nearest: Option<Entity>,
) {
    let current_entity = highlight.target.as_ref().map(|h| h.entity);

    // If the target hasn't changed, nothing to do
    if nearest == current_entity {
        return;
    }

    // Restore old target's original material
    clear_highlight(commands, materials, highlight);

    // Clone + tint new target's material
    if let Some(target_entity) = nearest
        && let Ok((_, _, _, material_handle)) = enemies_query.get(target_entity)
    {
        let original_handle = material_handle.0.clone();
        if let Some(original_mat) = materials.get(&original_handle) {
            let mut tinted_mat = original_mat.clone();
            let base_linear = tinted_mat.base_color.to_linear();
            let highlight_linear = constants::HIGHLIGHT_COLOR.to_linear();
            let blended = base_linear.mix(&highlight_linear, 0.6);
            tinted_mat.base_color = Color::from(blended);
            let tinted_handle = materials.add(tinted_mat);

            // Swap the entity's material to the tinted clone
            commands
                .entity(target_entity)
                .insert(MeshMaterial3d(tinted_handle.clone()));

            highlight.target = Some(HighlightedUnit {
                entity: target_entity,
                original_handle,
                tinted_handle,
            });
        }
    }
}

/// Restores the highlighted entity's original material and cleans up the tinted clone.
fn clear_highlight(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    highlight: &mut HighlightState,
) {
    if let Some(highlighted) = highlight.target.take() {
        // Restore the entity's original shared material
        commands
            .entity(highlighted.entity)
            .insert(MeshMaterial3d(highlighted.original_handle));

        // Remove the tinted clone from assets
        materials.remove(&highlighted.tinted_handle);
    }
}
