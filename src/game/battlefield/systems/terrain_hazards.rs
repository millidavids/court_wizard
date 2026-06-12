use bevy::prelude::*;

use crate::game::battlefield::constants::*;
use crate::game::units::components::{Corpse, Health, RoughTerrainModifier};

/// Deals damage to any living unit inside the lava pool.
pub fn apply_lava_damage(
    time: Res<Time>,
    mut units: Query<
        (&Transform, &mut Health),
        (
            Without<Corpse>,
            Without<crate::game::units::components::Flying>,
            Without<crate::game::units::boss::ray::RayEye>,
            // Ghost units carry `Health` for CRDT propagation; damaging them here
            // would double-apply lava damage through the network channel.
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
) {
    // Multiplayer setup stage: units are immune to damage.
    if crate::game::units::components::is_setup_immune() {
        return;
    }
    let damage = LAVA_DAMAGE_PER_SECOND * time.delta_secs();
    let lava_xz = Vec2::new(LAVA_POOL_POSITION.x, LAVA_POOL_POSITION.z);
    let radius_sq = LAVA_DAMAGE_RADIUS * LAVA_DAMAGE_RADIUS;

    for (transform, mut health) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        if unit_xz.distance_squared(lava_xz) <= radius_sq {
            health.take_damage(damage);
        }
    }
}

/// Applies a speed slow to units inside the water pool, removes it when they leave.
pub fn apply_water_slow(
    mut units: Query<(&Transform, &mut RoughTerrainModifier), Without<Corpse>>,
) {
    let water_xz = Vec2::new(WATER_POOL_POSITION.x, WATER_POOL_POSITION.z);
    let radius_sq = WATER_POOL_RADIUS * WATER_POOL_RADIUS;

    for (transform, mut terrain_mod) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        if unit_xz.distance_squared(water_xz) <= radius_sq {
            if terrain_mod.0 != WATER_SPEED_MODIFIER {
                terrain_mod.0 = WATER_SPEED_MODIFIER;
            }
        } else if terrain_mod.0 != 0.0 {
            terrain_mod.0 = 0.0;
        }
    }
}
