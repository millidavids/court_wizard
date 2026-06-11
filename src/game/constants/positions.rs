use bevy::prelude::*;

use super::spawn_math::{
    DEFENDER_GRID_CENTER_ANGLE, DEFENDER_GRID_COLS, GRID_ANGULAR_SPACING, GRID_ROW_DEPTH,
};

// ===== Battlefield Dimensions =====

/// Size of the battlefield (width and depth).
pub const BATTLEFIELD_SIZE: f32 = 6000.0;

/// Extra world units added to +X side of pathfinding grid for spawn area behind right wall.
pub const PATHFINDING_X_EXTENSION: f32 = 2000.0;

// ===== Castle Positioning =====

/// Castle position in 3D space (shifted back toward camera along the 45° diagonal).
pub const CASTLE_POSITION: Vec3 = Vec3::new(-1700.0, 1200.0, 1700.0);

/// Castle rotation in degrees.
pub const CASTLE_ROTATION_DEGREES: f32 = 35.0;

/// Castle width — used for the castle wall plane.
pub const CASTLE_WIDTH: f32 = 300.0;

/// Wizard offset from castle position.
/// Y offset = half sprite height so the wizard's feet rest on the castle platform.
pub(crate) const WIZARD_OFFSET: Vec3 = Vec3::new(300.0, 450.0, 0.0);

// ===== Unit Positioning =====

/// Wizard position in 3D space (on castle platform).
/// Calculated as castle position plus offset.
pub const WIZARD_POSITION: Vec3 = Vec3::new(
    CASTLE_POSITION.x + WIZARD_OFFSET.x,
    CASTLE_POSITION.y + WIZARD_OFFSET.y,
    CASTLE_POSITION.z + WIZARD_OFFSET.z,
);

/// Offset from wizard position to place the cauldron beside the wizard on the castle wall.
const SPELL_OFFSET: Vec3 = Vec3::new(100.0, 90.0, 30.0);

/// Spell origin point — where projectiles and beams originate from.
/// Same as wizard position since the wizard doesn't move.
///
/// This is the **host / single-player** origin. In multiplayer the guest's
/// local spells originate from `SPELL_2_ORIGIN` instead. Most code should
/// read the `LocalSpellOrigin` resource rather than this constant directly
/// so it produces the right value on both peers.
pub const SPELL_ORIGIN: Vec3 = Vec3::new(
    WIZARD_POSITION.x + SPELL_OFFSET.x,
    WIZARD_POSITION.y + SPELL_OFFSET.y,
    WIZARD_POSITION.z + SPELL_OFFSET.z,
);

/// Spell origin point for the **multiplayer guest** (mirrors `SPELL_ORIGIN`
/// across the world origin so the guest's spells visibly originate from
/// their own wizard's hand).
pub const SPELL_2_ORIGIN: Vec3 = Vec3::new(
    WIZARD_2_POSITION.x - SPELL_OFFSET.x,
    WIZARD_2_POSITION.y + SPELL_OFFSET.y,
    WIZARD_2_POSITION.z - SPELL_OFFSET.z,
);

// ===== Co-op (second wizard, shared SP battlefield) =====

/// Co-op (second) wizard position — beside the host wizard on the same wall:
/// continue to the right of the host wizard and past the cauldron, at the SAME
/// wall height (Y) and the SAME facing/camera as the host. Unlike versus (which
/// mirrors the guest to the opposite corner), the co-op guest stands beside the
/// host on the single-player battlefield.
///
/// The guest stands on the line from the host wizard through the cauldron,
/// extended the same distance again past the cauldron — i.e. the cauldron is the
/// midpoint between the host and the guest. Since the cauldron sits at
/// `WIZARD_POSITION + CAULDRON_OFFSET` (`+60 X / -64 Y / +90 Z`), the guest lands
/// at `WIZARD_POSITION + 2 × CAULDRON_OFFSET`. The co-op spell origin derives from
/// this, so moving the wizard moves it too.
pub const WIZARD_COOP_POSITION: Vec3 = Vec3::new(
    WIZARD_POSITION.x + 120.0,
    WIZARD_POSITION.y,
    WIZARD_POSITION.z + 180.0,
);

