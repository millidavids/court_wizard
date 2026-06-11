use bevy::prelude::*;

use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::ui::components::ButtonStyle;
use crate::ui::gamepad_glyphs::{CurrentControllerGlyphStyle, GamepadGlyphFonts};
use crate::ui::systems::spawn_button;

use super::super::components::{
    TutorialNextButton, TutorialOverlay, TutorialPanel, TutorialSkipButton, TutorialStepCounter,
    TutorialText,
};
use super::super::constants::*;
use super::super::definitions::PanelAnchor;
use super::super::resources::ActiveTutorial;
use super::super::text_glyphs::spawn_segmented_text;

/// Returns the flexbox alignment values for a given panel anchor.
pub(super) fn anchor_to_alignment(anchor: PanelAnchor) -> (JustifyContent, AlignItems) {
    match anchor {
        PanelAnchor::Center => (JustifyContent::Center, AlignItems::Center),
        PanelAnchor::TopLeft => (JustifyContent::FlexStart, AlignItems::FlexStart),
        PanelAnchor::TopRight => (JustifyContent::FlexStart, AlignItems::FlexEnd),
        PanelAnchor::BottomLeft => (JustifyContent::FlexEnd, AlignItems::FlexStart),
        PanelAnchor::BottomRight => (JustifyContent::FlexEnd, AlignItems::FlexEnd),
        PanelAnchor::TopCenter => (JustifyContent::FlexStart, AlignItems::Center),
        PanelAnchor::BottomCenter => (JustifyContent::FlexEnd, AlignItems::Center),
        PanelAnchor::CenterLeft => (JustifyContent::Center, AlignItems::FlexStart),
        PanelAnchor::CenterRight => (JustifyContent::Center, AlignItems::FlexEnd),
    }
}

/// Spawns the tutorial overlay UI when ActiveTutorial is inserted.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_tutorial_overlay(
    mut commands: Commands,
    active: Res<ActiveTutorial>,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
    active_input: Res<ActiveInputDevice>,
    glyph_style: Res<CurrentControllerGlyphStyle>,
    glyph_fonts: Option<Res<GamepadGlyphFonts>>,
) {
    if !overlay_query.is_empty() {
        return;
    }

    let steps = active.tutorial.steps();
    let step = &steps[active.step];
    let total = steps.len();

    let next_text = if active.step + 1 >= total {
        "Got it"
    } else {
        "Next"
    };

    let (justify, align) = anchor_to_alignment(step.anchor);

    commands
        .spawn((
            TutorialOverlay,
            GlobalZIndex(TUTORIAL_Z_INDEX),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                // Column direction so `anchor_to_alignment`'s
                // (justify, align) pairs match their semantic names: justify
                // controls vertical (Top/Bottom/Center) and align controls
                // horizontal (Left/Right/Center).
                flex_direction: FlexDirection::Column,
                justify_content: justify,
                align_items: align,
                padding: UiRect::all(Val::Px(PANEL_MARGIN)),
                ..default()
            },
            BackgroundColor(OVERLAY_BG),
            crate::ui::focus::ModalOverlay,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    TutorialPanel,
                    Node {
                        max_width: Val::Px(PANEL_MAX_WIDTH),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(PANEL_PADDING)),
                        border: UiRect::all(Val::Px(PANEL_BORDER_WIDTH)),
                        row_gap: Val::Px(16.0),
                        border_radius: BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BG),
                    BorderColor::all(PANEL_BORDER),
                ))
                .with_children(|panel| {
                    let display_text = if active_input.is_gamepad() {
                        step.text
                    } else {
                        step.text_kbm.unwrap_or(step.text)
                    };
                    let text_id = spawn_segmented_text(
                        panel,
                        display_text,
                        TEXT_FONT_SIZE,
                        TEXT_COLOR,
                        PANEL_MAX_WIDTH - PANEL_PADDING * 2.0,
                        active_input.is_gamepad(),
                        glyph_style.0,
                        glyph_fonts.as_deref(),
                    );
                    panel.commands().entity(text_id).insert(TutorialText);

                    panel.spawn((
                        TutorialStepCounter,
                        Text::new(format!("{} of {}", active.step + 1, total)),
                        TextFont::from_font_size(STEP_FONT_SIZE),
                        TextColor(MUTED_TEXT_COLOR),
                    ));

                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            ..default()
                        })
                        .with_children(|buttons| {
                            spawn_button(
                                buttons,
                                next_text,
                                TutorialNextButton,
                                &ButtonStyle {
                                    width: BUTTON_WIDTH,
                                    height: BUTTON_HEIGHT,
                                    border_width: BUTTON_BORDER_WIDTH,
                                    font_size: BUTTON_FONT_SIZE,
                                    background: NEXT_BUTTON_BG,
                                    border: NEXT_BUTTON_BORDER,
                                    text_color: TEXT_COLOR,
                                    text_shadow: true,
                                },
                            );

                            spawn_button(
                                buttons,
                                "Skip Tutorial",
                                TutorialSkipButton,
                                &ButtonStyle {
                                    width: BUTTON_WIDTH,
                                    height: BUTTON_HEIGHT,
                                    border_width: BUTTON_BORDER_WIDTH,
                                    font_size: BUTTON_FONT_SIZE,
                                    background: SKIP_BUTTON_BG,
                                    border: SKIP_BUTTON_BORDER,
                                    text_color: TEXT_COLOR,
                                    text_shadow: true,
                                },
                            );
                        });
                });
        });
}

/// Updates the overlay's flexbox alignment when the step changes.
pub(crate) fn position_tutorial_panel(
    active: Res<ActiveTutorial>,
    mut overlay_query: Query<&mut Node, With<TutorialOverlay>>,
) {
    if !active.is_changed() {
        return;
    }

    let Ok(mut overlay_node) = overlay_query.single_mut() else {
        return;
    };

    let steps = active.tutorial.steps();
    let (justify, align) = anchor_to_alignment(steps[active.step].anchor);
    overlay_node.justify_content = justify;
    overlay_node.align_items = align;
}

pub(crate) fn despawn_overlay(
    commands: &mut Commands,
    overlay_query: &Query<Entity, With<TutorialOverlay>>,
) {
    for entity in overlay_query.iter() {
        commands.entity(entity).try_despawn();
    }
}
