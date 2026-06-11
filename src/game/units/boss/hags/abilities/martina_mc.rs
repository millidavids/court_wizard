use bevy::prelude::*;

use super::super::constants::*;
use crate::game::components::Velocity;
use crate::game::constants::*;
use crate::game::pathfinding::FlowFieldInfluence;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, Corpse, FrozenSolidModifier, Health, Hitbox, MindControlled,
    MovementSpeed, Petrified, RetaliationTarget, RootedModifier, Stunned, TargetingVelocity, Team,
    TemporaryHitPoints, apply_damage_to_unit,
};

/// Filter for `mind_controlled_pursue_allies`: controlled units that aren't ghosts
/// and aren't crowd-controlled (rooted/stunned/frozen/banished/petrified).
type MindControlPursuitFilter = (
    With<MindControlled>,
    Without<crate::game::multiplayer::components::GhostEntity>,
    Without<RootedModifier>,
    Without<Stunned>,
    Without<FrozenSolidModifier>,
    Without<BanishedModifier>,
    Without<Petrified>,
);

/// Updates mind-controlled units — they target non-MC same-team allies.
pub fn update_mind_controlled_targeting(
    mut controlled: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            &MindControlled,
        ),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<MindControlled>,
            Without<BanishedModifier>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
) {
    for (entity, transform, team, mut targeting, _mc) in &mut controlled {
        // Find nearest ALLY to attack (reversed targeting)
        let pos = transform.translation;
        let mut nearest: Option<(f32, Vec3)> = None;

        for (other_entity, other_transform, other_team) in &all_units {
            if other_entity == entity {
                continue;
            }
            // Target same team (allies become enemies)
            if other_team != team {
                continue;
            }

            let dx = other_transform.translation.x - pos.x;
            let dz = other_transform.translation.z - pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if let Some((best_dist, _)) = nearest {
                if dist < best_dist {
                    nearest = Some((dist, other_transform.translation));
                }
            } else {
                nearest = Some((dist, other_transform.translation));
            }
        }

        if let Some((_, target_pos)) = nearest {
            let dir =
                Vec3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z).normalize_or_zero();
            targeting.velocity = dir;
        } else {
            targeting.velocity = Vec3::ZERO;
        }
    }
}

/// Mind-controlled units charge their nearest former ally so they close to melee
/// range (where `mind_controlled_combat` lands hits) instead of drifting with the
/// herd toward the enemy base. Runs AFTER `MovementCalculationSet` to override the
/// blended steering — the weighted blend keys off flow-field distance, so without
/// this override an MC'd attacker just keeps marching at the castle and never
/// engages its former allies. Mirrors the Pig Form / Dire Sheep velocity overrides.
///
/// Reuses the direction `update_mind_controlled_targeting` already computed into
/// `TargetingVelocity` (a unit-length XZ vector toward the nearest same-team ally,
/// or zero if none) rather than re-scanning every unit — so the two systems can't
/// pick different targets and oscillate. Crowd-controlled units are excluded so a
/// rooted / stunned / frozen / banished / petrified MC unit doesn't slide.
pub fn mind_controlled_pursue_allies(
    controlled: Query<(Entity, &TargetingVelocity, &MovementSpeed), MindControlPursuitFilter>,
    mut velocity_query: Query<&mut Velocity>,
) {
    for (entity, targeting, speed) in &controlled {
        if let Ok(mut velocity) = velocity_query.get_mut(entity) {
            // `targeting.velocity` is unit-length (or zero when no ally remains,
            // which correctly halts the unit).
            velocity.x = targeting.velocity.x * speed.0;
            velocity.z = targeting.velocity.z * speed.0;
        }
    }
}

