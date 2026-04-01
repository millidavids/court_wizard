use bevy::prelude::*;

use crate::game::constants::{ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR};
use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
use crate::game::units::constants::{DEFAULT_SPRITE_HEIGHT, DEFAULT_SPRITE_WIDTH};
use crate::game::units::systems::create_corpse_sprite_materials;

use super::constants::*;

/// Pre-loaded meshes and materials for healer units.
#[derive(Resource)]
#[allow(dead_code)]
pub struct HealerAssets {
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Healer walking sprite sheet texture.
    pub sprite_texture: Handle<Image>,
    /// Attack animation sprite sheet.
    pub attacking_texture: Handle<Image>,
    /// Casting animation sprite sheet (heal ability).
    pub casting_texture: Handle<Image>,
    /// Death animation sprite sheet.
    pub death_texture: Handle<Image>,
    pub bolt_mesh: Handle<Mesh>,
    pub bolt_material: Handle<StandardMaterial>,
    pub defender_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub attacker_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub undead_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
}

/// System to pre-load healer assets at startup.
pub(super) fn preload_healer_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_texture = asset_server.load("images/sprite_sheets/healer-walking_9-frames.png");
    let attacking_texture = asset_server.load("images/sprite_sheets/healer-attacking_6-frames.png");
    let casting_texture = asset_server.load("images/sprite_sheets/healer-casting_7-frames.png");
    let death_texture = asset_server.load("images/sprite_sheets/healer-death_6-frames.png");

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

    let assets = HealerAssets {
        sprite_mesh: meshes.add(Rectangle::new(DEFAULT_SPRITE_WIDTH, DEFAULT_SPRITE_HEIGHT)),
        sprite_texture,
        attacking_texture,
        casting_texture,
        death_texture,
        bolt_mesh: meshes.add(Circle::new(HEAL_BOLT_RADIUS)),
        bolt_material: materials.add(StandardMaterial {
            base_color: HEAL_BOLT_COLOR,
            unlit: true,
            ..default()
        }),
        defender_corpse_materials,
        attacker_corpse_materials,
        undead_corpse_materials,
    };

    commands.insert_resource(assets);
}
