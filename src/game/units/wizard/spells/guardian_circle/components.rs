use bevy::prelude::*;

/// Marker component for units that received a Guardian Circle shield.
///
/// Tracks which talent effects are active on this shielded unit for
/// Tier 2 and Tier 3 talent reactions (retaliation, martyrdom, chain ward).
#[derive(Component, Clone)]
pub(crate) struct GuardianCircleShielded {
    /// T2-0: Retaliating Wards — burst damage when temp HP fully breaks.
    pub retaliating_damage: f32,
    /// T2-0: Retaliating Wards — burst radius.
    pub retaliating_radius: f32,
    /// T2-1: Fortified Resolve — bonus damage multiplier while shielded.
    pub fortified_damage_bonus: f32,
    /// T3-0: Sanctuary — damage reduction while shielded.
    pub sanctuary_reduction: f32,
    /// T3-1: Martyrdom — explosion damage on death (stored at grant time).
    pub martyrdom_damage: f32,
    /// T3-1: Martyrdom — explosion radius.
    pub martyrdom_radius: f32,
    /// T3-2: Chain Ward — remaining hop count.
    pub chain_ward_hops: u32,
    /// T3-2: Chain Ward — temp HP amount to pass along.
    pub chain_ward_amount: f32,
    /// T3-2: Chain Ward — temp HP duration to pass along.
    pub chain_ward_duration: f32,
}

impl Default for GuardianCircleShielded {
    fn default() -> Self {
        Self {
            retaliating_damage: 0.0,
            retaliating_radius: 0.0,
            fortified_damage_bonus: 0.0,
            sanctuary_reduction: 0.0,
            martyrdom_damage: 0.0,
            martyrdom_radius: 0.0,
            chain_ward_hops: 0,
            chain_ward_amount: 0.0,
            chain_ward_duration: 0.0,
        }
    }
}
