use crate::game::cauldron::brews::{BrewConfig, BrewEffect};

pub const CONFIG: BrewConfig = BrewConfig {
    name: "Empowerment",
    description: "Increases spell power by 50% for 20 seconds.",
    brew_time: 10.0,
    buff_duration: 20.0,
    effects: &[BrewEffect::SpellPowerMultiplier(1.5)],
};
