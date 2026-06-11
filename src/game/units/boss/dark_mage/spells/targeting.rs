use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{BanishedModifier, Corpse, Team};

/// Returns the telegraph duration for a given spell type.
pub(crate) fn telegraph_duration(spell: DarkMageSpellType) -> f32 {
    match spell {
        DarkMageSpellType::DarkMeteor => METEOR_TELEGRAPH_DURATION,
        DarkMageSpellType::ShadowLightning => LIGHTNING_TELEGRAPH_DURATION,
        DarkMageSpellType::PlagueCloud => PLAGUE_TELEGRAPH_DURATION,
    }
}

/// Returns the base cooldown for a given spell type.
pub(crate) fn spell_cooldown(spell: DarkMageSpellType) -> f32 {
    match spell {
        DarkMageSpellType::DarkMeteor => METEOR_COOLDOWN,
        DarkMageSpellType::ShadowLightning => LIGHTNING_COOLDOWN,
        DarkMageSpellType::PlagueCloud => PLAGUE_COOLDOWN,
    }
}

/// Finds the best target position for a spell, avoiding the boss's own position.
pub(crate) fn find_spell_target(
    spell: DarkMageSpellType,
    boss_pos: Vec3,
    boss_team: &Team,
    targets: &Query<
        (Entity, &Transform, &Team),
        (
            Without<DarkMage>,
            Without<Corpse>,
            Without<Boss>,
            Without<BanishedModifier>,
        ),
    >,
) -> Option<(Vec3, Option<Vec3>)> {
    // Collect enemy positions within spell range and visible area
    let enemies: Vec<Vec3> = targets
        .iter()
        .filter(|(_, _, team)| boss_team.is_enemy(team))
        .map(|(_, transform, _)| transform.translation)
        .filter(|pos| {
            let dist = ((*pos - boss_pos) * Vec3::new(1.0, 0.0, 1.0)).length();
            dist <= MAX_SPELL_RANGE
                && pos.x >= VISIBLE_MIN_X
                && pos.x <= VISIBLE_MAX_X
                && pos.z >= VISIBLE_MIN_Z
                && pos.z <= VISIBLE_MAX_Z
        })
        .collect();

    if enemies.is_empty() {
        return None;
    }

    match spell {
        DarkMageSpellType::DarkMeteor | DarkMageSpellType::PlagueCloud => {
            // Find the densest cluster of enemies (most enemies within spell radius)
            let radius = if spell == DarkMageSpellType::DarkMeteor {
                METEOR_RADIUS
            } else {
                PLAGUE_RADIUS
            };

            let mut best_pos = None;
            let mut best_count = 0;

            for &pos in &enemies {
                // Skip if too close to boss
                let dx = pos.x - boss_pos.x;
                let dz = pos.z - boss_pos.z;
                if (dx * dx + dz * dz).sqrt() < MIN_TARGET_DISTANCE_FROM_SELF {
                    continue;
                }

                let count = enemies
                    .iter()
                    .filter(|&other| {
                        let d = (*other - pos).xz().length();
                        d <= radius
                    })
                    .count();

                if count > best_count {
                    best_count = count;
                    best_pos = Some(pos);
                }
            }

            best_pos.map(|p| (Vec3::new(p.x, INDICATOR_Y, p.z), None))
        }

        DarkMageSpellType::ShadowLightning => {
            // Find a direction that hits the most enemies in a corridor
            let mut best_pos = None;
            let mut best_dir = None;
            let mut best_count = 0;

            for &origin in &enemies {
                let dx = origin.x - boss_pos.x;
                let dz = origin.z - boss_pos.z;
                if (dx * dx + dz * dz).sqrt() < MIN_TARGET_DISTANCE_FROM_SELF {
                    continue;
                }

                // Try corridor from boss toward this enemy
                let dir = Vec3::new(dx, 0.0, dz).normalize_or_zero();
                if dir.length_squared() < 0.5 {
                    continue;
                }
                let perp = Vec3::new(-dir.z, 0.0, dir.x);
                let corridor_center = boss_pos + dir * (LIGHTNING_CORRIDOR_LENGTH / 2.0);

                let count = enemies
                    .iter()
                    .filter(|&other| {
                        let to = *other - corridor_center;
                        let along = to.dot(dir).abs();
                        let across = to.dot(perp).abs();
                        along <= LIGHTNING_CORRIDOR_LENGTH / 2.0
                            && across <= LIGHTNING_CORRIDOR_WIDTH / 2.0
                    })
                    .count();

                if count > best_count {
                    best_count = count;
                    best_pos = Some(corridor_center);
                    best_dir = Some(dir);
                }
            }

            if let (Some(pos), Some(dir)) = (best_pos, best_dir) {
                Some((Vec3::new(pos.x, INDICATOR_Y, pos.z), Some(dir)))
            } else {
                // Fallback: aim at nearest enemy
                let nearest = enemies
                    .iter()
                    .filter(|p| {
                        let d = (**p - boss_pos).xz().length();
                        d >= MIN_TARGET_DISTANCE_FROM_SELF
                    })
                    .min_by(|a, b| {
                        let da = (**a - boss_pos).length_squared();
                        let db = (**b - boss_pos).length_squared();
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    })?;
                let dir = (*nearest - boss_pos).normalize_or_zero();
                let center = boss_pos + dir * (LIGHTNING_CORRIDOR_LENGTH / 2.0);
                Some((
                    Vec3::new(center.x, INDICATOR_Y, center.z),
                    Some(Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero()),
                ))
            }
        }
    }
}
