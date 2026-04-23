//! Staff-to-ground aim line drawn while a gamepad is the active input device.
//!
//! Gives the player a visible analog to the mouse cursor's ground-plane
//! aim point when playing on a controller. The line originates at the
//! wizard's staff tip and terminates at the current aim target, clamped
//! to the wizard's spell range.

mod plugin;
mod systems;

pub(in crate::game) use plugin::AimLinePlugin;
