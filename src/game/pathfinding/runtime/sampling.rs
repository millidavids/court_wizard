use bevy::prelude::*;

use crate::game::constants::CENTER_STAGING_INDEX;
use crate::game::units::infantry::components::DefendersActivated;

use crate::game::pathfinding::components::{
    FlowFieldInfluence, FlowFieldVelocity, StagingAttacker, WaveGroup,
};
use crate::game::pathfinding::resources::PathfindingGrid;
use crate::game::units::components::Team;

use super::flow_field_rebuild::DEFENDER_SPAWN_RALLY_RADIUS;
use crate::game::pathfinding::setup::CELL_SIZE;

/// Samples flow fields and updates FlowFieldVelocity for all units.
///
/// Runs before movement systems to provide flow field guidance.
pub fn sample_flow_fields(
    pathfinding: Res<PathfindingGrid>,
    defenders_activated: Res<DefendersActivated>,
    mut units_query: Query<(
        &Transform,
        &FlowFieldInfluence,
        &mut FlowFieldVelocity,
        Option<&StagingAttacker>,
        Has<WaveGroup>,
        &Team,
    )>,
) {
    for (transform, influence, mut flow_velocity, staging, has_wave_group, team) in
        units_query.iter_mut()
    {
        let world_pos = transform.translation;

        // Sample terrain cost from base costs (always up-to-date)
        flow_velocity.terrain_cost = pathfinding.sample_base_cost(world_pos);

        let wmin = pathfinding.world_min;
        let cs = pathfinding.cell_size;

        // Attackers use the staging field until activated:
        // - has StagingAttacker component, OR
        // - doesn't have WaveGroup yet (1-frame command delay after spawn)
        let is_staging = crate::game::units::systems::is_staging_attacker(
            team,
            staging.is_some(),
            has_wave_group,
        );

        match influence {
            FlowFieldInfluence::Attacker | FlowFieldInfluence::Assassin if is_staging => {
                // Staging: follow the staging field for this unit's assigned staging point
                let field = staging
                    .map(|s| &pathfinding.staging_fields[s.0 as usize])
                    .unwrap_or(&pathfinding.staging_fields[CENTER_STAGING_INDEX]);
                let (vel, dist) = sample_field_or_zero(field, world_pos, wmin, cs);
                flow_velocity.velocity = vel;
                flow_velocity.pathfinding_distance = dist;
                flow_velocity.at_destination = vel == Vec3::ZERO;
            }
            FlowFieldInfluence::Attacker => {
                // Activated: follow attacker field toward King
                flow_velocity.at_destination = false;
                let (vel, dist) =
                    sample_field_or_zero(&pathfinding.attacker_field, world_pos, wmin, cs);
                flow_velocity.velocity = vel;
                flow_velocity.pathfinding_distance = dist;
            }
            FlowFieldInfluence::Defender { spawn_pos } => {
                if defenders_activated.active {
                    if let Some(ref field) = pathfinding.defender_field {
                        let direction = field.sample(world_pos, wmin, cs);
                        flow_velocity.velocity = direction;
                        flow_velocity.pathfinding_distance =
                            field.sample_distance(world_pos, wmin, cs);
                        flow_velocity.at_destination = direction == Vec3::ZERO;
                    } else {
                        rally_to_spawn(world_pos, spawn_pos, &mut flow_velocity);
                    }
                } else {
                    rally_to_spawn(world_pos, spawn_pos, &mut flow_velocity);
                }
            }
            FlowFieldInfluence::Assassin => {
                // Activated: follow assassin field, fall back to attacker field
                flow_velocity.at_destination = false;
                let field_ref = if pathfinding.assassin_field.is_some() {
                    &pathfinding.assassin_field
                } else {
                    &pathfinding.attacker_field
                };
                let (vel, dist) = sample_field_or_zero(field_ref, world_pos, wmin, cs);
                flow_velocity.velocity = vel;
                flow_velocity.pathfinding_distance = dist;
            }
        }
    }
}

/// Samples velocity and distance from a flow field, returning zeroes if the field is None.
fn sample_field_or_zero(
    field: &Option<crate::game::pathfinding::flow_field::FlowField>,
    world_pos: Vec3,
    world_min: Vec2,
    cell_size: f32,
) -> (Vec3, f32) {
    if let Some(f) = field {
        (
            f.sample(world_pos, world_min, cell_size),
            f.sample_distance(world_pos, world_min, cell_size),
        )
    } else {
        (Vec3::ZERO, f32::INFINITY)
    }
}

/// Moves a defender toward its spawn position with a satisfaction radius.
fn rally_to_spawn(world_pos: Vec3, spawn_pos: &Vec2, flow_velocity: &mut FlowFieldVelocity) {
    let current_pos = Vec2::new(world_pos.x, world_pos.z);
    let distance_to_spawn = current_pos.distance(*spawn_pos);
    let satisfaction_radius_world = DEFENDER_SPAWN_RALLY_RADIUS as f32 * CELL_SIZE;

    if distance_to_spawn > satisfaction_radius_world {
        let direction = (*spawn_pos - current_pos).normalize();
        flow_velocity.velocity = Vec3::new(direction.x, 0.0, direction.y);
        flow_velocity.at_destination = false;
    } else {
        flow_velocity.velocity = Vec3::ZERO;
        flow_velocity.at_destination = true;
    }
    flow_velocity.pathfinding_distance = distance_to_spawn;
}
