use std::cmp::Ordering;

use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::ArcherAssets;
use crate::game::attack_cycle::GlobalAttackCycle;
use crate::game::constants::*;
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::units::components::{
    AttackTiming, BanishedModifier, Corpse, Effectiveness, Health, Hitbox, SleepModifier, Team,
    TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::components::Wizard;

use super::ranged::is_valid_target;

/// Archer melee combat system (used when enemies are in melee range).
/// Archers deal reduced damage in melee compared to infantry.
#[allow(clippy::type_complexity)]
pub fn archer_melee_combat(
    mut commands: Commands,
    attack_cycle: Res<GlobalAttackCycle>,
    archer_assets: Res<ArcherAssets>,
    mut archers: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut AttackTiming,
            &Effectiveness,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
            Has<crate::game::units::infantry::components::Retreating>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Archer>, Without<Corpse>),
    >,
    targets: Query<
        (Entity, &Transform, &Hitbox, &Team),
        (Without<Corpse>, Without<BanishedModifier>, Without<Wizard>),
    >,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<crate::game::units::shielder::components::ShielderDamageReduction>,
        Has<crate::game::units::assassin::Assassin>,
    )>,
) {
    let current_time = attack_cycle.current_time;
    // `previous_time` is the cycle position before the most recent `tick()` —
    // see `GlobalAttackCycle` doc for why this is preferred over the
    // `current_time - APPROX_FRAME_TIME` approximation.
    let last_time = attack_cycle.previous_time;

    // Collect snapshot of all targets
    let targets_snapshot: Vec<_> = targets
        .iter()
        .map(|(entity, transform, hitbox, team)| (entity, transform.translation, *hitbox, *team))
        .collect();

    for (
        archer_entity,
        archer_transform,
        archer_hitbox,
        archer_team,
        mut attack_timing,
        effectiveness,
        sleeping,
        banished,
        is_retreating,
        has_staging,
        has_wave_group,
    ) in &mut archers
    {
        // Skip staging attackers (includes 1-frame delay before WaveGroup is added)
        if crate::game::units::systems::is_staging_attacker(
            archer_team,
            has_staging,
            has_wave_group,
        ) {
            continue;
        }

        // Skip attack if sleeping, banished, or retreating
        if sleeping.is_some() || banished.is_some() || is_retreating {
            continue;
        }

        // Find nearest enemy within melee range
        if let Some((target_entity, _, _)) = targets_snapshot
            .iter()
            .filter(|(entity, _, _, team)| {
                *entity != archer_entity && is_valid_target(archer_team, team)
            })
            .filter_map(|(entity, target_pos, target_hitbox, _)| {
                // Calculate distance on XZ plane only (ignore Y axis for attack range)
                let dx = archer_transform.translation.x - target_pos.x;
                let dz = archer_transform.translation.z - target_pos.z;
                let distance = (dx * dx + dz * dz).sqrt();
                let melee_range =
                    (archer_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
                if distance <= melee_range {
                    Some((entity, target_pos, distance))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
        {
            // Attack if we're in the unit's attack window
            if attack_timing.can_attack(current_time, last_time)
                && let Ok((mut target_health, mut temp_hp, has_shielder_reduction, is_assassin)) =
                    health_query.get_mut(*target_entity)
            {
                // Apply effectiveness multiplier to melee damage
                let mut modified_damage = ARCHER_MELEE_DAMAGE * effectiveness.multiplier();
                if has_shielder_reduction {
                    modified_damage *=
                        crate::game::units::shielder::constants::SHIELDER_DAMAGE_REDUCTION;
                }
                // Assassins take 50% less damage from archers (melee)
                if is_assassin {
                    modified_damage *=
                        crate::game::units::assassin::constants::ARCHER_DAMAGE_REDUCTION;
                }
                apply_damage_to_unit(&mut target_health, temp_hp.as_deref_mut(), modified_damage);
                attack_timing.last_attack_time = Some(current_time);

                // Trigger melee attack animation
                commands.entity(archer_entity).insert(
                    crate::game::units::components::CombatAnimation::new_attack(
                        archer_assets.attacking_texture.clone(),
                        archer_assets.sprite_texture.clone(),
                    ),
                );
            }
        }
    }
}
