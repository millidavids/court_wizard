use bevy::prelude::*;

use crate::game::units::components::{Corpse, Hitbox, RoughTerrain, RoughTerrainModifier};

/// Applies movement slowdown to units standing on rough terrain (corpses).
///
/// Units walking over corpses have their movement speed temporarily reduced.
/// This creates a tactical element where corpses affect battlefield movement.
pub fn apply_rough_terrain_slowdown(
    mut commands: Commands,
    units: Query<
        (Entity, &Transform, &Hitbox, Option<&RoughTerrainModifier>),
        (
            Without<Corpse>,
            Without<crate::game::units::wizard::components::Wizard>,
        ),
    >,
    corpses: Query<(&Transform, &Hitbox, &RoughTerrain), With<Corpse>>,
) {
    for (entity, unit_transform, unit_hitbox, _speed_modifier) in &units {
        let mut max_slowdown: f32 = 1.0; // No slowdown by default

        // Check all corpses for overlap
        for (corpse_transform, corpse_hitbox, rough_terrain) in &corpses {
            let distance = unit_transform
                .translation
                .distance(corpse_transform.translation);
            let overlap_threshold = unit_hitbox.radius + corpse_hitbox.radius;

            if distance < overlap_threshold {
                // Apply slowdown from this corpse
                max_slowdown = max_slowdown.min(rough_terrain.slowdown_factor);
            }
        }

        // Apply the worst slowdown encountered as a RoughTerrainModifier component
        // slowdown_factor of 0.4 means 60% slower = -0.6 (negative 60%)
        if max_slowdown < 1.0 {
            let slowdown_percentage = max_slowdown - 1.0; // e.g., 0.4 - 1.0 = -0.6
            commands
                .entity(entity)
                .insert(RoughTerrainModifier(slowdown_percentage));
        } else {
            // Not on rough terrain - remove slowdown component if it exists
            commands.entity(entity).remove::<RoughTerrainModifier>();
        }
    }
}
