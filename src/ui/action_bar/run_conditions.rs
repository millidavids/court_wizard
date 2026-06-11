use bevy::prelude::*;

use crate::config::GameConfig;

/// The action bar is hidden for archetypes that don't choose spells from it
/// (RuneCaster casts via rune combos, Randomancer via the roulette).
pub(super) fn action_bar_enabled(config: Res<GameConfig>) -> bool {
    !config.wizard_type.uses_exclusive_casting()
}
