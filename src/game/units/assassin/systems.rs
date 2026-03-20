use bevy::prelude::*;

use super::components::Assassin;
use super::constants::*;
use super::resources::AssassinAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    calculate_grid_cell_position, calculate_spawn_cells, calculate_total_assassins,
    calculate_total_infantry, cells_needed, distribute_units_to_cells, ATTACKER_HITBOX_HEIGHT,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::archer::Archer;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, Corpse, Effectiveness, FacingDirection,
    FlockingVelocity, Health, Hitbox, MovementSpeed, TargetingVelocity, Team,
    Teleportable, WalkingAnimation,
};
use crate::game::units::king::components::King;
use crate::game::units::random_position_in_cell;

/// Updates assassin targeting velocity.
///
/// Assassins specifically target archers. If no archers are alive, they fall back
/// to targeting the King. Direct targeting takes over from the flow field when
/// the assassin gets close enough (within TARGETING_CROSSOVER_DISTANCE).
pub fn update_assassin_targeting(
    mut assassins: Query<
        (
            &Transform,
            &Team,
            &mut TargetingVelocity,
        ),
        (With<Assassin>, Without<Corpse>),
    >,
    archers: Query<
        (Entity, &Transform, &Team),
        (With<Archer>, Without<Corpse>, Without<BanishedModifier>),
    >,
    kings: Query<
        (Entity, &Transform, &Team),
        (With<King>, Without<Corpse>),
    >,
) {
    for (transform, team, mut targeting_velocity) in &mut assassins {
        let pos = transform.translation;

        // Find nearest enemy archer
        let nearest_archer = archers
            .iter()
            .filter(|(_, _, archer_team)| team.is_enemy(archer_team))
            .map(|(_, archer_transform, _)| {
                let diff = archer_transform.translation - pos;
                let dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
                (dist, archer_transform.translation)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Find nearest enemy king as fallback
        let nearest_king = kings
            .iter()
            .filter(|(_, _, king_team)| team.is_enemy(king_team))
            .map(|(_, king_transform, _)| {
                let diff = king_transform.translation - pos;
                let dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
                (dist, king_transform.translation)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Prefer archers, fall back to king
        let target = nearest_archer.or(nearest_king);

        if let Some((distance, target_pos)) = target {
            targeting_velocity.distance_to_target = distance;

            if distance < TARGETING_CROSSOVER_DISTANCE {
                // Close enough — direct targeting takes over
                let direction = (target_pos - pos).normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            } else {
                // Far away — let flow field guide, minimal targeting
                targeting_velocity.velocity = Vec3::ZERO;
            }

        } else {
            // No targets at all
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
        }
    }
}

/// Assassin movement system.
///
/// Uses shared weighted movement but with the assassin's fast speed.
#[allow(clippy::type_complexity)]
pub fn assassin_movement(
    time: Res<Time>,
    mut assassin_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&crate::game::units::components::CommanderAuraSpeedModifier>,
            Option<&crate::game::units::components::RoughTerrainModifier>,
            Option<&crate::game::units::components::SlowMovementModifier>,
            (
                Option<&crate::game::cauldron::components::CauldronSpeedModifier>,
                Option<&crate::game::units::components::RootedModifier>,
                Option<&crate::game::units::components::HasteModifier>,
                Option<&crate::game::units::components::EliteSpeedBonus>,
            ),
            (
                Has<crate::game::units::components::SleepModifier>,
                Has<crate::game::units::components::Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&crate::game::units::components::PolymorphedModifier>,
                Option<&crate::game::units::components::SickenedModifier>,
                Option<&crate::game::units::components::FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
            ),
        ),
        With<Assassin>,
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
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned),
    ) in &mut assassin_units
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(
            rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned,
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

        // Use shared weighted movement
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            effectiveness,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            false, // Assassins never slow down in melee
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );
    }
}

/// Spawns a single attacker assassin at a specific index.
/// Assassins spawn behind infantry rows (further from defenders).
pub(in crate::game) fn spawn_single_attacker_assassin(
    commands: &mut Commands,
    assassin_assets: &AssassinAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    level: u32,
) {
    let total_infantry = calculate_total_infantry(level);
    let infantry_cells_needed = cells_needed(total_infantry);
    // Assassins need their own cells behind infantry
    let assassin_cells_needed = cells_needed(calculate_total_assassins(level));

    // Get infantry cell layout to find where to put assassins behind them
    let (infantry_cells, _) = calculate_spawn_cells(infantry_cells_needed, 0);
    let last_infantry_row = infantry_cells.iter().map(|&(r, _)| r).max().unwrap_or(0);

    // Assassins spawn 2 rows behind the last infantry row
    let assassin_row = last_infantry_row + 2;
    let col_fill_order: [u32; 6] = [2, 3, 1, 4, 0, 5];
    let units_per_cell = distribute_units_to_cells(calculate_total_assassins(level));

    let mut units_counted = 0;
    for cell_idx in 0..assassin_cells_needed.min(6) {
        if cell_idx as usize >= units_per_cell.len() {
            break;
        }
        let units_in_this_cell = units_per_cell[cell_idx as usize];
        if unit_index < units_counted + units_in_this_cell {
            let col = col_fill_order[cell_idx as usize];
            let (spawn_x, spawn_z) = calculate_grid_cell_position(assassin_row, col);
            let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

            let hitbox = Hitbox::new(ASSASSIN_RADIUS, ATTACKER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            let anim = WalkingAnimation::default();

            // Create translucent material using archer sprite (alpha in tint color)
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                assassin_assets.sprite_texture.clone(),
                ASSASSIN_SPRITE_TINT,
            );

            commands
                .spawn((
                    Mesh3d(assassin_assets.sprite_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(ASSASSIN_HEALTH),
                    MovementSpeed(ASSASSIN_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Attackers,
                    Assassin,
                ))
                .insert((
                    anim,
                    FacingDirection::default(),
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    FlowFieldInfluence::Assassin,
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}
