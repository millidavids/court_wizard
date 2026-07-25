//! Sleet Storm talent: evasion debuff applied inside the storm radius.

use bevy::prelude::*;

use super::super::components::SquallStorm;
use super::super::constants::{SLEET_STORM_EVASION_CHANCE, SLEET_STORM_EVASION_DURATION};
use crate::game::units::components::{FogEvasionModifier, Health, Team};
use crate::game::units::wizard::spells::utils::xz_distance;

/// Applies Sleet Storm evasion debuff to enemies inside the storm radius.
pub(crate) fn apply_sleet_storm_evasion(
    storms: Query<&SquallStorm, Without<crate::game::multiplayer::components::GhostSpellEffect>>,
    mut units: Query<
        (Entity, &Transform, &Team, Option<&mut FogEvasionModifier>),
        (
            With<Health>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut commands: Commands,
) {
    for storm in storms.iter() {
        if !storm.talent_params.sleet_storm {
            continue;
        }

        for (entity, unit_transform, team, fog_evasion) in units.iter_mut() {
            if *team == Team::Defenders {
                continue;
            }

            let distance = xz_distance(unit_transform.translation, storm.position);

            if distance <= storm.radius {
                if let Some(mut evasion) = fog_evasion {
                    evasion.refresh(SLEET_STORM_EVASION_DURATION);
                } else {
                    commands.entity(entity).insert(FogEvasionModifier::new(
                        SLEET_STORM_EVASION_CHANCE,
                        SLEET_STORM_EVASION_DURATION,
                    ));
                }
            }
        }
    }
}
