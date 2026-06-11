//! Grid initialization and static terrain/obstacle baking.

use bevy::prelude::*;

use crate::game::constants::{
    BATTLEFIELD_SIZE, PATHFINDING_X_EXTENSION, STAGING_POINT_COUNT, STAGING_POINTS,
    STAGING_SATISFACTION_RADIUS,
};
use crate::game::units::components::Team;
use crate::game::units::king::components::King;

use super::super::resources::PathfindingGrid;
use super::super::runtime::spawn_attacker_field_rebuild;

/// Cell size for the pathfinding grid (in world units).
pub(crate) const CELL_SIZE: f32 = 10.0;

/// Buffer zone added around obstacles/hazards in the pathfinding grid.
/// Equal to one cell size so units start rerouting one cell before hitting the obstacle.
pub(crate) const OBSTACLE_BUFFER: f32 = CELL_SIZE;

/// Initializes the pathfinding grid resource and registers static terrain obstacles
pub fn initialize_pathfinding(mut commands: Commands) {
    let mut pathfinding =
        PathfindingGrid::new(BATTLEFIELD_SIZE, CELL_SIZE, PATHFINDING_X_EXTENSION);

    info!(
        "Pathfinding initialized: {}x{} grid ({} cells, {} bytes per field)",
        pathfinding.grid_width,
        pathfinding.grid_height,
        pathfinding.grid_width * pathfinding.grid_height,
        pathfinding.grid_width * pathfinding.grid_height * std::mem::size_of::<Vec3>()
    );

    // Register static terrain on the base costs directly (no need for messages
    // since the grid isn't inserted yet — messages would be processed next frame).
    register_static_terrain(&mut pathfinding);

    // Build all 7 staging flow fields once (never change — each targets one staging point).
    build_staging_fields(&mut pathfinding);

    commands.insert_resource(pathfinding);
}

/// Builds static staging flow fields for all staging points.
/// Each field guides unactivated attackers from the spawn area to one staging point.
fn build_staging_fields(pathfinding: &mut PathfindingGrid) {
    let world_min = pathfinding.world_min;
    let cell_size = pathfinding.cell_size;

    for (i, &(sx, sz)) in STAGING_POINTS.iter().enumerate() {
        let mut field = pathfinding.create_field_with_base_costs();

        let goal_x = ((sx - world_min.x) / cell_size).floor().max(0.0) as usize;
        let goal_z = ((sz - world_min.y) / cell_size).floor().max(0.0) as usize;

        field.generate(goal_x, goal_z, STAGING_SATISFACTION_RADIUS);
        pathfinding.staging_fields[i] = Some(field);
    }
    info!(
        "Built {} staging flow fields, satisfaction_radius={}",
        STAGING_POINT_COUNT, STAGING_SATISFACTION_RADIUS
    );
}

/// Registers lava hazard and water slow terrain directly on the pathfinding
/// grid's base costs. The right wall is purely visual (no flow field collision).
fn register_static_terrain(pathfinding: &mut PathfindingGrid) {
    use crate::game::battlefield::constants::{
        LAVA_AVOIDANCE_RADIUS, LAVA_HAZARD_FLOW_COST, LAVA_POOL_POSITION, WATER_POOL_POSITION,
        WATER_POOL_RADIUS, WATER_SLOW_FLOW_COST,
    };

    register_circular_terrain(
        pathfinding,
        LAVA_POOL_POSITION,
        LAVA_AVOIDANCE_RADIUS,
        LAVA_HAZARD_FLOW_COST,
        "Lava hazard",
    );
    register_circular_terrain(
        pathfinding,
        WATER_POOL_POSITION,
        WATER_POOL_RADIUS,
        WATER_SLOW_FLOW_COST,
        "Water slow terrain",
    );
}

/// Registers a circular terrain hazard on the pathfinding grid's base costs.
fn register_circular_terrain(
    pathfinding: &mut PathfindingGrid,
    position: Vec3,
    radius: f32,
    cost: f32,
    label: &str,
) {
    let center = Vec2::new(position.x, position.z);
    let shape = super::super::messages::ObstacleShape::circle(center, radius);
    let bounds = Rect::new(
        center.x - radius,
        center.y - radius,
        center.x + radius,
        center.y + radius,
    );
    let cells = pathfinding.shape_filtered_cells(bounds, &shape);
    pathfinding.set_terrain_cost(&cells, cost);
    info!("{}: {} cells at cost {}", label, cells.len(), cost);
}

/// Continuously rebuilds all active flow fields in parallel on background threads.
///
/// Each frame, polls pending async tasks and applies completed fields. When a field
/// has no pending rebuild, immediately spawns a new one with fresh target data.
/// This keeps all fields constantly up-to-date without timers or thresholds.
#[allow(clippy::type_complexity)]
pub fn generate_initial_fields(
    mut pathfinding: ResMut<PathfindingGrid>,
    king_query: Query<(&Transform, &Team), (With<King>, Added<Transform>)>,
) {
    // Only run when the Defender King first spawns
    let Some((king_transform, _)) = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
    else {
        return;
    };

    let king_pos = Vec2::new(king_transform.translation.x, king_transform.translation.z);
    pathfinding.last_king_pos = king_pos;

    // Spawn initial attacker field rebuild
    spawn_attacker_field_rebuild(&mut pathfinding, king_pos);
    info!("Generating initial attacker flow field toward Defender King");
}
