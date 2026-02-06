use bevy::prelude::*;

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
            base_color: Color::srgb(0.6, 0.6, 0.4), // Grayish yellow
            unlit: true,
            ..default()
        }),
        attacker_corpse_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.4, 0.4), // Grayish red
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
