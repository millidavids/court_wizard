use crate::game::units::components::{AnthemResilience, BattleHymnModifier, EchoingSong};
use bevy::prelude::*;

/// Custom tick for BattleHymnModifier that handles EchoingSong re-apply on expiry.
pub fn update_battle_hymn_modifier(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BattleHymnModifier, Option<&mut EchoingSong>)>,
) {
    let delta = time.delta_secs();
    for (entity, mut modifier, echoing_song) in query.iter_mut() {
        if modifier.update(delta) {
            // Check for Echoing Song: re-apply at reduced duration
            if let Some(echo) = echoing_song {
                modifier.time_remaining = echo.echo_duration;
                // Consume the echo — only triggers once
                commands.entity(entity).remove::<EchoingSong>();
            } else {
                commands
                    .entity(entity)
                    .remove::<(BattleHymnModifier, AnthemResilience)>();
            }
        }
    }
}
