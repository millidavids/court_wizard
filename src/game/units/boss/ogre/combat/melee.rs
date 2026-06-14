use crate::game::units::animation::CombatAnimation;
use bevy::prelude::*;

use super::super::charge::ogre_combat_animation;
use super::super::components::*;
use super::super::constants::*;
use super::super::resources::OgreAssets;
use crate::game::constants::*;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    BanishedModifier, Corpse, Health, Hitbox, Knockback, TargetingVelocity, Team,
    TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::components::Wizard;

/// Updates ogre targeting velocity toward nearest enemy.
pub fn update_ogre_targeting(
    mut commands: Commands,
    mut bosses: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Boss>, Without<crate::game::units::boss::lich::Lich>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Boss>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
            Without<Wizard>,
        ),
    >,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, boss_transform, boss_team, mut targeting) in &mut bosses {
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            boss_transform,
            *boss_team,
            &mut targeting,
            &mut commands,
            None,
        );
    }
}

/// Ogre melee combat system — runs on its own cooldown timer (not the global attack cycle).
/// Finds nearest enemy in melee range, deals flat damage, and applies a tumbling
/// knockback effect to all nearby enemies.
#[allow(clippy::type_complexity)]
pub fn ogre_combat(
    time: Res<Time>,
    mut commands: Commands,
    ogre_assets: Res<OgreAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut bosses: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut OgreAttackCooldown,
            &OgreChargeState,
        ),
        (
            With<Boss>,
            Without<Corpse>,
            Without<CombatAnimation>,
            Without<OgreThrowWindup>,
        ),
    >,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<Boss>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<Wizard>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (boss_entity, boss_transform, boss_hitbox, boss_team, mut attack_cooldown, charge_state) in
        &mut bosses
    {
        // Skip normal melee attacks during charge
        if charge_state.is_movement_locked() {
            continue;
        }

        attack_cooldown.tick(delta);

        if !attack_cooldown.is_ready() {
            continue;
        }

        // Find nearest enemy in melee range
        let boss_pos = boss_transform.translation;
        let mut has_target = false;

        // First pass: check if any enemy is in melee range
        for (entity, target_transform, target_hitbox, team, _, _) in &targets {
            if entity == boss_entity {
                continue;
            }
            if !boss_team.is_enemy(team) {
                continue;
            }

            let dx = boss_pos.x - target_transform.translation.x;
            let dz = boss_pos.z - target_transform.translation.z;
            let distance = (dx * dx + dz * dz).sqrt();
            let attack_range =
                (boss_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
            if distance <= attack_range {
                has_target = true;
                break;
            }
        }

        if !has_target {
            continue;
        }

        // Reset cooldown — ogre attacked
        attack_cooldown.reset(OGRE_ATTACK_COOLDOWN);

        // Play swing sound effect
        crate::game::units::wizard::spells::audio::play_sfx_scaled(
            &mut commands,
            &ogre_assets.swing_sfx,
            boss_pos,
            &game_config,
            1.0,
        );

        // Trigger attack animation
        commands.entity(boss_entity).insert(ogre_combat_animation(
            OGRE_ATTACKING_DIRECTION_ROWS,
            ogre_assets.attacking_texture.clone(),
            ogre_assets.walking_texture.clone(),
        ));

        // Second pass: apply damage and knockback to all enemies within ogre melee reach
        for (entity, target_transform, target_hitbox, team, mut health, mut temp_hp) in &mut targets
        {
            if entity == boss_entity {
                continue;
            }
            let is_enemy = boss_team.is_enemy(team);
            if !is_enemy {
                continue;
            }

            let target_pos = target_transform.translation;
            let dx = target_pos.x - boss_pos.x;
            let dz = target_pos.z - boss_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();

            // Hit all enemies within the SAME range the first-pass gate used, so
            // an enemy that triggers the swing is actually damaged by it.
            let attack_range =
                (boss_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
            if distance > attack_range {
                continue;
            }

            // Apply damage
            apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), OGRE_ATTACK_DAMAGE);

            // Apply tumbling knockback (decays over time)
            let direction = if distance > 0.1 {
                Vec3::new(dx, 0.0, dz)
            } else {
                Vec3::X
            };
            commands.entity(entity).insert(Knockback::new(
                direction,
                OGRE_MELEE_KNOCKBACK_SPEED,
                OGRE_MELEE_KNOCKBACK_DURATION,
            ));
        }
    }
}
