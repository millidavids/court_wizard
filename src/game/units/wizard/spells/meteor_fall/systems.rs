//! Meteor Fall spell systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use super::components::{
    MeteorExplosion, MeteorFallCircleIndicator, MeteorFallStorm, MeteorGroundFire, MeteorProjectile,
};
use super::constants::*;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::input::MouseButtonState;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::multiplayer::spell_commands::SkipSpellSpawning;
use crate::networking::protocol::{NetworkMessage, SpellAction};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::wizard::components::{
    CastingState, Mana, PrimedSpell, Spell, SpellCaster, LocalWizard, Wizard, WizardInput,
};

/// Gets cursor position projected onto Y=0 plane.
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

    // Intersect ray with Y=0 plane
    if ray.direction.y.abs() < 0.0001 {
        return None; // Ray is parallel to plane
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None; // Intersection is behind camera
    }

    Some(ray.origin + ray.direction * t)
}

/// Clamps a position to be within the wizard's spell range.
fn clamp_to_spell_range(
    target: Vec3,
    wizard_pos: Vec3,
    spell_range: f32,
    storm_radius: f32,
) -> Vec3 {
    let wizard_height = wizard_pos.y;

    // Calculate max ground radius using Pythagorean theorem
    let max_ground_radius = if wizard_height < spell_range {
        (spell_range * spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };

    // Account for storm radius so entire circle stays within range
    let max_center_distance = (max_ground_radius - storm_radius).max(0.0);

    // Calculate XZ plane distance from wizard to cursor
    let direction = target - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();

    if distance > max_center_distance && distance > 0.001 {
        // Clamp to ensure entire circle stays within spell range
        let normalized_direction = direction / distance;
        wizard_pos + normalized_direction * max_center_distance
    } else {
        target
    }
}

/// Spawns the visual circle indicator during casting.
fn spawn_circle_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    empowerment: f32,
) -> Entity {
    let scale = empowerment;
    let radius = STORM_RADIUS * scale;

    let circle_mesh = meshes.add(Circle::new(radius));
    let circle_material = materials.add(StandardMaterial {
        base_color: CIRCLE_COLOR,
        unlit: true,
        ..default()
    });

    commands
        .spawn((
            Mesh3d(circle_mesh),
            MeshMaterial3d(circle_material),
            Transform::from_translation(Vec3::new(position.x, CIRCLE_Y_POSITION, position.z))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            MeteorFallCircleIndicator::new(position),
            OnGameplayScreen,
        ))
        .id()
}

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned/skipped).
    completed: bool,
    /// Cursor position at time of completion (for network message).
    cursor_pos: Option<Vec3>,
}

/// Local wizard meteor fall casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_meteor_fall_casting(
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
    mut indicator_query: Query<&mut MeteorFallCircleIndicator>,
    existing_storms: Query<Entity, With<MeteorFallStorm>>,
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
    if primed_spell.spell != Spell::MeteorFall { return; }

    let skip_spawn = skip_spawning.is_some();

    let cast_result = meteor_fall_casting_logic(
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
        &existing_storms,
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
                        spell: Spell::MeteorFall,
                        cursor_pos: [pos.x, pos.y, pos.z],
                        empowerment: primed_spell.empowerment,
                    },
                ));
            }
        }
    }
}

