use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data::{AchievementId, load_unified_save};
use crate::game::cauldron::brews::Ingredient;
use crate::game::units::UnitType;
use crate::game::units::components::{CompendiumSpriteSpec, SHEET_ROW_FORWARD_FACING};
use crate::game::units::wizard::components::{Spell, SpellCategory};
use crate::ui::components::{ButtonColors, ButtonFront, UnitCompendiumSpriteAssets};
use crate::ui::systems::{spawn_button, spawn_title_with_shadow};

use super::components::*;
use super::constants::*;

// ---------------------------------------------------------------------------
// Setup systems
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands, pause_menu: bool) {
    use crate::ui::systems::spawn_page_container;

    commands.insert_resource(CompendiumState::default());

    let content = spawn_page_container(
        &mut commands,
        OnCompendiumScreen,
        pause_menu,
        crate::ui::systems::default_content_node(),
    );

    commands.entity(content).with_children(|parent| {
        // Header row: title left, Back button right
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                ..default()
            })
            .with_children(|header| {
                spawn_title_with_shadow(
                    header,
                    "Compendium",
                    TITLE_FONT_SIZE,
                    TEXT_COLOR,
                    Node::default(),
                );
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                spawn_button(
                    header,
                    "Back",
                    (BackButton, crate::ui::focus::NoGamepadFocus),
                    &crate::ui::main_menu::BACK_BUTTON_STYLE,
                );
            });

        // Main content: left detail + right tabbed panel
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(90.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(COLUMN_GAP),
                ..default()
            })
            .with_children(|main| {
                // Left detail panel
                spawn_detail_panel(main);

                // Right panel: tabs + content
                spawn_right_panel(main);
            });
    });
}

fn spawn_detail_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Percent(LEFT_PANEL_PERCENT),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(SECTION_PADDING)),
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::scroll_y(),
                flex_shrink: 0.0,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            ScrollPosition::default(),
            BackgroundColor(DETAIL_BG),
            BorderColor::all(DETAIL_BORDER),
            DetailPanel,
            crate::ui::focus::GamepadScrollTarget,
        ))
        .with_children(|panel| {
            // Detail icon (hidden by default, shown when an item is selected).
            // Centered horizontally and pushed down so the corona stays inside
            // the panel. Locked silhouettes get the white corona via BoxShadow.
            panel.spawn((
                ImageNode::default(),
                Node {
                    width: Val::Px(DETAIL_ICON_SIZE),
                    height: Val::Px(DETAIL_ICON_SIZE),
                    align_self: AlignSelf::Center,
                    margin: UiRect {
                        top: Val::Px(DETAIL_ICON_TOP_MARGIN),
                        bottom: Val::Px(8.0),
                        ..default()
                    },
                    display: Display::None,
                    ..default()
                },
                BoxShadow(vec![]),
                DetailIcon,
            ));

            panel.spawn((
                Text::new("Select an item"),
                TextFont::from_font_size(DETAIL_NAME_FONT_SIZE),
                TextColor(TEXT_COLOR),
                DetailTitle,
            ));

            panel.spawn((
                Text::new(""),
                TextFont::from_font_size(DETAIL_CATEGORY_FONT_SIZE),
                TextColor(DESCRIPTION_COLOR),
                DetailCategory,
            ));

            panel.spawn((
                Text::new(""),
                TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                TextColor(UNLOCKED_COLOR),
                DetailDescription,
            ));

            panel.spawn((
                Text::new(""),
                TextFont::from_font_size(DETAIL_FLAVOR_FONT_SIZE),
                TextColor(LOCKED_COLOR),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                DetailFlavor,
            ));

            // Level history container (used by stats tab, hidden by default)
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    display: Display::None,
                    ..default()
                },
                LevelHistoryContainer,
            ));
        });
}

fn spawn_right_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Percent(RIGHT_PANEL_PERCENT),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            RightPanelContent,
        ))
        .with_children(|right| {
            // Tab bar
            right
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(4.0),
                    margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                    ..default()
                })
                .with_children(|tabs| {
                    for tab in CompendiumTab::all() {
                        let is_active = *tab == CompendiumTab::Spells;
                        let (bg, border) = if is_active {
                            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
                        } else {
                            (INACTIVE_TAB_BG, TAB_BORDER)
                        };

                        tabs.spawn((
                            Button,
                            Node {
                                height: Val::Px(TAB_HEIGHT),
                                padding: UiRect::horizontal(Val::Px(TAB_PADDING_H)),
                                border: UiRect::all(Val::Px(1.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(bg),
                            BorderColor::all(border),
                            ButtonColors {
                                background: bg,
                                border,
                            },
                            TabButton(*tab),
                            crate::ui::focus::Focusable,
                            crate::ui::focus::TabFocusable,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(tab.label()),
                                TextFont::from_font_size(TAB_FONT_SIZE),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    }
                });

            // Scrollable content area
            right
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        overflow: Overflow::scroll_y(),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    ScrollPosition::default(),
                    ScrollableCompendiumContainer,
                    BackgroundColor(SECTION_BG),
                ))
                .with_children(|scroll| {
                    scroll.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            padding: UiRect::all(Val::Px(SECTION_PADDING)),
                            ..default()
                        },
                        ItemsContainer,
                    ));
                });
        });
}

pub(super) fn setup_main_menu(commands: Commands) {
    setup(commands, false);
}

pub(super) fn setup_pause_menu(commands: Commands) {
    setup(commands, true);
}

// ---------------------------------------------------------------------------
// Tab switching
// ---------------------------------------------------------------------------

