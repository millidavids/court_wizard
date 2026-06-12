use bevy::prelude::*;

use crate::game::units::components::Team;

/// Commander component that provides aura buffs to nearby units.
///
/// Commanders can buff units within their aura radius based on team filtering.
/// The King is a special Commander with additional game-ending logic.
#[derive(Component)]
pub struct Commander {
    /// Radius of the commander's aura effect
    pub aura_radius: f32,
    /// Which teams are affected by this commander's aura
    pub team_filter: TeamFilter,
}

/// Team filter for commander auras.
///
/// Determines which teams are affected by a commander's aura buffs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamFilter {
    /// Only buff Defender units
    Defenders,
    /// Only buff Attacker units
    Attackers,
}

impl TeamFilter {
    /// Checks if this filter allows buffing the given team.
    pub const fn allows(&self, team: Team) -> bool {
        match self {
            Self::Defenders => matches!(team, Team::Defenders),
            Self::Attackers => matches!(team, Team::Attackers),
        }
    }
}

/// Damage buff component for commander auras.
///
/// Applied to commanders to grant nearby units increased damage.
/// Value is a percentage bonus (0.5 = +50% damage).
#[derive(Component, Clone, Copy)]
pub struct AuraDamageBuff(pub f32);

/// A particle that travels outward from a commander toward the aura edge.
#[derive(Component)]
pub struct CommanderAuraParticle {
    pub velocity: Vec3,
    pub time_alive: f32,
    pub lifetime: f32,
}

/// Speed buff component for commander auras.
///
/// Applied to commanders to grant nearby units increased movement speed.
/// Value is a percentage bonus (0.25 = +25% speed).
#[derive(Component, Clone, Copy)]
pub struct AuraSpeedBuff(pub f32);
