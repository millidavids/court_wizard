use bevy::prelude::*;

use crate::game::run_conditions::is_gameplay_running;

use super::systems;

pub struct ElitePlugin;

impl Plugin for ElitePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::apply_elite_health_bonus,
                systems::remove_elite_health_bonus,
            )
                .run_if(is_gameplay_running),
        );
    }
}
