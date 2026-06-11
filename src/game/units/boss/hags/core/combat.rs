use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::HagAssets;
use super::animation::hag_attack_animation;
use crate::game::constants::*;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{
    BanishedModifier, Corpse, Health, Hitbox, MindControlled, TargetingVelocity, Team,
    TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::components::Wizard;

/// Updates hag targeting velocity toward nearest enemy.
/// Blind hags skip normal targeting (handled by blind_hag_wandering).
pub fn update_hag_targeting(
    mut commands: Commands,
    mut hags: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            &HagEyeState,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<Wizard>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, hag_transform, hag_team, mut targeting, _eye_state) in &mut hags {
        // All hags use normal targeting (blind hags get noise added in hag_movement)
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            hag_transform,
            *hag_team,
            &mut targeting,
            &mut commands,
            None,
        );
    }
}

/// Hag melee combat — only hags with the invulnerability eye (or both eyes) attack.
/// Ability-only and blind hags skip combat. Consuming a corpse stops attacks.
#[allow(clippy::type_complexity)]
pub fn hag_combat(
    time: Res<Time>,
    mut commands: Commands,
    hag_assets: Res<HagAssets>,
    mut hags: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &HagIdentity,
            &HagEyeState,
            &mut HagAttackCooldown,
            Option<&MaulingState>,
            Option<&CorpseConsumeState>,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
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
            Without<Hag>,
            Without<Corpse>,
            Without<MindControlled>,
            Without<Wizard>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        hag_entity,
        hag_transform,
        hag_hitbox,
        hag_team,
        identity,
        eye_state,
        mut cooldown,
        mauling,
        consuming,
    ) in &mut hags
    {
        // Only hags with invulnerability eye do basic attacks
        if !eye_state.has_invulnerability_eye {
            continue;
        }

        // Consuming a corpse stops attacking
        if consuming.is_some() {
            continue;
        }

        cooldown.tick(delta);
        if !cooldown.is_ready() {
            continue;
        }

        let hag_pos = hag_transform.translation;
        let mut has_target = false;

        // First pass: check for any enemy in melee range
        for (entity, target_transform, target_hitbox, team, _, _) in &targets {
            if entity == hag_entity {
                continue;
            }
            if !hag_team.is_enemy(team) {
                continue;
            }

            let dx = hag_pos.x - target_transform.translation.x;
            let dz = hag_pos.z - target_transform.translation.z;
            let distance = (dx * dx + dz * dz).sqrt();
            let attack_range = (hag_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
            if distance <= attack_range {
                has_target = true;
                break;
            }
        }

        if !has_target {
            continue;
        }

        // Josephina's frenzy: 5x attack speed
        let attack_cd = if mauling.is_some_and(|m| m.is_frenzied()) {
            HAG_ATTACK_COOLDOWN / 5.0
        } else {
            HAG_ATTACK_COOLDOWN
        };
        cooldown.reset(attack_cd);

        // Hit nearest single enemy in melee range
        let mut nearest_target: Option<(Entity, f32)> = None;
        for (entity, target_transform, target_hitbox, team, _, _) in &targets {
            if entity == hag_entity {
                continue;
            }
            if !hag_team.is_enemy(team) {
                continue;
            }

            let target_pos = target_transform.translation;
            let dx = target_pos.x - hag_pos.x;
            let dz = target_pos.z - hag_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();

            if distance > (hag_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER {
                continue;
            }

            if let Some((_, best_dist)) = nearest_target {
                if distance < best_dist {
                    nearest_target = Some((entity, distance));
                }
            } else {
                nearest_target = Some((entity, distance));
            }
        }

        if let Some((target_entity, _)) = nearest_target
            && let Ok((_, _, _, _, mut health, mut temp_hp)) = targets.get_mut(target_entity)
        {
            apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), HAG_ATTACK_DAMAGE);

            // Josephina plays the melee swing animation on each landed attack.
            if *identity == HagIdentity::Josephina {
                commands
                    .entity(hag_entity)
                    .insert(hag_attack_animation(&hag_assets));
            }
        }
    }
}
