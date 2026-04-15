use std::cmp::Ordering;

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::DispellerAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Velocity};
use crate::game::multiplayer::components::NetworkedSpellEffect;
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
use crate::game::units::ranged_bolt::RangedAttackTimer;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::dispel::systems::{
    is_dispellable, spawn_dispel_projectile, spell_edge_distance,
};
use crate::game::units::wizard::spells::grease::components::{GreaseIgnited, GreaseZone};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Updates dispeller targeting — seeks nearest dispellable spell effect, or falls back to enemy targeting.
#[allow(clippy::too_many_arguments)]
pub fn update_dispeller_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut dispellers: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Dispeller>, Without<Corpse>),
    >,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect)>,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
    // Spell-specific queries for volume-aware distance
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
) {
    // Collect dispellable spell effects (entity + center position)
    let spell_targets: Vec<(Entity, Vec3)> = spell_effects
        .iter()
        .filter(|(_, _, nse)| is_dispellable(nse.kind))
        .map(|(entity, transform, _)| (entity, transform.translation))
        .collect();

    // Collect unit snapshot for enemy targeting fallback
    let unit_snapshot: Vec<(Entity, Vec3, Team)> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting_velocity) in &mut dispellers {
        // Skip inactive defender dispellers
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Priority 1: Nearest dispellable spell effect (using edge distance)
        if !spell_targets.is_empty() {
            let nearest_spell = spell_targets.iter().min_by(|a, b| {
                let dist_a = spell_edge_distance(
                    transform.translation,
                    a.0,
                    a.1,
                    &wall_of_fire_query,
                    &wall_of_stone_query,
                    &spike_growth_query,
                    &grease_query,
                    &meteor_fire_query,
                );
                let dist_b = spell_edge_distance(
                    transform.translation,
                    b.0,
                    b.1,
                    &wall_of_fire_query,
                    &wall_of_stone_query,
                    &spike_growth_query,
                    &grease_query,
                    &meteor_fire_query,
                );
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            if let Some(&(spell_entity, target_pos)) = nearest_spell {
                let distance = spell_edge_distance(
                    transform.translation,
                    spell_entity,
                    target_pos,
                    &wall_of_fire_query,
                    &wall_of_stone_query,
                    &spike_growth_query,
                    &grease_query,
                    &meteor_fire_query,
                );
                targeting_velocity.distance_to_target = distance;

                if distance <= DISPEL_RANGE {
                    // In dispel range — stop moving
                    targeting_velocity.velocity = Vec3::ZERO;
                } else {
                    // Move toward spell effect center
                    let diff = target_pos - transform.translation;
                    let direction = diff.normalize_or_zero();
                    targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
                }

                // Dispellers don't engage in melee — remove InMelee
                commands
                    .entity(entity)
                    .remove::<crate::game::units::components::InMelee>();
                continue;
            }
        }

        // Priority 2: Fall back to ranged enemy targeting (like archers)
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

            if distance <= ATTACK_RANGE {
                // In attack range — stop and shoot
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                // Move toward enemy
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

/// Dispeller movement system using shared weighted movement.
#[allow(clippy::type_complexity)]
pub fn dispeller_movement(
    time: Res<Time>,
    mut dispeller_units: Query<
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
        With<Dispeller>,
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
    ) in &mut dispeller_units
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

/// Starts a 5-second dispel channel when cooldown is ready and a dispellable
/// spell effect is within range.
#[allow(clippy::too_many_arguments)]
pub fn dispeller_start_dispel_channel(
    mut commands: Commands,
    time: Res<Time>,
    dispeller_assets: Res<DispellerAssets>,
    mut dispellers: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut DispellerDispelCooldown>,
            Has<ChannelingCast>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Dispeller>, Without<Corpse>),
    >,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect)>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
) {
    let delta = time.delta_secs();

    let dispellable_effects: Vec<(Entity, Vec3)> = spell_effects
        .iter()
        .filter(|(_, _, nse)| is_dispellable(nse.kind))
        .map(|(e, tf, _)| (e, tf.translation))
        .collect();

    if dispellable_effects.is_empty() {
        return;
    }

    for (entity, transform, team, cooldown, is_channeling, has_staging, has_wave_group) in
        &mut dispellers
    {
        if crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group) {
            continue;
        }
        if let Some(mut cd) = cooldown {
            cd.remaining -= delta;
            if cd.remaining > 0.0 {
                continue;
            }
            commands.entity(entity).remove::<DispellerDispelCooldown>();
        }

        if is_channeling {
            continue;
        }

        let has_in_range = dispellable_effects.iter().any(|&(spell_entity, spell_pos)| {
            spell_edge_distance(
                transform.translation,
                spell_entity,
                spell_pos,
                &wall_of_fire_query,
                &wall_of_stone_query,
                &spike_growth_query,
                &grease_query,
                &meteor_fire_query,
            ) <= DISPEL_RANGE
        });
        if !has_in_range {
            continue;
        }

        commands.entity(entity).insert((
            ChannelingCast { elapsed: 0.0 },
            CombatAnimation::new_casting(
                dispeller_assets.casting_texture.clone(),
                dispeller_assets.sprite_texture.clone(),
            ),
        ));
    }
}

/// Ticks active dispel channels. On completion spawns a dispel projectile at
/// a freshly-picked in-range dispellable effect and starts the cooldown.
#[allow(clippy::too_many_arguments)]
pub fn dispeller_tick_dispel_channel(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dispellers: Query<
        (Entity, &Transform, &mut ChannelingCast),
        (With<Dispeller>, Without<Corpse>),
    >,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect)>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
) {
    let delta = time.delta_secs();

    let mut dispellable_effects: Option<Vec<(Entity, Vec3)>> = None;

    for (entity, transform, mut channel) in &mut dispellers {
        channel.elapsed += delta;
        if channel.elapsed < DISPELLER_CAST_DURATION {
            continue;
        }

        commands
            .entity(entity)
            .remove::<ChannelingCast>()
            .insert(DispellerDispelCooldown {
                remaining: DISPEL_COOLDOWN,
            });

        let effects = dispellable_effects.get_or_insert_with(|| {
            spell_effects
                .iter()
                .filter(|(_, _, nse)| is_dispellable(nse.kind))
                .map(|(e, tf, _)| (e, tf.translation))
                .collect()
        });

        let nearest = effects
            .iter()
            .filter_map(|&(spell_entity, spell_pos)| {
                let dist = spell_edge_distance(
                    transform.translation,
                    spell_entity,
                    spell_pos,
                    &wall_of_fire_query,
                    &wall_of_stone_query,
                    &spike_growth_query,
                    &grease_query,
                    &meteor_fire_query,
                );
                if dist <= DISPEL_RANGE {
                    Some((spell_pos, dist))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

        if let Some((target_pos, _)) = nearest {
            spawn_dispel_projectile(
                &mut commands,
                &mut meshes,
                &mut materials,
                transform.translation,
                target_pos,
                0.0,
            );
        }
    }
}

/// Re-inserts the casting animation on channeling dispellers so it loops.
pub fn dispeller_refresh_casting_animation(
    mut commands: Commands,
    dispellers: Query<
        Entity,
        (
            With<Dispeller>,
            With<ChannelingCast>,
            Without<CombatAnimation>,
            Without<Corpse>,
        ),
    >,
    dispeller_assets: Res<DispellerAssets>,
) {
    for entity in &dispellers {
        commands.entity(entity).insert(CombatAnimation::new_casting(
            dispeller_assets.casting_texture.clone(),
            dispeller_assets.sprite_texture.clone(),
        ));
    }
}

/// Spawns inward-imploding white particles around each channeling dispeller.
pub fn dispeller_spawn_channel_particles(
    mut commands: Commands,
    dispellers: Query<(&Transform, &ChannelingCast, &Hitbox), (With<Dispeller>, Without<Corpse>)>,
    dispeller_assets: Res<DispellerAssets>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut game_rng: ResMut<GameRng>,
) {
    *timer += time.delta_secs();
    if *timer < DISPELLER_CHANNEL_PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *timer -= DISPELLER_CHANNEL_PARTICLE_SPAWN_INTERVAL;

    let spec = ChannelParticleSpec {
        start_radius: DISPELLER_CHANNEL_PARTICLE_START_RADIUS,
        max_radius: DISPELLER_CHANNEL_PARTICLE_MAX_RADIUS,
        size: DISPELLER_CHANNEL_PARTICLE_SIZE,
        lifetime: DISPELLER_CHANNEL_PARTICLE_LIFETIME,
        count_per_spawn: DISPELLER_CHANNEL_PARTICLE_COUNT_PER_SPAWN,
    };

    for (transform, channel, hitbox) in &dispellers {
        let progress = channel.elapsed / DISPELLER_CAST_DURATION;
        let center = transform.translation + Vec3::Y * (hitbox.height * 0.5);
        spawn_channel_particle_batch(
            &mut commands,
            center,
            progress,
            &visual_assets.particle_quad,
            &dispeller_assets.channel_particle_material,
            &spec,
            &mut game_rng.0,
        );
    }
}

/// Fires weak magic bolts at enemies when no spell effects exist to dispel.
#[allow(clippy::type_complexity)]
pub fn dispeller_ranged_combat(
    mut commands: Commands,
    time: Res<Time>,
    dispeller_assets: Res<DispellerAssets>,
    spell_effects: Query<&NetworkedSpellEffect>,
    mut dispellers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut RangedAttackTimer,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
            Has<ChannelingCast>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Dispeller>, Without<Corpse>),
    >,
    targets: Query<(Entity, &Transform, &Team), (Without<Corpse>, Without<BanishedModifier>)>,
) {
    // Only fire bolts when no dispellable spell effects exist
    let has_spell_targets = spell_effects.iter().any(|nse| is_dispellable(nse.kind));
    if has_spell_targets {
        return;
    }

    let delta = time.delta_secs();

    for (
        dispeller_entity,
        dispeller_transform,
        dispeller_team,
        mut attack_timer,
        sleeping,
        banished,
        is_channeling,
        has_staging,
        has_wave_group,
    ) in &mut dispellers
    {
        // Skip staging attackers (includes 1-frame delay before WaveGroup is added)
        if crate::game::units::systems::is_staging_attacker(
            dispeller_team,
            has_staging,
            has_wave_group,
        ) {
            continue;
        }

        attack_timer.time_since_last_attack += delta;

        if is_channeling || sleeping.is_some() || banished.is_some() {
            continue;
        }

        // Check cooldown
        if attack_timer.time_since_last_attack < ATTACK_COOLDOWN {
            continue;
        }

        // Find nearest enemy within attack range
        let nearest_enemy = targets
            .iter()
            .filter(|(entity, _, team)| {
                *entity != dispeller_entity && dispeller_team.is_enemy(team)
            })
            .filter(|(_, transform, _)| {
                let distance = dispeller_transform
                    .translation
                    .distance(transform.translation);
                distance <= ATTACK_RANGE
            })
            .min_by(|a, b| {
                let dist_a = dispeller_transform.translation.distance(a.1.translation);
                let dist_b = dispeller_transform.translation.distance(b.1.translation);
                dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
            });

        if let Some((_, target_transform, _)) = nearest_enemy {
            let origin = dispeller_transform.translation + Vec3::Y * 10.0;
            let diff = target_transform.translation - origin;
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();

            crate::game::units::ranged_bolt::spawn_magic_bolt(
                &mut commands,
                &dispeller_assets.bolt_mesh,
                &dispeller_assets.bolt_material,
                origin,
                direction,
                BOLT_SPEED,
                BOLT_DAMAGE,
                BOLT_RADIUS,
                BOLT_LIFETIME,
                *dispeller_team,
            );

            attack_timer.time_since_last_attack = 0.0;

            commands.entity(dispeller_entity).insert(
                crate::game::units::components::CombatAnimation::new_attack(
                    dispeller_assets.attacking_texture.clone(),
                    dispeller_assets.sprite_texture.clone(),
                ),
            );
        }
    }
}
