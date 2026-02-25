//! Systems for progress screen.

use bevy::ecs::relationship::Relationship;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::ComputedNode;

use crate::config::WizardType;
use crate::config::save_data::{
    AchievementId, clear_progress, get_insight, get_spell_research_progress, load_unified_save,
};
use crate::game::cauldron::brews::Ingredient;
use crate::game::units::wizard::components::Spell;
use crate::ui::systems::spawn_button;

use super::components::{
    BackButton, CancelClearButton, ClearProgressButton, ConfirmClearButton, ConfirmationPopup,
    OnProgressScreen, ScrollableProgressContainer,
};
use super::constants::{
    BUTTON_STYLE, COLUMN_GAP, COLUMN_TITLE_FONT_SIZE, COMPLETED_COLOR, DANGER_BUTTON_STYLE,
    DESCRIPTION_COLOR, IN_PROGRESS_COLOR, INSIGHT_COLOR, ITEM_DESC_FONT_SIZE, ITEM_NAME_FONT_SIZE,
    LOCKED_COLOR, MARGIN, MARGIN_SMALL, SECTION_BG, SECTION_PADDING, STAT_LABEL_COLOR,
    STAT_VALUE_FONT_SIZE, TEXT_COLOR, TITLE_FONT_SIZE, UNLOCKED_COLOR,
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
    let total_games = save
        .as_ref()
        .map(|s| s.player.total_games_played)
        .unwrap_or(0);
    let total_victories = save
        .as_ref()
        .map(|s| s.player.total_levels_completed)
        .unwrap_or(0);
    let total_attackers = save
        .as_ref()
        .map(|s| s.player.total_attackers_killed)
        .unwrap_or(0);
    let total_defenders = save
        .as_ref()
        .map(|s| s.player.total_defenders_killed)
        .unwrap_or(0);
    let total_undead = save
        .as_ref()
        .map(|s| s.player.total_undead_killed)
        .unwrap_or(0);

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
            TextFont::from_font_size(TITLE_FONT_SIZE),
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(MARGIN)),
                ..default()
            },
        ));

        // Columns container (horizontal row of individually scrollable columns)
        parent
            .spawn(Node {
                width: Val::Percent(95.0),
                height: Val::Percent(75.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(COLUMN_GAP),
                ..default()
            })
            .with_children(|columns| {
                // Statistics column
                let insight_balance = get_insight();
                spawn_column(columns, "Statistics", |section| {
                    spawn_stat_row(section, "Games Played", total_games);
                    spawn_stat_row(section, "Victories", total_victories);
                    spawn_stat_row(section, "Attackers Killed", total_attackers);
                    spawn_stat_row(section, "Defenders Lost", total_defenders);
                    spawn_stat_row(section, "Undead Killed", total_undead);
                    spawn_insight_row(section, "Arcane Insight", insight_balance);
                });

                // Achievements column (unlocked first)
                spawn_column(columns, "Achievements", |section| {
                    let mut achievements: Vec<_> = AchievementId::all()
                        .iter()
                        .map(|a| {
                            let is_unlocked = unlocked_achievements.contains(&a.id().to_string());
                            (a, is_unlocked)
                        })
                        .collect();
                    achievements.sort_by_key(|(_, unlocked)| !unlocked);
                    for (achievement, is_unlocked) in achievements {
                        spawn_achievement_row(section, achievement, is_unlocked);
                    }
                });

                // Spells column (researched first, then in-progress, then locked)
                spawn_column(columns, "Spells", |section| {
                    let mut spells: Vec<_> = Spell::all()
                        .iter()
                        .map(|spell| {
                            let debug_name = format!("{:?}", spell);
                            let is_unlocked = unlocked_content.spells.contains(&debug_name);
                            let progress = get_spell_research_progress(*spell);
                            let cost = spell.research_cost();
                            (spell, is_unlocked, progress, cost)
                        })
                        .collect();
                    // Sort: unlocked first, then in-progress (has progress), then locked
                    spells.sort_by_key(|(_, unlocked, progress, _)| {
                        if *unlocked {
                            0
                        } else if *progress > 0 {
                            1
                        } else {
                            2
                        }
                    });
                    for (spell, is_unlocked, progress, cost) in spells {
                        spawn_spell_research_row(section, spell, is_unlocked, progress, cost);
                    }
                });

                // Ingredients column (unlocked first)
                spawn_column(columns, "Ingredients", |section| {
                    let mut ingredients: Vec<_> = Ingredient::all()
                        .iter()
                        .map(|ingredient| {
                            let debug_name = format!("{:?}", ingredient);
                            let is_unlocked = unlocked_content.ingredients.contains(&debug_name);
                            (ingredient, is_unlocked)
                        })
                        .collect();
                    ingredients.sort_by_key(|(_, unlocked)| !unlocked);
                    for (ingredient, is_unlocked) in ingredients {
                        spawn_unlockable_row(
                            section,
                            ingredient.name(),
                            Some(ingredient.description()),
                            ingredient.locked_description(),
                            is_unlocked,
                            false, // Only show joke text when locked (like spells)
                        );
                    }
                });

                // Wizard Types column (unlocked first)
                spawn_column(columns, "Wizard Types", |section| {
                    let mut wizard_types: Vec<_> = WizardType::all()
                        .iter()
                        .map(|wizard_type| {
                            let debug_name = format!("{:?}", wizard_type);
                            let is_unlocked = *wizard_type == WizardType::BoringOleMage
                                || unlocked_content.wizard_types.contains(&debug_name);
                            (wizard_type, is_unlocked)
                        })
                        .collect();
                    wizard_types.sort_by_key(|(_, unlocked)| !unlocked);
                    for (wizard_type, is_unlocked) in wizard_types {
                        spawn_unlockable_row(
                            section,
                            wizard_type.display_name(),
                            Some(wizard_type.description()),
                            wizard_type.locked_description(),
                            is_unlocked,
                            true, // Show name when locked for wizard types
                        );
                    }
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

/// Spawns a scrollable column with a title header and content.
fn spawn_column(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    spawn_content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(SECTION_BG),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|column| {
            // Column title (fixed, non-scrolling)
            column.spawn((
                Text::new(title),
                TextFont::from_font_size(COLUMN_TITLE_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    padding: UiRect::new(
                        Val::Px(SECTION_PADDING),
                        Val::Px(SECTION_PADDING),
                        Val::Px(SECTION_PADDING),
                        Val::Px(MARGIN_SMALL),
                    ),
                    ..default()
                },
            ));

            // Scrollable content area
            column
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
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
                            row_gap: Val::Px(MARGIN_SMALL),
                            padding: UiRect::new(
                                Val::Px(SECTION_PADDING),
                                Val::Px(SECTION_PADDING),
                                Val::Px(0.0),
                                Val::Px(SECTION_PADDING),
                            ),
                            ..default()
                        })
                        .with_children(|content| {
                            spawn_content(content);
                        });
                });
        });
}

