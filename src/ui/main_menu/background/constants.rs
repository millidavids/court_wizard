pub const SKYBOX_PATH: &str = "images/static_sprites/menu_skybox.png";
pub const BACKGROUND_PATH: &str = "images/static_sprites/menu_background.png";
pub const FOREGROUND_PATH: &str = "images/static_sprites/menu_foreground.png";

/// Per-layer configuration.
///
/// Both dimensions are expressed as a percentage of the viewport rather than in
/// pixels. `Val::Px` in Bevy UI is multiplied by the display scale factor *and*
/// `UiScale`, so sizing a strip by its source image's pixel width made the node
/// balloon with screen size — at 4K that pushed the skybox strip past the GPU's
/// texture dimension ceiling and it silently stopped rendering, leaving the left
/// side of the menu bare.
pub struct LayerConfig {
    /// Scroll speed as a percentage of viewport width per second.
    pub speed: f32,
    /// Strip width as a percentage of viewport width.
    ///
    /// Set to the source image's aspect ratio divided by the viewport's, so the
    /// art keeps its proportions when scaled to the viewport height:
    /// `(image_w / image_h) / (16 / 9) * 100`.
    pub width_percent: f32,
    pub z_index: i32,
}

/// Percent of viewport width per second for the slowest (furthest) layer.
const BASE_SPEED: f32 = 0.8;

/// 5760x1080 art on a 16:9 viewport is three screens wide.
pub const SKYBOX: LayerConfig = LayerConfig {
    speed: BASE_SPEED,
    width_percent: 300.0,
    z_index: -3,
};

/// 3840x1080 art on a 16:9 viewport is two screens wide.
pub const BACKGROUND: LayerConfig = LayerConfig {
    speed: BASE_SPEED * 2.0,
    width_percent: 200.0,
    z_index: -2,
};

pub const FOREGROUND: LayerConfig = LayerConfig {
    speed: BASE_SPEED * 3.0,
    width_percent: 200.0,
    z_index: -1,
};
