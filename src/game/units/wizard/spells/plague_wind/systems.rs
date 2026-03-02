use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::components::{PlagueWindCloud, PlagueWindIndicator};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::{ATTACKER_GRID_CENTER_ANGLE, SPELL_ORIGIN};
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::units::wizard::spells::utils::{clamp_to_spell_range_ground, get_cursor_world_position, spawn_circle_indicator};

/// Local wizard plague wind casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_plague_wind_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
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

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::PlagueWind {
        return;
    }

    let wizard_pos = SPELL_ORIGIN;
    let scale = primed_spell.empowerment;
    let radius = constants::CLOUD_RADIUS * scale;
    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range_ground(pos, wizard_pos, wizard.spell_range, radius));

    // Spawn indicator on Resting -> Casting transition
    if matches!(*casting_state, CastingState::Resting)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(constants::MANA_COST)
        && let Some(pos) = clamped_pos
    {
        let circle_entity =spawn_circle_indicator(
            &mut commands,
            &visual_assets,
            visual_assets.plague_wind_indicator.clone(),
            pos,
            constants::CLOUD_RADIUS * scale,
            constants::CIRCLE_Y_POSITION,
        )
        .insert(PlagueWindIndicator::new(pos, constants::CLOUD_RADIUS * scale))
        .id();
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(circle_entity));
    }

    // Update indicator position during casting
    if matches!(*casting_state, CastingState::Casting { .. })
        && let Some(pos) = clamped_pos
        && let Ok(caster) = caster_query.get(wizard_entity)
        && let Some(indicator_entity) = caster.indicator_entity
        && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
    {
        indicator.position = pos;
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

    let completed = plague_wind_casting_logic(
        &effective_input,
        &time,
        wizard_entity,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut commands,
        &visual_assets,
        &mut materials,
        &mut obstacle_events,
    );

    if completed {
        mouse_state.left_consumed = true;
    }
}

/// Core plague wind casting logic.
#[allow(clippy::too_many_arguments)]
fn plague_wind_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) -> bool {
    let wizard_pos = SPELL_ORIGIN;
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
        return false;
    }

    let mut completed = false;

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
                        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
                        obstacle_type: ObstacleType::Hazard(10.0),
                        shape: Some(ObstacleShape::circle(origin_2d, buffered)),
                    });

                    let base_mat = materials
                        .get(&assets.plague_wind_zone)
                        .cloned()
                        .unwrap_or_default();
                    let cloud_material = materials.add(base_mat);

                    commands.spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(cloud_material),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3::splat(radius)),
                        PlagueWindCloud::new(
                            pos,
                            radius,
                            damage,
                            constants::TICK_INTERVAL,
                            constants::CLOUD_DURATION * scale,
                            constants::CLOUD_SPEED,
                            direction,
                        ),
                        NetworkedSpellEffect {
                            kind: SpellEffectKind::PlagueWindCloud,
                        },
                        OnGameplayScreen,
                    ));

                    completed = true;

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

    completed
}

pub fn update_plague_wind_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut PlagueWindIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(indicator.radius * pulse);
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
            shape: Some(ObstacleShape::circle(old_origin_2d, buffered)),
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
            shape: Some(ObstacleShape::circle(new_origin_2d, buffered)),
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
        Has<SpellShield>,
    )>,
) {
    let delta = time.delta_secs();

    for mut cloud in &mut clouds {
        cloud.time_alive += delta;
        cloud.time_since_last_tick += delta;

        if cloud.time_since_last_tick >= cloud.tick_interval {
            cloud.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp, has_spell_shield) in &mut units {
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
                        has_spell_shield,
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
                shape: Some(ObstacleShape::circle(origin_2d, buffered)),
            });
            commands.entity(entity).despawn();
        }
    }
}

// --- Helper functions ---
