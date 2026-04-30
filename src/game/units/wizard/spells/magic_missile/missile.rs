//! Magic missile movement, collision, and detonation.

use bevy::prelude::*;
use rand::Rng;

use super::super::super::components::Spell;
use super::components::*;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

#[allow(clippy::too_many_arguments)]
pub fn move_magic_missiles(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut missiles: Query<(&mut Transform, &mut MagicMissile)>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystal_transforms: Query<&Transform, (With<ArcaneCrystal>, Without<MagicMissile>)>,
    camera_query_3d: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut sparkle_timer: Local<f32>,
) {
    // Track sparkle spawn timing
    *sparkle_timer += time.delta_secs();
    let should_spawn_sparkles = *sparkle_timer >= vfx::constants::SPARKLE_SPAWN_INTERVAL;
    if should_spawn_sparkles {
        *sparkle_timer -= vfx::constants::SPARKLE_SPAWN_INTERVAL;
    }

    // Get cursor world position once for guided missiles
    let cursor_world_pos = get_cursor_world_position(&camera_query_3d, &corrected_cursor);

    for (mut missile_transform, mut missile) in &mut missiles {
        missile.time_alive += time.delta_secs();

        // Guided missiles: steer toward cursor, no target homing
        if missile.guided {
            let base_max_speed = missile.current_max_speed();

            if let Some(cursor_pos) = cursor_world_pos {
                let to_cursor = cursor_pos - missile_transform.translation;
                let distance = to_cursor.length();

                let proximity_speed_multiplier = if distance < constants::SLOWDOWN_DISTANCE {
                    let t = (distance / constants::SLOWDOWN_DISTANCE).clamp(0.0, 1.0);
                    let min_multiplier = constants::MIN_PROXIMITY_SPEED / base_max_speed;
                    min_multiplier + (1.0 - min_multiplier) * t
                } else {
                    1.0
                };

                let max_speed = base_max_speed * proximity_speed_multiplier;
                let direction = to_cursor.normalize_or_zero();

                // Smooth steering: blend current velocity toward cursor direction
                let steer_strength = 8.0;
                let desired_velocity = direction * max_speed;
                let current_velocity = missile.velocity;
                missile.velocity += (desired_velocity - current_velocity)
                    * (steer_strength * time.delta_secs()).min(1.0);

                let current_speed = missile.velocity.length();
                if current_speed > max_speed {
                    missile.velocity = missile.velocity.normalize() * max_speed;
                }
            }
            // If no cursor position, continue with current velocity

            missile_transform.translation += missile.velocity * time.delta_secs();

            // Spawn sparkle trail
            if should_spawn_sparkles {
                vfx::systems::spawn_missile_sparkles(
                    &mut commands,
                    &visual_assets,
                    missile_transform.translation,
                    missile.velocity,
                    time.elapsed_secs(),
                );
            }
            continue;
        }

        // Normal homing logic for non-guided missiles
        // Check if current target still exists (could be a unit or a crystal)
        let target_exists = missile.target.is_some_and(|target_entity| {
            targets.get(target_entity).is_ok() || crystal_transforms.get(target_entity).is_ok()
        });

        // Retarget if current target despawned
        if !target_exists {
            let rng = &mut game_rng.0;

            let enemies_in_range: Vec<Entity> = targets
                .iter()
                .filter(|(_, _, team)| missile.target_teams.matches(team))
                .filter(|(_, transform, _)| {
                    let distance = missile_transform
                        .translation
                        .distance(transform.translation);
                    distance <= missile.spell_range
                })
                .map(|(entity, _, _)| entity)
                .collect();

            missile.target = if !enemies_in_range.is_empty() {
                let index = rng.random_range(0..enemies_in_range.len());
                Some(enemies_in_range[index])
            } else {
                targets
                    .iter()
                    .filter(|(_, _, team)| missile.target_teams.matches(team))
                    .min_by(|a, b| {
                        let dist_a = missile_transform.translation.distance(a.1.translation);
                        let dist_b = missile_transform.translation.distance(b.1.translation);
                        dist_a
                            .partial_cmp(&dist_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(entity, _, _)| entity)
            };
        }

        // Get current target's transform (check units first, then crystals)
        let target_transform = missile.target.and_then(|target_entity| {
            targets
                .get(target_entity)
                .ok()
                .map(|(_, transform, _)| transform)
                .or_else(|| crystal_transforms.get(target_entity).ok())
        });

        if let Some(target_transform) = target_transform {
            let to_target = target_transform.translation - missile_transform.translation;
            let distance_to_target = to_target.length();
            let current_homing_strength = missile.current_homing_strength();

            let base_max_speed = missile.current_max_speed();

            let proximity_speed_multiplier = if distance_to_target < constants::SLOWDOWN_DISTANCE {
                let t = (distance_to_target / constants::SLOWDOWN_DISTANCE).clamp(0.0, 1.0);
                let min_multiplier = constants::MIN_PROXIMITY_SPEED / base_max_speed;
                min_multiplier + (1.0 - min_multiplier) * t
            } else {
                1.0
            };

            let max_speed = base_max_speed * proximity_speed_multiplier;

            let homing_force = if current_homing_strength.is_infinite() {
                to_target.normalize_or_zero()
            } else {
                to_target.normalize_or_zero() * current_homing_strength
            };

            let wobble = if missile.time_alive < constants::PERFECT_TRACKING_TIME {
                let t = missile.time_alive * constants::WOBBLE_FREQUENCY + missile.wobble_offset;

                Vec3::new(
                    t.sin() * constants::WOBBLE_AMPLITUDE,
                    (t * constants::WOBBLE_Y_FREQ_MULTIPLIER).cos()
                        * constants::WOBBLE_AMPLITUDE
                        * constants::WOBBLE_Y_AMPLITUDE_MULTIPLIER,
                    (t * constants::WOBBLE_Z_FREQ_MULTIPLIER).sin() * constants::WOBBLE_AMPLITUDE,
                )
            } else {
                Vec3::ZERO
            };

            if current_homing_strength.is_infinite() {
                missile.velocity = homing_force * max_speed;
            } else {
                missile.velocity += (homing_force + wobble) * time.delta_secs();

                let current_speed = missile.velocity.length();
                if current_speed > max_speed {
                    missile.velocity = missile.velocity.normalize() * max_speed;
                }
            }

            missile_transform.translation += missile.velocity * time.delta_secs();
        } else {
            missile_transform.translation += missile.velocity * time.delta_secs();
        }

        // Spawn sparkle trail particles
        if should_spawn_sparkles {
            vfx::systems::spawn_missile_sparkles(
                &mut commands,
                &visual_assets,
                missile_transform.translation,
                missile.velocity,
                time.elapsed_secs(),
            );
        }
    }
}

/// Checks for magic missile collisions with enemies (Attackers and Undead).
///
/// When a missile hits an enemy, it deals 50 damage and despawns.
#[allow(clippy::too_many_arguments)]
pub fn check_magic_missile_collisions(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut missiles: Query<(Entity, &Transform, &mut MagicMissile)>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &Team,
            Has<SpellShield>,
        ),
        (Without<MagicMissile>, Without<Corpse>),
    >,
    walls: Query<&WallOfStone>,
    rocks: Query<&crate::game::terrain::boulder::components::Boulder>,
    trees: Query<&crate::game::terrain::tree::components::Tree>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    visual_assets: Res<SpellVisualAssets>,
) {
    // Collect split spawns to avoid borrow conflicts
    let mut splits: Vec<(Vec3, MagicMissile)> = Vec::new();

    for (missile_entity, missile_transform, mut missile) in &mut missiles {
        // Wall collision
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(missile_transform.translation)
                && missile_transform.translation.y <= wall.height
            {
                commands.entity(missile_entity).try_despawn();
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
            if rock.blocks_projectile(missile_transform.translation) {
                commands.entity(missile_entity).try_despawn();
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
            if tree.blocks_projectile(missile_transform.translation) {
                commands.entity(missile_entity).try_despawn();
                hit_tree = true;
                break;
            }
        }
        if hit_tree {
            continue;
        }

        let mut should_despawn = false;
        let mut target_killed = false;
        for (enemy_entity, enemy_transform, mut health, mut temp_hp, team, has_spell_shield) in
            &mut enemies
        {
            if !missile.target_teams.matches(team) {
                continue;
            }

            let distance = missile_transform
                .translation
                .distance(enemy_transform.translation);

            // Check collision
            if distance < missile.radius {
                if has_spell_shield {
                    continue;
                }
                apply_spell_damage(
                    &mut commands,
                    enemy_entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    missile.damage,
                    DamageType::Force,
                    false,
                );
                target_killed = health.current <= 0.0;
                if let Some(ref mut progress) = talent_progress {
                    progress.increment(Spell::MagicMissile, 1);
                }

                // Arcane Detonation: spawn small AoE on impact
                if missile.detonation {
                    spawn_missile_detonation(
                        &mut commands,
                        &visual_assets,
                        missile_transform.translation,
                        missile.damage * 0.2,
                    );
                }

                // Piercing: pass through first target
                if missile.piercing && missile.pierced_count < 1 {
                    missile.pierced_count += 1;
                    // Retarget to a different enemy
                    missile.target = None;
                    continue;
                }

                should_despawn = true;
                break;
            }
        }

        // Seeker Swarm: split into 2 half-damage missiles on kill (max 2 generations)
        if should_despawn && target_killed && missile.seeker_swarm && missile.split_generation < 2 {
            let rng = &mut game_rng.0;
            for _ in 0..2 {
                let mut split = MagicMissile::new(
                    Vec3::new(
                        rng.random_range(-1000.0..1000.0),
                        rng.random_range(-500.0..500.0),
                        rng.random_range(-1000.0..1000.0),
                    ),
                    rng.random_range(0.0..std::f32::consts::TAU),
                    None, // Will retarget automatically
                    missile.empowerment,
                    missile.target_teams,
                    missile.spell_range,
                    missile_transform.translation,
                );
                split.damage = missile.damage * 0.2;
                split.radius = missile.radius;
                split.piercing = missile.piercing;
                split.detonation = missile.detonation;
                split.seeker_swarm = true;
                split.split_generation = missile.split_generation + 1;
                split.guided = missile.guided;
                splits.push((missile_transform.translation, split));
            }
        }

        if should_despawn {
            commands.entity(missile_entity).try_despawn();
        }
    }

    // Spawn split missiles outside the query loop
    for (pos, split_missile) in splits {
        let radius_mult =
            split_missile.radius / (constants::COLLISION_RADIUS * split_missile.empowerment);
        let entity = commands
            .spawn((
                Mesh3d(visual_assets.magic_missile_mesh.clone()),
                MeshMaterial3d(visual_assets.magic_missile.clone()),
                Transform::from_translation(pos),
                split_missile,
                OnGameplayScreen,
            ))
            .id();

        vfx::systems::spawn_missile_glow(
            &mut commands,
            &visual_assets,
            entity,
            pos,
            constants::COLLISION_RADIUS * radius_mult,
        );
    }
}

/// Spawns a small AoE detonation effect for the Arcane Detonation talent.
/// Uses pink magic missile material instead of fire material, deals Force damage (no burning).
fn spawn_missile_detonation(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    damage: f32,
) {
    use crate::game::units::DamageType;
    use crate::game::units::wizard::spells::fireball::components::FireballExplosion;

    let mut explosion = FireballExplosion::new(
        position,
        40.0, // small radius
        damage,
        DamageType::Force,
        1.0,
    );
    explosion.source_spell = Spell::MagicMissile;

    commands.spawn((
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.magic_missile.clone()),
        Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
        explosion,
        OnGameplayScreen,
    ));
}

/// Despawns magic missiles that exit their spell range or have been alive too long.
pub fn despawn_distant_magic_missiles(
    mut commands: Commands,
    missiles: Query<(Entity, &Transform, &MagicMissile)>,
) {
    for (entity, transform, missile) in &missiles {
        let distance_from_origin = transform.translation.distance(missile.origin_pos);
        if distance_from_origin > missile.spell_range || missile.time_alive > 5.0 {
            commands.entity(entity).try_despawn();
        }
    }
}
