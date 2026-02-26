use bevy::prelude::*;

use crate::game::constants::{
    ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_BASE, UNDEAD_CORPSE_COLOR,
};

use super::constants::ARROW_WIDTH;
use super::styles::*;

/// Pre-loaded meshes and materials for archer units.
#[derive(Resource)]
pub struct ArcherAssets {
    pub mesh: Handle<Mesh>,
    pub arrow_mesh: Handle<Mesh>,
    pub defender_material: Handle<StandardMaterial>,
    pub attacker_material: Handle<StandardMaterial>,
    pub arrow_material: Handle<StandardMaterial>,
    pub defender_corpse_material: Handle<StandardMaterial>,
    pub attacker_corpse_material: Handle<StandardMaterial>,
    pub undead_corpse_material: Handle<StandardMaterial>,
    pub undead_material: Handle<StandardMaterial>,
}

/// System to pre-load archer assets at startup.
pub(super) fn preload_archer_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = ArcherAssets {
        mesh: meshes.add(Circle::new(ARCHER_RADIUS)),
        arrow_mesh: meshes.add(Circle::new(ARROW_WIDTH)),
        defender_material: materials.add(StandardMaterial {
            base_color: DEFENDER_ARCHER_COLOR,
            unlit: true,
            ..default()
        }),
        attacker_material: materials.add(StandardMaterial {
            base_color: ATTACKER_ARCHER_COLOR,
            unlit: true,
            ..default()
        }),
        arrow_material: materials.add(StandardMaterial {
            base_color: ARROW_COLOR,
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
