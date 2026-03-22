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
    /// Floor texture between the right and left walls.
    pub wall_floor: Handle<Image>,
    /// Sprite sheet of 10 ground tile variants (160x16, ten 16x16 tiles).
    pub battlefield_tiles: Handle<Image>,
}

/// Marker component for the right wall backdrop.
#[derive(Component)]
pub struct RightWall;

/// Marker component for the left (back) wall backdrop.
#[derive(Component)]
pub struct LeftWall;

/// Marker component for the floor between the right and left walls.
#[derive(Component)]
pub struct WallFloor;

/// Marker for the lava pool environmental fire effect.
#[derive(Component)]
pub struct LavaPool;

/// A growing, fading annulus ripple on the water pool surface.
#[derive(Component)]
pub struct WaterRipple {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub max_scale: f32,
}

/// Pre-allocated annulus mesh for water ripples.
#[derive(Resource)]
pub struct WaterRippleAssets {
    pub mesh: Handle<Mesh>,
}

