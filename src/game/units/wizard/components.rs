use bevy::prelude::*;

/// Available spell types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum SpellType {
    MagicMissile,
    Disintegrate,
}

/// Component tracking which spell is currently primed for casting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct PrimedSpell {
    pub spell: SpellType,
}

impl PrimedSpell {
    /// Creates a new PrimedSpell with the given spell type.
    pub const fn new(spell: SpellType) -> Self {
        Self { spell }
    }
}

/// Wizard component with spell casting range.
#[derive(Component)]
pub struct Wizard {
    /// Maximum distance from wizard position that spells can be cast (in units).
    pub spell_range: f32,
}

impl Wizard {
    /// Creates a new Wizard with the given spell range.
    pub const fn new(spell_range: f32) -> Self {
        Self { spell_range }
    }
}

/// Mana component for the wizard.
///
/// Tracks current and maximum mana for spell casting.
#[derive(Component)]
pub struct Mana {
    /// Current mana amount.
    pub current: f32,
    /// Maximum mana capacity.
    pub max: f32,
}

impl Mana {
    /// Creates a new Mana component with the given maximum.
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    /// Returns true if there is enough mana for the cost.
    pub fn can_afford(&self, cost: f32) -> bool {
        self.current >= cost
    }

    /// Consumes mana, returning true if successful.
    pub fn consume(&mut self, cost: f32) -> bool {
        if self.can_afford(cost) {
            self.current -= cost;
            true
        } else {
            false
        }
    }

    /// Regenerates mana, clamped to max.
    pub fn regenerate(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Returns mana as a percentage (0.0 to 1.0).
    pub fn percentage(&self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }
}

/// Mana regeneration component.
///
/// Defines how fast mana regenerates per second.
#[derive(Component)]
pub struct ManaRegen {
    /// Mana regenerated per second.
    pub rate: f32,
}

impl ManaRegen {
    /// Creates a new ManaRegen component.
    pub const fn new(rate: f32) -> Self {
        Self { rate }
    }
}

/// Casting state component for the wizard.
///
/// Tracks active spell casting progress and channeling.
#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub enum CastingState {
    /// Not casting or channeling.
    #[default]
    Resting,
    /// Currently casting a spell.
    Casting {
        /// Time accumulated toward cast completion (in seconds).
        elapsed: f32,
    },
    /// Channeling after cast completion.
    Channeling {
        /// Total time spent channeling (in seconds).
        total_time: f32,
        /// Time since last channeled spell effect (in seconds).
        time_since_last_effect: f32,
    },
}

impl CastingState {
    /// Creates a new CastingState in the Resting state.
    pub const fn new() -> Self {
        Self::Resting
    }

    /// Starts a new cast.
    pub fn start_cast(&mut self) {
        *self = Self::Casting { elapsed: 0.0 };
    }

    /// Transitions from casting to channeling.
    pub fn start_channeling(&mut self) {
        *self = Self::Channeling {
            total_time: 0.0,
            time_since_last_effect: 0.0,
        };
    }

    /// Cancels the current cast or channel, returning to Resting.
    pub fn cancel(&mut self) {
        *self = Self::Resting;
    }

    /// Advances the cast by the given time (only during Casting state).
    pub fn advance(&mut self, delta: f32) {
        if let Self::Casting { elapsed } = self {
            *elapsed += delta;
        }
    }

    /// Advances channeling timers (only during Channeling state).
    pub fn advance_channel(&mut self, delta: f32) {
        if let Self::Channeling {
            total_time,
            time_since_last_effect,
        } = self
        {
            *total_time += delta;
            *time_since_last_effect += delta;
        }
    }

    /// Resets the time since last channel effect (call when spawning a channeled spell).
    pub fn reset_channel_interval(&mut self) {
        if let Self::Channeling {
            time_since_last_effect,
            ..
        } = self
        {
            *time_since_last_effect = 0.0;
        }
    }

    /// Returns the current channel interval based on how long channeling has been active.
    ///
    /// Starts at `initial_interval` and decreases to `min_interval` over `ramp_time`.
    pub fn channel_interval(
        &self,
        initial_interval: f32,
        min_interval: f32,
        ramp_time: f32,
    ) -> f32 {
        if let Self::Channeling { total_time, .. } = self {
            if ramp_time <= 0.0 {
                return min_interval;
            }

            let t = (total_time / ramp_time).min(1.0);
            initial_interval + (min_interval - initial_interval) * t
        } else {
            initial_interval
        }
    }

    /// Returns true if enough time has passed to spawn another channeled spell.
    pub fn should_channel(&self, initial_interval: f32, min_interval: f32, ramp_time: f32) -> bool {
        if let Self::Channeling {
            time_since_last_effect,
            ..
        } = self
        {
            *time_since_last_effect
                >= self.channel_interval(initial_interval, min_interval, ramp_time)
        } else {
            false
        }
    }

    /// Returns true if the cast is complete (ready to transition to channeling).
    pub fn is_complete(&self, cast_time: f32) -> bool {
        if let Self::Casting { elapsed } = self {
            *elapsed >= cast_time
        } else {
            false
        }
    }

    /// Returns cast progress as a percentage (0.0 to 1.0).
    /// Returns 1.0 when channeling to keep bar full.
    pub fn progress(&self, cast_time: f32) -> f32 {
        match self {
            Self::Resting => 0.0,
            Self::Casting { elapsed } => {
                if cast_time > 0.0 {
                    (elapsed / cast_time).min(1.0)
                } else {
                    0.0
                }
            }
            Self::Channeling { .. } => 1.0,
        }
    }
}
