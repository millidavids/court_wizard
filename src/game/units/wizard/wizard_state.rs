use bevy::prelude::*;

use super::spell_enum::Spell;

/// Component tracking which spell is currently primed for casting.
///
/// Contains both the spell type and its associated properties like cast time.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct PrimedSpell {
    pub spell: Spell,
    /// Time required to cast this spell before it activates (in seconds).
    pub cast_time: f32,
    /// Empowerment multiplier for spell effectiveness (1.0 = normal, 1.25 = 25% bonus, etc.).
    pub empowerment: f32,
    /// Whether the empowerment has been consumed by starting a cast.
    pub empowerment_consumed: bool,
    /// Mana cost multiplier (1.0 = normal, 0.5 = half cost, 2.0 = double cost).
    pub mana_multiplier: f32,
    /// Range multiplier for spell radius and distance (1.0 = normal, 1.5 = 50% more range).
    pub range_multiplier: f32,
}

impl PrimedSpell {
    /// Creates an empowered version of this primed spell with the given multiplier.
    pub const fn with_empowerment(mut self, multiplier: f32) -> Self {
        self.empowerment = multiplier;
        self.cast_time /= multiplier;
        self.empowerment_consumed = false;
        self
    }

    /// Applies empowerment scaling to a value.
    pub fn scale(&self, value: f32) -> f32 {
        value * self.empowerment
    }

    /// Marks the empowerment as consumed (called when starting a cast).
    pub fn consume_empowerment(&mut self) {
        self.empowerment_consumed = true;
    }

    /// Returns true if empowerment needs to be reset.
    pub fn should_reset_empowerment(&self) -> bool {
        self.empowerment != 1.0 && self.empowerment_consumed
    }

    /// Resets empowerment to 1.0 and restores original cast time.
    pub fn reset_empowerment(&mut self) {
        self.empowerment = 1.0;
        self.cast_time = self.spell.primed_config().cast_time;
        self.empowerment_consumed = false;
    }
}

/// Wizard component with spell casting stats.
///
/// These are the wizard's actual effective stats after all modifiers are applied.
/// Various systems (archetypes, cauldron buffs, etc.) modify these values.
#[derive(Component)]
pub struct Wizard {
    /// Maximum distance from wizard position that spells can be cast (in units).
    pub spell_range: f32,
    /// Multiplier for spell mana costs (1.0 = normal, 0.5 = half cost, 2.0 = double cost).
    pub mana_cost_multiplier: f32,
    /// Multiplier for spell power/damage (1.0 = normal, 1.5 = 50% more damage).
    pub spell_power_multiplier: f32,
    /// Multiplier for cast speed (1.0 = normal, 2.0 = twice as fast).
    pub cast_speed_multiplier: f32,
}

