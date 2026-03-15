use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Mind Control.
pub const PRIMED_MIND_CONTROL: PrimedSpell = PrimedSpell {
    spell: Spell::MindControl,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Cast time (seconds).
pub const CAST_TIME: f32 = 1.0;

/// Mana cost per cast.
pub const MANA_COST: f32 = 50.0;

/// Cooldown between casts (seconds).
pub const COOLDOWN: f32 = 2.0;

/// Duration of mind control effect (seconds away from caster before wearing off).
pub const EFFECT_WEAR_OFF_DURATION: f32 = 10.0;

/// Maximum number of mind-controlled units at once (wizard spell).
pub const MAX_CONTROLLED: u32 = 3;

/// Max distance from cursor to find a target during cast.
pub const TARGET_SEARCH_RADIUS: f32 = 80.0;

/// Purple highlight tint for the targeted unit during cast.
pub const HIGHLIGHT_COLOR: Color = Color::srgb(0.7, 0.2, 1.0);

/// Translucent purple for the Mass Hysteria AoE indicator ring.
pub const INDICATOR_COLOR: Color = Color::srgba(0.7, 0.2, 1.0, 0.3);

// --- Talent constants ---

// Tier 1
/// Iron Will: duration multiplier (+40% to all MC effect durations).
pub const IRON_WILL_DURATION_MULT: f32 = 1.4;
/// Deep Domination: controlled units deal 25% more damage.
pub const DEEP_DOMINATION_DAMAGE_MULT: f32 = 1.25;
/// Quick Subjugation: cast time multiplier (40% faster).
pub const QUICK_SUBJUGATION_CAST_MULT: f32 = 0.6;

// Tier 2
/// Puppet Master: max controlled units increased to 5.
pub const PUPPET_MASTER_MAX: u32 = 5;
/// Traitor's Mark: nearby enemies take 15% more damage.
pub const TRAITORS_MARK_DAMAGE_AMP: f32 = 0.15;
/// Traitor's Mark: aura radius around controlled units.
pub const TRAITORS_MARK_RADIUS: f32 = 40.0;
/// Amnesia: duration of confused state after MC ends.
pub const AMNESIA_DURATION: f32 = 3.0;

// Tier 3
/// Mass Hysteria: duration of AoE chaos effect.
pub const MASS_HYSTERIA_DURATION: f32 = 4.0;
/// Mass Hysteria: radius of AoE.
pub const MASS_HYSTERIA_RADIUS: f32 = 150.0;
/// Mass Hysteria: mana cost multiplier (2x base = 100 mana, full bar).
pub const MASS_HYSTERIA_MANA_MULT: f32 = 2.0;
/// Sleeper Agent: delay before betrayal after MC "ends."
pub const SLEEPER_AGENT_DELAY: f32 = 5.0;
/// Sleeper Agent: damage multiplier for the betrayal attack.
pub const SLEEPER_AGENT_DAMAGE_MULT: f32 = 2.0;
/// Sleeper Agent: generous attack range multiplier for the betrayal strike.
pub const SLEEPER_AGENT_RANGE_MULT: f32 = 5.0;

/// Base damage dealt by mind-controlled / confused units per attack.
/// Shared by both the wizard spell and the Hag boss (Martina).
pub const COMBAT_DAMAGE: f32 = 5.0;
/// Attack range multiplier for confused combat (Mass Hysteria, Amnesia).
pub const CONFUSED_ATTACK_RANGE_MULT: f32 = 2.5;
