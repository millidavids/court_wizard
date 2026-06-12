use bevy::prelude::*;
use rand::Rng;

use crate::config::GameConfig;
use crate::game::input::messages::MouseClicked;
use crate::ui::components::ButtonColors;

use super::components::{SeedInputBox, SeedInputState, SeedInputText, SeedRandomButton};
use super::constants::{LABEL_COLOR, MAX_SEED_CHARS, SEED_BORDER_FOCUSED, SEED_BORDER_UNFOCUSED};

fn random_seed() -> u64 {
    rand::rng().random_range(0..super::constants::MAX_SEED)
}

/// Assigns a fresh random seed to both the UI state and the game config, then
/// updates the visible seed text.  Shared by the click handler and the keyboard
/// handler so the "text is empty on unfocus" path has one canonical implementation.
fn apply_fresh_random_seed(
    seed_state: &mut SeedInputState,
    config: &mut GameConfig,
    text_query: &mut Query<&mut Text, With<SeedInputText>>,
) {
    let new_seed = random_seed();
    seed_state.text = new_seed.to_string();
    config.seed = Some(new_seed);
    for mut text in text_query {
        text.0 = new_seed.to_string();
    }
}

/// Spawns the seed input row with label, text input box, and Randomize button.
pub(super) fn spawn_seed_input_row(parent: &mut ChildSpawnerCommands, seed_text: &str) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(crate::ui::constants::SLIDER_GAP),
            margin: UiRect::bottom(Val::Px(super::constants::SECTION_MARGIN)),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new("Seed"),
                TextFont::from_font_size(crate::ui::constants::SLIDER_LABEL_FONT_SIZE),
                TextColor(LABEL_COLOR),
                Node {
                    min_width: Val::Px(200.0),
                    width: Val::Px(200.0),
                    ..default()
                },
            ));

            // Input text field
            let seed_bg = Color::hsla(270.0, 0.08, 0.08, 1.0);
            row.spawn((
                Button,
                Node {
                    width: Val::Px(280.0),
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(SEED_BORDER_UNFOCUSED),
                BackgroundColor(seed_bg),
                SeedInputBox,
                crate::ui::focus::Focusable,
                crate::ui::focus::FocusableFlatBackground { base: seed_bg },
            ))
            .with_children(|input_box| {
                input_box.spawn((
                    Text::new(seed_text),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    SeedInputText,
                ));
            });

            // Random button
            row.spawn((
                Button,
                Node {
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(SEED_BORDER_UNFOCUSED),
                BackgroundColor(Color::hsla(270.0, 0.08, 0.10, 1.0)),
                ButtonColors {
                    background: Color::hsla(270.0, 0.08, 0.10, 1.0),
                    border: SEED_BORDER_UNFOCUSED,
                },
                SeedRandomButton,
                crate::ui::focus::Focusable,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Randomize"),
                    TextFont::from_font_size(12.0),
                    TextColor(LABEL_COLOR),
                ));
            });
        });
}

/// Handles clicking the seed input box to focus it, or clicking Random / other buttons to unfocus.
pub(crate) fn seed_input_click(
    mut button_clicked: MessageReader<MouseClicked>,
    input_boxes: Query<Entity, With<SeedInputBox>>,
    random_buttons: Query<Entity, With<SeedRandomButton>>,
    mut seed_state: ResMut<SeedInputState>,
    mut config: ResMut<GameConfig>,
    mut text_query: Query<&mut Text, With<SeedInputText>>,
    mut border_query: Query<&mut BorderColor, With<SeedInputBox>>,
) {
    let focus_before = seed_state.focused;

    for event in button_clicked.read() {
        if input_boxes.get(event.button).is_ok() {
            seed_state.focused = !seed_state.focused;
            if seed_state.focused {
                seed_state.text.clear();
                for mut text in &mut text_query {
                    text.0 = String::new();
                }
            }
        } else if random_buttons.get(event.button).is_ok() {
            seed_state.focused = false;
            apply_fresh_random_seed(&mut seed_state, &mut config, &mut text_query);
        } else if seed_state.focused {
            seed_state.focused = false;
            if seed_state.text.is_empty() {
                apply_fresh_random_seed(&mut seed_state, &mut config, &mut text_query);
            }
        }
    }

    // Update border color only when focus state changed
    if seed_state.focused != focus_before {
        let new_border = if seed_state.focused {
            SEED_BORDER_FOCUSED
        } else {
            SEED_BORDER_UNFOCUSED
        };
        for mut border in &mut border_query {
            *border = BorderColor::all(new_border);
        }
    }
}

/// Handles keyboard input when the seed field is focused.
pub(crate) fn seed_input_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut seed_state: ResMut<SeedInputState>,
    mut config: ResMut<GameConfig>,
    mut text_query: Query<&mut Text, With<SeedInputText>>,
    mut border_query: Query<&mut BorderColor, With<SeedInputBox>>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    // Ctrl+C: Copy seed to clipboard (works even when not focused)
    if ctrl
        && keyboard.just_pressed(KeyCode::KeyC)
        && !seed_state.text.is_empty()
        && let Ok(mut clipboard) = arboard::Clipboard::new()
    {
        let _ = clipboard.set_text(seed_state.text.clone());
    }

    if !seed_state.focused {
        return;
    }

    let mut changed = false;

    // Ctrl+V: Paste seed from clipboard
    if ctrl
        && keyboard.just_pressed(KeyCode::KeyV)
        && let Ok(text) = arboard::Clipboard::new().and_then(|mut cb| cb.get_text())
    {
        let digits: String = text
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(MAX_SEED_CHARS)
            .collect();
        if !digits.is_empty() {
            seed_state.text = digits;
            changed = true;
        }
    }

    // Number keys (main keyboard)
    for (key, digit) in [
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ] {
        if keyboard.just_pressed(key) && seed_state.text.len() < MAX_SEED_CHARS {
            seed_state.text.push(digit);
            changed = true;
        }
    }

    // Numpad keys
    for (key, digit) in [
        (KeyCode::Numpad0, '0'),
        (KeyCode::Numpad1, '1'),
        (KeyCode::Numpad2, '2'),
        (KeyCode::Numpad3, '3'),
        (KeyCode::Numpad4, '4'),
        (KeyCode::Numpad5, '5'),
        (KeyCode::Numpad6, '6'),
        (KeyCode::Numpad7, '7'),
        (KeyCode::Numpad8, '8'),
        (KeyCode::Numpad9, '9'),
    ] {
        if keyboard.just_pressed(key) && seed_state.text.len() < MAX_SEED_CHARS {
            seed_state.text.push(digit);
            changed = true;
        }
    }

    // Backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        seed_state.text.pop();
        changed = true;
    }

    // Enter / Numpad Enter to unfocus
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        seed_state.focused = false;
        if seed_state.text.is_empty() {
            apply_fresh_random_seed(&mut seed_state, &mut config, &mut text_query);
        }
        for mut border in &mut border_query {
            *border = BorderColor::all(SEED_BORDER_UNFOCUSED);
        }
        return;
    }

    if changed {
        config.seed = seed_state.text.parse::<u64>().ok();

        for mut text in &mut text_query {
            text.0 = seed_state.text.clone();
        }
    }
}
