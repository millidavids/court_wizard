use std::collections::HashSet;

use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::OgreAssets;
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::StagingAttacker;
use crate::game::pathfinding::resources::PathfindingGrid;
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::utils::{
    animate_telegraph_material, despawn_indicators, indicator_rotation,
};
use crate::game::units::components::{
    BanishedModifier, Corpse, FrozenSolidModifier, Health, Hitbox, Knockback, RootedModifier,
    SickenedModifier, SleepModifier, Sleepwalking, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::components::Wizard;

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
            Without<Wizard>,
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
            Without<Wizard>,
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
