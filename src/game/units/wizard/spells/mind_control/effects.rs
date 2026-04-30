//! Mind-control side effects: aura, mass hysteria, amnesia, sleeper agent, mass control combat.

use bevy::prelude::*;

use super::components::*;
use super::constants;
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::components::{
    AttackTiming, Corpse, FlockingVelocity, Health, Hitbox, MindControlled, TargetingVelocity,
    Team, TemporaryHitPoints, apply_damage_to_unit,
};

/// Computes talent parameters from active talent selections.
pub(super) fn update_traitors_mark_aura(
    mut commands: Commands,
    aura_query: Query<
        &Transform,
        (
            With<MindControlled>,
            With<TraitorsMarkAura>,
            Without<Corpse>,
        ),
    >,
    mut enemies: Query<
        (Entity, &Transform, &Team, Has<Demoralized>),
        (Without<Corpse>, Without<MindControlled>),
    >,
) {
    for (entity, transform, team, has_demoralized) in &mut enemies {
        if !matches!(*team, Team::Attackers | Team::Undead) {
            continue;
        }

        // Check if within range of any aura
        let in_aura = aura_query.iter().any(|aura_transform| {
            crate::game::units::wizard::spells::utils::xz_distance(
                aura_transform.translation,
                transform.translation,
            ) <= constants::TRAITORS_MARK_RADIUS
        });

        if in_aura && !has_demoralized {
            commands.entity(entity).insert(Demoralized {
                damage_amplification: constants::TRAITORS_MARK_DAMAGE_AMP,
            });
        } else if !in_aura && has_demoralized {
            commands.entity(entity).remove::<Demoralized>();
        }
    }
}

/// Shared logic for confused combat: find nearest target in range and attack it.
/// Used by both Mass Hysteria and Amnesia effects.
fn confused_combat_attack(
    attacker_entity: Entity,
    attacker_pos: Vec3,
    attacker_hitbox: &Hitbox,
    current_time: f32,
    timing: &mut AttackTiming,
    potential_targets: &mut Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
) {
    let last_time = (current_time - crate::game::constants::APPROX_FRAME_TIME).max(0.0);
    if !timing.can_attack(current_time, last_time) {
        return;
    }

    let mut nearest: Option<(Entity, f32)> = None;

    for (target_entity, target_transform, target_hitbox, _, _) in potential_targets.iter() {
        if target_entity == attacker_entity {
            continue;
        }
        let dist = attacker_pos.distance(target_transform.translation);
        let attack_range =
            (attacker_hitbox.radius + target_hitbox.radius) * constants::CONFUSED_ATTACK_RANGE_MULT;
        if dist <= attack_range && nearest.as_ref().is_none_or(|n| dist < n.1) {
            nearest = Some((target_entity, dist));
        }
    }

    if let Some((target_entity, _)) = nearest
        && let Ok((_, _, _, mut health, mut temp_hp)) = potential_targets.get_mut(target_entity)
    {
        apply_damage_to_unit(
            &mut health,
            temp_hp.as_deref_mut(),
            constants::COMBAT_DAMAGE,
        );
        timing.last_attack_time = Some(current_time);
    }
}

/// Mass Hysteria: ticks down the effect timer and removes the component when expired.
/// Combat is handled by the shared combat system which checks `Has<MassHysteriaTarget>`
/// to allow team-agnostic attacks.
pub(super) fn tick_mass_hysteria(
    time: Res<Time>,
    mut commands: Commands,
    mut hysteria_query: Query<(Entity, &mut MassHysteriaTarget), Without<Corpse>>,
) {
    let delta = time.delta_secs();

    for (entity, mut hysteria) in &mut hysteria_query {
        hysteria.time_remaining -= delta;

        if hysteria.time_remaining <= 0.0 {
            commands.entity(entity).remove::<MassHysteriaTarget>();
        }
    }
}

