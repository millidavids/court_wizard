use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{WallHealth, WallOfStone, WallOfStoneCaster, WallOfStonePreview};
use super::constants::*;
use crate::config::GameConfig;
use crate::config::save_data::SavedWall;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{FlowFieldVelocity, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::plugin::GlobalAttackCycle;
use crate::game::units::components::{AttackTiming, Corpse, Hitbox, TargetingVelocity};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{clamp_to_spell_range, get_cursor_world_position};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (wall was placed).
    completed: bool,
    /// Whether preview should be despawned (release with too-short drag or no mana).
    despawn_preview: bool,
    /// Obstacle bounds for network sync (set when completed=true).
    obstacle_bounds: Option<[f32; 4]>,
    /// Center position of the placed wall (for sound effects).
    wall_center: Option<Vec3>,
}

/// Local wizard Wall of Stone casting — reads mouse input, manages preview.
#[allow(clippy::too_many_arguments)]
pub fn handle_wall_of_stone_casting(
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut caster_query: Query<&mut WallOfStoneCaster>,
    mut preview_query: Query<&mut Transform, (With<WallOfStonePreview>, Without<Wizard>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut connection: Option<ResMut<crate::networking::resources::NetworkConnection>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
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
    if primed_spell.spell != Spell::WallOfStone {
        return;
    }

    let mut caster = if let Ok(c) = caster_query.get_mut(wizard_entity) {
        c
    } else {
        commands
            .entity(wizard_entity)
            .insert(WallOfStoneCaster::new());
        return;
    };

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, SPELL_ORIGIN, wizard.spell_range));

    let cast_result = wall_of_stone_casting_logic(
        &input,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut caster,
        &mut commands,
        &visual_assets,
        &mut obstacle_events,
    );

    // Send wall placement over network so the other client updates pathfinding
    if cast_result.completed
        && let Some(bounds) = cast_result.obstacle_bounds
        && let Some(ref mut conn) = connection
    {
        conn.outgoing_messages
            .push(crate::networking::protocol::NetworkMessage::WallPlaced {
                bounds,
                placed: true,
            });
    }

    // Local-only: manage preview

    // Handle preview spawning on cast start
    if caster.anchor.is_some()
        && caster.preview_entity.is_none()
        && let Some(pos) = clamped_pos
    {
        let preview_entity = commands
            .spawn((
                Mesh3d(visual_assets.unit_cuboid.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WALL_PREVIEW_COLOR,
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_xyz(pos.x, WALL_HEIGHT / 2.0, pos.z).with_scale(Vec3::new(
                    0.0,
                    WALL_HEIGHT,
                    WALL_WIDTH,
                )),
                WallOfStonePreview,
                OnGameplayScreen,
            ))
            .id();

        caster.preview_entity = Some(preview_entity);
    }

    // Update preview during casting
    if matches!(*casting_state, CastingState::Casting { .. })
        && let Some(anchor) = caster.anchor
        && let Some(preview_entity) = caster.preview_entity
        && let Ok(mut preview_transform) = preview_query.get_mut(preview_entity)
        && let Some(pos) = clamped_pos
    {
        let diff = Vec3::new(pos.x - anchor.x, 0.0, pos.z - anchor.z);
        let length = diff.length().min(MAX_WALL_LENGTH);

        if length > 0.1 {
            let forward = diff.normalize();
            let center = anchor + forward * (length / 2.0);
            let rotation = Quat::from_rotation_arc(Vec3::X, forward);

            preview_transform.translation = Vec3::new(center.x, WALL_HEIGHT / 2.0, center.z);
            preview_transform.rotation = rotation;
            preview_transform.scale = Vec3::new(length, WALL_HEIGHT, WALL_WIDTH);
        }
    }

    // Despawn preview on completion or cancel
    if cast_result.completed || cast_result.despawn_preview {
        if let Some(preview_entity) = caster.preview_entity {
            commands.entity(preview_entity).try_despawn();
        }
        caster.preview_entity = None;
    }

    if cast_result.completed {
        if let Some(center) = cast_result.wall_center {
            audio::play_sfx(
                &mut commands,
                &sfx.wall_of_stone_cast,
                center,
                &game_config,
                &sfx,
            );
        }
        mouse_state.left_consumed = true;
    }
}

