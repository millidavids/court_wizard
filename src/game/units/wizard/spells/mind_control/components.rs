use bevy::prelude::*;

/// Cooldown timer for wizard mind control casting.
#[derive(Component)]
pub struct MindControlCooldown {
    pub remaining: f32,
}

impl crate::game::units::wizard::spells::utils::HasCooldownRemaining for MindControlCooldown {
    fn remaining_mut(&mut self) -> &mut f32 {
        &mut self.remaining
    }
}

/// Talent parameters computed at cast time from active talent selections.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MindControlTalentParams {
    // Tier 1: numeric modifiers
    pub duration_mult: f32,
    pub damage_multiplier: f32,
    pub cast_time_mult: f32,
    // Tier 2: behavioral flags
    pub puppet_master: bool,
    pub traitors_mark: bool,
    pub amnesia: bool,
    // Tier 3: transformative flags
    pub dominate: bool,
    pub mass_hysteria: bool,
    pub sleeper_agent: bool,
}

impl Default for MindControlTalentParams {
    fn default() -> Self {
        Self {
            duration_mult: 1.0,
            damage_multiplier: 1.0,
            cast_time_mult: 1.0,
            puppet_master: false,
            traitors_mark: false,
            amnesia: false,
            dominate: false,
            mass_hysteria: false,
            sleeper_agent: false,
        }
    }
}

/// Marker: MC'd unit has Traitor's Mark aura (nearby enemies take more damage).
#[derive(Component)]
pub(crate) struct TraitorsMarkAura;

/// Debuff applied to enemies near a unit with TraitorsMarkAura.
/// Increases damage taken from all sources.
#[derive(Component)]
pub(crate) struct Demoralized {
    pub damage_amplification: f32,
}

/// Marker: MC'd unit will be confused when mind control ends.
/// Inserted on the MC'd entity at cast time; the wear-off system reads it.
#[derive(Component)]
pub(crate) struct AmnesiaOnExpiry;

/// Active amnesia effect — unit attacks random targets (friend or foe).
#[derive(Component)]
pub(crate) struct AmnesiaEffect {
    pub time_remaining: f32,
}

/// Marker: permanently dominated unit (Tier 3 Dominate talent).
#[derive(Component)]
pub(crate) struct DominatedUnit;

/// Mass Hysteria effect — unit attacks random nearby units (friend or foe).
/// Not true mind control — just chaos.
#[derive(Component)]
pub(crate) struct MassHysteriaTarget {
    pub time_remaining: f32,
}

/// Sleeper Agent — when MC "wears off," the unit appears normal for a delay,
/// then attacks the nearest ally once with bonus damage.
#[derive(Component)]
pub(crate) struct SleeperAgentPending;

/// Active Sleeper Agent timer — ticking down to betrayal.
#[derive(Component)]
pub(crate) struct SleeperAgentActive {
    pub delay_remaining: f32,
    pub damage_multiplier: f32,
}
