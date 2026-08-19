use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::config::input_bindings::InputBindings;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::input::messages::MouseClicked;
use crate::game::units::wizard::archetypes::runes::resources::Rune;
use crate::game::units::wizard::archetypes::runes::{LastActivatedSpell, RuneSequence};
use crate::ui::color_utils::border_bright;
use crate::ui::components::{ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront};
use crate::ui::constants::{
    BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST, BUTTON_PRESSED_OUTLINE, BUTTON_REST_OUTLINE,
};

/// Animates a rune button when its rune key (Q/W/E/R) is held — mirrors the action
/// bar's keyboard-driven press visuals (`update_action_bar`). The rune buttons are
/// driven by the rune keys, not mouse hover, so this gives the same depress + glow
/// feedback the action bar gives on the 1–5 keys. The global `animate_button_3d`
/// system moves the front face once we set `ButtonAnimState.target`.
pub(crate) fn update_rune_button_press_visuals(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut buttons: Query<(
        Entity,
        &RuneButton,
        &ButtonColors,
        &Children,
        Option<&mut ButtonAnimState>,
        Has<RuneKeyHeld>,
    )>,
    mut front_query: Query<&mut BorderColor, (With<ButtonFront>, Without<ButtonEdge>)>,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    for (entity, rune_button, colors, children, anim, is_held) in &mut buttons {
        let key = match rune_button.rune {
            Rune::Q => bindings.rune_caster.rune_1,
            Rune::W => bindings.rune_caster.rune_2,
            Rune::E => bindings.rune_caster.rune_3,
            Rune::R => bindings.rune_caster.rune_4,
        };
        let pressed = key.is_some_and(|k| keyboard.pressed(k));

        // Re-assert the depress target EVERY frame (not just on the press edge) so
        // the global `button_interaction` (mouse hover/leave) can't reset a button
        // back to rest while its key is still held.
        if let Some(mut anim) = anim {
            anim.target = if pressed {
                BUTTON_3D_OFFSET_PRESSED
            } else {
                BUTTON_3D_OFFSET_REST
            };
        }

        // The border/edge glow only changes on the press/release edges.
        if pressed && !is_held {
            commands.entity(entity).insert(RuneKeyHeld);
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(border_bright(colors.border));
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = BUTTON_PRESSED_OUTLINE;
                }
            }
        } else if !pressed && is_held {
            commands.entity(entity).remove::<RuneKeyHeld>();
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(colors.border);
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = BUTTON_REST_OUTLINE;
                }
            }
        }
    }
}

/// Handles rune button clicks by adding the rune to the sequence.
pub(crate) fn handle_rune_button_click(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&RuneButton>,
    mut sequence: ResMut<RuneSequence>,
) {
    for event in button_clicked.read() {
        if let Ok(rune_button) = button_query.get(event.button) {
            // Add rune to sequence (respects max length and prevents duplicates)
            if sequence.len()
                < crate::game::units::wizard::archetypes::runes::constants::MAX_RUNE_SEQUENCE_LENGTH
            {
                sequence.push(rune_button.rune);
            }
        }
    }
}

/// Updates the rune sequence display text based on current sequence.
/// Shows spell name briefly when a valid sequence is activated, then fades out.
#[allow(clippy::type_complexity)]
pub(crate) fn update_rune_display(
    sequence: Res<RuneSequence>,
    active: Res<ActiveInputDevice>,
    mut commands: Commands,
    mut sequence_text_query: Query<
        (
            Entity,
            &mut Text,
            &mut TextFont,
            Option<&SpellNameFadeTimer>,
            &mut TextColor,
        ),
        With<RuneSequenceText>,
    >,
    mut shadow_query: Query<
        (&mut Text, &mut TextFont),
        (With<RuneSequenceTextShadow>, Without<RuneSequenceText>),
    >,
) {
    // The rendered form depends on the sequence AND (on a controller) the active
    // device, so re-render when either changes — otherwise plugging in a
    // controller mid-sequence leaves stale letters.
    if !sequence.is_changed() && !active.is_changed() {
        return;
    }

    if let Ok((entity, mut text, mut font, fade_timer, mut color)) =
        sequence_text_query.single_mut()
    {
        if !sequence.is_empty() && fade_timer.is_some() {
            commands.entity(entity).remove::<SpellNameFadeTimer>();
            color.0.set_alpha(1.0);
        }

        if fade_timer.is_some() && sequence.is_empty() {
            return;
        }

        // On a controller the sequence renders as inline direction arrows (the
        // same `GamepadAction::direction_arrow` mapping as the rune buttons),
        // sized up a bit so they read clearly. Both arrows and letters render in
        // the default pixel font, so no font swap is needed.
        let (new_text, want_size) = if sequence.is_empty() {
            (String::new(), RUNE_SEQUENCE_FONT_SIZE)
        } else if active.is_gamepad() {
            let arrows: String = sequence
                .runes
                .iter()
                .filter_map(|&rune| rune.dpad_action().direction_arrow())
                .collect();
            (arrows, RUNE_SEQUENCE_FONT_SIZE * 1.5)
        } else {
            (format!("{}", *sequence), RUNE_SEQUENCE_FONT_SIZE)
        };

        let want_size = FontSize::Px(want_size);
        **text = new_text.clone();
        if font.font_size != want_size {
            font.font_size = want_size;
        }
        if let Ok((mut shadow, mut shadow_font)) = shadow_query.single_mut() {
            **shadow = new_text;
            if shadow_font.font_size != want_size {
                shadow_font.font_size = want_size;
            }
        }
    }
}

/// Shows spell name briefly above the rune display when a valid rune sequence is activated.
pub(crate) fn show_spell_name_on_activation(
    mut last_activated: ResMut<LastActivatedSpell>,
    mut commands: Commands,
    mut activated_text_query: Query<(Entity, &mut Text, &mut TextColor), With<ActivatedSpellText>>,
    mut shadow_query: Query<
        (&mut Text, &mut TextColor),
        (With<ActivatedSpellTextShadow>, Without<ActivatedSpellText>),
    >,
) {
    if last_activated.just_activated {
        if let Some(spell) = last_activated.spell
            && let Ok((entity, mut text, mut text_color)) = activated_text_query.single_mut()
        {
            let name = spell.name().to_string();
            **text = name.clone();
            text_color.0 = Color::srgba(0.7, 0.5, 1.0, 1.0);

            // Sync shadow
            if let Ok((mut shadow_text, mut shadow_color)) = shadow_query.single_mut() {
                **shadow_text = name;
                shadow_color.0 = crate::ui::constants::TEXT_SHADOW_COLOR;
            }

            commands.entity(entity).insert(SpellNameFadeTimer {
                elapsed: 0.0,
                duration: SPELL_NAME_FADE_DURATION,
            });
        }
        last_activated.acknowledge();
    }
}
