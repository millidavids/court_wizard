use bevy::prelude::*;

use super::constants;
use crate::game::units::DamageType;
use crate::game::units::wizard::spells::utils::distance_to_line_segment_xz;

/// Persistent line-shaped fire effect on the ground.
///
/// Deals periodic damage to all units within its rectangular area.
/// Uses point-to-line-segment distance for damage checks.
#[derive(Component)]
pub struct WallOfFireEffect {
    /// Line segment start (XZ plane).
    pub start: Vec3,
    /// Line segment end (XZ plane).
    pub end: Vec3,
    /// Half-width of the wall (distance from center line to edge).
    pub half_width: f32,
    /// Damage dealt each tick.
    pub damage_per_tick: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Time between damage ticks (seconds).
    pub tick_interval: f32,
    /// Total lifetime (seconds).
    pub duration: f32,
    /// Elapsed time (seconds).
    pub time_alive: f32,
    /// Accumulator for tick timing.
    pub time_since_last_tick: f32,
    /// Talent parameters for this wall instance.
    pub talent_params: WallOfFireTalentParams,
}

impl WallOfFireEffect {
    pub fn new(
        start: Vec3,
        end: Vec3,
        half_width: f32,
        damage_per_tick: f32,
        damage_type: DamageType,
        tick_interval: f32,
        duration: f32,
        talent_params: WallOfFireTalentParams,
    ) -> Self {
        Self {
            start,
            end,
            half_width,
            damage_per_tick,
            damage_type,
            tick_interval,
            duration,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
            talent_params,
        }
    }

    /// Returns the shortest XZ-plane distance from a point to the wall's center line segment.
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        distance_to_line_segment_xz(point, self.start, self.end)
    }

    /// Returns the effective damage per tick, accounting for Consuming Inferno ramp.
    pub fn effective_damage(&self) -> f32 {
        let inferno_mult = if self.talent_params.consuming_inferno {
            1.0 + (self.time_alive * constants::CONSUMING_INFERNO_RAMP_PER_SECOND)
                .min(constants::CONSUMING_INFERNO_MAX_RAMP)
        } else {
            1.0
        };
        self.damage_per_tick * inferno_mult
    }
}

/// Component on the wizard tracking wall of fire placement state.
#[derive(Component)]
pub struct WallOfFireCaster {
    /// World position where the player first clicked.
    pub anchor: Option<Vec3>,
    /// Entity ID of the preview mesh.
    pub preview_entity: Option<Entity>,
}

impl WallOfFireCaster {
    pub const fn new() -> Self {
        Self {
            anchor: None,
            preview_entity: None,
        }
    }
}

/// Marker component for the wall of fire preview mesh shown during drag.
#[derive(Component)]
pub struct WallOfFirePreview;

/// Looping sound effect entity that follows a wall of fire.
#[derive(Component)]
pub(super) struct WallOfFireSfx {
    /// The parent wall of fire entity this sound tracks.
    pub wall_entity: Entity,
}

/// Talent parameters computed from active talent selections.
/// Stored on each WallOfFireEffect so all talent logic can reference it.
#[derive(Clone)]
pub(crate) struct WallOfFireTalentParams {
    // Tier 1: numeric modifiers
    pub damage_mult: f32,
    pub width_mult: f32,
    pub duration_mult: f32,
    pub max_length_mult: f32,
    // Tier 2: behavioral flags
    pub searing_heat: bool,
    pub scorched_earth: bool,
    pub spreading_flames: bool,
    // Tier 3: transformative flags
    pub firestorm: bool,
    pub twin_walls: bool,
    pub consuming_inferno: bool,
}

impl Default for WallOfFireTalentParams {
    fn default() -> Self {
        Self {
            damage_mult: 1.0,
            width_mult: 1.0,
            duration_mult: 1.0,
            max_length_mult: 1.0,
            searing_heat: false,
            scorched_earth: false,
            spreading_flames: false,
            firestorm: false,
            twin_walls: false,
            consuming_inferno: false,
        }
    }
}

/// Marks a unit currently inside a wall of fire zone (for tracking exit for Spreading Flames).
#[derive(Component)]
pub(crate) struct InsideWallOfFire;

/// Marks a unit with Searing Heat healing reduction while inside the wall.
/// Stores the reduction amount so it can be removed cleanly.
#[derive(Component)]
pub(crate) struct SearingHeatDebuff(pub f32);

/// Lingering fire DoT applied by Spreading Flames when a unit leaves the wall.
#[derive(Component)]
pub(crate) struct SpreadingFlamesDoT {
    pub damage_per_tick: f32,
    pub tick_interval: f32,
    pub time_remaining: f32,
    pub tick_timer: f32,
}

/// Marks a unit that was damaged by a wall of fire with the Firestorm talent.
/// When this unit dies, it triggers a fire explosion at its position.
#[derive(Component)]
pub(crate) struct FirestormMarked;

/// Marks a dead unit that has already triggered a Firestorm explosion.
#[derive(Component)]
pub(crate) struct FirestormProcessed;

/// Scorched Earth zone left behind after a wall of fire expires.
#[derive(Component)]
pub(crate) struct ScorchedEarthZone {
    pub start: Vec3,
    pub end: Vec3,
    pub half_width: f32,
    pub duration: f32,
    pub time_alive: f32,
    pub tick_timer: f32,
}

impl ScorchedEarthZone {
    /// Returns the shortest XZ-plane distance from a point to the zone's center line segment.
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        distance_to_line_segment_xz(point, self.start, self.end)
    }
}
