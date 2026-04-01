use crate::state::AppState;
use bevy::prelude::*;

use super::run_conditions::is_psychopath;
use super::systems;

pub struct PsychopathPlugin;

impl Plugin for PsychopathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            systems::apply_defender_spell_vulnerability.run_if(is_psychopath),
        );
    }
}
