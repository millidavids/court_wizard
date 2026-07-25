use std::cmp::Ordering;

use super::super::super::super::components::{
    CastingState, Mana, PrimedSpell, Wizard, WizardInput,
};
use super::super::components::{
    ContagiousBaas, DireSheep, ExplosiveSheep, PermanentLivestock, PigForm, PolymorphTalentParams,
};
use super::super::constants;
use super::super::sheep_visual::SheepBounce;
use super::shared::apply_sheep_visual;
use crate::game::components::Billboard;
use crate::game::units::components::{AttackTiming, Corpse, Health, PolymorphedModifier, Team};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Applies the polymorph effect to a single target entity.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_polymorph_to_target(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    target_entity: Entity,
    target_health: &Health,
    target_material: &MeshMaterial3d<StandardMaterial>,
    target_mesh: &Mesh3d,
    target_team: Team,
    duration: f32,
    talent_params: &PolymorphTalentParams,
    empowerment: f32,
    position: Vec3,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) {
    vfx::systems::spawn_smoke_poof_synced(
        commands,
        visual_assets,
        pending,
        &visual_assets.polymorph_poof,
        crate::networking::snapshot::PoofVariant::Polymorph,
        position,
        8,
        time_secs,
    );
    let (sheep_hp, color) = if talent_params.dire {
        (constants::DIRE_SHEEP_HP, constants::DIRE_SHEEP_COLOR)
    } else if talent_params.pig_form {
        (talent_params.sheep_hp, constants::PIG_COLOR)
    } else {
        (talent_params.sheep_hp, constants::SHEEP_COLOR)
    };

    let mut entity_cmds = commands.entity(target_entity);
    entity_cmds.insert((
        PolymorphedModifier::new(
            duration,
            target_health.current,
            target_health.max,
            target_material.0.clone(),
            target_mesh.0.clone(),
            target_team,
        ),
        Health::new(sheep_hp),
        SheepBounce {
            base_y: position.y,
            elapsed: 0.0,
        },
        Billboard,
    ));
    entity_cmds.remove::<AttackTiming>();
    apply_sheep_visual(&mut entity_cmds, materials, visual_assets, color);

    // Insert talent-specific behavioral components
    if talent_params.explosive {
        entity_cmds.insert(ExplosiveSheep);
    }
    if talent_params.contagious {
        // Spread targets also get ContagiousBaas so it keeps jumping
        entity_cmds.insert(ContagiousBaas {
            empowerment,
            talent_params: *talent_params,
        });
    }
    if talent_params.pig_form {
        entity_cmds.insert(PigForm);
    }
    if talent_params.permanent {
        entity_cmds.insert(PermanentLivestock);
    }
    if talent_params.dire {
        entity_cmds.insert((DireSheep::new(), Team::Defenders, AttackTiming::new()));
    }
}

/// Core polymorph casting logic. Returns the number of enemies polymorphed (0 if cancelled/failed).
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(crate) fn polymorph_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    targets_query: &Query<
        (
            Entity,
            &Transform,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
            &Mesh3d,
            &Team,
        ),
        (
            Without<Corpse>,
            Without<PolymorphedModifier>,
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    talent_params: &PolymorphTalentParams,
    visual_assets: &SpellVisualAssets,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) -> u32 {
    let time_secs = time.elapsed_secs();
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return 0;
    }

    let mut polymorphed_count = 0;
    let cast_time = primed_spell.cast_time * talent_params.cast_time_mult;
    let mana_cost = if talent_params.mass {
        constants::MANA_COST * constants::MASS_POLYMORPH_MANA_MULT
    } else {
        constants::MANA_COST
    };

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(mana_cost) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if casting_state.is_complete(cast_time) {
                if mana.consume(mana_cost)
                    && let Some(cursor_pos) = input.cursor_pos
                {
                    let duration = talent_params.duration * primed_spell.empowerment;

                    if talent_params.mass {
                        // Mass Polymorph: collect target entity IDs first, then apply
                        let target_entities: Vec<Entity> = targets_query
                            .iter()
                            .filter(|(_, transform, _, _, _, _)| {
                                transform.translation.distance(cursor_pos)
                                    <= constants::MASS_POLYMORPH_RADIUS
                            })
                            .map(|(entity, _, _, _, _, _)| entity)
                            .collect();

                        for entity in &target_entities {
                            if let Ok((_, transform, health, material, mesh, team)) =
                                targets_query.get(*entity)
                            {
                                apply_polymorph_to_target(
                                    commands,
                                    materials,
                                    *entity,
                                    health,
                                    material,
                                    mesh,
                                    *team,
                                    duration,
                                    talent_params,
                                    primed_spell.empowerment,
                                    transform.translation,
                                    visual_assets,
                                    time_secs,
                                    pending,
                                );
                                polymorphed_count += 1;
                            }
                        }
                    } else {
                        // Single target: find nearest enemy in radius
                        if let Some((
                            target_entity,
                            _,
                            target_transform,
                            target_health,
                            target_material,
                            target_mesh,
                            target_team,
                        )) = targets_query
                            .iter()
                            .filter_map(|(entity, transform, health, material, mesh, team)| {
                                let dist = transform.translation.distance(cursor_pos);
                                if dist <= constants::TARGET_SEARCH_RADIUS {
                                    Some((entity, dist, transform, health, material, mesh, team))
                                } else {
                                    None
                                }
                            })
                            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
                        {
                            apply_polymorph_to_target(
                                commands,
                                materials,
                                target_entity,
                                target_health,
                                target_material,
                                target_mesh,
                                *target_team,
                                duration,
                                talent_params,
                                primed_spell.empowerment,
                                target_transform.translation,
                                visual_assets,
                                time_secs,
                                pending,
                            );
                            polymorphed_count += 1;
                        }
                    }
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    polymorphed_count
}
