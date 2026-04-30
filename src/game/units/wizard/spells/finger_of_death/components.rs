use std::collections::HashSet;

use bevy::prelude::*;

/// Marker component indicating wizard is waiting for mouse release before allowing another Finger of Death cast.
///
/// This prevents the spell from immediately recasting after completion if the mouse is still held.
#[derive(Component)]
pub struct AwaitingFingerOfDeathRelease;

/// Purple lightning-like particle that meanders along the ground from a hit unit.
#[derive(Component)]
pub struct NecroticVein {
    pub velocity: Vec3,
    pub time_alive: f32,
    pub lifetime: f32,
    pub base_size: f32,
    pub wander_phase: f32,
}

/// Dark glow aura surrounding the Finger of Death beam.
#[derive(Component)]
pub struct FingerOfDeathGlow {
    pub beam_entity: Entity,
}

/// Bright flare at the beam origin point.
#[derive(Component)]
pub struct FingerOfDeathFlare {
    pub beam_entity: Entity,
}

/// Expanding dark ring on the ground from beam origin when it fires.
#[derive(Component)]
pub struct NecroticPulse {
    pub time_alive: f32,
    pub lifetime: f32,
    pub max_radius: f32,
}

/// Finger of Death beam component tracking the devastating instant-cast beam.
#[derive(Component)]
pub struct FingerOfDeathBeam {
    /// Beam starting position (origin point).
    pub origin: Vec3,
    /// Normalized direction vector.
    pub direction: Vec3,
    /// Full beam length.
    pub length: f32,
    /// Time since spawn (for growth animation and despawn timing).
    pub time_alive: f32,
    /// Cast progress (0.0 to 1.0).
    pub cast_progress: f32,
    /// Whether damage has been applied (fires once).
    pub has_fired: bool,
    /// Time since the beam fired (for fade out animation).
    pub time_since_fired: f32,
    /// Whether this beam is empowered.
    pub empowerment: f32,
    /// Talent parameters applied to this beam.
    pub talent_params: FodTalentParams,
}

/// Talent-computed parameters for Finger of Death.
#[derive(Clone, Debug)]
pub struct FodTalentParams {
    pub damage: f32,
    pub beam_width: f32,
    pub beam_width_fired: f32,
    pub mana_threshold: f32,
    pub cast_time_mult: f32,
    pub cooldown_mult: f32,
    /// T1-1: Soul Harvest — mana refund on kill
    pub soul_harvest_refund: f32,
    /// T2-0: Finger of Undeath — raise killed as undead
    pub finger_of_undeath: bool,
    /// T2-2: Siphon Life — heal nearest defender
    pub siphon_life_percent: f32,
    /// T3-0: Reaper's Scythe — sweep arc
    pub reapers_scythe: bool,
    /// T3-1: Necrotic Explosion — AoE on kill
    pub necrotic_explosion: bool,
    /// T3-2: Deathmark — debuff instead of damage
    pub deathmark: bool,
    /// Whether this is an auto-fired chain beam (from Deathmark)
    pub is_chain_beam: bool,
    /// Damage multiplier for chain beams
    pub chain_damage_mult: f32,
}

impl Default for FodTalentParams {
    fn default() -> Self {
        Self {
            damage: super::constants::DAMAGE,
            beam_width: super::constants::BEAM_WIDTH,
            beam_width_fired: super::constants::BEAM_WIDTH_FIRED,
            mana_threshold: super::constants::MANA_REQUIREMENT_PERCENT,
            cast_time_mult: 1.0,
            cooldown_mult: 1.0,
            soul_harvest_refund: 0.0,
            finger_of_undeath: false,
            siphon_life_percent: 0.0,
            reapers_scythe: false,
            necrotic_explosion: false,
            deathmark: false,
            is_chain_beam: false,
            chain_damage_mult: 1.0,
        }
    }
}

impl FingerOfDeathBeam {
    /// Creates a new beam with talent params.
    pub fn with_talents(
        origin: Vec3,
        direction: Vec3,
        length: f32,
        empowerment: f32,
        talent_params: FodTalentParams,
    ) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            length,
            time_alive: 0.0,
            cast_progress: 0.0,
            has_fired: false,
            time_since_fired: 0.0,
            empowerment,
            talent_params,
        }
    }

    /// Gets the damage value, scaled by empowerment and talents.
    pub fn damage(&self) -> f32 {
        self.talent_params.damage * self.empowerment * self.talent_params.chain_damage_mult
    }

    /// Gets the beam width, scaled by empowerment and talents.
    pub fn beam_width(&self) -> f32 {
        self.talent_params.beam_width * self.empowerment
    }

    /// Gets the fired beam width, scaled by empowerment and talents.
    pub fn beam_width_fired(&self) -> f32 {
        self.talent_params.beam_width_fired * self.empowerment
    }

    /// Checks if a point is within the beam.
    pub fn contains_point(&self, point: Vec3, beam_width: f32) -> bool {
        let to_point = point - self.origin;
        let projection_length = to_point.dot(self.direction);

        // Check if within beam length
        if projection_length < 0.0 || projection_length > self.length {
            return false;
        }

        // Check distance from centerline
        let projected_point = self.origin + self.direction * projection_length;
        let distance_from_centerline = point.distance(projected_point);

        distance_from_centerline <= beam_width
    }
}

/// Cooldown timer preventing Finger of Death from being recast.
#[derive(Component)]
pub struct FingerOfDeathCooldown {
    pub remaining: f32,
}

impl crate::game::units::wizard::spells::utils::HasCooldownRemaining for FingerOfDeathCooldown {
    fn remaining_mut(&mut self) -> &mut f32 {
        &mut self.remaining
    }
}

/// Deathmark debuff applied to a target by the Deathmark talent.
/// If the target dies while this is active, a chain Finger of Death fires.
#[derive(Component)]
pub struct DeathmarkDebuff {
    pub time_remaining: f32,
    /// Beam origin for the chain beam.
    pub beam_origin: Vec3,
    /// Empowerment of the original beam.
    pub empowerment: f32,
    /// Talent params to propagate to the chain beam.
    pub talent_params: FodTalentParams,
}

/// Reaper's Scythe sweep state — beam rotates through an arc.
#[derive(Component)]
pub struct ReapersScytheSweep {
    /// Center direction of the arc.
    pub center_direction: Vec3,
    /// Total sweep time elapsed.
    pub time_elapsed: f32,
    /// Total sweep duration.
    pub duration: f32,
    /// Beam origin point.
    pub origin: Vec3,
    /// Beam length.
    pub length: f32,
    /// Beam empowerment.
    pub empowerment: f32,
    /// Talent params for the sweep.
    pub talent_params: FodTalentParams,
    /// Entities already hit during this sweep (to avoid double-hits).
    pub hit_entities: HashSet<Entity>,
}

/// Visual marker for necrotic explosion burst.
#[derive(Component)]
pub struct NecroticExplosionBurst {
    pub time_alive: f32,
    pub lifetime: f32,
    pub max_radius: f32,
    /// AoE damage to deal.
    pub damage: f32,
    /// Whether AoE damage has already been applied (one-shot).
    pub damage_applied: bool,
}

/// Resource storing pending undead raises from Finger of Undeath.
/// Deferred because killed units haven't become corpses yet on the same frame.
#[derive(Resource)]
pub struct PendingUndeadRaise {
    pub kill_positions: Vec<Vec3>,
}
