use bevy::prelude::*;

use super::constants::*;
use crate::game::units::systems::create_sprite_material;

/// Pre-loaded meshes and materials for the Dark Mage boss.
#[derive(Resource)]
pub struct DarkMageAssets {
    /// Body sprite quad (Rectangle mesh sized for the 4-frame sheet).
    pub mesh: Handle<Mesh>,
    /// Floating animation sprite material (used outside of cast wind-up).
    pub floating_material: Handle<StandardMaterial>,
    /// Casting animation sprite material (used during Telegraphing/Casting).
    pub casting_material: Handle<StandardMaterial>,
    /// Circle indicator mesh (unit circle, scaled per-spell).
    pub circle_mesh: Handle<Mesh>,
    /// Rectangle mesh for lightning corridor indicator (1x1, scaled).
    pub rect_mesh: Handle<Mesh>,
    /// Plague cloud zone material.
    pub plague_zone_material: Handle<StandardMaterial>,
    /// Lightning strike material.
    pub lightning_strike_material: Handle<StandardMaterial>,
}

/// System to pre-load Dark Mage assets at startup.
pub(super) fn preload_dark_mage_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floating_texture =
        asset_server.load("images/sprite_sheets/dark-mage-floating_4-frames.png");
    let casting_texture = asset_server.load("images/sprite_sheets/dark-mage-casting_4-frames.png");

    let body_mesh = meshes.add(Rectangle::new(
        DARK_MAGE_SPRITE_WIDTH,
        DARK_MAGE_SPRITE_HEIGHT,
    ));

    let floating_material = create_sprite_material(
        &mut materials,
        floating_texture,
        Color::WHITE,
        DARK_MAGE_FRAME_UV,
        Vec2::ZERO,
    );
    let casting_material = create_sprite_material(
        &mut materials,
        casting_texture,
        Color::WHITE,
        DARK_MAGE_FRAME_UV,
        Vec2::ZERO,
    );

    let assets = DarkMageAssets {
        mesh: body_mesh,
        floating_material,
        casting_material,
        circle_mesh: meshes.add(Circle::new(1.0)),
        rect_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
        plague_zone_material: materials.add(StandardMaterial {
            base_color: PLAGUE_ZONE_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        lightning_strike_material: materials.add(StandardMaterial {
            base_color: LIGHTNING_FILL_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
