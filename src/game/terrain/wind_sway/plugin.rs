use bevy::prelude::*;

use super::material::WindSwayMaterial;
use super::systems::update_wind_sway_time;
use crate::game::run_conditions::is_spell_effects_active;

pub struct WindSwayPlugin;

impl Plugin for WindSwayPlugin {
    fn build(&self, app: &mut App) {
        // Wind sway is a global shader time tick — runs on both MP peers so
        // trees and flora animate on the guest. No gameplay state mutated.
        app.add_plugins(MaterialPlugin::<WindSwayMaterial>::default())
            .add_systems(
                Update,
                update_wind_sway_time.run_if(is_spell_effects_active),
            );
    }
}
