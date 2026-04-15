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
pub(super) const CHANNEL_CHANGE_DURATION: f32 = 0.3;

/// Duration of the screen desaturation pulse in seconds.
pub(super) const DESATURATION_DURATION: f32 = 0.15;

/// Gravitational lensing distortion strength (how far pixels are pulled toward black holes).
pub(super) const LENSING_STRENGTH: f32 = 0.015;

/// Multiplier for black hole visual radius to get screen-space influence radius.
pub(super) const LENSING_INFLUENCE_MULT: f32 = 3.5;

// --- Colorblind correction matrices (Machado et al. 2009, severity 1.0) ---
// Each is a row-major 3x3 simulation matrix that maps linear RGB to simulated CVD vision.

/// Protanopia (red-blind) simulation matrix.
#[rustfmt::skip]
pub(super) const PROTAN_SIM: [f32; 9] = [
    0.152286,  1.052583, -0.204868,
    0.114503,  0.786281,  0.099216,
   -0.003882, -0.048116,  1.051998,
];

/// Deuteranopia (green-blind) simulation matrix.
#[rustfmt::skip]
pub(super) const DEUTAN_SIM: [f32; 9] = [
    0.367322,  0.860646, -0.227968,
    0.280085,  0.672501,  0.047413,
   -0.011820,  0.042940,  0.968881,
];

/// Tritanopia (blue-blind) simulation matrix.
#[rustfmt::skip]
pub(super) const TRITAN_SIM: [f32; 9] = [
    1.255528, -0.076749, -0.178779,
   -0.078411,  0.930809,  0.147602,
    0.004733,  0.691367,  0.303900,
];

/// Error redistribution matrix for protanopia/deuteranopia.
/// Shifts lost red/green info into green+blue channels.
#[rustfmt::skip]
pub(super) const PROTAN_DEUTAN_ERR: [f32; 9] = [
    0.0, 0.0, 0.0,
    0.7, 1.0, 0.0,
    0.7, 0.0, 1.0,
];

/// Error redistribution matrix for tritanopia.
/// Shifts lost blue info into red+green channels.
#[rustfmt::skip]
pub(super) const TRITAN_ERR: [f32; 9] = [
    1.0, 0.0, 0.7,
    0.0, 1.0, 0.7,
    0.0, 0.0, 0.0,
];
