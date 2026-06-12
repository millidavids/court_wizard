use bevy::prelude::*;

use super::constants;

/// Talent parameters computed at cast time, stored on each polymorphed entity.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PolymorphTalentParams {
    // Tier 1: numeric modifiers
    pub sheep_hp: f32,
    pub duration: f32,
    pub cast_time_mult: f32,
    // Tier 2: behavioral flags
    pub explosive: bool,
    pub contagious: bool,
    pub pig_form: bool,
    // Tier 3: transformative flags
    pub permanent: bool,
    pub mass: bool,
    pub dire: bool,
}

impl Default for PolymorphTalentParams {
    fn default() -> Self {
        Self {
            sheep_hp: constants::SHEEP_HP,
            duration: constants::POLYMORPH_DURATION,
            cast_time_mult: 1.0,
            explosive: false,
            contagious: false,
            pig_form: false,
            permanent: false,
            mass: false,
            dire: false,
        }
    }
}

/// Marker: sheep explodes on death for AoE damage.
#[derive(Component)]
pub(crate) struct ExplosiveSheep;

/// Talent: when this sheep's polymorph expires, it jumps to the nearest unit.
#[derive(Component)]
pub(crate) struct ContagiousBaas {
    /// Empowerment value to pass to the spread polymorph.
    pub empowerment: f32,
    /// Talent params to copy to the spread target.
    pub talent_params: PolymorphTalentParams,
}

/// Marker: unit is in pig form (flees from nearest defender at high speed).
#[derive(Component)]
pub(crate) struct PigForm;

/// Marker: if the sheep survives its full duration, kill it permanently.
#[derive(Component)]
pub(crate) struct PermanentLivestock;

/// Dire Sheep: friendly sheep that headbutts enemies.
#[derive(Component)]
pub(crate) struct DireSheep {
    /// Time until next attack.
    pub attack_timer: f32,
}

impl DireSheep {
    pub fn new() -> Self {
        Self {
            attack_timer: constants::DIRE_SHEEP_ATTACK_INTERVAL,
        }
    }
}

impl Default for DireSheep {
    fn default() -> Self {
        Self::new()
    }
}
