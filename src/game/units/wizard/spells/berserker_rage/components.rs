use bevy::prelude::*;

/// Talent parameters computed at cast time from active talent selections.
pub(crate) struct BerserkerRageTalentParams {
    // Tier 1: numeric modifiers
    pub damage_bonus: f32,
    pub vulnerability: f32,
    pub radius_mult: f32,
    // Tier 2: behavioral flags
    pub bloodlust: bool,
    pub undying_fury: bool,
    pub frenzy: bool,
    // Tier 3: transformative flags
    pub wrath_incarnate: bool,
    pub contagious_rage: bool,
    pub final_stand: bool,
}

impl Default for BerserkerRageTalentParams {
    fn default() -> Self {
        Self {
            damage_bonus: super::constants::DAMAGE_BONUS,
            vulnerability: super::constants::DAMAGE_VULNERABILITY,
            radius_mult: 1.0,
            bloodlust: false,
            undying_fury: false,
            frenzy: false,
            wrath_incarnate: false,
            contagious_rage: false,
            final_stand: false,
        }
    }
}

/// Tier 2: Enraged units heal for a fraction of damage dealt.
#[derive(Component)]
pub(crate) struct Bloodlust {
    pub heal_fraction: f32,
}

/// Tier 2: Enraged units that would die instead survive at 1 HP.
/// One-shot trigger: removed after activation.
#[derive(Component)]
pub(crate) struct UndyingFury;

/// Active death protection from Undying Fury.
/// While present, unit cannot drop below 1 HP. Expires after timer.
#[derive(Component)]
pub(crate) struct UndyingFuryActive {
    pub time_remaining: f32,
}

/// Tier 2: Enraged units gain attack speed when below HP threshold.
/// A separate system toggles `FrenzyActive` based on current HP.
#[derive(Component)]
pub(crate) struct Frenzy {
    pub attack_speed_bonus: f32,
    pub hp_threshold: f32,
}

/// Marker inserted by `frenzy_check_system` when a unit with `Frenzy`
/// is below the HP threshold. The combat system reads the bonus from `Frenzy`.
#[derive(Component)]
pub(crate) struct FrenzyActive;

/// Tier 3: When an enraged unit kills an enemy, rage spreads to the nearest calm ally.
/// Stored on the enraged unit. The rage params are used for the spread.
#[derive(Component)]
pub(crate) struct ContagiousRage {
    pub damage_bonus: f32,
    pub vulnerability: f32,
    pub duration: f32,
}

/// Tier 3: When an enraged unit dies, it explodes for AoE damage.
/// Persists through corpse conversion and fires once.
#[derive(Component)]
pub(crate) struct FinalStand {
    pub damage_fraction: f32,
    pub radius: f32,
}

/// Visual-only expanding fireball explosion spawned by Final Stand.
/// Grows from small to max_radius then despawns.
#[derive(Component)]
pub(crate) struct FinalStandExplosionVfx {
    pub time_alive: f32,
    pub max_radius: f32,
    pub lifetime: f32,
}
