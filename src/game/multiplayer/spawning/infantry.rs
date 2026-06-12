//! Multiplayer infantry and king's guard spawn helpers.

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, WaveGroup};
use crate::game::units::components::{
    Effectiveness, FacingDirection, FlockingVelocity, Health, Hitbox, MovementSpeed,
    TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::infantry::Infantry;
use crate::game::units::infantry::constants::UNIT_RADIUS;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::random_position_in_cell;

use super::super::components::OnMultiplayerGameScreen;
use super::utils::staggered_attack_timing;

/// Spawns a single infantry unit for multiplayer.
///
/// `host_side` = true spawns near Castle 1 using standard grid positions.
/// `host_side` = false spawns near Castle 2 using mirrored grid positions.
pub(in crate::game::multiplayer) fn spawn_mp_infantry(
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    team: Team,
    host_side: bool,
) {
    let total_units = MP_INFANTRY_COUNT;
    let cells_needed = cells_needed(total_units);
    let units_per_cell = distribute_units_to_cells(total_units);

    // Use defender grid layout (units defend near their castle)
    let mut cells = Vec::new();
    let mut cells_added = 0;
    'outer: for row in (0..DEFENDER_GRID_ROWS).rev() {
        for col in 0..DEFENDER_GRID_COLS {
            cells.push((row, col));
            cells_added += 1;
            if cells_added >= cells_needed {
                break 'outer;
            }
        }
    }

    let mut units_counted = 0;
    for (cell_idx, (row, col)) in cells.iter().enumerate() {
        let units_in_this_cell = units_per_cell[cell_idx];
        if unit_index < units_counted + units_in_this_cell {
            let (spawn_x, spawn_z) = if host_side {
                calculate_mp_defender_grid_position(*row, *col)
            } else {
                calculate_mp_guest_defender_grid_position(*row, *col)
            };
            let mut rng = rand::rng();
            let (final_x, final_z) = random_position_in_cell(&mut rng, spawn_x, spawn_z);

            let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;
            let spawn_pos = Vec2::new(spawn_x, spawn_z);

            let tint = crate::game::units::systems::sprite_tint_for_team(team);
            let anim = WalkingAnimation::default();
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                infantry_assets.sprite_texture.clone(),
                tint,
            );

            let flow_field = if host_side {
                FlowFieldInfluence::Defender { spawn_pos }
            } else {
                FlowFieldInfluence::Attacker
            };

            let mut ec = commands.spawn((
                Mesh3d(infantry_assets.sprite_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(final_x, spawn_y, final_z),
                crate::game::components::Velocity::default(),
                crate::game::components::Acceleration::new(),
                hitbox,
                Health::new(UNIT_HEALTH),
                MovementSpeed(UNIT_MOVEMENT_SPEED),
                staggered_attack_timing(),
                Effectiveness::new(),
                team,
                Infantry,
            ));
            ec.insert((
                anim,
                FacingDirection::default(),
                TargetingVelocity::default(),
                FlockingVelocity::default(),
                FlowFieldVelocity::default(),
                flow_field,
                Teleportable,
                Billboard,
                OnMultiplayerGameScreen,
            ));
            // MP attackers are pre-activated — no staging phase. `WaveGroup(0)`
            // marks them as already-tagged so `is_staging_attacker` returns
            // false (otherwise the missing `WaveGroup` would have implicitly
            // classed them as staging, and dispeller/wave-speedup logic
            // would treat them as inactive). Predicate on `team` rather than
            // `host_side` so future spawn paths that decouple the two don't
            // silently break the staging guard.
            if team == Team::Attackers {
                ec.insert(WaveGroup(0));
            }
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a King's Guard unit for multiplayer at the given position origin.
pub(in crate::game::multiplayer) fn spawn_mp_kings_guard(
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    guard_index: u32,
    wizard_position: Vec3,
    center_angle: f32,
    team: Team,
) {
    use crate::game::units::components::KingsGuard;
    use crate::game::units::elite::{EliteDamageBonus, EliteHealthBonus, EliteSpeedBonus};
    use crate::game::units::infantry::constants::KINGS_GUARD_SPRITE_TINT;

    // King's position: same calculation as spawn_mp_king
    let radius = MP_DEFENDER_GRID_GROUND_RANGE + 600.0;
    let king_x = wizard_position.x + radius * center_angle.cos();
    let king_z = wizard_position.z + radius * center_angle.sin();

    let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let angle = guard_index as f32 * (std::f32::consts::TAU / KINGS_GUARD_COUNT as f32);
    let final_x = king_x + KINGS_GUARD_ORBIT_RADIUS * angle.cos();
    let final_z = king_z + KINGS_GUARD_ORBIT_RADIUS * angle.sin();

    let anim = WalkingAnimation::default();
    let guard_material = crate::game::units::systems::create_default_sprite_material(
        materials,
        infantry_assets.sprite_texture.clone(),
        KINGS_GUARD_SPRITE_TINT,
    );

    let mut ec = commands.spawn((
        Mesh3d(infantry_assets.sprite_mesh.clone()),
        MeshMaterial3d(guard_material),
        Transform::from_xyz(final_x, spawn_y, final_z),
        // `Velocity` is required so the entity passes the host's
        // `send_state_snapshots` query (`&Velocity`). Without it King's
        // Guards silently fall out of every snapshot → guest never sees
        // them and they appear invincible/missing.
        crate::game::components::Velocity::default(),
        hitbox,
        Health::new(UNIT_HEALTH),
        staggered_attack_timing(),
        Effectiveness::new(),
        team,
        Infantry,
        KingsGuard(guard_index),
    ));
    ec.insert((
        anim,
        FacingDirection::default(),
        Teleportable,
        Billboard,
        OnMultiplayerGameScreen,
        EliteHealthBonus(crate::game::units::elite::ELITE_HEALTH_BONUS),
        EliteDamageBonus(crate::game::units::elite::ELITE_DAMAGE_BONUS),
        EliteSpeedBonus(crate::game::units::elite::ELITE_SPEED_BONUS),
        crate::game::units::elite::EliteAttackSpeedBonus(
            crate::game::units::elite::ELITE_ATTACK_SPEED_BONUS,
        ),
    ));
    if team == Team::Attackers {
        ec.insert(WaveGroup(0));
    }
}
