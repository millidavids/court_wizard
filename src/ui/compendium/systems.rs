//! Re-export hub for compendium systems split (Phase 16).

pub(super) use super::setup::*;

use bevy::prelude::*;

use super::components::CompendiumState;

pub(super) fn cleanup_compendium_state(mut commands: Commands) {
    commands.remove_resource::<CompendiumState>();
}
