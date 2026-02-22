use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, SpellCaster, LocalWizard, Wizard, WizardInput};
use super::components::{SpikeGrowthIndicator, SpikeGrowthZone};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::networking::snapshot::SpellEffectKind;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleType};
use crate::game::units::components::{
    Health, SpellDamaged, SpikeGrowthSlowModifier, TemporaryHitPoints, apply_damage_to_unit,
};

/// Local wizard spike growth casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_spike_growth_casting(
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
    mut indicator_query: Query<&mut SpikeGrowthIndicator>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::SpikeGrowth { return; }

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_range(input.cursor_pos, wizard_transform, wizard, primed_spell);

    // Handle release — clean up indicator
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    let Some(clamped_cursor) = clamped_cursor else {
        return;
    };

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    clamped_cursor,
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

            // Update indicator position
            if let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = clamped_cursor;
            }

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
                            spawn_spike_growth_zone(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                indicator.position,
                                radius,
                                indicator.empowerment,
                                &mut obstacle_events,
                            );
                        }
                        commands.entity(indicator_entity).despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    mouse_state.left_consumed = true;
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
}

/// Guest wizard spike growth casting — reads network signals.
#[allow(clippy::too_many_arguments)]
pub fn handle_spike_growth_casting_guest(
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

    let Ok((_wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::SpikeGrowth { return; }

    let clamped_cursor = clamp_cursor_to_range(input.cursor_pos, wizard_transform, wizard, primed_spell);

    let cast_result = spike_growth_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        clamped_cursor,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut obstacle_events,
    );

    let _ = cast_result;
}

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned).
    completed: bool,
}

/// Core spike growth casting logic for guest — no indicator management.
#[allow(clippy::too_many_arguments)]
fn spike_growth_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    clamped_cursor: Option<Vec3>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) -> CastResult {
    let mut result = CastResult { completed: false };

    if input.just_released {
        casting_state.cancel();
        return result;
    }

    let Some(clamped_cursor) = clamped_cursor else {
        return result;
    };

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(constants::MANA_COST) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    let radius = constants::CIRCLE_RADIUS * primed_spell.empowerment;
                    spawn_spike_growth_zone(
                        commands,
                        meshes,
                        materials,
                        clamped_cursor,
                        radius,
                        primed_spell.empowerment,
                        obstacle_events,
                    );
                    result.completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    result
}

/// Clamps cursor position to wizard's spell range, accounting for the circle radius.
fn clamp_cursor_to_range(
    cursor_pos: Option<Vec3>,
    wizard_transform: &Transform,
    wizard: &Wizard,
    primed_spell: &PrimedSpell,
) -> Option<Vec3> {
    let mut cursor_world_pos = cursor_pos?;

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

    Some(cursor_world_pos)
}

pub fn update_spike_growth_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut SpikeGrowthIndicator, &mut Transform)>,
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

/// Applies periodic damage and slow to ALL units within the spike growth zone.
pub fn apply_spike_growth_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<&mut SpikeGrowthZone>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Option<&mut SpikeGrowthSlowModifier>,
    )>,
) {
    let delta = time.delta_secs();

    for mut zone in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;

        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp, existing_slow) in &mut targets {
                let distance = Vec3::new(
                    zone.origin.x - transform.translation.x,
                    0.0,
                    zone.origin.z - transform.translation.z,
                )
                .length();

                if distance <= zone.radius {
                    // Apply damage
                    apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), zone.damage_per_tick);
                    commands.entity(entity).insert(SpellDamaged);

                    // Apply or refresh spike growth slow
                    if let Some(mut slow) = existing_slow {
                        slow.refresh(zone.slow_duration);
                    } else {
                        commands.entity(entity).insert(SpikeGrowthSlowModifier::new(
                            zone.slow_modifier,
                            zone.slow_duration,
                        ));
                    }
                }
            }
        }
    }
}

/// Fades spike growth zone visual over the last few seconds.
pub fn fade_spike_growth_zone(
    zones: Query<(&SpikeGrowthZone, &MeshMaterial3d<StandardMaterial>)>,
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

        material.base_color = Color::srgba(0.15, 0.4, 0.05, 0.4 * fade);
    }
}

/// Despawns expired spike growth zones.
pub fn cleanup_spike_growth_zone(
    mut commands: Commands,
    zones: Query<(Entity, &SpikeGrowthZone)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.duration {
            let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
            let buffered_radius = zone.radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                obstacle_type: ObstacleType::Removed,
            });
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_spike_growth_zone(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let duration = constants::ZONE_DURATION * empowerment;
    let damage = constants::DAMAGE_PER_TICK * empowerment;
    let slow_mod = constants::SLOW_MODIFIER * empowerment;
    let slow_dur = constants::SLOW_DURATION * empowerment;

    // Notify pathfinding about hazard zone (buffered so units reroute before reaching it)
    let origin_2d = Vec2::new(position.x, position.z);
    let buffered_radius = radius + OBSTACLE_BUFFER;
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type: ObstacleType::Hazard(15.0),
    });

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
        SpikeGrowthZone::new(
            Vec3::new(position.x, 0.0, position.z),
            radius,
            damage,
            constants::TICK_INTERVAL,
            slow_mod,
            slow_dur,
            duration,
        ),
        NetworkedSpellEffect { kind: SpellEffectKind::SpikeGrowthZone },
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
            SpikeGrowthIndicator::new(position, empowerment),
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
