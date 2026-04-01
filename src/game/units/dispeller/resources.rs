use bevy::prelude::*;

use crate::game::constants::{ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR};
use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
use crate::game::units::systems::create_corpse_sprite_materials;

use super::constants::*;

/// Pre-loaded meshes and materials for dispeller units.
#[derive(Resource)]
#[allow(dead_code)]
pub struct DispellerAssets {
    pub bolt_mesh: Handle<Mesh>,
    pub bolt_material: Handle<StandardMaterial>,
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Dispeller walking sprite sheet texture.
    pub sprite_texture: Handle<Image>,
    /// Attack animation sprite sheet.
    pub attacking_texture: Handle<Image>,
    /// Casting animation sprite sheet (dispel ability).
    pub casting_texture: Handle<Image>,
    /// Death animation sprite sheet.
    pub death_texture: Handle<Image>,
    pub defender_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub attacker_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub undead_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
}

/// System to pre-load dispeller assets at startup.
pub(super) fn preload_dispeller_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_texture = asset_server.load("images/sprite_sheets/dispeller-walking_9-frames.png");
    let attacking_texture =
        asset_server.load("images/sprite_sheets/dispeller-attacking_6-frames.png");
    let casting_texture = asset_server.load("images/sprite_sheets/dispeller-casting_7-frames.png");
    let death_texture = asset_server.load("images/sprite_sheets/dispeller-death_6-frames.png");

    let defender_corpse_materials = create_corpse_sprite_materials(
        &mut materials,
        sprite_texture.clone(),
        DEFENDER_CORPSE_COLOR,
    );
    let attacker_corpse_materials = create_corpse_sprite_materials(
        &mut materials,
        sprite_texture.clone(),
        ATTACKER_CORPSE_COLOR,
    );
    let undead_corpse_materials =
        create_corpse_sprite_materials(&mut materials, sprite_texture.clone(), UNDEAD_CORPSE_COLOR);

    let assets = DispellerAssets {
        bolt_mesh: meshes.add(Circle::new(BOLT_RADIUS)),
        bolt_material: materials.add(StandardMaterial {
            base_color: BOLT_COLOR,
            unlit: true,
            ..default()
        }),
        sprite_mesh: meshes.add(Rectangle::new(
            DISPELLER_SPRITE_WIDTH,
            DISPELLER_SPRITE_HEIGHT,
        )),
        sprite_texture,
        attacking_texture,
        casting_texture,
        death_texture,
        defender_corpse_materials,
        attacker_corpse_materials,
        undead_corpse_materials,
    };

    commands.insert_resource(assets);
}
