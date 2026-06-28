//! Controller glyph font + lookup infrastructure.
//!
//! Loads the four Kenney input fonts (Xbox, PlayStation, Steam Deck,
//! Nintendo Switch 2) as asset handles and provides a stable API for
//! picking the right glyph + font for a given `GamepadButton`. Consumers
//! read the resolved `CurrentControllerGlyphStyle` resource and call
//! `glyph_char` / `glyph_font` when they want to render a button prompt.

mod plugin;
mod resources;
mod systems;

pub(crate) use plugin::GamepadGlyphsPlugin;
pub(crate) use resources::{
    CurrentControllerGlyphStyle, GamepadGlyphFonts, SteamGlyphs, glyph_char,
};
