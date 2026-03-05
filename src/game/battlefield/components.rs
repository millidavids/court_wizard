use bevy::prelude::*;

/// Marker component for the battlefield background.
#[derive(Component)]
pub struct Battlefield;

/// Marker component for the castle battlements.
#[derive(Component)]
pub struct Castle;

/// Pre-loaded castle wall texture for the castle top surface.
#[derive(Resource)]
pub struct CastleWallAssets {
    pub texture: Handle<Image>,
}

/// Pre-loaded right wall backdrop texture.
#[derive(Resource)]
pub struct RightWallAssets {
    pub texture: Handle<Image>,
}

/// Marker component for the right wall backdrop.
#[derive(Component)]
pub struct RightWall;
