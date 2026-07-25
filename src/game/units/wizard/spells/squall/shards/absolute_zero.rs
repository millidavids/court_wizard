//! Absolute Zero talent: channeled mana drain, stacking slow/damage, and on-release cleanup.

use bevy::prelude::*;

use super::super::casting::{apply_frost_accumulation, apply_or_insert_slow, despawn_storm_rings};
use super::super::components::{AbsoluteZeroSlow, SquallStorm, SquallStormRing};
use super::super::constants::*;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{FrostAccumulation, Health, SlowMovementModifier, Team};
use crate::game::units::wizard::components::{LocalWizard, Mana};
use crate::game::units::wizard::spells::utils::xz_distance;

/// Handles Absolute Zero: continuously drains mana, applies stacking slow + damage to units in storm.
/// Staging attackers (not yet activated at their rally point) are excluded.
pub(crate) fn update_absolute_zero(
    time: Res<Time>,
    // Host-only — guest's ghost SquallStorm would otherwise drain the
    // guest's wizard mana from a host-cast Absolute Zero spell, and the
    // guest's mouse-release would prematurely despawn the host's storm
    // ghost.
    storms: Query<
        (Entity, &SquallStorm),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    rings: Query<Entity, With<SquallStormRing>>,
    mut wizard_query: Query<&mut Mana, With<LocalWizard>>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut AbsoluteZeroSlow>,
            Option<&mut SlowMovementModifier>,
            Option<&mut FrostAccumulation>,
        ),
        Without<crate::game::pathfinding::StagingAttacker>,
    >,
    mut commands: Commands,
) {
    let delta = time.delta_secs();

    for (storm_entity, storm) in storms.iter() {
        if !storm.talent_params.absolute_zero {
            continue;
        }

        // Drain mana continuously
        let Ok(mut mana) = wizard_query.single_mut() else {
            continue;
        };
        let mana_cost = ABSOLUTE_ZERO_MANA_PER_SEC * delta;
        if !mana.consume(mana_cost) {
            // Out of mana — end the channeled storm and its ring
            commands.entity(storm_entity).try_despawn();
            despawn_storm_rings(&mut commands, &rings);
            continue;
        }

        let damage_this_frame = ABSOLUTE_ZERO_DPS * delta;

        for (entity, unit_transform, team, mut health, az_slow, slow_mod, frost_accum) in
            units.iter_mut()
        {
            if *team == Team::Defenders {
                continue;
            }

            let distance = xz_distance(unit_transform.translation, storm.position);

            // During the multiplayer setup stage units are immune, so Absolute Zero
            // applies neither damage nor its slow/frost debuffs (no pre-loading a
            // movement debuff on the frozen enemy army before the fight begins).
            if distance <= storm.radius && !crate::game::units::components::is_setup_immune() {
                health.take_damage(damage_this_frame);

                // Stack slow (Absolute Zero has its own stacking on top of frost
                // accumulation). Framerate-independent: accrue per second via delta.
                let slow_this_frame = ABSOLUTE_ZERO_SLOW_PER_SEC * delta;
                if let Some(mut az) = az_slow {
                    az.accumulated_slow =
                        (az.accumulated_slow - slow_this_frame).max(-ABSOLUTE_ZERO_MAX_SLOW);
                    az.decay_timer = ABSOLUTE_ZERO_SLOW_DECAY_TIME;

                    apply_or_insert_slow(
                        &mut commands,
                        entity,
                        slow_mod,
                        az.accumulated_slow,
                        ABSOLUTE_ZERO_SLOW_DECAY_TIME,
                    );
                } else {
                    commands.entity(entity).insert(AbsoluteZeroSlow {
                        accumulated_slow: -slow_this_frame,
                        decay_timer: ABSOLUTE_ZERO_SLOW_DECAY_TIME,
                    });
                    apply_or_insert_slow(
                        &mut commands,
                        entity,
                        slow_mod,
                        -slow_this_frame,
                        ABSOLUTE_ZERO_SLOW_DECAY_TIME,
                    );
                }

                // Also build frost accumulation (drives blue tint + eventual freeze)
                apply_frost_accumulation(
                    &mut commands,
                    entity,
                    frost_accum,
                    // Continuous frost accumulation at ~5 hits/sec equivalent
                    // (FROST_PER_HIT * 5 per second, scaled by delta for frame-rate independence).
                    FROST_PER_HIT * delta * 5.0,
                );
            }
        }
    }
}

/// Decays and cleans up Absolute Zero slow when units leave the zone or channeling stops.
pub(crate) fn decay_absolute_zero_slow(
    time: Res<Time>,
    storms: Query<&SquallStorm, Without<crate::game::multiplayer::components::GhostSpellEffect>>,
    mut units: Query<(Entity, &Transform, &mut AbsoluteZeroSlow)>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();

    // Check if any active storm has absolute zero
    let has_active_az = storms.iter().any(|s| s.talent_params.absolute_zero);

    for (entity, unit_transform, mut az) in units.iter_mut() {
        // Check if unit is currently inside an active AZ storm
        let mut in_zone = false;
        if has_active_az {
            for storm in storms.iter() {
                if !storm.talent_params.absolute_zero {
                    continue;
                }
                let distance = xz_distance(unit_transform.translation, storm.position);
                if distance <= storm.radius {
                    in_zone = true;
                    break;
                }
            }
        }

        // Only decay if NOT in the zone (or no active AZ storm exists)
        if !in_zone {
            az.decay_timer -= delta;
            if az.decay_timer <= 0.0 {
                commands.entity(entity).remove::<AbsoluteZeroSlow>();
            }
        }
    }
}

/// Ends the Absolute Zero channeled storm when the mouse is released.
pub(crate) fn end_absolute_zero_on_release(
    mut mouse_released: MessageReader<MouseLeftReleased>,
    // Host-only — guest's ghost SquallStorm would otherwise drain the
    // guest's wizard mana from a host-cast Absolute Zero spell, and the
    // guest's mouse-release would prematurely despawn the host's storm
    // ghost.
    storms: Query<
        (Entity, &SquallStorm),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    rings: Query<Entity, With<SquallStormRing>>,
    mut commands: Commands,
) {
    if mouse_released.read().next().is_none() {
        return;
    }

    for (entity, storm) in storms.iter() {
        if storm.talent_params.absolute_zero {
            commands.entity(entity).try_despawn();
            despawn_storm_rings(&mut commands, &rings);
        }
    }
}
