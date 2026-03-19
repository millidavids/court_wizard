use bevy::prelude::*;

use crate::game::constants::{
    ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_BASE, UNDEAD_CORPSE_COLOR,
};

use super::constants::*;

/// Pre-loaded meshes and materials for shielder units.
#[derive(Resource)]
#[allow(dead_code)]
pub struct ShielderAssets {
    pub mesh: Handle<Mesh>,
    pub attacker_material: Handle<StandardMaterial>,
    pub defender_corpse_material: Handle<StandardMaterial>,
    pub attacker_corpse_material: Handle<StandardMaterial>,
    pub undead_corpse_material: Handle<StandardMaterial>,
    pub undead_material: Handle<StandardMaterial>,
}

/// System to pre-load shielder assets at startup.
pub(super) fn preload_shielder_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = ShielderAssets {
        mesh: meshes.add(Circle::new(SHIELDER_RADIUS)),
        attacker_material: materials.add(StandardMaterial {
            base_color: ATTACKER_SHIELDER_COLOR,
            unlit: true,
            ..default()
        }),
        defender_corpse_material: materials.add(StandardMaterial {
            base_color: DEFENDER_CORPSE_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        attacker_corpse_material: materials.add(StandardMaterial {
            base_color: ATTACKER_CORPSE_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        undead_corpse_material: materials.add(StandardMaterial {
            base_color: UNDEAD_CORPSE_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        undead_material: materials.add(StandardMaterial {
            base_color: UNDEAD_BASE,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
