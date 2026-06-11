use super::super::super::super::components::Spell;
use super::super::components::{
    GreaseIgnited, GreaseOilSlickDebuff, GreaseRegenerating, GreaseZone, GreaseZonePresenceTracker,
};
use super::super::constants;
use crate::game::units::components::{Corpse, Health, RootedModifier, SlowMovementModifier};
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use bevy::prelude::*;
use rand::Rng;

/// Applies slow to units inside grease zones, ticks time_alive for non-ignited zones,
/// and handles Slip and Fall / Oil Slick talent effects.
#[allow(clippy::too_many_arguments)]
pub fn apply_grease_slow(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut zones: Query<(
        Entity,
        &mut GreaseZone,
        Has<GreaseIgnited>,
        Has<GreaseRegenerating>,
    )>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            Option<&mut SlowMovementModifier>,
            Option<&mut GreaseZonePresenceTracker>,
            Option<&GreaseOilSlickDebuff>,
            Option<&mut Health>,
        ),
        Without<Corpse>,
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let delta = time.delta_secs();
    let rng = &mut game_rng.0;

    for (zone_entity, mut zone, is_ignited, is_regenerating) in &mut zones {
        // Only track time_alive for non-ignited zones
        // (ignited zones track time_alive in apply_grease_burn instead)
        if !is_ignited {
            zone.time_alive += delta;
        }

        // Don't apply slow while regenerating (not yet slippery)
        if is_regenerating {
            continue;
        }

        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;

            let mut units_slowed: u32 = 0;
            let needs_tracking = zone.talent_params.slip_and_fall || zone.talent_params.oil_slick;

            for (entity, transform, existing_slow, existing_tracker, has_oil_debuff, mut health) in
                &mut targets
            {
                let dist = xz_distance(zone.origin, transform.translation);

                if dist <= zone.radius {
                    // Apply slow
                    if let Some(mut slow) = existing_slow {
                        slow.apply(zone.slow_modifier, zone.slow_duration);
                    } else {
                        let modifier =
                            SlowMovementModifier::new(zone.slow_modifier, zone.slow_duration);
                        commands
                            .entity(entity)
                            .queue_silenced(move |mut e: EntityWorldMut| {
                                e.insert(modifier);
                            });
                    }
                    units_slowed += 1;

                    if needs_tracking {
                        let is_new = existing_tracker
                            .as_ref()
                            .is_none_or(|t| t.zone_entity != zone_entity);

                        if is_new {
                            commands
                                .entity(entity)
                                .insert(GreaseZonePresenceTracker { zone_entity });

                            // Talent: Slip and Fall — stun on zone entry
                            if zone.talent_params.slip_and_fall {
                                let roll: f32 = rng.random_range(0.0..1.0);
                                if roll < constants::SLIP_AND_FALL_CHANCE {
                                    commands.entity(entity).insert(RootedModifier::new(
                                        constants::SLIP_AND_FALL_STUN_DURATION,
                                    ));
                                }
                            }

                            // Talent: Oil Slick — apply vulnerability debuff (once per unit)
                            if zone.talent_params.oil_slick
                                && has_oil_debuff.is_none()
                                && let Some(ref mut health) = health
                            {
                                health.spell_vulnerability += constants::OIL_SLICK_VULNERABILITY;
                                commands.entity(entity).insert(GreaseOilSlickDebuff::new());
                            }
                        }
                    }
                } else if needs_tracking {
                    // Unit is outside the zone — clean up tracker and debuffs
                    if let Some(ref tracker) = existing_tracker
                        && tracker.zone_entity == zone_entity
                    {
                        commands
                            .entity(entity)
                            .remove::<GreaseZonePresenceTracker>();

                        // Remove Oil Slick vulnerability when leaving
                        if let Some(debuff) = has_oil_debuff {
                            if let Some(ref mut health) = health {
                                health.spell_vulnerability =
                                    (health.spell_vulnerability - debuff.vulnerability).max(0.0);
                            }
                            commands.entity(entity).remove::<GreaseOilSlickDebuff>();
                        }
                    }
                }
            }

            // Track talent progress
            if units_slowed > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::Grease, units_slowed);
            }
        }
    }
}
