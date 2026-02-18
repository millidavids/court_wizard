use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, SpellCaster, Wizard};
use super::components::*;
use super::constants;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleType};
use crate::game::units::components::{Health, Team, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Handles fireball casting with left-click.
///
/// Left-click starts cast. Must hold for full cast time.
/// After cast completes, spawns a single fireball projectile toward the cursor.
/// Only casts when Fireball is the primed spell.
///
/// Note: Spell priming, input blocking, and mouse state checks are handled by run_if conditions.
#[allow(clippy::too_many_arguments)]
pub fn handle_fireball_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<(Entity, &mut CastingState, &mut Mana, &PrimedSpell), With<Wizard>>,
    caster_query: Query<&SpellCaster, With<Wizard>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut()
    else {
        return;
    };

    // Check for release event - this is spell-specific logic
    if mouse_left_released.read().next().is_some() {
        // Cancel cast on release and remove marker
        if caster_query.get(wizard_entity).is_ok() {
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    // Mouse is held - handle casting based on state
    match *casting_state {
        CastingState::Channeling { .. } => {
            // Fireball doesn't channel - just cancel
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - consume mana and spawn fireball
                if mana.consume(constants::MANA_COST)
                    && let Some(target_pos) =
                        get_cursor_world_position(&camera_query, &window_query)
                {
                    spawn_fireball(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        WIZARD_POSITION + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0),
                        target_pos,
                        primed_spell,
                    );
                }
                // Remove caster marker immediately (don't keep it blocking future casts)
                commands.entity(wizard_entity).remove::<SpellCaster>();
                // Return to resting state (no channeling for fireball)
                casting_state.cancel();
                // Consume mouse to require release before next cast
                mouse_state.left_consumed = true;
            }
        }
        CastingState::Resting => {
            // Only start if we don't have a caster marker and have enough mana
            if caster_query.get(wizard_entity).is_err() && mana.can_afford(constants::MANA_COST) {
                // Mark wizard as casting
                commands.entity(wizard_entity).insert(SpellCaster::new());
                casting_state.start_cast();
            }
        }
    }
}

/// Gets the cursor position projected onto the battlefield surface (Y=0 plane).
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    // Create a ray from the camera through the cursor position
    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    // Intersect ray with Y=0 plane (battlefield surface)
    // Ray equation: origin + direction * t
    // Plane equation: y = 0
    // Solve for t: origin.y + direction.y * t = 0
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

/// Spawns a fireball projectile.
fn spawn_fireball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    target: Vec3,
    primed_spell: &PrimedSpell,
) {
    let direction = (target - origin).normalize();
    let speed = primed_spell.scale(constants::PROJECTILE_SPEED);
    let velocity = direction * speed;

    let radius = primed_spell.scale(FIREBALL_RADIUS);
    let sphere = Sphere::new(radius);

    commands.spawn((
        Mesh3d(meshes.add(sphere)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: FIREBALL_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(origin),
        Fireball::new(
            velocity,
            primed_spell.scale(constants::DAMAGE_PER_TICK),
            constants::DAMAGE_TYPE,
            primed_spell.scale(constants::EXPLOSION_RADIUS),
            primed_spell.scale(constants::PROJECTILE_COLLISION_RADIUS),
            primed_spell.empowerment,
        ),
        OnGameplayScreen,
    ));
}

/// Updates fireball projectile positions based on velocity.
pub fn move_fireballs(time: Res<Time>, mut fireballs: Query<(&mut Transform, &Fireball)>) {
    for (mut transform, fireball) in &mut fireballs {
        transform.translation += fireball.velocity * time.delta_secs();
    }
}

/// Checks for fireball collisions with units or the ground.
///
/// When a fireball hits a unit or the ground, it explodes.
#[allow(clippy::too_many_arguments)]
pub fn check_fireball_collisions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    fireballs: Query<(Entity, &Transform, &Fireball)>,
    targets: Query<(&Transform, &Team)>,
    walls: Query<&WallOfStone>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (fireball_entity, fireball_transform, fireball) in &fireballs {
        let fireball_pos = fireball_transform.translation;

        // Check collision with walls
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(fireball_pos) && fireball_pos.y <= wall.height {
                let explosion_pos = fireball_pos;
                spawn_explosion(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut obstacle_events,
                    explosion_pos,
                    fireball.explosion_radius,
                    fireball.damage,
                    fireball.empowerment,
                );
                commands.entity(fireball_entity).despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Check collision with ground (Y <= 0)
        if fireball_pos.y <= 0.0 {
            // Hit ground - spawn explosion at ground level
            let explosion_pos = Vec3::new(fireball_pos.x, 0.0, fireball_pos.z);
            spawn_explosion(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut obstacle_events,
                explosion_pos,
                fireball.explosion_radius,
                fireball.damage,
                fireball.empowerment,
            );
            commands.entity(fireball_entity).despawn();
            continue;
        }

        // Check collision with units
        for (target_transform, _team) in &targets {
            let distance = fireball_pos.distance(target_transform.translation);

            if distance < fireball.radius {
                // Hit unit - spawn explosion at impact point
                spawn_explosion(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut obstacle_events,
                    fireball_pos,
                    fireball.explosion_radius,
                    fireball.damage,
                    fireball.empowerment,
                );
                commands.entity(fireball_entity).despawn();
                break;
            }
        }
    }
}

/// Spawns a fireball explosion at the given position.
#[allow(clippy::too_many_arguments)]
fn spawn_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    position: Vec3,
    max_radius: f32,
    damage: f32,
    empowerment: f32,
) {
    let sphere = Sphere::new(1.0); // Unit sphere, scaled by transform

    let buffered_radius = max_radius + OBSTACLE_BUFFER;

    // Notify pathfinding system about the explosion (100x movement cost)
    let origin_2d = Vec2::new(position.x, position.z);
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type: ObstacleType::SlowTerrain(100.0),
    });

    commands.spawn((
        Mesh3d(meshes.add(sphere)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: EXPLOSION_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
        FireballExplosion::new(
            position,
            max_radius,
            damage,
            constants::DAMAGE_TYPE,
            empowerment,
        ),
        OnGameplayScreen,
    ));
}

