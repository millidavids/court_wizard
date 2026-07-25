use bevy::prelude::*;

use super::HEALER_CAST_DURATION;
use super::targeting_helpers::{find_best_heal_target, find_heal_priority};
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::units::brute::components::Brute;
use crate::game::units::commander::components::Commander;
use crate::game::units::components::{
    BanishedModifier, CombatAnimation, Corpse, EliteSpeedBonus, Health, SleepModifier, Team,
};
use crate::game::units::dispeller::components::Dispeller;
use crate::game::units::healer::components::{HealBolt, Healer, HealerAttackTimer};
use crate::game::units::healer::constants::{
    HEAL_BOLT_LIFETIME, HEAL_BOLT_SPEED, HEAL_COOLDOWN, HEAL_RANGE,
};
use crate::game::units::healer::resources::HealerAssets;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::vfx::channel::ChannelingCast;

/// Starts a 5-second heal channel when cooldown is ready and a valid target is in range.
/// Staging attackers (not yet activated at their rally point) cannot be healed.
#[allow(clippy::type_complexity)]
pub fn healer_start_heal_channel(
    mut commands: Commands,
    time: Res<Time>,
    healer_assets: Res<HealerAssets>,
    mut healers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut HealerAttackTimer,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
            Has<ChannelingCast>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Healer>, Without<Corpse>),
    >,
    potential_targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &Health,
            Option<&Commander>,
            Option<&Brute>,
            Option<&EliteSpeedBonus>,
            Option<&Dispeller>,
            Option<&Healer>,
        ),
        (Without<Corpse>, Without<Wizard>, Without<StagingAttacker>),
    >,
) {
    let delta = time.delta_secs();

    let ally_snapshot: Vec<(Entity, Vec3, Team, f32, f32, u32)> = potential_targets
        .iter()
        .map(
            |(entity, transform, team, health, commander, brute, elite, dispeller, healer)| {
                let priority = find_heal_priority(
                    commander.is_some(),
                    brute.is_some(),
                    elite.is_some(),
                    dispeller.is_some(),
                    healer.is_some(),
                );
                (
                    entity,
                    transform.translation,
                    *team,
                    health.current,
                    health.max,
                    priority,
                )
            },
        )
        .collect();

    for (
        healer_entity,
        healer_transform,
        healer_team,
        mut attack_timer,
        sleeping,
        banished,
        is_channeling,
        has_staging,
        has_wave_group,
    ) in &mut healers
    {
        if crate::game::units::systems::is_staging_attacker(
            healer_team,
            has_staging,
            has_wave_group,
        ) {
            continue;
        }
        attack_timer.time_since_last_attack += delta;

        if is_channeling || sleeping.is_some() || banished.is_some() {
            continue;
        }
        if attack_timer.time_since_last_attack < HEAL_COOLDOWN {
            continue;
        }

        let best_target = find_best_heal_target(
            &ally_snapshot,
            healer_entity,
            healer_transform.translation,
            *healer_team,
        );
        let Some((_, _, distance)) = best_target else {
            continue;
        };
        if distance > HEAL_RANGE {
            continue;
        }

        commands.entity(healer_entity).insert((
            ChannelingCast { elapsed: 0.0 },
            CombatAnimation::new_casting(
                healer_assets.casting_texture.clone(),
                healer_assets.sprite_texture.clone(),
            ),
        ));
    }
}

/// Ticks active heal channels. When the channel completes, spawns a heal bolt
/// at a freshly-picked valid target and starts the cooldown. Staging attackers
/// (not yet activated at their rally point) cannot be healed.
#[allow(clippy::type_complexity)]
pub fn healer_tick_heal_channel(
    mut commands: Commands,
    time: Res<Time>,
    healer_assets: Res<HealerAssets>,
    mut healers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut HealerAttackTimer,
            &mut ChannelingCast,
        ),
        (With<Healer>, Without<Corpse>),
    >,
    potential_targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &Health,
            Option<&Commander>,
            Option<&Brute>,
            Option<&EliteSpeedBonus>,
            Option<&Dispeller>,
            Option<&Healer>,
        ),
        (Without<Corpse>, Without<Wizard>, Without<StagingAttacker>),
    >,
) {
    let delta = time.delta_secs();

    let mut ally_snapshot: Option<Vec<(Entity, Vec3, Team, f32, f32, u32)>> = None;

    for (healer_entity, healer_transform, healer_team, mut attack_timer, mut channel) in
        &mut healers
    {
        channel.elapsed += delta;
        if channel.elapsed < HEALER_CAST_DURATION {
            continue;
        }

        commands.entity(healer_entity).remove::<ChannelingCast>();
        attack_timer.time_since_last_attack = 0.0;

        let snapshot = ally_snapshot.get_or_insert_with(|| {
            potential_targets
                .iter()
                .map(
                    |(
                        entity,
                        transform,
                        team,
                        health,
                        commander,
                        brute,
                        elite,
                        dispeller,
                        healer,
                    )| {
                        let priority = find_heal_priority(
                            commander.is_some(),
                            brute.is_some(),
                            elite.is_some(),
                            dispeller.is_some(),
                            healer.is_some(),
                        );
                        (
                            entity,
                            transform.translation,
                            *team,
                            health.current,
                            health.max,
                            priority,
                        )
                    },
                )
                .collect()
        });

        let best_target = find_best_heal_target(
            snapshot,
            healer_entity,
            healer_transform.translation,
            *healer_team,
        );
        let Some((target_entity, target_pos, distance)) = best_target else {
            continue;
        };
        if distance > HEAL_RANGE {
            continue;
        }

        let origin = healer_transform.translation + Vec3::Y * 10.0;
        let diff = target_pos - origin;
        let direction = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();

        commands.spawn((
            Mesh3d(healer_assets.bolt_mesh.clone()),
            MeshMaterial3d(healer_assets.bolt_material.clone()),
            Transform::from_translation(origin)
                .with_scale(Vec3::splat(1.0))
                .looking_to(direction, Vec3::Y),
            HealBolt {
                target: target_entity,
                speed: HEAL_BOLT_SPEED,
                source_team: *healer_team,
                lifetime: HEAL_BOLT_LIFETIME,
            },
            Billboard,
            OnGameplayScreen,
        ));
    }
}
