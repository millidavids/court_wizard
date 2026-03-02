use std::cmp::Ordering;

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::DispellerAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, Corpse, Effectiveness, EliteSpeedBonus,
    FlockingVelocity, FrostSlowModifier, GreaseSlipModifier, HasteModifier, Health, Hitbox,
    MovementSpeed, PolymorphedModifier, RootedModifier, RoughTerrainModifier,
    SleepModifier, SpikeGrowthSlowModifier, TargetingVelocity, Team, TemporaryHitPoints,
    apply_damage_to_unit,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::wizard::spells::dispel::systems::{
    is_dispellable, spawn_dispel_projectile, spell_edge_distance,
};
use crate::game::units::wizard::spells::grease::components::GreaseZone;
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
    all_units: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    // Spell-specific queries for volume-aware distance
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<&GreaseZone>,
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
                *other_entity != entity
                    && team.is_enemy(other_team)
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
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&crate::game::units::components::InMelee>,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            (Option<&FrostSlowModifier>, Option<&SpikeGrowthSlowModifier>),
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Option<&SleepModifier>,
                Option<&BanishedModifier>,
                Option<&GreaseSlipModifier>,
                Option<&PolymorphedModifier>,
            ),
        ),
        With<Dispeller>,
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        effectiveness,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
        (frost_modifier, spike_growth_modifier),
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, banished, grease, polymorphed),
    ) in &mut dispeller_units
    {
        // CC'd units cannot move
        if rooted.is_some() || sleeping.is_some() || banished.is_some() {
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
            effectiveness,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            in_melee.is_some(),
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            frost_modifier.map(|m| m.modifier),
            spike_growth_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
            grease.map(|g| g.modifier),
        );

        // Stop completely when in optimal position (not in melee, not on hazard)
        if in_melee.is_none() && flow_field_velocity.terrain_cost <= 1.0 {
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

/// Dispellers fire dispel projectiles at nearby spell effects using a cooldown.
#[allow(clippy::too_many_arguments)]
pub fn dispeller_cast_dispel(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dispellers: Query<
        (
            Entity,
            &Transform,
            Option<&mut DispellerDispelCooldown>,
        ),
        (With<Dispeller>, Without<Corpse>),
    >,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect)>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<&GreaseZone>,
    meteor_fire_query: Query<&MeteorGroundFire>,
) {
    let delta = time.delta_secs();

    // Collect dispellable spell effects
    let dispellable_effects: Vec<(Entity, Vec3)> = spell_effects
        .iter()
        .filter(|(_, _, nse)| is_dispellable(nse.kind))
        .map(|(e, tf, _)| (e, tf.translation))
        .collect();

    if dispellable_effects.is_empty() {
        return;
    }

    for (entity, transform, cooldown) in &mut dispellers {
        // Tick cooldown if present
        if let Some(mut cd) = cooldown {
            cd.remaining -= delta;
            if cd.remaining > 0.0 {
                continue;
            }
            // Cooldown expired — remove it and allow casting
            commands.entity(entity).remove::<DispellerDispelCooldown>();
        }

        // Find nearest dispellable spell effect in range
        let nearest = dispellable_effects
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
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((target_pos, _)) = nearest {
            // Fire dispel projectile at ground level
            spawn_dispel_projectile(
                &mut commands,
                &mut meshes,
                &mut materials,
                transform.translation,
                target_pos,
                0.0,
            );

            // Set cooldown
            commands.entity(entity).insert(DispellerDispelCooldown {
                remaining: DISPEL_COOLDOWN,
            });
        }
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
            &mut DispellerAttackTimer,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
        ),
        (With<Dispeller>, Without<Corpse>),
    >,
    targets: Query<(Entity, &Transform, &Team), Without<Corpse>>,
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
    ) in &mut dispellers
    {
        attack_timer.time_since_last_attack += delta;

        // Skip if CC'd
        if sleeping.is_some() || banished.is_some() {
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
                *entity != dispeller_entity
                    && dispeller_team.is_enemy(team)
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
            // Spawn bolt toward target
            let origin = dispeller_transform.translation + Vec3::Y * 10.0;
            let diff = target_transform.translation - origin;
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();

            commands.spawn((
                Mesh3d(dispeller_assets.bolt_mesh.clone()),
                MeshMaterial3d(dispeller_assets.bolt_material.clone()),
                Transform::from_translation(origin),
                DispellerBolt {
                    velocity: direction * BOLT_SPEED,
                    damage: BOLT_DAMAGE,
                    source_team: *dispeller_team,
                    lifetime: BOLT_LIFETIME,
                },
                Billboard,
                OnGameplayScreen,
            ));

            attack_timer.time_since_last_attack = 0.0;
        }
    }
}

/// Moves dispeller bolts and despawns expired ones.
pub fn move_dispeller_bolts(
    mut commands: Commands,
    time: Res<Time>,
    mut bolts: Query<(Entity, &mut Transform, &mut DispellerBolt)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut bolt) in &mut bolts {
        transform.translation += bolt.velocity * delta;
        bolt.lifetime -= delta;
        if bolt.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Checks bolt collisions with enemy units.
pub fn check_bolt_collisions(
    mut commands: Commands,
    bolts: Query<(Entity, &Transform, &DispellerBolt)>,
    mut targets: Query<
        (
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
) {
    #[allow(clippy::significant_drop_in_scrutinee)]
    for (bolt_entity, bolt_transform, bolt) in &bolts {
        let bolt_pos = bolt_transform.translation;

        for (target_transform, hitbox, team, mut health, mut temp_hp) in &mut targets {
            // Skip same team
            if *team == bolt.source_team {
                continue;
            }

            let is_enemy = bolt.source_team.is_enemy(team);

            if !is_enemy {
                continue;
            }

            // Check collision
            let distance = bolt_pos.distance(target_transform.translation);
            if distance < hitbox.radius + BOLT_RADIUS {
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), bolt.damage);
                commands.entity(bolt_entity).despawn();
                break;
            }
        }
    }
}
