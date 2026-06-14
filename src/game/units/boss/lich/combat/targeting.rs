use bevy::prelude::*;

use super::super::components::*;
use crate::game::resources::KillStats;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    BanishedModifier, Corpse, SleepModifier, TargetingVelocity, Team,
};
use crate::game::units::wizard::components::Wizard;

/// Phase 2 targeting: Updates the Lich's movement targeting toward nearest enemy.
/// Only runs in Combat phase.
pub(crate) fn update_lich_targeting(
    mut commands: Commands,
    mut lich_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            &LichPhase,
        ),
        (With<Lich>, Without<Corpse>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Lich>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<Wizard>,
        ),
    >,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting, phase) in &mut lich_query {
        if *phase != LichPhase::Combat {
            targeting.velocity = Vec3::ZERO;
            continue;
        }

        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            transform,
            *team,
            &mut targeting,
            &mut commands,
            None,
        );
    }
}

/// Phase 2: Selects a random defender as the beam target.
/// The King is excluded until he is the last living defender — guards and any
/// other defender are valid targets in the meantime.
#[allow(clippy::type_complexity)]
pub(crate) fn lich_combat_targeting(
    kill_stats: Res<KillStats>,
    mut lich_query: Query<(&mut LichFingerOfDeath, &LichPhase), (With<Lich>, Without<Corpse>)>,
    defenders: Query<
        (Entity, &Team),
        (
            Without<Corpse>,
            Without<Lich>,
            Without<Boss>,
            Without<SleepModifier>,
            Without<BanishedModifier>,
            Without<Wizard>,
        ),
    >,
    king_query: Query<
        Entity,
        (
            With<crate::game::units::king::components::King>,
            Without<Corpse>,
        ),
    >,
) {
    let king_entity = king_query.iter().next();

    // Single pass over defenders: collect non-king entities, and detect whether
    // the King is also alive. The King only becomes a valid FoD target once
    // every other defender (including guards) is dead.
    let mut non_king: Vec<Entity> = Vec::new();
    let mut king_alive = false;
    for (entity, team) in &defenders {
        if *team != Team::Defenders {
            continue;
        }
        if Some(entity) == king_entity {
            king_alive = true;
        } else {
            non_king.push(entity);
        }
    }

    let eligible: Vec<Entity> = if non_king.is_empty() && king_alive {
        king_entity.into_iter().collect()
    } else {
        non_king
    };

    for (mut fod, phase) in &mut lich_query {
        if *phase != LichPhase::Combat {
            continue;
        }
        if fod.target.is_some() || !fod.is_ready() {
            continue;
        }
        if eligible.is_empty() {
            continue;
        }
        let index = (kill_stats.elapsed_time * 1000.0) as usize % eligible.len();
        fod.target = Some(eligible[index]);
    }
}
