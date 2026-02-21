//! Squall spell systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use super::components::{IceExplosion, IceProjectile, SquallCircleIndicator, SquallStorm};
use super::constants::*;
use super::styles::*;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::input::MouseButtonState;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::DamageType;
use crate::game::units::components::{Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::wizard::components::{
    CastingState, Mana, PrimedSpell, SpellCaster, LocalWizard, Wizard,
};
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

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
            SquallCircleIndicator::new(position, empowerment),
            OnGameplayScreen,
        ))
        .id()
}

/// Handles Squall spell casting with circle indicator.
///
/// Left-click starts cast. Must hold for full cast time.
/// After cast completes, spawns squall storm entity that persists until concentration ends.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_squall_casting(
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
    mut indicator_query: Query<&mut SquallCircleIndicator>,
    existing_storms: Query<Entity, With<SquallStorm>>,
) {
    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };

    // Check for release event - cancel cast
    if mouse_left_released.read().next().is_some() {
        if let Ok(caster) = caster_query.single() {
            // Despawn circle indicator if it exists
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            // Remove caster marker
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    // Get cursor world position and clamp to wizard's spell range
    let Some(mut cursor_world_pos) = get_cursor_world_position(&camera_query, &window_query) else {
        return;
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

    // Mouse is held - handle casting based on state
    match *casting_state {
        CastingState::Resting => {
            // Only start if we don't have a caster marker and have enough mana
            if caster_query.get(wizard_entity).is_err() && mana.can_afford(MANA_COST) {
                // Start casting - spawn circle indicator
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    cursor_world_pos,
                    primed_spell.empowerment,
                );

                // Mark wizard as casting Squall
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
            if let Ok(caster) = caster_query.single()
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = cursor_world_pos;
            }

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - spawn storm entity
                if mana.consume(MANA_COST) {
                    // Despawn any existing storms (only one storm at a time)
                    for existing_storm in existing_storms.iter() {
                        commands.entity(existing_storm).despawn();
                    }

                    // Get final circle position and spawn storm
                    if let Ok(caster) = caster_query.single()
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            // Spawn the storm entity (invisible marker)
                            commands.spawn((
                                SquallStorm::new(
                                    indicator.position,
                                    storm_radius,
                                    primed_spell.empowerment,
                                ),
                                ConcentrationSpell {
                                    spell_name: "Squall",
                                },
                                OnGameplayScreen,
                            ));
                        }

                        // Despawn circle indicator
                        commands.entity(indicator_entity).despawn();
                    }

                    // Remove caster marker immediately (don't keep it blocking future casts)
                    commands.entity(wizard_entity).remove::<SpellCaster>();

                    // Return to resting state
                    casting_state.cancel();
                    // Consume mouse to require release before next cast
                    mouse_state.left_consumed = true;
                } else {
                    // Out of mana - cancel cast
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
            // Squall doesn't use channeling, cancel if we somehow get here
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

/// Updates circle indicator visuals during casting.
///
/// Applies pulse animation and updates position.
pub(super) fn update_circle_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut SquallCircleIndicator, &mut Transform)>,
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

/// Spawns ice projectiles periodically from active storms.
///
/// Projectiles spawn at random positions within the storm radius, high above the battlefield.
pub(super) fn spawn_ice_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut storms: Query<&mut SquallStorm>,
) {
    let mut rng = rand::thread_rng();

    for mut storm in storms.iter_mut() {
        storm.update_timers(time.delta_secs());

        // Check if it's time to spawn another projectile
        if storm.time_since_spawn >= ICE_SPAWN_INTERVAL {
            storm.reset_spawn_timer();

            // Random position within storm circle (on XZ plane)
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..storm.radius);
            let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let spawn_pos = Vec3::new(
                storm.position.x + offset.x,
                ICE_SPAWN_HEIGHT,
                storm.position.z + offset.z,
            );

            // Create ice projectile mesh (small sphere)
            let sphere = Sphere::new(ICE_PROJECTILE_MESH_RADIUS);
            let material = materials.add(StandardMaterial {
                base_color: ICE_PROJECTILE_COLOR,
                unlit: true,
                ..default()
            });

            // Spawn projectile
            let damage = FROST_DAMAGE * storm.empowerment;
            let explosion_radius = EXPLOSION_RADIUS * storm.empowerment;

            commands.spawn((
                IceProjectile::new(
                    Vec3::new(0.0, ICE_INITIAL_VELOCITY, 0.0),
                    damage,
                    explosion_radius,
                    ICE_PROJECTILE_RADIUS,
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

/// Updates ice projectile physics - applies gravity and moves projectiles.
pub(super) fn update_ice_projectiles(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &mut IceProjectile)>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut projectile) in projectiles.iter_mut() {
        // Apply gravity
        projectile.velocity.y += ICE_GRAVITY * delta;

        // Move projectile
        transform.translation += projectile.velocity * delta;
    }
}

/// Checks for ice projectile collisions with ground or walls, spawns explosions.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_ice_projectile_collisions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    projectiles: Query<(Entity, &Transform, &IceProjectile)>,
    walls: Query<&WallOfStone>,
) {
    for (entity, transform, projectile) in projectiles.iter() {
        let projectile_pos = transform.translation;

        // Check wall collision
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(projectile_pos) && projectile_pos.y <= wall.height {
                // Hit wall - spawn explosion at wall surface
                let explosion_pos = Vec3::new(projectile_pos.x, wall.height, projectile_pos.z);
                spawn_ice_explosion(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    explosion_pos,
                    projectile.explosion_radius,
                    projectile.damage,
                    projectile.empowerment,
                );
                commands.entity(entity).despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Check ground collision (Y <= 0)
        if projectile_pos.y <= 0.0 {
            // Hit ground - spawn explosion at ground level
            let explosion_pos = Vec3::new(projectile_pos.x, 0.0, projectile_pos.z);
            spawn_ice_explosion(
                &mut commands,
                &mut meshes,
                &mut materials,
                explosion_pos,
                projectile.explosion_radius,
                projectile.damage,
                projectile.empowerment,
            );
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns an ice explosion at the given position.
fn spawn_ice_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    max_radius: f32,
    damage: f32,
    empowerment: f32,
) {
    // Create a 2D circle mesh that lies flat on the ground
    let circle = Circle::new(1.0); // Unit circle, scaled by transform

    // Position slightly above battlefield (y=1) to avoid z-fighting
    let explosion_pos = Vec3::new(position.x, 1.0, position.z);

    commands.spawn((
        Mesh3d(meshes.add(circle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: EXPLOSION_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(explosion_pos)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)) // Rotate to lie flat
            .with_scale(Vec3::splat(0.1)),
        IceExplosion::new(position, max_radius, damage, empowerment),
        NetworkedSpellEffect { kind: SpellEffectKind::IceExplosion },
        OnGameplayScreen,
    ));
}

/// Updates explosion visuals and applies damage.
pub(super) fn update_ice_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut IceExplosion, &mut Transform)>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<IceExplosion>,
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
                        DamageType::Frost,
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
