use bevy::prelude::*;

use super::resources::CurrentControllerGlyphStyle;
use super::systems::{load_glyph_fonts, resolve_glyph_style};

/// Loads the Kenney controller-glyph fonts and keeps
/// `CurrentControllerGlyphStyle` synced with the user's preference + the
/// active gamepad's vendor.
pub(crate) struct GamepadGlyphsPlugin;

impl Plugin for GamepadGlyphsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentControllerGlyphStyle>()
            .add_systems(Startup, load_glyph_fonts)
            .add_systems(Update, resolve_glyph_style);
    }
}
