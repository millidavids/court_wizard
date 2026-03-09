use bevy::prelude::*;

use crate::game::units::components::{Health, Team};

use super::constants::DEFENDER_SPELL_VULNERABILITY;

/// Sets spell vulnerability on all defender units when Psychopath is active.
/// Runs once on entering InGame state.
pub(super) fn apply_defender_spell_vulnerability(
    mut defenders: Query<(&Team, &mut Health)>,
) {
    for (team, mut health) in defenders.iter_mut() {
        if *team == Team::Defenders {
            health.spell_vulnerability = DEFENDER_SPELL_VULNERABILITY;
        }
    }
}