/// Core Wall of Stone casting logic.
#[allow(clippy::too_many_arguments)]
fn wall_of_stone_casting_logic(
    input: &WizardInput,
    clamped_pos: Option<Vec3>,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster: &mut WallOfStoneCaster,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) -> CastResult {
    let mut result = CastResult {
        completed: false,
        despawn_preview: false,
        obstacle_bounds: None,
        wall_center: None,
    };

    let Some(clamped_pos) = clamped_pos else {
        return result;
    };

    // Handle release — place wall or cancel
    if input.just_released {
        if let Some(anchor) = caster.anchor {
            let diff = Vec3::new(clamped_pos.x - anchor.x, 0.0, clamped_pos.z - anchor.z);
            let length = diff.length();

            if length >= MIN_WALL_LENGTH && mana.can_afford(MANA_COST) {
                let clamped_length = length.min(MAX_WALL_LENGTH);
                let forward = diff.normalize();
                let right = Vec3::new(-forward.z, 0.0, forward.x);
                let center = anchor + forward * (clamped_length / 2.0);

                mana.consume(MANA_COST);

                // Spawn the actual wall
                let rotation = Quat::from_rotation_arc(Vec3::X, forward);

                // Apply empowerment scaling
                let scale = primed_spell.empowerment;
                let wall_width = WALL_WIDTH * scale;
                let wall_height = WALL_HEIGHT * scale;

                let wall = WallOfStone {
                    center,
                    half_length: clamped_length / 2.0,
                    half_width: wall_width / 2.0,
                    forward,
                    right,
                    height: wall_height,
                    time_alive: 0.0,
                    duration: f32::MAX,
                    sinking: false,
                    empowerment: primed_spell.empowerment,
                    permanent: true,
                };

                let obs_bounds = wall.obstacle_bounds();

                commands.spawn((
                    Mesh3d(assets.unit_cuboid.clone()),
                    MeshMaterial3d(assets.wall_of_stone.clone()),
                    Transform::from_xyz(center.x, wall_height / 2.0, center.z)
                        .with_rotation(rotation)
                        .with_scale(Vec3::new(clamped_length, wall_height, wall_width)),
                    wall,
                    WallHealth::new(WALL_HEALTH),
                    NetworkedSpellEffect {
                        kind: SpellEffectKind::WallOfStone,
                    },
                    OnGameplayScreen,
                ));

                obstacle_events.write(ObstacleChanged {
                    bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                    obstacle_type: ObstacleType::Blocked,
                    shape: Some(ObstacleShape::obb_from_center(
                        center,
                        forward,
                        clamped_length / 2.0,
                        wall_width / 2.0,
                    )),
                });

                result.completed = true;
                result.obstacle_bounds = Some(obs_bounds);
                result.wall_center = Some(center);
            } else {
                // Too short or can't afford — signal preview despawn
                result.despawn_preview = true;
            }

            caster.anchor = None;
            casting_state.cancel();
        }
        return result;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(MANA_COST) {
                caster.anchor = Some(clamped_pos);
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Preview update is handled by the local wrapper only
        }
        _ => {}
    }

    result
}

/// Handles right-click cancellation of wall placement.
pub fn handle_wall_of_stone_cancel(
    mut mouse_right_pressed: MessageReader<crate::game::input::messages::MouseRightPressed>,
    mut commands: Commands,
    mut wizard_query: Query<&mut CastingState, With<LocalWizard>>,
    mut caster_query: Query<&mut WallOfStoneCaster, With<LocalWizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    if mouse_right_pressed.read().next().is_none() {
        return;
    }

    let Ok(mut casting_state) = wizard_query.single_mut() else {
        return;
    };

    let Ok(mut caster) = caster_query.single_mut() else {
        return;
    };

    if let Some(preview_entity) = caster.preview_entity {
        commands.entity(preview_entity).try_despawn();
    }

    caster.anchor = None;
    caster.preview_entity = None;
    casting_state.cancel();
    mouse_state.left_consumed = true;
}

