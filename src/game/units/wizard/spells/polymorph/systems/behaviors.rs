use super::super::components::{DireSheep, PigForm};
use super::super::constants;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, PolymorphedModifier, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use bevy::prelude::*;

/// Pig Form: polymorphed pigs flee away from the nearest unit at high speed.
pub fn handle_pig_movement(
    pig_query: Query<(Entity, &Transform), (With<PigForm>, With<PolymorphedModifier>)>,
    units_query: Query<&Transform, (Without<Corpse>, Without<PolymorphedModifier>)>,
    mut velocity_query: Query<&mut crate::game::components::Velocity>,
) {
    for (pig_entity, pig_transform) in &pig_query {
        // Find nearest living unit to flee from
        let mut nearest_dist = f32::MAX;
        let mut flee_dir = Vec3::new(0.0, 0.0, -1.0); // Default flee direction

        for unit_transform in &units_query {
            let dist = pig_transform
                .translation
                .distance(unit_transform.translation);
            if dist < nearest_dist {
                nearest_dist = dist;
                let dir = pig_transform.translation - unit_transform.translation;
                if dir.length_squared() > 0.01 {
                    flee_dir = dir.normalize();
                }
            }
        }

        if let Ok(mut velocity) = velocity_query.get_mut(pig_entity) {
            velocity.x = flee_dir.x * constants::PIG_SPEED;
            velocity.z = flee_dir.z * constants::PIG_SPEED;
        }
    }
}

/// Dire Sheep: friendly sheep that moves toward and attacks nearby enemies.
pub fn tick_dire_sheep(
    mut commands: Commands,
    time: Res<Time>,
    mut sheep_query: Query<
        (Entity, &Transform, &mut DireSheep),
        (With<PolymorphedModifier>, Without<Corpse>),
    >,
    mut enemies_query: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            &Team,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
    mut velocity_query: Query<&mut crate::game::components::Velocity>,
) {
    let delta = time.delta_secs();

    for (sheep_entity, sheep_transform, mut dire) in &mut sheep_query {
        dire.attack_timer -= delta;

        // Find nearest enemy (attackers or undead)
        let mut nearest_enemy: Option<(Entity, f32, Vec3)> = None;
        for (entity, transform, _, team, _, _) in &enemies_query {
            if Team::Defenders.is_enemy(team) {
                let dist = sheep_transform.translation.distance(transform.translation);
                if nearest_enemy.as_ref().is_none_or(|e| dist < e.1) {
                    nearest_enemy = Some((entity, dist, transform.translation));
                }
            }
        }

        if let Some((enemy_entity, dist, enemy_pos)) = nearest_enemy {
            // Move toward nearest enemy
            let dir = (enemy_pos - sheep_transform.translation).normalize_or_zero();
            if let Ok(mut velocity) = velocity_query.get_mut(sheep_entity) {
                velocity.x = dir.x * constants::DIRE_SHEEP_MOVE_SPEED;
                velocity.z = dir.z * constants::DIRE_SHEEP_MOVE_SPEED;
            }

            // Attack if in range and timer ready
            if dist <= constants::DIRE_SHEEP_ATTACK_RADIUS && dire.attack_timer <= 0.0 {
                dire.attack_timer = constants::DIRE_SHEEP_ATTACK_INTERVAL;
                if let Ok((_, _, mut enemy_health, _, mut temp_hp, has_spell_shield)) =
                    enemies_query.get_mut(enemy_entity)
                {
                    apply_spell_damage(
                        &mut commands,
                        enemy_entity,
                        &mut enemy_health,
                        temp_hp.as_deref_mut(),
                        constants::DIRE_SHEEP_DAMAGE,
                        DamageType::Nature,
                        has_spell_shield,
                    );
                }
            }
        } else {
            // No enemies found, stop moving
            if let Ok(mut velocity) = velocity_query.get_mut(sheep_entity) {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
        }
    }
}
