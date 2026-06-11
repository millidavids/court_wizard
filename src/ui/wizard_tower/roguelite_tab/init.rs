use bevy::prelude::*;
use rand::Rng;

use crate::config::GameConfig;
use crate::game::game_mode::components::{ActiveToggles, RogueliteModifiers};

use super::components::{ExpandedToggles, PendingToggles, SeedInputState};

fn random_seed() -> u64 {
    rand::rng().random_range(0..super::constants::MAX_SEED)
}

/// Initializes roguelite tab resources when the tab is first shown.
/// Call this when entering the roguelite tab (no active run).
pub(crate) fn init_roguelite_tab_resources(
    commands: &mut Commands,
    config: &mut GameConfig,
    existing_modifiers: Option<&RogueliteModifiers>,
    existing_pending: Option<&PendingToggles>,
    existing_active_toggles: Option<&ActiveToggles>,
) {
    let mods = existing_modifiers.cloned().unwrap_or_default();
    if existing_modifiers.is_none() {
        commands.insert_resource(mods.clone());
    }

    // Only preserve pending toggles if ActiveToggles exists (returning from wizard select).
    // Otherwise start fresh to prevent stale selections.
    let pending_toggles = if existing_active_toggles.is_some() {
        existing_pending.cloned().unwrap_or_default()
    } else {
        PendingToggles::default()
    };
    commands.insert_resource(pending_toggles);

    // Always generate a fresh random seed when opening this tab
    let seed = random_seed();
    config.seed = Some(seed);
    let seed_text = seed.to_string();
    commands.insert_resource(SeedInputState {
        text: seed_text,
        focused: false,
    });

    commands.insert_resource(ExpandedToggles::default());
}

/// Removes roguelite tab resources.
pub(crate) fn cleanup_roguelite_tab_resources(mut commands: Commands) {
    commands.remove_resource::<SeedInputState>();
    commands.remove_resource::<ExpandedToggles>();
    commands.remove_resource::<PendingToggles>();
}
