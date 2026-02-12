//! Systems for progress screen.

use bevy::ecs::relationship::Relationship;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::ComputedNode;

use crate::config::WizardType;
use crate::config::save_data::{AchievementId, clear_progress, load_unified_save};
use crate::game::cauldron::brews::Ingredient;
use crate::game::units::wizard::components::Spell;
use crate::ui::systems::spawn_button;

use super::components::{
    BackButton, ClearProgressButton, OnProgressScreen, ScrollableProgressContainer,
};
use super::constants::{
    BUTTON_STYLE, COMPLETED_COLOR, DANGER_BUTTON_STYLE, DESCRIPTION_COLOR, ITEM_DESC_FONT_SIZE,
    ITEM_NAME_FONT_SIZE, LOCKED_COLOR, MARGIN, MARGIN_SMALL, SECTION_BG, SECTION_COLOR,
    SECTION_FONT_SIZE, SECTION_PADDING, TEXT_COLOR, TITLE_FONT_SIZE, UNLOCKED_COLOR,
};

/// Spawns the progress screen UI.
fn setup(mut commands: Commands, transparent_bg: bool) {
    let background_color = if transparent_bg {
        Color::srgba(0.0, 0.0, 0.0, 0.9)
    } else {
        Color::BLACK
    };

    // Load save data
    let save = load_unified_save();
    let unlocked_achievements: Vec<String> = save
        .as_ref()
        .map(|s| s.player.unlocked_achievements.clone())
        .unwrap_or_default();
    let unlocked_content = save
        .as_ref()
        .map(|s| s.player.unlocked_content.clone())
        .unwrap_or_default();

    let mut entity_commands = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(background_color),
        OnProgressScreen,
    ));

    if transparent_bg {
        entity_commands.insert(GlobalZIndex(500));
    }

    entity_commands.with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Progress"),
            TextFont {
                font_size: TITLE_FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(MARGIN)),
                ..default()
            },
        ));

        // Scrollable content
        parent
            .spawn((
                Node {
                    width: Val::Percent(90.0),
                    max_width: Val::Px(700.0),
                    height: Val::Percent(75.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                ScrollableProgressContainer,
            ))
            .with_children(|scroll| {
                scroll
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(MARGIN),
                        padding: UiRect::all(Val::Px(MARGIN_SMALL)),
                        ..default()
                    })
                    .with_children(|content| {
                        // Achievements section
                        spawn_section(content, "Achievements", |section| {
                            for achievement in AchievementId::all() {
                                let is_unlocked =
                                    unlocked_achievements.contains(&achievement.id().to_string());
                                spawn_achievement_row(section, achievement, is_unlocked);
                            }
                        });

                        // Spells section
                        spawn_section(content, "Spells", |section| {
                            for spell in Spell::all() {
                                let name = spell.name().replace('\n', " ");
                                let debug_name = format!("{:?}", spell);
                                let is_unlocked = unlocked_content.spells.contains(&debug_name);
                                spawn_unlockable_row(section, &name, None, is_unlocked);
                            }
                        });

                        // Ingredients section
                        spawn_section(content, "Ingredients", |section| {
                            for ingredient in Ingredient::all() {
                                let debug_name = format!("{:?}", ingredient);
                                let is_unlocked =
                                    unlocked_content.ingredients.contains(&debug_name);
                                spawn_unlockable_row(
                                    section,
                                    ingredient.name(),
                                    Some(ingredient.description()),
                                    is_unlocked,
                                );
                            }
                        });

                        // Wizard Types section
                        spawn_section(content, "Wizard Types", |section| {
                            for wizard_type in WizardType::all() {
                                let debug_name = format!("{:?}", wizard_type);
                                let is_unlocked =
                                    unlocked_content.wizard_types.contains(&debug_name);
                                spawn_unlockable_row(
                                    section,
                                    wizard_type.display_name(),
                                    Some(wizard_type.description()),
                                    is_unlocked,
                                );
                            }
                        });
                    });
            });

        // Buttons row
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(MARGIN),
                margin: UiRect::top(Val::Px(MARGIN_SMALL)),
                ..default()
            })
            .with_children(|row| {
                spawn_button(row, "Back", BackButton, &BUTTON_STYLE);
                spawn_button(
                    row,
                    "Clear Progress",
                    ClearProgressButton,
                    &DANGER_BUTTON_STYLE,
                );
            });
    });
}

/// Spawns a section with a title and content.
fn spawn_section(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    spawn_content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(SECTION_PADDING)),
                row_gap: Val::Px(MARGIN_SMALL),
                ..default()
            },
            BackgroundColor(SECTION_BG),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|section| {
            // Section title
            section.spawn((
                Text::new(title),
                TextFont {
                    font_size: SECTION_FONT_SIZE,
                    ..default()
                },
                TextColor(SECTION_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                    ..default()
                },
            ));

            spawn_content(section);
        });
}

