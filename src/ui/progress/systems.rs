//! Systems for progress screen.

use bevy::prelude::*;

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
fn setup(mut commands: Commands, pause_menu: bool) {
    use crate::ui::systems::spawn_page_container;

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

    let content = spawn_page_container(
        &mut commands,
        OnProgressScreen,
        pause_menu,
        Overflow::clip(),
    );

    commands.entity(content).with_children(|parent| {
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
                width: Val::Percent(100.0),
                height: Val::Percent(75.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
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

                // Achievements column (unlocked first, alphabetical within each group)
                spawn_column(columns, "Achievements", |section| {
                    let mut achievements: Vec<_> = AchievementId::all()
                        .iter()
                        .map(|a| {
                            let is_unlocked = unlocked_achievements.contains(&a.id().to_string());
                            (a, is_unlocked)
                        })
                        .collect();
                    achievements.sort_by(|(a, a_unlocked), (b, b_unlocked)| {
                        b_unlocked.cmp(a_unlocked)
                            .then_with(|| a.display_name().cmp(b.display_name()))
                    });
                    for (achievement, is_unlocked) in achievements {
                        spawn_achievement_row(section, achievement, is_unlocked);
                    }
                });

                // Spells column (researched first, then in-progress, then locked — alphabetical within each group)
                spawn_column(columns, "Spells", |section| {
                    let mut spells: Vec<_> = Spell::all()
                        .iter()
                        .map(|spell| {
                            let debug_name = format!("{:?}", spell);
                            let is_unlocked = spell.research_cost() == 0
                                || unlocked_content.spells.contains(&debug_name);
                            let progress = get_spell_research_progress(*spell);
                            let cost = spell.research_cost();
                            (spell, is_unlocked, progress, cost)
                        })
                        .collect();
                    // Sort: unlocked first, then in-progress, then locked — alphabetical within each group
                    spells.sort_by(|(a, a_unlocked, a_progress, _), (b, b_unlocked, b_progress, _)| {
                        let a_group = if *a_unlocked { 0 } else if *a_progress > 0 { 1 } else { 2 };
                        let b_group = if *b_unlocked { 0 } else if *b_progress > 0 { 1 } else { 2 };
                        a_group.cmp(&b_group)
                            .then_with(|| a.display_name().cmp(b.display_name()))
                    });
                    for (spell, is_unlocked, progress, cost) in spells {
                        spawn_spell_research_row(section, spell, is_unlocked, progress, cost);
                    }
                });

                // Ingredients column (unlocked first, alphabetical within each group)
                spawn_column(columns, "Ingredients", |section| {
                    let mut ingredients: Vec<_> = Ingredient::all()
                        .iter()
                        .map(|ingredient| {
                            let debug_name = format!("{:?}", ingredient);
                            let is_unlocked = unlocked_content.ingredients.contains(&debug_name);
                            (ingredient, is_unlocked)
                        })
                        .collect();
                    ingredients.sort_by(|(a, a_unlocked), (b, b_unlocked)| {
                        b_unlocked.cmp(a_unlocked)
                            .then_with(|| a.name().cmp(b.name()))
                    });
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

                // Wizard Types column (unlocked first, alphabetical within each group)
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
                    wizard_types.sort_by(|(a, a_unlocked), (b, b_unlocked)| {
                        b_unlocked.cmp(a_unlocked)
                            .then_with(|| a.display_name().cmp(b.display_name()))
                    });
                    for (wizard_type, is_unlocked) in wizard_types {
                        spawn_unlockable_row(
                            section,
                            wizard_type.display_name(),
                            Some(wizard_type.description()),
                            wizard_type.locked_description(),
                            is_unlocked,
                            false, // Hide name when locked
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
                min_width: Val::Px(0.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
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
                        TextColor(UNLOCKED_COLOR),
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

/// Spawns progress for main menu.
pub(super) fn setup_main_menu(commands: Commands) {
    setup(commands, false);
}

/// Spawns progress for pause menu.
pub(super) fn setup_pause_menu(commands: Commands) {
    setup(commands, true);
}

/// Despawns all progress screen entities.
pub(super) fn cleanup(mut commands: Commands, query: Query<Entity, With<OnProgressScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

