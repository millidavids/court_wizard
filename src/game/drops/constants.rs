use bevy::prelude::*;

/// Chance per enemy kill to spawn an ingredient drop (0.0 to 1.0).
pub(super) const DROP_CHANCE: f64 = 0.005;

/// Size of the drop billboard sprite (width and height).
pub(super) const DROP_SPRITE_SIZE: f32 = 32.0;

/// Y position of the drop (slightly above ground).
pub(super) const DROP_BASE_Y: f32 = 12.0;

/// Amplitude of the bobbing animation (units up and down from base).
pub(super) const BOB_AMPLITUDE: f32 = 5.0;

/// Frequency of the bobbing animation (Hz).
pub(super) const BOB_FREQUENCY: f32 = 1.5;

/// Frequency of the pulse (scale) animation (Hz).
pub(super) const PULSE_FREQUENCY: f32 = 1.0;

/// Amplitude of the pulse animation (fraction of base scale).
pub(super) const PULSE_AMPLITUDE: f32 = 0.2;

/// Speed at which a collected drop flies to the wizard (units per second).
pub(super) const FLY_SPEED: f32 = 2000.0;

/// Distance from wizard at which a flying drop is considered collected.
pub(super) const COLLECTION_DISTANCE: f32 = 50.0;

/// Maximum Y height during the fly arc.
pub(super) const FLY_ARC_HEIGHT: f32 = 200.0;

/// Drop glow color (generic magical shimmer — white/gold).
pub(super) const DROP_COLOR: Color = Color::srgb(0.95, 0.9, 0.6);

/// Drop emissive color for glow effect.
pub(super) const DROP_EMISSIVE: Color = Color::srgb(2.0, 1.8, 1.0);
