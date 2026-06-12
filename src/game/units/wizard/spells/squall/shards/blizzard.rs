//! Blizzard talent: storm follows the cursor slowly.

use bevy::prelude::*;

use super::super::components::SquallStorm;
use super::super::constants::BLIZZARD_FOLLOW_SPEED;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::wizard::components::{LocalWizard, Wizard};
use crate::game::units::wizard::spells::utils::{
    clamp_to_spell_range_ground, get_cursor_world_position,
};

/// Handles Blizzard talent: storm follows cursor slowly.
pub(crate) fn update_blizzard_position(
    time: Res<Time>,
    // Host-only — guest's ghost SquallStorm must NOT independently spawn
    // ice / apply CC; the host's authoritative storm drives gameplay and
    // CRDT carries the result.
    mut storms: Query<
        &mut SquallStorm,
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    wizard_query: Query<&Wizard, With<LocalWizard>>,
    local_origin: Res<crate::game::units::wizard::spells::utils::LocalSpellOrigin>,
) {
    let Some(cursor_pos) = get_cursor_world_position(&camera_query, &corrected_cursor) else {
        return;
    };

    let Ok(wizard) = wizard_query.single() else {
        return;
    };

    for mut storm in storms.iter_mut() {
        // Both Blizzard and Absolute Zero make the storm follow the cursor
        if !storm.talent_params.blizzard && !storm.talent_params.absolute_zero {
            continue;
        }

        // Clamp target to spell range
        let target = clamp_to_spell_range_ground(
            cursor_pos,
            local_origin.0,
            wizard.spell_range,
            storm.radius,
        );

        // Lerp storm position toward cursor
        let direction = Vec3::new(
            target.x - storm.position.x,
            0.0,
            target.z - storm.position.z,
        );
        let distance = direction.length();

        if distance > 1.0 {
            let move_amount = BLIZZARD_FOLLOW_SPEED * time.delta_secs();
            let move_vec = direction.normalize() * move_amount.min(distance);
            storm.position += move_vec;
        }
    }
}
