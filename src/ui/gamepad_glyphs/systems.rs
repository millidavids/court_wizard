use bevy::prelude::*;

use super::resources::{CurrentControllerGlyphStyle, GamepadGlyphFonts};
use crate::config::{ControllerGlyphStyle, GameConfig};
use crate::game::input::action_state::{ControllerKind, GamepadActionState};
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
/// (`GameConfig.controller_glyph_style`) and, when set to `Auto`, with the active
/// controller's type. The active producer (Steam Input poll or gilrs adapter)
/// resolves `GamepadActionState.controller_kind` — from Steam's
/// `GetInputTypeForHandle` or the USB vendor id respectively — so this is correct
/// on both paths.
pub(super) fn resolve_glyph_style(
    config: Res<GameConfig>,
    active: Res<ActiveInputDevice>,
    action_state: Res<GamepadActionState>,
    mut current: ResMut<CurrentControllerGlyphStyle>,
) {
    let style = match config.controller_glyph_style {
        ControllerGlyphStyle::Auto if active.is_gamepad() => {
            kind_to_style(action_state.controller_kind)
        }
        // Mouse/keyboard (no controller) falls back to the broadly-recognized
        // A/B/X/Y convention.
        ControllerGlyphStyle::Auto => ControllerGlyphStyle::Xbox,
        explicit => explicit,
    };
    if current.0 != style {
        current.0 = style;
    }
}

fn kind_to_style(kind: ControllerKind) -> ControllerGlyphStyle {
    match kind {
        ControllerKind::Xbox => ControllerGlyphStyle::Xbox,
        ControllerKind::PlayStation => ControllerGlyphStyle::PlayStation,
        ControllerKind::Switch => ControllerGlyphStyle::Switch,
        ControllerKind::SteamDeck => ControllerGlyphStyle::SteamDeck,
    }
}
