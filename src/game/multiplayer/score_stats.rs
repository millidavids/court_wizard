//! Per-match wizard spell-stat accumulation and the assembled scoreboard.
//!
//! Kills/deaths come from the host's authoritative `KillStats` (side-level).
//! Spell damage and healing are per-wizard: each peer tallies only its OWN
//! wizard's spell output locally (via the `units::spell_stats` tally markers),
//! then the two halves are exchanged at game-over (see
//! `host_systems::check_mp_king_death` and `guest_visuals::handle_game_over_message`).
//!
//! Resource lifetimes are intentionally split because the two resources have
//! different valid windows:
//! - `LocalWizardStats` is a whole-match accumulator (inserted in `init_mp_game`,
//!   removed in `cleanup_mp_game`) — it must live the entire match to sum every
//!   spell hit, and is read once at game-over.
//! - `MatchStats` is only constructible at game-over (it needs the final
//!   `KillStats` plus the network exchange), so it is inserted then and removed
//!   in `cleanup_mp_score_screen`. Both cleanups fire on every exit path because
//!   `MultiplayerGameState` is a sub-state of `AppState::MultiplayerGame`.

use bevy::prelude::*;

use crate::game::units::spell_stats::{SpellDamageTally, SpellHealTally, SpellTally};

/// The LOCAL player's wizard spell stats, accumulated across the whole match.
///
/// Inserted in `init_mp_game`, removed in `cleanup_mp_game`. The accumulator
/// runs only in multiplayer, so this is never present (or needed) in single-player.
#[derive(Resource, Default)]
pub(crate) struct LocalWizardStats {
    pub spell_damage: f32,
    pub spell_healing: f32,
}

/// Both players' four scoreboard stats. "your_*" is always the local player's side.
#[derive(Resource, Default)]
pub(crate) struct MatchStats {
    pub your_kills: u32,
    pub your_deaths: u32,
    pub your_damage: f32,
    pub your_healing: f32,
    pub enemy_kills: u32,
    pub enemy_deaths: u32,
    pub enemy_damage: f32,
    pub enemy_healing: f32,
}

impl MatchStats {
    /// Assembles the scoreboard from the host-authoritative side kill counts plus
    /// both wizards' spell stats, mapping kills/deaths to the LOCAL player's role.
    ///
    /// This is the single source of the side ↔ stat mapping so the host and guest
    /// can't drift: the host passes `local_is_defender = true`, the guest `false`.
    /// In a 1v1 the enemy's kills/deaths are exactly the mirror of yours.
    pub(crate) fn assemble(
        local_is_defender: bool,
        defenders_killed: u32,
        attackers_and_undead_killed: u32,
        your_damage: f32,
        your_healing: f32,
        enemy_damage: f32,
        enemy_healing: f32,
    ) -> Self {
        let (your_kills, your_deaths) = if local_is_defender {
            // Defenders side: you killed attackers/undead; your losses are defenders.
            (attackers_and_undead_killed, defenders_killed)
        } else {
            // Attackers side: you killed defenders; your losses are attackers/undead.
            (defenders_killed, attackers_and_undead_killed)
        };
        Self {
            your_kills,
            your_deaths,
            your_damage,
            your_healing,
            enemy_kills: your_deaths,
            enemy_deaths: your_kills,
            enemy_damage,
            enemy_healing,
        }
    }
}

/// Sums one tally type's markers into `target` and removes them.
fn drain_tally<T: SpellTally>(
    commands: &mut Commands,
    query: &Query<(Entity, &T)>,
    target: &mut f32,
) {
    for (entity, tally) in query {
        *target += tally.amount();
        commands.entity(entity).remove::<T>();
    }
}

/// Folds the one-frame spell tally markers into `LocalWizardStats` and clears them.
///
/// Registered with `run_if(in_mp_running)`, so it runs on both multiplayer peers
/// (each accumulating its own wizard's output — host on real units, guest on
/// ghost units) and NOT in single-player. `LocalWizardStats` is always present
/// in multiplayer (inserted by `init_mp_game` before the first `Update`).
pub(crate) fn accumulate_wizard_spell_stats(
    mut commands: Commands,
    mut stats: ResMut<LocalWizardStats>,
    damage_tallies: Query<(Entity, &SpellDamageTally)>,
    heal_tallies: Query<(Entity, &SpellHealTally)>,
) {
    drain_tally(&mut commands, &damage_tallies, &mut stats.spell_damage);
    drain_tally(&mut commands, &heal_tallies, &mut stats.spell_healing);
}
