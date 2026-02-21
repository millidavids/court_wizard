use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, LocalWizard};
use super::components::*;
use super::constants;
use super::styles::{arc_color_at_depth, arc_width_at_depth};
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::lightning_rod::LightningRod;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

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
    mut wizard_query: Query<(&mut CastingState, &mut Mana, &PrimedSpell), With<LocalWizard>>,
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

                        // Scale damage by empowerment
                        let initial_damage = primed_spell.scale(constants::INITIAL_DAMAGE);

                        // Apply initial damage
                        if let Ok((mut health, mut temp_hp)) = health_query.get_mut(target_entity) {
                            apply_spell_damage(
                                &mut commands,
                                target_entity,
                                &mut health,
                                temp_hp.as_deref_mut(),
                                initial_damage,
                                constants::DAMAGE_TYPE,
                            );
                        }

                        // Spawn first arc from wizard to target (depth 0 for initial arc)
                        spawn_arc(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            wizard_pos,
                            target_pos,
                            0,
                            primed_spell.empowerment,
                        );

                        // Spawn shared hit tracking group
                        let group_entity = commands
                            .spawn((
                                ChainLightningGroup {
                                    hit_entities: vec![target_entity],
                                },
                                OnGameplayScreen,
                            ))
                            .id();

                        // Spawn chain lightning bolt to track splitting
                        commands.spawn((
                            ChainLightningBolt {
                                group_entity,
                                current_damage: initial_damage * constants::DAMAGE_FALLOFF,
                                damage_type: constants::DAMAGE_TYPE,
                                bounces_remaining: constants::MAX_BOUNCES,
                                last_hit_position: target_pos,
                                bounce_delay_timer: primed_spell.scale(constants::BOUNCE_DELAY),
                                empowerment: primed_spell.empowerment,
                                split_depth: 0,
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

/// Spawns a lightning arc visual between two points as a parabolic curve.
/// The arc rises upward at the midpoint and comes back down, creating a
/// natural lightning-through-the-sky effect.
fn spawn_arc(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start: Vec3,
    end: Vec3,
    depth: u32,
    empowerment: f32,
) {
    let arc_width = arc_width_at_depth(depth, empowerment);
    let color = arc_color_at_depth(depth);
    let segments = constants::ARC_SEGMENTS;

    // Calculate peak height based on horizontal distance and depth.
    // Depth 0 (initial bolt from wizard) is a straight line.
    // Each subsequent depth arcs progressively higher.
    let horizontal_dist = Vec3::new(start.x - end.x, 0.0, start.z - end.z).length();
    let height_factor = constants::ARC_HEIGHT_FACTOR + constants::ARC_HEIGHT_GROWTH * depth as f32;
    let peak_height = if depth == 0 {
        0.0
    } else {
        horizontal_dist * height_factor
    };

    // Pre-create shared mesh and material
    let rectangle = Rectangle::new(arc_width, arc_width);
    let mesh_handle = meshes.add(rectangle);
    let material_handle = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    });

    // Spawn segments along a parabolic curve
    for i in 0..segments {
        let t0 = i as f32 / segments as f32;
        let t1 = (i + 1) as f32 / segments as f32;

        let p0 = arc_point(start, end, peak_height, t0);
        let p1 = arc_point(start, end, peak_height, t1);

        let seg_midpoint = (p0 + p1) / 2.0;
        let seg_direction = (p1 - p0).normalize();
        let seg_length = p0.distance(p1);

        let rotation = Quat::from_rotation_arc(Vec3::Y, seg_direction);

        commands.spawn((
            ChainLightningArc {
                start: p0,
                end: p1,
                lifetime: constants::ARC_LIFETIME,
                time_alive: 0.0,
                depth,
            },
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material_handle.clone()),
            Transform::from_translation(seg_midpoint)
                .with_rotation(rotation)
                .with_scale(Vec3::new(1.0, seg_length / arc_width, 1.0)),
            OnGameplayScreen,
        ));
    }
}

