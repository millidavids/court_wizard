//! Compendium systems and re-exports from the setup sub-module.

pub(super) use super::setup::*;

use bevy::prelude::*;

use super::components::CompendiumState;

pub(super) fn cleanup_compendium_state(mut commands: Commands) {
    commands.remove_resource::<CompendiumState>();
}
