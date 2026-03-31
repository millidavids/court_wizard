use bevy::prelude::*;
use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{
    CollapseExploded, DispelledWall, LivingStoneTracker, PermafrostAuraTimer, WallHealth,
    WallOfStone, WallOfStoneCaster, WallOfStonePreview, WallOfStoneTalentParams, WallRising,
    WallTalents,
};
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
use crate::game::units::components::{AttackTiming, Corpse, Hitbox, SlowMovementModifier, TargetingVelocity, Team};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{clamp_to_spell_range, get_cursor_world_position};
use crate::game::units::wizard::spells::vfx;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> WallOfStoneTalentParams {
    let mut params = WallOfStoneTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::WallOfStone, 0);
    let t2 = talents.get_selection(Spell::WallOfStone, 1);
    let t3 = talents.get_selection(Spell::WallOfStone, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Quarry Master
            params.mana_mult = QUARRY_MASTER_MANA_MULT;
            params.max_length_mult = QUARRY_MASTER_LENGTH_MULT;
        }
        Some(1) => {
            // Reinforced Stone
            params.health_mult = REINFORCED_STONE_HEALTH_MULT;
            params.width_mult = REINFORCED_STONE_WIDTH_MULT;
        }
        Some(2) => {
            // Quick Foundations
            params.quick_foundations = true;
            params.mana_mult = QUICK_FOUNDATIONS_MANA_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.jagged_stone = true,
        Some(1) => params.permafrost_aura = true,
        Some(2) => params.living_stone = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.collapsing_wall = true,
        Some(1) => {
            params.terraformer = true;
        }
        Some(2) => {
            params.maze_architect = true;
            params.mana_mult *= MAZE_ARCHITECT_MANA_MULT;
        }
        _ => {}
    }

    params
}

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
    time: Res<Time>,
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
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut caster_query: Query<&mut WallOfStoneCaster>,
    mut preview_query: Query<&mut Transform, (With<WallOfStonePreview>, Without<Wizard>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut connection: Option<ResMut<crate::networking::resources::NetworkConnection>>,
    (sfx, game_config, active_talents, mut talent_progress): (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
    ),
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
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

    let talent_params = compute_talent_params(active_talents.as_deref());

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
        &talent_params,
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
        let max_length = MAX_WALL_LENGTH * talent_params.max_length_mult;
        let length = diff.length().min(max_length);

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
        vfx::systems::spawn_school_flare(&mut commands, &visual_assets, vfx::systems::SpellSchool::Force, time.elapsed_secs());
        // Track talent progress (count walls placed, not casts)
        let walls_placed: u32 = if talent_params.quick_foundations { 2 } else { 1 };
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::WallOfStone, walls_placed);
        }

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
    talent_params: &WallOfStoneTalentParams,
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

    let mana_cost = MANA_COST * talent_params.mana_mult;
    let max_length = MAX_WALL_LENGTH * talent_params.max_length_mult;
    let wall_count = if talent_params.quick_foundations { 2u32 } else { 1 };
    let total_mana_cost = mana_cost * wall_count as f32;

    // Handle release — place wall or cancel
    if input.just_released {
        if let Some(anchor) = caster.anchor {
            let diff = Vec3::new(clamped_pos.x - anchor.x, 0.0, clamped_pos.z - anchor.z);
            let length = diff.length();

            if length >= MIN_WALL_LENGTH && mana.can_afford(total_mana_cost) {
                let clamped_length = length.min(max_length);
                let forward = diff.normalize();
                let right = Vec3::new(-forward.z, 0.0, forward.x);

                mana.consume(total_mana_cost);

                // Apply empowerment scaling
                let scale = primed_spell.empowerment;
                let wall_width = WALL_WIDTH * talent_params.width_mult * scale;
                let wall_height = WALL_HEIGHT * scale;
                let wall_health = WALL_HEALTH * talent_params.health_mult;

                // Walls are temporary by default; Terraformer makes them permanent
                let (permanent, duration) = if talent_params.terraformer {
                    (true, f32::MAX)
                } else {
                    (false, DEFAULT_WALL_DURATION)
                };

                // Quick Foundations: split into two walls end-to-end
                let segment_length = clamped_length / wall_count as f32;

                for i in 0..wall_count {
                    let segment_start = anchor + forward * (segment_length * i as f32);
                    let center = segment_start + forward * (segment_length / 2.0);
                    let rotation = Quat::from_rotation_arc(Vec3::X, forward);

                    let wall = WallOfStone {
                        center,
                        half_length: segment_length / 2.0,
                        half_width: wall_width / 2.0,
                        forward,
                        right,
                        height: wall_height,
                        time_alive: 0.0,
                        duration,
                        sinking: false,
                        empowerment: primed_spell.empowerment,
                        permanent,
                    };

                    let obs_bounds = wall.obstacle_bounds();

                    let mut entity_commands = commands.spawn((
                        Mesh3d(assets.unit_cuboid.clone()),
                        MeshMaterial3d(assets.wall_of_stone.clone()),
                        Transform::from_xyz(center.x, wall_height / 2.0, center.z)
                            .with_rotation(rotation)
                            .with_scale(Vec3::new(segment_length, wall_height, wall_width)),
                        wall,
                        WallHealth::new(wall_health),
                        WallTalents(talent_params.clone()),
                        NetworkedSpellEffect {
                            kind: SpellEffectKind::WallOfStone,
                        },
                        OnGameplayScreen,
                    ));

                    // Wall rises from the ground
                    entity_commands.insert(WallRising::new(WALL_RISE_DURATION));

                    // Add Living Stone tracker if talent is active
                    if talent_params.living_stone {
                        entity_commands.insert(LivingStoneTracker::new());
                    }

                    obstacle_events.write(ObstacleChanged {
                        bounds: Rect::new(
                            obs_bounds[0],
                            obs_bounds[1],
                            obs_bounds[2],
                            obs_bounds[3],
                        ),
                        obstacle_type: ObstacleType::Blocked,
                        shape: Some(ObstacleShape::obb_from_center(
                            center,
                            forward,
                            segment_length / 2.0,
                            wall_width / 2.0,
                        )),
                        rebuild: false,
                    });

                    // Use the last wall's bounds for network sync
                    result.obstacle_bounds = Some(obs_bounds);
                    result.wall_center = Some(center);
                }

                result.completed = true;
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
            if (input.just_pressed || input.pressed) && mana.can_afford(total_mana_cost) {
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
                rebuild: false,
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
            rebuild: false,
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
            &mut crate::game::units::components::Health,
            Option<&mut crate::game::units::components::TemporaryHitPoints>,
        ),
        (Without<Corpse>, Without<WallOfStone>),
    >,
    king_query: Query<&Transform, With<crate::game::units::king::components::King>>,
    mut walls: Query<(Entity, &WallOfStone, &mut WallHealth, Option<&WallTalents>, Option<&mut LivingStoneTracker>)>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - crate::game::constants::APPROX_FRAME_TIME).max(0.0);

    let king_pos = king_query.iter().next().map(|t| t.translation);

    for (transform, hitbox, flow_vel, mut targeting_vel, mut attack_timing, mut health, temp_hp) in &mut blocked_units {
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

        for (entity, wall, _, _, _) in walls.iter() {
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
            && let Ok((_, _, mut wall_health, wall_talents, living_stone_tracker)) = walls.get_mut(wall_entity)
        {
            wall_health.take_damage(WALL_DAMAGE_PER_HIT);
            attack_timing.record_attack(current_time);

            // Reset Living Stone regen timer on damage
            if let Some(mut tracker) = living_stone_tracker {
                tracker.time_since_last_damage = 0.0;
            }

            // Jagged Stone: reflect damage back to attacker
            if let Some(talents) = wall_talents
                && talents.0.jagged_stone
            {
                crate::game::units::components::apply_damage_to_unit(
                    &mut health,
                    temp_hp.map(|t| t.into_inner()),
                    JAGGED_STONE_REFLECT_DAMAGE,
                );
            }
        }
    }
}

/// Processes walls marked for dispel — starts the sinking animation.
pub fn handle_dispelled_walls(
    mut commands: Commands,
    mut walls: Query<(Entity, &mut WallOfStone, &DispelledWall)>,
) {
    for (entity, mut wall, dispelled) in &mut walls {
        if !wall.sinking {
            wall.sinking = true;
            wall.permanent = false;
            wall.duration = wall.time_alive + dispelled.sink_duration;
        }
        commands.entity(entity).remove::<DispelledWall>();
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

// =============================================================================
// Talent Systems
// =============================================================================

/// Permafrost Aura: slows enemies within range of any wall that has the talent.
pub fn apply_permafrost_aura(
    time: Res<Time>,
    mut timer: ResMut<PermafrostAuraTimer>,
    walls: Query<(&WallOfStone, &WallTalents), Without<Corpse>>,
    mut enemies: Query<
        (
            &Transform,
            &Team,
            Option<&mut SlowMovementModifier>,
            Entity,
        ),
        Without<Corpse>,
    >,
    mut commands: Commands,
) {
    timer.0 += time.delta_secs();
    if timer.0 < PERMAFROST_AURA_TICK_INTERVAL {
        return;
    }
    timer.0 = 0.0;

    // Collect wall positions that have the permafrost aura talent
    let frost_walls: Vec<_> = walls
        .iter()
        .filter(|(_, talents)| talents.0.permafrost_aura)
        .map(|(wall, _)| wall.center)
        .collect();

    if frost_walls.is_empty() {
        return;
    }

    let radius_sq = PERMAFROST_AURA_RADIUS * PERMAFROST_AURA_RADIUS;

    for (transform, team, slow_mod, entity) in &mut enemies {
        // Only slow attackers and undead
        if *team == Team::Defenders {
            continue;
        }

        let pos = transform.translation;
        let in_range = frost_walls
            .iter()
            .any(|center| {
                let dx = pos.x - center.x;
                let dz = pos.z - center.z;
                dx * dx + dz * dz <= radius_sq
            });

        if in_range {
            if let Some(mut existing) = slow_mod {
                existing.apply(PERMAFROST_AURA_SLOW, PERMAFROST_AURA_SLOW_DURATION);
            } else {
                commands.entity(entity).insert(SlowMovementModifier::new(
                    PERMAFROST_AURA_SLOW,
                    PERMAFROST_AURA_SLOW_DURATION,
                ));
            }
        }
    }
}

/// Living Stone: regenerates wall HP when not being attacked recently.
pub fn regenerate_living_stone(
    time: Res<Time>,
    mut walls: Query<(&mut WallHealth, &mut LivingStoneTracker), With<WallOfStone>>,
) {
    let delta = time.delta_secs();
    for (mut health, mut tracker) in &mut walls {
        tracker.time_since_last_damage += delta;

        if tracker.time_since_last_damage >= LIVING_STONE_REGEN_DELAY
            && health.current < health.max
        {
            let regen = health.max * LIVING_STONE_REGEN_FRACTION * delta;
            health.current = (health.current + regen).min(health.max);
        }
    }
}

/// Collapsing Wall: deals AoE damage when a wall is destroyed.
/// Uses `CollapseExploded` marker to ensure each wall only explodes once.
pub fn collapsing_wall_explosion(
    mut commands: Commands,
    walls: Query<(Entity, &WallOfStone, &WallHealth, &WallTalents), Without<CollapseExploded>>,
    mut enemies: Query<
        (
            &Transform,
            &Team,
            &mut crate::game::units::components::Health,
            Option<&mut crate::game::units::components::TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
) {
    let radius_sq = COLLAPSING_WALL_RADIUS * COLLAPSING_WALL_RADIUS;

    for (entity, wall, health, talents) in &walls {
        if !talents.0.collapsing_wall || !health.is_dead() || !wall.sinking {
            continue;
        }

        // Mark as exploded so we don't fire again
        commands.entity(entity).insert(CollapseExploded);

        let center = wall.center;

        for (transform, team, mut unit_health, temp_hp) in &mut enemies {
            if *team == Team::Defenders {
                continue;
            }
            let dx = transform.translation.x - center.x;
            let dz = transform.translation.z - center.z;
            if dx * dx + dz * dz <= radius_sq {
                crate::game::units::components::apply_damage_to_unit(
                    &mut unit_health,
                    temp_hp.map(|t| t.into_inner()),
                    COLLAPSING_WALL_DAMAGE,
                );
            }
        }
    }
}

/// Maze Architect: when 3+ walls exist, boost all wall max HP.
/// Runs every frame to adjust wall health as walls are placed or destroyed.
pub fn maze_architect_bonus(
    mut walls: Query<(&WallTalents, &mut WallHealth), With<WallOfStone>>,
) {
    // Single pass: count walls and check for maze talent simultaneously
    let mut wall_count = 0usize;
    let mut has_maze = false;
    for (talents, _) in walls.iter() {
        wall_count += 1;
        if talents.0.maze_architect {
            has_maze = true;
        }
    }

    if !has_maze {
        return;
    }

    let bonus_active = wall_count >= MAZE_ARCHITECT_WALL_THRESHOLD;

    for (talents, mut health) in &mut walls {
        let base = WALL_HEALTH * talents.0.health_mult;
        let expected_max = if bonus_active {
            base * MAZE_ARCHITECT_HEALTH_MULT
        } else {
            base
        };

        // Only adjust if the max HP doesn't match expectation
        if (health.max - expected_max).abs() > 0.1 {
            let hp_fraction = health.fraction();
            health.max = expected_max;
            health.current = expected_max * hp_fraction;
        }
    }
}

// =============================================================================
// VFX Systems
// =============================================================================

/// Animates walls rising up from the ground when first placed.
/// Moves the wall from below ground to its final position over WALL_RISE_DURATION.
pub fn animate_rising_walls(
    mut commands: Commands,
    time: Res<Time>,
    mut walls: Query<(Entity, &WallOfStone, &mut WallRising, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, wall, mut rising, mut transform) in &mut walls {
        rising.elapsed += delta;
        let progress = rising.progress();

        // Ease-out: starts fast, slows at the top
        let eased = 1.0 - (1.0 - progress) * (1.0 - progress);

        // Move wall from underground to final height
        let final_y = wall.height / 2.0;
        transform.translation.y = final_y * eased - wall.height * (1.0 - eased);

        if progress >= 1.0 {
            // Snap to final position and remove rising component
            transform.translation.y = final_y;
            commands.entity(entity).remove::<WallRising>();
        }
    }
}

/// Spawns dust puffs along walls that are rising or sinking.
pub fn spawn_wall_dust(
    mut commands: Commands,
    rising_walls: Query<(&WallOfStone, &WallRising)>,
    sinking_walls: Query<&WallOfStone, Without<WallRising>>,
    visual_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < WALL_DUST_INTERVAL {
        return;
    }
    *timer -= WALL_DUST_INTERVAL;

    let t = time.elapsed_secs();

    // Spawn dust for rising walls
    for (wall, _rising) in &rising_walls {
        spawn_dust_along_wall(&mut commands, &visual_assets, wall, t);
    }

    // Spawn dust for sinking walls
    for wall in &sinking_walls {
        if wall.sinking {
            spawn_dust_along_wall(&mut commands, &visual_assets, wall, t);
        }
    }
}

/// Helper: spawns dust puffs distributed along a wall's length.
fn spawn_dust_along_wall(
    commands: &mut Commands,
    assets: &crate::game::units::wizard::spells::visual_assets::SpellVisualAssets,
    wall: &WallOfStone,
    time_secs: f32,
) {
    let wall_len = wall.half_length * 2.0;
    let num_points = ((wall_len / 50.0) as usize).max(2);

    for j in 0..num_points {
        let frac = (j as f32 + (time_secs * 2.3 + j as f32 * 1.7).fract()) / num_points as f32;
        let pos = wall.center - wall.forward * wall.half_length
            + wall.forward * (wall_len * frac.clamp(0.0, 1.0));

        crate::game::units::wizard::spells::vfx::systems::spawn_dust_smoke(
            commands,
            assets,
            pos,
            wall.half_width,
            WALL_DUST_PUFFS_PER_POINT,
            time_secs + j as f32,
        );
    }
}
