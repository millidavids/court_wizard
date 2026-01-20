use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::{Acceleration, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::units::attacker::components::Attacker;
use crate::game::units::components::{AttackTiming, Health, Hitbox, MovementSpeed, Team};

/// Spawns initial defenders when entering the game.
///
/// Spawns defenders clustered together, letting collision resolution push them apart.
pub fn spawn_initial_defenders(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for i in 0..INITIAL_DEFENDER_COUNT {
        // Define defender hitbox (cylinder) - this determines sprite size
        let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);

        // Spawn defender as a circle billboard sized to match the hitbox
        let circle = Circle::new(hitbox.radius);

        // Distribute spawns in a circular pattern
        let offset = i as f32 * SPAWN_OFFSET_MULTIPLIER;
        let spawn_x = DEFENDER_SPAWN_X_MIN
            + (offset.sin() * SPAWN_DISTRIBUTION_RADIUS + SPAWN_DISTRIBUTION_RADIUS);
        let spawn_z = DEFENDER_SPAWN_Z_MIN
            + (offset.cos() * SPAWN_DISTRIBUTION_RADIUS + SPAWN_DISTRIBUTION_RADIUS);

        commands.spawn((
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: DEFENDER_COLOR,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(spawn_x, UNIT_Y_POSITION, spawn_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(UNIT_HEALTH),
            MovementSpeed::new(UNIT_MOVEMENT_SPEED),
            AttackTiming::new(),
            Team::Player,
            Defender,
            OnGameplayScreen,
        ));
    }
}

/// Updates defender targeting to apply steering force toward nearest attacker.
///
/// Uses boids-style steering: applies a force toward the target instead of directly setting velocity.
/// All defenders share activation - once ANY attacker is within range of ANY defender,
/// all defenders activate and start moving.
/// Adds random movement when in melee range to simulate combat chaos.
pub fn update_defender_targets(
    time: Res<Time>,
    mut defenders: Query<(&Transform, &mut Acceleration, &MovementSpeed, &Hitbox), With<Defender>>,
    attackers: Query<(&Transform, &Hitbox), With<Attacker>>,
    mut defenders_activated: ResMut<DefendersActivated>,
) {
    // Targeting parameters are defined in constants.rs

    // Check if any attacker is within activation distance of any defender
    if !defenders_activated.active {
        for def_transform in defenders.iter().map(|(t, _, _, _)| t) {
            for (attacker_transform, _) in attackers.iter() {
                let distance = def_transform
                    .translation
                    .distance(attacker_transform.translation);
                if distance < DEFENDER_ACTIVATION_DISTANCE {
                    defenders_activated.active = true;
                    break;
                }
            }
            if defenders_activated.active {
                break;
            }
        }
    }

    // If defenders are activated, apply steering force toward nearest attacker
    if defenders_activated.active {
        for (def_transform, mut def_acceleration, _movement_speed, def_hitbox) in &mut defenders {
            if let Some((nearest_attacker, att_hitbox)) = attackers.iter().min_by(|a, b| {
                let dist_a = def_transform.translation.distance(a.0.translation);
                let dist_b = def_transform.translation.distance(b.0.translation);
                dist_a.partial_cmp(&dist_b).unwrap()
            }) {
                let diff = nearest_attacker.translation - def_transform.translation;
                let distance = (diff.x * diff.x + diff.z * diff.z).sqrt();

                // Check if in melee range (within attack range)
                let melee_range = (def_hitbox.radius + att_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;

                if distance < melee_range {
                    // Add random movement in melee to simulate combat chaos
                    // Use multiple frequency components for more natural randomness
                    let seed =
                        def_transform.translation.x * 0.1 + def_transform.translation.z * 0.13;
                    let t = time.elapsed_secs();
                    let random_angle = (t * 3.7 + seed).sin() * 2.0 + (t * 2.3 + seed * 1.7).cos();
                    let random_x = random_angle.sin() * MELEE_RANDOM_FORCE * time.delta_secs();
                    let random_z = random_angle.cos() * MELEE_RANDOM_FORCE * time.delta_secs();
                    def_acceleration.add_force(Vec3::new(random_x, 0.0, random_z));
                }

                let steering = diff.normalize_or_zero() * STEERING_FORCE;
                def_acceleration.add_force(steering);
            }
        }
    }
}
