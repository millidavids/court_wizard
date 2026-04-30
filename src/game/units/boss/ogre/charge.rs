//! Ogre charge attack and visuals.

use super::combat::enrage_phase_tint;
use std::collections::HashSet;

use bevy::math::Affine2;
use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::OgreAssets;
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::StagingAttacker;
use crate::game::pathfinding::resources::PathfindingGrid;
use crate::game::terrain::boulder::constants::{ROCK_THROW_COOLDOWN, ROCK_THROW_RANGE};
use crate::game::terrain::boulder::messages::BoulderThrownMessage;
use crate::game::units::boss::components::Boss;
use crate::game::units::brute::components::RockThrowCooldown;
use crate::game::units::components::{
    BanishedModifier, Corpse, FrozenSolidModifier, Health, Hitbox, Knockback, PolymorphedModifier,
    RootedModifier, SickenedModifier, SleepModifier, Sleepwalking, Team, TemporaryHitPoints,
    apply_damage_to_unit,
};
use crate::game::units::components::{CombatAnimation, FacingDirection, WalkingAnimation};

/// Spawns the ogre at one of the tunnel spawn points.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn ogre_charge_system(
    time: Res<Time>,
    mut commands: Commands,
    ogre_assets: Res<OgreAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    pathfinding: Res<PathfindingGrid>,
    mut bosses: Query<
        (
            Entity,
            &mut Transform,
            &Team,
            &mut OgreChargeState,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        (With<Boss>, Without<Corpse>),
    >,
    potential_targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<Boss>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
            Without<OgreChargeIndicator>,
        ),
    >,
    mut charge_targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<Boss>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<OgreChargeIndicator>,
        ),
    >,
    mut indicator_query: Query<&mut Transform, (With<OgreChargeIndicator>, Without<Boss>)>,
) {
    let delta = time.delta_secs();

    for (
        _boss_entity,
        mut boss_transform,
        boss_team,
        mut charge_state,
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned, petrified),
    ) in &mut bosses
    {
        match charge_state.as_mut() {
            OgreChargeState::Idle { cooldown } => {
                *cooldown -= delta;
                if *cooldown <= 0.0 {
                    *charge_state = OgreChargeState::Targeting;
                }
            }

            OgreChargeState::Targeting => {
                let boss_pos = boss_transform.translation;

                let mut best_target: Option<(Vec3, f32)> = None;
                for (_entity, target_transform, target_team) in &potential_targets {
                    if !boss_team.is_enemy(target_team) {
                        continue;
                    }
                    let dx = target_transform.translation.x - boss_pos.x;
                    let dz = target_transform.translation.z - boss_pos.z;
                    let distance = (dx * dx + dz * dz).sqrt();

                    if !(OGRE_CHARGE_TARGET_MIN_DISTANCE..=OGRE_CHARGE_TARGET_MAX_DISTANCE)
                        .contains(&distance)
                    {
                        continue;
                    }

                    if best_target.is_none_or(|(_, d)| distance < d) {
                        best_target = Some((target_transform.translation, distance));
                    }
                }

                if let Some((target_pos, _)) = best_target {
                    let direction =
                        Vec3::new(target_pos.x - boss_pos.x, 0.0, target_pos.z - boss_pos.z)
                            .normalize_or_zero();

                    let charge_distance = OGRE_CHARGE_MAX_DISTANCE;
                    let rotation = indicator_rotation(direction);
                    let perp = Vec3::new(-direction.z, 0.0, direction.x);
                    let lane_center = Vec3::new(
                        boss_pos.x + direction.x * (charge_distance / 2.0),
                        OGRE_CHARGE_INDICATOR_Y,
                        boss_pos.z + direction.z * (charge_distance / 2.0),
                    );
                    let half_width = OGRE_CHARGE_LANE_WIDTH / 2.0;

                    let mesh = ogre_assets.charge_rect_mesh.clone();
                    let mat = ogre_assets.charge_line_material.clone();

                    // Spawn outline: left, right, near (at ogre), far (at end)
                    let spawn_line = |cmds: &mut Commands, pos: Vec3, sx: f32, sy: f32| {
                        cmds.spawn((
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(mat.clone()),
                            Transform::from_translation(pos)
                                .with_rotation(rotation)
                                .with_scale(Vec3::new(sx, sy, 1.0)),
                            OgreChargeIndicator,
                            OnGameplayScreen,
                        ))
                        .id()
                    };
                    let t = OGRE_CHARGE_LINE_THICKNESS;
                    let left = spawn_line(
                        &mut commands,
                        lane_center + perp * half_width,
                        t,
                        charge_distance,
                    );
                    let right = spawn_line(
                        &mut commands,
                        lane_center - perp * half_width,
                        t,
                        charge_distance,
                    );
                    let near = spawn_line(
                        &mut commands,
                        boss_pos.with_y(OGRE_CHARGE_INDICATOR_Y),
                        OGRE_CHARGE_LANE_WIDTH,
                        t,
                    );
                    let far_pos = boss_pos + direction * charge_distance;
                    let far = spawn_line(
                        &mut commands,
                        far_pos.with_y(OGRE_CHARGE_INDICATOR_Y),
                        OGRE_CHARGE_LANE_WIDTH,
                        t,
                    );

                    // Unique emissive material for the fill — pulsed each frame
                    let fill_material = materials.add(StandardMaterial {
                        base_color: OGRE_CHARGE_FILL_BASE_COLOR,
                        emissive: bevy::color::LinearRgba::new(0.0, 0.0, 0.0, 1.0),
                        alpha_mode: AlphaMode::Blend,
                        unlit: false,
                        ..default()
                    });

                    let fill = commands
                        .spawn((
                            Mesh3d(ogre_assets.charge_rect_mesh.clone()),
                            MeshMaterial3d(fill_material.clone()),
                            Transform::from_translation(boss_pos.with_y(OGRE_CHARGE_INDICATOR_Y))
                                .with_rotation(rotation)
                                .with_scale(Vec3::new(OGRE_CHARGE_LANE_WIDTH, 1.0, 1.0)),
                            OgreChargeIndicator,
                            OnGameplayScreen,
                        ))
                        .id();

                    *charge_state = OgreChargeState::Telegraphing {
                        elapsed: 0.0,
                        direction,
                        target_distance: charge_distance,
                        indicators: ChargeIndicators {
                            left,
                            right,
                            near,
                            far,
                            fill,
                            fill_material,
                        },
                    };
                } else {
                    *charge_state = OgreChargeState::Idle { cooldown: 2.0 };
                }
            }

            OgreChargeState::Telegraphing {
                elapsed,
                direction,
                target_distance,
                indicators,
            } => {
                if crate::game::units::systems::is_cc_immobilized(
                    rooted,
                    sleeping,
                    sleepwalking,
                    banished,
                    sickened,
                    frozen,
                    stunned,
                    petrified,
                ) {
                    despawn_indicators(&mut commands, &indicators.all());
                    *charge_state = OgreChargeState::Idle {
                        cooldown: OGRE_CHARGE_COOLDOWN / 2.0,
                    };
                    continue;
                }

                *elapsed += delta;
                let progress = (*elapsed / OGRE_CHARGE_TELEGRAPH_DURATION).min(1.0);
                let current_length = *target_distance * progress;

                let boss_pos = boss_transform.translation;
                let dir = *direction;

                // Grow the fill rectangle outward from the ogre
                if let Ok(mut fill_transform) = indicator_query.get_mut(indicators.fill) {
                    fill_transform.scale.y = current_length.max(1.0);

                    let half_length = current_length / 2.0;
                    fill_transform.translation = Vec3::new(
                        boss_pos.x + dir.x * half_length,
                        OGRE_CHARGE_INDICATOR_Y,
                        boss_pos.z + dir.z * half_length,
                    );
                }

                // Emissive glow: ramps up with progress, pulses ominously on top
                if let Some(mat) = materials.get_mut(&indicators.fill_material) {
                    animate_telegraph_material(mat, *elapsed, progress, 0.6);
                }

                if *elapsed >= OGRE_CHARGE_TELEGRAPH_DURATION {
                    despawn_indicators(&mut commands, &indicators.all());

                    // Play charge sound effect
                    crate::game::units::wizard::spells::audio::play_sfx_scaled(
                        &mut commands,
                        &ogre_assets.charge_sfx,
                        boss_transform.translation,
                        &game_config,
                        1.0,
                    );

                    *charge_state = OgreChargeState::Charging {
                        direction: dir,
                        distance_traveled: 0.0,
                        max_distance: *target_distance,
                        hit_entities: HashSet::new(),
                    };
                }
            }

            OgreChargeState::Charging {
                direction,
                distance_traveled,
                max_distance,
                hit_entities,
            } => {
                let move_delta = OGRE_CHARGE_SPEED * delta;
                let dir = *direction;

                // Check if the next position hits an obstacle (boulder, tree, wall)
                let next_pos = Vec3::new(
                    boss_transform.translation.x + dir.x * move_delta,
                    boss_transform.translation.y,
                    boss_transform.translation.z + dir.z * move_delta,
                );
                if pathfinding.sample_base_cost(next_pos) == f32::INFINITY {
                    *charge_state = OgreChargeState::Recovery {
                        timer: OGRE_CHARGE_RECOVERY_DURATION,
                    };
                    continue;
                }

                boss_transform.translation.x += dir.x * move_delta;
                boss_transform.translation.z += dir.z * move_delta;
                *distance_traveled += move_delta;

                let boss_pos = boss_transform.translation;

                for (entity, target_transform, target_hitbox, target_team, mut health, temp_hp) in
                    &mut charge_targets
                {
                    if !boss_team.is_enemy(target_team) {
                        continue;
                    }
                    if hit_entities.contains(&entity) {
                        continue;
                    }

                    let dx = target_transform.translation.x - boss_pos.x;
                    let dz = target_transform.translation.z - boss_pos.z;
                    let distance = (dx * dx + dz * dz).sqrt();
                    let hit_range = OGRE_RADIUS + target_hitbox.radius + OGRE_CHARGE_HIT_EXTRA;

                    if distance > hit_range {
                        continue;
                    }

                    apply_damage_to_unit(
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        OGRE_CHARGE_DAMAGE,
                    );
                    hit_entities.insert(entity);

                    // Knock units away perpendicular to the charge direction
                    let perp = Vec3::new(-dir.z, 0.0, dir.x);
                    let to_unit = Vec3::new(dx, 0.0, dz).normalize_or_zero();
                    let side = to_unit.dot(perp);
                    let knockback_dir = if side.abs() > 0.01 {
                        perp * side.signum()
                    } else {
                        perp
                    };

                    commands.entity(entity).insert(Knockback::new(
                        knockback_dir,
                        OGRE_CHARGE_KNOCKBACK_SPEED,
                        OGRE_CHARGE_KNOCKBACK_DURATION,
                    ));
                }

                if *distance_traveled >= *max_distance {
                    *charge_state = OgreChargeState::Recovery {
                        timer: OGRE_CHARGE_RECOVERY_DURATION,
                    };
                }
            }

            OgreChargeState::Recovery { timer } => {
                *timer -= delta;
                if *timer <= 0.0 {
                    *charge_state = OgreChargeState::Idle {
                        cooldown: OGRE_CHARGE_COOLDOWN,
                    };
                }
            }
        }
    }
}