/// Spawns a stat row with label and value.
fn spawn_stat_row(parent: &mut ChildSpawnerCommands, label: &str, value: u32) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(STAT_LABEL_COLOR),
            ));

            row.spawn((
                Text::new(format!("{}", value)),
                TextFont::from_font_size(STAT_VALUE_FONT_SIZE),
                TextColor(TEXT_COLOR),
            ));
        });
}

/// Spawns a stat row with Insight-colored value.
fn spawn_insight_row(parent: &mut ChildSpawnerCommands, label: &str, value: u32) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(STAT_LABEL_COLOR),
            ));

            row.spawn((
                Text::new(format!("{}", value)),
                TextFont::from_font_size(STAT_VALUE_FONT_SIZE),
                TextColor(INSIGHT_COLOR),
            ));
        });
}

/// Spawns a spell row showing research progress.
fn spawn_spell_research_row(
    parent: &mut ChildSpawnerCommands,
    spell: &Spell,
    is_unlocked: bool,
    progress: u32,
    cost: u32,
) {
    let is_default = cost == 0;

    let (indicator, indicator_color) = if is_unlocked {
        ("*", COMPLETED_COLOR)
    } else if progress > 0 {
        ("~", IN_PROGRESS_COLOR)
    } else {
        ("-", LOCKED_COLOR)
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
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(indicator_color),
                Node {
                    width: Val::Px(16.0),
                    ..default()
                },
            ));

            if is_unlocked {
                // Researched spell: show name
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new(spell.display_name()),
                        TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                        TextColor(if is_default {
                            UNLOCKED_COLOR
                        } else {
                            COMPLETED_COLOR
                        }),
                    ));
                });
            } else if progress > 0 {
                // In-progress: show name + progress
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new(spell.display_name()),
                        TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                        TextColor(IN_PROGRESS_COLOR),
                    ));

                    col.spawn((
                        Text::new(format!("Researching: {}/{}", progress, cost)),
                        TextFont::from_font_size(ITEM_DESC_FONT_SIZE),
                        TextColor(IN_PROGRESS_COLOR),
                    ));
                });
            } else {
                // Locked: show flavor text only
                row.spawn((
                    Text::new(spell.locked_description()),
                    TextFont::from_font_size(ITEM_DESC_FONT_SIZE),
                    TextColor(LOCKED_COLOR),
                ));
            }
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
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(indicator_color),
                Node {
                    width: Val::Px(16.0),
                    ..default()
                },
            ));

            if is_unlocked {
                // Name and description column
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new(achievement.display_name()),
                        TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                        TextColor(name_color),
                    ));

                    col.spawn((
                        Text::new(achievement.description()),
                        TextFont::from_font_size(ITEM_DESC_FONT_SIZE),
                        TextColor(desc_color),
                    ));

                    // Show unlock reward if this achievement has one
                    if let Some(reward) = achievement.unlock_reward() {
                        col.spawn((
                            Text::new(reward),
                            TextFont::from_font_size(ITEM_DESC_FONT_SIZE),
                            TextColor(COMPLETED_COLOR),
                        ));
                    }
                });
            } else {
                // Locked: only show the description as a hint
                row.spawn((
                    Text::new(achievement.description()),
                    TextFont::from_font_size(ITEM_DESC_FONT_SIZE),
                    TextColor(LOCKED_COLOR),
                ));
            }
        });
}