/// Co-op guest spell origin (the tip of their staff). Uses the SAME offset
/// direction as the host's `SPELL_OFFSET` (NOT mirrored) because the co-op
/// guest faces the same way and is viewed from the same camera as the host.
pub const SPELL_COOP_ORIGIN: Vec3 = Vec3::new(
    WIZARD_COOP_POSITION.x + SPELL_OFFSET.x,
    WIZARD_COOP_POSITION.y + SPELL_OFFSET.y,
    WIZARD_COOP_POSITION.z + SPELL_OFFSET.z,
);

// ===== Castle 2 Positioning (Multiplayer — opposite corner) =====

/// Castle 2 position in 3D space (diagonally opposite from Castle 1).
///
/// Exact 180° Y-axis mirror of `CASTLE_POSITION` (x and z negated, y kept).
/// This is required so the guest's mirrored visual battlefield (which
/// rotates Castle 1's mesh 180° around world origin) lands the mesh at the
/// SAME world position as `CASTLE_2_POSITION` — keeping the guest's wizard
/// sprite (in shared world coords) correctly aligned on the castle.
pub const CASTLE_2_POSITION: Vec3 =
    Vec3::new(-CASTLE_POSITION.x, CASTLE_POSITION.y, -CASTLE_POSITION.z);

/// Castle 2 rotation in degrees (facing opposite direction).
pub const CASTLE_2_ROTATION_DEGREES: f32 = CASTLE_ROTATION_DEGREES + 180.0;

/// Wizard 2 position (on Castle 2 platform).
pub const WIZARD_2_POSITION: Vec3 = Vec3::new(
    CASTLE_2_POSITION.x - WIZARD_OFFSET.x,
    CASTLE_2_POSITION.y + WIZARD_OFFSET.y,
    CASTLE_2_POSITION.z - WIZARD_OFFSET.z,
);

// ===== Multiplayer Constants =====

/// Fixed infantry count for each side in multiplayer (no level scaling).
pub const MP_INFANTRY_COUNT: u32 = 60;

/// Fixed archer count for each side in multiplayer.
pub const MP_ARCHER_COUNT: u32 = 10;

/// Fixed "level" passed to the single-player terrain generator for multiplayer
/// matches (multiplayer has no level progression). Scales the count of boulders,
/// trees, ponds, and bushes; tier saturates at level 5.
pub const MP_TERRAIN_LEVEL: u32 = 3;

/// Ground-plane distance from wizard to defender spawn grid in multiplayer.
/// Much closer than single-player so units spawn right against the castles.
pub const MP_DEFENDER_GRID_GROUND_RANGE: f32 = 100.0;

/// Calculates the world position of a defender grid cell for MP host side (Castle 1).
pub fn calculate_mp_defender_grid_position(row: u32, col: u32) -> (f32, f32) {
    let col_offset = col as f32 - (DEFENDER_GRID_COLS as f32 - 1.0) / 2.0;
    let angle = DEFENDER_GRID_CENTER_ANGLE + col_offset * GRID_ANGULAR_SPACING;
    let radius = MP_DEFENDER_GRID_GROUND_RANGE + GRID_ROW_DEPTH / 2.0 + row as f32 * GRID_ROW_DEPTH;
    let x = WIZARD_POSITION.x + radius * angle.cos();
    let z = WIZARD_POSITION.z + radius * angle.sin();
    (x, z)
}

/// Calculates the world position of a guest defender grid cell for MP (Castle 2).
pub fn calculate_mp_guest_defender_grid_position(row: u32, col: u32) -> (f32, f32) {
    let col_offset = col as f32 - (DEFENDER_GRID_COLS as f32 - 1.0) / 2.0;
    let mirrored_angle = DEFENDER_GRID_CENTER_ANGLE + std::f32::consts::PI;
    let angle = mirrored_angle + col_offset * GRID_ANGULAR_SPACING;
    let radius = MP_DEFENDER_GRID_GROUND_RANGE + GRID_ROW_DEPTH / 2.0 + row as f32 * GRID_ROW_DEPTH;
    let x = WIZARD_2_POSITION.x + radius * angle.cos();
    let z = WIZARD_2_POSITION.z + radius * angle.sin();
    (x, z)
}
