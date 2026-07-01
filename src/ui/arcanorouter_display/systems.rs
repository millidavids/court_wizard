use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::input_bindings::{BindingContext, InputBindings, key_display_name};
use crate::game::components::OnGameplayScreen;
use crate::game::game_mode::components::ArchetypeUI;
use crate::game::units::wizard::archetypes::arcanorouter::{
    ArcanoRouterState, SliderAdjustMessage, SliderType,
};
use crate::ui::gamepad_glyphs::{ButtonPrompt, ButtonPromptImage, PromptKey};

/// The keyboard key bound to raising a given slider.
fn slider_key(bindings: &InputBindings, slider: SliderType) -> Option<KeyCode> {
    bindings.get(BindingContext::ArcanoRouter, slider.binding_action())
}

/// Returns the color for a slider type
fn slider_color(slider_type: SliderType) -> Color {
    match slider_type {
        SliderType::Range => RANGE_COLOR,
        SliderType::Mana => MANA_COLOR,
        SliderType::Power => POWER_COLOR,
        SliderType::Speed => SPEED_COLOR,
    }
}

/// Returns the label text for a slider type
fn slider_label(slider_type: SliderType) -> &'static str {
    match slider_type {
        SliderType::Range => "RANGE",
        SliderType::Mana => "MANA",
        SliderType::Power => "POWER",
        SliderType::Speed => "SPEED",
    }
}

/// Spawns the Arcanorouter display with 4 vertical sliders
pub(crate) fn spawn_arcanorouter_display(
    mut commands: Commands,
    state: Res<ArcanoRouterState>,
    bindings: Res<InputBindings>,
) {
    // Root container - absolute positioned at bottom center
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BOTTOM_OFFSET),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            OnGameplayScreen,
            ArcanoRouterDisplay,
            ArchetypeUI,
        ))
        .with_children(|parent| {
            // Container for all sliders
            parent
                .spawn((
                    Node {
                        column_gap: Val::Px(SLIDER_GAP),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(BACKGROUND_COLOR),
                ))
                .with_children(|sliders_parent| {
                    // Spawn each slider
                    for slider_type in SliderType::all() {
                        let allocation = state.get_allocation(slider_type);
                        let color = slider_color(slider_type);

                        sliders_parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(2.0),
                                ..default()
                            })
                            .with_children(|slider_parent| {
                                // Key/glyph hint (top): bound key or controller
                                // glyph — kept in sync by `adapt_button_prompts`.
                                let action = slider_type.binding_action();
                                let prompt = slider_parent
                                    .spawn((
                                        Text::new(key_display_name(
                                            bindings.get(BindingContext::ArcanoRouter, action),
                                        )),
                                        TextFont::from_font_size(LABEL_FONT_SIZE),
                                        TextColor(TEXT_COLOR),
                                        ButtonPrompt {
                                            action: slider_type.dpad_action(),
                                            key: PromptKey::Binding(
                                                BindingContext::ArcanoRouter,
                                                action,
                                            ),
                                            glyph_px: SLIDER_GLYPH_SIZE,
                                            keyboard_px: LABEL_FONT_SIZE,
                                        },
                                    ))
                                    .id();
                                slider_parent.spawn((
                                    ImageNode::new(Handle::default()),
                                    Node {
                                        width: Val::Px(SLIDER_GLYPH_SIZE),
                                        height: Val::Px(SLIDER_GLYPH_SIZE),
                                        display: Display::None,
                                        ..default()
                                    },
                                    ButtonPromptImage { text: prompt },
                                ));

                                // Bar container (background + fill)
                                slider_parent
                                    .spawn((
                                        Node {
                                            width: Val::Px(SLIDER_WIDTH),
                                            height: Val::Px(SLIDER_HEIGHT),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::FlexEnd, // Fill from bottom
                                            border: UiRect::all(Val::Px(2.0)),
                                            border_radius: BorderRadius::all(Val::Px(4.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BAR_BACKGROUND_COLOR),
                                        BorderColor::all(color),
                                        SliderBar,
                                    ))
                                    .with_children(|bar_parent| {
                                        // Fill bar (grows upward from bottom)
                                        let fill_height = (allocation / 200.0) * SLIDER_HEIGHT;
                                        bar_parent.spawn((
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(fill_height),
                                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(color),
                                            SliderFill { slider_type },
                                        ));
                                    });

                                // Label text at bottom
                                slider_parent.spawn((
                                    Text::new(slider_label(slider_type)),
                                    TextFont::from_font_size(LABEL_FONT_SIZE),
                                    TextColor(TEXT_COLOR),
                                ));
                            });
                    }
                });
        });
}

/// Updates slider visual fills based on current allocations
pub(super) fn update_slider_visuals(
    state: Res<ArcanoRouterState>,
    mut fill_query: Query<(&SliderFill, &mut Node)>,
) {
    // Only update if state changed
    if !state.is_changed() {
        return;
    }

    // Update fill bar heights
    for (fill, mut node) in fill_query.iter_mut() {
        let allocation = state.get_allocation(fill.slider_type);
        let fill_height = (allocation / 200.0) * SLIDER_HEIGHT;
        node.height = Val::Px(fill_height);
    }
}

/// Handles keyboard input for slider adjustment.
///
/// Only increment keys exist — sliders are reactive, so pushing one up pulls
/// the others down proportionally.
pub(super) fn handle_slider_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut writer: MessageWriter<SliderAdjustMessage>,
) {
    for slider in SliderType::all() {
        if let Some(key) = slider_key(&bindings, slider)
            && keyboard.just_pressed(key)
        {
            writer.write(SliderAdjustMessage {
                slider,
                delta: crate::game::units::wizard::archetypes::arcanorouter::constants::SLIDER_KEY_STEP,
            });
        }
    }
}
