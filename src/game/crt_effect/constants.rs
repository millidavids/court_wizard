/// Barrel distortion strength — how much the screen curves outward at edges.
pub(super) const DEFAULT_BARREL_DISTORTION: f32 = 0.15;

/// How dark the scanlines appear (0.0 = invisible, 1.0 = fully black).
pub(super) const DEFAULT_SCANLINE_INTENSITY: f32 = 0.25;

/// Number of horizontal scanlines across the screen height.
pub(super) const DEFAULT_SCANLINE_COUNT: f32 = 400.0;

/// Visibility of the RGB subpixel grid (0.0 = off, 1.0 = full channel masking).
pub(super) const DEFAULT_RGB_GRID_INTENSITY: f32 = 0.2;

/// Edge/corner darkening strength (0.0 = off, 1.0 = heavy vignette).
pub(super) const DEFAULT_VIGNETTE_INTENSITY: f32 = 0.3;

/// How large the bright center region is before vignette kicks in (smaller = more darkening).
pub(super) const DEFAULT_VIGNETTE_RADIUS: f32 = 0.8;

/// RGB channel separation strength — increases toward screen edges.
pub(super) const DEFAULT_CHROMATIC_ABERRATION: f32 = 0.003;

/// Subtle brightness oscillation amount (0.0 = off, ~0.03 = subtle).
pub(super) const DEFAULT_FLICKER_INTENSITY: f32 = 0.03;

/// How much the screen corners are rounded off (0.0 = sharp, 0.1+ = very round).
/// Must be larger than barrel distortion to be visible beyond its natural rounding.
pub(super) const DEFAULT_CORNER_RADIUS: f32 = 0.14;

/// Phosphor bloom strength on bright areas (0.0 = off).
pub(super) const DEFAULT_GLOW_INTENSITY: f32 = 0.2;

/// Duration of the channel-change flicker effect in seconds.
pub(super) const CHANNEL_CHANGE_DURATION: f32 = 0.4;

/// Duration of the screen desaturation pulse in seconds.
pub(super) const DESATURATION_DURATION: f32 = 0.15;