pub(super) fn handle_tab_click(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    tab_query: Query<&TabButton>,
    mut state: ResMut<CompendiumState>,
) {
    for event in button_clicked.read() {
        if let Ok(tab_btn) = tab_query.get(event.button)
            && state.active_tab != tab_btn.0
        {
            state.active_tab = tab_btn.0;
            state.selected_item = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Item selection
// ---------------------------------------------------------------------------

pub(super) fn handle_item_click(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    item_query: Query<&ItemButton>,
    mut state: ResMut<CompendiumState>,
) {
    for event in button_clicked.read() {
        if let Ok(item_btn) = item_query.get(event.button) {
            state.selected_item = Some(item_btn.0.clone());
        }
    }
}

/// Updates ButtonActive markers on item buttons when the selection changes.
/// Separate from rebuild_on_state_change to avoid the system parameter limit.
pub(super) fn update_item_active_state(
    mut commands: Commands,
    state: Res<CompendiumState>,
    item_buttons: Query<(Entity, &ItemButton)>,
) {
    if !state.is_changed() {
        return;
    }
    for (entity, item_btn) in &item_buttons {
        if state.selected_item.as_ref() == Some(&item_btn.0) {
            commands
                .entity(entity)
                .insert(crate::ui::components::ButtonActive);
        } else {
            commands
                .entity(entity)
                .remove::<crate::ui::components::ButtonActive>();
        }
    }
}

pub(super) fn handle_toggle_save_run(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    toggle_query: Query<&ToggleSaveRunButton>,
    mut state: ResMut<CompendiumState>,
) {
    for event in button_clicked.read() {
        if let Ok(toggle_btn) = toggle_query.get(event.button) {
            crate::config::save_data::toggle_roguelite_run_saved(toggle_btn.0);
            // Force rebuild by re-setting the same state (triggers is_changed)
            state.set_changed();
        }
    }
}

pub(super) fn handle_copy_seed(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    copy_query: Query<&CopySeedButton>,
) {
    for event in button_clicked.read() {
        if let Ok(copy_btn) = copy_query.get(event.button)
            && let Ok(mut clipboard) = arboard::Clipboard::new()
        {
            let _ = clipboard.set_text(copy_btn.0.to_string());
        }
    }
}

// ---------------------------------------------------------------------------
// Rebuild content when tab or selection changes
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn rebuild_on_state_change(
    mut commands: Commands,
    mut state: ResMut<CompendiumState>,
    detail_icons: (
        Res<crate::ui::components::SpellIconAssets>,
        Res<UnitCompendiumSpriteAssets>,
    ),
    items_container: Query<Entity, With<ItemsContainer>>,
    tab_buttons: Query<(&TabButton, Entity, &Children)>,
    mut tab_bg: Query<(&mut BackgroundColor, &mut BorderColor, &mut ButtonColors)>,
    mut front_bg: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (With<ButtonFront>, Without<ButtonColors>),
    >,
    mut detail_title: Query<
        &mut Text,
        (
            With<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailDescription>,
            Without<DetailFlavor>,
        ),
    >,
    mut detail_category: Query<
        &mut Text,
        (
            With<DetailCategory>,
            Without<DetailTitle>,
            Without<DetailDescription>,
            Without<DetailFlavor>,
        ),
    >,
    mut detail_desc: Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailFlavor>,
        ),
    >,
    mut detail_flavor: Query<
        &mut Text,
        (
            With<DetailFlavor>,
            Without<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailDescription>,
        ),
    >,
    mut detail_cat_color: Query<&mut TextColor, (With<DetailCategory>, Without<DetailTitle>)>,
    mut detail_icon: Query<(&mut ImageNode, &mut Node, &mut BoxShadow), With<DetailIcon>>,
    level_history: Query<Entity, With<LevelHistoryContainer>>,
    mut detail_desc_node: Query<
        &mut Node,
        (
            With<DetailDescription>,
            Without<DetailIcon>,
            Without<LevelHistoryContainer>,
        ),
    >,
    mut detail_flavor_node: Query<
        &mut Node,
        (
            With<DetailFlavor>,
            Without<DetailIcon>,
            Without<DetailDescription>,
            Without<LevelHistoryContainer>,
        ),
    >,
) {
    if !state.is_changed() {
        return;
    }

    let tab_changed = state.prev_tab != Some(state.active_tab);
    if tab_changed {
        state.prev_tab = Some(state.active_tab);
    }

    // Update tab button visuals (wrapper ButtonColors + front face colors).
    for (tab_btn, entity, children) in &tab_buttons {
        let is_active = tab_btn.0 == state.active_tab;
        let (bg, border) = if is_active {
            commands
                .entity(entity)
                .insert(crate::ui::components::ButtonActive);
            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
        } else {
            commands
                .entity(entity)
                .remove::<crate::ui::components::ButtonActive>();
            (INACTIVE_TAB_BG, TAB_BORDER)
        };
        if let Ok((mut bg_color, mut border_color, mut colors)) = tab_bg.get_mut(entity) {
            *bg_color = bg.into();
            *border_color = BorderColor::all(border);
            colors.background = bg;
            colors.border = border;
        }
        // Also update the 3D front face child.
        for child in children.iter() {
            if let Ok((mut front_bg_color, mut front_border_color)) = front_bg.get_mut(child) {
                *front_bg_color = crate::ui::systems::opaque(bg).into();
                *front_border_color = BorderColor::all(border);
            }
        }
    }

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
    let research_progress = save
        .as_ref()
        .map(|s| s.player.spell_research_progress.clone())
        .unwrap_or_default();

    // Rebuild items list
    // Only rebuild the item list when the tab changes — not on item selection.
    if tab_changed && let Ok(container) = items_container.single() {
        commands.entity(container).despawn_related::<Children>();
        commands
            .entity(container)
            .with_children(|parent| match state.active_tab {
                CompendiumTab::Spells => {
                    spawn_spell_items(parent, &unlocked_content.spells, &research_progress, &state)
                }
                CompendiumTab::Ingredients => {
                    spawn_ingredient_items(parent, &unlocked_content.ingredients, &state)
                }
                CompendiumTab::Units => spawn_unit_items(parent, &unlocked_content.units, &state),
                CompendiumTab::Wizards => {
                    spawn_wizard_items(parent, &unlocked_content.wizard_types, &state)
                }
                CompendiumTab::Achievements => {
                    spawn_achievement_items(parent, &unlocked_achievements, &state)
                }
                CompendiumTab::Stats => spawn_stats_items(parent, save.as_ref()),
                CompendiumTab::Endless => spawn_endless_items(parent, save.as_ref(), &state),
                CompendiumTab::Roguelite => spawn_roguelite_items(parent, save.as_ref(), &state),
            });
    }

    // Update detail panel (including icon)
    let (icon_assets, unit_sprite_assets) = (&detail_icons.0, &detail_icons.1);
    update_detail_panel(
        &state,
        icon_assets,
        unit_sprite_assets,
        &unlocked_content.spells,
        &unlocked_content.ingredients,
        &unlocked_content.units,
        &unlocked_content.wizard_types,
        &unlocked_achievements,
        &mut detail_title,
        &mut detail_category,
        &mut detail_desc,
        &mut detail_flavor,
        &mut detail_cat_color,
        &mut detail_icon,
    );

    // Determine if the level history container should be shown
    // Stats tab: always show level history (no selectable items)
    // Endless/Roguelite: show detail when an item IS selected
    let show_level_history = match state.active_tab {
        CompendiumTab::Stats => state.selected_item.is_none(),
        CompendiumTab::Endless => matches!(
            state.selected_item,
            Some(CompendiumItemId::EndlessWizardType(_))
        ),
        CompendiumTab::Roguelite => {
            matches!(state.selected_item, Some(CompendiumItemId::RogueliteRun(_)))
        }
        _ => false,
    };
    if let Ok(container) = level_history.single() {
        commands.entity(container).despawn_related::<Children>();
        if show_level_history {
            // Show level history, hide description/flavor text
            commands.entity(container).insert(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                display: Display::Flex,
                ..default()
            });
            let selected = state.selected_item.clone();
            commands.entity(container).with_children(|parent| {
                match (&state.active_tab, &selected) {
                    (CompendiumTab::Endless, Some(CompendiumItemId::EndlessWizardType(name))) => {
                        spawn_endless_detail_for_wizard(parent, save.as_ref(), name);
                    }
                    (
                        CompendiumTab::Roguelite,
                        Some(CompendiumItemId::RogueliteRun(started_at)),
                    ) => {
                        spawn_roguelite_run_detail(parent, save.as_ref(), *started_at);
                    }
                    _ => spawn_level_history_rows(parent, save.as_ref()),
                }
            });
        } else {
            commands.entity(container).insert(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                display: Display::None,
                ..default()
            });
        }
    }

    // Hide/show description and flavor text based on level history display
    if let Ok(mut node) = detail_desc_node.single_mut() {
        node.display = if show_level_history {
            Display::None
        } else {
            Display::Flex
        };
    }
    if let Ok(mut node) = detail_flavor_node.single_mut() {
        node.display = if show_level_history {
            Display::None
        } else {
            Display::Flex
        };
    }
}

