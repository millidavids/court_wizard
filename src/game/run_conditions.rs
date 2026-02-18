use bevy::prelude::*;

use crate::config::{GameConfig, WizardType};

/// Check if any entities with the specified component exist.
/// Used to avoid running systems when there are no relevant entities.
///
/// Example: `any_exist::<MagicMissile>()` will only return true if there are magic missiles in the world.
///
/// This is more efficient than running systems with empty queries every frame.
pub fn any_exist<T: Component>() -> impl Fn(Query<(), With<T>>) -> bool {
    |query: Query<(), With<T>>| !query.is_empty()
}

/// Returns true if the active wizard type is RuneCaster.
pub fn is_rune_caster(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::RuneCaster
}

/// Returns true if the active wizard type is Randomancer.
pub fn is_randomancer(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::Randomancer
}

/// Returns true if the active wizard type is Arcanorouter.
pub fn is_arcanorouter(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::Arcanorouter
}
