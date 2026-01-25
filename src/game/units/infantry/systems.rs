use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::{Acceleration, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::units::components::{AttackTiming, Health, Hitbox, MovementSpeed, Team};
use crate::game::units::constants as unit_constants;

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
            Team::Defenders,
            Infantry,
            OnGameplayScreen,
        ));
    }
}

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
        let hitbox = Hitbox::new(UNIT_RADIUS, ATTACKER_HITBOX_HEIGHT);

        // Spawn attacker as a circle billboard sized to match the hitbox
        let circle = Circle::new(hitbox.radius);

        // Distribute spawns in a circular pattern
        let offset = i as f32 * SPAWN_OFFSET_MULTIPLIER;
        let spawn_x = ATTACKER_SPAWN_X_MIN
            + (offset.sin() * SPAWN_DISTRIBUTION_RADIUS + SPAWN_DISTRIBUTION_RADIUS);
        let spawn_z = ATTACKER_SPAWN_Z_MIN
            + (offset.cos() * SPAWN_DISTRIBUTION_RADIUS + SPAWN_DISTRIBUTION_RADIUS);

        commands.spawn((
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: ATTACKER_COLOR,
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
            Team::Attackers,
            Infantry,
            OnGameplayScreen,
        ));
    }
}

/// Updates infantry targeting to apply steering force toward nearest enemy.
///
/// Uses boids-style steering: applies a force toward the target instead of directly setting
/// velocity. Defenders share activation - once any attacker is within range of any defender,
/// all defenders activate and start moving.
/// Adds random movement when in melee range to simulate combat chaos.
pub fn update_infantry_targets(
    time: Res<Time>,
    mut units: Query<(&Transform, &mut Acceleration, &Hitbox, &Team), With<Infantry>>,
    mut defenders_activated: ResMut<DefendersActivated>,
) {
    // First pass: collect position/hitbox data for both teams
    let attackers: Vec<(Vec3, f32)> = units
        .iter()
        .filter(|(_, _, _, team)| **team == Team::Attackers)
        .map(|(transform, _, hitbox, _)| (transform.translation, hitbox.radius))
        .collect();

    let defenders: Vec<(Vec3, f32)> = units
        .iter()
        .filter(|(_, _, _, team)| **team == Team::Defenders)
        .map(|(transform, _, hitbox, _)| (transform.translation, hitbox.radius))
        .collect();

    // Check defender activation
    if !defenders_activated.active {
        'activation: for &(def_pos, _) in &defenders {
            for &(att_pos, _) in &attackers {
                if def_pos.distance(att_pos) < DEFENDER_ACTIVATION_DISTANCE {
                    defenders_activated.active = true;
                    break 'activation;
                }
            }
        }
    }

    // Second pass: apply steering and melee forces
    for (transform, mut acceleration, hitbox, team) in &mut units {
        let enemies = match *team {
            Team::Defenders => {
                if !defenders_activated.active {
                    continue;
                }
                &attackers
            }
            Team::Attackers => &defenders,
        };

        // Find nearest enemy
        let nearest = enemies.iter().min_by(|a, b| {
            let dist_a = transform.translation.distance(a.0);
            let dist_b = transform.translation.distance(b.0);
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        if let Some(&(enemy_pos, enemy_radius)) = nearest {
            let diff = enemy_pos - transform.translation;
            let distance = (diff.x * diff.x + diff.z * diff.z).sqrt();

            // Check if in melee range (within attack range)
            let melee_range = (hitbox.radius + enemy_radius) * ATTACK_RANGE_MULTIPLIER;

            if distance < melee_range {
                // Add random movement in melee to simulate combat chaos
                let seed = transform.translation.x * unit_constants::MELEE_RANDOM_SEED_X_MULTIPLIER
                    + transform.translation.z * unit_constants::MELEE_RANDOM_SEED_Z_MULTIPLIER;
                let t = time.elapsed_secs();
                let random_angle = (t * unit_constants::MELEE_RANDOM_FREQ_PRIMARY + seed).sin()
                    * unit_constants::MELEE_RANDOM_AMPLITUDE_PRIMARY
                    + (t * unit_constants::MELEE_RANDOM_FREQ_SECONDARY
                        + seed * unit_constants::MELEE_RANDOM_SEED_FREQ_MULTIPLIER)
                        .cos();
                let random_x = random_angle.sin() * MELEE_RANDOM_FORCE * time.delta_secs();
                let random_z = random_angle.cos() * MELEE_RANDOM_FORCE * time.delta_secs();
                acceleration.add_force(Vec3::new(random_x, 0.0, random_z));
            }

            let steering = diff.normalize_or_zero() * STEERING_FORCE;
            acceleration.add_force(steering);
        }
    }
}
