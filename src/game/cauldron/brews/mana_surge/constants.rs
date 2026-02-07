use crate::game::cauldron::brews::{BrewConfig, BrewEffect};

pub const CONFIG: BrewConfig = BrewConfig {
    name: "Mana Surge",
    description: "Doubles mana regeneration for 30 seconds.",
    brew_time: 8.0,
    buff_duration: 30.0,
    effects: &[BrewEffect::ManaRegenMultiplier(2.0)],
};
