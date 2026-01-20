use bevy::prelude::*;

use super::components::*;
use crate::game::units::components::{Health, Team};
use crate::game::units::infantry::components::Infantry;

/// Updates all projectile positions based on their direction and speed.
///
/// Projectiles move in a straight line until they hit a target or despawn.
pub fn move_projectiles(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &Projectile), With<Projectile>>,
) {
    for (mut transform, projectile) in &mut projectiles {
        transform.translation += projectile.direction * projectile.speed * time.delta_secs();
    }
}

/// Checks for projectile collisions with enemy units.
///
/// When a projectile hits an enemy, it deals damage and despawns.
pub fn check_projectile_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &Projectile), With<Projectile>>,
    mut enemies: Query<(&Transform, &mut Health, &Team), With<Infantry>>,
) {
    for (projectile_entity, proj_transform, projectile) in &projectiles {
        for (enemy_transform, mut health, team) in &mut enemies {
            // Only damage attackers (projectiles are from defenders/wizard)
            if *team != Team::Attackers {
                continue;
            }

            let distance = proj_transform
                .translation
                .distance(enemy_transform.translation);

            // Check if projectile hit the enemy
            if distance < projectile.radius {
                health.take_damage(projectile.damage);
                commands.entity(projectile_entity).despawn();
                break; // Projectile is destroyed, stop checking
            }
        }
    }
}

/// Updates spell effects and despawns them when their lifetime expires.
///
/// Spell effects have a lifetime timer that counts down each frame.
pub fn update_spell_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut SpellEffect)>,
) {
    for (entity, mut effect) in &mut effects {
        effect.lifetime -= time.delta_secs();

        if effect.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawns projectiles that travel too far from the battlefield.
///
/// This prevents projectiles from traveling infinitely and consuming resources.
pub fn despawn_distant_projectiles(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
) {
    const MAX_DISTANCE: f32 = 10000.0;

    for (entity, transform) in &projectiles {
        let distance_from_origin = transform.translation.length();

        if distance_from_origin > MAX_DISTANCE {
            commands.entity(entity).despawn();
        }
    }
}
