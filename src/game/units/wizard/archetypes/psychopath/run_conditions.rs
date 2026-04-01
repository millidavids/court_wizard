use crate::config::{GameConfig, WizardType};
use bevy::prelude::*;

/// Returns true when the Psychopath archetype is selected.
pub(crate) fn is_psychopath(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::Psychopath
}