/// Amnesia effect: confused units attack random nearby targets (friend or foe).
pub(super) fn tick_amnesia_effect(
    time: Res<Time>,
    attack_cycle: Res<crate::game::plugin::GlobalAttackCycle>,
    mut commands: Commands,
    mut amnesia_query: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut AmnesiaEffect,
            &mut AttackTiming,
        ),
        Without<Corpse>,
    >,
    mut potential_targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
) {
    let delta = time.delta_secs();
    let current_time = attack_cycle.current_time;

    for (entity, transform, hitbox, mut amnesia, mut timing) in &mut amnesia_query {
        amnesia.time_remaining -= delta;

        if amnesia.time_remaining <= 0.0 {
            commands.entity(entity).remove::<AmnesiaEffect>();
            continue;
        }

        confused_combat_attack(
            entity,
            transform.translation,
            hitbox,
            current_time,
            &mut timing,
            &mut potential_targets,
        );
    }
}

/// Sleeper Agent: ticks the delay timer and triggers betrayal attack.
pub(super) fn tick_sleeper_agent(
    time: Res<Time>,
    mut commands: Commands,
    mut agent_query: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut SleeperAgentActive),
        Without<Corpse>,
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
        (Without<Corpse>, Without<SleeperAgentActive>),
    >,
) {
    let delta = time.delta_secs();

    for (entity, transform, hitbox, team, mut agent) in &mut agent_query {
        agent.delay_remaining -= delta;

        if agent.delay_remaining <= 0.0 {
            // Betrayal! Attack nearest same-team unit with bonus damage
            let pos = transform.translation;
            let mut nearest: Option<(Entity, f32)> = None;

            for (target_entity, target_transform, target_hitbox, target_team, _, _) in
                &potential_targets
            {
                if target_entity == entity || target_team != team {
                    continue;
                }
                let dist = pos.distance(target_transform.translation);
                let attack_range =
                    (hitbox.radius + target_hitbox.radius) * constants::SLEEPER_AGENT_RANGE_MULT;
                if dist <= attack_range && nearest.as_ref().is_none_or(|n| dist < n.1) {
                    nearest = Some((target_entity, dist));
                }
            }

            if let Some((target_entity, _)) = nearest
                && let Ok((_, _, _, _, mut health, mut temp_hp)) =
                    potential_targets.get_mut(target_entity)
            {
                let damage = constants::COMBAT_DAMAGE * agent.damage_multiplier;
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
            }

            commands.entity(entity).remove::<SleeperAgentActive>();
        }
    }
}

/// Strips all mind-control talent components from an entity.
/// Mass Hysteria targeting: affected units move toward the nearest unit regardless of team.
pub(super) fn update_mass_hysteria_targeting(
    mut hysteria_units: Query<
        (
            Entity,
            &Transform,
            &mut TargetingVelocity,
            &mut FlowFieldVelocity,
            &mut FlockingVelocity,
        ),
        (With<MassHysteriaTarget>, Without<Corpse>),
    >,
    all_units: Query<(Entity, &Transform), Without<Corpse>>,
) {
    for (entity, transform, mut targeting, mut flow_field, mut flocking) in &mut hysteria_units {
        let pos = transform.translation;
        let mut nearest: Option<(f32, Vec3)> = None;

        for (other_entity, other_transform) in &all_units {
            if other_entity == entity {
                continue;
            }
            let dx = other_transform.translation.x - pos.x;
            let dz = other_transform.translation.z - pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if nearest.as_ref().is_none_or(|(best, _)| dist < *best) {
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

        // Zero out flow field and flocking so targeting fully controls movement
        // Guard to avoid triggering Bevy change detection unnecessarily
        if flow_field.velocity != Vec3::ZERO {
            flow_field.velocity = Vec3::ZERO;
        }
        if flocking.velocity != Vec3::ZERO {
            flocking.velocity = Vec3::ZERO;
        }
    }
}

pub(crate) fn strip_mind_control_talent_components(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<TraitorsMarkAura>()
        .remove::<AmnesiaOnExpiry>()
        .remove::<AmnesiaEffect>()
        .remove::<DominatedUnit>()
        .remove::<SleeperAgentPending>()
        .remove::<SleeperAgentActive>();
}