/// Updates explosion visuals and timing.
pub fn update_explosions(
    time: Res<Time>,
    mut explosions: Query<(&mut FireballExplosion, &mut Transform)>,
) {
    for (mut explosion, mut transform) in &mut explosions {
        explosion.time_alive += time.delta_secs();
        explosion.time_since_last_tick += time.delta_secs();

        // Calculate current radius based on time
        let current_radius = explosion.current_radius(constants::EXPLOSION_DURATION);

        // Scale the sphere mesh to match current radius
        // The mesh is a unit sphere (radius 1.0), so we scale it
        transform.scale = Vec3::splat(current_radius);
    }
}

/// Applies damage to units hit by the explosion on a tick interval.
///
/// Targets closer to the center stay in the explosion longer and take more damage.
pub fn apply_explosion_damage(
    mut commands: Commands,
    mut explosions: Query<&mut FireballExplosion>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
    )>,
) {
    for mut explosion in &mut explosions {
        // Check if it's time for a damage tick
        if explosion.time_since_last_tick >= constants::DAMAGE_TICK_INTERVAL {
            explosion.time_since_last_tick = 0.0;

            let current_radius = explosion.current_radius(constants::EXPLOSION_DURATION);

            // Apply damage to all units within the current explosion radius
            for (entity, transform, mut health, mut temp_hp) in &mut targets {
                let distance = explosion.origin.distance(transform.translation);

                if distance <= current_radius {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage_per_tick,
                        constants::DAMAGE_TYPE,
                    );
                }
            }
        }
    }
}

/// Cleans up explosions that have finished animating.
pub fn cleanup_finished_explosions(
    mut commands: Commands,
    explosions: Query<(Entity, &FireballExplosion)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, explosion) in &explosions {
        if explosion.time_alive >= constants::EXPLOSION_DURATION {
            // Remove the explosion obstacle
            let buffered_explosion_radius = explosion.max_radius + OBSTACLE_BUFFER;
            let origin_2d = Vec2::new(explosion.origin.x, explosion.origin.z);
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(
                    origin_2d,
                    Vec2::splat(buffered_explosion_radius * 2.0),
                ),
                obstacle_type: ObstacleType::Removed,
            });

            commands.entity(entity).despawn();
        }
    }
}

/// Despawns fireballs that travel beyond the wizard's spell range.
pub fn despawn_distant_fireballs(
    mut commands: Commands,
    fireballs: Query<(Entity, &Transform), With<Fireball>>,
    wizard_query: Query<(&Transform, &Wizard), Without<Fireball>>,
) {
    let Ok((wizard_transform, wizard)) = wizard_query.single() else {
        return;
    };

    let wizard_pos = wizard_transform.translation;
    let spell_range = wizard.spell_range;

    for (entity, transform) in &fireballs {
        let distance_from_wizard = transform.translation.distance(wizard_pos);

        if distance_from_wizard > spell_range {
            commands.entity(entity).despawn();
        }
    }
}
