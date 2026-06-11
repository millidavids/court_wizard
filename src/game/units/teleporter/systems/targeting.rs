use bevy::prelude::*;

use crate::game::units::components::{Corpse, InMelee, TargetingVelocity, Team};
use crate::game::units::king::components::King;
use crate::game::units::teleporter::components::{Teleporter, TeleporterState};

pub(crate) fn update_teleporter_targeting(
    mut commands: Commands,
    mut teleporters: Query<
        (Entity, &Transform, &TeleporterState, &mut TargetingVelocity),
        (With<Teleporter>, Without<Corpse>),
    >,
    king_query: Query<(&Transform, &Team), (With<King>, Without<Corpse>)>,
) {
    let Some((king_transform, _)) = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
    else {
        return;
    };
    let king_pos = king_transform.translation;

    for (entity, transform, state, mut targeting) in &mut teleporters {
        if matches!(state, TeleporterState::Channeling { .. }) {
            targeting.velocity = Vec3::ZERO;
            targeting.distance_to_target = 0.0;
            commands.entity(entity).remove::<InMelee>();
            continue;
        }

        let to_king = Vec3::new(
            king_pos.x - transform.translation.x,
            0.0,
            king_pos.z - transform.translation.z,
        );
        targeting.velocity = to_king.normalize_or_zero();
        targeting.distance_to_target = to_king.length();
        commands.entity(entity).remove::<InMelee>();
    }
}
