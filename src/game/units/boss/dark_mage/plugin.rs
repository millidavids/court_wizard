use bevy::prelude::*;

use super::resources;
use super::systems::*;
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;
use crate::game::units::boss::dark_mage::components::DarkMage;

pub struct DarkMagePlugin;

impl Plugin for DarkMagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_dark_mage_assets)
            .add_systems(
                Update,
                (
                    dark_mage_movement.in_set(MovementCalculationSet),
                    dark_mage_spell_queue,
                    dark_mage_ai.after(dark_mage_spell_queue),
                    dark_mage_teleport,
                    update_meteor_explosions,
                    update_meteor_projectiles,
                    update_lightning_strikes,
                    update_lightning_bolts,
                    update_visual_effects,
                    update_plague_clouds,
                    update_plague_particles,
                    broadcast_plague_hazards,
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<DarkMage>),
            )
            .add_systems(
                Update,
                dark_mage_enrage
                    .after(PostCombatSet)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<DarkMage>),
            );
    }
}
