use bevy::prelude::*;

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
        if self.elapsed >= super::super::constants::WIZARD_FRAME_DURATION {
            self.elapsed -= super::super::constants::WIZARD_FRAME_DURATION;
            self.current_frame =
                (self.current_frame + 1) % super::super::constants::WIZARD_SPRITE_FRAMES;
            true
        } else {
            false
        }
    }

    /// Calculates UV offset for current frame in a 3x3 grid.
    pub fn uv_offset(&self) -> (f32, f32) {
        let grid_size = super::super::constants::WIZARD_SPRITE_GRID_SIZE as f32;
        let frame_size = 1.0 / grid_size;

        let row = (self.current_frame / super::super::constants::WIZARD_SPRITE_GRID_SIZE) as f32;
        let col = (self.current_frame % super::super::constants::WIZARD_SPRITE_GRID_SIZE) as f32;

        (col * frame_size, row * frame_size)
    }
}
