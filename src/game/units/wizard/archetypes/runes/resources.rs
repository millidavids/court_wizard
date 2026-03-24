use bevy::prelude::*;
use std::fmt;

/// Individual rune types corresponding to Q, W, E, R keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rune {
    Q,
    W,
    E,
    R,
}

impl Rune {
    /// Returns the display character for this rune.
    pub const fn as_char(&self) -> char {
        match self {
            Rune::Q => 'Q',
            Rune::W => 'W',
            Rune::E => 'E',
            Rune::R => 'R',
        }
    }
}

/// Resource tracking the current rune sequence being built.
#[derive(Resource, Debug, Clone, Default)]
pub struct RuneSequence {
    /// Current sequence of runes pressed.
    pub runes: Vec<Rune>,
    /// Timer since last rune press (for timeout).
    pub idle_timer: f32,
}

impl RuneSequence {
    /// Adds a rune to the sequence and resets the idle timer.
    /// Prevents duplicate consecutive runes.
    pub fn push(&mut self, rune: Rune) {
        self.runes.push(rune);
        self.idle_timer = 0.0;
    }

    /// Clears the sequence.
    pub fn clear(&mut self) {
        self.runes.clear();
        self.idle_timer = 0.0;
    }

    /// Updates the idle timer.
    pub fn tick(&mut self, delta: f32) {
        self.idle_timer += delta;
    }

    /// Returns true if the sequence has timed out.
    pub fn has_timed_out(&self, timeout_duration: f32) -> bool {
        !self.runes.is_empty() && self.idle_timer >= timeout_duration
    }

    /// Returns true if sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.runes.is_empty()
    }

    /// Returns the number of runes in the sequence.
    pub fn len(&self) -> usize {
        self.runes.len()
    }
}

impl fmt::Display for RuneSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rune in &self.runes {
            write!(f, "{}", rune.as_char())?;
        }
        Ok(())
    }
}

/// Resource tracking the last activated spell for UI display purposes.
#[derive(Resource, Debug, Clone, Default)]
pub struct LastActivatedSpell {
    pub spell: Option<crate::game::units::wizard::components::Spell>,
    pub just_activated: bool,
}

impl LastActivatedSpell {
    /// Sets the last activated spell and marks it as just activated.
    pub fn activate(&mut self, spell: crate::game::units::wizard::components::Spell) {
        self.spell = Some(spell);
        self.just_activated = true;
    }

    /// Clears the just_activated flag (called after UI reads it).
    pub fn acknowledge(&mut self) {
        self.just_activated = false;
    }
}
