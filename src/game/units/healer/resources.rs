use bevy::prelude::*;

use crate::game::constants::{
    ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_BASE, UNDEAD_CORPSE_COLOR,
};

use super::constants::*;

/// Pre-loaded meshes and materials for healer units.
#[derive(Resource)]
#[allow(dead_code)]
pub struct HealerAssets {
    pub mesh: Handle<Mesh>,
    pub bolt_mesh: Handle<Mesh>,
    pub attacker_material: Handle<StandardMaterial>,
    pub bolt_material: Handle<StandardMaterial>,
    pub defender_corpse_material: Handle<StandardMaterial>,
    pub attacker_corpse_material: Handle<StandardMaterial>,
    pub undead_corpse_material: Handle<StandardMaterial>,
    pub undead_material: Handle<StandardMaterial>,
}

/// System to pre-load healer assets at startup.
pub(super) fn preload_healer_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = HealerAssets {
        mesh: meshes.add(Circle::new(HEALER_RADIUS)),
        bolt_mesh: meshes.add(Circle::new(HEAL_BOLT_RADIUS)),
        attacker_material: materials.add(StandardMaterial {
            base_color: ATTACKER_HEALER_COLOR,
            unlit: true,
            ..default()
        }),
        bolt_material: materials.add(StandardMaterial {
            base_color: HEAL_BOLT_COLOR,
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