/// Updates mind control wear-off timer — removes when duration expires.
/// Also cleans up RetaliationTarget components that point at freed entities.
/// Handles talent on-expiry effects: Amnesia (confused state) and Sleeper Agent (delayed betrayal).
pub fn update_mind_control_wear_off(
    time: Res<Time>,
    mut commands: Commands,
    mut controlled: Query<
        (
            Entity,
            &mut MindControlled,
            Has<crate::game::units::wizard::spells::mind_control::components::AmnesiaOnExpiry>,
            Has<crate::game::units::wizard::spells::mind_control::components::SleeperAgentPending>,
        ),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
    retaliators: Query<(Entity, &RetaliationTarget)>,
) {
    use crate::game::units::wizard::spells::mind_control::components::{
        AmnesiaEffect, SleeperAgentActive,
    };
    use crate::game::units::wizard::spells::mind_control::constants;

    let delta = time.delta_secs();

    for (entity, mut mc, has_amnesia, has_sleeper) in &mut controlled {
        mc.time_elapsed += delta;

        if mc.time_elapsed >= mc.wear_off_duration {
            // Restore original flow field influence before removing mind control
            if let Some(spawn_pos) = mc.original_spawn_pos {
                commands
                    .entity(entity)
                    .insert(FlowFieldInfluence::Defender { spawn_pos });
            }

            commands.entity(entity).remove::<MindControlled>();

            // Clean up talent marker components
            crate::game::units::wizard::spells::mind_control::systems::strip_mind_control_talent_components(
                &mut commands, entity,
            );

            // Amnesia: apply confused state on expiry
            if has_amnesia {
                commands.entity(entity).insert(AmnesiaEffect {
                    time_remaining: constants::AMNESIA_DURATION,
                });
            }

            // Sleeper Agent: start delayed betrayal timer
            if has_sleeper {
                commands.entity(entity).insert(SleeperAgentActive {
                    delay_remaining: constants::SLEEPER_AGENT_DELAY,
                    damage_multiplier: constants::SLEEPER_AGENT_DAMAGE_MULT,
                });
            }

            // Remove RetaliationTarget from any units retaliating against this entity
            for (retaliator_entity, retaliation) in &retaliators {
                if retaliation.0 == entity {
                    commands
                        .entity(retaliator_entity)
                        .remove::<RetaliationTarget>();
                }
            }
        }
    }
}

/// Mind-controlled units attack their own team (gated by global attack cycle).
/// They skip other mind-controlled units and only attack non-MC allies.
#[allow(clippy::type_complexity)]
pub fn mind_controlled_combat(
    attack_cycle: Res<crate::game::attack_cycle::GlobalAttackCycle>,
    mut commands: Commands,
    mut controlled: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut AttackTiming,
            &MindControlled,
        ),
        (
            Without<Corpse>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
    mut potential_targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<Corpse>,
            Without<MindControlled>,
            Without<BanishedModifier>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
) {
    let current_time = attack_cycle.current_time;
    // `previous_time` is the cycle position before the most recent `tick()` —
    // see `GlobalAttackCycle` doc for why this is preferred over the
    // `current_time - APPROX_FRAME_TIME` approximation.
    let last_time = attack_cycle.previous_time;

    for (mc_entity, mc_transform, mc_hitbox, mc_team, mut timing, mc) in &mut controlled {
        if !timing.can_attack(current_time, last_time) {
            continue;
        }

        let mc_pos = mc_transform.translation;

        // Find nearest same-team unit to attack (mind controlled = attacks allies)
        for (entity, target_transform, target_hitbox, target_team, mut health, mut temp_hp) in
            &mut potential_targets
        {
            if entity == mc_entity {
                continue;
            }
            // Attack same team (reversed)
            if target_team != mc_team {
                continue;
            }

            let dx = target_transform.translation.x - mc_pos.x;
            let dz = target_transform.translation.z - mc_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            let attack_range = (mc_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;

            if dist <= attack_range {
                let damage = MIND_CONTROL_COMBAT_DAMAGE * mc.damage_multiplier;
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
                timing.last_attack_time = Some(current_time);

                // Victim retaliates — consider the MC attacker a valid target
                commands.entity(entity).insert(RetaliationTarget(mc_entity));

                break; // One attack per cycle
            }
        }
    }
}

/// Cleans up RetaliationTarget when the target entity is dead or no longer mind-controlled.
pub fn cleanup_retaliation_targets(
    mut commands: Commands,
    retaliators: Query<
        (Entity, &RetaliationTarget),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
    mc_units: Query<Entity, With<MindControlled>>,
) {
    for (entity, retaliation) in &retaliators {
        // Remove if the retaliation target is no longer mind-controlled (or despawned)
        if mc_units.get(retaliation.0).is_err() {
            commands.entity(entity).remove::<RetaliationTarget>();
        }
    }
}
