use bevy::input::gamepad::GamepadButton;
use bevy::prelude::*;

use crate::config::ControllerGlyphStyle;

/// Loaded handles for the four Kenney input fonts. One entry per vendor
/// style; consumers pick a handle via `font_for`.
#[derive(Resource, Debug, Clone)]
pub(crate) struct GamepadGlyphFonts {
    pub xbox: Handle<Font>,
    pub playstation: Handle<Font>,
    pub steam_deck: Handle<Font>,
    pub switch: Handle<Font>,
}

impl GamepadGlyphFonts {
    /// Returns the font handle for a concrete (non-Auto) style. Resolving
    /// `Auto` is done upstream in `CurrentControllerGlyphStyle`.
    pub(crate) fn font_for(&self, style: ControllerGlyphStyle) -> Handle<Font> {
        match style {
            ControllerGlyphStyle::Auto | ControllerGlyphStyle::Xbox => self.xbox.clone(),
            ControllerGlyphStyle::PlayStation => self.playstation.clone(),
            ControllerGlyphStyle::SteamDeck => self.steam_deck.clone(),
            ControllerGlyphStyle::Switch => self.switch.clone(),
        }
    }
}

/// The style the UI should actually render with this frame — either the
/// user's explicit pick from `GameConfig.controller_glyph_style`, or an
/// auto-detected vendor when `Auto`. Populated by `resolve_glyph_style` and
/// defaults to `Xbox` as the most broadly recognizable fallback.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurrentControllerGlyphStyle(pub ControllerGlyphStyle);

impl Default for CurrentControllerGlyphStyle {
    fn default() -> Self {
        Self(ControllerGlyphStyle::Xbox)
    }
}

impl CurrentControllerGlyphStyle {
    pub(crate) fn font_handle(&self, fonts: &GamepadGlyphFonts) -> Handle<Font> {
        fonts.font_for(self.0)
    }
}

/// Returns the Kenney-font codepoint for the given `GamepadButton` in the
/// given style, or `None` if that particular button has no assigned glyph
/// (dpad diagonals, mode, etc. — the game currently doesn't prompt for
/// those). Codepoints are taken from `assets/fonts/kenney_input_*_map.txt`.
#[allow(clippy::too_many_lines)]
pub(crate) fn glyph_char(button: GamepadButton, style: ControllerGlyphStyle) -> Option<char> {
    // Resolve Auto to the default fallback (Xbox) at the callsite; this fn
    // expects a concrete style.
    let style = match style {
        ControllerGlyphStyle::Auto => ControllerGlyphStyle::Xbox,
        other => other,
    };

    let code = match style {
        ControllerGlyphStyle::Xbox => match button {
            GamepadButton::South => 0xE004,         // xbox_button_a
            GamepadButton::East => 0xE006,          // xbox_button_b
            GamepadButton::West => 0xE01E,          // xbox_button_x
            GamepadButton::North => 0xE020,         // xbox_button_y
            GamepadButton::LeftTrigger => 0xE043,   // xbox_lb
            GamepadButton::LeftTrigger2 => 0xE047,  // xbox_lt
            GamepadButton::RightTrigger => 0xE049,  // xbox_rb
            GamepadButton::RightTrigger2 => 0xE04D, // xbox_rt
            GamepadButton::DPadUp => 0xE035,
            GamepadButton::DPadDown => 0xE024,
            GamepadButton::DPadLeft => 0xE028,
            GamepadButton::DPadRight => 0xE02B,
            GamepadButton::LeftThumb => 0xE04F,  // xbox_stick_l
            GamepadButton::RightThumb => 0xE057, // xbox_stick_r
            GamepadButton::Start => 0xE018,      // xbox_button_start
            GamepadButton::Select => 0xE008,     // xbox_button_back
            _ => return None,
        },
        ControllerGlyphStyle::PlayStation => match button {
            GamepadButton::South => 0xE049,         // playstation_button_cross
            GamepadButton::East => 0xE03F,          // playstation_button_circle
            GamepadButton::West => 0xE04F,          // playstation_button_square
            GamepadButton::North => 0xE051,         // playstation_button_triangle
            GamepadButton::LeftTrigger => 0xE076,   // playstation_trigger_l1
            GamepadButton::LeftTrigger2 => 0xE07A,  // playstation_trigger_l2
            GamepadButton::RightTrigger => 0xE07E,  // playstation_trigger_r1
            GamepadButton::RightTrigger2 => 0xE082, // playstation_trigger_r2
            GamepadButton::DPadUp => 0xE05E,
            GamepadButton::DPadDown => 0xE055,
            GamepadButton::DPadLeft => 0xE059,
            GamepadButton::DPadRight => 0xE05C,
            GamepadButton::LeftThumb => 0xE062,
            GamepadButton::RightThumb => 0xE06A,
            GamepadButton::Start => 0xE022, // playstation5_button_options
            GamepadButton::Select => 0xE01C, // playstation5_button_create
            _ => return None,
        },
        ControllerGlyphStyle::SteamDeck => match button {
            GamepadButton::South => 0xE001,
            GamepadButton::East => 0xE003,
            GamepadButton::West => 0xE01D,
            GamepadButton::North => 0xE01F,
            GamepadButton::LeftTrigger => 0xE007, // steamdeck_button_l1
            GamepadButton::LeftTrigger2 => 0xE009, // steamdeck_button_l2
            GamepadButton::RightTrigger => 0xE013, // steamdeck_button_r1
            GamepadButton::RightTrigger2 => 0xE015, // steamdeck_button_r2
            GamepadButton::DPadUp => 0xE02C,
            GamepadButton::DPadDown => 0xE023,
            GamepadButton::DPadLeft => 0xE027,
            GamepadButton::DPadRight => 0xE02A,
            GamepadButton::LeftThumb => 0xE030,
            GamepadButton::RightThumb => 0xE038,
            GamepadButton::Start => 0xE00F, // steamdeck_button_options
            GamepadButton::Select => 0xE01B, // steamdeck_button_view
            _ => return None,
        },
        ControllerGlyphStyle::Switch => match button {
            GamepadButton::South => 0xE004,         // switch_button_a
            GamepadButton::East => 0xE006,          // switch_button_b
            GamepadButton::West => 0xE01E,          // switch_button_x
            GamepadButton::North => 0xE020,         // switch_button_y
            GamepadButton::LeftTrigger => 0xE010,   // switch_button_l
            GamepadButton::LeftTrigger2 => 0xE022,  // switch_button_zl
            GamepadButton::RightTrigger => 0xE016,  // switch_button_r
            GamepadButton::RightTrigger2 => 0xE024, // switch_button_zr
            GamepadButton::DPadUp => 0xE042,
            GamepadButton::DPadDown => 0xE039,
            GamepadButton::DPadLeft => 0xE03D,
            GamepadButton::DPadRight => 0xE040,
            GamepadButton::LeftThumb => 0xE064,
            GamepadButton::RightThumb => 0xE06C,
            GamepadButton::Start => 0xE014,  // switch_button_plus
            GamepadButton::Select => 0xE012, // switch_button_minus
            _ => return None,
        },
        ControllerGlyphStyle::Auto => unreachable!("Auto normalized above"),
    };
    char::from_u32(code)
}