impl Wizard {
    /// Creates a new Wizard with the given spell range and default multipliers.
    pub const fn new(spell_range: f32) -> Self {
        Self {
            spell_range,
            mana_cost_multiplier: 1.0,
            spell_power_multiplier: 1.0,
            cast_speed_multiplier: 1.0,
        }
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
    /// Per-wizard spell-cost multiplier, mirrored from `Wizard.mana_cost_multiplier`
    /// by `sync_mana_cost_multiplier`. Applied to EVERY `consume`/`can_afford` so
    /// the Arcanorouter's mana routing, the BoringOleMage discount, and insight
    /// bonuses affect ALL spells (not just the few that read the Wizard field).
    /// 1.0 = no change.
    pub cost_multiplier: f32,
}

impl Mana {
    /// Creates a new Mana component with the given maximum.
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            cost_multiplier: 1.0,
        }
    }

    /// The cost actually deducted for a base `cost`, after the wizard multiplier.
    fn effective_cost(&self, cost: f32) -> f32 {
        cost * self.cost_multiplier
    }

    /// Returns true if there is enough mana for the cost (after the multiplier).
    pub fn can_afford(&self, cost: f32) -> bool {
        self.current >= self.effective_cost(cost)
    }

    /// Consumes mana, returning true if successful.
    pub fn consume(&mut self, cost: f32) -> bool {
        if self.can_afford(cost) {
            self.current -= self.effective_cost(cost);
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

/// Post-cast cooldown applied to the wizard after any charge spell fires.
/// While present, spell input is blocked so holding the cast button auto-
/// cycles at a defined interval rather than firing as fast as cast_time
/// allows. Does not affect channeled spells (they use `CastingState::Channeling`
/// which isn't a "completion" event) or Magic Missile (which has its own
/// `MagicMissileCooldown` with talent modifiers).
#[derive(Component, Debug, Clone, Copy)]
pub struct GlobalCastCooldown {
    pub remaining: f32,
}

/// Seconds of post-cast cooldown on every charge spell.
pub const GLOBAL_CAST_COOLDOWN_SECS: f32 = 1.0;

/// Marker component to track when the wizard is actively casting a spell.
///
/// This marker is added when a spell cast begins and removed immediately after
/// the spell completes. It prevents immediate recast while the mouse button is
/// held down, but allows immediate casting on the next mouse press without
/// requiring a double-click.
///
/// The marker stores an optional entity (like a circle indicator) that should
/// be despawned when the cast is cancelled or completed.
#[derive(Component)]
pub struct SpellCaster {
    /// Optional entity to despawn when cast is cancelled/completed (e.g., circle indicator).
    pub indicator_entity: Option<Entity>,
}

impl SpellCaster {
    /// Creates a new SpellCaster marker with no indicator.
    pub const fn new() -> Self {
        Self {
            indicator_entity: None,
        }
    }

    /// Creates a new SpellCaster marker with an indicator entity.
    pub const fn with_indicator(indicator_entity: Entity) -> Self {
        Self {
            indicator_entity: Some(indicator_entity),
        }
    }
}

/// Marker for the locally-controlled wizard.
///
/// In single-player, the one wizard gets this marker.
/// In multiplayer, each peer's own wizard gets this marker.
/// Spell casting systems query `With<LocalWizard>` instead of `With<Wizard>`
/// to ensure `single()` works with two wizard entities present.
#[derive(Component)]
pub struct LocalWizard;

/// Tracks the wizard sprite sheet animation state.
#[derive(Component)]
pub struct WizardAnimation {
    pub current_frame: usize,
    pub elapsed: f32,
}

impl WizardAnimation {
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            elapsed: 0.0,
        }
    }

    /// Advances the animation timer and returns true if frame changed.
    pub fn tick(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        if self.elapsed >= super::constants::WIZARD_FRAME_DURATION {
            self.elapsed -= super::constants::WIZARD_FRAME_DURATION;
            self.current_frame = (self.current_frame + 1) % super::constants::WIZARD_SPRITE_FRAMES;
            true
        } else {
            false
        }
    }

    /// Calculates UV offset for current frame in a 3x3 grid.
    pub fn uv_offset(&self) -> (f32, f32) {
        let grid_size = super::constants::WIZARD_SPRITE_GRID_SIZE as f32;
        let frame_size = 1.0 / grid_size;

        let row = (self.current_frame / super::constants::WIZARD_SPRITE_GRID_SIZE) as f32;
        let col = (self.current_frame % super::constants::WIZARD_SPRITE_GRID_SIZE) as f32;

        (col * frame_size, row * frame_size)
    }
}

/// Stores the wizard sprite sheet texture handle.
#[derive(Resource)]
pub struct WizardAssets {
    pub sprite_texture: Handle<Image>,
}

/// Marker for the guest's wizard entity on the host.
///
/// Only added in multiplayer on the host. The host processes incoming
/// `SpellCommand` messages and drives this wizard's casting state.
#[derive(Component)]
pub struct GuestWizard;

/// Abstract wizard input for spell casting — same shape regardless of source.
///
/// Built from mouse state + camera raycast.
/// Spell casting logic consumes this without knowing the input source.
pub struct WizardInput {
    /// True on the frame the cast button was first pressed.
    pub just_pressed: bool,
    /// True while the cast button is held.
    pub pressed: bool,
    /// True on the frame the cast button was released.
    pub just_released: bool,
    /// Cursor world position on the battlefield (Y=0 plane).
    pub cursor_pos: Option<Vec3>,
}
