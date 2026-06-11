use super::super::components::{
    SpikeGrowthLingeringPoison, SpikeGrowthZone, SpikeStormProjectile, ZonePresenceTracker,
};
use super::super::constants;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, RootedModifier, SlowMovementModifier, Team, TemporaryHitPoints,
    apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::utils::{UniqueHitTracker, local_player_team, xz_distance};
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

/// Applies periodic damage and slow to ALL units within the spike growth zone.
/// Handles Thorn Maze (enhanced slow), Quicksand (root after threshold),
/// and Poisoned Spikes (zone presence tracking + lingering on exit).
/// Also tracks Death Garden kill extensions.
pub fn apply_spike_growth_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<(Entity, &mut SpikeGrowthZone, &mut UniqueHitTracker)>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Option<&mut SlowMovementModifier>,
        Has<SpellShield>,
        Option<&mut ZonePresenceTracker>,
        &Team,
    )>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let delta = time.delta_secs();
    let caster_team = local_player_team(session.as_deref());

    for (zone_entity, mut zone, mut hit_tracker) in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;

        if zone.time_since_last_tick < zone.tick_interval {
            continue;
        }
        zone.time_since_last_tick = 0.0;

        let effective_radius = zone.effective_radius();
        let needs_tracking = zone.talent_params.quicksand || zone.talent_params.poisoned_spikes;

        let slow_mod = if zone.talent_params.thorn_maze {
            constants::THORN_MAZE_SLOW_MODIFIER
        } else {
            zone.slow_modifier
        };

        let mut units_hit: u32 = 0;

        for (
            entity,
            transform,
            mut health,
            mut temp_hp,
            existing_slow,
            has_spell_shield,
            zone_tracker,
            team,
        ) in &mut targets
        {
            let distance = xz_distance(zone.origin, transform.translation);

            if distance <= effective_radius {
                let was_alive = !health.is_dead();

                apply_spell_damage_with_team(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    zone.damage_per_tick,
                    DamageType::Poison,
                    has_spell_shield,
                    caster_team,
                    *team,
                );

                // The damage above is team-aware (your own zone hurts your own
                // King), but the crowd-control side-effects below must NOT apply
                // to a shielded King — slowing/rooting your own King could stall
                // it (it is deliberately unteleportable). Any shielded unit skips
                // the slow / zone-tracking / Quicksand root.
                if has_spell_shield {
                    continue;
                }

                // Track unique units hit for talent progress
                if hit_tracker.track_hit(entity) {
                    units_hit += 1;
                }

                // Death Garden: extend duration on kills
                if zone.talent_params.death_garden
                    && was_alive
                    && health.is_dead()
                    && zone.death_garden_extension < constants::DEATH_GARDEN_MAX_EXTENSION
                {
                    zone.death_garden_extension = (zone.death_garden_extension
                        + constants::DEATH_GARDEN_KILL_EXTENSION)
                        .min(constants::DEATH_GARDEN_MAX_EXTENSION);
                }

                // Apply or refresh slow
                if let Some(mut slow) = existing_slow {
                    slow.apply(slow_mod, zone.slow_duration);
                } else {
                    commands
                        .entity(entity)
                        .insert(SlowMovementModifier::new(slow_mod, zone.slow_duration));
                }

                // Zone presence tracking (for Quicksand and/or Poisoned Spikes)
                if needs_tracking {
                    if let Some(mut tracker) = zone_tracker {
                        if tracker.zone_entity == zone_entity {
                            tracker.time_in_zone += zone.tick_interval;
                            // Quicksand: root after threshold
                            if zone.talent_params.quicksand
                                && !tracker.rooted
                                && tracker.time_in_zone >= constants::QUICKSAND_TIME_THRESHOLD
                            {
                                tracker.rooted = true;
                                commands.entity(entity).insert(RootedModifier::new(
                                    constants::QUICKSAND_ROOT_DURATION,
                                ));
                            }
                        }
                    } else {
                        commands.entity(entity).insert(ZonePresenceTracker {
                            zone_entity,
                            time_in_zone: 0.0,
                            rooted: false,
                        });
                    }
                }
            } else if needs_tracking
                && let Some(ref tracker) = zone_tracker
                && tracker.zone_entity == zone_entity
            {
                // Unit is outside the zone — handle exit
                commands.entity(entity).remove::<ZonePresenceTracker>();
                // Poisoned Spikes: apply lingering poison on exit
                if zone.talent_params.poisoned_spikes {
                    commands
                        .entity(entity)
                        .insert(SpikeGrowthLingeringPoison::new());
                }
            }
        }

        // Track talent progress
        if units_hit > 0
            && let Some(ref mut progress) = talent_progress
        {
            progress.increment(Spell::SpikeGrowth, units_hit);
        }
    }
}

/// Ticks lingering poison effect and applies damage.
pub fn tick_lingering_poison(
    mut commands: Commands,
    time: Res<Time>,
    mut targets: Query<(
        Entity,
        &mut SpikeGrowthLingeringPoison,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    session: Option<Res<MultiplayerSession>>,
) {
    let delta = time.delta_secs();
    let caster_team = local_player_team(session.as_deref());

    for (entity, mut poison, mut health, mut temp_hp, has_spell_shield, team) in &mut targets {
        poison.time_remaining -= delta;
        poison.time_since_last_tick += delta;

        if poison.time_remaining <= 0.0 {
            commands
                .entity(entity)
                .remove::<SpikeGrowthLingeringPoison>();
            continue;
        }

        if poison.time_since_last_tick >= poison.tick_interval {
            poison.time_since_last_tick = 0.0;
            apply_spell_damage_with_team(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                poison.damage_per_tick,
                DamageType::Poison,
                has_spell_shield,
                caster_team,
                *team,
            );
        }
    }
}

/// Updates spike storm projectile positions and checks for collisions.
#[allow(clippy::too_many_arguments)]
pub fn update_spike_storm_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Transform, &mut SpikeStormProjectile)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        Without<SpikeStormProjectile>,
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    let delta = time.delta_secs();
    let caster_team = local_player_team(session.as_deref());

    for (proj_entity, mut proj_transform, mut projectile) in &mut projectiles {
        projectile.time_alive += delta;

        if projectile.time_alive >= projectile.max_lifetime {
            commands.entity(proj_entity).try_despawn();
            continue;
        }

        proj_transform.translation += projectile.direction * projectile.speed * delta;

        let mut hit = false;
        for (target_entity, target_transform, mut health, mut temp_hp, has_spell_shield, team) in
            &mut targets
        {
            if health.is_dead() {
                continue;
            }
            let dist = xz_distance(proj_transform.translation, target_transform.translation);

            if dist <= projectile.radius + constants::UNIT_COLLISION_RADIUS {
                apply_spell_damage_with_team(
                    &mut commands,
                    target_entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    projectile.damage,
                    DamageType::Poison,
                    has_spell_shield,
                    caster_team,
                    *team,
                );
                hit = true;
                break;
            }
        }

        if hit {
            commands.entity(proj_entity).try_despawn();
        }
    }
}