/// Spawns an unlockable item row.
/// When locked, shows the `locked_hint` flavor text (and optionally the name).
fn spawn_unlockable_row(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    description: Option<&str>,
    locked_hint: &str,
    is_unlocked: bool,
    show_name_when_locked: bool,
) {
    let (indicator, indicator_color) = if is_unlocked {
        ("*", COMPLETED_COLOR)
    } else {
        ("-", LOCKED_COLOR)
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
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(indicator_color),
                Node {
                    width: Val::Px(16.0),
                    ..default()
                },
            ));

            if is_unlocked {
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
                            TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                            TextColor(UNLOCKED_COLOR),
                        ));

                        col.spawn((
                            Text::new(desc),
                            TextFont::from_font_size(ITEM_DESC_FONT_SIZE),
                            TextColor(DESCRIPTION_COLOR),
                        ));
                    });
                } else {
                    // Name only
                    row.spawn((
                        Text::new(name),
                        TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                        TextColor(UNLOCKED_COLOR),
                    ));
                }
            } else {
                // Locked: show hint flavor text (and optionally name)
                if show_name_when_locked {
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    })
                    .with_children(|col| {
                        col.spawn((
                            Text::new(name),
                            TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                            TextColor(LOCKED_COLOR),
                        ));

                        col.spawn((
                            Text::new(locked_hint),
                            TextFont::from_font_size(ITEM_DESC_FONT_SIZE),
                            TextColor(LOCKED_COLOR),
                        ));
                    });
                } else {
                    // Only show the locked hint (no name)
                    row.spawn((
                        Text::new(locked_hint),
                        TextFont::from_font_size(ITEM_DESC_FONT_SIZE),
                        TextColor(LOCKED_COLOR),
                    ));
                }
            }
        });
}

/// Spawns a confirmation popup asking if the user really wants to clear progress.
pub(super) fn spawn_confirmation_popup(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(1000),
            ConfirmationPopup,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(500.0),
                        padding: UiRect::all(Val::Px(30.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(20.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.8, 0.3, 0.3)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|popup| {
                    // Title
                    popup.spawn((
                        Text::new("Clear All Progress?"),
                        TextFont::from_font_size(24.0),
                        TextColor(Color::srgb(1.0, 0.4, 0.4)),
                    ));

                    // Warning message
                    popup.spawn((
                        Text::new(
                            "This will permanently delete:\n\n\
                            - All achievements\n\
                            - All unlocked spells and wizards\n\
                            - All wizard progress and levels\n\
                            - All statistics\n\n\
                            This action cannot be undone!",
                        ),
                        TextFont::from_font_size(16.0),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextLayout::new_with_justify(Justify::Left),
                        Node {
                            padding: UiRect::horizontal(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // Buttons row
                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(20.0),
                            ..default()
                        })
                        .with_children(|buttons| {
                            spawn_button(buttons, "Cancel", CancelClearButton, &BUTTON_STYLE);
                            spawn_button(
                                buttons,
                                "Clear Everything",
                                ConfirmClearButton,
                                &DANGER_BUTTON_STYLE,
                            );
                        });
                });
        });
}

/// Clears all achievements and unlockable progress from the save file.
pub(super) fn handle_clear_progress() {
    clear_progress();
}

/// Despawns all progress screen entities and respawns with fresh data (main menu).
pub(super) fn clear_and_refresh_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<OnProgressScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    setup(commands, false);
}

/// Despawns all progress screen entities and respawns with fresh data (pause menu).
pub(super) fn clear_and_refresh_pause_menu(
    mut commands: Commands,
    query: Query<Entity, With<OnProgressScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    setup(commands, true);
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
