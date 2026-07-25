use crate::game::multiplayer::components::GhostEntity;
use crate::game::units::components::{Corpse, HasteModifier, SlowMovementModifier, Team};
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::haste::components::{
    ChainHasteSource, FleetFeet, HasteSlowZone, MomentumBuff, MomentumPending,
};
use crate::game::units::wizard::spells::haste::constants;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

use super::casting::compute_talent_params;

/// Handles HasteModifier expiry effects: Momentum buff and Chain Haste.
/// Detects units whose HasteModifier just expired by checking for marker components
/// (MomentumPending, ChainHasteSource) that outlive the HasteModifier.
#[allow(clippy::type_complexity)]
pub fn handle_haste_expiry(
    mut commands: Commands,
    // Units with momentum pending but no haste (haste just expired)
    momentum_expired: Query<
        Entity,
        (
            With<MomentumPending>,
            Without<HasteModifier>,
            Without<Corpse>,
            Without<GhostEntity>,
        ),
    >,
    // Units with chain haste source but no haste (haste just expired)
    chain_sources: Query<
        (Entity, &Transform, &ChainHasteSource, &Team),
        (Without<HasteModifier>, Without<GhostEntity>),
    >,
    // Potential targets for chain haste (alive units without haste)
    potential_targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<HasteModifier>,
            Without<ChainHasteSource>,
            Without<Corpse>,
            Without<Wizard>,
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    active_talents: Option<Res<ActiveTalents>>,
) {
    let talent_params = compute_talent_params(active_talents.as_deref());

    // Apply Momentum buff to units whose haste just expired (non-chain case)
    for entity in &momentum_expired {
        commands.entity(entity).insert(MomentumBuff::new(
            constants::MOMENTUM_DAMAGE_MULT,
            constants::MOMENTUM_DURATION,
        ));
        commands.entity(entity).remove::<MomentumPending>();
    }

    // Handle chain haste expiry
    for (source_entity, source_transform, chain_source, source_team) in &chain_sources {
        // Apply Momentum buff if talent is also active
        if talent_params.momentum {
            commands.entity(source_entity).insert(MomentumBuff::new(
                constants::MOMENTUM_DAMAGE_MULT,
                constants::MOMENTUM_DURATION,
            ));
            commands.entity(source_entity).remove::<MomentumPending>();
        }

        // Chain Haste: jump to nearest un-hasted ally
        if chain_source.hops_remaining > 0 {
            let new_effectiveness = chain_source.effectiveness * constants::CHAIN_HASTE_FALLOFF;
            let new_modifier = constants::HASTE_MODIFIER * new_effectiveness;
            let new_duration = constants::HASTE_DURATION * new_effectiveness;
            let new_attack_speed = chain_source.attack_speed * constants::CHAIN_HASTE_FALLOFF;

            // Find nearest ally without haste
            if let Some((target_entity, _, _)) = potential_targets
                .iter()
                .filter(|(_, _, team)| **team == *source_team)
                .map(|(entity, transform, team)| {
                    let dist = transform.translation.distance(source_transform.translation);
                    (entity, dist, team)
                })
                .filter(|(_, dist, _)| *dist <= constants::CHAIN_HASTE_RADIUS)
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                let mut target_cmds = commands.entity(target_entity);
                target_cmds.insert((
                    HasteModifier::with_attack_speed(new_modifier, new_duration, new_attack_speed),
                    ChainHasteSource {
                        hops_remaining: chain_source.hops_remaining - 1,
                        effectiveness: new_effectiveness,
                        attack_speed: new_attack_speed,
                    },
                ));

                // Carry over talent effects to chained target
                if talent_params.momentum {
                    target_cmds.insert(MomentumPending);
                }
                if talent_params.fleet_feet {
                    target_cmds.insert(FleetFeet::new(1));
                }
            }
        }

        // Remove the chain source from the expired unit
        commands.entity(source_entity).remove::<ChainHasteSource>();
    }
}

/// Ticks HasteSlowZone timer and applies slow to non-hasted units within range.
#[allow(clippy::type_complexity)]
pub fn tick_haste_slow_zone(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<
        (Entity, &mut HasteSlowZone),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut units: Query<
        (Entity, &Transform, Option<&mut SlowMovementModifier>),
        (
            Without<Corpse>,
            Without<HasteModifier>,
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();
    for (zone_entity, mut zone) in zones.iter_mut() {
        zone.time_remaining -= delta;
        if zone.time_remaining <= 0.0 {
            commands.entity(zone_entity).try_despawn();
            continue;
        }

        // Apply slow to all non-hasted units within the zone
        for (unit_entity, transform, existing_slow) in units.iter_mut() {
            let distance = crate::game::units::wizard::spells::utils::xz_distance(
                transform.translation,
                zone.position,
            );
            if distance <= zone.radius {
                if let Some(mut slow) = existing_slow {
                    slow.apply(zone.slow_amount, 0.5);
                } else {
                    commands
                        .entity(unit_entity)
                        .insert(SlowMovementModifier::new(zone.slow_amount, 0.5));
                }
            }
        }
    }
}
