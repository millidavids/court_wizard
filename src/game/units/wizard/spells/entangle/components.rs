use bevy::prelude::*;

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
    pub fn new(
        duration: f32,
        center: Vec3,
        radius: f32,
        talent_params: EntangleTalentParams,
    ) -> Self {
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

/// Marker component on units rooted by the Entangle spell.
/// Tracks talent-specific data for Tier 2/3 effects.
#[derive(Component)]
pub(crate) struct EntangleRooted {
    /// Total root duration this unit was given.
    pub total_root_duration: f32,
    /// Whether this unit is a defender.
    pub is_defender: bool,
    /// Copy of talent params for effect processing.
    pub talent_params: EntangleTalentParams,
}

/// Thorny Vines talent: deals periodic DPS to rooted enemies.
#[derive(Component)]
pub(crate) struct ThornyVines {
    pub tick_timer: f32,
}
