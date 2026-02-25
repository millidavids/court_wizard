use bevy::prelude::*;

use crate::game::constants::{ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR};

use super::styles::*;

/// Pre-loaded meshes and materials for infantry units.
#[derive(Resource)]
pub struct InfantryAssets {
    pub mesh: Handle<Mesh>,
    pub defender_material: Handle<StandardMaterial>,
    pub attacker_material: Handle<StandardMaterial>,
    pub kings_guard_material: Handle<StandardMaterial>,
    pub defender_corpse_material: Handle<StandardMaterial>,
    pub attacker_corpse_material: Handle<StandardMaterial>,
    pub undead_corpse_material: Handle<StandardMaterial>,
    pub undead_material: Handle<StandardMaterial>,
}

/// System to pre-load infantry assets at startup.
pub(super) fn preload_infantry_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = InfantryAssets {
        mesh: meshes.add(Circle::new(UNIT_RADIUS)),
        defender_material: materials.add(StandardMaterial {
            base_color: DEFENDER_COLOR,
            unlit: true,
            ..default()
        }),
        attacker_material: materials.add(StandardMaterial {
            base_color: ATTACKER_COLOR,
            unlit: true,
            ..default()
        }),
        kings_guard_material: materials.add(StandardMaterial {
            base_color: KINGS_GUARD_COLOR,
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
            base_color: Color::srgb(0.3, 0.8, 0.4),
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