// ---------------------------------------------------------------------------
// Item spawning per tab
// ---------------------------------------------------------------------------

fn spawn_spell_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_spells: &[String],
    research_progress: &std::collections::HashMap<String, u32>,
    state: &CompendiumState,
) {
    for category in SpellCategory::all() {
        // Category header
        parent.spawn((
            Text::new(category.display_name()),
            TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
            TextColor(crate::ui::constants::spell_category_color(*category)),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(6.0), Val::Px(2.0)),
                ..default()
            },
        ));

        for spell in category.spells() {
            let debug_name = format!("{:?}", spell);
            let is_unlocked = spell.research_cost() == 0 || unlocked_spells.contains(&debug_name);
            let progress = research_progress.get(&debug_name).copied().unwrap_or(0);
            let cost = spell.research_cost();

            let display_text = if is_unlocked {
                spell.display_name().to_string()
            } else if progress > 0 {
                format!("{} ({}/{})", spell.display_name(), progress, cost)
            } else {
                "???".to_string()
            };

            let text_color = if is_unlocked {
                UNLOCKED_COLOR
            } else if progress > 0 {
                IN_PROGRESS_COLOR
            } else {
                LOCKED_COLOR
            };

            spawn_item_button(
                parent,
                &display_text,
                text_color,
                CompendiumItemId::Spell(debug_name),
                &state.selected_item,
            );
        }
    }
}

fn spawn_ingredient_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_ingredients: &[String],
    state: &CompendiumState,
) {
    let mut ingredients: Vec<_> = Ingredient::all()
        .iter()
        .map(|i| {
            let debug_name = format!("{:?}", i);
            let is_unlocked = unlocked_ingredients.contains(&debug_name);
            (i, debug_name, is_unlocked)
        })
        .collect();
    ingredients.sort_by(|(_, _, a), (_, _, b)| b.cmp(a));

    for (ingredient, debug_name, is_unlocked) in ingredients {
        let display_text = if is_unlocked {
            ingredient.name().to_string()
        } else {
            "???".to_string()
        };
        let text_color = if is_unlocked {
            UNLOCKED_COLOR
        } else {
            LOCKED_COLOR
        };
        spawn_item_button(
            parent,
            &display_text,
            text_color,
            CompendiumItemId::Ingredient(debug_name),
            &state.selected_item,
        );
    }
}

fn team_label_color(label: &str) -> Color {
    match label {
        "Defender" => TEAM_DEFENDER_COLOR,
        "Attacker" => TEAM_ATTACKER_COLOR,
        "Boss" => TEAM_BOSS_COLOR,
        _ => DESCRIPTION_COLOR,
    }
}

fn spawn_unit_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_units: &[String],
    state: &CompendiumState,
) {
    // Group by team label
    for team_label in &["Defender", "Attacker", "Boss"] {
        parent.spawn((
            Text::new(*team_label),
            TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
            TextColor(team_label_color(team_label)),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(6.0), Val::Px(2.0)),
                ..default()
            },
        ));

        for unit_type in UnitType::all() {
            if unit_type.team_label() != *team_label {
                continue;
            }
            let debug_name = format!("{:?}", unit_type);
            let is_unlocked = unlocked_units.contains(&debug_name);

            let display_text = if is_unlocked {
                unit_type.display_name().to_string()
            } else {
                "???".to_string()
            };
            let text_color = if is_unlocked {
                UNLOCKED_COLOR
            } else {
                LOCKED_COLOR
            };
            spawn_item_button(
                parent,
                &display_text,
                text_color,
                CompendiumItemId::Unit(debug_name),
                &state.selected_item,
            );
        }
    }
}

