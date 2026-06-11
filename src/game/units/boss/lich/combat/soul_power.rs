use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::resources::KillStats;
use crate::game::units::components::Corpse;

/// Checks if it's time to spawn the Lich mid-game.
/// The Lich spawns as an extra wave after all normal waves have been dispatched
/// and every attacker (including staging) is dead.
#[allow(clippy::too_many_arguments)]
pub(crate) fn track_soul_power(
    kill_stats: Res<KillStats>,
    mut query: Query<(&mut SoulPower, &LichPhase), (With<Lich>, Without<Corpse>)>,
) {
    for (mut soul_power, phase) in &mut query {
        // Only accumulate during summoning phase
        if *phase != LichPhase::Summoning {
            continue;
        }

        let current_undead_killed = kill_stats.undead_killed;
        if current_undead_killed > soul_power.last_known_undead_killed {
            let new_kills = current_undead_killed - soul_power.last_known_undead_killed;
            soul_power.current =
                (soul_power.current + new_kills as f32 * SOUL_POWER_PER_KILL).min(soul_power.max);
            soul_power.last_known_undead_killed = current_undead_killed;
        }
    }
}

/// Checks if soul power is full and transitions to Phase 2.
pub(crate) fn lich_phase_transition(
    mut commands: Commands,
    mut query: Query<(Entity, &SoulPower, &mut LichPhase), (With<Lich>, Without<Corpse>)>,
) {
    for (entity, soul_power, mut phase) in &mut query {
        if *phase != LichPhase::Summoning || !soul_power.is_full() {
            continue;
        }

        *phase = LichPhase::Combat;

        commands
            .entity(entity)
            .remove::<LichSummonTimer>()
            .insert(LichFingerOfDeath::new());
    }
}