/// Updates ogre sprite visuals during charge attack phases.
/// Swaps to attacking texture, sets the correct frame, applies red flash and vibration.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_ogre_charge_visuals(
    time: Res<Time>,
    ogre_assets: Res<OgreAssets>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Boss>)>,
    mut bosses: Query<
        (
            Entity,
            &mut Transform,
            &OgreChargeState,
            &OgreEnrageState,
            &MeshMaterial3d<StandardMaterial>,
            &WalkingAnimation,
            &mut FacingDirection,
            Option<&mut OgreChargeVisuals>,
        ),
        (With<Boss>, Without<Corpse>, Without<Camera3d>),
    >,
) {
    let cam_forward_xz = camera_query
        .single()
        .ok()
        .map(|cam| {
            let fwd = cam.forward();
            Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero()
        })
        .unwrap_or(Vec3::NEG_Z);

    let delta = time.delta_secs();

    for (
        entity,
        mut transform,
        charge_state,
        enrage_state,
        material_handle,
        walking_anim,
        mut facing,
        charge_visuals,
    ) in &mut bosses
    {
        match charge_state {
            OgreChargeState::Telegraphing {
                elapsed, direction, ..
            } => {
                // Set facing direction from charge direction
                let new_facing = facing_from_world_direction(*direction, cam_forward_xz);
                *facing = new_facing;

                if let Some(mut visuals) = charge_visuals {
                    // Ongoing telegraph — update effects
                    visuals.elapsed += delta;
                    let progress = (*elapsed / OGRE_CHARGE_TELEGRAPH_DURATION).min(1.0);

                    if let Some(mat) = materials.get_mut(&material_handle.0) {
                        // First frame: swap texture to attacking sheet
                        if !visuals.texture_swapped {
                            mat.base_color_texture = Some(ogre_assets.attacking_texture.clone());
                            visuals.texture_swapped = true;
                        }

                        // Show frame 0 (wind-up) in correct direction
                        let row = OGRE_ATTACKING_DIRECTION_ROWS[new_facing as usize];
                        mat.uv_transform = ogre_frame_uv_transform(0, row);

                        // Red flash: pulse between enrage tint and flash color
                        let flash_t =
                            (visuals.elapsed * OGRE_CHARGE_FLASH_FREQUENCY * std::f32::consts::TAU)
                                .sin()
                                * 0.5
                                + 0.5;
                        let base_tint = enrage_phase_tint(enrage_state.phase);
                        mat.base_color = Color::LinearRgba(
                            base_tint
                                .to_linear()
                                .mix(&OGRE_CHARGE_FLASH_COLOR.to_linear(), flash_t),
                        );
                    }

                    // Vibration: sinusoidal offset scaled by progress
                    let amp = OGRE_CHARGE_VIBRATION_AMPLITUDE * progress;
                    let vib_x =
                        (visuals.elapsed * OGRE_CHARGE_VIBRATION_FREQ_X * std::f32::consts::TAU)
                            .sin()
                            * amp;
                    let vib_z =
                        (visuals.elapsed * OGRE_CHARGE_VIBRATION_FREQ_Z * std::f32::consts::TAU)
                            .sin()
                            * amp;
                    transform.translation.x = visuals.base_position.x + vib_x;
                    transform.translation.z = visuals.base_position.z + vib_z;
                } else {
                    // First frame of telegraph — insert visuals component
                    // and remove any active combat/throw animations
                    commands
                        .entity(entity)
                        .remove::<CombatAnimation>()
                        .remove::<OgreThrowWindup>()
                        .insert(OgreChargeVisuals {
                            texture_swapped: false,
                            elapsed: 0.0,
                            base_position: transform.translation,
                        });
                }
            }

            OgreChargeState::Charging { direction, .. } => {
                if let Some(mut visuals) = charge_visuals
                    && let Some(mat) = materials.get_mut(&material_handle.0)
                {
                    // Restore base position on first charging frame
                    // (remove vibration offset before charge movement begins)
                    if visuals.elapsed > 0.0 {
                        transform.translation.x = visuals.base_position.x;
                        transform.translation.z = visuals.base_position.z;
                        visuals.elapsed = 0.0;
                    }

                    // Show frame 1 (charge pose)
                    let new_facing = facing_from_world_direction(*direction, cam_forward_xz);
                    *facing = new_facing;
                    let row = OGRE_ATTACKING_DIRECTION_ROWS[new_facing as usize];
                    mat.uv_transform = ogre_frame_uv_transform(1, row);

                    // Restore normal tint (stop red flash)
                    mat.base_color = enrage_phase_tint(enrage_state.phase);
                }
            }

            OgreChargeState::Recovery { .. } => {
                if charge_visuals.is_some()
                    && let Some(mat) = materials.get_mut(&material_handle.0)
                {
                    let row = OGRE_ATTACKING_DIRECTION_ROWS[*facing as usize];
                    mat.uv_transform = ogre_frame_uv_transform(2, row);
                }
            }

            OgreChargeState::Idle { .. } | OgreChargeState::Targeting => {
                // Cleanup: restore walking texture and remove visuals
                if let Some(visuals) = charge_visuals {
                    if visuals.texture_swapped
                        && let Some(mat) = materials.get_mut(&material_handle.0)
                    {
                        mat.base_color_texture = Some(ogre_assets.walking_texture.clone());
                        mat.base_color = enrage_phase_tint(enrage_state.phase);
                        // Reset UV to walking idle frame
                        mat.uv_transform = walking_anim.uv_transform(*facing);
                    }
                    // Only restore base position if vibration was still active
                    // (CC interruption during telegraph). After charging starts,
                    // elapsed is reset to 0 and the ogre has moved legitimately.
                    if visuals.elapsed > 0.0 {
                        transform.translation.x = visuals.base_position.x;
                        transform.translation.z = visuals.base_position.z;
                    }
                    commands.entity(entity).remove::<OgreChargeVisuals>();
                }
            }
        }
    }
}

