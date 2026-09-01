use super::super::components::{
    LivingStoneTracker, WallHealth, WallOfStone, WallOfStoneTalentParams, WallTalents,
};
use super::super::constants::*;
use crate::config::save_data::SavedWall;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Talent state a carried-over wall needs to behave like the wall it was.
///
/// Exactly the set of `WallOfStoneTalentParams` fields that anything reads
/// *after* spawn — the rest (`mana_mult`, `max_length_mult`, `quick_foundations`)
/// only matter while casting, and `width_mult` is already baked into the saved
/// half-extents so re-applying it would widen the wall on every level.
///
/// Deliberately has no `Default`: a zeroed `health_mult` would spawn a wall with
/// 0 HP. Build it with [`PermanentWallTalents::from_params`].
#[derive(Clone, Copy)]
pub(crate) struct PermanentWallTalents {
    pub health_mult: f32,
    pub jagged_stone: bool,
    pub permafrost_aura: bool,
    pub living_stone: bool,
    pub collapsing_wall: bool,
    pub maze_architect: bool,
}

impl PermanentWallTalents {
    pub(crate) fn from_params(params: &WallOfStoneTalentParams) -> Self {
        Self {
            health_mult: params.health_mult,
            jagged_stone: params.jagged_stone,
            permafrost_aura: params.permafrost_aura,
            living_stone: params.living_stone,
            collapsing_wall: params.collapsing_wall,
            maze_architect: params.maze_architect,
        }
    }
}

/// Spawns a permanent wall entity from saved wall data.
pub(crate) fn spawn_permanent_wall(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    saved: &SavedWall,
    talents: PermanentWallTalents,
) {
    let forward = Vec3::new(saved.forward_x, 0.0, saved.forward_z);
    let right = Vec3::new(-forward.z, 0.0, forward.x);
    let center = Vec3::new(saved.center_x, 0.0, saved.center_z);
    let rotation = Quat::from_rotation_arc(Vec3::X, forward);

    // Rebuild the talent params this wall runs on. Width is intentionally left
    // at 1.0: the saved half-extents already include it from placement time.
    let wall_talents = WallOfStoneTalentParams {
        health_mult: talents.health_mult,
        jagged_stone: talents.jagged_stone,
        permafrost_aura: talents.permafrost_aura,
        living_stone: talents.living_stone,
        collapsing_wall: talents.collapsing_wall,
        maze_architect: talents.maze_architect,
        terraformer: true,
        ..Default::default()
    };

    let mut entity_commands = commands.spawn((
        Mesh3d(assets.unit_cuboid.clone()),
        MeshMaterial3d(assets.wall_of_stone.clone()),
        Transform::from_xyz(center.x, saved.height / 2.0, center.z)
            .with_rotation(rotation)
            .with_scale(Vec3::new(
                saved.half_length * 2.0,
                saved.height,
                saved.half_width * 2.0,
            )),
        WallOfStone {
            center,
            half_length: saved.half_length,
            half_width: saved.half_width,
            forward,
            right,
            height: saved.height,
            time_alive: 0.0,
            duration: f32::MAX,
            sinking: false,
            empowerment: saved.empowerment,
            permanent: true,
        },
        WallHealth::new(WALL_HEALTH * talents.health_mult),
        WallTalents(wall_talents),
        NetworkedSpellEffect {
            kind: SpellEffectKind::WallOfStone,
        },
        OnGameplayScreen,
    ));

    // Matches what placement.rs inserts for a freshly cast Living Stone wall.
    if talents.living_stone {
        entity_commands.insert(LivingStoneTracker::new());
    }
}

/// Registers pathfinding obstacles for all permanent walls after loading completes.
pub(crate) fn register_permanent_wall_obstacles(
    walls: Query<&WallOfStone>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for wall in &walls {
        if !wall.permanent {
            continue;
        }
        let obs_bounds = wall.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
            obstacle_type: ObstacleType::Blocked,
            shape: Some(ObstacleShape::obb_from_center(
                wall.center,
                wall.forward,
                wall.half_length,
                wall.half_width,
            )),
            rebuild: false,
        });
    }
}
