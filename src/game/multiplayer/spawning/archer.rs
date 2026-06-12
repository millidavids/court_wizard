//! Multiplayer archer spawn helper.

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, WaveGroup};
use crate::game::units::archer::components::{ArcherMovementTimer, AttackRange};
use crate::game::units::archer::constants::ARCHER_RADIUS;
use crate::game::units::archer::constants::{
    ARCHER_MAX_RANGE, ARCHER_MIN_RANGE, ARCHER_MOVEMENT_SPEED,
};
use crate::game::units::archer::{Archer, ArcherAssets};
use crate::game::units::components::{
    Effectiveness, FacingDirection, FlockingVelocity, Health, Hitbox, MovementSpeed,
    TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::random_position_in_cell;

use super::super::components::OnMultiplayerGameScreen;
use super::utils::staggered_attack_timing;

/// Spawns a single archer unit for multiplayer.
pub(in crate::game::multiplayer) fn spawn_mp_archer(
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    team: Team,
    host_side: bool,
) {
    // Position archers one row behind infantry
    let infantry_cells = cells_needed(MP_INFANTRY_COUNT);
    let infantry_rows = infantry_cells.div_ceil(DEFENDER_GRID_COLS);
    let last_infantry_row = DEFENDER_GRID_ROWS.saturating_sub(infantry_rows);
    let archer_row = last_infantry_row.saturating_sub(1);

    let archer_cells_needed = cells_needed(MP_ARCHER_COUNT);
    let units_per_cell = distribute_units_to_cells(MP_ARCHER_COUNT);

    let mut units_counted = 0;
    for cell_idx in 0..archer_cells_needed.min(DEFENDER_GRID_COLS) {
        let units_in_this_cell = units_per_cell[cell_idx as usize];
        if unit_index < units_counted + units_in_this_cell {
            let (spawn_x, spawn_z) = if host_side {
                calculate_mp_defender_grid_position(archer_row, cell_idx)
            } else {
                calculate_mp_guest_defender_grid_position(archer_row, cell_idx)
            };
            let mut rng = rand::rng();
            let (final_x, final_z) = random_position_in_cell(&mut rng, spawn_x, spawn_z);

            let hitbox = Hitbox::new(ARCHER_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            let tint = crate::game::units::systems::archer_sprite_tint_for_team(team);
            let anim = WalkingAnimation::default();
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                archer_assets.sprite_texture.clone(),
                tint,
            );

            let flow_field = if host_side {
                FlowFieldInfluence::Defender {
                    spawn_pos: Vec2::new(spawn_x, spawn_z),
                }
            } else {
                FlowFieldInfluence::Attacker
            };

            let mut ec = commands.spawn((
                Mesh3d(archer_assets.sprite_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(final_x, spawn_y, final_z),
                crate::game::components::Velocity::default(),
                crate::game::components::Acceleration::new(),
                hitbox,
                Health::new(UNIT_HEALTH),
                MovementSpeed(ARCHER_MOVEMENT_SPEED),
                staggered_attack_timing(),
                Effectiveness::new(),
                team,
                Archer,
            ));
            ec.insert((
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
                flow_field,
                crate::game::units::components::FlockingModifier::new(1.0, 1.0, 0.0),
                Teleportable,
                Billboard,
                OnMultiplayerGameScreen,
            ));
            if team == Team::Attackers {
                ec.insert(WaveGroup(0));
            }
            return;
        }
        units_counted += units_in_this_cell;
    }
}
