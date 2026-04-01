use bevy::prelude::*;

/// A bush obstacle on the battlefield. Passable but slows units (rough terrain).
/// Does NOT block projectiles. Flammable — catches fire from fire spells.
#[derive(Component)]
pub struct Bush {
    /// Center position in world space.
    pub center: Vec3,
    /// Collision radius on the XZ plane.
    pub radius: f32,
}

/// Marker for a bush that has caught fire.
/// Burns for the rest of the battle, dealing fire damage to units inside.
/// Burning bushes are excluded from save and disappear next level.
#[derive(Component)]
pub struct BurningBush {
    /// Tick timer for periodic fire damage.
    pub tick_timer: f32,
}