/// Advances wall lifetime and triggers sinking phase (skips permanent walls).
pub fn tick_wall_lifetime(time: Res<Time>, mut walls: Query<&mut WallOfStone>) {
    let delta = time.delta_secs();
    for mut wall in &mut walls {
        if wall.permanent {
            continue;
        }
        wall.time_alive += delta;
        if !wall.sinking && wall.time_alive >= wall.duration - WALL_SINK_DURATION {
            wall.sinking = true;
        }
    }
}

/// Animates walls sinking into the ground during their final seconds.
pub fn animate_sinking_walls(mut walls: Query<(&WallOfStone, &mut Transform)>) {
    for (wall, mut transform) in &mut walls {
        if wall.sinking {
            let sink_elapsed = wall.time_alive - (wall.duration - WALL_SINK_DURATION);
            let sink_progress = (sink_elapsed / WALL_SINK_DURATION).clamp(0.0, 1.0);
            let target_y = wall.height / 2.0 - wall.height * sink_progress;
            transform.translation.y = target_y;
        }
    }
}

/// Despawns walls that have exceeded their duration (skips permanent walls).
pub fn cleanup_expired_walls(
    mut commands: Commands,
    walls: Query<(Entity, &WallOfStone)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut connection: Option<ResMut<crate::networking::resources::NetworkConnection>>,
) {
    for (entity, wall) in &walls {
        if wall.permanent {
            continue;
        }
        if wall.time_alive >= wall.duration {
            commands.entity(entity).try_despawn();

            // Notify pathfinding system that the obstacle is removed
            let obs_bounds = wall.obstacle_bounds();
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::obb_from_center(
                    wall.center,
                    wall.forward,
                    wall.half_length,
                    wall.half_width,
                )),
            });

            // Notify remote peer to update their pathfinding grid
            if let Some(ref mut conn) = connection {
                conn.outgoing_messages.push(
                    crate::networking::protocol::NetworkMessage::WallPlaced {
                        bounds: obs_bounds,
                        placed: false,
                    },
                );
            }
        }
    }
}

/// Spawns a permanent wall entity from saved wall data.
pub(crate) fn spawn_permanent_wall(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    saved: &SavedWall,
) {
    let forward = Vec3::new(saved.forward_x, 0.0, saved.forward_z);
    let right = Vec3::new(-forward.z, 0.0, forward.x);
    let center = Vec3::new(saved.center_x, 0.0, saved.center_z);
    let rotation = Quat::from_rotation_arc(Vec3::X, forward);

    commands.spawn((
        Mesh3d(assets.unit_cuboid.clone()),
        MeshMaterial3d(assets.wall_of_stone.clone()),
        Transform::from_xyz(center.x, saved.height / 2.0, center.z)
            .with_rotation(rotation)
            .with_scale(Vec3::new(
                saved.half_length * 2.0,
                saved.height,
                saved.half_width * 2.0,
            )),
        WallOfStone {
            center,
            half_length: saved.half_length,
            half_width: saved.half_width,
            forward,
            right,
            height: saved.height,
            time_alive: 0.0,
            duration: f32::MAX,
            sinking: false,
            empowerment: saved.empowerment,
            permanent: true,
        },
        WallHealth::new(WALL_HEALTH),
        NetworkedSpellEffect {
            kind: SpellEffectKind::WallOfStone,
        },
        OnGameplayScreen,
    ));
}

/// Registers pathfinding obstacles for all permanent walls after loading completes.
pub(crate) fn register_permanent_wall_obstacles(
    walls: Query<&WallOfStone>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for wall in &walls {
        if !wall.permanent {
            continue;
        }
        let obs_bounds = wall.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
            obstacle_type: ObstacleType::Blocked,
            shape: Some(ObstacleShape::obb_from_center(
                wall.center,
                wall.forward,
                wall.half_length,
                wall.half_width,
            )),
        });
    }
}

