use bevy::prelude::*;

use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::InfantryAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    calculate_defender_grid_position, cells_needed, distribute_units_to_cells, *,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::components::{
    AttackTiming, Effectiveness, EliteSpeedBonus, FacingDirection, FlockingVelocity, Health,
    Hitbox, KingsGuard, MovementSpeed, TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::elite::{EliteAttackSpeedBonus, EliteDamageBonus, EliteHealthBonus};
use crate::game::units::random_position_in_cell;

/// Spawns a single defender infantry unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_defender(
    rng: &mut impl Rng,
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
            let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

            let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;
            let spawn_pos = Vec2::new(spawn_x, spawn_z);

            let anim = WalkingAnimation::new_staggered(rng);
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
    rng: &mut impl Rng,
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    _level: u32,
) -> Entity {
    let (spawn_x, spawn_z) = attacker_spawn_position(unit_index, 0.0);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(UNIT_RADIUS, ATTACKER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let anim = WalkingAnimation::new_staggered(rng);
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
        ))
        .id()
}

/// Spawns a single king's guard unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_kings_guard(
    rng: &mut impl Rng,
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

    let anim = WalkingAnimation::new_staggered(rng);
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
            // `Velocity` is required by `update_walking_animation` and
            // `update_facing_direction`. The guard's position is driven by
            // `snap_kings_guard_to_king` rather than the normal movement
            // pipeline; that system writes the per-frame snap delta into
            // this `Velocity` so the animation/facing queries see motion.
            Velocity::default(),
            // King's Guard are all elites
            EliteHealthBonus(crate::game::units::elite::ELITE_HEALTH_BONUS),
            EliteDamageBonus(crate::game::units::elite::ELITE_DAMAGE_BONUS),
            EliteSpeedBonus(crate::game::units::elite::ELITE_SPEED_BONUS),
            EliteAttackSpeedBonus(crate::game::units::elite::ELITE_ATTACK_SPEED_BONUS),
        ));
}
