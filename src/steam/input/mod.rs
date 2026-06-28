//! Steam Input API integration: action-based controller reading, Steam glyph art,
//! and haptics. The gilrs path in `crate::game::input::gamepad` remains as a
//! fallback when no Steam Input controller is present.

mod action_sets;
mod glyphs;
pub(crate) mod handles;
mod haptics;
mod init;
mod plugin;
mod poll;
mod run_frame;

pub(crate) use plugin::SteamInputPlugin;
