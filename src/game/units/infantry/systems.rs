use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    calculate_defender_grid_position, calculate_grid_cell_position, calculate_spawn_cells,
    calculate_total_archers, calculate_total_infantry, cells_needed, distribute_units_to_cells, *,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, Effectiveness,
    EliteSpeedBonus, FacingDirection, FlockingVelocity, FrostSlowModifier, GreaseSlipModifier,
    HasteModifier, Health, Hitbox, KingsGuard, MovementSpeed, PolymorphedModifier,
    RootedModifier, RoughTerrainModifier, SleepModifier, SpikeGrowthSlowModifier,
    TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::elite::{EliteDamageBonus, EliteHealthBonus};
use crate::game::units::random_position_in_cell;

use super::resources::InfantryAssets;

/// Checks if any attacker is within activation range of any defender.
///
/// Once a single defender detects an enemy within DEFENDER_ACTIVATION_RANGE,
/// ALL defenders activate collectively via the DefendersActivated resource.
/// Activation persists for the entire game.
pub fn check_defender_activation(
    mut defenders_activated: ResMut<DefendersActivated>,
    defender_query: Query<(&Transform, &Team), (With<Infantry>, Without<Corpse>)>,
    attacker_query: Query<(&Transform, &Team), Without<Corpse>>,
) {
    // If already activated, check whether any enemies remain on the battlefield.
    // If not, deactivate so defenders hold position until the next wave arrives.
    if defenders_activated.active {
        let enemies_exist = attacker_query.iter().any(|(_, team)| {
            *team == Team::Attackers || *team == Team::Undead
        });
        if !enemies_exist {
            defenders_activated.active = false;
            info!("Defenders deactivated — no enemies on battlefield");
        }
        return;
    }

    // Check if ANY attacker is within activation range of ANY defender
    // As soon as one defender "sees" an enemy, all defenders activate
    for (defender_transform, defender_team) in defender_query.iter() {
        // Only check defender infantry, not attacker infantry
        if *defender_team != Team::Defenders {
            continue;
        }

        for (attacker_transform, attacker_team) in attacker_query.iter() {
            // Only check against Attackers and Undead, not other Defenders
            if *attacker_team != Team::Attackers && *attacker_team != Team::Undead {
                continue;
            }

            let dx = defender_transform.translation.x - attacker_transform.translation.x;
            let dz = defender_transform.translation.z - attacker_transform.translation.z;
            let distance = (dx * dx + dz * dz).sqrt();

            if distance <= DEFENDER_ACTIVATION_RANGE {
                // Activate ALL defenders collectively
                defenders_activated.active = true;
                info!(
                    "Defenders activated! Enemy within {} units - all defenders now active",
                    DEFENDER_ACTIVATION_RANGE
                );
                return;
            }
        }
    }
}

/// Updates infantry targeting velocity toward nearest enemy.
///
/// Infantry always move directly toward the nearest enemy.
/// Also sets InMelee component if an enemy is within melee range.
/// Defender infantry are gated by the DefendersActivated resource.
pub fn update_infantry_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut infantry: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut crate::game::units::components::TargetingVelocity,
            Option<&crate::game::units::components::RetaliationTarget>,
        ),
        (
            With<Infantry>,
            Without<crate::game::units::components::Corpse>,
        ),
    >,
    all_units: Query<(Entity, &Transform, &Team), Without<crate::game::units::components::Corpse>>,
) {
    // Collect snapshot of all unit positions
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Update each infantry's targeting velocity
    for (entity, transform, team, mut targeting_velocity, retaliation) in &mut infantry {
        // Skip inactive defender infantry (but always process attackers)
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands.entity(entity).remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Use shared melee targeting function
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            transform,
            *team,
            &mut targeting_velocity,
            &mut commands,
            retaliation.map(|r| r.0),
        );
    }
}

/// Infantry-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// Units slow down when in melee to prevent erratic movement.
#[allow(clippy::type_complexity)]
pub fn infantry_movement(
    time: Res<Time>,
    mut infantry_units: Query<
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
        With<Infantry>,
    >,
) {
    // Process each infantry unit
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
    ) in &mut infantry_units
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
    }
}

