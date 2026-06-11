use bevy::prelude::*;

use crate::game::constants::{ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR};
use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
use crate::game::units::constants::{DEFAULT_SPRITE_HEIGHT, DEFAULT_SPRITE_WIDTH};
use crate::game::units::systems::create_corpse_sprite_materials;

/// Pre-loaded meshes and materials for shielder units.
#[allow(dead_code)]
#[derive(Resource)]
pub struct ShielderAssets {
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Shielder walking sprite sheet texture.
    pub sprite_texture: Handle<Image>,
    /// Attack animation sprite sheet.
    pub attacking_texture: Handle<Image>,
    /// Casting animation sprite sheet (shield ability).
    pub casting_texture: Handle<Image>,
    /// Death animation sprite sheet.
    pub death_texture: Handle<Image>,
    pub defender_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub attacker_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub undead_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    /// Emissive yellow material used for the pre-shield channel particles.
    pub channel_particle_material: Handle<StandardMaterial>,
}

/// System to pre-load shielder assets at startup.
pub(super) fn preload_shielder_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_texture = asset_server.load("images/sprite_sheets/shielder-walking_9-frames.png");
    let attacking_texture =
        asset_server.load("images/sprite_sheets/shielder-attacking_6-frames.png");
    let casting_texture = asset_server.load("images/sprite_sheets/shielder-casting_7-frames.png");
    let death_texture = asset_server.load("images/sprite_sheets/shielder-death_6-frames.png");

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

    let assets = ShielderAssets {
        sprite_mesh: meshes.add(Rectangle::new(DEFAULT_SPRITE_WIDTH, DEFAULT_SPRITE_HEIGHT)),
        sprite_texture,
        attacking_texture,
        casting_texture,
        death_texture,
        defender_corpse_materials,
        attacker_corpse_materials,
        undead_corpse_materials,
        channel_particle_material: materials.add(StandardMaterial {
            base_color: Color::srgb(1.3, 1.1, 0.2),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(3.0, 2.4, 0.3, 1.0),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
