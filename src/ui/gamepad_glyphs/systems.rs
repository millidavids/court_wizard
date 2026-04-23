use bevy::prelude::*;

use super::resources::{CurrentControllerGlyphStyle, GamepadGlyphFonts};
use crate::config::{ControllerGlyphStyle, GameConfig};
use crate::game::input::gamepad::resources::ActiveInputDevice;

/// Loads the four Kenney controller fonts at startup.
pub(super) fn load_glyph_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GamepadGlyphFonts {
        xbox: asset_server.load("fonts/kenney_input_xbox_series.otf"),
        playstation: asset_server.load("fonts/kenney_input_playstation_series.otf"),
        steam_deck: asset_server.load("fonts/kenney_input_steam_deck.otf"),
        switch: asset_server.load("fonts/kenney_input_nintendo_switch_2.otf"),
    });
}

/// Keeps `CurrentControllerGlyphStyle` synced with the user's preference
/// (`GameConfig.controller_glyph_style`) and, when set to `Auto`, with the
/// active gamepad's vendor.
pub(super) fn resolve_glyph_style(
    config: Res<GameConfig>,
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut current: ResMut<CurrentControllerGlyphStyle>,
) {
    let style = match config.controller_glyph_style {
        ControllerGlyphStyle::Auto => auto_detect(&active, &gamepads),
        explicit => explicit,
    };
    if current.0 != style {
        current.0 = style;
    }
}

/// Best-effort vendor sniff via USB vendor id. Bevy's `Gamepad` exposes
/// `vendor_id()` but doesn't expose a display name, so we rely on the
/// well-known vendor IDs for the platforms we support. Anything else falls
/// back to Xbox glyphs since the A/B/X/Y convention is broadly recognized.
fn auto_detect(active: &ActiveInputDevice, gamepads: &Query<&Gamepad>) -> ControllerGlyphStyle {
    let entity = match active {
        ActiveInputDevice::Gamepad(e) => *e,
        ActiveInputDevice::MouseKeyboard => return ControllerGlyphStyle::Xbox,
    };
    let Ok(gamepad) = gamepads.get(entity) else {
        return ControllerGlyphStyle::Xbox;
    };
    match gamepad.vendor_id() {
        Some(0x054C) => ControllerGlyphStyle::PlayStation, // Sony
        Some(0x057E) => ControllerGlyphStyle::Switch,      // Nintendo
        Some(0x28DE) => ControllerGlyphStyle::SteamDeck,   // Valve
        _ => ControllerGlyphStyle::Xbox,
    }
}
