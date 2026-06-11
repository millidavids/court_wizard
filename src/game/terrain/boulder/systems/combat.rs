use bevy::prelude::*;

use super::super::components::Boulder;
use super::super::constants::*;
use crate::game::attack_cycle::GlobalAttackCycle;
use crate::game::components::ObstacleHealth;
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::components::{AttackTiming, Corpse, Health, Hitbox, TemporaryHitPoints};
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorExplosion;
use crate::game::units::wizard::spells::squall::components::IceExplosion;

/// Units with no valid path attack nearby boulders (same pattern as wall of stone).
pub fn units_attack_blocking_rocks(
    attack_cycle: Res<GlobalAttackCycle>,
    mut blocked_units: Query<
        (
            &Transform,
            &Hitbox,
            &FlowFieldVelocity,
            &mut AttackTiming,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (Without<Corpse>, Without<Boulder>),
    >,
    mut rocks: Query<(Entity, &Boulder, &mut ObstacleHealth)>,
) {
    let current_time = attack_cycle.current_time;
    // `previous_time` is the cycle position before the most recent `tick()` —
    // see `GlobalAttackCycle` doc for why this is preferred over the
    // `current_time - APPROX_FRAME_TIME` approximation.
    let last_time = attack_cycle.previous_time;

    for (transform, hitbox, flow_vel, mut attack_timing, _health, _temp_hp) in &mut blocked_units {
        // Only target boulders if this unit has no valid path
        if !flow_vel.pathfinding_distance.is_infinite() {
            continue;
        }

        let unit_pos = transform.translation;

        // Find nearest boulder by distance to surface
        let mut nearest_rock_entity = None;
        let mut nearest_distance = f32::MAX;

        for (entity, rock, _) in rocks.iter() {
            if rock.sinking {
                continue;
            }
            let dist = rock.distance_to_surface(unit_pos);
            if dist < nearest_distance {
                nearest_distance = dist;
                nearest_rock_entity = Some(entity);
            }
        }

        // Deal damage if close enough to a boulder
        let attack_range = hitbox.radius + ROCK_ATTACK_RANGE;
        if let Some(rock_entity) = nearest_rock_entity
            && nearest_distance <= attack_range
            && attack_timing.can_attack(current_time, last_time)
            && let Ok((_, _, mut rock_health)) = rocks.get_mut(rock_entity)
        {
            rock_health.take_damage(ROCK_DAMAGE_PER_HIT);
            attack_timing.record_attack(current_time);
        }
    }
}

/// Applies damage to boulders from spell AoE explosions (fireball, meteor, squall).
pub fn apply_spell_damage_to_rocks(
    fireball_explosions: Query<&FireballExplosion>,
    meteor_explosions: Query<&MeteorExplosion>,
    ice_explosions: Query<&IceExplosion>,
    mut rocks: Query<(&Boulder, &mut ObstacleHealth)>,
) {
    let xz_distance = |a: Vec3, b: Vec3| -> f32 {
        let dx = a.x - b.x;
        let dz = a.z - b.z;
        (dx * dx + dz * dz).sqrt()
    };

    for (rock, mut health) in &mut rocks {
        if rock.sinking || health.is_dead() {
            continue;
        }

        for explosion in &fireball_explosions {
            if explosion.damage_per_tick > 0.0
                && xz_distance(explosion.origin, rock.center)
                    <= explosion.current_radius() + rock.radius
            {
                health.take_damage(explosion.damage_per_tick);
            }
        }

        for explosion in &meteor_explosions {
            if !explosion.damage_applied
                && xz_distance(explosion.origin, rock.center) <= explosion.max_radius + rock.radius
            {
                health.take_damage(explosion.damage);
            }
        }

        for explosion in &ice_explosions {
            if !explosion.damage_applied
                && xz_distance(explosion.origin, rock.center) <= explosion.max_radius + rock.radius
            {
                health.take_damage(explosion.damage);
            }
        }
    }
}
