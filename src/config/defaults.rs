use super::resources::*;

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            mode: WindowMode::default(),
            vsync: VsyncMode::default(),
            scale_factor: Some(1.0),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 0.8,
        }
    }
}