/// Units with no valid path (pathfinding_distance == INFINITY) move toward the
/// king and attack any wall they end up pressed against. This prevents players
/// from exploiting wall placement to permanently trap units — blocked attackers
/// naturally converge on the walls surrounding the king rather than scattering
/// to the nearest wall on the map.
pub fn units_attack_blocking_walls(
    attack_cycle: Res<GlobalAttackCycle>,
    mut blocked_units: Query<
        (
            &Transform,
            &Hitbox,
            &FlowFieldVelocity,
            &mut TargetingVelocity,
            &mut AttackTiming,
        ),
        (Without<Corpse>, Without<WallOfStone>),
    >,
    king_query: Query<&Transform, With<crate::game::units::king::components::King>>,
    mut walls: Query<(Entity, &WallOfStone, &mut WallHealth)>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - crate::game::constants::APPROX_FRAME_TIME).max(0.0);

    let king_pos = king_query.iter().next().map(|t| t.translation);

    for (transform, hitbox, flow_vel, mut targeting_vel, mut attack_timing) in &mut blocked_units {
        // Only target walls if this unit has no valid path
        if !flow_vel.pathfinding_distance.is_infinite() {
            continue;
        }

        let unit_pos = transform.translation;

        // Move toward the king — wall collision will stop the unit at the
        // blocking wall, causing units to pile up where they need to attack.
        if let Some(king) = king_pos {
            let diff = Vec3::new(king.x - unit_pos.x, 0.0, king.z - unit_pos.z);
            targeting_vel.velocity = diff.normalize_or_zero();
        }

        // Find nearest wall by distance to surface for melee damage
        let mut nearest_wall_entity = None;
        let mut nearest_distance = f32::MAX;

        for (entity, wall, _) in walls.iter() {
            let dist = wall.distance_to_surface(unit_pos);
            if dist < nearest_distance {
                nearest_distance = dist;
                nearest_wall_entity = Some(entity);
            }
        }

        // Deal damage if close enough to a wall
        let attack_range = hitbox.radius + WALL_ATTACK_RANGE;
        if let Some(wall_entity) = nearest_wall_entity
            && nearest_distance <= attack_range
            && attack_timing.can_attack(current_time, last_time)
            && let Ok((_, _, mut wall_health)) = walls.get_mut(wall_entity)
        {
            wall_health.take_damage(WALL_DAMAGE_PER_HIT);
            attack_timing.record_attack(current_time);
        }
    }
}

/// Destroys walls that have lost all HP by triggering the existing sink + cleanup pipeline.
pub fn destroy_dead_walls(
    mut walls: Query<(&mut WallOfStone, &WallHealth)>,
) {
    for (mut wall, wall_health) in &mut walls {
        if wall_health.is_dead() && !wall.sinking {
            // Enter sinking phase — existing tick_wall_lifetime + cleanup_expired_walls
            // will handle the rest (obstacle removal, despawn, network sync).
            wall.sinking = true;
            wall.permanent = false;
            wall.duration = wall.time_alive + WALL_SINK_DURATION;
        }
    }
}

/// Tints wall material from base color to damaged color based on remaining HP.
///
/// On first damage, clones the shared material into a per-wall instance so
/// tinting one wall doesn't affect others.
pub fn update_wall_damage_tint(
    mut walls: Query<(&WallHealth, &mut MeshMaterial3d<StandardMaterial>), With<WallOfStone>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
) {
    let base = WALL_BASE_COLOR.to_srgba();
    let damaged = WALL_DAMAGED_COLOR.to_srgba();

    for (wall_health, mut material_handle) in &mut walls {
        if wall_health.current >= wall_health.max {
            continue;
        }

        // If still using the shared material, clone it into a per-wall instance
        if material_handle.0 == visual_assets.wall_of_stone {
            let Some(shared_mat) = materials.get(&visual_assets.wall_of_stone) else {
                continue;
            };
            let cloned = shared_mat.clone();
            material_handle.0 = materials.add(cloned);
        }

        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        // Lerp from damaged color (0 HP) to base color (full HP)
        let hp_frac = wall_health.fraction();
        let r = damaged.red + (base.red - damaged.red) * hp_frac;
        let g = damaged.green + (base.green - damaged.green) * hp_frac;
        let b = damaged.blue + (base.blue - damaged.blue) * hp_frac;
        material.base_color = Color::srgba(r, g, b, 1.0);
    }
}
