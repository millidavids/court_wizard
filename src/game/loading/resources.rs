use bevy::prelude::*;

/// Total frames to spread loading across.
pub const TOTAL_LOADING_FRAMES: u32 = 100;

/// Tracks the progress of progressive loading.
/// This is a simple frame counter - individual plugins track their own spawn progress.
#[derive(Resource, Default)]
pub struct LoadingProgress {
    /// Current frame in the loading sequence (0-99 for 100 frames).
    pub current_frame: u32,
}

impl LoadingProgress {
    pub fn new() -> Self {
        Self { current_frame: 0 }
    }

    pub fn is_complete(&self) -> bool {
        self.current_frame >= TOTAL_LOADING_FRAMES
    }

    pub fn advance(&mut self) {
        self.current_frame += 1;
    }
}