fn spawn_wizard_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_wizard_types: &[String],
    state: &CompendiumState,
) {
    for wizard_type in WizardType::all() {
        let debug_name = format!("{:?}", wizard_type);
        let is_unlocked = *wizard_type == WizardType::BoringOleMage
            || unlocked_wizard_types.contains(&debug_name);

        let display_text = if is_unlocked {
            wizard_type.display_name().to_string()
        } else {
            "???".to_string()
        };
        let text_color = if is_unlocked {
            UNLOCKED_COLOR
        } else {
            LOCKED_COLOR
        };
        spawn_item_button(
            parent,
            &display_text,
            text_color,
            CompendiumItemId::Wizard(debug_name),
            &state.selected_item,
        );
    }
}

fn spawn_achievement_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_achievements: &[String],
    state: &CompendiumState,
) {
    let mut achievements: Vec<_> = AchievementId::all()
        .iter()
        .map(|a| {
            let is_unlocked = unlocked_achievements.contains(&a.id().to_string());
            (a, is_unlocked)
        })
        .collect();
    achievements.sort_by(|(a, a_unlocked), (b, b_unlocked)| {
        b_unlocked
            .cmp(a_unlocked)
            .then_with(|| a.display_name().cmp(b.display_name()))
    });

    for (achievement, is_unlocked) in achievements {
        let display_text = if is_unlocked {
            achievement.display_name().to_string()
        } else {
            "???".to_string()
        };
        let text_color = if is_unlocked {
            UNLOCKED_COLOR
        } else {
            LOCKED_COLOR
        };
        spawn_item_button(
            parent,
            &display_text,
            text_color,
            CompendiumItemId::Achievement(achievement.id().to_string()),
            &state.selected_item,
        );
    }
}