/// Calculates a point along a parabolic arc between start and end.
/// t ranges from 0.0 (start) to 1.0 (end). The arc peaks at t=0.5.
fn arc_point(start: Vec3, end: Vec3, peak_height: f32, t: f32) -> Vec3 {
    // Linear interpolation for the base position
    let base = start.lerp(end, t);
    // Parabolic height: 4 * h * t * (1 - t) peaks at h when t = 0.5
    let height_offset = 4.0 * peak_height * t * (1.0 - t);
    Vec3::new(base.x, base.y + height_offset, base.z)
}

/// Processes chain lightning bounces with binary splitting.
/// Each bolt finds up to SPLIT_COUNT targets and spawns child bolts for each.
/// All bolts from the same cast share a hit list via ChainLightningGroup.
/// Lightning rods are valid targets — hitting one triggers an immediate strike pulse.
#[allow(clippy::too_many_arguments)]
pub fn process_chain_lightning_bounces(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bolts: Query<(Entity, &mut ChainLightningBolt)>,
    mut groups: Query<&mut ChainLightningGroup>,
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
    mut rods: Query<(Entity, &Transform, &mut LightningRod)>,
    crystals: Query<(Entity, &Transform), With<ArcaneCrystal>>,
    walls: Query<&WallOfStone>,
) {
    // Collect bolt data to avoid borrow conflicts when spawning child bolts
    let mut bolts_to_process: Vec<(Entity, ChainLightningBoltSnapshot)> = Vec::new();

    for (bolt_entity, mut bolt) in &mut bolts {
        bolt.bounce_delay_timer -= time.delta_secs();

        if bolt.bounce_delay_timer <= 0.0 && bolt.bounces_remaining > 0 {
            bolts_to_process.push((
                bolt_entity,
                ChainLightningBoltSnapshot {
                    group_entity: bolt.group_entity,
                    current_damage: bolt.current_damage,
                    damage_type: bolt.damage_type,
                    bounces_remaining: bolt.bounces_remaining,
                    last_hit_position: bolt.last_hit_position,
                    empowerment: bolt.empowerment,
                    split_depth: bolt.split_depth,
                },
            ));
            // Mark as done so it gets cleaned up
            bolt.bounces_remaining = 0;
        }

        // Despawn bolt if no more bounces and timer expired
        if bolt.bounces_remaining == 0 && bolt.bounce_delay_timer <= 0.0 {
            commands.entity(bolt_entity).despawn();
        }
    }

    // Process collected bolts
    for (bolt_entity, snapshot) in bolts_to_process {
        // Look up shared hit list
        let Ok(mut group) = groups.get_mut(snapshot.group_entity) else {
            // Group was despawned (shouldn't happen), clean up bolt
            commands.entity(bolt_entity).try_despawn();
            continue;
        };

        let bounce_range = constants::BOUNCE_RANGE * snapshot.empowerment;

        // Find up to SPLIT_COUNT targets (units + lightning rods + crystals)
        let targets = find_next_bounce_targets(
            snapshot.last_hit_position,
            &group.hit_entities,
            &enemies,
            &rods,
            &crystals,
            bounce_range,
            constants::SPLIT_COUNT,
            &walls,
        );

        for (target_entity, target_pos) in &targets {
            // Check if this target is a lightning rod
            if let Ok((_, _, mut rod)) = rods.get_mut(*target_entity) {
                // Trigger an immediate lightning strike on the rod
                rod.time_since_strike = f32::MAX;
            } else {
                // Apply damage to unit
                if let Ok((_, _, _, mut health, mut temp_hp)) = enemies.get_mut(*target_entity) {
                    apply_spell_damage(
                        &mut commands,
                        *target_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        snapshot.current_damage,
                        snapshot.damage_type,
                    );
                }
            }

            // Add to shared hit list
            group.hit_entities.push(*target_entity);

            // Spawn arc visual
            spawn_arc(
                &mut commands,
                &mut meshes,
                &mut materials,
                snapshot.last_hit_position,
                *target_pos,
                snapshot.split_depth + 1,
                snapshot.empowerment,
            );

            // Spawn child bolt if more bounces remain
            if snapshot.bounces_remaining > 1 {
                commands.spawn((
                    ChainLightningBolt {
                        group_entity: snapshot.group_entity,
                        current_damage: snapshot.current_damage * constants::DAMAGE_FALLOFF,
                        damage_type: snapshot.damage_type,
                        bounces_remaining: snapshot.bounces_remaining - 1,
                        last_hit_position: *target_pos,
                        bounce_delay_timer: constants::BOUNCE_DELAY * snapshot.empowerment,
                        empowerment: snapshot.empowerment,
                        split_depth: snapshot.split_depth + 1,
                    },
                    OnGameplayScreen,
                ));
            }
        }
    }
}

