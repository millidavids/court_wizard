//! Cross-cutting unit component types.
//!
//! This module re-exports all component types at the same public paths they
//! occupied in the original `components.rs`, preserving every import site
//! crate-wide without modification.

mod combat;
mod dots;
mod effectiveness;
mod health;
mod materials;
mod movement;
mod team;
mod timed_modifier;
mod unit_markers;

// Sub-module re-exports (Phase 9 split — kept from original)
pub use super::animation::*;
pub use super::status_effects::*;
pub use super::unit_type::*;

// Elite re-export (kept from original)
pub use super::elite::{EliteAttackSpeedBonus, EliteDamageBonus, EliteSpeedBonus};

// --- team ---
pub use team::Team;

// --- health ---
pub use health::sync_setup_immunity_flag;
pub use health::{
    Health, Invulnerable, PendingDamageEffect, PendingSpellHit, ResidualFireDamaged, SpellDamaged,
    TemporaryHitPoints, apply_damage_to_unit, apply_spell_damage, apply_spell_damage_with_team,
};
pub(crate) use health::{is_setup_immune, set_setup_immunity};

// --- movement ---
pub use movement::{
    Airborne, CommanderAuraSpeedModifier, FALL_DAMAGE_SCALE, FlockingModifier, FlockingVelocity,
    FrostAccumulation, Knockback, MovementSpeed, RoughTerrain, RoughTerrainModifier,
    SlowMovementModifier, TargetingVelocity,
};

// --- combat ---
pub use combat::{
    AttackTiming, DamageMultiplier, Hitbox, InMelee, MeleeDamageReduction, MeleeRangeBonus,
};

// --- dots ---
pub use dots::{FireDoT, Shocked};

// --- effectiveness ---
pub use effectiveness::Effectiveness;

// --- unit_markers ---
pub use unit_markers::{
    Corpse, Flying, HasShadow, KingsGuard, MindControlled, PermanentCorpse, RetaliationTarget,
    Teleportable, UnitShadow, UnitTypeGlow,
};

// --- materials ---
pub use materials::{
    OriginalMaterial, RemoteBattleHymnEffect, RemoteElectricEffect, RemoteFireEffect,
    RemoteFrostEffect, RemoteHasteEffect, RemoteHealingEffect, RemotePoisonEffect,
    RemotePolymorphEffect, RemoteRageEffect, RemoteTempHpEffect,
};

// --- timed_modifier ---
pub use timed_modifier::TimedModifier;
pub(crate) use timed_modifier::impl_timed_modifier;

// Apply TimedModifier to all relevant types (requires all types to be in scope via re-exports above).
impl_timed_modifier!(
    SlowMovementModifier,
    TemporaryHitPoints,
    RootedModifier,
    HasteModifier,
    MarkedForDeathModifier,
    SleepModifier,
    // BattleHymnModifier has a custom tick system (handles EchoingSong)
    BerserkerRageModifier,
    FogEvasionModifier,
    FrozenSolidModifier,
    Stunned,
    Petrified,
    FearModifier,
);