/// Core meteor fall casting logic.
#[allow(clippy::too_many_arguments)]
fn meteor_fall_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard_transform: &Transform,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut MeteorFallCircleIndicator>,
    existing_storms: &Query<Entity, With<MeteorFallStorm>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    skip_spawn: bool,
) -> CastResult {
    let mut result = CastResult { completed: false, cursor_pos: None };

    // Check for release event - cancel cast
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

    // Get cursor world position and clamp to wizard's spell range
    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return result;
    };

    let wizard_pos = wizard_transform.translation;
    let scale = primed_spell.empowerment;
    let storm_radius = STORM_RADIUS * scale;

    cursor_world_pos = clamp_to_spell_range(
        cursor_world_pos,
        wizard_pos,
        wizard.spell_range,
        storm_radius,
    );

    // Handle casting based on state
    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(MANA_COST)
            {
                // Start casting - spawn circle indicator
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

                // Start the cast
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Update circle position to follow cursor
            if let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = cursor_world_pos;
            }

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - spawn storm entity
                if mana.consume(MANA_COST) {
                    if !skip_spawn {
                        // Despawn any existing storms (only one storm at a time)
                        for existing_storm in existing_storms.iter() {
                            commands.entity(existing_storm).despawn();
                        }

                        // Get final circle position and spawn storm
                        if let Ok(caster) = caster_query.get(wizard_entity)
                            && let Some(indicator_entity) = caster.indicator_entity
                        {
                            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                                commands.spawn((
                                    MeteorFallStorm::new(
                                        indicator.position,
                                        storm_radius,
                                        primed_spell.empowerment,
                                    ),
                                    ConcentrationSpell {
                                        spell_name: "Meteor Fall",
                                    },
                                    OnGameplayScreen,
                                ));
                            }

                            // Despawn circle indicator
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

                    // Remove caster marker immediately
                    commands.entity(wizard_entity).remove::<SpellCaster>();

                    // Return to resting state
                    casting_state.cancel();
                    result.completed = true;
                    result.cursor_pos = Some(cursor_world_pos);
                } else {
                    // Out of mana - cancel cast
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
            // Meteor Fall doesn't use channeling, cancel if we somehow get here
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

/// Updates circle indicator visuals during casting.
///
/// Applies pulse animation and updates position.
pub(super) fn update_circle_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut MeteorFallCircleIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        // Update time alive for pulse animation
        indicator.time_alive += time.delta_secs();

        // Apply pulse scale
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(pulse);

        // Update position
        transform.translation.x = indicator.position.x;
        transform.translation.y = CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

/// Spawns meteor projectiles periodically from active storms.
///
/// Projectiles spawn at random positions within the storm radius, high above the battlefield.
pub(super) fn spawn_meteor_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut storms: Query<&mut MeteorFallStorm>,
) {
    let mut rng = rand::thread_rng();

    for mut storm in storms.iter_mut() {
        storm.update_timers(time.delta_secs());

        // Check if it's time to spawn another meteor
        if storm.time_since_spawn >= METEOR_SPAWN_INTERVAL {
            storm.reset_spawn_timer();

            // Random position within storm circle (on XZ plane)
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..storm.radius);
            let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let spawn_pos = Vec3::new(
                storm.position.x + offset.x,
                METEOR_SPAWN_HEIGHT,
                storm.position.z + offset.z,
            );

            // Create meteor projectile mesh (sphere)
            let sphere = Sphere::new(METEOR_MESH_RADIUS);
            let material = materials.add(StandardMaterial {
                base_color: METEOR_COLOR,
                unlit: true,
                ..default()
            });

            // Spawn projectile
            let damage = METEOR_DAMAGE * storm.empowerment;
            let explosion_radius = EXPLOSION_RADIUS * storm.empowerment;

            commands.spawn((
                MeteorProjectile::new(
                    Vec3::new(0.0, METEOR_INITIAL_VELOCITY, 0.0),
                    damage,
                    explosion_radius,
                    storm.empowerment,
                ),
                Mesh3d(meshes.add(sphere)),
                MeshMaterial3d(material),
                Transform::from_translation(spawn_pos),
                OnGameplayScreen,
            ));
        }
    }
}

/// Updates meteor projectile physics - applies gravity and moves projectiles.
pub(super) fn update_meteor_projectiles(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &mut MeteorProjectile)>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut projectile) in projectiles.iter_mut() {
        // Apply gravity
        projectile.velocity.y += METEOR_GRAVITY * delta;

        // Move projectile
        transform.translation += projectile.velocity * delta;
    }
}