/// Creates a CombatAnimation configured for the ogre's sprite sheet dimensions.
pub(super) fn ogre_combat_animation(
    direction_rows: [usize; 4],
    combat_texture: Handle<Image>,
    walking_texture: Handle<Image>,
) -> CombatAnimation {
    CombatAnimation {
        current_frame: 0,
        elapsed: 0.0,
        columns: OGRE_SPRITE_COLUMNS,
        frame_uv: OGRE_FRAME_UV,
        direction_rows,
        combat_texture,
        walking_texture,
        started: false,
    }
}

/// Returns the UV transform for a specific frame and direction row in the ogre sprite sheet.
fn ogre_frame_uv_transform(frame: usize, direction_row: usize) -> Affine2 {
    let uv_offset = Vec2::new(
        frame as f32 * OGRE_FRAME_UV.x,
        direction_row as f32 * OGRE_FRAME_UV.y,
    );
    Affine2::from_scale_angle_translation(OGRE_FRAME_UV, 0.0, uv_offset)
}

/// Derives a FacingDirection from a world-space direction vector relative to the camera.
fn facing_from_world_direction(dir: Vec3, cam_forward_xz: Vec3) -> FacingDirection {
    let cam_right = Vec3::new(-cam_forward_xz.z, 0.0, cam_forward_xz.x);
    let forward_dot = dir.dot(cam_forward_xz);
    let right_dot = dir.dot(cam_right);
    if forward_dot.abs() > right_dot.abs() {
        if forward_dot < 0.0 {
            FacingDirection::Back
        } else {
            FacingDirection::Forward
        }
    } else if right_dot > 0.0 {
        FacingDirection::Right
    } else {
        FacingDirection::Left
    }
}

