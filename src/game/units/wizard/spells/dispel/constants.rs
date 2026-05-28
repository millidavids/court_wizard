use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Dispel.
pub const PRIMED_DISPEL: PrimedSpell = PrimedSpell {
    spell: Spell::Dispel,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Cast time (instant).
pub const CAST_TIME: f32 = 0.0;

/// Mana cost per cast.
pub const MANA_COST: f32 = 5.0;

/// Cooldown between casts (seconds).
pub const COOLDOWN: f32 = 0.5;

/// Speed of the dispel projectile (units/second).
pub const PROJECTILE_SPEED: f32 = 1600.0;

/// Visual radius of the projectile circle.
pub const PROJECTILE_RADIUS: f32 = 5.0;

/// Color of the dispel projectile and impact sphere.
pub const PROJECTILE_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

/// How long the impact sphere lasts (seconds).
pub const IMPACT_DURATION: f32 = 0.5;

/// How fast the impact sphere expands (units/second in radius).
pub const IMPACT_EXPAND_SPEED: f32 = 200.0;

/// Height offset above spell origin for spawning the projectile.
pub const SPAWN_HEIGHT_OFFSET: f32 = 0.0;

/// Maximum projectile lifetime before auto-despawn (seconds).
pub const PROJECTILE_LIFETIME: f32 = 3.0;

// ===== Talent Constants =====

// Tier 1
/// Swift Cancellation: cooldown multiplier (40% reduction).
pub const SWIFT_CANCELLATION_COOLDOWN_MULT: f32 = 0.6;
/// Efficient Nullification: near-zero mana cost.
pub const EFFICIENT_NULLIFICATION_MANA_COST: f32 = 1.0;

// Tier 2
/// Mana Drain: fraction of the dispelled spell's mana cost refunded to the wizard.
pub const MANA_DRAIN_REFUND_FRACTION: f32 = 0.5;
/// Explosive Nullification: damage dealt to units near a dispelled effect.
pub const EXPLOSIVE_NULLIFICATION_DAMAGE: f32 = 40.0;
/// Explosive Nullification: radius of the damage burst around a dispelled effect.
pub const EXPLOSIVE_NULLIFICATION_RADIUS: f32 = 60.0;
/// Counterspell: projectile speed multiplier.
pub const COUNTERSPELL_SPEED_MULT: f32 = 1.5;
/// Counterspell: impact expand speed multiplier (larger collision radius).
pub const COUNTERSPELL_EXPAND_MULT: f32 = 1.25;
/// Spell Reflection: damage dealt to enemies near the reflected target.
pub const SPELL_REFLECTION_DAMAGE: f32 = 60.0;
/// Spell Reflection: radius of the reflected damage burst.
pub const SPELL_REFLECTION_RADIUS: f32 = 60.0;

// Tier 3
/// Antimagic Pulse: radius of the wizard-centered radial pulse.
pub const ANTIMAGIC_PULSE_RADIUS: f32 = 400.0;
/// Antimagic Pulse: duration of the expanding pulse visual.
pub const ANTIMAGIC_PULSE_DURATION: f32 = 0.6;
/// Null Zone: duration of the persistent anti-magic field (seconds).
pub const NULL_ZONE_DURATION: f32 = 10.0;
/// Null Zone: radius of the anti-magic field.
pub const NULL_ZONE_RADIUS: f32 = 80.0;
/// Null Zone: visual color (pale purple anti-magic field).
pub const NULL_ZONE_COLOR: Color = Color::srgba(0.6, 0.4, 0.9, 0.15);
/// Null Zone: height of the visual cylinder.
pub const NULL_ZONE_HEIGHT: f32 = 40.0;

/// Mana cost lookup for SpellEffectKind (for Mana Drain talent).
/// Returns the mana cost of the spell that created this effect.
pub fn spell_effect_mana_cost(kind: crate::networking::snapshot::SpellEffectKind) -> f32 {
    use crate::game::units::wizard::spells as s;
    use crate::networking::snapshot::SpellEffectKind;
    match kind {
        SpellEffectKind::SpikeGrowthZone => s::spike_growth::constants::MANA_COST,
        SpellEffectKind::HealingPlumeZone => s::healing_plume::constants::MANA_COST,
        SpellEffectKind::EntangleGround => s::entangle::constants::MANA_COST,
        SpellEffectKind::FogCloudZone => s::fog_cloud::constants::MANA_COST,
        SpellEffectKind::GreaseZone | SpellEffectKind::GreaseFire => {
            s::grease::constants::MANA_COST
        }
        SpellEffectKind::PlagueWindCloud => s::plague_wind::constants::MANA_COST,
        SpellEffectKind::MeteorGroundFire => s::meteor_fall::constants::MANA_COST,
        SpellEffectKind::BlackHole => s::black_hole::constants::MANA_COST,
        SpellEffectKind::ArcaneCrystal => s::arcane_crystal::constants::MANA_COST,
        SpellEffectKind::LightningRod => s::lightning_rod::constants::MANA_COST,
        SpellEffectKind::WallOfStone => s::wall_of_stone::constants::MANA_COST,
        SpellEffectKind::WallOfFire => s::wall_of_fire::constants::MANA_COST,
        SpellEffectKind::FireballExplosion
        | SpellEffectKind::MeteorExplosion
        | SpellEffectKind::IceExplosion
        | SpellEffectKind::DispelImpact
        | SpellEffectKind::SquallStorm
        | SpellEffectKind::ScorchedEarthFire
        | SpellEffectKind::NapalmTrail
        // Boulders are physical projectiles / obstacles, not magic spells,
        // so Mana Drain doesn't refund mana for them.
        | SpellEffectKind::BoulderProjectileEffect
        | SpellEffectKind::BoulderObstacle => 0.0, // Not dispellable (or self-dispel)
    }
}
