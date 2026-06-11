use bevy::prelude::*;
use rand::Rng;

use super::super::components::Aerialist;
use super::super::constants::*;
use super::super::resources::AerialistAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    ARCHER_SPAWN_DEPTH_OFFSET, ATTACKER_HITBOX_HEIGHT, attacker_spawn_position,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::components::{
    AttackTiming, Effectiveness, FacingDirection, FlockingVelocity, Flying, Health, Hitbox,
    MovementSpeed, TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::random_position_in_cell;

/// Spawns a single attacker aerialist unit at a specific index.
pub(in crate::game) fn spawn_single_attacker_aerialist(
    rng: &mut impl Rng,
    commands: &mut Commands,
    aerialist_assets: &AerialistAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    _level: u32,
) -> Entity {
    // Spawn with archers (same depth offset)
    let (spawn_x, spawn_z) = attacker_spawn_position(unit_index, ARCHER_SPAWN_DEPTH_OFFSET);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(AERIALIST_RADIUS, ATTACKER_HITBOX_HEIGHT);

    // Flying animation: 2 frames per direction (not the default 9-frame walk cycle)
    use crate::game::units::components::{
        SPRITE_DIRECTION_ROWS, SPRITE_SHEET_IMAGE_HEIGHT, sprite_frame_uv,
    };
    let anim = WalkingAnimation {
        current_frame: 0,
        elapsed: rng.random::<f32>() * 0.125, // stagger start
        columns: AERIALIST_FLYING_FRAMES,
        frame_uv: sprite_frame_uv(SPRITE_SHEET_IMAGE_HEIGHT),
        direction_rows: SPRITE_DIRECTION_ROWS,
    };
    let tint = Color::WHITE;
    let material = crate::game::units::systems::create_sprite_material(
        materials,
        aerialist_assets.sprite_texture.clone(),
        tint,
        anim.frame_uv,
        anim.uv_offset(FacingDirection::default()),
    );

    commands
        .spawn((
            Mesh3d(aerialist_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, AERIALIST_FLY_HEIGHT, final_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(crate::game::constants::UNIT_HEALTH * 1.2),
            MovementSpeed(AERIALIST_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Aerialist,
            Flying,
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