/// Ogre rock throw — picks a target enemy within range and starts the throwing animation.
/// The boulder is launched when the animation finishes (see `ogre_throw_release`).
/// Skipped during charge phases or if already winding up.
#[allow(clippy::type_complexity)]
pub fn ogre_rock_throw(
    time: Res<Time>,
    ogre_assets: Res<OgreAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut commands: Commands,
    mut bosses: Query<
        (
            Entity,
            &Transform,
            &Team,
            &OgreChargeState,
            &mut RockThrowCooldown,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
                Option<&PolymorphedModifier>,
            ),
        ),
        (
            With<Boss>,
            Without<Corpse>,
            Without<OgreThrowWindup>,
            Without<CombatAnimation>,
        ),
    >,
    targets: Query<
        (&Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        entity,
        boss_transform,
        boss_team,
        charge_state,
        mut cooldown,
        (
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
            polymorphed,
        ),
    ) in &mut bosses
    {
        if charge_state.is_movement_locked() {
            continue;
        }
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) || polymorphed.is_some()
        {
            continue;
        }

        cooldown.tick(delta);
        if !cooldown.is_ready() {
            continue;
        }

        if let Some(target_pos) = crate::game::units::systems::find_closest_enemy_in_range(
            boss_transform.translation,
            boss_team,
            ROCK_THROW_RANGE,
            &targets,
        ) {
            // Play grunt sound effect
            crate::game::units::wizard::spells::audio::play_sfx_scaled(
                &mut commands,
                &ogre_assets.grunt_sfx,
                boss_transform.translation,
                &game_config,
                1.0,
            );

            // Start throwing animation and store target for release
            commands.entity(entity).insert((
                OgreThrowWindup {
                    target: target_pos,
                    sprite_index: 1,
                },
                ogre_combat_animation(
                    OGRE_THROWING_DIRECTION_ROWS,
                    ogre_assets.throwing_texture.clone(),
                    ogre_assets.walking_texture.clone(),
                ),
            ));
            cooldown.reset(ROCK_THROW_COOLDOWN);
        }
    }
}

/// Fires the boulder when the throwing animation finishes.
/// Detects completion by checking for `OgreThrowWindup` without `CombatAnimation`
/// (the shared animation system removes `CombatAnimation` when it's done).
pub fn ogre_throw_release(
    mut commands: Commands,
    mut rock_events: MessageWriter<BoulderThrownMessage>,
    bosses: Query<
        (Entity, &Transform, &OgreThrowWindup),
        (With<Boss>, Without<CombatAnimation>, Without<Corpse>),
    >,
) {
    for (entity, boss_transform, windup) in &bosses {
        rock_events.write(BoulderThrownMessage {
            origin: boss_transform.translation,
            target: windup.target,
            sprite_index: windup.sprite_index,
        });
        commands.entity(entity).remove::<OgreThrowWindup>();
    }
}

use crate::game::units::boss::utils::{
    animate_telegraph_material, despawn_indicators, indicator_rotation,
};
