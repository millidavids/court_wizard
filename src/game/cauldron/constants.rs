use bevy::prelude::*;

use crate::game::constants::WIZARD_POSITION;

/// Cauldron color (charcoal).
pub const CAULDRON_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);

/// Visual radius of the cauldron circle.
pub const CAULDRON_RADIUS: f32 = 20.0;

/// Offset from wizard position to place the cauldron beside the wizard on the castle wall.
const CAULDRON_OFFSET: Vec3 = Vec3::new(-60.0, 0.0, -60.0);

/// Cauldron position in 3D space (on castle platform, next to wizard).
pub const CAULDRON_POSITION: Vec3 = Vec3::new(
    WIZARD_POSITION.x + CAULDRON_OFFSET.x,
    WIZARD_POSITION.y + CAULDRON_OFFSET.y,
    WIZARD_POSITION.z + CAULDRON_OFFSET.z,
);
