use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::messages::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};

use super::resources::BehemothAssets;
use crate::game::resources::CurrentLevel;
use crate::game::units::components::{
    AttackTiming, CommanderAuraSpeedModifier, Corpse, DamageMultiplier, Effectiveness,
    EliteSpeedBonus, FlockingModifier, FlockingVelocity, FrostSlowModifier, HasteModifier, Health,
    Hitbox, InMelee, MovementSpeed, RootedModifier, RoughTerrainModifier, TargetingVelocity, Team,
    Teleportable,
};
use crate::game::units::random_position_in_cell;

/// Spawns behemoth attackers based on level (every 5 levels).
/// Behemoths spawn in the archer row alongside archers.
pub fn spawn_initial_behemoths(
    mut commands: Commands,
    behemoth_assets: Res<BehemothAssets>,
    current_level: Res<CurrentLevel>,
) {
    let level = current_level.0;

    // Only spawn behemoths at the specified interval
    if BEHEMOTH_SPAWN_LEVEL_INTERVAL > 1 && !level.is_multiple_of(BEHEMOTH_SPAWN_LEVEL_INTERVAL) {
        return;
    }

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

    // Behemoth spawns in first available column in archer row
    // Use column 0 to keep it separate from main archer formation
    let behemoth_col = 0;
    let (spawn_x, spawn_z) = calculate_grid_cell_position(archer_row, behemoth_col);

    // Randomly position near center of grid cell
    let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

    // Create hitbox
    let hitbox = Hitbox::new(BEHEMOTH_RADIUS, BEHEMOTH_HITBOX_HEIGHT);

    // Position unit so bottom edge is 1 unit above battlefield (Y=0)
    // The ellipse is billboarded (rotated to face camera), so when tilted,
    // half of its depth extends below the center point
    let spawn_y = hitbox.height / 2.0 + (BEHEMOTH_ELLIPSE_DEPTH / 2.0) + 1.0;

    // Initial velocity toward castle (center)
    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * BEHEMOTH_MOVEMENT_SPEED;

    commands
        .spawn((
            // Rendering
            Mesh3d(behemoth_assets.mesh.clone()),
            MeshMaterial3d(behemoth_assets.material.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            // Physics
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
            },
            Acceleration::new(),
            // Core
            hitbox,
            Health::new(BEHEMOTH_HEALTH),
            MovementSpeed(BEHEMOTH_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Behemoth,
            // Movement systems - MUST be in same spawn() for flow field to work!
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

/// Updates behemoth targeting velocity toward nearest enemy.
/// Uses same team-based targeting logic as infantry.
pub fn update_behemoth_targeting(
    mut commands: Commands,
    mut behemoths: Query<(Entity, &Transform, &Team, &mut TargetingVelocity), With<Behemoth>>,
    all_units: Query<(Entity, &Transform, &Team), (Without<Behemoth>, Without<Corpse>)>,
) {
    // Collect snapshot of all unit positions
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, behemoth_transform, behemoth_team, mut targeting) in &mut behemoths {
        // Use shared melee targeting function
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            behemoth_transform,
            *behemoth_team,
            &mut targeting,
            &mut commands,
        );
    }
}

/// Behemoth movement system using weighted velocities based on distance to target.
/// Uses same logic as infantry movement.
#[allow(clippy::type_complexity)]
pub fn behemoth_movement(
    time: Res<Time>,
    mut behemoths: Query<
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
            Option<&FrostSlowModifier>,
            Option<&CauldronSpeedModifier>,
            Option<&RootedModifier>,
            Option<&HasteModifier>,
            Option<&EliteSpeedBonus>,
        ),
        With<Behemoth>,
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
        frost_modifier,
        cauldron_modifier,
        rooted,
        haste_modifier,
        elite_speed,
    ) in &mut behemoths
    {
        // Rooted units cannot move
        if rooted.is_some() {
            velocity.x = 0.0;
            velocity.z = 0.0;
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
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );
    }
}

/// Tracks the position of the behemoth's attack target before combat happens.
/// Sends BehemothAttackMessage when a behemoth is about to attack.
pub fn track_behemoth_attack_target(
    attack_cycle: Res<crate::game::plugin::GlobalAttackCycle>,
    mut attack_events: MessageWriter<BehemothAttackMessage>,
    mut behemoths: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut AttackTiming),
        (With<Behemoth>, Without<Corpse>),
    >,
    all_units: Query<(Entity, &Transform, &Hitbox, &Team), Without<Corpse>>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - crate::game::constants::APPROX_FRAME_TIME).max(0.0);

    // Collect snapshot of all units
    let units_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, hitbox, team)| (entity, transform.translation, *hitbox, *team))
        .collect();

    for (behemoth_entity, behemoth_transform, behemoth_hitbox, behemoth_team, mut attack_timing) in
        &mut behemoths
    {
        // Only track target when behemoth is about to attack
        if attack_timing.can_attack(current_time, last_time) {
            // Find nearest enemy within attack range (same logic as combat system)
            if let Some((_, target_pos, _)) = units_snapshot
                .iter()
                .filter(|(entity, _, _, team)| {
                    *entity != behemoth_entity
                        && match (behemoth_team, team) {
                            (Team::Undead, Team::Undead) => false,
                            (Team::Undead, _) => true,
                            (_, Team::Undead) => true,
                            _ => *team != *behemoth_team,
                        }
                })
                .filter_map(|(entity, target_pos, target_hitbox, _)| {
                    // Calculate distance on XZ plane only (ignore Y axis for attack range)
                    let dx = behemoth_transform.translation.x - target_pos.x;
                    let dz = behemoth_transform.translation.z - target_pos.z;
                    let distance = (dx * dx + dz * dz).sqrt();
                    let attack_range = (behemoth_hitbox.radius + target_hitbox.radius)
                        * crate::game::constants::ATTACK_RANGE_MULTIPLIER;
                    if distance <= attack_range {
                        Some((entity, target_pos, distance))
                    } else {
                        None
                    }
                })
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
            {
                // Send message that behemoth is attacking
                attack_events.write(BehemothAttackMessage {
                    target_position: *target_pos,
                    behemoth_team: *behemoth_team,
                });

                // Record the attack to respect the attack cycle
                attack_timing.record_attack(current_time);
            }
        }
    }
}

/// Applies AOE splash damage around the behemoth's attack target.
/// Runs after combat system. Reads BehemothAttackMessage messages.
pub fn behemoth_aoe_splash_damage(
    mut attack_events: MessageReader<BehemothAttackMessage>,
    mut all_units: Query<
        (&Transform, &Team, &Hitbox, &mut Health),
        (Without<Behemoth>, Without<Corpse>),
    >,
) {
    for event in attack_events.read() {
        // Apply damage to all enemies within AOE radius of the target
        for (unit_transform, unit_team, hitbox, mut health) in &mut all_units {
            // Skip if same team
            if *unit_team == event.behemoth_team {
                continue;
            }

            // Calculate distance from target position to this unit (XZ plane only)
            let dx = unit_transform.translation.x - event.target_position.x;
            let dz = unit_transform.translation.z - event.target_position.z;
            let distance_to_target = (dx * dx + dz * dz).sqrt();

            // Check if within AOE radius (accounting for hitbox)
            if distance_to_target <= BEHEMOTH_AOE_RADIUS + hitbox.radius {
                // Apply AOE damage
                health.current -= BEHEMOTH_AOE_DAMAGE;
            }
        }
    }
}
