use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Velocity};
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::wizard::spells::vfx::channel::{
    ChannelingCast, ChannelParticleSpec, spawn_channel_particle_batch,
};
use crate::game::units::components::{
    BanishedModifier, CombatAnimation, CommanderAuraSpeedModifier, Corpse, EliteSpeedBonus,
    FlockingVelocity, FrozenSolidModifier, HasteModifier, Hitbox, MovementSpeed,
    PolymorphedModifier, RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier,
    Sleepwalking, SlowMovementModifier, TargetingVelocity, Team,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Updates shielder targeting — seeks nearest same-team ally without a spell shield,
/// or falls back to following the army toward nearest enemy.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_shielder_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut shielders: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Shielder>, Without<Corpse>),
    >,
    potential_targets: Query<
        (Entity, &Transform, &Team, Has<SpellShield>),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<Shielder>,
        ),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    // Snapshot ally data for shield targeting
    let ally_snapshot: Vec<(Entity, Vec3, Team, bool)> = potential_targets
        .iter()
        .map(|(entity, transform, team, has_shield)| {
            (entity, transform.translation, *team, has_shield)
        })
        .collect();

    // Collect unit snapshot for enemy targeting fallback
    let unit_snapshot: Vec<(Entity, Vec3, Team)> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting_velocity) in &mut shielders {
        // Skip inactive defender shielders
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Priority 1: Find nearest same-team ally without a spell shield
        let best_target = ally_snapshot
            .iter()
            .filter(|(ally_entity, _, ally_team, has_shield)| {
                *ally_entity != entity && *ally_team == *team && !has_shield
            })
            .min_by(|a, b| {
                let dist_a = (transform.translation.x - a.1.x).powi(2)
                    + (transform.translation.z - a.1.z).powi(2);
                let dist_b = (transform.translation.x - b.1.x).powi(2)
                    + (transform.translation.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(&(_, ally_pos, _, _)) = best_target {
            let diff = ally_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            if distance <= SHIELD_RANGE {
                // In shield range — stop moving
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                // Move toward unshielded ally
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }

            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Priority 2: Fall back to following army toward nearest enemy
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
            })
            .min_by(|a, b| {
                let dist_a = (transform.translation.x - a.1.x).powi(2)
                    + (transform.translation.z - a.1.z).powi(2);
                let dist_b = (transform.translation.x - b.1.x).powi(2)
                    + (transform.translation.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(&(_, target_pos, _)) = nearest_enemy {
            let diff = target_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            // Stay at shield range from enemies
            if distance <= SHIELD_RANGE {
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
        }

        commands
            .entity(entity)
            .remove::<crate::game::units::components::InMelee>();
    }
}

/// Shielder movement system using shared weighted movement.
#[allow(clippy::type_complexity)]
pub fn shielder_movement(
    time: Res<Time>,
    mut shielder_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&crate::game::units::components::InMelee>,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            Option<&SlowMovementModifier>,
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                &Team,
                Has<StagingAttacker>,
                Has<WaveGroup>,
            ),
        ),
        With<Shielder>,
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (
            sleeping,
            sleepwalking,
            banished,
            polymorphed,
            sickened,
            frozen,
            stunned,
            team,
            has_staging,
            has_wave_group,
        ),
    ) in &mut shielder_units
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
        ) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // Polymorphed units wander randomly
        if polymorphed.is_some() {
            let angle = (time.elapsed_secs() * 0.5 + velocity.x.to_bits() as f32).sin()
                * std::f32::consts::TAU;
            velocity.x = angle.cos() * 20.0;
            velocity.z = angle.sin() * 20.0;
            continue;
        }

        // Use shared weighted movement function
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            in_melee.is_some(),
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );

        // Stop completely when in optimal position (not in melee, not on hazard)
        // Skip for staging units — they need to keep following the flow field
        let is_staging =
            crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group);
        if !is_staging && in_melee.is_none() && flow_field_velocity.terrain_cost <= 1.0 {
            let targeting_is_zero = targeting_velocity.velocity.length_squared() < 0.01;
            if targeting_is_zero {
                velocity.x = 0.0;
                velocity.z = 0.0;
                acceleration.x = 0.0;
                acceleration.z = 0.0;
            }
        }
    }
}

/// Finds the nearest unshielded ally within shield range, if any.
fn find_shielder_target(
    ally_snapshot: &[(Entity, Vec3, Team, bool)],
    self_pos: Vec3,
    self_team: Team,
) -> Option<Entity> {
    ally_snapshot
        .iter()
        .filter(|(_, _, ally_team, has_shield)| *ally_team == self_team && !has_shield)
        .filter_map(|&(ally_entity, ally_pos, _, _)| {
            let dist = ((self_pos.x - ally_pos.x).powi(2) + (self_pos.z - ally_pos.z).powi(2))
                .sqrt();
            if dist <= SHIELD_RANGE {
                Some((ally_entity, dist))
            } else {
                None
            }
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(e, _)| e)
}

/// Starts a 5-second shield channel when cooldown is ready and a target is in range.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn shielder_start_shield_channel(
    mut commands: Commands,
    time: Res<Time>,
    shielder_assets: Res<super::resources::ShielderAssets>,
    mut shielders: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut ShielderShieldCooldown>,
            Option<&SleepModifier>,
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

/// Re-inserts the casting animation on channeling shielders so the animation loops.
pub fn shielder_refresh_casting_animation(
    mut commands: Commands,
    shielders: Query<
        Entity,
        (
            With<Shielder>,
            With<ChannelingCast>,
            Without<CombatAnimation>,
            Without<Corpse>,
        ),
    >,
    shielder_assets: Res<super::resources::ShielderAssets>,
) {
    for entity in &shielders {
        commands.entity(entity).insert(CombatAnimation::new_casting(
            shielder_assets.casting_texture.clone(),
            shielder_assets.sprite_texture.clone(),
        ));
    }
}

/// Spawns inward-imploding yellow particles around each channeling shielder.
pub fn shielder_spawn_channel_particles(
    mut commands: Commands,
    shielders: Query<(&Transform, &ChannelingCast, &Hitbox), (With<Shielder>, Without<Corpse>)>,
    shielder_assets: Res<super::resources::ShielderAssets>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut game_rng: ResMut<GameRng>,
) {
    *timer += time.delta_secs();
    if *timer < SHIELDER_CHANNEL_PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *timer -= SHIELDER_CHANNEL_PARTICLE_SPAWN_INTERVAL;

    let spec = ChannelParticleSpec {
        start_radius: SHIELDER_CHANNEL_PARTICLE_START_RADIUS,
        max_radius: SHIELDER_CHANNEL_PARTICLE_MAX_RADIUS,
        size: SHIELDER_CHANNEL_PARTICLE_SIZE,
        lifetime: SHIELDER_CHANNEL_PARTICLE_LIFETIME,
        count_per_spawn: SHIELDER_CHANNEL_PARTICLE_COUNT_PER_SPAWN,
    };

    for (transform, channel, hitbox) in &shielders {
        let progress = channel.elapsed / SHIELDER_CAST_DURATION;
        let center = transform.translation + Vec3::Y * (hitbox.height * 0.5);
        spawn_channel_particle_batch(
            &mut commands,
            center,
            progress,
            &visual_assets.particle_quad,
            &shielder_assets.channel_particle_material,
            &spec,
            &mut game_rng.0,
        );
    }
}
