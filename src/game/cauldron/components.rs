use bevy::prelude::*;

use super::brews::Recipe;

/// Marker component for the cauldron entity.
#[derive(Component)]
pub struct Cauldron;

/// Expanding translucent sphere that plays when a brew completes.
#[derive(Component)]
pub struct BrewBubble {
    pub time_alive: f32,
    pub duration: f32,
    pub color: Color,
}

/// Tracks the current state of the cauldron.
#[derive(Debug, Clone, Component, Default)]
pub enum CauldronState {
    /// Ready to start a brew.
    #[default]
    Idle,
    /// Currently brewing — wizard cannot cast spells.
    Brewing {
        recipe: Recipe,
        elapsed: f32,
        duration: f32,
    },
    /// Short cooldown between brews.
    #[allow(dead_code)]
    Cooldown { remaining: f32 },
}

impl CauldronState {
    /// Returns true if the cauldron is currently brewing.
    pub fn is_brewing(&self) -> bool {
        matches!(self, Self::Brewing { .. })
    }

    /// Returns true if the cauldron is idle and ready for a new brew.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Cancels the current brew, returning to idle.
    pub fn cancel(&mut self) {
        *self = Self::Idle;
    }

    /// Starts brewing the given recipe.
    pub fn start_brewing(&mut self, recipe: Recipe, duration: f32) {
        *self = Self::Brewing {
            recipe,
            elapsed: 0.0,
            duration,
        };
    }

    /// Advances the brew timer. Returns Some(recipe) if brewing just completed.
    pub fn tick(&mut self, delta: f32) -> Option<Recipe> {
        match self {
            Self::Brewing {
                elapsed, duration, ..
            } => {
                *elapsed += delta;
                if *elapsed >= *duration {
                    // Take the recipe out before transitioning to Idle
                    let old = std::mem::take(self);
                    if let CauldronState::Brewing { recipe, .. } = old {
                        Some(recipe)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Self::Cooldown { remaining } => {
                *remaining -= delta;
                if *remaining <= 0.0 {
                    *self = Self::Idle;
                }
                None
            }
            Self::Idle => None,
        }
    }

    /// Returns brew progress as a percentage (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        match self {
            Self::Brewing {
                elapsed, duration, ..
            } => {
                if *duration > 0.0 {
                    (elapsed / duration).min(1.0)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
}

/// Tracks the cauldron sprite sheet animation state.
#[derive(Component)]
pub struct CauldronAnimation {
    pub current_frame: usize,
    pub elapsed: f32,
}

/// Damage bonus applied to defenders by cauldron buff (Mistletoe).
///
/// Stores a percentage bonus (e.g., 0.5 = +50% damage).
/// Added/removed by the cauldron buff system each frame.
#[derive(Component)]
pub struct CauldronDamageBonus(pub f32);

/// Damage resistance applied to defenders by cauldron buff (Wormwood).
///
/// Stores a resistance percentage (e.g., 0.25 = 25% damage reduction).
/// Added/removed by the cauldron buff system each frame.
#[derive(Component)]
pub struct CauldronDamageResistance(pub f32);

/// Speed modifier applied by cauldron buff (Meadowsweet/Valerian).
///
/// Stores a percentage modifier (positive for speed boost, negative for slow).
/// Added/removed by the cauldron buff system each frame.
#[derive(Component)]
pub struct CauldronSpeedModifier(pub f32);

/// Tracks brewing visual effects (pulsing and color).
#[derive(Component)]
pub struct CauldronBrewingEffects {
    pub pulse_timer: f32,
    pub original_scale: Vec3,
    pub recipe_color: Color,
}

impl CauldronAnimation {
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            elapsed: 0.0,
        }
    }

    /// Advances the animation timer and returns true if frame changed.
    pub fn tick(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        if self.elapsed >= super::constants::CAULDRON_FRAME_DURATION {
            self.elapsed -= super::constants::CAULDRON_FRAME_DURATION;
            self.current_frame =
                (self.current_frame + 1) % super::constants::CAULDRON_SPRITE_FRAMES;
            true
        } else {
            false
        }
    }

    /// Calculates UV offset for current frame in a 3x3 grid.
    pub fn uv_offset(&self) -> (f32, f32) {
        let grid_size = super::constants::CAULDRON_SPRITE_GRID_SIZE as f32;
        let frame_size = 1.0 / grid_size;

        let row = (self.current_frame / super::constants::CAULDRON_SPRITE_GRID_SIZE) as f32;
        let col = (self.current_frame % super::constants::CAULDRON_SPRITE_GRID_SIZE) as f32;

        (col * frame_size, row * frame_size)
    }
}
