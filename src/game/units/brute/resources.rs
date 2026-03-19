use bevy::prelude::*;

/// Brute resource (kept for plugin registration; brute now uses infantry sprite assets).
#[derive(Resource, Default)]
pub struct BruteAssets;

/// System to initialize brute resource at startup.
pub(super) fn preload_brute_assets(mut commands: Commands) {
    commands.insert_resource(BruteAssets);
}
