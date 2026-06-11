/// Timer resource that drives the channel-change animation.
/// Inserted when a `ChannelChangeMessage` is received, removed when finished.
#[derive(bevy::prelude::Resource)]
pub(crate) struct ChannelChangeTimer {
    pub elapsed: f32,
    pub duration: f32,
}

impl ChannelChangeTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
        }
    }

    /// Returns the current intensity (0→1→0) using a sine curve.
    pub fn intensity(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin()
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Timer resource that drives the screen flash animation.
/// Inserted when a `ScreenFlashMessage` is received, removed when finished.
#[derive(bevy::prelude::Resource)]
pub(crate) struct ScreenFlashTimer {
    pub elapsed: f32,
    pub duration: f32,
    pub color: [f32; 3],
    pub peak_intensity: f32,
}

impl ScreenFlashTimer {
    pub fn new(color: [f32; 3], duration: f32, peak_intensity: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
            color,
            peak_intensity,
        }
    }

    /// Returns the current intensity (0→peak→0) using a sine curve.
    pub fn intensity(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin() * self.peak_intensity
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Timer resource that drives the vignette pulse animation.
#[derive(bevy::prelude::Resource)]
pub(crate) struct VignettePulseTimer {
    pub elapsed: f32,
    pub duration: f32,
    pub peak_intensity: f32,
}

impl VignettePulseTimer {
    pub fn new(duration: f32, peak_intensity: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
            peak_intensity,
        }
    }

    /// Returns the current intensity (0→peak→0) using a sine curve.
    pub fn intensity(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin() * self.peak_intensity
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Timer resource that drives the screen desaturation animation.
/// Inserted when a `ScreenDesaturateMessage` is received, removed when finished.
#[derive(bevy::prelude::Resource)]
pub(crate) struct DesaturationTimer {
    pub elapsed: f32,
    pub duration: f32,
}

impl DesaturationTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
        }
    }

    /// Returns the current intensity (0→1→0) using a sine curve.
    pub fn intensity(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin()
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}
