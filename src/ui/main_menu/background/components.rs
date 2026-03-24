use bevy::prelude::*;

#[derive(Component)]
pub(super) struct OnMenuBackground;

/// Drives horizontal parallax scrolling for a layer.
#[derive(Component)]
pub(super) struct ParallaxLayer {
    pub speed: f32,
    pub image_width: f32,
    pub offset: f32,
    /// Cached entity of the inner flex row to avoid Children traversal each frame.
    pub flex_row: Entity,
}
