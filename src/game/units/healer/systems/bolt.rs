use bevy::prelude::*;

use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{BanishedModifier, Corpse, Health, Hitbox, Team};
use crate::game::units::healer::components::HealBolt;
use crate::game::units::healer::constants::HEAL_BOLT_RADIUS;

/// Moves heal bolts toward their targets (homing). Despawns if target is gone or lifetime expires.
pub fn move_heal_bolts(
    mut commands: Commands,
    time: Res<Time>,
    mut bolts: Query<(Entity, &mut Transform, &mut HealBolt)>,
    targets: Query<&Transform, Without<HealBolt>>,
) {
    let delta = time.delta_secs();
    for (entity, mut bolt_transform, mut bolt) in &mut bolts {
        bolt.lifetime -= delta;
        if bolt.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Recalculate direction toward target each frame (homing)
        if let Ok(target_transform) = targets.get(bolt.target) {
            let diff = target_transform.translation - bolt_transform.translation;
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();
            bolt_transform.translation += direction * bolt.speed * delta;
        } else {
            // Target gone — despawn bolt
            commands.entity(entity).try_despawn();
        }
    }
}

/// Checks if heal bolts have reached their targets and applies healing.
/// Staging attackers (not yet activated at their rally point) cannot be healed.
pub fn check_heal_bolt_arrivals(
    mut commands: Commands,
    bolts: Query<(Entity, &Transform, &HealBolt)>,
    mut targets: Query<
        (&Transform, &Hitbox, &Team, &mut Health),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    for (bolt_entity, bolt_transform, bolt) in &bolts {
        let bolt_pos = bolt_transform.translation;

        if let Ok((target_transform, hitbox, team, mut health)) = targets.get_mut(bolt.target) {
            // Verify same team
            if *team != bolt.source_team {
                commands.entity(bolt_entity).try_despawn();
                continue;
            }

            // Check if bolt has arrived
            let distance = bolt_pos.distance(target_transform.translation);
            if distance < hitbox.radius + HEAL_BOLT_RADIUS {
                // Heal to full
                health.current = health.max;
                commands.entity(bolt_entity).try_despawn();
            }
        }
    }
}
