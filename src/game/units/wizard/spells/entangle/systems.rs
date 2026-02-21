use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, SpellCaster, LocalWizard, Wizard};
use super::components::{EntangleGroundEffect, EntangleIndicator};
use super::constants;
use crate::game::achievements::messages::EntangleHitDefenderMessage;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{RootedModifier, Team};

#[allow(clippy::too_many_arguments)]
pub fn handle_entangle_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
            &Transform,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster, With<LocalWizard>>,
    mut indicator_query: Query<&mut EntangleIndicator>,
    targets_query: Query<(Entity, &Transform, &Team), Without<Wizard>>,
    mut defender_hit_msg: MessageWriter<EntangleHitDefenderMessage>,
) {
    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };

    if mouse_left_released.read().next().is_some() {
        if let Ok(caster) = caster_query.single() {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    let Some(mut cursor_world_pos) = get_cursor_world_position(&camera_query, &window_query) else {
        return;
    };

    // Clamp cursor to spell range accounting for circle radius
    let wizard_pos = wizard_transform.translation;
    let wizard_height = wizard_pos.y;
    let max_ground_radius = if wizard_height < wizard.spell_range {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };
    let scale = primed_spell.empowerment;
    let circle_radius = constants::CIRCLE_RADIUS * scale;
    let max_center_distance = (max_ground_radius - circle_radius).max(0.0);
    let direction = cursor_world_pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        cursor_world_pos = wizard_pos + normalized_direction * max_center_distance;
    }

    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err() && mana.can_afford(constants::MANA_COST) {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    cursor_world_pos,
                    primed_spell.empowerment,
                );
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if let Ok(caster) = caster_query.single()
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = cursor_world_pos;
            }

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Ok(caster) = caster_query.single()
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
                            let root_duration = constants::ROOT_DURATION * indicator.empowerment;
                            apply_entangle(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                indicator.position,
                                radius,
                                root_duration,
                                &targets_query,
                                &mut defender_hit_msg,
                            );
                        }
                        commands.entity(indicator_entity).despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    mouse_state.left_consumed = true;
                } else {
                    if let Ok(caster) = caster_query.single()
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        commands.entity(indicator_entity).despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.single() {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
        }
    }
}

pub fn update_entangle_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut EntangleIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(pulse);
        transform.translation.x = indicator.position.x;
        transform.translation.y = constants::CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

/// Fades entangle ground effect over time.
pub fn fade_entangle_ground_effect(
    time: Res<Time>,
    mut effects: Query<(&mut EntangleGroundEffect, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();
    for (mut effect, material_handle) in &mut effects {
        effect.time_remaining -= delta;
        let Some(material) = materials.get_mut(material_handle) else {
            continue;
        };
        let progress = (effect.time_remaining / effect.duration).max(0.0);
        material.base_color = Color::srgba(0.1, 0.6, 0.15, 0.35 * progress);
    }
}

/// Despawns expired entangle ground effects.
pub fn cleanup_entangle_ground_effect(
    mut commands: Commands,
    effects: Query<(Entity, &EntangleGroundEffect)>,
) {
    for (entity, effect) in &effects {
        if effect.time_remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Applies root to ALL units in radius (magic is indiscriminate).
#[allow(clippy::too_many_arguments)]
fn apply_entangle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    circle_pos: Vec3,
    radius: f32,
    root_duration: f32,
    targets: &Query<(Entity, &Transform, &Team), Without<Wizard>>,
    defender_hit_msg: &mut MessageWriter<EntangleHitDefenderMessage>,
) {
    for (entity, transform, team) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);
        if distance <= radius {
            commands
                .entity(entity)
                .insert(RootedModifier::new(root_duration));

            // Friendly Thorns: Entangle rooted a defender
            if *team == Team::Defenders {
                defender_hit_msg.write(EntangleHitDefenderMessage);
            }
        }
    }

    // Spawn ground visual
    let circle_mesh = meshes.add(Circle::new(radius));
    let circle_material = materials.add(StandardMaterial {
        base_color: constants::GROUND_EFFECT_COLOR,
        unlit: true,
        alpha_mode: bevy::prelude::AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(circle_mesh),
        MeshMaterial3d(circle_material),
        Transform::from_translation(Vec3::new(
            circle_pos.x,
            constants::CIRCLE_Y_POSITION,
            circle_pos.z,
        ))
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        EntangleGroundEffect::new(root_duration),
        OnGameplayScreen,
    ));
}

fn spawn_circle_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    empowerment: f32,
) -> Entity {
    let radius = constants::CIRCLE_RADIUS * empowerment;
    let circle_mesh = meshes.add(Circle::new(radius));
    let circle_material = materials.add(StandardMaterial {
        base_color: constants::CIRCLE_COLOR,
        unlit: true,
        ..default()
    });

    commands
        .spawn((
            Mesh3d(circle_mesh),
            MeshMaterial3d(circle_material),
            Transform::from_translation(Vec3::new(
                position.x,
                constants::CIRCLE_Y_POSITION,
                position.z,
            ))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            EntangleIndicator::new(position, empowerment),
            OnGameplayScreen,
        ))
        .id()
}

fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return None;
    };
    let Ok(window) = window_query.single() else {
        return None;
    };
    let cursor_position = window.cursor_position()?;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return None;
    };
    if ray.direction.y.abs() < 0.0001 {
        return None;
    }
    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None;
    }
    Some(ray.origin + ray.direction * t)
}
