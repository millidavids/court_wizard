use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::components::{PlagueWindCloud, PlagueWindIndicator};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::ATTACKER_GRID_CENTER_ANGLE;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::game::units::wizard::components::{
    CastingState, GuestWizard, Mana, PrimedSpell, Spell, SpellCaster, LocalWizard, Wizard, WizardInput,
};

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned).
    completed: bool,
}

/// Local wizard plague wind casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_plague_wind_casting(
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
    mut indicator_query: Query<&mut PlagueWindIndicator>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::PlagueWind { return; }

    let wizard_pos = wizard_transform.translation;
    let scale = primed_spell.empowerment;
    let radius = constants::CLOUD_RADIUS * scale;
    let clamped_pos = input.cursor_pos.map(|pos| clamp_to_spell_range(pos, wizard_pos, wizard.spell_range, radius));

    // Spawn indicator on Resting -> Casting transition
    if matches!(*casting_state, CastingState::Resting)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(constants::MANA_COST)
        && let Some(pos) = clamped_pos
    {
        let circle_entity = spawn_circle_indicator(
            &mut commands,
            &mut meshes,
            &mut materials,
            pos,
            scale,
        );
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(circle_entity));
    }

    // Update indicator position during casting
    if matches!(*casting_state, CastingState::Casting { .. }) {
        if let Some(pos) = clamped_pos
            && let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
            && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
        {
            indicator.position = pos;
        }
    }

    // Get the final spawn position from indicator if available
    let indicator_pos = caster_query
        .get(wizard_entity)
        .ok()
        .and_then(|caster| caster.indicator_entity)
        .and_then(|ie| indicator_query.get(ie).ok())
        .map(|indicator| indicator.position);

    let effective_input = WizardInput {
        cursor_pos: indicator_pos.or(clamped_pos),
        ..input
    };

    let cast_result = plague_wind_casting_logic(
        &effective_input,
        &time,
        wizard_entity,
        wizard_transform,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut obstacle_events,
    );

    if cast_result.completed {
        mouse_state.left_consumed = true;
    }
}

/// Guest wizard plague wind casting -- reads network signals.
#[allow(clippy::too_many_arguments)]
pub fn handle_plague_wind_casting_guest(
    time: Res<Time>,
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
        With<GuestWizard>,
    >,
    caster_query: Query<&SpellCaster>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    guest_cursor: Res<GuestCursorPosition>,
    guest_input: Res<GuestInputState>,
) {
    let input = WizardInput {
        just_pressed: guest_input.just_pressed,
        pressed: guest_input.pressed,
        just_released: guest_input.just_released,
        cursor_pos: guest_cursor.position,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::PlagueWind { return; }

    let wizard_pos = wizard_transform.translation;
    let scale = primed_spell.empowerment;
    let radius = constants::CLOUD_RADIUS * scale;

    // Insert SpellCaster::new() (no indicator) on Resting -> Casting transition
    if matches!(*casting_state, CastingState::Resting)
        && (input.just_pressed || input.pressed)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(constants::MANA_COST)
        && input.cursor_pos.is_some()
    {
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::new());
    }

    // Clamp cursor for guest
    let clamped_input = WizardInput {
        cursor_pos: input.cursor_pos.map(|pos| clamp_to_spell_range(pos, wizard_pos, wizard.spell_range, radius)),
        ..input
    };

    plague_wind_casting_logic(
        &clamped_input,
        &time,
        wizard_entity,
        wizard_transform,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut obstacle_events,
    );
}