/// Checks for meteor collisions with the ground, spawns explosions and ground fires.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_meteor_collisions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    projectiles: Query<(Entity, &Transform, &MeteorProjectile)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, transform, projectile) in projectiles.iter() {
        let projectile_pos = transform.translation;

        // Check ground collision (Y <= 0)
        if projectile_pos.y <= 0.0 {
            let pos = Vec3::new(projectile_pos.x, 0.0, projectile_pos.z);

            // Spawn explosion visual and damage
            commands.spawn((
                Mesh3d(meshes.add(Circle::new(1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: EXPLOSION_COLOR,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(0.1)),
                MeteorExplosion::new(pos, projectile.explosion_radius, projectile.damage),
                NetworkedSpellEffect { kind: SpellEffectKind::MeteorExplosion },
                OnGameplayScreen,
            ));

            // Spawn ground fire hazard
            let fire_radius = GROUND_FIRE_RADIUS * projectile.empowerment;
            let fire_damage = GROUND_FIRE_DAMAGE * projectile.empowerment;
            let fire_duration = GROUND_FIRE_DURATION;

            let origin_2d = Vec2::new(pos.x, pos.z);
            let buffered = fire_radius + OBSTACLE_BUFFER;

            // Notify pathfinding system about the hazard
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
                obstacle_type: ObstacleType::Hazard(8.0),
            });

            commands.spawn((
                Mesh3d(meshes.add(Circle::new(fire_radius))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: GROUND_FIRE_COLOR,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(pos.x, 0.5, pos.z))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                MeteorGroundFire::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    fire_radius,
                    fire_damage,
                    GROUND_FIRE_TICK,
                    fire_duration,
                ),
                NetworkedSpellEffect { kind: SpellEffectKind::MeteorGroundFire },
                OnGameplayScreen,
            ));

            // Despawn the projectile
            commands.entity(entity).despawn();
        }
    }
}

/// Updates explosion visuals and applies one-time impact damage.
pub(super) fn update_meteor_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut MeteorExplosion, &mut Transform)>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<MeteorExplosion>,
    >,
) {
    for (explosion_entity, mut explosion, mut transform) in explosions.iter_mut() {
        explosion.time_alive += time.delta_secs();

        // Update visual scale (growth animation)
        let current_radius = explosion.current_radius(EXPLOSION_GROWTH_TIME);
        transform.scale = Vec3::splat(current_radius);

        // Apply damage once when explosion spawns
        if !explosion.damage_applied {
            explosion.damage_applied = true;

            for (unit_entity, unit_transform, mut health, mut temp_hp) in units.iter_mut() {
                let distance = unit_transform.translation.distance(explosion.origin);

                if distance <= explosion.max_radius {
                    apply_spell_damage(
                        &mut commands,
                        unit_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage,
                        DamageType::Fire,
                    );
                }
            }
        }

        // Despawn explosion after lifetime
        if explosion.time_alive >= EXPLOSION_LIFETIME {
            commands.entity(explosion_entity).despawn();
        }
    }
}

/// Applies periodic fire damage to units standing in ground fire zones.
pub(super) fn apply_ground_fire_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut fires: Query<&mut MeteorGroundFire>,
    mut units: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
    )>,
) {
    let delta = time.delta_secs();

    for mut fire in &mut fires {
        fire.time_alive += delta;
        fire.time_since_last_tick += delta;

        if fire.time_since_last_tick >= fire.tick_interval {
            fire.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp) in &mut units {
                let dist = Vec3::new(
                    fire.origin.x - transform.translation.x,
                    0.0,
                    fire.origin.z - transform.translation.z,
                )
                .length();

                if dist <= fire.radius {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        fire.damage_per_tick,
                        DamageType::Fire,
                    );
                }
            }
        }
    }
}

/// Fades ground fire visual opacity as it approaches expiration.
pub(super) fn fade_ground_fire(
    fires: Query<(&MeteorGroundFire, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (fire, material_handle) in &fires {
        let Some(material) = materials.get_mut(material_handle) else {
            continue;
        };

        let remaining = fire.duration - fire.time_alive;
        let fade = if remaining < GROUND_FIRE_FADE_DURATION {
            (remaining / GROUND_FIRE_FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        material.base_color = Color::srgba(0.9, 0.25, 0.05, 0.5 * fade);
    }
}

/// Cleans up expired ground fire zones and notifies pathfinding.
pub(super) fn cleanup_ground_fire(
    mut commands: Commands,
    fires: Query<(Entity, &MeteorGroundFire)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, fire) in &fires {
        if fire.time_alive >= fire.duration {
            let origin_2d = Vec2::new(fire.origin.x, fire.origin.z);
            let buffered = fire.radius + OBSTACLE_BUFFER;

            // Notify pathfinding that the hazard is removed
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
                obstacle_type: ObstacleType::Removed,
            });

            commands.entity(entity).despawn();
        }
    }
}
