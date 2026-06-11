use super::super::components::SleepTalentParams;
use super::super::constants;
use crate::game::units::components::{
    Comatose, Corpse, Health, NarcolepticWave, NightTerrors, SleepModifier, Sleepwalking, Team,
    TemporaryHitPoints, apply_damage_to_unit,
};
use bevy::prelude::*;

/// Apply sleep to all targets in radius, returning the number of enemies hit.
pub(crate) fn apply_sleep(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    empowerment: f32,
    targets: &Query<(Entity, &Transform, &Health, &Team), Without<Corpse>>,
    talent_params: &SleepTalentParams,
) -> u32 {
    let duration = constants::SLEEP_DURATION * empowerment * talent_params.duration_mult;
    let bonus = constants::BONUS_DAMAGE_MULTIPLIER * talent_params.bonus_damage_mult;
    let mut hit_count = 0u32;

    for (entity, transform, health, team) in targets.iter() {
        let distance = crate::game::units::wizard::spells::utils::xz_distance(
            transform.translation,
            circle_pos,
        );
        if distance > radius {
            continue;
        }

        // Eternal Slumber: enemies below 25% HP are killed instantly
        if talent_params.eternal_slumber
            && *team != Team::Defenders
            && health.current <= health.max * constants::ETERNAL_SLUMBER_HP_THRESHOLD
        {
            // Set health to 0 to kill (death system will handle corpse conversion)
            commands.entity(entity).insert(Health {
                current: 0.0,
                max: health.max,
                spell_vulnerability: health.spell_vulnerability,
                healing_reduction: health.healing_reduction,
            });
            hit_count += 1;
            continue;
        }

        let mut modifier = if talent_params.dreamwalker {
            SleepModifier::new(constants::DREAMWALKER_DURATION, bonus)
        } else {
            SleepModifier::new(duration, bonus)
        };

        let mut entity_commands = commands.entity(entity);

        // Night Terrors: minor DPS while sleeping
        if talent_params.night_terrors {
            entity_commands.insert(NightTerrors::new(constants::NIGHT_TERRORS_DPS));
        }

        // Comatose: require 30% max HP damage to wake
        if talent_params.comatose {
            entity_commands.insert(Comatose::new(constants::COMATOSE_WAKE_THRESHOLD));
        }

        // Narcoleptic Wave: spreading sleep after delay
        if talent_params.narcoleptic_wave {
            entity_commands.insert(NarcolepticWave::new(
                constants::NARCOLEPTIC_SPREAD_DELAY,
                constants::NARCOLEPTIC_SPREAD_RADIUS,
            ));
        }

        // Dreamwalker: enemies sleepwalk back toward spawn
        if talent_params.dreamwalker {
            entity_commands.insert(Sleepwalking::new(constants::DREAMWALKER_SPEED_MULT));
            modifier.full_duration = constants::DREAMWALKER_DURATION;
        }

        entity_commands.insert(modifier);
        hit_count += 1;
    }

    hit_count
}

/// Tick sleep timer and remove when expired (along with all sub-components).
pub fn update_sleep_modifiers(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut SleepModifier),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
) {
    let delta = time.delta_secs();
    for (entity, mut sleep) in query.iter_mut() {
        if sleep.update(delta) {
            commands.entity(entity).remove::<(
                SleepModifier,
                NightTerrors,
                Comatose,
                NarcolepticWave,
                Sleepwalking,
            )>();
        }
    }
}

/// Night Terrors talent: apply DPS to sleeping units.
#[allow(clippy::type_complexity)]
pub fn update_night_terrors(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut NightTerrors,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
) {
    let delta = time.delta_secs();
    for (entity, mut terrors, mut health, mut temp_hp) in query.iter_mut() {
        terrors.tick_accumulator += delta;
        if terrors.tick_accumulator >= constants::NIGHT_TERRORS_TICK_INTERVAL {
            let damage = terrors.dps * terrors.tick_accumulator;
            apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
            terrors.tick_accumulator = 0.0;
            if health.current <= 0.0 {
                commands.entity(entity).insert(Corpse);
            }
        }
    }
}

/// Narcoleptic Wave: after the delay, spread sleep to nearby awake enemies.
#[allow(clippy::type_complexity)]
pub fn update_narcoleptic_wave(
    time: Res<Time>,
    mut commands: Commands,
    mut wave_query: Query<
        (
            Entity,
            &mut NarcolepticWave,
            &SleepModifier,
            &Transform,
            &Team,
            Has<NightTerrors>,
            Has<Comatose>,
            Option<&Sleepwalking>,
        ),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
    awake_targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<SleepModifier>,
            Without<Corpse>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
) {
    let delta = time.delta_secs();

    struct SpreadFrame {
        position: Vec3,
        radius: f32,
        duration: f32,
        bonus_damage: f32,
        has_night_terrors: bool,
        has_comatose: bool,
        sleepwalking: Option<f32>,
    }

    let mut spread_events: Vec<SpreadFrame> = Vec::new();

    for (entity, mut wave, sleep, transform, team, has_night_terrors, has_comatose, sleepwalking) in
        wave_query.iter_mut()
    {
        wave.timer -= delta;
        if wave.timer <= 0.0 {
            // Remove the component now that it's spread — avoids iterating every frame
            commands.entity(entity).remove::<NarcolepticWave>();
            // Only spread from non-defender sleepers
            if *team != Team::Defenders {
                spread_events.push(SpreadFrame {
                    position: transform.translation,
                    radius: wave.radius,
                    duration: sleep.full_duration * 0.5,
                    bonus_damage: sleep.bonus_damage_multiplier,
                    has_night_terrors,
                    has_comatose,
                    sleepwalking: sleepwalking.map(|s| s.speed_mult),
                });
            }
        }
    }

    // Apply spreads
    for event in spread_events {
        for (entity, transform, team) in awake_targets.iter() {
            if *team == Team::Defenders {
                continue;
            }
            let dist = crate::game::units::wizard::spells::utils::xz_distance(
                transform.translation,
                event.position,
            );
            if dist <= event.radius {
                let modifier = SleepModifier::new(event.duration, event.bonus_damage);
                let mut entity_commands = commands.entity(entity);
                entity_commands.insert(modifier);
                if event.has_night_terrors {
                    entity_commands.insert(NightTerrors::new(constants::NIGHT_TERRORS_DPS));
                }
                if event.has_comatose {
                    entity_commands.insert(Comatose::new(constants::COMATOSE_WAKE_THRESHOLD));
                }
                if let Some(speed_mult) = event.sleepwalking {
                    entity_commands.insert(Sleepwalking::new(speed_mult));
                }
            }
        }
    }
}
