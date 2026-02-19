use bevy::prelude::*;

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
            base_color: Color::srgb(0.5, 0.5, 0.5), // Darker gray (defender corpse)
            unlit: true,
            ..default()
        }),
        attacker_corpse_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.25, 0.25), // Very dark gray (attacker corpse)
            unlit: true,
            ..default()
        }),
        undead_corpse_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.5, 0.4), // Grayish green
            unlit: true,
            ..default()
        }),
        undead_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.8, 0.4), // Bright green (living undead)
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
