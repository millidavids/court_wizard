use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::input_bindings::{
    BindingAction, BindingContext, InputBindings, key_display_name,
};
use crate::game::components::OnGameplayScreen;
use crate::game::game_mode::components::ArchetypeUI;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::input::messages::MouseClicked;
use crate::game::units::wizard::archetypes::meteorologist::constants::{
    BLIZZARD_COLOR, DROUGHT_COLOR, STORM_COLOR,
};
use crate::game::units::wizard::archetypes::meteorologist::messages::WeatherChangedMessage;
use crate::game::units::wizard::archetypes::meteorologist::resources::{WeatherState, WeatherType};
use crate::game::units::wizard::archetypes::meteorologist::systems::try_switch_weather;
use crate::game::units::wizard::components::{LocalWizard, Mana};
use crate::ui::gamepad_glyphs::{ButtonPrompt, ButtonPromptImage, PromptKey};
use crate::ui::systems::scale_font_by_text_width;

/// Returns the theme color for a weather type.
fn weather_color(weather: &WeatherType) -> Color {
    match weather {
        WeatherType::Storm => STORM_COLOR,
        WeatherType::Blizzard => BLIZZARD_COLOR,
        WeatherType::Drought => DROUGHT_COLOR,
    }
}

/// Spawns the weather bar UI.
pub(crate) fn spawn_weather_bar(mut commands: Commands, bindings: Res<InputBindings>) {
    // Root container: full-width bar at bottom, content centered
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BAR_BOTTOM_MARGIN),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(3.0),
                ..default()
            },
            WeatherBarRoot,
            OnGameplayScreen,
            // Despawned + respawned by the wizard-cycle handler like other
            // archetype HUDs.
            ArchetypeUI,
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            // Inner container with background
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(3.0),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
                    BorderColor::all(Color::srgba(0.3, 0.3, 0.4, 0.5)),
                    Pickable::IGNORE,
                ))
                .with_children(|inner| {
                    // Status text (shows active weather description), confined width with wrapping
                    inner
                        .spawn(Node {
                            max_width: Val::Px(STATUS_TEXT_MAX_WIDTH),
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|wrapper| {
                            wrapper.spawn((
                                Text::new("Clear Skies"),
                                TextFont::from_font_size(STATUS_FONT_SIZE),
                                TextColor(STATUS_TEXT_COLOR),
                                TextLayout::new_with_justify(Justify::Center),
                                WeatherStatusText,
                            ));
                        });

                    // Button row
                    inner
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(BUTTON_GAP),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            for weather in WeatherType::all() {
                                spawn_weather_button(row, *weather, &bindings);
                            }
                        });
                });
        });
}

/// Spawns a single weather button.
fn spawn_weather_button(
    parent: &mut ChildSpawnerCommands,
    weather: WeatherType,
    bindings: &InputBindings,
) {
    let color = weather_color(&weather);

    parent
        .spawn((
            Node {
                width: Val::Px(BUTTON_WIDTH),
                height: Val::Px(BUTTON_HEIGHT),
                border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(3.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(INACTIVE_BG),
            BorderColor::all(INACTIVE_BORDER),
            crate::ui::components::ButtonColors {
                background: INACTIVE_BG,
                border: INACTIVE_BORDER,
            },
            WeatherButton { weather },
            Button,
            Interaction::default(),
        ))
        .with_children(|btn| {
            // Key/glyph hint (top): bound keyboard key, or the controller glyph —
            // kept in sync by the shared `adapt_button_prompts` system.
            let action = weather.binding_action();
            let prompt = btn
                .spawn((
                    Text::new(key_display_name(
                        bindings.get(BindingContext::Meteorologist, action),
                    )),
                    TextFont::from_font_size(HOTKEY_FONT_SIZE),
                    TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                    ButtonPrompt {
                        action: weather.dpad_action(),
                        key: PromptKey::Binding(BindingContext::Meteorologist, action),
                        glyph_px: WEATHER_GLYPH_SIZE,
                        keyboard_px: HOTKEY_FONT_SIZE,
                    },
                    Pickable::IGNORE,
                ))
                .id();
            // Controller glyph image, occupying the same top slot (hidden until a
            // Steam glyph is active).
            btn.spawn((
                ImageNode::new(Handle::default()),
                Node {
                    width: Val::Px(WEATHER_GLYPH_SIZE),
                    height: Val::Px(WEATHER_GLYPH_SIZE),
                    display: Display::None,
                    ..default()
                },
                ButtonPromptImage { text: prompt },
                Pickable::IGNORE,
            ));

            // Weather name (shrink font for long names)
            let name = weather.display_name();
            let font_size =
                scale_font_by_text_width(name.len() as f32, 7.0, 12.0, 0.65, LABEL_FONT_SIZE);
            btn.spawn((
                Text::new(name),
                TextFont::from_font_size(font_size),
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));

            // Intensity fill bar (bottom, grows upward)
            btn.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(BUTTON_BORDER_WIDTH),
                    right: Val::Px(BUTTON_BORDER_WIDTH),
                    height: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(color),
                WeatherIntensityFill { weather },
                Pickable::IGNORE,
            ));
        });
}

