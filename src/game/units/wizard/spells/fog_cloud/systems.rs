use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Spell, SpellCaster, LocalWizard, Wizard, WizardInput};
use super::components::{FogCloudIndicator, FogCloudZone};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::multiplayer::spell_commands::SkipSpellSpawning;
use crate::networking::protocol::{NetworkMessage, SpellAction};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, FogEvasionModifier};

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned/skipped).
    completed: bool,
    /// Cursor position at time of completion (for network message).
    cursor_pos: Option<Vec3>,
}

/// Local wizard fog cloud casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_fog_cloud_casting(
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
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut FogCloudIndicator>,
    skip_spawning: Option<Res<SkipSpellSpawning>>,
    mut connection: Option<ResMut<NetworkConnection>>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions already ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::FogCloud { return; }

    let skip_spawn = skip_spawning.is_some();

    let cast_result = fog_cloud_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard_transform,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &mut commands,
        &mut meshes,
        &mut materials,
        skip_spawn,
    );

    if cast_result.completed {
        mouse_state.left_consumed = true;

        if skip_spawn {
            if let (Some(conn), Some(pos)) = (connection.as_mut(), cast_result.cursor_pos) {
                conn.outgoing_messages.push(NetworkMessage::SpellResult(
                    SpellAction::SpellCast {
                        spell: Spell::FogCloud,
                        cursor_pos: [pos.x, pos.y, pos.z],
                        empowerment: primed_spell.empowerment,
                    },
                ));
            }
        }
    }
}

/// Core fog cloud casting logic.
#[allow(clippy::too_many_arguments)]
fn fog_cloud_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard_transform: &Transform,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut FogCloudIndicator>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    skip_spawn: bool,
) -> CastResult {
    let mut result = CastResult { completed: false, cursor_pos: None };

    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return result;
    }

    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return result;
    };

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
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                let circle_entity = spawn_circle_indicator(
                    commands,
                    meshes,
                    materials,
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
            if let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = cursor_world_pos;
            }
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if !skip_spawn {
                        if let Ok(caster) = caster_query.get(wizard_entity)
                            && let Some(indicator_entity) = caster.indicator_entity
                        {
                            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                                let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
                                spawn_fog_cloud_zone(
                                    commands,
                                    meshes,
                                    materials,
                                    indicator.position,
                                    radius,
                                    indicator.empowerment,
                                );
                            }
                            commands.entity(indicator_entity).despawn();
                        }
                    } else {
                        // Skip spawn: still clean up indicator
                        if let Ok(caster) = caster_query.get(wizard_entity)
                            && let Some(indicator_entity) = caster.indicator_entity
                        {
                            commands.entity(indicator_entity).despawn();
                        }
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    result.completed = true;
                    result.cursor_pos = Some(cursor_world_pos);
                } else {
                    if let Ok(caster) = caster_query.get(wizard_entity)
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
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
        }
    }

    result
}

pub fn update_fog_cloud_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut FogCloudIndicator, &mut Transform)>,
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

pub fn apply_fog_cloud_evasion(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<&mut FogCloudZone>,
    mut targets: Query<(Entity, &Transform, Option<&mut FogEvasionModifier>), Without<Corpse>>,
) {
    let delta = time.delta_secs();
    for mut zone in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;
            for (entity, transform, existing_evasion) in &mut targets {
                let dist = Vec3::new(
                    zone.origin.x - transform.translation.x,
                    0.0,
                    zone.origin.z - transform.translation.z,
                )
                .length();
                if dist <= zone.radius {
                    if let Some(mut evasion) = existing_evasion {
                        evasion.refresh(zone.evasion_refresh_duration);
                    } else {
                        commands.entity(entity).insert(FogEvasionModifier::new(
                            zone.evasion_chance,
                            zone.evasion_refresh_duration,
                        ));
                    }
                }
            }
        }
    }
}

pub fn fade_fog_cloud_zone(
    zones: Query<(&FogCloudZone, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (zone, material_handle) in &zones {
        let Some(material) = materials.get_mut(material_handle) else {
            continue;
        };
        let remaining = zone.duration - zone.time_alive;
        let fade = if remaining < constants::FADE_DURATION {
            (remaining / constants::FADE_DURATION).max(0.0)
        } else {
            1.0
        };
        material.base_color = Color::srgba(0.6, 0.65, 0.7, 0.35 * fade);
    }
}

pub fn cleanup_fog_cloud_zone(mut commands: Commands, zones: Query<(Entity, &FogCloudZone)>) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.duration {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn spawn_fog_cloud_zone(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
    empowerment: f32,
) {
    let duration = constants::ZONE_DURATION * empowerment;
    let evasion = constants::EVASION_CHANCE;
    let refresh_dur = constants::EVASION_REFRESH_DURATION * empowerment;

    let circle_mesh = meshes.add(Circle::new(radius));
    let circle_material = materials.add(StandardMaterial {
        base_color: constants::ZONE_COLOR,
        unlit: true,
        alpha_mode: bevy::prelude::AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(circle_mesh),
        MeshMaterial3d(circle_material),
        Transform::from_translation(Vec3::new(
            position.x,
            constants::CIRCLE_Y_POSITION,
            position.z,
        ))
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        FogCloudZone::new(
            Vec3::new(position.x, 0.0, position.z),
            radius,
            evasion,
            refresh_dur,
            constants::TICK_INTERVAL,
            duration,
        ),
        NetworkedSpellEffect { kind: SpellEffectKind::FogCloudZone },
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
            FogCloudIndicator::new(position, empowerment),
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
