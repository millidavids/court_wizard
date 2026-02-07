mod empowerment;
mod mana_surge;
mod plugin;

pub use plugin::BrewsPlugin;

/// A single effect that a brew applies when active.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrewEffect {
    /// Multiplies wizard mana regeneration rate.
    ManaRegenMultiplier(f32),
    /// Multiplies spell power/empowerment.
    SpellPowerMultiplier(f32),
}

/// Configuration defining all properties of a brew.
pub struct BrewConfig {
    /// Display name shown in the UI.
    pub name: &'static str,
    /// Description of the brew's effect.
    pub description: &'static str,
    /// Time required to brew (seconds).
    pub brew_time: f32,
    /// Duration of the buff after brewing (seconds).
    pub buff_duration: f32,
    /// Effects this brew applies when active.
    pub effects: &'static [BrewEffect],
}

/// Available brews that can be crafted in the cauldron.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Brew {
    /// Doubles wizard mana regeneration rate.
    ManaSurge,
    /// Increases spell power by 50%.
    Empowerment,
}

#[allow(dead_code)]
impl Brew {
    /// Returns the full configuration for this brew.
    pub fn config(&self) -> &'static BrewConfig {
        match self {
            Brew::ManaSurge => &mana_surge::constants::CONFIG,
            Brew::Empowerment => &empowerment::constants::CONFIG,
        }
    }

    /// Returns all available brews.
    pub const fn all() -> &'static [Brew] {
        &[Brew::ManaSurge, Brew::Empowerment]
    }

    /// Returns the display name for this brew.
    pub fn name(&self) -> &'static str {
        self.config().name
    }

    /// Returns the description for this brew.
    pub fn description(&self) -> &'static str {
        self.config().description
    }

    /// Returns the time required to brew (seconds).
    pub fn brew_time(&self) -> f32 {
        self.config().brew_time
    }

    /// Returns how long the buff lasts after brewing (seconds).
    pub fn buff_duration(&self) -> f32 {
        self.config().buff_duration
    }
}
