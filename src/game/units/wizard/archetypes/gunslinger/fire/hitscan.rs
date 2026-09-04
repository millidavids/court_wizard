use bevy::prelude::*;

use super::super::components::*;
use super::super::constants;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{Corpse, Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;

/// Process all hitscan rays — find closest enemy along each ray within cylinder radius,
/// apply damage, then despawn the ray. Staging attackers (not yet activated at their
/// rally point) are excluded.
pub fn check_hitscan_collisions(
    mut commands: Commands,
    rays: Query<(Entity, &HitscanRay)>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &crate::game::units::components::Team,
            Has<SpellShield>,
        ),
        (Without<Corpse>, Without<StagingAttacker>),
    >,
) {
    for (ray_entity, ray) in &rays {
        let mut closest_hit: Option<(Entity, f32)> = None;

        for (enemy_entity, enemy_transform, _health, _temp_hp, _team, has_spell_shield) in &enemies
        {
            if has_spell_shield {
                continue;
            }

            // Point-to-line-segment distance test (cylinder collision)
            let to_enemy = enemy_transform.translation - ray.origin;
            let t = to_enemy.dot(ray.direction).clamp(0.0, ray.max_range);
            let closest_point = ray.origin + ray.direction * t;
            let distance = closest_point.distance(enemy_transform.translation);

            if distance < ray.cylinder_radius {
                // Track closest hit along the ray
                if closest_hit.is_none_or(|(_, prev_t)| t < prev_t) {
                    closest_hit = Some((enemy_entity, t));
                }
            }
        }

        // Apply damage to the closest hit
        if let Some((hit_entity, _)) = closest_hit
            && let Ok((_entity, _transform, mut health, mut temp_hp, _team, _shield)) =
                enemies.get_mut(hit_entity)
        {
            apply_spell_damage(
                &mut commands,
                hit_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                ray.damage,
                DamageType::Force,
                false,
            );
            // No direct HitFlash insert: `apply_spell_damage` above already
            // banks a `PendingSpellHit`, and routing through it means bullet
            // hits obey the same per-unit cooldown and per-frame flash cap as
            // everything else. Inserting here instead would strobe the target
            // at the machine gun's 0.08s fire interval and let a 30-pellet
            // shotgun blast spawn 30 uncapped overlays in one frame.
        }

        // Always despawn the ray after processing
        commands.entity(ray_entity).try_despawn();
    }
}

// ===== Helper functions =====

pub(crate) fn spawn_hitscan_ray(
    commands: &mut Commands,
    origin: Vec3,
    direction: Vec3,
    max_range: f32,
    damage: f32,
) {
    commands.spawn((
        HitscanRay {
            origin,
            direction,
            max_range,
            cylinder_radius: constants::HITSCAN_CYLINDER_RADIUS,
            damage,
        },
        crate::game::components::OnGameplayScreen,
    ));
}
