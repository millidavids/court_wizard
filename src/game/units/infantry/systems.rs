use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::units::components::{
    AttackTiming, Health, Hitbox, MovementSpeed, Team, Teleportable,
};

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

        // Position unit so bottom edge is 1 unit above battlefield (Y=0)
        let spawn_y = hitbox.height / 2.0 + 1.0;

        commands.spawn((
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: DEFENDER_COLOR,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(spawn_x, spawn_y, spawn_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(UNIT_HEALTH),
            MovementSpeed::new(UNIT_MOVEMENT_SPEED),
            AttackTiming::new(),
            Team::Defenders,
            Infantry,
            Teleportable,
            Billboard,
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

        // Position unit so bottom edge is 1 unit above battlefield (Y=0)
        let spawn_y = hitbox.height / 2.0 + 1.0;

        commands.spawn((
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: ATTACKER_COLOR,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(spawn_x, spawn_y, spawn_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(UNIT_HEALTH),
            MovementSpeed::new(UNIT_MOVEMENT_SPEED),
            AttackTiming::new(),
            Team::Attackers,
            Infantry,
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ));
    }
}
