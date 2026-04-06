use bevy::prelude::*;

use super::material::WindSwayMaterial;
use super::systems::update_wind_sway_time;
use crate::game::run_conditions::is_gameplay_running;

pub struct WindSwayPlugin;

impl Plugin for WindSwayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WindSwayMaterial>::default())
            .add_systems(Update, update_wind_sway_time.run_if(is_gameplay_running));
    }
}
