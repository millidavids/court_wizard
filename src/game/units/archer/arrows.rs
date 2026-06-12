//! Archer arrows: spawn, move, collisions.

use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::ArcherAssets;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{
    Corpse, Health, Hitbox, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Spawns an arrow projectile entity at `origin` aimed at `target`.
pub(in crate::game) fn spawn_arrow(
    rng: &mut impl Rng,
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    origin: Vec3,
    target: Vec3,
    source_team: Team,
) {
    // Calculate horizontal direction and distance
    let horizontal_diff = Vec3::new(target.x - origin.x, 0.0, target.z - origin.z);
    let horizontal_distance = horizontal_diff.length();

    // Avoid division by zero
    if horizontal_distance < 0.1 {
        return;
    }

    let horizontal_direction = horizontal_diff.normalize();

    // Add random variations for realism

    // Random power variation (±5%)
    let power_multiplier = 1.0 + rng.random_range(-ARROW_POWER_VARIATION..ARROW_POWER_VARIATION);

    // Random angle variation (±1 degree)
    let angle_offset =
        rng.random_range(-ARROW_ANGLE_VARIATION_DEGREES..ARROW_ANGLE_VARIATION_DEGREES);
    let launch_angle = (ARROW_LAUNCH_ANGLE_DEGREES + angle_offset).to_radians();

    // Calculate velocity needed to hit target at launch angle, accounting for height difference.
    // Projectile equation: h = d*tan(θ) - g*d² / (2*v²*cos²(θ))
    // Solving for v: v = (d/cos(θ)) * sqrt(g / (2*(d*tan(θ) - h)))
    let height_diff = target.y - origin.y;
    let tan_theta = launch_angle.tan();
    let cos_theta = launch_angle.cos();
    let denominator = 2.0 * (horizontal_distance * tan_theta - height_diff);

    let required_speed = if denominator > 0.1 {
        (horizontal_distance / cos_theta) * (ARROW_GRAVITY / denominator).sqrt() * power_multiplier
    } else {
        // Fallback for nearly-vertical or unreachable shots: use flat-ground formula
        let sin_2theta = (2.0 * launch_angle).sin();
        ((horizontal_distance * ARROW_GRAVITY) / sin_2theta).sqrt() * power_multiplier
    };

    // Calculate velocity components
    let horizontal_velocity = horizontal_direction * required_speed * launch_angle.cos();
    let vertical_velocity = required_speed * launch_angle.sin();

    let velocity = Vec3::new(
        horizontal_velocity.x,
        vertical_velocity,
        horizontal_velocity.z,
    );

    // Spawn arrow using pre-loaded assets
    commands.spawn((
        Mesh3d(archer_assets.arrow_mesh.clone()),
        MeshMaterial3d(archer_assets.arrow_material.clone()),
        Transform::from_translation(origin),
        Arrow {
            velocity,
            damage: ARCHER_ATTACK_DAMAGE,
            source_team,
        },
        crate::game::components::Billboard,
        OnGameplayScreen,
    ));
}

/// Updates arrow positions with gravity.
pub fn move_arrows(time: Res<Time>, mut arrows: Query<(&mut Transform, &mut Arrow)>) {
    let delta = time.delta_secs();
    for (mut transform, mut arrow) in &mut arrows {
        // Apply gravity
        arrow.velocity.y -= ARROW_GRAVITY * delta;

        // Update position
        transform.translation += arrow.velocity * delta;
    }
}

/// Checks arrow collisions with units and ground.
pub fn check_arrow_collisions(
    mut commands: Commands,
    arrows: Query<(Entity, &Transform, &Arrow)>,
    mut targets: Query<
        (
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<crate::game::units::shielder::components::ShielderDamageReduction>,
            Has<crate::game::units::assassin::Assassin>,
        ),
        Without<Corpse>,
    >,
    walls: Query<&WallOfStone>,
    rocks: Query<&crate::game::terrain::boulder::components::Boulder>,
    trees: Query<&crate::game::terrain::tree::components::Tree>,
) {
    #[allow(clippy::significant_drop_in_scrutinee)]
    for (arrow_entity, arrow_transform, arrow) in &arrows {
        let arrow_pos = arrow_transform.translation;

        // Wall collision
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(arrow_pos) && arrow_pos.y <= wall.height {
                commands.entity(arrow_entity).try_despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Rock collision
        let mut hit_rock = false;
        for rock in &rocks {
            if rock.blocks_projectile(arrow_pos) {
                commands.entity(arrow_entity).try_despawn();
                hit_rock = true;
                break;
            }
        }
        if hit_rock {
            continue;
        }

        // Tree collision
        let mut hit_tree = false;
        for tree in &trees {
            if tree.blocks_projectile(arrow_pos) {
                commands.entity(arrow_entity).try_despawn();
                hit_tree = true;
                break;
            }
        }
        if hit_tree {
            continue;
        }

        // Ground collision
        if arrow_pos.y <= 0.0 {
            commands.entity(arrow_entity).try_despawn();
            continue;
        }

        // Unit collision (skip friendly fire)
        for (
            target_transform,
            hitbox,
            team,
            mut health,
            mut temp_hp,
            has_shielder_reduction,
            is_assassin,
        ) in &mut targets
        {
            // Skip non-enemies (same team, or Undead-vs-Undead)
            if !arrow.source_team.is_enemy(team) {
                continue;
            }

            // Check collision (full 3D distance — arrows are true projectiles)
            let distance = arrow_pos.distance(target_transform.translation);
            if distance < hitbox.radius + ARROW_WIDTH {
                let mut damage = arrow.damage;
                if has_shielder_reduction {
                    damage *= crate::game::units::shielder::constants::SHIELDER_DAMAGE_REDUCTION;
                }
                // Assassins take 50% less damage from archers (arrows)
                if is_assassin {
                    damage *= crate::game::units::assassin::constants::ARCHER_DAMAGE_REDUCTION;
                }
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
                commands.entity(arrow_entity).try_despawn();
                break;
            }
        }
    }
}
