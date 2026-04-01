use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded assets for ponds.
#[derive(Resource)]
pub struct PondAssets {
    /// Shared mesh for the pond surface circle (unit circle, scaled per-pond).
    pub mesh: Handle<Mesh>,
    /// Material for the pond surface.
    pub material: Handle<StandardMaterial>,
    /// Shared mesh for ripples (annulus ring).
    pub ripple_mesh: Handle<Mesh>,
}

/// System to pre-load pond assets at startup.
pub(super) fn preload_pond_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = PondAssets {
        mesh: meshes.add(Circle::new(1.0)), // Unit circle, scaled by transform
        material: materials.add(StandardMaterial {
            base_color: POND_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        ripple_mesh: meshes.add(Annulus::new(0.9, 1.0)),
    };
    commands.insert_resource(assets);
}
