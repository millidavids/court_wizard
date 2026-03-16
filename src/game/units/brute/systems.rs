use std::cmp::Ordering;

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::messages::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};

use super::resources::BruteAssets;
use crate::game::resources::CurrentLevel;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, DamageMultiplier,
    Effectiveness, EliteSpeedBonus, FlockingModifier, FlockingVelocity, HasteModifier, Health,
    Hitbox, InMelee, MovementSpeed, PolymorphedModifier, RetaliationTarget, RootedModifier,
    FrozenSolidModifier, RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking,
    SlowMovementModifier, TargetingVelocity, Team, Teleportable,
};
use crate::game::units::random_position_in_cell;

/// Spawns a brute attacker.
/// Brutes spawn in the archer row alongside archers.
pub fn spawn_brute(
    mut commands: Commands,
    brute_assets: Res<BruteAssets>,
    current_level: Res<CurrentLevel>,
) {
    let level = current_level.0;

    // Calculate spawn position in archer row
    // Use the same logic as archers - spawn in the row behind infantry
    let total_infantry = calculate_total_infantry(level);
    let total_archers = calculate_total_archers(level);
    let infantry_cells_needed = cells_needed(total_infantry);
    let archer_cells_needed = cells_needed(total_archers);

    let (infantry_cells, archer_cells) =
        calculate_spawn_cells(infantry_cells_needed, archer_cells_needed);

    // Find the archer row (one behind last infantry row)
    let archer_row = if let Some(&(row, _)) = archer_cells.first() {
        row
    } else {
        // Fallback if no archer cells (shouldn't happen)
        let last_infantry_row = infantry_cells.iter().map(|&(r, _)| r).max().unwrap_or(0);
        last_infantry_row + 1
    };

    // Brute spawns in first available column in archer row
    // Use column 0 to keep it separate from main archer formation
    let brute_col = 0;
    let (spawn_x, spawn_z) = calculate_grid_cell_position(archer_row, brute_col);

    // Randomly position near center of grid cell
    let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

    // Create hitbox
    let hitbox = Hitbox::new(BRUTE_RADIUS, BRUTE_HITBOX_HEIGHT);

    // Position unit so bottom edge is 1 unit above battlefield (Y=0)
    let spawn_y = hitbox.height / 2.0 + (BRUTE_ELLIPSE_DEPTH / 2.0) + 1.0;

    // Initial velocity toward castle (center)
    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * BRUTE_MOVEMENT_SPEED;

    commands
        .spawn((
            // Rendering
            Mesh3d(brute_assets.mesh.clone()),
            MeshMaterial3d(brute_assets.material.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            // Physics
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
            },
            Acceleration::new(),
            // Core
            hitbox,
            Health::new(BRUTE_HEALTH),
            MovementSpeed(BRUTE_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Brute,
            // Movement systems
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
        ))
        .insert((
            // Primary attack does no damage (-1.0 multiplier means 50 * (1.0 - 1.0) = 0)
            // All damage comes from AOE splash
            DamageMultiplier(-1.0),
            FlockingVelocity::default(),
            FlockingModifier::new(1.0, 1.0, 1.0),
            CommanderAuraSpeedModifier(0.0),
            RoughTerrainModifier(0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ));
}

/// Updates brute targeting velocity toward nearest enemy.
pub fn update_brute_targeting(
    mut commands: Commands,
    mut brutes: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            Option<&RetaliationTarget>,
        ),
        With<Brute>,
    >,
    all_units: Query<(Entity, &Transform, &Team), (Without<Brute>, Without<Corpse>, Without<BanishedModifier>)>,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, brute_transform, brute_team, mut targeting, retaliation) in &mut brutes {
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            brute_transform,
            *brute_team,
            &mut targeting,
            &mut commands,
            retaliation.map(|r| r.0),
        );
    }
}

/// Brute movement system using weighted velocities.
#[allow(clippy::type_complexity)]
pub fn brute_movement(
    time: Res<Time>,
    mut brutes: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&InMelee>,
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
            ),
        ),
        With<Brute>,
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
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned),
    ) in &mut brutes
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned) {
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
    }
}

/// Tracks the position of the brute's attack target before combat happens.
/// Sends BruteAttackMessage when a brute is about to attack.
pub fn track_brute_attack_target(
    attack_cycle: Res<crate::game::plugin::GlobalAttackCycle>,
    mut attack_events: MessageWriter<BruteAttackMessage>,
    mut brutes: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut AttackTiming),
        (With<Brute>, Without<Corpse>),
    >,
    all_units: Query<(Entity, &Transform, &Hitbox, &Team), Without<Corpse>>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - crate::game::constants::APPROX_FRAME_TIME).max(0.0);

    let units_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, hitbox, team)| (entity, transform.translation, *hitbox, *team))
        .collect();

    for (brute_entity, brute_transform, brute_hitbox, brute_team, mut attack_timing) in &mut brutes
    {
        if attack_timing.can_attack(current_time, last_time)
            && let Some((_, target_pos, _)) = units_snapshot
                .iter()
                .filter(|(entity, _, _, team)| *entity != brute_entity && brute_team.is_enemy(team))
                .filter_map(|(entity, target_pos, target_hitbox, _)| {
                    let dx = brute_transform.translation.x - target_pos.x;
                    let dz = brute_transform.translation.z - target_pos.z;
                    let distance = (dx * dx + dz * dz).sqrt();
                    let attack_range = (brute_hitbox.radius + target_hitbox.radius)
                        * crate::game::constants::ATTACK_RANGE_MULTIPLIER;
                    if distance <= attack_range {
                        Some((entity, target_pos, distance))
                    } else {
                        None
                    }
                })
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
        {
            attack_events.write(BruteAttackMessage {
                target_position: *target_pos,
                brute_team: *brute_team,
            });
            attack_timing.record_attack(current_time);
        }
    }
}

/// Applies AOE splash damage around the brute's attack target.
pub fn brute_aoe_splash_damage(
    mut attack_events: MessageReader<BruteAttackMessage>,
    mut all_units: Query<
        (&Transform, &Team, &Hitbox, &mut Health),
        (Without<Brute>, Without<Corpse>),
    >,
) {
    for event in attack_events.read() {
        for (unit_transform, unit_team, hitbox, mut health) in &mut all_units {
            if *unit_team == event.brute_team {
                continue;
            }

            let dx = unit_transform.translation.x - event.target_position.x;
            let dz = unit_transform.translation.z - event.target_position.z;
            let distance_to_target = (dx * dx + dz * dz).sqrt();

            if distance_to_target <= BRUTE_AOE_RADIUS + hitbox.radius {
                health.current -= BRUTE_AOE_DAMAGE;
            }
        }
    }
}