/// Handles weather button clicks via the MouseClicked message system.
pub(super) fn handle_weather_button_click(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WeatherButton>,
    mut weather: ResMut<WeatherState>,
    // Local wizard only — MP has two `Mana`-bearing wizards; an unfiltered
    // single query would error and the weather button would silently no-op.
    mut mana_query: Query<&mut Mana, With<LocalWizard>>,
    mut writer: MessageWriter<WeatherChangedMessage>,
) {
    for event in button_clicked.read() {
        let Ok(btn) = button_query.get(event.button) else {
            continue;
        };

        let Ok(mut mana) = mana_query.single_mut() else {
            continue;
        };
        try_switch_weather(&mut weather, &mut mana, btn.weather, &mut writer);
    }
}

/// Updates weather button visuals based on active weather and hover state.
pub(super) fn update_weather_buttons(
    weather: Res<WeatherState>,
    input_device: Res<ActiveInputDevice>,
    bindings: Res<InputBindings>,
    mut buttons: Query<(
        &WeatherButton,
        &mut BorderColor,
        &mut BackgroundColor,
        &Interaction,
    )>,
    mut fills: Query<(&WeatherIntensityFill, &mut Node)>,
    mut status_text: Query<&mut Text, With<WeatherStatusText>>,
) {
    // The bar shows the LOCAL player's own weather; the opponent's shows via VFX.
    let active = weather.local.active;
    let intensity = weather.local.intensity;

    // Update button borders and backgrounds
    for (btn, mut border, mut bg, interaction) in buttons.iter_mut() {
        if active == Some(btn.weather) {
            *border = BorderColor::all(ACTIVE_BORDER);
            let color = weather_color(&btn.weather);
            *bg = BackgroundColor(Color::srgba(
                color.to_srgba().red * 0.3,
                color.to_srgba().green * 0.3,
                color.to_srgba().blue * 0.3,
                0.6,
            ));
        } else if *interaction == Interaction::Hovered {
            *border = BorderColor::all(HOVERED_BORDER);
            *bg = BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.7));
        } else {
            *border = BorderColor::all(INACTIVE_BORDER);
            *bg = BackgroundColor(INACTIVE_BG);
        }
    }

    // Update intensity fill bars
    for (fill, mut node) in fills.iter_mut() {
        if active == Some(fill.weather) {
            // Intensity goes from 1.0 to 1.5, normalize to 0..1
            let normalized = ((intensity - 1.0) / 0.5).clamp(0.0, 1.0);
            node.height = Val::Px(INTENSITY_FILL_MAX_HEIGHT * normalized);
        } else {
            node.height = Val::Px(0.0);
        }
    }

    // Update status text
    if let Ok(mut text) = status_text.single_mut() {
        match active {
            Some(w) => {
                let intensity_pct = ((intensity - 1.0) / 0.5 * 100.0) as u32;
                **text = format!(
                    "{} — {} ({}%)",
                    w.display_name(),
                    w.description(),
                    intensity_pct
                );
            }
            None => {
                if weather.cooldown > 0.0 {
                    **text = format!("Changing... ({:.1}s)", weather.cooldown);
                } else if input_device.is_gamepad() {
                    // On a controller the per-button glyphs show the directions.
                    **text = "Clear Skies — pick a weather".to_string();
                } else {
                    // Show the player's actual bound keys, not hardcoded Q/W/E.
                    **text = format!(
                        "Clear Skies — Press {}/{}/{}",
                        key_display_name(
                            bindings.get(BindingContext::Meteorologist, BindingAction::Weather1)
                        ),
                        key_display_name(
                            bindings.get(BindingContext::Meteorologist, BindingAction::Weather2)
                        ),
                        key_display_name(
                            bindings.get(BindingContext::Meteorologist, BindingAction::Weather3)
                        ),
                    );
                }
            }
        }
    }
}

/// Cleans up the weather bar UI.
pub(super) fn cleanup_weather_bar(
    mut commands: Commands,
    query: Query<Entity, With<WeatherBarRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).try_despawn();
    }
}
