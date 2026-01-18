use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::{OnGameplayScreen, Velocity};
use crate::game::wizard::components::Wizard;

/// Spawns attacker units periodically.
///
/// Uses a local timer to spawn attackers every SPAWN_INTERVAL seconds.
/// Attackers spawn far away (at the horizon) and move toward the castle.
pub fn spawn_attackers(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    *timer += time.delta_secs();

    if *timer >= SPAWN_INTERVAL {
        *timer = 0.0;

        // Spawn attacker as a circle mesh far away
        let circle = Circle::new(UNIT_RADIUS);

        commands.spawn((
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: ATTACKER_COLOR,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(1200.0, 50.0, -1200.0), // Top-right corner
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Attacker,
            OnGameplayScreen,
        ));
    }
}

/// Updates attacker velocities to target the wizard.
///
/// Attackers move in 3D space toward the wizard's position.
pub fn update_attacker_targets(
    mut attackers: Query<(&Transform, &mut Velocity), With<Attacker>>,
    wizard_query: Query<&Transform, With<Wizard>>,
) {
    let Some(wizard_transform) = wizard_query.iter().next() else {
        return; // No wizard, no target
    };

    for (att_transform, mut att_velocity) in &mut attackers {
        let direction =
            (wizard_transform.translation - att_transform.translation).normalize_or_zero();
        att_velocity.x = direction.x * UNIT_SPEED;
        att_velocity.y = direction.y * UNIT_SPEED;
        att_velocity.z = direction.z * UNIT_SPEED;
    }
}