fn spawn_stats_items(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
) {
    let total_games = save.map(|s| s.player.total_games_played).unwrap_or(0);
    let total_victories = save.map(|s| s.player.total_levels_completed).unwrap_or(0);
    let total_attackers = save.map(|s| s.player.total_attackers_killed).unwrap_or(0);
    let total_defenders = save.map(|s| s.player.total_defenders_killed).unwrap_or(0);
    let total_undead = save.map(|s| s.player.total_undead_killed).unwrap_or(0);
    let insight_balance = save.map(|s| s.player.arcane_insight).unwrap_or(0);

    // Highest level achieved across all wizards
    let highest_level = save
        .map(|s| {
            s.wizards
                .iter()
                .map(|w| w.highest_level_achieved)
                .max()
                .unwrap_or(1)
        })
        .unwrap_or(1);

    // Total wizards created
    let wizards_created = save.map(|s| s.wizards.len() as u32).unwrap_or(0);

    // Spells unlocked
    let spells_unlocked = save
        .map(|s| s.player.unlocked_content.spells.len() as u32)
        .unwrap_or(0);
    let total_spells = Spell::all().len() as u32;

    // Ingredients unlocked
    let ingredients_unlocked = save
        .map(|s| s.player.unlocked_content.ingredients.len() as u32)
        .unwrap_or(0);
    let total_ingredients = Ingredient::all().len() as u32;

    // Achievements unlocked
    let achievements_unlocked = save
        .map(|s| s.player.unlocked_achievements.len() as u32)
        .unwrap_or(0);
    let total_achievements = AchievementId::all().len() as u32;

    // Talents unlocked (count non-negative selections across all spells)
    let talents_unlocked = save
        .map(|s| {
            s.player
                .spell_talent_selections
                .values()
                .flat_map(|v| v.iter())
                .filter(|&&sel| sel >= 0)
                .count() as u32
        })
        .unwrap_or(0);
    // Total possible talents = 3 tiers × number of spells that have talent trees
    // (all spells have talent trees)
    let total_talents = Spell::all().len() as u32 * 3;

    // Total kills
    let total_kills = total_attackers + total_undead;

    // Win rate
    let win_rate = if total_games > 0 {
        format!(
            "{:.0}%",
            (total_victories as f32 / total_games as f32) * 100.0
        )
    } else {
        "N/A".to_string()
    };

    // Two-column layout
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(COLUMN_GAP),
            ..default()
        })
        .with_children(|columns| {
            // Left column
            columns
                .spawn(Node {
                    width: Val::Percent(50.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|col| {
                    spawn_stat_section_header(col, "Battle Stats");
                    spawn_stat_row(col, "Games Played", total_games);
                    spawn_stat_row(col, "Victories", total_victories);
                    spawn_stat_text_row(col, "Win Rate", &win_rate);
                    spawn_stat_row(col, "Highest Level", highest_level);

                    spawn_stat_section_header(col, "Kill Stats");
                    spawn_stat_row(col, "Total Kills", total_kills);
                    spawn_stat_row(col, "Attackers Killed", total_attackers);
                    spawn_stat_row(col, "Undead Killed", total_undead);
                    spawn_stat_row(col, "Defenders Lost", total_defenders);
                });

            // Right column
            columns
                .spawn(Node {
                    width: Val::Percent(50.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|col| {
                    spawn_stat_section_header(col, "Collection");
                    spawn_stat_text_row(
                        col,
                        "Spells Unlocked",
                        &format!("{}/{}", spells_unlocked, total_spells),
                    );
                    spawn_stat_text_row(
                        col,
                        "Ingredients Found",
                        &format!("{}/{}", ingredients_unlocked, total_ingredients),
                    );
                    spawn_stat_text_row(
                        col,
                        "Talents Unlocked",
                        &format!("{}/{}", talents_unlocked, total_talents),
                    );
                    spawn_stat_text_row(
                        col,
                        "Achievements",
                        &format!("{}/{}", achievements_unlocked, total_achievements),
                    );
                    spawn_stat_row(col, "Wizards Created", wizards_created);

                    spawn_stat_section_header(col, "Economy");
                    spawn_insight_row(col, "Arcane Insight", insight_balance);
                });
        });
}

// ---------------------------------------------------------------------------
// Endless mode tab
// ---------------------------------------------------------------------------

fn spawn_endless_items(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
    state: &CompendiumState,
) {
    // Show wizard types that have endless data as clickable items
    let mut wizard_types_with_data: Vec<(WizardType, u32)> = Vec::new();

    if let Some(save) = save {
        // Collect wizard types that have endless stats, with their highest level
        let mut type_levels: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        for wizard in &save.wizards {
            if wizard.endless_best_stats.is_empty() {
                continue;
            }
            let type_name = format!("{:?}", wizard.wizard_type);
            let max_level = wizard
                .endless_best_stats
                .keys()
                .filter_map(|k| k.parse::<u32>().ok())
                .max()
                .unwrap_or(0);
            let entry = type_levels.entry(type_name).or_insert(0);
            if max_level > *entry {
                *entry = max_level;
            }
        }

        for wizard_type in WizardType::all() {
            let debug_name = format!("{:?}", wizard_type);
            if let Some(&highest) = type_levels.get(&debug_name) {
                wizard_types_with_data.push((*wizard_type, highest));
            }
        }
    }

    if wizard_types_with_data.is_empty() {
        parent.spawn((
            Text::new("No endless levels completed yet.\nPlay Endless mode to track your best stats per level."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    }

    for (wizard_type, highest_level) in &wizard_types_with_data {
        let debug_name = format!("{:?}", wizard_type);
        let label = format!("{} (Lv{})", wizard_type.display_name(), highest_level);
        spawn_item_button(
            parent,
            &label,
            UNLOCKED_COLOR,
            CompendiumItemId::EndlessWizardType(debug_name),
            &state.selected_item,
        );
    }
}

fn spawn_endless_detail_for_wizard(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
    wizard_type_name: &str,
) {
    use crate::game::game_mode::components::format_time;

    // Aggregate best stats for this wizard type across all wizard saves of that type
    let mut all_levels: std::collections::BTreeMap<
        u32,
        crate::config::save_data::EndlessLevelBest,
    > = std::collections::BTreeMap::new();

    if let Some(save) = save {
        for wizard in &save.wizards {
            if format!("{:?}", wizard.wizard_type) != wizard_type_name {
                continue;
            }
            for (level_str, stats) in &wizard.endless_best_stats {
                if let Ok(level) = level_str.parse::<u32>() {
                    all_levels
                        .entry(level)
                        .and_modify(|existing| {
                            if stats.best_efficiency > existing.best_efficiency {
                                *existing = stats.clone();
                            }
                        })
                        .or_insert_with(|| stats.clone());
                }
            }
        }
    }

    if all_levels.is_empty() {
        parent.spawn((
            Text::new("No data yet."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    }

    for (&level, stats) in &all_levels {
        let boss = crate::game::constants::boss_name_for_level(level);
        let label = if let Some(name) = boss {
            format!("Level {} ({})", level, name)
        } else {
            format!("Level {}", level)
        };
        let color = efficiency_color(stats.best_efficiency);

        // Level header
        parent.spawn((
            Text::new(&label),
            TextFont {
                font_size: STAT_SECTION_FONT_SIZE,
                ..default()
            },
            TextColor(color),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(8.0), Val::Px(2.0)),
                ..default()
            },
        ));

        spawn_stat_text_row(
            parent,
            "Efficiency",
            &format!("{:.0}%", stats.best_efficiency * 100.0),
        );
        spawn_stat_row(parent, "Attackers Killed", stats.attackers_killed);
        spawn_stat_row(parent, "Undead Killed", stats.undead_killed);
        spawn_stat_row(parent, "Defenders Lost", stats.defenders_lost);
        spawn_stat_text_row(parent, "Time", &format_time(stats.elapsed_time));
    }
}

// ---------------------------------------------------------------------------
// Roguelite mode tab
// ---------------------------------------------------------------------------

/// Collects all roguelite runs across all wizards, sorted by most recent first.
fn collect_all_roguelite_runs(
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
) -> Vec<&crate::config::save_data::RogueliteRun> {
    let mut all_runs: Vec<&crate::config::save_data::RogueliteRun> = Vec::new();
    if let Some(save) = save {
        for wizard in &save.wizards {
            all_runs.extend(wizard.roguelite.run_history.iter());
        }
    }
    all_runs.sort_by(|a, b| b.ended_at.cmp(&a.ended_at));
    all_runs
}

fn spawn_roguelite_items(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
    state: &CompendiumState,
) {
    let all_runs = collect_all_roguelite_runs(save);

    if all_runs.is_empty() {
        parent.spawn((
            Text::new("No roguelite runs completed yet.\nPlay Roguelite mode to track your runs."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    }

    let total_runs = all_runs.len();
    let victories = all_runs.iter().filter(|r| r.victory).count();

    // Summary header
    parent.spawn((
        Text::new(format!("{} runs — {} victories", total_runs, victories)),
        TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
        TextColor(DESCRIPTION_COLOR),
        Node {
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
    ));

    // Saved runs section
    let saved_runs: Vec<(usize, &crate::config::save_data::RogueliteRun)> = all_runs
        .iter()
        .enumerate()
        .filter(|(_, r)| r.saved)
        .map(|(i, r)| (total_runs - i, *r))
        .collect();

    if !saved_runs.is_empty() {
        spawn_stat_section_header(parent, "Saved Runs");
        for (run_number, run) in &saved_runs {
            spawn_roguelite_run_button(parent, run, state, *run_number);
        }
    }

    // Recent (unsaved) runs section
    let recent_runs: Vec<(usize, &crate::config::save_data::RogueliteRun)> = all_runs
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.saved)
        .take(20)
        .map(|(i, r)| (total_runs - i, *r))
        .collect();

    if !recent_runs.is_empty() {
        spawn_stat_section_header(parent, "Recent Runs");
        for (run_number, run) in &recent_runs {
            spawn_roguelite_run_button(parent, run, state, *run_number);
        }
    }
}

fn spawn_roguelite_run_button(
    parent: &mut ChildSpawnerCommands,
    run: &crate::config::save_data::RogueliteRun,
    state: &CompendiumState,
    run_number: usize,
) {
    let outcome = if run.victory { "Victory" } else { "Defeat" };
    let outcome_color = if run.victory {
        Color::srgb(0.3, 0.8, 0.3)
    } else {
        Color::srgb(0.8, 0.3, 0.3)
    };

    let assist_marker = if run.accessibility_assists { " *" } else { "" };
    let label = format!(
        "Run #{} — {} (Lv{}, {}){}",
        run_number,
        outcome,
        run.levels_completed,
        run.wizard_type.display_name(),
        assist_marker,
    );
    spawn_item_button(
        parent,
        &label,
        outcome_color,
        CompendiumItemId::RogueliteRun(run.started_at),
        &state.selected_item,
    );
}

fn spawn_roguelite_run_detail(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
    started_at: u64,
) {
    use crate::game::game_mode::components::{RunAggregateStats, format_time};

    // Find the run by started_at timestamp directly instead of re-sorting all runs
    let run = save.and_then(|s| {
        s.wizards
            .iter()
            .flat_map(|w| w.roguelite.run_history.iter())
            .find(|r| r.started_at == started_at)
    });

    let Some(run) = run else {
        parent.spawn((
            Text::new("Run not found."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    };

    // Save/Unsave button
    let (btn_label, btn_color) = if run.saved {
        ("Unsave Run", Color::srgb(0.8, 0.3, 0.3))
    } else {
        ("Save Run", Color::srgb(0.3, 0.6, 0.9))
    };
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(ITEM_BG),
            BorderColor::all(btn_color),
            ButtonColors {
                background: ITEM_BG,
                border: btn_color,
            },
            ToggleSaveRunButton(run.started_at),
            crate::ui::focus::Focusable,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(btn_label),
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(btn_color),
            ));
        });

    // Run summary stats
    let agg = RunAggregateStats::from_level_stats(&run.level_stats);
    spawn_stat_section_header(parent, "Run Summary");
    spawn_stat_row(parent, "Levels Completed", run.levels_completed);
    spawn_stat_row(parent, "Total Kills", agg.total_kills);
    spawn_stat_text_row(
        parent,
        "Avg Efficiency",
        &format!("{:.0}%", agg.avg_efficiency * 100.0),
    );
    spawn_stat_text_row(parent, "Total Time", &format_time(agg.total_time));

    // Seed with copy button
    if let Some(seed) = run.seed {
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new("Seed"),
                    TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                    TextColor(STAT_LABEL_COLOR),
                ));
                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(seed.to_string()),
                        TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                    ));
                    row.spawn((
                        Button,
                        Node {
                            padding: UiRect::new(
                                Val::Px(6.0),
                                Val::Px(6.0),
                                Val::Px(2.0),
                                Val::Px(2.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(ITEM_BG),
                        BorderColor::all(Color::srgb(0.3, 0.6, 0.9)),
                        ButtonColors {
                            background: ITEM_BG,
                            border: Color::srgb(0.3, 0.6, 0.9),
                        },
                        CopySeedButton(seed),
                        crate::ui::focus::Focusable,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Copy"),
                            TextFont::from_font_size(10.0),
                            TextColor(Color::srgb(0.3, 0.6, 0.9)),
                        ));
                    });
                });
            });
    }

    // Modifier display
    if let Some(ref mods) = run.modifiers {
        spawn_stat_section_header(parent, "Run Settings");
        spawn_stat_text_row(
            parent,
            "Wave Speed",
            &format!("{}%", (mods.game_speed * 100.0) as u32),
        );
        spawn_stat_text_row(
            parent,
            "Enemy Strength",
            &format!("{}%", (mods.enemy_effectiveness * 100.0) as u32),
        );
        spawn_stat_text_row(
            parent,
            "Enemy Count",
            &format!("{}%", (mods.enemy_count * 100.0) as u32),
        );
        spawn_stat_text_row(
            parent,
            "Terrain",
            &format!("{}%", (mods.terrain_density * 100.0) as u32),
        );
    }

    if run.accessibility_assists {
        spawn_stat_text_row(parent, "Accessibility", "Assists Used");
    }

    // Level-by-level breakdown
    spawn_stat_section_header(parent, "Level Breakdown");
    for stats in &run.level_stats {
        let color = efficiency_color(stats.efficiency);
        let label = format!("Level {}:", stats.level);
        let value = format!(
            "{:.0}% — {} kills, {}",
            stats.efficiency * 100.0,
            stats.total_kills(),
            format_time(stats.elapsed_time),
        );

        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(label),
                    TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                    TextColor(color),
                    Node {
                        flex_shrink: 0.0,
                        min_width: Val::Px(70.0),
                        ..default()
                    },
                ));
                row.spawn((
                    Text::new(value),
                    TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                    TextColor(color),
                    Node {
                        flex_shrink: 1.0,
                        ..default()
                    },
                ));
            });
    }
}