/// Core plague wind casting logic -- called by both local and guest systems.
#[allow(clippy::too_many_arguments)]
fn plague_wind_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard_transform: &Transform,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) -> CastResult {
    let mut result = CastResult { completed: false };

    let wizard_pos = wizard_transform.translation;
    let scale = primed_spell.empowerment;
    let radius = constants::CLOUD_RADIUS * scale;

    // Check for release event
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

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
                && input.cursor_pos.is_some()
            {
                // SpellCaster insertion handled by the wrapper
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    let pos = input.cursor_pos.unwrap_or(wizard_pos);

                    // Cloud drifts toward attacker spawn direction
                    let direction = Vec3::new(
                        ATTACKER_GRID_CENTER_ANGLE.cos(),
                        0.0,
                        ATTACKER_GRID_CENTER_ANGLE.sin(),
                    )
                    .normalize();

                    let damage = constants::DAMAGE_PER_TICK * scale;

                    // Notify pathfinding
                    let origin_2d = Vec2::new(pos.x, pos.z);
                    let buffered = radius + OBSTACLE_BUFFER;
                    obstacle_events.write(ObstacleChanged {
                        bounds: Rect::from_center_size(
                            origin_2d,
                            Vec2::splat(buffered * 2.0),
                        ),
                        obstacle_type: ObstacleType::Hazard(10.0),
                    });

                    let cloud_mesh = meshes.add(Circle::new(radius));
                    let cloud_material = materials.add(StandardMaterial {
                        base_color: constants::CLOUD_COLOR,
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        cull_mode: None,
                        ..default()
                    });

                    commands.spawn((
                        Mesh3d(cloud_mesh),
                        MeshMaterial3d(cloud_material),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(Quat::from_rotation_x(
                                -std::f32::consts::FRAC_PI_2,
                            )),
                        PlagueWindCloud::new(
                            pos,
                            radius,
                            damage,
                            constants::TICK_INTERVAL,
                            constants::CLOUD_DURATION * scale,
                            constants::CLOUD_SPEED,
                            direction,
                        ),
                        NetworkedSpellEffect { kind: SpellEffectKind::PlagueWindCloud },
                        OnGameplayScreen,
                    ));

                    result.completed = true;

                    // Clean up indicator
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        commands.entity(indicator_entity).despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
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

pub fn update_plague_wind_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut PlagueWindIndicator, &mut Transform)>,
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

/// Moves the plague wind cloud in its drift direction and updates pathfinding.
pub fn move_plague_wind_cloud(
    time: Res<Time>,
    mut clouds: Query<(&mut PlagueWindCloud, &mut Transform)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    for (mut cloud, mut transform) in clouds.iter_mut() {
        // Remove old pathfinding bounds
        let old_origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
        let buffered = cloud.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(old_origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Removed,
        });

        // Move cloud
        let movement = cloud.direction * cloud.speed * delta;
        cloud.origin += movement;
        transform.translation.x = cloud.origin.x;
        transform.translation.z = cloud.origin.z;

        // Add new pathfinding bounds
        let new_origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(new_origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Hazard(10.0),
        });
    }
}

/// Applies periodic necrotic damage to all units within the cloud.
pub fn apply_plague_wind_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut clouds: Query<&mut PlagueWindCloud>,
    mut units: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
    )>,
) {
    let delta = time.delta_secs();

    for mut cloud in &mut clouds {
        cloud.time_alive += delta;
        cloud.time_since_last_tick += delta;

        if cloud.time_since_last_tick >= cloud.tick_interval {
            cloud.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp) in &mut units {
                let dist = Vec3::new(
                    cloud.origin.x - transform.translation.x,
                    0.0,
                    cloud.origin.z - transform.translation.z,
                )
                .length();

                if dist <= cloud.radius {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        cloud.damage_per_tick,
                        DamageType::Necrotic,
                    );
                }
            }
        }
    }
}

/// Fades cloud visual opacity as it approaches expiration.
pub fn fade_plague_wind_cloud(
    clouds: Query<(&PlagueWindCloud, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (cloud, material_handle) in &clouds {
        let Some(material) = materials.get_mut(material_handle) else {
            continue;
        };

        let remaining = cloud.duration - cloud.time_alive;
        let fade = if remaining < constants::FADE_DURATION {
            (remaining / constants::FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        material.base_color = Color::srgba(0.2, 0.6, 0.1, 0.4 * fade);
    }
}

/// Cleans up expired plague wind clouds and notifies pathfinding.
pub fn cleanup_plague_wind_cloud(
    mut commands: Commands,
    clouds: Query<(Entity, &PlagueWindCloud)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, cloud) in &clouds {
        if cloud.time_alive >= cloud.duration {
            let origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
            let buffered = cloud.radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
                obstacle_type: ObstacleType::Removed,
            });
            commands.entity(entity).despawn();
        }
    }
}

// --- Helper functions ---

fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    if ray.direction.y.abs() < 0.0001 {
        return None;
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None;
    }

    Some(ray.origin + ray.direction * t)
}

fn clamp_to_spell_range(target: Vec3, wizard_pos: Vec3, spell_range: f32, radius: f32) -> Vec3 {
    let wizard_height = wizard_pos.y;
    let max_ground_radius = if wizard_height < spell_range {
        (spell_range * spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };
    let max_center_distance = (max_ground_radius - radius).max(0.0);
    let direction = target - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();

    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        wizard_pos + normalized_direction * max_center_distance
    } else {
        target
    }
}

fn spawn_circle_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    empowerment: f32,
) -> Entity {
    let radius = constants::CLOUD_RADIUS * empowerment;
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
            PlagueWindIndicator::new(position),
            OnGameplayScreen,
        ))
        .id()
}
