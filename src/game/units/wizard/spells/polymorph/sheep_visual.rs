//! Sheep sprite + idle bounce animation for polymorphed units.

use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

const SHEEP_BOUNCE_AMPLITUDE: f32 = 4.0;
const SHEEP_BOUNCE_FREQ: f32 = 6.0;

/// Side length of the sheep quad in world units. Matches the unit-sprite
/// pixel density (24×32 frames render at `UNIT_SCALE`).
pub(crate) const SHEEP_QUAD_SIZE: f32 = 16.0 * UNIT_SCALE;

/// Attached to a unit while it is polymorphed. Drives the idle hop animation.
#[derive(Component)]
pub(crate) struct SheepBounce {
    pub(crate) base_y: f32,
    pub(crate) elapsed: f32,
}

/// Advances each polymorphed unit's hop animation. One-sided bounce so the
/// sheep never sinks below its resting Y.
pub(crate) fn bounce_sheep_units(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &mut SheepBounce)>,
) {
    let delta = time.delta_secs();
    for (mut transform, mut bounce) in &mut q {
        bounce.elapsed += delta;
        let offset = SHEEP_BOUNCE_AMPLITUDE * (bounce.elapsed * SHEEP_BOUNCE_FREQ).sin().abs();
        transform.translation.y = bounce.base_y + offset;
    }
}
