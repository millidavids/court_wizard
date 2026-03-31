use bevy::prelude::*;

/// Broadcast message requesting a CRT channel-change flicker effect.
///
/// Send this alongside any state transition that should show the effect:
/// ```ignore
/// commands.send(ChannelChangeMessage);
/// ```
#[derive(Message)]
pub(crate) struct ChannelChangeMessage;

/// Broadcast message requesting a brief screen desaturation pulse.
#[derive(Message)]
pub(crate) struct ScreenDesaturateMessage;

/// Broadcast message requesting a screen flash effect with a specific color.
#[derive(Message)]
pub(crate) struct ScreenFlashMessage {
    /// Flash color (RGB, 0–1).
    pub color: [f32; 3],
    /// Flash duration in seconds.
    pub duration: f32,
    /// Peak intensity (0.0–1.0).
    pub intensity: f32,
}

/// Broadcast message requesting a brief vignette darkening pulse.
#[derive(Message)]
pub(crate) struct VignettePulseMessage {
    /// Duration of the pulse in seconds.
    pub duration: f32,
    /// Peak extra vignette intensity (0.0–1.0).
    pub intensity: f32,
}
