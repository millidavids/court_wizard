//! Geometry helpers and spawn utilities for placed fire walls.

use super::super::components::{WallOfFireSfx, WallOfFireTalentParams};
use super::super::constants;
use crate::config::GameConfig;
use crate::game::pathfinding::OBSTACLE_BUFFER;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

/// Computes the axis-aligned bounding box of a rotated wall, expanded by the obstacle buffer.
///
/// The wall is defined by its start/end points and half-width. The AABB covers the
/// rotated rectangle plus a buffer zone so units start rerouting before reaching it.
pub(crate) fn wall_obstacle_bounds(start: Vec3, end: Vec3, half_width: f32) -> Rect {
    let a = Vec2::new(start.x, start.z);
    let b = Vec2::new(end.x, end.z);
    let dir = b - a;
    let perp = Vec2::new(-dir.y, dir.x).normalize_or_zero() * half_width;

    // Four corners of the rotated rectangle
    let c0 = a + perp;
    let c1 = a - perp;
    let c2 = b + perp;
    let c3 = b - perp;

    let min_x = c0.x.min(c1.x).min(c2.x).min(c3.x);
    let max_x = c0.x.max(c1.x).max(c2.x).max(c3.x);
    let min_y = c0.y.min(c1.y).min(c2.y).min(c3.y);
    let max_y = c0.y.max(c1.y).max(c2.y).max(c3.y);

    // Expand by obstacle buffer so units start rerouting before reaching the wall
    Rect::new(
        min_x - OBSTACLE_BUFFER,
        min_y - OBSTACLE_BUFFER,
        max_x + OBSTACLE_BUFFER,
        max_y + OBSTACLE_BUFFER,
    )
}

/// Computes the Transform for a wall entity given its start/end points and half-width.
pub(crate) fn wall_transform(start: Vec3, end: Vec3, half_width: f32) -> Transform {
    let wall_dir = (end - start).normalize_or_zero();
    let wall_len = start.distance(end);
    let center = start + wall_dir * (wall_len / 2.0);
    let rotation = Quat::from_rotation_arc(Vec3::X, wall_dir);
    Transform::from_xyz(
        center.x,
        super::super::constants::WALL_RENDER_HEIGHT / 2.0 + 1.0,
        center.z,
    )
    .with_rotation(rotation)
    .with_scale(Vec3::new(
        wall_len,
        super::super::constants::WALL_RENDER_HEIGHT,
        half_width * 2.0,
    ))
}

/// Spawns fire sparks and looping SFX along a wall segment.
pub(crate) fn spawn_wall_vfx(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    start: Vec3,
    end: Vec3,
    wall_entity: Entity,
) {
    let wall_dir = (end - start).normalize_or_zero();
    let wall_len = start.distance(end);
    let spark_points = 4;
    let t_secs = start.x * 0.01;
    for j in 0..spark_points {
        let frac = (j as f32 + 0.5) / spark_points as f32;
        let pos = start + wall_dir * (wall_len * frac);
        vfx::systems::spawn_fire_sparks(
            commands,
            visual_assets,
            pos,
            vfx::constants::SPARK_COUNT / 2,
            t_secs + j as f32,
        );
    }

    let midpoint = (start + end) / 2.0;
    let sfx_entity = audio::play_looping_sfx_at(
        commands,
        &sfx.wall_of_fire_persistent,
        midpoint,
        game_config,
        sfx,
    );
    commands
        .entity(sfx_entity)
        .insert(WallOfFireSfx { wall_entity });
}

/// Data returned by shared logic so the wrapper can decide what to do with preview/mouse.
pub(crate) struct WallOfFireCastResult {
    /// Whether the spell completed successfully (fire wall was placed).
    pub(crate) completed: bool,
    /// Whether the cast was released but failed (too short / can't afford) — preview should be despawned.
    pub(crate) despawn_preview: bool,
    /// If the wall was placed, stores the wall segment info so local wrapper can convert preview.
    pub(crate) wall_placed: Option<WallPlacedInfo>,
}

/// Info about a successfully placed fire wall, used by the local wrapper to convert preview.
pub(crate) struct WallPlacedInfo {
    pub(crate) wall_start: Vec3,
    pub(crate) wall_end: Vec3,
    pub(crate) half_width: f32,
    pub(crate) damage: f32,
    pub(crate) fire_duration: f32,
    pub(crate) talent_params: WallOfFireTalentParams,
}

/// Computes talent parameters from the player's active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> WallOfFireTalentParams {
    let mut params = WallOfFireTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::WallOfFire, 0);
    let t2 = talents.get_selection(Spell::WallOfFire, 1);
    let t3 = talents.get_selection(Spell::WallOfFire, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => params.damage_mult = constants::INFERNAL_INTENSITY_DAMAGE_MULT,
        Some(1) => {
            params.width_mult = constants::FIREBREAK_WIDTH_MULT;
            params.duration_mult = constants::FIREBREAK_DURATION_MULT;
        }
        Some(2) => {
            params.max_length_mult = constants::FLASH_FIRE_MAX_LENGTH_MULT;
            params.damage_mult = constants::FLASH_FIRE_DAMAGE_MULT;
            params.duration_mult = constants::FLASH_FIRE_DURATION_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.searing_heat = true,
        Some(1) => params.scorched_earth = true,
        Some(2) => params.spreading_flames = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.firestorm = true,
        Some(1) => params.twin_walls = true,
        Some(2) => params.consuming_inferno = true,
        _ => {}
    }

    params
}
