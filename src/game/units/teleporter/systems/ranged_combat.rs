use bevy::prelude::*;

use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::units::components::{
    BanishedModifier, CombatAnimation, Corpse, SleepModifier, Team,
};
use crate::game::units::ranged_bolt::RangedAttackTimer;
use crate::game::units::teleporter::components::{Teleporter, TeleporterState};
use crate::game::units::teleporter::constants::*;
use crate::game::units::teleporter::resources::TeleporterAssets;
use crate::game::units::wizard::components::Wizard;

#[allow(clippy::type_complexity)]
pub(crate) fn teleporter_ranged_combat(
    mut commands: Commands,
    time: Res<Time>,
    teleporter_assets: Res<TeleporterAssets>,
    mut teleporters: Query<
        (
            Entity,
            &Transform,
            &Team,
            &TeleporterState,
            &mut RangedAttackTimer,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Teleporter>, Without<Corpse>),
    >,
    targets: Query<
        (Entity, &Transform, &Team),
        (Without<Corpse>, Without<BanishedModifier>, Without<Wizard>),
    >,
) {
    let delta = time.delta_secs();

    for (
        teleporter_entity,
        teleporter_transform,
        teleporter_team,
        state,
        mut attack_timer,
        sleeping,
        banished,
        has_staging,
        has_wave_group,
    ) in &mut teleporters
    {
        if crate::game::units::systems::is_staging_attacker(
            teleporter_team,
            has_staging,
            has_wave_group,
        ) {
            continue;
        }
        attack_timer.time_since_last_attack += delta;

        if matches!(state, TeleporterState::Channeling { .. })
            || sleeping.is_some()
            || banished.is_some()
        {
            continue;
        }

        if attack_timer.time_since_last_attack < TELEPORTER_ATTACK_COOLDOWN {
            continue;
        }

        let nearest_enemy = targets
            .iter()
            .filter(|(entity, _, team)| {
                *entity != teleporter_entity && teleporter_team.is_enemy(team)
            })
            .filter(|(_, transform, _)| {
                teleporter_transform
                    .translation
                    .distance(transform.translation)
                    <= TELEPORTER_ATTACK_RANGE
            })
            .min_by(|a, b| {
                let dist_a = teleporter_transform.translation.distance(a.1.translation);
                let dist_b = teleporter_transform.translation.distance(b.1.translation);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((_, target_transform, _)) = nearest_enemy {
            let origin = teleporter_transform.translation + Vec3::Y * 10.0;
            let diff = target_transform.translation - origin;
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();

            crate::game::units::ranged_bolt::spawn_magic_bolt(
                &mut commands,
                &teleporter_assets.bolt_mesh,
                &teleporter_assets.bolt_material,
                origin,
                direction,
                TELEPORTER_BOLT_SPEED,
                TELEPORTER_BOLT_DAMAGE,
                TELEPORTER_BOLT_RADIUS,
                TELEPORTER_BOLT_LIFETIME,
                *teleporter_team,
            );

            commands
                .entity(teleporter_entity)
                .insert(CombatAnimation::new_casting(
                    teleporter_assets.casting_texture.clone(),
                    teleporter_assets.sprite_texture.clone(),
                ));

            attack_timer.time_since_last_attack = 0.0;
        }
    }
}
