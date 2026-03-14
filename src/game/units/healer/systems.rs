use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::HealerAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::brute::components::Brute;
use crate::game::units::commander::components::Commander;
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, Corpse, Effectiveness, EliteSpeedBonus,
    FlockingVelocity, HasteModifier, Health, Hitbox, MovementSpeed, PolymorphedModifier,
    FrozenSolidModifier, RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier,
    Sleepwalking, SlowMovementModifier,
    TargetingVelocity, Team,
};
use crate::game::units::dispeller::components::Dispeller;
use crate::game::units::infantry::components::DefendersActivated;

/// Updates healer targeting — seeks hurt same-team allies, or falls back to following army.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_healer_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut healers: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
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
        Without<Corpse>,
    >,
    all_units: Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    // Snapshot ally data for heal targeting
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

    // Collect unit snapshot for enemy targeting fallback
    let unit_snapshot: Vec<(Entity, Vec3, Team)> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting_velocity) in &mut healers {
        // Skip inactive defender healers
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Priority 1: Find hurt allies to heal / move toward
        let best_hurt_ally =
            find_best_heal_target(&ally_snapshot, entity, transform.translation, *team);

        if let Some((_, ally_pos, _)) = best_hurt_ally {
            let diff = ally_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            if distance <= HEAL_RANGE {
                // In heal range — stop moving
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                // Move toward hurt ally
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

            // Stay at heal range from enemies
            if distance <= HEAL_RANGE {
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

/// Healer movement system using shared weighted movement.
#[allow(clippy::type_complexity)]
pub fn healer_movement(
    time: Res<Time>,
    mut healer_units: Query<
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
            ),
        ),
        With<Healer>,
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
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen),
    ) in &mut healer_units
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(rooted, sleeping, sleepwalking, banished, sickened, frozen) {
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
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
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

/// Fires a heal bolt at the best hurt ally within range when cooldown is ready.
#[allow(clippy::type_complexity)]
pub fn healer_fire_heal_bolt(
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
        Without<Corpse>,
    >,
) {
    let delta = time.delta_secs();

    // Snapshot ally data for heal targeting
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

    for (healer_entity, healer_transform, healer_team, mut attack_timer, sleeping, banished) in
        &mut healers
    {
        attack_timer.time_since_last_attack += delta;

        // Skip if CC'd
        if sleeping.is_some() || banished.is_some() {
            continue;
        }

        // Check cooldown
        if attack_timer.time_since_last_attack < HEAL_COOLDOWN {
            continue;
        }

        // Find best heal target within range
        let best_target = find_best_heal_target(
            &ally_snapshot,
            healer_entity,
            healer_transform.translation,
            *healer_team,
        );

        if let Some((target_entity, target_pos, distance)) = best_target {
            if distance > HEAL_RANGE {
                continue;
            }

            // Spawn homing heal bolt toward target
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

            attack_timer.time_since_last_attack = 0.0;
        }
    }
}

/// Moves heal bolts toward their targets (homing). Despawns if target is gone or lifetime expires.
pub fn move_heal_bolts(
    mut commands: Commands,
    time: Res<Time>,
    mut bolts: Query<(Entity, &mut Transform, &mut HealBolt)>,
    targets: Query<&Transform, Without<HealBolt>>,
) {
    let delta = time.delta_secs();
    for (entity, mut bolt_transform, mut bolt) in &mut bolts {
        bolt.lifetime -= delta;
        if bolt.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Recalculate direction toward target each frame (homing)
        if let Ok(target_transform) = targets.get(bolt.target) {
            let diff = target_transform.translation - bolt_transform.translation;
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();
            bolt_transform.translation += direction * bolt.speed * delta;
        } else {
            // Target gone — despawn bolt
            commands.entity(entity).try_despawn();
        }
    }
}

/// Checks if heal bolts have reached their targets and applies healing.
pub fn check_heal_bolt_arrivals(
    mut commands: Commands,
    bolts: Query<(Entity, &Transform, &HealBolt)>,
    mut targets: Query<(&Transform, &Hitbox, &Team, &mut Health), Without<Corpse>>,
) {
    for (bolt_entity, bolt_transform, bolt) in &bolts {
        let bolt_pos = bolt_transform.translation;

        if let Ok((target_transform, hitbox, team, mut health)) = targets.get_mut(bolt.target) {
            // Verify same team
            if *team != bolt.source_team {
                commands.entity(bolt_entity).try_despawn();
                continue;
            }

            // Check if bolt has arrived
            let distance = bolt_pos.distance(target_transform.translation);
            if distance < hitbox.radius + HEAL_BOLT_RADIUS {
                // Heal to full
                health.current = health.max;
                commands.entity(bolt_entity).try_despawn();
            }
        }
    }
}

/// Returns a priority score for heal targeting.
/// Commander=5, Brute=4, Elite=3, Dispeller=2, Healer=1, regular=0.
fn find_heal_priority(
    is_commander: bool,
    is_brute: bool,
    is_elite: bool,
    is_dispeller: bool,
    is_healer: bool,
) -> u32 {
    if is_commander {
        5
    } else if is_brute {
        4
    } else if is_elite {
        3
    } else if is_dispeller {
        2
    } else if is_healer {
        1
    } else {
        0
    }
}

/// Finds the best heal target from a snapshot of allies.
/// Prioritizes by unit type priority, then by lowest HP percentage within same priority.
/// Only considers hurt allies (current < max) on the same team, excluding self.
/// Returns (entity, position, distance) if a target is found.
fn find_best_heal_target(
    ally_snapshot: &[(Entity, Vec3, Team, f32, f32, u32)],
    self_entity: Entity,
    self_pos: Vec3,
    self_team: Team,
) -> Option<(Entity, Vec3, f32)> {
    ally_snapshot
        .iter()
        .filter(|(entity, _, team, current, max, _)| {
            *entity != self_entity && *team == self_team && *current < *max
        })
        .max_by(|a, b| {
            // Higher priority first
            let priority_cmp = a.5.cmp(&b.5);
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }
            // Within same priority, lower HP percentage is better (more hurt = higher priority)
            let hp_pct_a = a.3 / a.4;
            let hp_pct_b = b.3 / b.4;
            hp_pct_b
                .partial_cmp(&hp_pct_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, pos, _, _, _, _)| {
            let distance = ((self_pos.x - pos.x).powi(2) + (self_pos.z - pos.z).powi(2)).sqrt();
            (*entity, *pos, distance)
        })
}