// ---------------------------------------------------------------------------
// Shared item button spawner
// ---------------------------------------------------------------------------

fn spawn_item_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    text_color: Color,
    item_id: CompendiumItemId,
    selected: &Option<CompendiumItemId>,
) {
    let is_selected = *selected == Some(item_id.clone());
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(ITEM_BG),
            BorderColor::all(ITEM_BORDER),
            ButtonColors {
                background: ITEM_BG,
                border: ITEM_BORDER,
            },
            crate::ui::focus::Focusable,
            ItemButton(item_id),
        ))
        .insert_if(crate::ui::components::ButtonActive, || is_selected)
        .with_children(|btn| {
            btn.spawn((
                Text::new(text),
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(text_color),
            ));
        });
}

// ---------------------------------------------------------------------------
// Stat rows (for stats tab)
// ---------------------------------------------------------------------------

fn spawn_stat_section_header(parent: &mut ChildSpawnerCommands, title: &str) {
    parent.spawn((
        Text::new(title),
        TextFont {
            font_size: STAT_SECTION_FONT_SIZE,
            ..default()
        },
        TextColor(STAT_SECTION_COLOR),
        Node {
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(10.0), Val::Px(2.0)),
            ..default()
        },
    ));
}

fn spawn_stat_row(parent: &mut ChildSpawnerCommands, label: &str, value: u32) {
    spawn_stat_value_row(parent, label, &format!("{}", value), TEXT_COLOR);
}

fn spawn_stat_text_row(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    spawn_stat_value_row(parent, label, value, TEXT_COLOR);
}

fn spawn_insight_row(parent: &mut ChildSpawnerCommands, label: &str, value: u32) {
    spawn_stat_value_row(parent, label, &format!("{}", value), INSIGHT_COLOR);
}

fn spawn_stat_value_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    value_color: Color,
) {
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
                Text::new(value),
                TextFont::from_font_size(STAT_VALUE_FONT_SIZE),
                TextColor(value_color),
            ));
        });
}

