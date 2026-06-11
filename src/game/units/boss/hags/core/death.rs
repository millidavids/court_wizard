use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::HagDeathTracker;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{Corpse, HasteModifier, Health, Invulnerable};

/// Prevents hags with eyes from dying — if their health hits zero, immediately
/// resurrect them. Runs BEFORE corpse conversion so they never become corpses.
pub fn resurrect_eyed_hags(
    mut dying_hags: Query<
        (&HagEyeState, &mut Health),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
) {
    for (eye_state, mut health) in &mut dying_hags {
        if health.is_dead() && (eye_state.has_invulnerability_eye || eye_state.has_ability_eye) {
            health.current = health.max * RESURRECT_HEAL_PERCENT;
        }
    }
}

/// Handles permanent death of blind hags (no eyes) after they become corpses.
pub fn intercept_blind_hag_death(
    mut commands: Commands,
    hag_corpses: Query<
        Entity,
        (
            With<Hag>,
            With<Corpse>,
            Without<PermanentlyDead>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
    mut living_eye_states: Query<
        (Entity, &mut HagEyeState),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
    mut death_tracker: ResMut<HagDeathTracker>,
    eye_visuals: Query<(Entity, &ChildOf, &EyeVisual)>,
    eyes_in_flight: Query<(Entity, &EyeInFlight)>,
) {
    for entity in &hag_corpses {
        commands.entity(entity).insert(PermanentlyDead);
        death_tracker.permanent_deaths += 1;

        match death_tracker.permanent_deaths {
            1 => {
                // First permanent death: invulnerability eye disappears
                for (eye_entity, _, eye_visual) in &eye_visuals {
                    if eye_visual.eye_type == EyeType::Invulnerability {
                        commands.entity(eye_entity).try_despawn();
                    }
                }
                for (flight_entity, flight) in &eyes_in_flight {
                    if flight.eye_type == EyeType::Invulnerability {
                        commands.entity(flight_entity).try_despawn();
                    }
                }
                for (hag_entity, mut eye_state) in &mut living_eye_states {
                    eye_state.has_invulnerability_eye = false;
                    commands.entity(hag_entity).remove::<Invulnerable>();
                }
            }
            2 => {
                // Second permanent death: ability eye also disappears
                for (eye_entity, _, _) in &eye_visuals {
                    commands.entity(eye_entity).try_despawn();
                }
                for (flight_entity, _) in &eyes_in_flight {
                    commands.entity(flight_entity).try_despawn();
                }
                for (_, mut eye_state) in &mut living_eye_states {
                    eye_state.has_ability_eye = false;
                    eye_state.has_invulnerability_eye = false;
                }
            }
            _ => {}
        }
    }
}

/// Applies enrage haste to the last surviving hag when 2 are permanently dead.
pub fn apply_enrage_to_last_hag(
    mut commands: Commands,
    hags: Query<
        (Entity, &HagEyeState),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
    death_tracker: Res<HagDeathTracker>,
    existing_haste: Query<&HasteModifier, With<Hag>>,
) {
    if death_tracker.permanent_deaths < 2 {
        return;
    }

    for (entity, _) in &hags {
        // Only add haste if not already enraged
        if existing_haste.get(entity).is_err() {
            commands
                .entity(entity)
                .insert(HasteModifier::new(ENRAGE_SPEED_BONUS, f32::MAX));
        }
    }
}
