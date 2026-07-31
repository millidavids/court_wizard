use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::ArcherAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    calculate_defender_grid_position, cells_needed, distribute_units_to_cells, *,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::components::{
    AttackTiming, Effectiveness, FacingDirection, FlockingModifier, FlockingVelocity, Health,
    Hitbox, MovementSpeed, TargetingVelocity, Teleportable, WalkingAnimation,
};
use crate::game::units::random_position_in_cell;

/// Spawns a single defender archer unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_defender_archer(
    rng: &mut impl Rng,
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
) {
    // Calculate where infantry spawned to determine archer row
    let infantry_cells = cells_needed(INITIAL_DEFENDER_COUNT);
    let infantry_rows = infantry_cells.div_ceil(DEFENDER_GRID_COLS);
    // Infantry start at row (ROWS-1) and fill `infantry_rows` rows, ending at (ROWS-1-infantry_rows+1)
    // Archers go one row lower than that
    let last_infantry_row = DEFENDER_GRID_ROWS.saturating_sub(infantry_rows);
    let archer_row = last_infantry_row.saturating_sub(1);

    let archer_cells_needed = cells_needed(INITIAL_ARCHER_DEFENDER_COUNT);
    let units_per_cell = distribute_units_to_cells(INITIAL_ARCHER_DEFENDER_COUNT);

    // Calculate which cell this unit belongs to
    let mut units_counted = 0;
    for cell_idx in 0..archer_cells_needed.min(DEFENDER_GRID_COLS) {
        let units_in_this_cell = units_per_cell[cell_idx as usize];
        if unit_index < units_counted + units_in_this_cell {
            // This unit goes in this cell
            let (spawn_x, spawn_z) = calculate_defender_grid_position(archer_row, cell_idx);
            let (raw_x, raw_z) = random_position_in_cell(rng, spawn_x, spawn_z);
            // Archers hold the back row, which sits past the playable-area edge.
            // Clamp the spawn point as well as the rally target so they start
            // where they will rest instead of being yanked in on frame one.
            let post = crate::game::movement_systems::clamp_defender_post(Vec2::new(raw_x, raw_z));
            let (final_x, final_z) = (post.x, post.y);

            let hitbox = Hitbox::new(ARCHER_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            let anim = WalkingAnimation::new_staggered(rng);
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                archer_assets.sprite_texture.clone(),
                DEFENDER_SPRITE_TINT,
            );

            commands
                .spawn((
                    Mesh3d(archer_assets.sprite_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed(ARCHER_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    crate::game::units::components::Team::Defenders,
                    Archer,
                ))
                .insert((
                    anim,
                    FacingDirection::default(),
                    AttackRange {
                        min_range: ARCHER_MIN_RANGE,
                        max_range: ARCHER_MAX_RANGE,
                    },
                    ArcherMovementTimer::new(),
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    FlowFieldInfluence::Defender {
                        spawn_pos: crate::game::movement_systems::clamp_defender_post(Vec2::new(
                            spawn_x, spawn_z,
                        )),
                    },
                    FlockingModifier::new(1.0, 1.0, 0.0),
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a single attacker archer unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_attacker_archer(
    rng: &mut impl Rng,
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    _level: u32,
) -> Entity {
    let (spawn_x, spawn_z) = attacker_spawn_position(unit_index, ARCHER_SPAWN_DEPTH_OFFSET);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(ARCHER_RADIUS, ATTACKER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let anim = WalkingAnimation::new_staggered(rng);
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        archer_assets.sprite_texture.clone(),
        ATTACKER_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(archer_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, spawn_y, final_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(UNIT_HEALTH),
            MovementSpeed(ARCHER_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            crate::game::units::components::Team::Attackers,
            Archer,
        ))
        .insert((
            anim,
            FacingDirection::default(),
            AttackRange {
                min_range: ARCHER_MIN_RANGE,
                max_range: ARCHER_MAX_RANGE,
            },
            ArcherMovementTimer::new(),
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
