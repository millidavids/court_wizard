use bevy::prelude::*;

use crate::game::constants::{
    ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_BASE, UNDEAD_CORPSE_COLOR,
};

use super::constants::*;

/// Pre-loaded meshes and materials for dispeller units.
#[derive(Resource)]
#[allow(dead_code)]
pub struct DispellerAssets {
    pub mesh: Handle<Mesh>,
    pub bolt_mesh: Handle<Mesh>,
    pub attacker_material: Handle<StandardMaterial>,
    pub bolt_material: Handle<StandardMaterial>,
    pub defender_corpse_material: Handle<StandardMaterial>,
    pub attacker_corpse_material: Handle<StandardMaterial>,
    pub undead_corpse_material: Handle<StandardMaterial>,
    pub undead_material: Handle<StandardMaterial>,
}

/// System to pre-load dispeller assets at startup.
pub(super) fn preload_dispeller_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = DispellerAssets {
        mesh: meshes.add(Circle::new(DISPELLER_RADIUS)),
        bolt_mesh: meshes.add(Circle::new(BOLT_RADIUS)),
        attacker_material: materials.add(StandardMaterial {
            base_color: ATTACKER_DISPELLER_COLOR,
            unlit: true,
            ..default()
        }),
        bolt_material: materials.add(StandardMaterial {
            base_color: BOLT_COLOR,
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
