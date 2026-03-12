use bevy::prelude::*;

/// Marker component for the battlefield background.
#[derive(Component)]
pub struct Battlefield;

/// Marker component for the castle battlements.
#[derive(Component)]
pub struct Castle;

/// Pre-loaded texture assets for the battlefield.
#[derive(Resource)]
pub struct BattlefieldAssets {
    /// Castle wall texture for the castle top surface.
    pub castle_wall: Handle<Image>,
    /// Right wall backdrop texture.
    pub right_wall: Handle<Image>,
    /// Left (back) wall backdrop texture.
    pub left_wall: Handle<Image>,
}

/// Marker component for the right wall backdrop.
#[derive(Component)]
pub struct RightWall;

/// Marker component for the left (back) wall backdrop.
#[derive(Component)]
pub struct LeftWall;
