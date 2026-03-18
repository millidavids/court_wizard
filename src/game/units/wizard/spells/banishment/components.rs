use bevy::prelude::*;

/// Talent parameters computed at cast time from active talent selections.
pub(crate) struct BanishmentTalentParams {
    // Tier 1: numeric modifiers
    pub duration: f32,
    pub cast_time_mult: f32,
    pub mana_mult: f32,
    // Tier 2: behavioral flags
    pub painful_return: bool,
    pub displacement: bool,
    pub dual_banishment: bool,
    // Tier 3: transformative flags
    pub dimensional_shunt: bool,
    pub mass_banishment: bool,
    pub one_way_trip: bool,
}

impl Default for BanishmentTalentParams {
    fn default() -> Self {
        Self {
            duration: super::constants::BANISH_DURATION,
            cast_time_mult: 1.0,
            mana_mult: 1.0,
            painful_return: false,
            displacement: false,
            dual_banishment: false,
            dimensional_shunt: false,
            mass_banishment: false,
            one_way_trip: false,
        }
    }
}

/// Tier 2: Banished unit takes heavy damage when it returns.
#[derive(Component)]
pub(crate) struct PainfulReturn {
    pub damage: f32,
}

/// Tier 2: Banished unit reappears at a random location far from where it was banished.
#[derive(Component)]
pub(crate) struct Displacement {
    pub radius: f32,
}

/// Tier 3: Banished unit returns at half HP regardless of original HP.
#[derive(Component)]
pub(crate) struct DimensionalShunt {
    pub hp_fraction: f32,
}

/// Tier 3: Unit was below HP threshold when banished -- killed on "return" instead of restored.
#[derive(Component)]
pub(crate) struct OneWayTrip;

/// Visual-only shrinking lensing sphere spawned when a unit is banished.
/// Shrinks from start_radius to zero over lifetime, then despawns.
#[derive(Component)]
pub(crate) struct BanishmentVfx {
    pub time_alive: f32,
    pub lifetime: f32,
    pub start_radius: f32,
}
