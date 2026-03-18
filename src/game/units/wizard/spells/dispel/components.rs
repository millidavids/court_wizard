use bevy::prelude::*;

/// Straight-line projectile that flies toward the ground.
/// Detonates on battlefield impact (y<=0) or lifetime expiry, spawning a DispelImpact.
#[derive(Component)]
pub struct DispelProjectile {
    /// Velocity vector (direction * speed).
    pub velocity: Vec3,
    /// Remaining lifetime before forced despawn.
    pub lifetime: f32,
    /// Expansion speed for the impact sphere (pre-computed from Counterspell talent).
    pub expand_speed: f32,
}

/// Cooldown timer for wizard dispel casting.
#[derive(Component)]
pub struct DispelCooldown {
    pub remaining: f32,
}

/// Expanding translucent sphere that dispels spell effects it overlaps.
#[derive(Component)]
pub struct DispelImpact {
    /// Time this impact has been alive (seconds).
    pub time_alive: f32,
    /// Total duration before despawn (seconds).
    pub duration: f32,
    /// Expansion speed (units/second). Modified by Counterspell talent.
    pub expand_speed: f32,
}

/// Talent parameters computed at cast time from active talent selections.
pub(crate) struct DispelTalentParams {
    // Tier 1
    pub broad_spectrum: bool,
    pub cooldown_mult: f32,
    pub mana_cost: f32,
    // Tier 2
    pub mana_drain: bool,
    pub explosive_nullification: bool,
    pub counterspell_speed_mult: f32,
    pub counterspell_expand_mult: f32,
    // Tier 3
    pub antimagic_pulse: bool,
    pub spell_reflection: bool,
    pub null_zone: bool,
}

impl Default for DispelTalentParams {
    fn default() -> Self {
        Self {
            broad_spectrum: false,
            cooldown_mult: 1.0,
            mana_cost: super::constants::MANA_COST,
            mana_drain: false,
            explosive_nullification: false,
            counterspell_speed_mult: 1.0,
            counterspell_expand_mult: 1.0,
            antimagic_pulse: false,
            spell_reflection: false,
            null_zone: false,
        }
    }
}

/// Marker on DispelImpact: also remove buffs from units inside the impact radius.
#[derive(Component)]
pub(crate) struct BroadSpectrum;

/// Marker on DispelImpact: refund mana for each spell effect dispelled.
#[derive(Component)]
pub(crate) struct ManaDrain;

/// Marker on DispelImpact: dispelled effects explode for AoE damage.
#[derive(Component)]
pub(crate) struct ExplosiveNullification;

/// Marker on DispelImpact: dispelled offensive spells redirect at nearest enemy.
#[derive(Component)]
pub(crate) struct SpellReflection;

/// Marker on DispelImpact: spawn a null zone at the impact point.
#[derive(Component)]
pub(crate) struct NullZoneOnImpact;

/// Persistent anti-magic zone that suppresses spell effects inside it.
#[derive(Component)]
pub(crate) struct NullZone {
    pub time_remaining: f32,
    pub radius: f32,
    pub origin: Vec3,
}