/// Snapshot of bolt data for deferred processing.
struct ChainLightningBoltSnapshot {
    group_entity: Entity,
    current_damage: f32,
    damage_type: DamageType,
    bounces_remaining: u32,
    last_hit_position: Vec3,
    empowerment: f32,
    split_depth: u32,
}

/// Finds up to `max_targets` enemies, lightning rods, or arcane crystals within bounce range
/// that haven't been hit yet. Targets all living units (defenders, attackers, and undead),
/// lightning rods, and arcane crystals, but excludes corpses.
/// Filters out targets blocked by WallOfStone line of sight.
#[allow(clippy::too_many_arguments)]
fn find_next_bounce_targets(
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
    rods: &Query<(Entity, &Transform, &mut LightningRod)>,
    crystals: &Query<(Entity, &Transform), With<ArcaneCrystal>>,
    bounce_range: f32,
    max_targets: usize,
    walls: &Query<&WallOfStone>,
) -> Vec<(Entity, Vec3)> {
    let mut candidates: Vec<(Entity, Vec3, f32)> = enemies
        .iter()
        // No team filter - spell damages ALL units indiscriminately
        .filter(|(entity, _, _, _, _)| !hit_entities.contains(entity))
        .filter_map(|(entity, transform, _, _, _)| {
            let distance = origin.distance(transform.translation);
            if distance <= bounce_range {
                // Check wall line-of-sight
                let blocked = walls.iter().any(|wall| {
                    wall.line_segment_intersects(origin, transform.translation)
                        .is_some()
                });
                if !blocked {
                    Some((entity, transform.translation, distance))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Also consider lightning rods as valid targets
    for (entity, transform, _) in rods.iter() {
        if hit_entities.contains(&entity) {
            continue;
        }
        let distance = origin.distance(transform.translation);
        if distance <= bounce_range {
            let blocked = walls.iter().any(|wall| {
                wall.line_segment_intersects(origin, transform.translation)
                    .is_some()
            });
            if !blocked {
                candidates.push((entity, transform.translation, distance));
            }
        }
    }

    // Also consider arcane crystals as valid targets
    for (entity, transform) in crystals.iter() {
        if hit_entities.contains(&entity) {
            continue;
        }
        let distance = origin.distance(transform.translation);
        if distance <= bounce_range {
            let blocked = walls.iter().any(|wall| {
                wall.line_segment_intersects(origin, transform.translation)
                    .is_some()
            });
            if !blocked {
                candidates.push((entity, transform.translation, distance));
            }
        }
    }

    // Sort by distance (closest first)
    candidates.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    // Take up to max_targets
    candidates
        .into_iter()
        .take(max_targets)
        .map(|(entity, pos, _)| (entity, pos))
        .collect()
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

        // Update material color with pulsing effect (using depth-scaled base color)
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let base = arc_color_at_depth(arc.depth);
            let base_srgba = base.to_srgba();
            material.base_color = Color::srgba(
                base_srgba.red * intensity,
                base_srgba.green * intensity,
                base_srgba.blue * intensity,
                base_srgba.alpha,
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

/// Cleans up chain lightning groups that have no remaining bolts.
pub fn cleanup_chain_lightning_groups(
    mut commands: Commands,
    groups: Query<Entity, With<ChainLightningGroup>>,
    bolts: Query<&ChainLightningBolt>,
) {
    for group_entity in &groups {
        let has_bolts = bolts.iter().any(|bolt| bolt.group_entity == group_entity);
        if !has_bolts {
            commands.entity(group_entity).despawn();
        }
    }
}
