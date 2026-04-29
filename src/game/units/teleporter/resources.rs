use bevy::prelude::*;

use crate::game::constants::ATTACKER_CORPSE_COLOR;
use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
use crate::game::units::constants::{DEFAULT_SPRITE_HEIGHT, DEFAULT_SPRITE_WIDTH};
use crate::game::units::systems::create_corpse_sprite_materials;

use super::constants::{TELEPORTER_BOLT_COLOR, TELEPORTER_BOLT_RADIUS};

#[derive(Resource)]
#[allow(dead_code)]
pub struct TeleporterAssets {
    pub sprite_mesh: Handle<Mesh>,
    pub sprite_texture: Handle<Image>,
    pub casting_texture: Handle<Image>,
    pub death_texture: Handle<Image>,
    pub attacker_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub channel_particle_material: Handle<StandardMaterial>,
    pub bolt_mesh: Handle<Mesh>,
    pub bolt_material: Handle<StandardMaterial>,
}

pub(super) fn preload_teleporter_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_texture = asset_server.load("images/sprite_sheets/teleporter-walking_9-frames.png");
    let casting_texture = asset_server.load("images/sprite_sheets/teleporter-casting_7-frames.png");
    let death_texture = asset_server.load("images/sprite_sheets/teleporter-death_6-frames.png");

    let attacker_corpse_materials = create_corpse_sprite_materials(
        &mut materials,
        sprite_texture.clone(),
        ATTACKER_CORPSE_COLOR,
    );

    let channel_particle_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.7, 1.4),
        unlit: true,
        emissive: bevy::color::LinearRgba::new(0.6, 1.6, 3.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let bolt_mesh = meshes.add(Circle::new(TELEPORTER_BOLT_RADIUS));
    let bolt_material = materials.add(StandardMaterial {
        base_color: TELEPORTER_BOLT_COLOR,
        unlit: true,
        ..default()
    });

    commands.insert_resource(TeleporterAssets {
        sprite_mesh: meshes.add(Rectangle::new(DEFAULT_SPRITE_WIDTH, DEFAULT_SPRITE_HEIGHT)),
        sprite_texture,
        casting_texture,
        death_texture,
        attacker_corpse_materials,
        channel_particle_material,
        bolt_mesh,
        bolt_material,
    });
}
