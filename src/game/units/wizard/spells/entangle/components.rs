use bevy::prelude::*;

use super::constants;
use crate::game::units::wizard::spells::utils::{CircleIndicator, indicator_pulse_scale};

/// Visual indicator for the Entangle area during casting.
#[derive(Component)]
pub struct EntangleIndicator {
    pub position: Vec3,
    pub time_alive: f32,
    pub empowerment: f32,
}

impl EntangleIndicator {
    pub const fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
        }
    }
}

impl CircleIndicator for EntangleIndicator {
    fn position(&self) -> Vec3 {
        self.position
    }
    fn time_alive(&self) -> f32 {
        self.time_alive
    }
    fn set_time_alive(&mut self, time: f32) {
        self.time_alive = time;
    }
    fn base_radius(&self) -> f32 {
        constants::CIRCLE_RADIUS * self.empowerment
    }
    fn circle_y_position(&self) -> f32 {
        constants::CIRCLE_Y_POSITION
    }
    fn pulse_scale(&self) -> f32 {
        indicator_pulse_scale(self.time_alive)
    }
}

/// Persistent ground visual showing the entangle area.
#[derive(Component)]
pub struct EntangleGroundEffect {
    pub time_remaining: f32,
    pub duration: f32,
    /// Talent params for this specific entangle cast.
    pub talent_params: EntangleTalentParams,
    /// Position of the entangle zone center.
    pub center: Vec3,
    /// Base radius (before overgrowth expansion).
    pub base_radius: f32,
    /// Current radius (grows with Overgrowth).
    pub current_radius: f32,
    /// Timer for Overgrowth periodic root checks.
    pub overgrowth_check_timer: f32,
    /// Timer for animated vine ring particle spawning.
    pub animated_vine_timer: f32,
}

impl EntangleGroundEffect {
    pub fn new(duration: f32, center: Vec3, radius: f32, talent_params: EntangleTalentParams) -> Self {
        Self {
            time_remaining: duration,
            duration,
            talent_params,
            center,
            base_radius: radius,
            current_radius: radius,
            overgrowth_check_timer: 0.0,
            animated_vine_timer: 0.0,
        }
    }
}

/// Talent parameters computed from active talent selections.
/// Stored on each EntangleGroundEffect entity so talent logic can reference it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EntangleTalentParams {
    // Tier 1: numeric modifiers
    pub radius_mult: f32,
    pub duration_mult: f32,
    pub mana_mult: f32,
    pub cast_time_mult: f32,
    // Tier 2: behavioral flags
    pub thorny_vines: bool,
    pub clinging_roots: bool,
    pub nourishing_roots: bool,
    // Tier 3: transformative flags
    pub overgrowth: bool,
    pub sanctuary: bool,
    pub stranglehold: bool,
}

impl Default for EntangleTalentParams {
    fn default() -> Self {
        Self {
            radius_mult: 1.0,
            duration_mult: 1.0,
            mana_mult: 1.0,
            cast_time_mult: 1.0,
            thorny_vines: false,
            clinging_roots: false,
            nourishing_roots: false,
            overgrowth: false,
            sanctuary: false,
            stranglehold: false,
        }
    }
}

/// Visual vine torus that rises from the ground during entangle.
#[derive(Component)]
pub(crate) struct EntangleVine {
    /// Final Y position when fully risen.
    pub final_y: f32,
    /// Rise animation elapsed time.
    pub rise_elapsed: f32,
    /// Rise animation duration.
    pub rise_duration: f32,
    /// Total lifetime (matches root duration).
    pub duration: f32,
    /// Time remaining before despawn.
    pub time_remaining: f32,
}

/// Animated vine ring particle that grows outward and fades, giving a "living vines" look.
#[derive(Component)]
pub(crate) struct AnimatedVineRing {
    /// Seconds since this ring was spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub lifetime: f32,
    /// Maximum scale this ring grows to.
    pub max_scale: f32,
}

/// Marker component on units rooted by the Entangle spell.
/// Tracks talent-specific data for Tier 2/3 effects.
#[derive(Component)]
pub(crate) struct EntangleRooted {
    /// Total root duration this unit was given.
    pub total_root_duration: f32,
    /// Whether this unit is a defender.
    pub is_defender: bool,
    /// Thorny Vines: tick timer for DPS.
    pub thorny_tick_timer: f32,
    /// Copy of talent params for effect processing.
    pub talent_params: EntangleTalentParams,
}
