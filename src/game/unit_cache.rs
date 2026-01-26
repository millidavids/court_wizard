use bevy::prelude::*;

use super::units::components::{Hitbox, Team};

/// Cached snapshot of all unit data for the current frame.
///
/// This resource is populated once per frame and used by multiple systems
/// to avoid redundant queries and iterations.
#[derive(Resource, Default)]
pub struct UnitCache {
    /// All units with their essential data
    pub units: Vec<UnitSnapshot>,
    /// Pre-computed nearest enemy for each unit (index matches units vec)
    pub nearest_enemies: Vec<Option<NearestEnemy>>,
}

/// Snapshot of a single unit's data for this frame
#[derive(Clone, Copy)]
pub struct UnitSnapshot {
    pub entity: Entity,
    pub position: Vec3,
    pub team: Team,
    pub hitbox: Hitbox,
    pub velocity: Vec3,
}

/// Information about the nearest enemy to a unit
#[derive(Clone, Copy)]
pub struct NearestEnemy {
    pub entity: Entity,
    pub position: Vec3,
    pub distance: f32,
    pub hitbox_radius: f32,
}

impl UnitCache {
    /// Find the index of a unit in the cache by entity
    pub fn find_unit_index(&self, entity: Entity) -> Option<usize> {
        self.units.iter().position(|u| u.entity == entity)
    }

    /// Get nearest enemy info for a unit
    pub fn nearest_enemy(&self, entity: Entity) -> Option<NearestEnemy> {
        let idx = self.find_unit_index(entity)?;
        self.nearest_enemies.get(idx).copied().flatten()
    }
}

/// System to populate the unit cache at the start of each frame.
///
/// This runs FIRST in the movement chain to provide data for all other systems.
pub fn populate_unit_cache(
    mut cache: ResMut<UnitCache>,
    units: Query<(
        Entity,
        &Transform,
        &Team,
        &Hitbox,
        &super::components::Velocity,
    )>,
    corpses: Query<Entity, With<super::units::components::Corpse>>,
) {
    cache.units.clear();
    cache.nearest_enemies.clear();

    // Collect all living units (exclude corpses)
    for (entity, transform, team, hitbox, velocity) in &units {
        if corpses.contains(entity) {
            continue;
        }

        cache.units.push(UnitSnapshot {
            entity,
            position: transform.translation,
            team: *team,
            hitbox: *hitbox,
            velocity: Vec3::new(velocity.x, 0.0, velocity.z),
        });
    }

    // Pre-compute nearest enemy for each unit
    for i in 0..cache.units.len() {
        let unit = cache.units[i];
        let mut nearest: Option<NearestEnemy> = None;
        let mut min_dist_sq = f32::MAX;

        for other in &cache.units {
            if other.entity == unit.entity {
                continue;
            }

            // Check if enemy based on team
            let is_enemy = match (unit.team, other.team) {
                (Team::Undead, Team::Undead) => false, // Undead don't attack each other
                (Team::Undead, _) => true,             // Undead attack living
                (_, Team::Undead) => true,             // Living attack undead
                _ => unit.team != other.team,          // Normal team logic
            };

            if !is_enemy {
                continue;
            }

            // Calculate XZ distance squared (avoid sqrt for comparison)
            let dx = unit.position.x - other.position.x;
            let dz = unit.position.z - other.position.z;
            let dist_sq = dx * dx + dz * dz;

            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                nearest = Some(NearestEnemy {
                    entity: other.entity,
                    position: other.position,
                    distance: dist_sq.sqrt(),
                    hitbox_radius: other.hitbox.radius,
                });
            }
        }

        cache.nearest_enemies.push(nearest);
    }
}
