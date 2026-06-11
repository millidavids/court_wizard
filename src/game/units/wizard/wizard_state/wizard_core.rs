use bevy::prelude::*;

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

/// Marker for the locally-controlled wizard.
///
/// In single-player, the one wizard gets this marker.
/// In multiplayer, each peer's own wizard gets this marker.
/// Spell casting systems query `With<LocalWizard>` instead of `With<Wizard>`
/// to ensure `single()` works with two wizard entities present.
#[derive(Component)]
pub struct LocalWizard;

/// Stores the wizard sprite sheet texture handles.
#[derive(Resource)]
pub struct WizardAssets {
    /// Default wizard idle sheet (host / single-player).
    pub sprite_texture: Handle<Image>,
    /// Distinct idle sheet for the co-op GUEST wizard so the two players are
    /// visually distinguishable on the shared wall.
    pub guest_sprite_texture: Handle<Image>,
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
