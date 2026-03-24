pub const SKYBOX_PATH: &str = "images/static_sprites/menu_skybox.png";
pub const BACKGROUND_PATH: &str = "images/static_sprites/menu_background.png";
pub const FOREGROUND_PATH: &str = "images/static_sprites/menu_foreground.png";

/// Per-layer configuration: (speed px/s, image width px, z-index).
pub struct LayerConfig {
    pub speed: f32,
    pub width: f32,
    pub z_index: i32,
}

const BASE_SPEED: f32 = 10.0;

pub const SKYBOX: LayerConfig = LayerConfig {
    speed: BASE_SPEED,
    width: 5760.0,
    z_index: -3,
};

pub const BACKGROUND: LayerConfig = LayerConfig {
    speed: BASE_SPEED * 2.0,
    width: 3840.0,
    z_index: -2,
};

pub const FOREGROUND: LayerConfig = LayerConfig {
    speed: BASE_SPEED * 3.0,
    width: 3840.0,
    z_index: -1,
};