/// Spawns a single defender infantry unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_defender(
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
) {
    let total_units = INITIAL_DEFENDER_COUNT;
    let cells_needed = cells_needed(total_units);
    let units_per_cell = distribute_units_to_cells(total_units);

    // Generate cell list in reverse row order
    let mut defender_cells = Vec::new();
    let mut cells_added = 0;
    'outer: for row in (0..DEFENDER_GRID_ROWS).rev() {
        for col in 0..DEFENDER_GRID_COLS {
            defender_cells.push((row, col));
            cells_added += 1;
            if cells_added >= cells_needed {
                break 'outer;
            }
        }
    }

    // Calculate which cell this unit belongs to
    let mut units_counted = 0;
    for (cell_idx, (row, col)) in defender_cells.iter().enumerate() {
        let units_in_this_cell = units_per_cell[cell_idx];
        if unit_index < units_counted + units_in_this_cell {
            // This unit goes in this cell
            let (spawn_x, spawn_z) = calculate_defender_grid_position(*row, *col);
            let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

            let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;
            let spawn_pos = Vec2::new(spawn_x, spawn_z);

            let anim = WalkingAnimation::default();
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                infantry_assets.sprite_texture.clone(),
                DEFENDER_SPRITE_TINT,
            );

            commands
                .spawn((
                    Mesh3d(infantry_assets.sprite_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed(UNIT_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Defenders,
                    Infantry,
                ))
                .insert((
                    anim,
                    FacingDirection::default(),
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    FlowFieldInfluence::Defender { spawn_pos },
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a single attacker infantry unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_attacker(
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    level: u32,
) {
    let total_units = calculate_total_infantry(level);
    let num_cells = cells_needed(total_units);
    let units_per_cell = distribute_units_to_cells(total_units);

    // Calculate spawn cells
    let infantry_cells_needed = num_cells;
    let total_archers = calculate_total_archers(level);
    let archer_cells_needed = cells_needed(total_archers);
    let (infantry_cells, _) = calculate_spawn_cells(infantry_cells_needed, archer_cells_needed);

    // Calculate which cell this unit belongs to
    let mut units_counted = 0;
    for (cell_idx, (row, col)) in infantry_cells.iter().enumerate() {
        if cell_idx >= units_per_cell.len() {
            break;
        }
        let units_in_this_cell = units_per_cell[cell_idx];
        if unit_index < units_counted + units_in_this_cell {
            // This unit goes in this cell
            let (spawn_x, spawn_z) = calculate_grid_cell_position(*row, *col);
            let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

            let hitbox = Hitbox::new(UNIT_RADIUS, ATTACKER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            let anim = WalkingAnimation::default();
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                infantry_assets.sprite_texture.clone(),
                ATTACKER_SPRITE_TINT,
            );

            commands
                .spawn((
                    Mesh3d(infantry_assets.sprite_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed(UNIT_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Attackers,
                    Infantry,
                ))
                .insert((
                    anim,
                    FacingDirection::default(),
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    FlowFieldInfluence::Attacker,
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a single king's guard unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_kings_guard(
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    guard_index: u32,
) {
    // King spawns at centroid_x + 100, centroid_z
    let centroid_x = (-1700.0 + -1400.0 + -1700.0 + -1400.0) / 4.0;
    let centroid_z = (1200.0 + 1200.0 + 1500.0 + 1500.0) / 4.0;
    let spawn_x = centroid_x + 100.0;
    let spawn_z = centroid_z;

    let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    // Calculate position in orbit around king
    let angle = guard_index as f32 * (std::f32::consts::TAU / KINGS_GUARD_COUNT as f32);
    let final_x = spawn_x + KINGS_GUARD_ORBIT_RADIUS * angle.cos();
    let final_z = spawn_z + KINGS_GUARD_ORBIT_RADIUS * angle.sin();

    let anim = WalkingAnimation::default();
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        infantry_assets.sprite_texture.clone(),
        KINGS_GUARD_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(infantry_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, spawn_y, final_z),
            hitbox,
            Health::new(UNIT_HEALTH),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Defenders,
            Infantry,
            KingsGuard(guard_index),
        ))
        .insert((
            anim,
            FacingDirection::default(),
            Teleportable,
            Billboard,
            OnGameplayScreen,
            // King's Guard are all elites
            EliteHealthBonus(crate::game::units::elite::ELITE_HEALTH_BONUS),
            EliteDamageBonus(crate::game::units::elite::ELITE_DAMAGE_BONUS),
            EliteSpeedBonus(crate::game::units::elite::ELITE_SPEED_BONUS),
        ));
}
