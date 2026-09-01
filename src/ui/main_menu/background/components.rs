use bevy::prelude::*;

#[derive(Component)]
pub(super) struct OnMenuBackground;

/// Drives horizontal parallax scrolling for a layer.
///
/// Both `speed` and `offset` are percentages of viewport width, matching the
/// percentage-based strip sizing in [`super::constants`].
#[derive(Component)]
pub(super) struct ParallaxLayer {
    /// Percent of viewport width per second.
    pub speed: f32,
    /// Strip width as a percentage of viewport width; the scroll wraps here.
    pub width_percent: f32,
    /// Current scroll offset, in percent of viewport width.
    pub offset: f32,
    /// Cached entity of the inner flex row to avoid Children traversal each frame.
    pub flex_row: Entity,
}