// ---------------------------------------------------------------------------
// Level history for stats detail panel
// ---------------------------------------------------------------------------

fn spawn_level_history_rows(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
) {
    use crate::game::constants::boss_name_for_level;

    let Some(save) = save else {
        parent.spawn((
            Text::new("No data yet."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    };

    // Aggregate best efficiency per level across all wizards
    let mut best_efficiency: std::collections::BTreeMap<u32, f32> =
        std::collections::BTreeMap::new();

    for wizard in &save.wizards {
        for (level_str, &eff) in &wizard.efficiency_ratios {
            if let Ok(level) = level_str.parse::<u32>() {
                let entry = best_efficiency.entry(level).or_insert(0.0);
                if eff > *entry {
                    *entry = eff;
                }
            }
        }
    }

    if best_efficiency.is_empty() {
        parent.spawn((
            Text::new("No levels completed yet."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    }

    for (level, eff) in &best_efficiency {
        let label = if let Some(boss) = boss_name_for_level(*level) {
            format!("Level {} ({}):", level, boss)
        } else {
            format!("Level {}:", level)
        };
        let pct_text = format!("{:.0}%", eff * 100.0);
        let color = efficiency_color(*eff);

        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(label),
                    TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                    TextColor(color),
                    Node {
                        flex_shrink: 1.0,
                        ..default()
                    },
                ));
                row.spawn((
                    Text::new(pct_text),
                    TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                    TextColor(color),
                    Node {
                        min_width: Val::Px(40.0),
                        justify_content: JustifyContent::FlexEnd,
                        ..default()
                    },
                    TextLayout::new_with_justify(Justify::Right),
                ));
            });
    }
}

/// Returns a color for the efficiency value:
/// - 100%: gold glow
/// - 0-99%: gradient from red (0%) to green (99%)
use crate::ui::constants::efficiency_color;

// ---------------------------------------------------------------------------
// Detail panel update
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn update_detail_panel(
    state: &CompendiumState,
    icon_assets: &crate::ui::components::SpellIconAssets,
    unit_sprite_assets: &UnitCompendiumSpriteAssets,
    unlocked_spells: &[String],
    unlocked_ingredients: &[String],
    unlocked_units: &[String],
    unlocked_wizard_types: &[String],
    unlocked_achievements: &[String],
    title_q: &mut Query<
        &mut Text,
        (
            With<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailDescription>,
            Without<DetailFlavor>,
        ),
    >,
    category_q: &mut Query<
        &mut Text,
        (
            With<DetailCategory>,
            Without<DetailTitle>,
            Without<DetailDescription>,
            Without<DetailFlavor>,
        ),
    >,
    desc_q: &mut Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailFlavor>,
        ),
    >,
    flavor_q: &mut Query<
        &mut Text,
        (
            With<DetailFlavor>,
            Without<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailDescription>,
        ),
    >,
    cat_color_q: &mut Query<&mut TextColor, (With<DetailCategory>, Without<DetailTitle>)>,
    detail_icon: &mut Query<(&mut ImageNode, &mut Node, &mut BoxShadow), With<DetailIcon>>,
) {
    // Helper to hide the icon and reset all per-item state so subsequent
    // shows start from a clean baseline (no leftover tint/atlas/size/corona).
    let hide_icon =
        |detail_icon: &mut Query<(&mut ImageNode, &mut Node, &mut BoxShadow), With<DetailIcon>>| {
            if let Ok((mut img, mut node, mut shadow)) = detail_icon.single_mut() {
                node.display = Display::None;
                node.width = Val::Px(DETAIL_ICON_SIZE);
                node.height = Val::Px(DETAIL_ICON_SIZE);
                img.color = Color::WHITE;
                img.texture_atlas = None;
                shadow.0.clear();
            }
        };

    let apply_locked_appearance =
        |img: &mut ImageNode, shadow: &mut BoxShadow, is_unlocked: bool, base_tint: Color| {
            if is_unlocked {
                img.color = base_tint;
                shadow.0.clear();
            } else {
                img.color = Color::BLACK;
                shadow.0 = vec![locked_silhouette_corona()];
            }
        };

    let Some(ref item) = state.selected_item else {
        hide_icon(detail_icon);

        let (title, subtitle) = match state.active_tab {
            CompendiumTab::Stats => ("Level History", "Best efficiency per level"),
            CompendiumTab::Endless => ("Endless Mode", "Select a wizard to view stats"),
            CompendiumTab::Roguelite => ("Roguelite Mode", "Select a run to view details"),
            _ => ("Select an item", ""),
        };
        if let Ok(mut t) = title_q.single_mut() {
            **t = title.to_string();
        }
        if let Ok(mut t) = category_q.single_mut() {
            **t = subtitle.to_string();
        }
        if let Ok(mut t) = desc_q.single_mut() {
            **t = String::new();
        }
        if let Ok(mut t) = flavor_q.single_mut() {
            **t = String::new();
        }
        return;
    };

    match item {
        CompendiumItemId::Spell(debug_name) => {
            let spell = Spell::all()
                .iter()
                .find(|s| format!("{:?}", s) == *debug_name);
            if let Some(spell) = spell {
                let is_unlocked =
                    spell.research_cost() == 0 || unlocked_spells.contains(debug_name);

                // Show spell icon — silhouette + corona when locked, full color when unlocked.
                if let Ok((mut img, mut node, mut shadow)) = detail_icon.single_mut() {
                    match icon_assets.get(spell) {
                        Some(handle) => {
                            img.image = handle.clone();
                            img.texture_atlas = None;
                            node.width = Val::Px(DETAIL_ICON_SIZE);
                            node.height = Val::Px(DETAIL_ICON_SIZE);
                            node.display = Display::Flex;
                            apply_locked_appearance(
                                &mut img,
                                &mut shadow,
                                is_unlocked,
                                Color::WHITE,
                            );
                        }
                        None => {
                            node.display = Display::None;
                            shadow.0.clear();
                        }
                    }
                }

                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        spell.display_name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = if is_unlocked {
                        format!(
                            "{} - {}",
                            spell.category().display_name(),
                            spell.damage_type().display_name()
                        )
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut c) = cat_color_q.single_mut() {
                    c.0 = if is_unlocked {
                        crate::ui::constants::spell_category_color(spell.category())
                    } else {
                        DESCRIPTION_COLOR
                    };
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = if is_unlocked {
                        format!("{}\n\n{}", spell.description(), spell.instructions())
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = spell.locked_description().to_string();
                }
            }
        }
        CompendiumItemId::Ingredient(debug_name) => {
            hide_icon(detail_icon);
            let ingredient = Ingredient::all()
                .iter()
                .find(|i| format!("{:?}", i) == *debug_name);
            if let Some(ingredient) = ingredient {
                let is_unlocked = unlocked_ingredients.contains(debug_name);
                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        ingredient.name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = if is_unlocked {
                        ingredient.category().display_name().to_string()
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = if is_unlocked {
                        ingredient.description()
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = ingredient.locked_description().to_string();
                }
            }
        }
        CompendiumItemId::Unit(debug_name) => {
            let unit_type = UnitType::all()
                .iter()
                .find(|u| format!("{:?}", u) == *debug_name);
            if let Some(unit_type) = unit_type {
                let is_unlocked = unlocked_units.contains(debug_name);

                // Render the unit portrait — colored when unlocked, BLACK silhouette
                // with a white corona when locked.
                if let Ok((mut img, mut node, mut shadow)) = detail_icon.single_mut() {
                    match unit_sprite_assets.images.get(unit_type) {
                        Some(image_handle) => {
                            let spec = unit_type.compendium_sprite();
                            img.image = image_handle.clone();
                            img.texture_atlas = match spec {
                                CompendiumSpriteSpec::Atlas {
                                    sheet_size,
                                    frame_px,
                                    ..
                                } => unit_sprite_assets.layouts.get(unit_type).map(|layout| {
                                    let cols = (sheet_size.0 / frame_px) as usize;
                                    TextureAtlas {
                                        layout: layout.clone(),
                                        index: SHEET_ROW_FORWARD_FACING * cols,
                                    }
                                }),
                                CompendiumSpriteSpec::Static { .. } => None,
                            };
                            let (base_tint, size_mul) = match spec {
                                CompendiumSpriteSpec::Static {
                                    size_multiplier, ..
                                } => (Color::WHITE, size_multiplier),
                                CompendiumSpriteSpec::Atlas {
                                    tint,
                                    size_multiplier,
                                    ..
                                } => (tint, size_multiplier),
                            };
                            apply_locked_appearance(&mut img, &mut shadow, is_unlocked, base_tint);
                            let px = DETAIL_UNIT_ICON_SIZE * size_mul;
                            node.width = Val::Px(px);
                            node.height = Val::Px(px);
                            node.display = Display::Flex;
                        }
                        None => hide_icon(detail_icon),
                    }
                }
                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        unit_type.display_name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = if is_unlocked {
                        unit_type.team_label().to_string()
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut c) = cat_color_q.single_mut() {
                    c.0 = if is_unlocked {
                        team_label_color(unit_type.team_label())
                    } else {
                        DESCRIPTION_COLOR
                    };
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = if is_unlocked {
                        unit_type.description().to_string()
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = if is_unlocked {
                        unit_type.flavor_text().to_string()
                    } else {
                        unit_type.locked_description().to_string()
                    };
                }
                return; // Early return to skip the category color reset below
            }
        }
        CompendiumItemId::Wizard(debug_name) => {
            hide_icon(detail_icon);
            let wizard_type = WizardType::all()
                .iter()
                .find(|w| format!("{:?}", w) == *debug_name);
            if let Some(wizard_type) = wizard_type {
                let is_unlocked = *wizard_type == WizardType::BoringOleMage
                    || unlocked_wizard_types.contains(debug_name);
                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        wizard_type.display_name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = "Wizard Type".to_string();
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = if is_unlocked {
                        format!(
                            "{}\n\n{}",
                            wizard_type.description(),
                            wizard_type.long_description()
                        )
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = wizard_type.locked_description().to_string();
                }
            }
        }
        CompendiumItemId::Achievement(id_str) => {
            hide_icon(detail_icon);
            let achievement = AchievementId::all()
                .iter()
                .find(|a| a.id() == id_str.as_str());
            if let Some(achievement) = achievement {
                let is_unlocked = unlocked_achievements.contains(id_str);
                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        achievement.display_name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = "Achievement".to_string();
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = achievement.description().to_string();
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = if is_unlocked {
                        achievement.unlock_reward().unwrap_or("").to_string()
                    } else {
                        String::new()
                    };
                }
            }
        }
        CompendiumItemId::EndlessWizardType(debug_name) => {
            hide_icon(detail_icon);
            let wizard_type = WizardType::all()
                .iter()
                .find(|w| format!("{:?}", w) == *debug_name);
            if let Ok(mut t) = title_q.single_mut() {
                **t = wizard_type
                    .map(|w| format!("{} — Endless", w.display_name()))
                    .unwrap_or_else(|| "Endless Mode".to_string());
            }
            if let Ok(mut t) = category_q.single_mut() {
                **t = "Best stats per level".to_string();
            }
            // desc/flavor hidden by level history display
            if let Ok(mut t) = desc_q.single_mut() {
                **t = String::new();
            }
            if let Ok(mut t) = flavor_q.single_mut() {
                **t = String::new();
            }
        }
        CompendiumItemId::RogueliteRun(_) => {
            hide_icon(detail_icon);
            if let Ok(mut t) = title_q.single_mut() {
                **t = "Run Details".to_string();
            }
            if let Ok(mut t) = category_q.single_mut() {
                **t = "Level-by-level breakdown".to_string();
            }
            // desc/flavor hidden by level history display
            if let Ok(mut t) = desc_q.single_mut() {
                **t = String::new();
            }
            if let Ok(mut t) = flavor_q.single_mut() {
                **t = String::new();
            }
        }
    }

    // Reset category color to default for non-unit items
    if let Ok(mut c) = cat_color_q.single_mut()
        && !matches!(state.selected_item, Some(CompendiumItemId::Unit(_)))
    {
        c.0 = DESCRIPTION_COLOR;
    }
}
