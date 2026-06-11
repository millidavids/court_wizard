use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::HagDeathTracker;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{Corpse, Health};

/// Josephina's frenzy — 5x attack speed after leap landing for MAULING_DURATION.
pub fn josephina_vicious_mauling(
    time: Res<Time>,
    death_tracker: Res<HagDeathTracker>,
    mut josephina_query: Query<
        (&HagIdentity, &HagEyeState, &LeapState, &mut MaulingState),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();
    let enraged = death_tracker.permanent_deaths >= 2;

    for (identity, eye_state, leap, mut mauling) in &mut josephina_query {
        if *identity != HagIdentity::Josephina || (!eye_state.has_ability_eye && !enraged) {
            mauling.frenzy_timer = 0.0;
            continue;
        }

        // Activate frenzy on leap landing
        if matches!(leap, LeapState::Landing { .. }) && !mauling.is_frenzied() {
            mauling.frenzy_timer = MAULING_DURATION;
        }

        // Tick down frenzy timer
        if mauling.is_frenzied() {
            mauling.frenzy_timer = (mauling.frenzy_timer - delta).max(0.0);
        }
    }
}

/// Josephina's corpse consume — stationary for 3s near a corpse, heals, despawns corpse.
#[allow(clippy::type_complexity)]
pub fn josephina_corpse_consume(
    time: Res<Time>,
    mut commands: Commands,
    death_tracker: Res<HagDeathTracker>,
    mut josephina_query: Query<
        (
            Entity,
            &Transform,
            &HagIdentity,
            &HagEyeState,
            &mut Health,
            Option<&mut CorpseConsumeState>,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    corpses: Query<(Entity, &Transform), With<Corpse>>,
) {
    let delta = time.delta_secs();
    let enraged = death_tracker.permanent_deaths >= 2;

    for (entity, transform, identity, eye_state, mut health, consume_state) in &mut josephina_query
    {
        if *identity != HagIdentity::Josephina || (!eye_state.has_ability_eye && !enraged) {
            // Cancel consume if eye lost
            if consume_state.is_some() {
                commands.entity(entity).remove::<CorpseConsumeState>();
            }
            continue;
        }

        if let Some(mut state) = consume_state {
            // Currently consuming a corpse
            state.timer -= delta;
            if state.timer <= 0.0 {
                // Heal and despawn corpse
                let heal = health.max * CORPSE_CONSUME_HEAL_PERCENT;
                health.current = (health.current + heal).min(health.max);
                commands.entity(state.corpse_entity).try_despawn();
                commands.entity(entity).remove::<CorpseConsumeState>();
            }
        } else {
            // Check if there's a nearby corpse to consume (only if health not full)
            if health.current >= health.max * CORPSE_CONSUME_HEALTH_THRESHOLD {
                continue;
            }

            let hag_pos = transform.translation;
            let mut nearest_corpse: Option<(Entity, f32)> = None;

            for (corpse_entity, corpse_transform) in &corpses {
                let dx = corpse_transform.translation.x - hag_pos.x;
                let dz = corpse_transform.translation.z - hag_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist < CORPSE_CONSUME_RANGE {
                    if let Some((_, best_dist)) = nearest_corpse {
                        if dist < best_dist {
                            nearest_corpse = Some((corpse_entity, dist));
                        }
                    } else {
                        nearest_corpse = Some((corpse_entity, dist));
                    }
                }
            }

            if let Some((corpse_entity, _)) = nearest_corpse {
                commands.entity(entity).insert(CorpseConsumeState {
                    timer: CORPSE_CONSUME_DURATION,
                    corpse_entity,
                });
            }
        }
    }
}
