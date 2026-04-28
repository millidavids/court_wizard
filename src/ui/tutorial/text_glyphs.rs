//! Inline-glyph templating for tutorial text.
//!
//! Tutorial step text may contain `{token}` placeholders that render as a
//! controller glyph (in the active vendor font) when a gamepad is the active
//! input device, or as a plain-text keyboard fallback otherwise. This keeps
//! one source string per step that adapts to the player's input.

use bevy::input::gamepad::GamepadButton;
use bevy::prelude::*;

use crate::config::ControllerGlyphStyle;
use crate::ui::gamepad_glyphs::{GamepadGlyphFonts, glyph_char};

#[derive(Debug, Clone, Copy)]
pub(super) enum GlyphToken {
    South,
    East,
    North,
    West,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LeftTrigger,
    RightTrigger,
    LeftBumper,
    RightBumper,
    Start,
    Select,
    LeftStick,
    RightStick,
}

impl GlyphToken {
    fn from_tag(tag: &str) -> Option<Self> {
        Some(match tag {
            "south" => Self::South,
            "east" => Self::East,
            "north" => Self::North,
            "west" => Self::West,
            "dpad_up" => Self::DPadUp,
            "dpad_down" => Self::DPadDown,
            "dpad_left" => Self::DPadLeft,
            "dpad_right" => Self::DPadRight,
            "lt" => Self::LeftTrigger,
            "rt" => Self::RightTrigger,
            "lb" => Self::LeftBumper,
            "rb" => Self::RightBumper,
            "start" => Self::Start,
            "select" => Self::Select,
            "lstick" => Self::LeftStick,
            "rstick" => Self::RightStick,
            _ => return None,
        })
    }

    pub(super) fn button(self) -> GamepadButton {
        match self {
            Self::South => GamepadButton::South,
            Self::East => GamepadButton::East,
            Self::North => GamepadButton::North,
            Self::West => GamepadButton::West,
            Self::DPadUp => GamepadButton::DPadUp,
            Self::DPadDown => GamepadButton::DPadDown,
            Self::DPadLeft => GamepadButton::DPadLeft,
            Self::DPadRight => GamepadButton::DPadRight,
            Self::LeftTrigger => GamepadButton::LeftTrigger2,
            Self::RightTrigger => GamepadButton::RightTrigger2,
            Self::LeftBumper => GamepadButton::LeftTrigger,
            Self::RightBumper => GamepadButton::RightTrigger,
            Self::Start => GamepadButton::Start,
            Self::Select => GamepadButton::Select,
            Self::LeftStick => GamepadButton::LeftThumb,
            Self::RightStick => GamepadButton::RightThumb,
        }
    }

    /// Plain-text label shown when the player is on keyboard/mouse.
    /// Most KBM interaction is mouse-driven, so prefer descriptive verbs over
    /// key names. Steps that read awkwardly with these defaults should set
    /// `TutorialStep::text_kbm` to a fully-rewritten KBM string.
    pub(super) fn keyboard_label(self) -> &'static str {
        match self {
            Self::South => "click",
            Self::East => "Esc",
            Self::North => "click",
            Self::West => "click",
            Self::DPadUp => "click",
            Self::DPadDown => "click",
            Self::DPadLeft => "click",
            Self::DPadRight => "click",
            Self::LeftTrigger => "Right Click",
            Self::RightTrigger => "Left Click",
            Self::LeftBumper => "click a tab",
            Self::RightBumper => "click a tab",
            Self::Start => "Esc",
            Self::Select => "click the spell book button",
            Self::LeftStick => "the mouse",
            Self::RightStick => "click slot 1–5",
        }
    }
}

pub(super) enum TextSegment<'a> {
    Plain(&'a str),
    Glyph(GlyphToken),
}

/// Splits `text` into plain runs and glyph tokens. Unknown `{...}` blocks
/// pass through as plain text so we don't lose content on a typo.
pub(super) fn parse_tutorial_text(text: &str) -> Vec<TextSegment<'_>> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        if start > 0 {
            out.push(TextSegment::Plain(&rest[..start]));
        }
        let after_brace = &rest[start + 1..];
        let Some(end) = after_brace.find('}') else {
            out.push(TextSegment::Plain(&rest[start..]));
            return out;
        };
        let tag = &after_brace[..end];
        match GlyphToken::from_tag(tag) {
            Some(token) => out.push(TextSegment::Glyph(token)),
            None => out.push(TextSegment::Plain(&rest[start..start + 1 + end + 1])),
        }
        rest = &after_brace[end + 1..];
    }
    if !rest.is_empty() {
        out.push(TextSegment::Plain(rest));
    }
    out
}

/// Spawns a segmented `Text` entity as a child of `parent`, returning its
/// entity id so callers can attach markers.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_segmented_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    base_font_size: f32,
    base_color: Color,
    max_width: f32,
    gamepad_active: bool,
    style: ControllerGlyphStyle,
    glyph_fonts: Option<&GamepadGlyphFonts>,
) -> Entity {
    let mut entity = parent.spawn((
        Text::default(),
        TextFont::from_font_size(base_font_size),
        TextColor(base_color),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            max_width: Val::Px(max_width),
            ..default()
        },
    ));
    let id = entity.id();

    entity.with_children(|p| {
        spawn_text_spans(
            p,
            text,
            base_font_size,
            base_color,
            gamepad_active,
            style,
            glyph_fonts,
        );
    });

    id
}

/// Inner span-loop shared by `spawn_segmented_text` (initial spawn) and the
/// in-place rebuild in `update_tutorial_content`. Spawns one `TextSpan` per
/// parsed segment as a child of `parent`.
pub(super) fn spawn_text_spans(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    base_font_size: f32,
    base_color: Color,
    gamepad_active: bool,
    style: ControllerGlyphStyle,
    glyph_fonts: Option<&GamepadGlyphFonts>,
) {
    let glyph_font_size = base_font_size * 4.8;
    for seg in parse_tutorial_text(text) {
        match seg {
            TextSegment::Plain(s) => {
                parent.spawn((
                    TextSpan::new(s.to_string()),
                    TextFont::from_font_size(base_font_size),
                    TextColor(base_color),
                ));
            }
            TextSegment::Glyph(token) => {
                if gamepad_active
                    && let Some(fonts) = glyph_fonts
                    && let Some(ch) = glyph_char(token.button(), style)
                {
                    parent.spawn((
                        TextSpan::new(ch.to_string()),
                        TextFont {
                            font: fonts.font_for(style),
                            font_size: glyph_font_size,
                            ..default()
                        },
                        TextColor(base_color),
                    ));
                } else {
                    parent.spawn((
                        TextSpan::new(token.keyboard_label().to_string()),
                        TextFont::from_font_size(base_font_size),
                        TextColor(base_color),
                    ));
                }
            }
        }
    }
}
