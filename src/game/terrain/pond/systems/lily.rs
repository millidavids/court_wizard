use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::components::{Corpse, Health, RoughTerrainModifier};
use crate::game::units::wizard::archetypes::meteorologist::components::{
    WET_DURATION, WetModifier,
};

/// Helper: checks if a position is inside any non-frozen pond.
fn is_in_any_liquid_pond(unit_xz: Vec2, ponds: &Query<(&Pond, Has<PondFrozen>)>) -> bool {
    ponds.iter().any(|(pond, frozen)| {
        if frozen {
            return false;
        }
        let pond_xz = Vec2::new(pond.center.x, pond.center.z);
        unit_xz.distance_squared(pond_xz) <= pond.radius * pond.radius
    })
}

/// Single-pass system that applies Wet to units in liquid (non-frozen) ponds and refreshes their timer.
/// Units not in a pond are left alone (timer ticks down via `tick_wet_timer`).
pub fn apply_pond_wet(
    mut commands: Commands,
    ponds: Query<(&Pond, Has<PondFrozen>)>,
    mut units: Query<
        (Entity, &Transform, Option<&mut WetModifier>),
        (With<Health>, Without<Corpse>),
    >,
) {
    for (entity, transform, wet) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        if !is_in_any_liquid_pond(unit_xz, &ponds) {
            continue;
        }
        if let Some(mut wet) = wet {
            wet.time_remaining = WET_DURATION;
        } else {
            commands.entity(entity).insert(WetModifier {
                intensity: 1.0,
                time_remaining: WET_DURATION,
            });
        }
    }
}

/// Ticks the wet timer on all wet units and removes expired WetModifier.
pub fn tick_wet_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut wet_units: Query<(Entity, &mut WetModifier)>,
) {
    let delta = time.delta_secs();
    for (entity, mut wet) in &mut wet_units {
        wet.time_remaining -= delta;
        if wet.time_remaining <= 0.0 {
            commands.entity(entity).remove::<WetModifier>();
        }
    }
}

/// Applies a stronger movement slow to units inside frozen ponds.
///
/// Inserts `RoughTerrainModifier(FROZEN_POND_SPEED_MODIFIER)` when a unit overlaps a pond
/// whose `freeze_level` is above the pathfinding threshold. Overrides weaker existing
/// modifiers only (won't override a stronger slow from a different source).
pub fn apply_frozen_pond_slow(
    mut commands: Commands,
    ponds: Query<(&Pond, &PondFrozen)>,
    mut units: Query<
        (Entity, &Transform, Option<&mut RoughTerrainModifier>),
        (With<Health>, Without<Corpse>),
    >,
) {
    for (entity, transform, terrain_mod) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        let on_frozen_pond = ponds.iter().any(|(pond, frozen)| {
            if frozen.freeze_level < POND_FREEZE_PATHFINDING_THRESHOLD {
                return false;
            }
            let pond_xz = Vec2::new(pond.center.x, pond.center.z);
            unit_xz.distance_squared(pond_xz) <= pond.radius * pond.radius
        });
        if !on_frozen_pond {
            continue;
        }
        if let Some(mut tm) = terrain_mod {
            if tm.0 > FROZEN_POND_SPEED_MODIFIER {
                tm.0 = FROZEN_POND_SPEED_MODIFIER;
            }
        } else {
            commands
                .entity(entity)
                .insert(RoughTerrainModifier(FROZEN_POND_SPEED_MODIFIER));
        }
    }
}
