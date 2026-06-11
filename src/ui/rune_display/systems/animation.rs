use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::units::wizard::archetypes::runes::resources::Rune;
use crate::ui::gamepad_glyphs::{CurrentControllerGlyphStyle, GamepadGlyphFonts, glyph_char};

/// System to handle the activated spell name fade timer.
pub(crate) fn update_spell_name_fade(
    time: Res<Time>,
    mut commands: Commands,
    mut fade_query: Query<
        (Entity, &mut SpellNameFadeTimer, &mut TextColor),
        With<ActivatedSpellText>,
    >,
    mut shadow_query: Query<
        (&mut Text, &mut TextColor),
        (With<ActivatedSpellTextShadow>, Without<ActivatedSpellText>),
    >,
) {
    for (entity, mut timer, mut color) in &mut fade_query {
        timer.elapsed += time.delta_secs();

        let alpha = (1.0 - (timer.elapsed / timer.duration)).max(0.0);
        color.0.set_alpha(alpha);

        // Fade shadow in sync
        if let Ok((_, mut shadow_color)) = shadow_query.single_mut() {
            shadow_color.0.set_alpha(alpha * 0.5);
        }

        if timer.elapsed >= timer.duration {
            commands.entity(entity).remove::<SpellNameFadeTimer>();
            color.0.set_alpha(0.0);

            // Clear both texts
            if let Ok((mut shadow_text, mut shadow_color)) = shadow_query.single_mut() {
                **shadow_text = "".to_string();
                shadow_color.0.set_alpha(0.0);
            }
        }
    }
}

/// Swaps each rune button's "Q/W/E/R" label for a D-pad direction glyph
/// when a gamepad is the active input device. Mapping matches the gameplay
/// binding laid out in the controller plan: Q=Up, W=Down, E=Right, R=Left.
pub(crate) fn adapt_rune_labels_to_input_device(
    active: Res<ActiveInputDevice>,
    style: Res<CurrentControllerGlyphStyle>,
    fonts: Option<Res<GamepadGlyphFonts>>,
    mut labels: Query<(&RuneButtonLabel, &mut Text, &mut TextFont)>,
) {
    let gamepad = active.is_gamepad();
    for (label, mut text, mut font) in &mut labels {
        if gamepad && let Some(ref fonts) = fonts {
            let button = match label.rune {
                Rune::Q => GamepadButton::DPadUp,
                Rune::W => GamepadButton::DPadDown,
                Rune::E => GamepadButton::DPadRight,
                Rune::R => GamepadButton::DPadLeft,
            };
            if let Some(glyph) = glyph_char(button, style.0) {
                let s = glyph.to_string();
                if **text != s {
                    **text = s;
                }
                let want = fonts.font_for(style.0);
                if font.font != want {
                    font.font = want;
                    font.font_size = RUNE_BUTTON_STYLE.font_size * 1.4;
                }
            }
        } else {
            let s = format!("{}", label.rune.as_char());
            if **text != s {
                **text = s;
            }
            if font.font != Handle::<Font>::default() {
                font.font = Handle::<Font>::default();
                font.font_size = RUNE_BUTTON_STYLE.font_size;
            }
        }
    }
}
