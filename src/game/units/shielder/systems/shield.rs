use bevy::prelude::*;

use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::units::components::{BanishedModifier, CombatAnimation, Corpse, Team};
use crate::game::units::king::components::SpellShield;
use crate::game::units::shielder::components::{
    Shielder, ShielderDamageReduction, ShielderShieldCooldown,
};
use crate::game::units::shielder::constants::{SHIELD_COOLDOWN, SHIELDER_CAST_DURATION};
use crate::game::units::shielder::systems::targeting::find_shielder_target;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::vfx::channel::ChannelingCast;

/// Starts a 5-second shield channel when cooldown is ready and a target is in range.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn shielder_start_shield_channel(
    mut commands: Commands,
    time: Res<Time>,
    shielder_assets: Res<super::super::resources::ShielderAssets>,
    mut shielders: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut ShielderShieldCooldown>,
            Option<&crate::game::units::components::SleepModifier>,
            Option<&BanishedModifier>,
            Has<ChannelingCast>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Shielder>, Without<Corpse>),
    >,
    potential_targets: Query<
        (Entity, &Transform, &Team, Has<SpellShield>),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<Shielder>,
            Without<Wizard>,
        ),
    >,
) {
    let delta = time.delta_secs();

    let ally_snapshot: Vec<(Entity, Vec3, Team, bool)> = potential_targets
        .iter()
        .map(|(entity, transform, team, has_shield)| {
            (entity, transform.translation, *team, has_shield)
        })
        .collect();

    for (
        entity,
        transform,
        team,
        cooldown,
        sleeping,
        banished,
        is_channeling,
        has_staging,
        has_wave_group,
    ) in &mut shielders
    {
        if crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group) {
            continue;
        }
        if let Some(mut cd) = cooldown {
            cd.remaining -= delta;
            if cd.remaining > 0.0 {
                continue;
            }
            commands.entity(entity).remove::<ShielderShieldCooldown>();
        }

        if is_channeling || sleeping.is_some() || banished.is_some() {
            continue;
        }

        if find_shielder_target(&ally_snapshot, transform.translation, *team).is_none() {
            continue;
        }

        commands.entity(entity).insert((
            ChannelingCast { elapsed: 0.0 },
            CombatAnimation::new_casting(
                shielder_assets.casting_texture.clone(),
                shielder_assets.sprite_texture.clone(),
            ),
        ));
    }
}

/// Ticks active shield channels. On completion applies the spell shield to a
/// freshly-picked target and starts the cooldown.
#[allow(clippy::type_complexity)]
pub fn shielder_tick_shield_channel(
    mut commands: Commands,
    time: Res<Time>,
    mut shielders: Query<
        (Entity, &Transform, &Team, &mut ChannelingCast),
        (With<Shielder>, Without<Corpse>),
    >,
    potential_targets: Query<
        (Entity, &Transform, &Team, Has<SpellShield>),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<Shielder>,
            Without<Wizard>,
        ),
    >,
) {
    let delta = time.delta_secs();

    let mut ally_snapshot: Option<Vec<(Entity, Vec3, Team, bool)>> = None;

    for (entity, transform, team, mut channel) in &mut shielders {
        channel.elapsed += delta;
        if channel.elapsed < SHIELDER_CAST_DURATION {
            continue;
        }

        commands
            .entity(entity)
            .remove::<ChannelingCast>()
            .insert(ShielderShieldCooldown {
                remaining: SHIELD_COOLDOWN,
            });

        let snapshot = ally_snapshot.get_or_insert_with(|| {
            potential_targets
                .iter()
                .map(|(entity, transform, team, has_shield)| {
                    (entity, transform.translation, *team, has_shield)
                })
                .collect()
        });

        if let Some(target_entity) = find_shielder_target(snapshot, transform.translation, *team) {
            commands
                .entity(target_entity)
                .insert((SpellShield, ShielderDamageReduction));
        }
    }
}
