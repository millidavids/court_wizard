use bevy::prelude::*;

use super::resources::{CurrentControllerGlyphStyle, SteamGlyphs};
use super::systems::{load_glyph_fonts, resolve_glyph_style};
use crate::config::GameConfig;
use crate::game::input::gamepad::resources::ActiveInputDevice;

/// Loads the Kenney controller-glyph fonts and keeps
/// `CurrentControllerGlyphStyle` synced with the user's preference + the
/// active gamepad's vendor.
pub(crate) struct GamepadGlyphsPlugin;

impl Plugin for GamepadGlyphsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentControllerGlyphStyle>()
            // Populated by `steam::input` when a Steam pad is active; empty
            // otherwise (callers fall back to the Kenney font placeholder).
            .init_resource::<SteamGlyphs>()
            .add_systems(Startup, load_glyph_fonts)
            // Only re-resolve when the user's glyph preference changes or the
            // active gamepad swaps. Otherwise the resource is already correct.
            .add_systems(
                Update,
                resolve_glyph_style.run_if(
                    resource_changed::<GameConfig>.or(resource_changed::<ActiveInputDevice>),
                ),
            );
    }
}
