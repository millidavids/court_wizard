use super::super::components::{LivingStoneTracker, WallHealth, WallOfStone, WallTalents};
use super::super::constants::*;
use crate::game::attack_cycle::GlobalAttackCycle;
use crate::game::pathfinding::{FlowFieldVelocity, StagingAttacker};
use crate::game::units::components::{
    AttackTiming, Corpse, Hitbox, TargetingVelocity, TemporaryHitPoints,
};
use bevy::prelude::*;

/// Units with no valid path (pathfinding_distance == INFINITY) move toward the
/// king and attack any wall they end up pressed against. This prevents players
/// from exploiting wall placement to permanently trap units — blocked attackers
/// naturally converge on the walls surrounding the king rather than scattering
/// to the nearest wall on the map.
#[allow(clippy::type_complexity)]
pub fn units_attack_blocking_walls(
    attack_cycle: Res<GlobalAttackCycle>,
    mut blocked_units: Query<
        (
            &Transform,
            &Hitbox,
            &FlowFieldVelocity,
            &mut TargetingVelocity,
            &mut AttackTiming,
            &mut crate::game::units::components::Health,
            Option<&mut TemporaryHitPoints>,
            Has<StagingAttacker>,
        ),
        (Without<Corpse>, Without<WallOfStone>),
    >,
    king_query: Query<&Transform, With<crate::game::units::king::components::King>>,
    mut walls: Query<(
        Entity,
        &WallOfStone,
        &mut WallHealth,
        Option<&WallTalents>,
        Option<&mut LivingStoneTracker>,
    )>,
) {
    let current_time = attack_cycle.current_time;
    // `previous_time` is the cycle position before the most recent `tick()` —
    // see `GlobalAttackCycle` doc for why this is preferred over the
    // `current_time - APPROX_FRAME_TIME` approximation.
    let last_time = attack_cycle.previous_time;

    let king_pos = king_query.iter().next().map(|t| t.translation);

    for (
        transform,
        hitbox,
        flow_vel,
        mut targeting_vel,
        mut attack_timing,
        mut health,
        temp_hp,
        is_staging,
    ) in &mut blocked_units
    {
        // Only target walls if this unit has no valid path
        if !flow_vel.pathfinding_distance.is_infinite() {
            continue;
        }

        let unit_pos = transform.translation;

        // Move toward the king — wall collision will stop the unit at the
        // blocking wall, causing units to pile up where they need to attack.
        if let Some(king) = king_pos {
            let diff = Vec3::new(king.x - unit_pos.x, 0.0, king.z - unit_pos.z);
            targeting_vel.velocity = diff.normalize_or_zero();
        }

        // Find nearest wall by distance to surface for melee damage
        let mut nearest_wall_entity = None;
        let mut nearest_distance = f32::MAX;

        for (entity, wall, _, _, _) in walls.iter() {
            let dist = wall.distance_to_surface(unit_pos);
            if dist < nearest_distance {
                nearest_distance = dist;
                nearest_wall_entity = Some(entity);
            }
        }

        // Deal damage if close enough to a wall
        let attack_range = hitbox.radius + WALL_ATTACK_RANGE;
        if let Some(wall_entity) = nearest_wall_entity
            && nearest_distance <= attack_range
            && attack_timing.can_attack(current_time, last_time)
            && let Ok((_, _, mut wall_health, wall_talents, living_stone_tracker)) =
                walls.get_mut(wall_entity)
        {
            wall_health.take_damage(WALL_DAMAGE_PER_HIT);
            attack_timing.record_attack(current_time);

            // Reset Living Stone regen timer on damage
            if let Some(mut tracker) = living_stone_tracker {
                tracker.time_since_last_damage = 0.0;
            }

            // Jagged Stone: reflect damage back to attacker. Skipped for staging
            // units — reflect is spell damage, and staging units are spell-immune,
            // even though they can still deal (and the wall still takes) melee damage.
            if let Some(talents) = wall_talents
                && talents.0.jagged_stone
                && !is_staging
            {
                crate::game::units::components::apply_damage_to_unit(
                    &mut health,
                    temp_hp.map(|t| t.into_inner()),
                    JAGGED_STONE_REFLECT_DAMAGE,
                );
            }
        }
    }
}
