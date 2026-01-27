use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Wizard};
use super::components::*;
use super::constants;
use super::styles::arc_color;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::MouseButtonState;
use crate::game::input::events::MouseLeftReleased;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_damage_to_unit,
};

/// Handles chain lightning casting with left-click.
///
/// Left-click starts cast. Must hold for full cast time.
/// After cast completes, finds enemy under cursor and spawns chain lightning bolt.
/// Only casts when ChainLightning is the primed spell.
///
/// Note: Spell priming, input blocking, and mouse state checks are handled by run_if conditions.
#[allow(clippy::too_many_arguments)]
pub fn handle_chain_lightning_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<(&mut CastingState, &mut Mana, &PrimedSpell), With<Wizard>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    enemies_query: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
) {
    let Ok((mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };

    // Check for release event - this is spell-specific logic
    if mouse_left_released.read().next().is_some() {
        // Cancel cast on release
        casting_state.cancel();
        return;
    }

    // Mouse is held - handle casting based on state
    match *casting_state {
        CastingState::Channeling { .. } => {
            // Chain Lightning doesn't channel - just cancel
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - consume mana and find initial target
                if mana.consume(constants::MANA_COST)
                    && let Some(cursor_pos) =
                        get_cursor_world_position(&camera_query, &window_query)
                {
                    // Find enemy near cursor
                    if let Some((target_entity, target_pos)) =
                        find_target_near_position(cursor_pos, &enemies_query)
                    {
                        let wizard_pos =
                            WIZARD_POSITION + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);

                        // Apply initial damage
                        if let Ok((mut health, mut temp_hp)) = health_query.get_mut(target_entity) {
                            apply_damage_to_unit(
                                &mut health,
                                temp_hp.as_deref_mut(),
                                constants::INITIAL_DAMAGE,
                            );
                        }

                        // Spawn first arc from wizard to target
                        spawn_arc(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            wizard_pos,
                            target_pos,
                        );

                        // Spawn chain lightning bolt to track bouncing
                        commands.spawn((
                            ChainLightningBolt {
                                hit_entities: vec![target_entity],
                                current_damage: constants::INITIAL_DAMAGE
                                    * constants::DAMAGE_FALLOFF,
                                bounces_remaining: constants::MAX_BOUNCES,
                                last_hit_position: target_pos,
                                bounce_delay_timer: constants::BOUNCE_DELAY,
                            },
                            OnGameplayScreen,
                        ));
                    }
                }

                // Return to resting state (no channeling)
                casting_state.cancel();
                mouse_state.left_consumed = true; // Require release before next cast
            }
        }
        CastingState::Resting => {
            // Not casting - check mana before starting cast
            if mana.can_afford(constants::MANA_COST) {
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
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

/// Finds the closest enemy near the given position within TARGETING_RADIUS.
/// Note: position should be at Y=0 (battlefield plane). Uses XZ distance for targeting.
/// Targets all living units (defenders, attackers, and undead) but excludes corpses.
fn find_target_near_position(
    position: Vec3,
    enemies: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
) -> Option<(Entity, Vec3)> {
    // Use XZ distance only (ignore Y difference) for targeting
    let target_pos_2d = Vec3::new(position.x, 0.0, position.z);

    enemies
        .iter()
        // No team filter - spell damages ALL units indiscriminately
        .filter(|(_, transform, _)| {
            let unit_pos_2d = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
            let distance = target_pos_2d.distance(unit_pos_2d);
            distance <= constants::TARGETING_RADIUS
        })
        .min_by(|a, b| {
            let a_pos_2d = Vec3::new(a.1.translation.x, 0.0, a.1.translation.z);
            let b_pos_2d = Vec3::new(b.1.translation.x, 0.0, b.1.translation.z);
            let dist_a = target_pos_2d.distance(a_pos_2d);
            let dist_b = target_pos_2d.distance(b_pos_2d);
            dist_a.partial_cmp(&dist_b).unwrap()
        })
        .map(|(entity, transform, _)| (entity, transform.translation))
}

/// Spawns a lightning arc visual between two points.
fn spawn_arc(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start: Vec3,
    end: Vec3,
) {
    let midpoint = (start + end) / 2.0;
    let direction = (end - start).normalize();
    let length = start.distance(end);

    // Create a rectangle mesh for the arc
    let rectangle = Rectangle::new(constants::ARC_WIDTH, constants::ARC_WIDTH);

    // Calculate rotation to align Y axis with direction
    let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

    commands.spawn((
        ChainLightningArc {
            start,
            end,
            lifetime: constants::ARC_LIFETIME,
            time_alive: 0.0,
        },
        Mesh3d(meshes.add(rectangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: arc_color(),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(midpoint)
            .with_rotation(rotation)
            .with_scale(Vec3::new(1.0, length / constants::ARC_WIDTH, 1.0)),
        OnGameplayScreen,
    ));
}

/// Processes chain lightning bounces to nearby enemies.
/// Targets all living units (defenders, attackers, and undead) but excludes corpses.
pub fn process_chain_lightning_bounces(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bolts: Query<(Entity, &mut ChainLightningBolt)>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
) {
    for (bolt_entity, mut bolt) in &mut bolts {
        // Decrement bounce delay timer
        bolt.bounce_delay_timer -= time.delta_secs();

        // Check if it's time to bounce
        if bolt.bounce_delay_timer <= 0.0 && bolt.bounces_remaining > 0 {
            // Find next bounce target
            if let Some((target_entity, target_pos)) =
                find_next_bounce_target(bolt.last_hit_position, &bolt.hit_entities, &enemies)
            {
                // Apply damage to target
                if let Ok((_, _, _, mut health, mut temp_hp)) = enemies.get_mut(target_entity) {
                    apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), bolt.current_damage);
                }

                // Spawn arc from last position to new target
                spawn_arc(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    bolt.last_hit_position,
                    target_pos,
                );

                // Update bolt state
                bolt.hit_entities.push(target_entity);
                bolt.current_damage *= constants::DAMAGE_FALLOFF;
                bolt.last_hit_position = target_pos;
                bolt.bounces_remaining -= 1;
                bolt.bounce_delay_timer = constants::BOUNCE_DELAY;
            } else {
                // No valid targets - end chain
                bolt.bounces_remaining = 0;
            }
        }

        // Despawn bolt if no more bounces
        if bolt.bounces_remaining == 0 && bolt.bounce_delay_timer <= 0.0 {
            commands.entity(bolt_entity).despawn();
        }
    }
}

/// Finds the closest enemy within bounce range that hasn't been hit yet.
/// Targets all living units (defenders, attackers, and undead) but excludes corpses.
fn find_next_bounce_target(
    origin: Vec3,
    hit_entities: &[Entity],
    enemies: &Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
) -> Option<(Entity, Vec3)> {
    enemies
        .iter()
        // No team filter - spell damages ALL units indiscriminately
        .filter(|(entity, _, _, _, _)| !hit_entities.contains(entity))
        .filter(|(_, transform, _, _, _)| {
            origin.distance(transform.translation) <= constants::BOUNCE_RANGE
        })
        .min_by(|a, b| {
            let dist_a = origin.distance(a.1.translation);
            let dist_b = origin.distance(b.1.translation);
            dist_a.partial_cmp(&dist_b).unwrap()
        })
        .map(|(entity, transform, _, _, _)| (entity, transform.translation))
}

/// Updates chain lightning arc visuals with pulsing animation.
pub fn update_chain_lightning_arcs(
    time: Res<Time>,
    mut arcs: Query<(
        &mut ChainLightningArc,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut arc, material_handle) in &mut arcs {
        // Update timers
        arc.time_alive += time.delta_secs();
        arc.lifetime -= time.delta_secs();

        // Calculate pulsing intensity
        let intensity = 0.7 + 0.3 * (arc.time_alive * 20.0).sin();

        // Update material color with pulsing effect
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let base = arc_color();
            material.base_color = Color::srgba(
                base.to_srgba().red * intensity,
                base.to_srgba().green * intensity,
                base.to_srgba().blue * intensity,
                base.to_srgba().alpha,
            );
        }
    }
}

/// Cleans up chain lightning arcs that have expired.
pub fn cleanup_chain_lightning(mut commands: Commands, arcs: Query<(Entity, &ChainLightningArc)>) {
    for (entity, arc) in &arcs {
        if arc.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