/// Spawns an achievement row with status indicator.
fn spawn_achievement_row(
    parent: &mut ChildSpawnerCommands,
    achievement: &AchievementId,
    is_unlocked: bool,
) {
    let (indicator, indicator_color) = if is_unlocked {
        ("*", COMPLETED_COLOR)
    } else {
        ("-", LOCKED_COLOR)
    };
    let name_color = if is_unlocked {
        UNLOCKED_COLOR
    } else {
        LOCKED_COLOR
    };
    let desc_color = if is_unlocked {
        DESCRIPTION_COLOR
    } else {
        LOCKED_COLOR
    };

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(MARGIN_SMALL),
            ..default()
        })
        .with_children(|row| {
            // Status indicator
            row.spawn((
                Text::new(indicator),
                TextFont {
                    font_size: ITEM_NAME_FONT_SIZE,
                    ..default()
                },
                TextColor(indicator_color),
                Node {
                    width: Val::Px(16.0),
                    ..default()
                },
            ));

            // Name and description column
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new(achievement.display_name()),
                    TextFont {
                        font_size: ITEM_NAME_FONT_SIZE,
                        ..default()
                    },
                    TextColor(name_color),
                ));

                col.spawn((
                    Text::new(achievement.description()),
                    TextFont {
                        font_size: ITEM_DESC_FONT_SIZE,
                        ..default()
                    },
                    TextColor(desc_color),
                ));
            });
        });
}

/// Spawns an unlockable item row.
fn spawn_unlockable_row(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    description: Option<&str>,
    is_unlocked: bool,
) {
    let (indicator, indicator_color) = if is_unlocked {
        ("*", COMPLETED_COLOR)
    } else {
        ("-", LOCKED_COLOR)
    };
    let name_color = if is_unlocked {
        UNLOCKED_COLOR
    } else {
        LOCKED_COLOR
    };

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(MARGIN_SMALL),
            ..default()
        })
        .with_children(|row| {
            // Status indicator
            row.spawn((
                Text::new(indicator),
                TextFont {
                    font_size: ITEM_NAME_FONT_SIZE,
                    ..default()
                },
                TextColor(indicator_color),
                Node {
                    width: Val::Px(16.0),
                    ..default()
                },
            ));

            if let Some(desc) = description {
                // Name + description column
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new(name),
                        TextFont {
                            font_size: ITEM_NAME_FONT_SIZE,
                            ..default()
                        },
                        TextColor(name_color),
                    ));

                    let desc_color = if is_unlocked {
                        DESCRIPTION_COLOR
                    } else {
                        LOCKED_COLOR
                    };
                    col.spawn((
                        Text::new(desc),
                        TextFont {
                            font_size: ITEM_DESC_FONT_SIZE,
                            ..default()
                        },
                        TextColor(desc_color),
                    ));
                });
            } else {
                // Name only
                row.spawn((
                    Text::new(name),
                    TextFont {
                        font_size: ITEM_NAME_FONT_SIZE,
                        ..default()
                    },
                    TextColor(name_color),
                ));
            }
        });
}

/// Clears all achievements and unlockable progress from the save file.
pub(super) fn handle_clear_progress() {
    clear_progress();
}

/// Spawns progress with solid black background (for main menu).
pub(super) fn setup_main_menu(commands: Commands) {
    setup(commands, false);
}

/// Spawns progress with transparent background (for pause menu).
pub(super) fn setup_pause_menu(commands: Commands) {
    setup(commands, true);
}

/// Updates button colors on hover using each button's stored ButtonColors.
pub(super) fn update_button_colors(
    mut button_query: Query<
        (
            &Interaction,
            &crate::ui::components::ButtonColors,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    use crate::ui::styles::{item_hovered, item_pressed};

    for (interaction, colors, mut bg_color, mut border_color) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = item_pressed(colors.background).into();
                *border_color = BorderColor::all(item_pressed(colors.border));
            }
            Interaction::Hovered => {
                *bg_color = item_hovered(colors.background).into();
                *border_color = BorderColor::all(item_hovered(colors.border));
            }
            Interaction::None => {
                *bg_color = colors.background.into();
                *border_color = BorderColor::all(colors.border);
            }
        }
    }
}

/// Despawns all progress screen entities.
pub(super) fn cleanup(mut commands: Commands, query: Query<Entity, With<OnProgressScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handles mouse wheel scrolling for the progress container.
pub(super) fn handle_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    mut scrollable_query: Query<
        (&mut ScrollPosition, &ComputedNode),
        With<ScrollableProgressContainer>,
    >,
    parent_query: Query<&ChildOf>,
) {
    const LINE_HEIGHT: f32 = 10.0;
    const PIXEL_SCROLL_MULTIPLIER: f32 = 0.3;

    for event in mouse_wheel_events.read() {
        let dy = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => -event.y * LINE_HEIGHT,
            bevy::input::mouse::MouseScrollUnit::Pixel => -event.y * PIXEL_SCROLL_MULTIPLIER,
        };

        for pointer_map in hover_map.values() {
            for (hovered_entity, _) in pointer_map.iter() {
                let mut current_entity = *hovered_entity;
                loop {
                    if let Ok((mut scroll_position, computed)) =
                        scrollable_query.get_mut(current_entity)
                    {
                        let visible_size = computed.size();
                        let content_size = computed.content_size();
                        let max_scroll = (content_size.y - visible_size.y).max(0.0)
                            * computed.inverse_scale_factor();

                        scroll_position.y = (scroll_position.y + dy).clamp(0.0, max_scroll);
                        break;
                    }

                    if let Ok(parent) = parent_query.get(current_entity) {
                        current_entity = parent.get();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
