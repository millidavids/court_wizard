use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::{Acceleration, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::plugin::GlobalAttackCycle;
use crate::game::units::components::{AttackTiming, Health, Hitbox, MovementSpeed, Team};
use crate::game::units::defender::components::Defender;

/// Spawns initial attackers when entering the game.
///
/// Spawns attackers clustered together, letting collision resolution push them apart.
pub fn spawn_initial_attackers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for i in 0..INITIAL_ATTACKER_COUNT {
        // Define attacker hitbox (cylinder) - this determines sprite size
        let hitbox = Hitbox::new(UNIT_RADIUS, 20.0);

        // Spawn attacker as a circle billboard sized to match the hitbox
        let circle = Circle::new(hitbox.radius);

        // Spawn all attackers within 100 pixels of each other
        let offset = i as f32 * 0.31415;
        let spawn_x = ATTACKER_SPAWN_X_MIN + (offset.sin() * 50.0 + 50.0);
        let spawn_z = ATTACKER_SPAWN_Z_MIN + (offset.cos() * 50.0 + 50.0);

        commands.spawn((
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: ATTACKER_COLOR,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(spawn_x, UNIT_Y_POSITION, spawn_z),
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Acceleration::new(),
            hitbox,
            Health::new(50.0),
            MovementSpeed::new(200.0),
            AttackTiming::new(),
            Team::Enemy,
            Attacker,
            OnGameplayScreen,
        ));
    }
}

/// Updates attacker targeting to apply steering force toward nearest defender.
///
/// Uses boids-style steering: applies a force toward the target instead of directly setting velocity.
/// Adds random movement when in melee range to simulate combat chaos.
pub fn update_attacker_targets(
    time: Res<Time>,
    mut attackers: Query<(&Transform, &mut Acceleration, &MovementSpeed, &Hitbox), With<Attacker>>,
    defenders: Query<(&Transform, &Hitbox), With<Defender>>,
) {
    const STEERING_FORCE: f32 = 500.0;
    const MELEE_RANDOM_FORCE: f32 = 150.0;

    for (att_transform, mut att_acceleration, _movement_speed, att_hitbox) in &mut attackers {
        if let Some((nearest_defender, def_hitbox)) = defenders.iter().min_by(|a, b| {
            let dist_a = att_transform.translation.distance(a.0.translation);
            let dist_b = att_transform.translation.distance(b.0.translation);
            dist_a.partial_cmp(&dist_b).unwrap()
        }) {
            let diff = nearest_defender.translation - att_transform.translation;
            let distance = (diff.x * diff.x + diff.z * diff.z).sqrt();

            // Check if in melee range (within attack range)
            let melee_range = (att_hitbox.radius + def_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;

            if distance < melee_range {
                // Add random movement in melee to simulate combat chaos
                // Use multiple frequency components for more natural randomness
                let seed = att_transform.translation.x * 0.1 + att_transform.translation.z * 0.13;
                let t = time.elapsed_secs();
                let random_angle = (t * 3.7 + seed).sin() * 2.0 + (t * 2.3 + seed * 1.7).cos();
                let random_x = random_angle.sin() * MELEE_RANDOM_FORCE * time.delta_secs();
                let random_z = random_angle.cos() * MELEE_RANDOM_FORCE * time.delta_secs();
                att_acceleration.add_force(Vec3::new(random_x, 0.0, random_z));
            }

            let steering = diff.normalize_or_zero() * STEERING_FORCE;
            att_acceleration.add_force(steering);
        }
    }
}

/// Handles combat between attackers and defenders.
///
/// Attackers attack the nearest defender within range (1.5x combined hitbox radius).
/// Attacks are time-based: each attacker attacks at a specific time offset in the global
/// attack cycle, naturally staggering attacks across all attackers.
pub fn combat(
    attack_cycle: Res<GlobalAttackCycle>,
    mut attackers: Query<(&Transform, &Hitbox, &mut AttackTiming), With<Attacker>>,
    mut defenders: Query<(Entity, &Transform, &Hitbox, &mut Health), With<Defender>>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - 0.016).max(0.0); // Approximate last frame time

    for (att_transform, att_hitbox, mut att_attack_timing) in &mut attackers {
        // Find the nearest defender within attack range
        if let Some((_, _, _, mut target_health)) = defenders
            .iter_mut()
            .filter(|(_, def_transform, def_hitbox, _)| {
                let distance = att_transform
                    .translation
                    .distance(def_transform.translation);
                let attack_range =
                    (att_hitbox.radius + def_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
                distance <= attack_range
            })
            .min_by(|(_, a_transform, _, _), (_, b_transform, _, _)| {
                let dist_a = att_transform.translation.distance(a_transform.translation);
                let dist_b = att_transform.translation.distance(b_transform.translation);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
        {
            // Attack if we're in the unit's attack window
            if att_attack_timing.can_attack(current_time, last_time) {
                target_health.take_damage(ATTACK_DAMAGE);
                att_attack_timing.record_attack(current_time);
            }
        }
    }
}
