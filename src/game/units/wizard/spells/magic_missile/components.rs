use bevy::prelude::*;

use super::constants;
use crate::game::units::DamageType;
use crate::game::units::components::Team;

/// Concentration entity for Arcane Barrage talent.
/// Periodically auto-casts magic missile volleys while active.
#[derive(Component)]
pub struct ArcaneBarrage {
    /// Time since last volley was fired.
    pub timer: f32,
    /// Interval between volleys.
    pub interval: f32,
    /// Number of missiles per volley (from tier 1 talent).
    pub missile_count: u32,
    /// Damage multiplier (from tier 1/3 talents).
    pub damage_mult: f32,
    /// Whether missiles pierce (tier 2, choice 2 — Piercing Bolts).
    pub piercing: bool,
    /// Whether missiles use heavy ordnance radius (tier 1, choice 1).
    pub heavy: bool,
    /// Whether missiles have storm wobble (tier 3, choice 0).
    pub storm_wobble: bool,
    /// Whether missiles detonate on impact (tier 3, choice 1).
    pub detonation: bool,
    /// Whether missiles split on kill (tier 2, choice 0 — Seeker Swarm).
    pub seeker_swarm: bool,
    /// Whether missiles are cursor-guided (tier 3, choice 2 — Guided Devastation).
    pub guided: bool,
    /// Which teams to target.
    pub target_teams: TargetTeams,
    /// Wizard spell range.
    pub spell_range: f32,
    /// Empowerment multiplier.
    pub empowerment: f32,
}

/// Cooldown timer for magic missile casting.
///
/// Attached to the wizard entity. When `remaining > 0`, the wizard cannot cast another volley.
#[derive(Component)]
pub struct MagicMissileCooldown {
    pub remaining: f32,
}

/// Defines which teams a magic missile can target.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum TargetTeams {
    /// Host wizard: targets Attackers and Undead.
    AttackersAndUndead,
    /// Guest wizard: targets Defenders and Undead.
    DefendersAndUndead,
}

impl TargetTeams {
    /// Returns true if the given team is a valid target.
    pub fn matches(&self, team: &Team) -> bool {
        match self {
            TargetTeams::AttackersAndUndead => *team == Team::Attackers || *team == Team::Undead,
            TargetTeams::DefendersAndUndead => *team == Team::Defenders || *team == Team::Undead,
        }
    }
}

/// Component for magic missile projectiles.
///
/// Magic missiles lock onto a target when launched and track it until it despawns.
#[derive(Component)]
pub struct MagicMissile {
    /// Current velocity of the missile.
    pub velocity: Vec3,
    /// Initial homing strength (increases over time).
    pub base_homing_strength: f32,
    /// Damage dealt on impact.
    pub damage: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Collision radius.
    pub radius: f32,
    /// Accumulated time for path variability.
    pub time_alive: f32,
    /// Random offset for this specific missile's wobble pattern.
    pub wobble_offset: f32,
    /// Locked target entity (retargets only if this despawns).
    pub target: Option<Entity>,
    /// Whether this missile is empowered.
    pub empowerment: f32,
    /// Which teams this missile targets.
    pub target_teams: TargetTeams,
    /// The wizard's spell range (for despawn distance and retargeting).
    pub spell_range: f32,
    /// Where the missile was spawned from (for despawn distance check).
    pub origin_pos: Vec3,
    /// Whether this missile pierces through the first target (Piercing Bolts talent).
    pub piercing: bool,
    /// Number of targets already pierced (for piercing talent).
    pub pierced_count: u8,
    /// Whether this missile detonates on impact for AoE (Arcane Detonation talent).
    pub detonation: bool,
    /// Whether this missile splits into 2 on kill (Seeker Swarm talent).
    pub seeker_swarm: bool,
    /// Split generation (0 = original, limits recursive splitting).
    pub split_generation: u8,
    /// Whether this missile is cursor-guided (Guided Devastation talent).
    pub guided: bool,
}

impl MagicMissile {
    /// Creates a new magic missile.
    ///
    /// # Arguments
    ///
    /// * `initial_velocity` - Starting velocity vector
    /// * `wobble_offset` - Random offset for wobble pattern
    /// * `target` - Initial target entity to lock onto
    /// * `empowerment` - Damage/effect multiplier
    /// * `target_teams` - Which teams to target
    /// * `spell_range` - Wizard's spell range for retargeting/despawn
    /// * `origin_pos` - Spawn position for despawn distance check
    pub fn new(
        initial_velocity: Vec3,
        wobble_offset: f32,
        target: Option<Entity>,
        empowerment: f32,
        target_teams: TargetTeams,
        spell_range: f32,
        origin_pos: Vec3,
    ) -> Self {
        let scale = empowerment;
        Self {
            velocity: initial_velocity,
            base_homing_strength: constants::BASE_HOMING_STRENGTH * scale,
            damage: constants::DAMAGE * scale,
            damage_type: constants::DAMAGE_TYPE,
            radius: constants::COLLISION_RADIUS * scale,
            time_alive: 0.0,
            wobble_offset,
            target,
            empowerment,
            target_teams,
            spell_range,
            origin_pos,
            piercing: false,
            pierced_count: 0,
            detonation: false,
            seeker_swarm: false,
            split_generation: 0,
            guided: false,
        }
    }

    /// Calculates current homing strength based on time alive.
    ///
    /// Homing increases over perfect tracking time, then becomes perfect tracking.
    pub fn current_homing_strength(&self) -> f32 {
        if self.time_alive >= constants::PERFECT_TRACKING_TIME {
            // After perfect tracking time, return effectively infinite homing (perfect tracking)
            f32::INFINITY
        } else {
            // Ramp up over perfect tracking time
            let t = self.time_alive / constants::PERFECT_TRACKING_TIME;
            let strength_multiplier = t * constants::HOMING_RAMP_MULTIPLIER;
            self.base_homing_strength * (1.0 + strength_multiplier)
        }
    }

    /// Calculates current max speed based on time alive.
    ///
    /// Speed increases based on multipliers over perfect tracking time.
    pub fn current_max_speed(&self) -> f32 {
        let scale = self.empowerment;
        let base_speed = constants::BASE_SPEED * scale;
        if self.time_alive >= constants::PERFECT_TRACKING_TIME {
            // After perfect tracking time, max speed reaches final multiplier
            base_speed * constants::FINAL_SPEED_MULTIPLIER
        } else {
            // Ramp up from 1x to final multiplier over perfect tracking time
            let t = self.time_alive / constants::PERFECT_TRACKING_TIME;
            base_speed * (1.0 + t * constants::SPEED_RAMP_MULTIPLIER)
        }
    }
}
