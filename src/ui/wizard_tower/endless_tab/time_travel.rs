use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::constants::boss_name_for_level;
use crate::game::input::messages::MouseClicked;
use crate::ui::systems::spawn_button;

use super::super::components::*;
use super::super::constants::*;
use super::actions::EndlessAction;

// ---------------------------------------------------------------------------
// Time Travel section
// ---------------------------------------------------------------------------

/// Initializes the time travel resource. Call before spawning the time travel UI.
pub(super) fn init_time_travel_resource(commands: &mut Commands) {
    commands.insert_resource(SelectedTimeTravelLevel::default());
}

pub(super) fn spawn_time_travel_section(parent: &mut ChildSpawnerCommands, config: &GameConfig) {
    // Title
    parent.spawn((
        Text::new("Time Travel"),
        TextFont::from_font_size(20.0),
        TextColor(TIME_TRAVEL_BOSS_COLOR),
        Node {
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        },
    ));

    // Level list container with border
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                width: Val::Px(240.0),
                ..default()
            },
            BackgroundColor(TIME_TRAVEL_SECTION_BG),
            BorderColor::all(TIME_TRAVEL_SECTION_BORDER),
            TimeTravelContainer,
        ))
        .with_children(|section| {
            // Scrollable level list
            section
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        max_height: Val::Px(TIME_TRAVEL_LIST_MAX_HEIGHT),
                        width: Val::Percent(100.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollPosition::default(),
                    crate::ui::focus::GamepadScrollTarget,
                    TimeTravelSection,
                ))
                .with_children(|list| {
                    for level in 1..config.highest_level_achieved {
                        let label = if let Some(boss) = boss_name_for_level(level) {
                            format!("Level {} ({})", level, boss)
                        } else {
                            format!("Level {}", level)
                        };
                        let text_color = if boss_name_for_level(level).is_some() {
                            TIME_TRAVEL_BOSS_COLOR
                        } else {
                            TEXT_COLOR
                        };

                        list.spawn((
                            Button,
                            Node {
                                height: Val::Px(TIME_TRAVEL_LEVEL_HEIGHT),
                                padding: UiRect::horizontal(Val::Px(8.0)),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            TimeTravelLevelButton(level),
                        ))
                        .with_child((
                            Text::new(label),
                            TextFont::from_font_size(TIME_TRAVEL_LEVEL_FONT_SIZE),
                            TextColor(text_color),
                        ));
                    }
                });

            // Selected level display
            section.spawn((
                Text::new("Select a level..."),
                TextFont::from_font_size(TIME_TRAVEL_LEVEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
                TimeTravelSelectedDisplay,
            ));

            // Start button
            spawn_button(
                section,
                "Start Time Travel",
                EndlessAction::StartTimeTravel,
                &START_TIME_TRAVEL_BUTTON_STYLE,
            );
        });
}

// ---------------------------------------------------------------------------
// Time travel interaction systems
// ---------------------------------------------------------------------------

/// Handles clicks on time travel level buttons.
pub(crate) fn handle_time_travel_level_clicks(
    mut button_clicked: MessageReader<MouseClicked>,
    level_buttons: Query<&TimeTravelLevelButton>,
    mut selected: ResMut<SelectedTimeTravelLevel>,
    mut level_button_nodes: Query<(&TimeTravelLevelButton, &mut BackgroundColor, &Children)>,
    grandchildren_query: Query<&Children>,
    mut text_queries: ParamSet<(
        Query<(&mut Text, &mut TextColor), With<TimeTravelSelectedDisplay>>,
        Query<&mut TextColor>,
    )>,
) {
    for event in button_clicked.read() {
        if let Ok(btn) = level_buttons.get(event.button) {
            let selected_level = btn.0;
            selected.0 = Some(selected_level);

            // Update display text
            for (mut text, mut color) in &mut text_queries.p0() {
                text.0 = format!("Selected: Level {}", selected_level);
                color.0 = TIME_TRAVEL_SELECTED_TEXT;
            }

            // Collect descendant updates (children + grandchildren for shadow wrappers)
            let mut updates: Vec<(Entity, Color)> = Vec::new();
            for (lb, _, children) in &level_button_nodes {
                let is_selected = lb.0 == selected_level;
                let color = if is_selected {
                    TIME_TRAVEL_SELECTED_TEXT
                } else if boss_name_for_level(lb.0).is_some() {
                    TIME_TRAVEL_BOSS_COLOR
                } else {
                    TEXT_COLOR
                };
                for child in children.iter() {
                    updates.push((child, color));
                    if let Ok(gcs) = grandchildren_query.get(child) {
                        for gc in gcs.iter() {
                            updates.push((gc, color));
                        }
                    }
                }
            }

            // Apply background highlights
            for (lb, mut bg, _) in &mut level_button_nodes {
                bg.0 = if lb.0 == selected_level {
                    TIME_TRAVEL_SELECTED_BG
                } else {
                    Color::NONE
                };
            }

            // Apply text color updates
            let mut text_colors = text_queries.p1();
            for (entity, color) in updates {
                if let Ok(mut tc) = text_colors.get_mut(entity) {
                    tc.0 = color;
                }
            }
        }
    }
}

/// Handles hover effects on time travel level buttons.
pub(crate) fn handle_time_travel_level_hover(
    mut level_buttons: Query<
        (&Interaction, &TimeTravelLevelButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    selected: Option<Res<SelectedTimeTravelLevel>>,
) {
    let selected_level = selected.and_then(|s| s.0);
    for (interaction, btn, mut bg) in &mut level_buttons {
        let is_selected = selected_level == Some(btn.0);
        bg.0 = match *interaction {
            Interaction::Hovered | Interaction::Pressed if !is_selected => TIME_TRAVEL_HOVER_BG,
            _ if is_selected => TIME_TRAVEL_SELECTED_BG,
            _ => Color::NONE,
        };
    }
}
