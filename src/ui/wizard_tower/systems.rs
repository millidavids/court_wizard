use bevy::input::mouse::{MouseButton, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::ActiveSave;
#[cfg(debug_assertions)]
use crate::config::save_data::grant_insight;
use crate::config::save_data::{
    add_spell_research_progress, get_insight, get_spell_research_progress,
    get_spell_talent_progress, get_spell_talent_selections, load_unified_save, spend_insight,
};
use crate::game::constants::boss_name_for_level;
use crate::game::crt_effect::{ChannelChangeMessage, CorrectedCursorPosition};
use crate::game::input::messages::MouseClicked;
use crate::game::messages::SpellResearchedMessage;
use crate::game::resources::{BattleInsightData, CurrentLevel, KillStats, TimeTravelState};
use crate::game::units::DamageType;
use crate::game::units::wizard::components::Spell;
use crate::state::{AppState, MetaGameState};
use crate::ui::main_menu::settings::components::SliderAdjusted;
use crate::ui::systems::{scale_font_by_text_width, spawn_button};

use super::components::*;
use super::constants::*;
use super::graph::build_spell_graph;
use super::materials::{RadialProgressData, RadialProgressMaterial, StarSkyData, StarSkyMaterial};

// ===========================================================================
// Shared helpers
// ===========================================================================

/// Calculates font size for talent card names based on the longest word.
fn calculate_talent_font_size(name: &str) -> f32 {
    let max_word_width = name.split_whitespace().map(|w| w.len()).max().unwrap_or(0) as f32;
    scale_font_by_text_width(max_word_width, 7.0, 13.0, 0.65, TALENT_CARD_FONT)
}

/// Returns the number of spells the player has fully researched.
fn count_researched_spells() -> u32 {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();

    Spell::researchable()
        .iter()
        .filter(|spell| {
            let name = format!("{:?}", spell);
            unlocked.contains(&name)
        })
        .count() as u32
}

/// Returns true if a spell's prerequisite is met.
fn is_prereq_met(spell: Spell) -> bool {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();

    if let Some(prereq) = spell.prerequisite() {
        let prereq_name = format!("{:?}", prereq);
        if !unlocked.contains(&prereq_name) {
            return false;
        }
    }

    let required = spell.required_total_spells();
    if required > 0 {
        let researched = count_researched_spells();
        if researched < required {
            return false;
        }
    }

    true
}

/// Returns true if this spell is fully researched (unlocked).
fn is_spell_unlocked(spell: Spell) -> bool {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();
    let name = format!("{:?}", spell);
    unlocked.contains(&name)
}

/// Returns the color associated with a damage type for UI display.
fn element_color(damage_type: DamageType) -> Color {
    match damage_type {
        DamageType::Fire => FIRE_COLOR,
        DamageType::Nature => NATURE_COLOR,
        DamageType::Electric => ELECTRIC_COLOR,
        DamageType::Necrotic => NECROTIC_COLOR,
        DamageType::Force => FORCE_COLOR,
        DamageType::Frost => FROST_COLOR,
        DamageType::Poison => POISON_COLOR,
        DamageType::Poop => POOP_COLOR,
    }
}

/// Converts a graph-space position to screen-space given the current view state.
fn graph_to_screen(graph_pos: Vec2, view: &GraphViewState, container_center: Vec2) -> Vec2 {
    graph_pos * view.scale + view.offset + container_center
}

/// Computes progress and allocation fractions for the unified slider.
/// Returns `(progress_frac, alloc_frac, handle_pos)` where handle_pos = progress_frac + alloc_frac.
fn compute_slider_fracs(progress: u32, alloc: u32, cost: u32) -> (f32, f32, f32) {
    let progress_frac = if cost > 0 {
        (progress as f32 / cost as f32).min(1.0)
    } else {
        0.0
    };
    let alloc_frac = if cost > 0 {
        (alloc as f32 / cost as f32).min(1.0 - progress_frac)
    } else {
        0.0
    };
    (progress_frac, alloc_frac, progress_frac + alloc_frac)
}

// ===========================================================================
// Hub (MetaGameState::WizardTower) systems
// ===========================================================================

/// Sets up the wizard tower main hub screen.
pub(super) fn setup_wizard_tower_main(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    config: Res<crate::config::GameConfig>,
) {
    let insight_balance = get_insight();
    commands.insert_resource(SelectedTimeTravelLevel::default());

    let show_time_travel = config.highest_level_achieved > 1;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(60.0),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
            OnMainScreen,
        ))
        .with_children(|root| {
            // Left column - main hub content
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            })
            .with_children(|left| {
                // Title
                left.spawn((
                    Text::new("Wizard's Tower"),
                    TextFont::from_font_size(TITLE_FONT_SIZE),
                    TextColor(TITLE_COLOR),
                ));

                // Level display
                left.spawn((
                    Text::new(format!("Level {}", current_level.0)),
                    TextFont::from_font_size(LEVEL_FONT_SIZE),
                    TextColor(TEXT_COLOR),
                    LevelDisplay,
                ));

                // Insight balance
                left.spawn((
                    Text::new(format!("Arcane Insight: {}", insight_balance)),
                    TextFont::from_font_size(INSIGHT_FONT_SIZE),
                    TextColor(INSIGHT_COLOR),
                    InsightDisplay,
                ));

                // Buttons
                left.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|buttons| {
                    spawn_button(
                        buttons,
                        "Start Next Battle",
                        WizardTowerButtonAction::StartNextBattle,
                        &BUTTON_STYLE,
                    );

                    spawn_button(
                        buttons,
                        "Study Spells",
                        WizardTowerButtonAction::StudySpells,
                        &BUTTON_STYLE,
                    );

                    spawn_button(
                        buttons,
                        "Compendium",
                        WizardTowerButtonAction::Compendium,
                        &BUTTON_STYLE,
                    );

                    spawn_button(
                        buttons,
                        "Return to Menu",
                        WizardTowerButtonAction::ReturnToMenu,
                        &BUTTON_STYLE,
                    );

                    // Debug level controls
                    #[cfg(debug_assertions)]
                    {
                        buttons
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                margin: UiRect::top(Val::Px(10.0)),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_button(
                                    row,
                                    "Level -1",
                                    WizardTowerButtonAction::DebugLevelDown,
                                    &DEBUG_BUTTON_STYLE,
                                );
                                spawn_button(
                                    row,
                                    "Level +1",
                                    WizardTowerButtonAction::DebugLevelUp,
                                    &DEBUG_BUTTON_STYLE,
                                );
                            });
                    }
                });
            });

            // Right column - Time Travel (only if player has beaten at least level 1)
            if show_time_travel {
                root.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|right| {
                    // Title
                    right.spawn((
                        Text::new("Time Travel"),
                        TextFont::from_font_size(20.0),
                        TextColor(TIME_TRAVEL_BOSS_COLOR),
                    ));

                    // Level list container with border
                    right
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
                                WizardTowerButtonAction::StartTimeTravel,
                                &START_TIME_TRAVEL_BUTTON_STYLE,
                            );
                        });
                });
            }
        });
}

/// Handles button actions on the hub screen.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_main_button_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardTowerButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_wt_state: ResMut<NextState<MetaGameState>>,
    mut kill_stats: ResMut<KillStats>,
    mut current_level: ResMut<CurrentLevel>,
    mut active_save: ResMut<ActiveSave>,
    selected_tt_level: Option<Res<SelectedTimeTravelLevel>>,
    #[cfg(debug_assertions)] mut config: ResMut<crate::config::GameConfig>,
    #[cfg(debug_assertions)] mut level_texts: Query<&mut Text, With<LevelDisplay>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                WizardTowerButtonAction::StudySpells => {
                    next_wt_state.set(MetaGameState::Study);
                }
                WizardTowerButtonAction::Compendium => {
                    next_wt_state.set(MetaGameState::Compendium);
                }
                WizardTowerButtonAction::StartNextBattle => {
                    channel_change.write(ChannelChangeMessage);
                    kill_stats.reset();
                    next_app_state.set(AppState::Loading);
                }
                WizardTowerButtonAction::ReturnToMenu => {
                    channel_change.write(ChannelChangeMessage);
                    kill_stats.reset();
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
                WizardTowerButtonAction::StartTimeTravel => {
                    if let Some(ref sel) = selected_tt_level
                        && let Some(level) = sel.0
                    {
                        commands.insert_resource(TimeTravelState {
                            real_level: current_level.0,
                        });
                        current_level.0 = level;
                        channel_change.write(ChannelChangeMessage);
                        kill_stats.reset();
                        next_app_state.set(AppState::Loading);
                    }
                }
                #[cfg(debug_assertions)]
                WizardTowerButtonAction::DebugLevelUp => {
                    current_level.0 += 1;
                    config.current_level = current_level.0;
                    for mut text in &mut level_texts {
                        text.0 = format!("Level {}", current_level.0);
                    }
                }
                #[cfg(debug_assertions)]
                WizardTowerButtonAction::DebugLevelDown => {
                    if current_level.0 > 1 {
                        current_level.0 -= 1;
                        config.current_level = current_level.0;
                        for mut text in &mut level_texts {
                            text.0 = format!("Level {}", current_level.0);
                        }
                    }
                }
            }
        }
    }
}

/// Handles clicks on time travel level buttons.
pub(super) fn handle_time_travel_level_clicks(
    mut button_clicked: MessageReader<MouseClicked>,
    level_buttons: Query<&TimeTravelLevelButton>,
    mut selected: ResMut<SelectedTimeTravelLevel>,
    mut level_button_nodes: Query<(&TimeTravelLevelButton, &mut BackgroundColor, &Children)>,
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

            // Collect child updates needed (to avoid borrow conflicts)
            let updates: Vec<(Entity, Color)> = level_button_nodes
                .iter()
                .flat_map(|(lb, _, children)| {
                    let is_selected = lb.0 == selected_level;
                    let color = if is_selected {
                        TIME_TRAVEL_SELECTED_TEXT
                    } else if boss_name_for_level(lb.0).is_some() {
                        TIME_TRAVEL_BOSS_COLOR
                    } else {
                        TEXT_COLOR
                    };
                    children.iter().map(move |child| (child, color))
                })
                .collect();

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
pub(super) fn handle_time_travel_level_hover(
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

// ===========================================================================
// Study (MetaGameState::Study) systems
// ===========================================================================

/// Sets up the study screen with the spell graph.
pub(super) fn setup_study_screen(
    mut commands: Commands,
    battle_insight: Res<BattleInsightData>,
    asset_server: Res<AssetServer>,
    mut progress_materials: ResMut<Assets<RadialProgressMaterial>>,
    mut star_sky_materials: ResMut<Assets<StarSkyMaterial>>,
) {
    commands.insert_resource(InsightAllocation::default());
    commands.insert_resource(GraphViewState::default());
    commands.insert_resource(GraphDragState::default());
    commands.insert_resource(SelectedStudySpell::default());

    spawn_study_screen(
        &mut commands,
        &battle_insight,
        &asset_server,
        &mut progress_materials,
        &mut star_sky_materials,
    );
}

/// Cleans up study screen-specific resources when exiting the state.
pub(super) fn cleanup_study_resources(mut commands: Commands) {
    commands.remove_resource::<InsightAllocation>();
    commands.remove_resource::<GraphViewState>();
    commands.remove_resource::<GraphDragState>();
    commands.remove_resource::<SelectedStudySpell>();
    commands.remove_resource::<GraphViewAnimation>();
    commands.remove_resource::<GraphBounds>();
}

/// Handles Commit and Back button actions on the study screen.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_study_button_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&StudyButtonAction>,
    mut next_wt_state: ResMut<NextState<MetaGameState>>,
    allocation: Option<Res<InsightAllocation>>,
    battle_insight: Res<BattleInsightData>,
    mut spell_researched: MessageWriter<SpellResearchedMessage>,
    screen_query: Query<Entity, With<OnStudyScreen>>,
    asset_server: Res<AssetServer>,
    mut selected: Option<ResMut<SelectedStudySpell>>,
    mut progress_materials: ResMut<Assets<RadialProgressMaterial>>,
    mut star_sky_materials: ResMut<Assets<StarSkyMaterial>>,
) {
    for event in button_clicked.read() {
        let Ok(action) = button_query.get(event.button) else {
            continue;
        };

        match action {
            StudyButtonAction::Back => {
                next_wt_state.set(MetaGameState::WizardTower);
            }
            StudyButtonAction::Commit => {
                let Some(alloc) = &allocation else {
                    continue;
                };

                let total = alloc.total_allocated();
                if total == 0 {
                    continue;
                }

                if !spend_insight(total) {
                    continue;
                }

                let affinities = &battle_insight.damage_types_used;
                let mut newly_unlocked = Vec::new();

                for (spell, &amount) in &alloc.allocations {
                    if amount == 0 {
                        continue;
                    }

                    let has_affinity = affinities.contains(&spell.damage_type());
                    let actual_progress = if has_affinity { amount * 2 } else { amount };

                    let unlocked = add_spell_research_progress(*spell, actual_progress);
                    if unlocked {
                        newly_unlocked.push(*spell);
                    }
                }

                for spell in newly_unlocked {
                    spell_researched.write(SpellResearchedMessage { spell });
                }

                rebuild_study_ui(
                    &mut commands,
                    &screen_query,
                    &mut selected,
                    &battle_insight,
                    &asset_server,
                    &mut progress_materials,
                    &mut star_sky_materials,
                    true,
                );
            }
            #[cfg(debug_assertions)]
            StudyButtonAction::DebugGrantInsight => {
                grant_insight(10000);
                rebuild_study_ui(
                    &mut commands,
                    &screen_query,
                    &mut selected,
                    &battle_insight,
                    &asset_server,
                    &mut progress_materials,
                    &mut star_sky_materials,
                    false,
                );
            }
        }
    }
}

/// Tears down and rebuilds the study screen UI. Optionally animates back to default view.
#[allow(clippy::too_many_arguments)]
fn rebuild_study_ui(
    commands: &mut Commands,
    screen_query: &Query<Entity, With<OnStudyScreen>>,
    selected: &mut Option<ResMut<SelectedStudySpell>>,
    battle_insight: &BattleInsightData,
    asset_server: &AssetServer,
    progress_materials: &mut Assets<RadialProgressMaterial>,
    star_sky_materials: &mut Assets<StarSkyMaterial>,
    animate_to_default: bool,
) {
    for entity in screen_query {
        commands.entity(entity).try_despawn();
    }
    commands.remove_resource::<InsightAllocation>();
    commands.insert_resource(InsightAllocation::default());
    if let Some(sel) = selected {
        sel.0 = None;
    }
    if animate_to_default {
        commands.insert_resource(GraphViewAnimation {
            target_offset: Vec2::ZERO,
            target_scale: 1.0,
            speed: GRAPH_ANIMATION_SPEED,
        });
    }
    spawn_study_screen(
        commands,
        battle_insight,
        asset_server,
        progress_materials,
        star_sky_materials,
    );
}

/// Spawns the study screen graph UI.
fn spawn_study_screen(
    commands: &mut Commands,
    battle_insight: &BattleInsightData,
    asset_server: &AssetServer,
    progress_materials: &mut Assets<RadialProgressMaterial>,
    star_sky_materials: &mut Assets<StarSkyMaterial>,
) {
    let insight_balance = get_insight();
    let (node_defs, edge_defs) = build_spell_graph();
    let affinities = &battle_insight.damage_types_used;

    // Compute graph bounds from node positions for pan clamping.
    let mut bounds_min = Vec2::ZERO;
    let mut bounds_max = Vec2::ZERO;
    for node_def in &node_defs {
        bounds_min = bounds_min.min(node_def.position);
        bounds_max = bounds_max.max(node_def.position);
    }
    commands.insert_resource(GraphBounds {
        min: bounds_min,
        max: bounds_max,
    });

    // Root container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
            OnStudyScreen,
        ))
        .with_children(|root| {
            // -- Graph Area (full size, clipped) with starry sky background --
            let sky_mat = star_sky_materials.add(StarSkyMaterial {
                data: StarSkyData {
                    base_color: GRAPH_AREA_BG.to_linear(),
                    time: 0.0,
                    zoom: 1.0,
                    pan_offset: Vec2::ZERO,
                },
            });
            root.spawn((
                MaterialNode(sky_mat),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                SpellGraphArea,
            ))
            .with_children(|graph_area| {
                // Edges as rotated line segments between waypoints
                for edge_def in &edge_defs {
                    let to_unlocked = is_spell_unlocked(edge_def.to_spell);
                    let to_prereq_met = is_prereq_met(edge_def.to_spell);
                    let edge_color = if to_unlocked || to_prereq_met {
                        GRAPH_EDGE_COLOR
                    } else {
                        GRAPH_EDGE_LOCKED_COLOR
                    };

                    // One entity per consecutive waypoint pair
                    for pair in edge_def.waypoints.windows(2) {
                        graph_area.spawn((
                            Node {
                                width: Val::Px(0.0),
                                height: Val::Px(GRAPH_EDGE_THICKNESS),
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                ..default()
                            },
                            BackgroundColor(edge_color),
                            UiTransform::default(),
                            SpellGraphEdge {
                                start: pair[0],
                                end: pair[1],
                            },
                        ));
                    }
                }

                // Central "Free" anchor node
                graph_area
                    .spawn((
                        Node {
                            width: Val::Px(GRAPH_FREE_NODE_SIZE),
                            height: Val::Px(GRAPH_FREE_NODE_SIZE),
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(GRAPH_NODE_BG),
                        BorderColor::all(GRAPH_NODE_FREE_BORDER),
                        BorderRadius::all(Val::Percent(50.0)),
                        ZIndex(1),
                        FreeNode,
                    ))
                    .with_children(|node| {
                        node.spawn((
                            ImageNode::new(asset_server.load("images/logo.png")),
                            Node {
                                width: Val::Percent(80.0),
                                height: Val::Percent(80.0),
                                ..default()
                            },
                        ));
                    });

                // Spell nodes
                for node_def in &node_defs {
                    let Some(spell) = node_def.spell else {
                        continue; // Skip central anchor (already spawned)
                    };

                    let unlocked = is_spell_unlocked(spell);
                    let prereq_met = is_prereq_met(spell);
                    let has_affinity = affinities.contains(&spell.damage_type());
                    let progress = get_spell_research_progress(spell);
                    let cost = spell.research_cost();
                    let is_free = cost == 0; // MagicMissile, Telekinesis

                    let (bg, border) = if is_free || unlocked {
                        (GRAPH_NODE_BG, GRAPH_NODE_COMPLETED_BORDER)
                    } else if has_affinity && prereq_met {
                        (GRAPH_NODE_BG, AFFINITY_COLOR)
                    } else if prereq_met {
                        (GRAPH_NODE_BG, GRAPH_NODE_BORDER)
                    } else {
                        (GRAPH_NODE_LOCKED_BG, GRAPH_NODE_LOCKED_BORDER)
                    };

                    graph_area
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(GRAPH_NODE_SIZE),
                                height: Val::Px(GRAPH_NODE_SIZE),
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(bg),
                            BorderColor::all(border),
                            BorderRadius::all(Val::Percent(50.0)),
                            ZIndex(1),
                            SpellGraphNode {
                                spell,
                                graph_position: node_def.position,
                            },
                        ))
                        .with_children(|node| {
                            // Radial progress ring (behind icon, covers full node)
                            if cost > 0 && !unlocked && prereq_met && progress > 0 {
                                let fill_frac = (progress as f32 / cost as f32).min(1.0);
                                let mat = progress_materials.add(RadialProgressMaterial {
                                    data: RadialProgressData {
                                        fill_color: PROGRESS_BAR_FILL.to_linear(),
                                        bg_color: LinearRgba::new(0.15, 0.15, 0.15, 0.6),
                                        progress: fill_frac,
                                        ring_width: 0.14,
                                    },
                                });
                                node.spawn((
                                    MaterialNode(mat),
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(0.0),
                                        top: Val::Px(0.0),
                                        ..default()
                                    },
                                ));
                            }

                            if prereq_met || unlocked || is_free {
                                if let Some(icon_path) = spell.icon_path() {
                                    node.spawn((
                                        ImageNode::new(asset_server.load(icon_path)),
                                        Node {
                                            width: Val::Percent(55.0),
                                            height: Val::Percent(55.0),
                                            ..default()
                                        },
                                    ));
                                }
                            } else {
                                node.spawn((
                                    Text::new("???"),
                                    TextFont::from_font_size(14.0),
                                    TextColor(LOCKED_TEXT_COLOR),
                                ));
                            }
                        });
                }
            });

            // -- UI Overlay (above graph, full-screen flex column) --
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|overlay| {
                // -- Header --
                overlay
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_shrink: 0.0,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.85)),
                    ))
                    .with_children(|header| {
                        header.spawn((
                            Text::new("Study Spells"),
                            TextFont::from_font_size(TITLE_FONT_SIZE),
                            TextColor(TITLE_COLOR),
                        ));

                        header.spawn((
                            Text::new(format!("Arcane Insight: {}", insight_balance)),
                            TextFont::from_font_size(INSIGHT_FONT_SIZE),
                            TextColor(INSIGHT_COLOR),
                            StudyInsightDisplay,
                        ));

                        header.spawn((
                            Text::new("Pending: 0"),
                            TextFont::from_font_size(INSIGHT_FONT_SIZE),
                            TextColor(PENDING_COLOR),
                            PendingInsightDisplay,
                        ));
                    });

                // -- Middle area (fills space between header & footer) --
                overlay
                    .spawn((
                        Node {
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            min_height: Val::Px(0.0),
                            overflow: Overflow::clip_y(),
                            padding: UiRect::new(
                                Val::Px(12.0),
                                Val::Px(0.0),
                                Val::Px(8.0),
                                Val::Px(8.0),
                            ),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|middle| {
                        // -- Detail Panel (hidden by default) --
                        middle.spawn((
                            Node {
                                width: Val::Px(DETAIL_PANEL_WIDTH),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(DETAIL_PANEL_PADDING)),
                                row_gap: Val::Px(8.0),
                                border: UiRect::all(Val::Px(2.0)),
                                overflow: Overflow::clip_y(),
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(DETAIL_PANEL_BG),
                            BorderColor::all(DETAIL_PANEL_BORDER),
                            BorderRadius::all(Val::Px(8.0)),
                            Interaction::None,
                            Pickable {
                                should_block_lower: true,
                                is_hoverable: true,
                            },
                            StudyDetailPanel,
                        ));
                    });

                // -- Footer --
                overlay
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_shrink: 0.0,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.85)),
                    ))
                    .with_children(|footer| {
                        spawn_button(
                            footer,
                            "Commit",
                            StudyButtonAction::Commit,
                            &COMMIT_BUTTON_STYLE,
                        );

                        spawn_button(footer, "Back", StudyButtonAction::Back, &BACK_BUTTON_STYLE);

                        #[cfg(debug_assertions)]
                        spawn_button(
                            footer,
                            "+10000 Insight",
                            StudyButtonAction::DebugGrantInsight,
                            &DEBUG_BUTTON_STYLE,
                        );
                    });
            });
        });
}

// ===========================================================================
// Graph node selection
// ===========================================================================

/// Detects clicks on spell graph nodes and updates the selected spell.
pub(super) fn handle_graph_node_clicks(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    node_query: Query<&SpellGraphNode>,
    mut selected: ResMut<SelectedStudySpell>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    panel_query: Query<(&Interaction, &Node), With<StudyDetailPanel>>,
) {
    // Block node clicks when cursor is interacting with the detail panel
    let panel_active = panel_query.iter().any(|(interaction, node)| {
        node.display != Display::None && *interaction != Interaction::None
    });
    if panel_active {
        for _ in button_clicked.read() {}
        return;
    }

    for event in button_clicked.read() {
        if let Ok(node) = node_query.get(event.button) {
            if selected.0 == Some(node.spell) {
                selected.0 = None;
                commands.remove_resource::<GraphViewAnimation>();
            } else {
                selected.0 = Some(node.spell);

                // Animate pan+zoom so the node ends up centered in the right 2/3
                if let Ok(computed) = graph_area_query.single() {
                    let size = computed.size() * computed.inverse_scale_factor();
                    let container_center = size / 2.0;
                    let target_x = DETAIL_PANEL_WIDTH + (size.x - DETAIL_PANEL_WIDTH) / 2.0;
                    let target_y = size.y / 2.0;
                    let target = Vec2::new(target_x, target_y);
                    let target_scale = GRAPH_ZOOM_MAX;
                    let target_offset =
                        target - container_center - node.graph_position * target_scale;
                    commands.insert_resource(GraphViewAnimation {
                        target_offset,
                        target_scale,
                        speed: GRAPH_ANIMATION_SPEED,
                    });
                }
            }
        }
    }
}

// ===========================================================================
// Pan & Zoom
// ===========================================================================

/// Clamps the view offset so the outermost graph nodes can reach roughly
/// the screen center but no further.
fn clamp_view_offset(view: &mut GraphViewState, bounds: &GraphBounds) {
    let margin = GRAPH_NODE_SIZE;
    // When panning, stop when the outermost node reaches near the center.
    // offset.x range: [-max_x * scale - margin, -min_x * scale + margin]
    let min_x = -bounds.max.x * view.scale - margin;
    let max_x = -bounds.min.x * view.scale + margin;
    let min_y = -bounds.max.y * view.scale - margin;
    let max_y = -bounds.min.y * view.scale + margin;

    view.offset.x = view.offset.x.clamp(min_x, max_x);
    view.offset.y = view.offset.y.clamp(min_y, max_y);
}

/// Handles panning the graph via left-click drag on the background.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_graph_pan(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    ui_scale: Res<bevy::ui::UiScale>,
    mut view: ResMut<GraphViewState>,
    mut drag: ResMut<GraphDragState>,
    bounds: Option<Res<GraphBounds>>,
    mut selected: ResMut<SelectedStudySpell>,
    node_interactions: Query<&Interaction, With<SpellGraphNode>>,
    panel_query: Query<(&Interaction, &Node), With<StudyDetailPanel>>,
    slider_interactions: Query<
        &Interaction,
        Or<(With<StudyAllocationSlider>, With<StudyAllocationHandle>)>,
    >,
) {
    let Some(cursor_pos) = corrected_cursor.0 else {
        if drag.dragging {
            drag.dragging = false;
        }
        return;
    };
    // Convert window-logical cursor to UI space
    let cursor_ui = cursor_pos / ui_scale.0;

    // Check if cursor is over the detail panel using Bevy's Interaction state
    let cursor_over_panel = panel_query.iter().any(|(interaction, node)| {
        node.display != Display::None && *interaction != Interaction::None
    });

    if buttons.just_pressed(MouseButton::Left) {
        // Don't start dragging if a node, the detail panel, or a slider is being pressed
        let any_node_pressed = node_interactions.iter().any(|i| *i == Interaction::Pressed);
        let slider_pressed = slider_interactions.iter().any(|i| *i != Interaction::None);
        if !any_node_pressed && !cursor_over_panel && !slider_pressed {
            drag.dragging = true;
            drag.last_cursor = cursor_ui;
            drag.start_cursor = cursor_ui;
            commands.remove_resource::<GraphViewAnimation>();
        }
    }

    if buttons.just_released(MouseButton::Left) && drag.dragging {
        let total_moved = (cursor_ui - drag.start_cursor).length();
        // Deselect on a click (not a drag) on empty space, but not over the detail panel
        if total_moved < 4.0 && !cursor_over_panel {
            selected.0 = None;
        }
        drag.dragging = false;
        return;
    }

    if !buttons.pressed(MouseButton::Left) {
        drag.dragging = false;
        return;
    }

    if drag.dragging {
        let delta = cursor_ui - drag.last_cursor;
        view.offset += delta;
        if let Some(bounds) = &bounds {
            clamp_view_offset(&mut view, bounds);
        }
        drag.last_cursor = cursor_ui;
    }
}

/// Handles zooming the graph via mouse scroll wheel.
pub(super) fn handle_graph_zoom(
    mut commands: Commands,
    mut mouse_wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    ui_scale: Res<bevy::ui::UiScale>,
    mut view: ResMut<GraphViewState>,
    bounds: Option<Res<GraphBounds>>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = corrected_cursor.0 else {
        return;
    };
    // Convert window-logical cursor to UI space (ComputedNode sizes are in UI space)
    let cursor_ui = cursor_pos / ui_scale.0;

    let container_center = if let Ok(computed) = graph_area_query.single() {
        computed.size() * computed.inverse_scale_factor() / 2.0
    } else {
        Vec2::new(window.width() / 2.0, window.height() / 2.0) / ui_scale.0
    };

    for event in mouse_wheel.read() {
        let scroll_delta = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 100.0,
        };

        let old_scale = view.scale;
        let new_scale = (old_scale * (1.0 + scroll_delta * GRAPH_ZOOM_SPEED))
            .clamp(GRAPH_ZOOM_MIN, GRAPH_ZOOM_MAX);

        if (new_scale - old_scale).abs() > f32::EPSILON {
            // Cancel any running animation
            commands.remove_resource::<GraphViewAnimation>();
            // Adjust offset to keep point under cursor stationary
            let cursor_from_center = cursor_ui - container_center;
            let graph_point = (cursor_from_center - view.offset) / old_scale;
            view.offset = cursor_from_center - graph_point * new_scale;
            view.scale = new_scale;
            if let Some(bounds) = &bounds {
                clamp_view_offset(&mut view, bounds);
            }
        }
    }
}

/// Smoothly animates the graph view toward a target offset and zoom.
/// Removed automatically when the animation reaches its destination.
pub(super) fn animate_graph_view(
    mut commands: Commands,
    time: Res<Time>,
    animation: Option<Res<GraphViewAnimation>>,
    bounds: Option<Res<GraphBounds>>,
    mut view: ResMut<GraphViewState>,
) {
    let Some(anim) = animation else {
        return;
    };

    let t = (anim.speed * time.delta_secs()).min(1.0);
    view.offset = view.offset.lerp(anim.target_offset, t);
    view.scale = view.scale + (anim.target_scale - view.scale) * t;

    if let Some(bounds) = &bounds {
        clamp_view_offset(&mut view, bounds);
    }

    // Stop when close enough
    let offset_dist = (view.offset - anim.target_offset).length();
    let scale_dist = (view.scale - anim.target_scale).abs();
    if offset_dist < 0.5 && scale_dist < 0.001 {
        view.offset = anim.target_offset;
        view.scale = anim.target_scale;
        commands.remove_resource::<GraphViewAnimation>();
    }
}

// ===========================================================================
// Position update systems
// ===========================================================================

/// Updates the screen position of all graph nodes based on pan/zoom state.
pub(super) fn update_graph_node_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut node_query: Query<(&mut Node, &SpellGraphNode), Without<FreeNode>>,
    mut free_node_query: Query<&mut Node, (With<FreeNode>, Without<SpellGraphNode>)>,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_center = computed.size() * computed.inverse_scale_factor() / 2.0;

    for (mut node, graph_node) in &mut node_query {
        let screen_pos = graph_to_screen(graph_node.graph_position, &view, container_center);
        let scaled_size = GRAPH_NODE_SIZE * view.scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }

    for mut node in &mut free_node_query {
        let screen_pos = graph_to_screen(Vec2::ZERO, &view, container_center);
        let scaled_size = GRAPH_FREE_NODE_SIZE * view.scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }
}

/// Updates border colors on spell nodes based on the selected spell.
/// Only runs when selection changes, avoiding per-frame save data loads.
pub(super) fn update_graph_node_borders(
    selected: Res<SelectedStudySpell>,
    mut border_query: Query<(&mut BorderColor, &SpellGraphNode)>,
) {
    for (mut border_color, graph_node) in &mut border_query {
        if selected.0 == Some(graph_node.spell) {
            *border_color = BorderColor::all(GRAPH_NODE_SELECTED_BORDER);
        } else {
            let spell = graph_node.spell;
            let unlocked = is_spell_unlocked(spell);
            let prereq_met = is_prereq_met(spell);
            let cost = spell.research_cost();
            let is_free = cost == 0;

            let border = if is_free || unlocked {
                GRAPH_NODE_COMPLETED_BORDER
            } else if prereq_met {
                GRAPH_NODE_BORDER
            } else {
                GRAPH_NODE_LOCKED_BORDER
            };
            *border_color = BorderColor::all(border);
        }
    }
}

/// Updates graph edge segment positions based on pan/zoom state.
/// Each segment is a rotated rectangle connecting two consecutive waypoints.
pub(super) fn update_graph_edge_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut segments: Query<(&mut Node, &mut UiTransform, &SpellGraphEdge)>,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_center = computed.size() * computed.inverse_scale_factor() / 2.0;
    let thickness = (GRAPH_EDGE_THICKNESS * view.scale).max(1.0);

    for (mut node, mut ui_transform, edge) in &mut segments {
        let screen_a = graph_to_screen(edge.start, &view, container_center);
        let screen_b = graph_to_screen(edge.end, &view, container_center);

        let delta = screen_b - screen_a;
        let length = delta.length();
        let angle = delta.y.atan2(delta.x);

        // Position at the midpoint of the segment
        let midpoint = (screen_a + screen_b) / 2.0;
        node.left = Val::Px(midpoint.x - length / 2.0);
        node.top = Val::Px(midpoint.y - thickness / 2.0);
        node.width = Val::Px(length);
        node.height = Val::Px(thickness);

        ui_transform.rotation = Rot2::radians(angle);
    }
}

// ===========================================================================
// Detail panel
// ===========================================================================

/// Updates the detail panel based on the currently selected spell.
pub(super) fn update_study_detail_panel(
    mut commands: Commands,
    selected: Res<SelectedStudySpell>,
    battle_insight: Res<BattleInsightData>,
    allocation: Option<Res<InsightAllocation>>,
    mut panel_query: Query<(Entity, &mut Node), With<StudyDetailPanel>>,
    asset_server: Res<AssetServer>,
) {
    if !selected.is_changed() {
        return;
    }

    let Ok((panel_entity, mut panel_node)) = panel_query.single_mut() else {
        return;
    };

    // Clear existing children
    commands.entity(panel_entity).despawn_related::<Children>();

    let Some(spell) = selected.0 else {
        panel_node.display = Display::None;
        return;
    };

    panel_node.display = Display::Flex;

    let unlocked = is_spell_unlocked(spell);
    let prereq_met = is_prereq_met(spell);
    let cost = spell.research_cost();
    let progress = get_spell_research_progress(spell);
    let is_free = cost == 0;
    let affinities = &battle_insight.damage_types_used;
    let has_affinity = affinities.contains(&spell.damage_type());

    commands.entity(panel_entity).with_children(|panel| {
        // Spell icon + name row
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|row| {
                if let Some(icon_path) = spell.icon_path()
                    && (unlocked || prereq_met || is_free)
                {
                    row.spawn((
                        ImageNode::new(asset_server.load(icon_path)),
                        Node {
                            width: Val::Px(SPELL_ICON_SIZE),
                            height: Val::Px(SPELL_ICON_SIZE),
                            ..default()
                        },
                    ));
                }
                row.spawn((
                    Text::new(if !prereq_met && !unlocked && !is_free {
                        "???"
                    } else {
                        spell.display_name()
                    }),
                    TextFont::from_font_size(DETAIL_TITLE_FONT_SIZE),
                    TextColor(if unlocked || is_free {
                        COMPLETED_COLOR
                    } else if prereq_met {
                        TEXT_COLOR
                    } else {
                        LOCKED_TEXT_COLOR
                    }),
                ));
            });

        // Element type
        if unlocked || prereq_met || is_free {
            let element_text = if has_affinity && prereq_met && !unlocked && !is_free {
                format!("{} (2x Affinity)", spell.damage_type().display_name())
            } else {
                spell.damage_type().display_name().to_string()
            };
            panel.spawn((
                Text::new(element_text),
                TextFont::from_font_size(DETAIL_TEXT_FONT_SIZE),
                TextColor(if has_affinity && !unlocked && !is_free {
                    AFFINITY_COLOR
                } else {
                    element_color(spell.damage_type())
                }),
            ));
        }

        // Description
        if unlocked || prereq_met || is_free {
            let desc = spell.description();
            let font_size = if desc.len() > DESC_SHRINK_THRESHOLD {
                DETAIL_SMALL_FONT_SIZE - 2.0
            } else {
                DETAIL_SMALL_FONT_SIZE
            };
            panel.spawn((
                Text::new(desc),
                TextFont::from_font_size(font_size),
                TextColor(TEXT_COLOR),
                Node {
                    max_width: Val::Px(DETAIL_PANEL_WIDTH - DETAIL_PANEL_PADDING * 2.0),
                    ..default()
                },
            ));
        } else {
            let desc = spell.locked_description();
            let font_size = if desc.len() > DESC_SHRINK_THRESHOLD {
                DETAIL_SMALL_FONT_SIZE - 2.0
            } else {
                DETAIL_SMALL_FONT_SIZE
            };
            panel.spawn((
                Text::new(desc),
                TextFont::from_font_size(font_size),
                TextColor(LOCKED_TEXT_COLOR),
            ));
        }

        // Research status
        if is_free {
            panel.spawn((
                Text::new("Default Spell"),
                TextFont::from_font_size(DETAIL_TEXT_FONT_SIZE),
                TextColor(COMPLETED_COLOR),
            ));
            spawn_talent_section(panel, spell);
        } else if unlocked {
            panel.spawn((
                Text::new("Researched"),
                TextFont::from_font_size(DETAIL_TEXT_FONT_SIZE),
                TextColor(COMPLETED_COLOR),
            ));
            spawn_talent_section(panel, spell);
        } else if prereq_met {
            // Unified progress + allocation slider
            let current_alloc = allocation.as_ref().map(|a| a.get(&spell)).unwrap_or(0);
            spawn_detail_unified_slider(panel, spell, progress, cost, current_alloc);

            // Allocation text
            let effective = if has_affinity {
                current_alloc * 2
            } else {
                current_alloc
            };
            let alloc_text = if current_alloc > 0 {
                format!("{}+{}/{}", progress, effective, cost)
            } else {
                format!("{}/{}", progress, cost)
            };
            panel.spawn((
                Text::new(alloc_text),
                TextFont::from_font_size(DETAIL_TEXT_FONT_SIZE),
                TextColor(TEXT_COLOR),
                StudyAllocationText { spell },
            ));
        } else {
            // Locked — show requirements
            panel.spawn(Node {
                height: Val::Px(6.0),
                ..default()
            });

            if let Some(prereq) = spell.prerequisite() {
                let prereq_done = is_spell_unlocked(prereq);
                panel.spawn((
                    Text::new(format!(
                        "Requires: {}{}",
                        prereq.display_name(),
                        if prereq_done { " ✓" } else { "" }
                    )),
                    TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
                    TextColor(if prereq_done {
                        COMPLETED_COLOR
                    } else {
                        LOCKED_TEXT_COLOR
                    }),
                ));
            }

            let required = spell.required_total_spells();
            if required > 0 {
                let researched = count_researched_spells();
                panel.spawn((
                    Text::new(format!(
                        "Spells researched: {}/{}{}",
                        researched,
                        required,
                        if researched >= required { " ✓" } else { "" }
                    )),
                    TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
                    TextColor(if researched >= required {
                        COMPLETED_COLOR
                    } else {
                        LOCKED_TEXT_COLOR
                    }),
                ));
            }
        }
    });
}

/// Spawns the talent section below the research status for unlocked spells.
fn spawn_talent_section(parent: &mut ChildSpawnerCommands, spell: Spell) {
    use crate::game::units::wizard::talents::{constants as talent_consts, definitions};

    let talent_progress = get_spell_talent_progress(spell);
    let thresholds = talent_consts::tier_thresholds(spell);
    let metric_label = talent_consts::progress_metric_label(spell);
    let defs = definitions::talent_definitions(spell);
    let selections = get_spell_talent_selections(spell);

    // Separator
    parent.spawn(Node {
        height: Val::Px(6.0),
        ..default()
    });

    // "Talents" header with progress count
    let max_threshold = thresholds[2];
    parent.spawn((
        Text::new(format!(
            "-- Talents -- ({}/{})",
            talent_progress.min(max_threshold),
            max_threshold
        )),
        TextFont::from_font_size(TALENT_TIER_LABEL_FONT),
        TextColor(Color::srgba(0.6, 0.6, 0.7, 0.8)),
    ));

    // Progress metric label
    parent.spawn((
        Text::new(metric_label),
        TextFont::from_font_size(TALENT_CARD_FONT),
        TextColor(Color::srgba(0.5, 0.5, 0.6, 0.6)),
    ));

    // Main talent layout: progress bar on left, tier cards on right
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(6.0),
            margin: UiRect::top(Val::Px(4.0)),
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|row| {
            // Vertical progress bar (stretches to match tier column height)
            row.spawn((
                Node {
                    width: Val::Px(TALENT_BAR_WIDTH),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(TALENT_BAR_BG),
                BorderRadius::all(Val::Px(3.0)),
            ))
            .with_children(|bar| {
                // Fill from top
                let fill_frac = if max_threshold > 0 {
                    (talent_progress as f32 / max_threshold as f32).min(1.0)
                } else {
                    0.0
                };
                bar.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(fill_frac * 100.0),
                        ..default()
                    },
                    BackgroundColor(TALENT_BAR_FILL),
                    BorderRadius::all(Val::Px(3.0)),
                    TalentProgressBarFill { spell },
                ));
            });

            // Tier cards column
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            })
            .with_children(|tiers_col| {
                for tier in 0..3u8 {
                    let tier_unlocked = talent_progress >= thresholds[tier as usize];
                    let tier_defs = &defs[tier as usize];
                    let current_selection = selections[tier as usize];

                    // Tier label
                    let tier_label = if tier_unlocked {
                        format!("Tier {}", tier + 1)
                    } else {
                        format!(
                            "Tier {} ({}/{})",
                            tier + 1,
                            talent_progress.min(thresholds[tier as usize]),
                            thresholds[tier as usize]
                        )
                    };
                    tiers_col.spawn((
                        Text::new(tier_label),
                        TextFont::from_font_size(TALENT_TIER_LABEL_FONT),
                        TextColor(if tier_unlocked {
                            TEXT_COLOR
                        } else {
                            LOCKED_TEXT_COLOR
                        }),
                    ));

                    // Three cards in a row
                    tiers_col
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(TALENT_CARD_GAP),
                            ..default()
                        })
                        .with_children(|card_row| {
                            for choice in 0..3u8 {
                                let def = &tier_defs[choice as usize];
                                let is_selected = current_selection == Some(choice);

                                let bg_color = if tier_unlocked {
                                    TALENT_UNLOCKED_BG
                                } else {
                                    TALENT_LOCKED_BG
                                };
                                let border_color = if is_selected {
                                    TALENT_SELECTED_BORDER
                                } else if tier_unlocked {
                                    TALENT_UNLOCKED_BORDER
                                } else {
                                    TALENT_LOCKED_BORDER
                                };

                                let display_name = if tier_unlocked { def.name } else { "???" };

                                card_row
                                    .spawn((
                                        Node {
                                            width: Val::Px(TALENT_CARD_WIDTH),
                                            height: Val::Px(TALENT_CARD_HEIGHT),
                                            border: UiRect::all(Val::Px(if is_selected {
                                                2.0
                                            } else {
                                                1.0
                                            })),
                                            padding: UiRect::all(Val::Px(3.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        BackgroundColor(bg_color),
                                        BorderColor::all(border_color),
                                        BorderRadius::all(Val::Px(4.0)),
                                        Interaction::default(),
                                        TalentCard {
                                            spell,
                                            tier,
                                            choice,
                                        },
                                    ))
                                    .with_children(|card| {
                                        card.spawn((
                                            Text::new(display_name),
                                            TextFont::from_font_size(calculate_talent_font_size(
                                                display_name,
                                            )),
                                            TextColor(if tier_unlocked {
                                                TEXT_COLOR
                                            } else {
                                                LOCKED_TEXT_COLOR
                                            }),
                                            TextLayout::new_with_justify(Justify::Center),
                                        ));
                                    });
                            }
                        });
                }
            });
        });

    // Description area for hovered/selected talent
    parent.spawn((
        Text::new(""),
        TextFont::from_font_size(TALENT_DESC_FONT),
        TextColor(TEXT_COLOR),
        Node {
            max_width: Val::Px(DETAIL_PANEL_WIDTH - DETAIL_PANEL_PADDING * 2.0),
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        },
        TalentDescriptionText,
    ));
}

/// Spawns a progress bar in the detail panel.
/// Spawns a unified progress + allocation slider in the detail panel.
/// Committed progress is shown as a non-reducible filled region on the left.
/// The slider handle controls the pending allocation region that starts after the
/// committed progress.
fn spawn_detail_unified_slider(
    parent: &mut ChildSpawnerCommands,
    spell: Spell,
    progress: u32,
    cost: u32,
    current_alloc: u32,
) {
    let (progress_frac, alloc_frac, handle_pos) =
        compute_slider_fracs(progress, current_alloc, cost);

    parent
        .spawn((
            Node {
                width: Val::Px(SLIDER_TRACK_WIDTH),
                height: Val::Px(SLIDER_TRACK_HEIGHT),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                position_type: PositionType::Relative,
                ..default()
            },
            BorderColor::all(SLIDER_TRACK_BORDER),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(SLIDER_TRACK_BG),
            Interaction::default(),
            RelativeCursorPosition::default(),
            StudyAllocationSlider { spell },
        ))
        .with_children(|track| {
            // Committed progress fill (non-draggable floor)
            track.spawn((
                Node {
                    width: Val::Percent(progress_frac * 100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BorderRadius {
                    top_left: Val::Px(8.0),
                    bottom_left: Val::Px(8.0),
                    top_right: Val::Px(0.0),
                    bottom_right: Val::Px(0.0),
                },
                BackgroundColor(PROGRESS_BAR_FILL),
                StudyProgressFill,
            ));

            // Pending allocation fill (on top of progress, extends right)
            track.spawn((
                Node {
                    width: Val::Percent(alloc_frac * 100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(progress_frac * 100.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(SLIDER_FILL_COLOR),
                StudyAllocationFill { spell },
            ));

            // Handle
            track.spawn((
                Node {
                    width: Val::Px(SLIDER_HANDLE_WIDTH),
                    height: Val::Px(SLIDER_HANDLE_HEIGHT),
                    position_type: PositionType::Absolute,
                    left: Val::Px(handle_pos * SLIDER_TRACK_WIDTH - SLIDER_HANDLE_WIDTH / 2.0),
                    top: Val::Px(-(SLIDER_HANDLE_HEIGHT - SLIDER_TRACK_HEIGHT) / 2.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(SLIDER_HANDLE_COLOR),
                Interaction::default(),
                RelativeCursorPosition::default(),
                StudyAllocationHandle {
                    spell,
                    is_dragging: false,
                },
            ));
        });
}

// ===========================================================================
// Detail panel slider interaction
// ===========================================================================

/// Handles click and drag on the detail panel allocation slider.
pub(super) fn handle_detail_slider_interaction(
    buttons: Res<ButtonInput<MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut StudyAllocationHandle)>,
    slider_tracks: Query<(
        &Interaction,
        &RelativeCursorPosition,
        &StudyAllocationSlider,
    )>,
    mut allocation: ResMut<InsightAllocation>,
    mut slider_adjusted: MessageWriter<SliderAdjusted>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        for (interaction, cursor_pos, track) in &slider_tracks {
            if !matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
                continue;
            }
            if cursor_pos.normalized.is_none() {
                continue;
            }

            for (_hi, mut handle) in &mut slider_handles {
                if handle.spell == track.spell {
                    handle.is_dragging = true;
                }
            }
        }

        for (interaction, mut handle) in &mut slider_handles {
            if *interaction == Interaction::Pressed {
                handle.is_dragging = true;
            }
        }
    }

    if !buttons.pressed(MouseButton::Left) {
        for (_interaction, mut handle) in &mut slider_handles {
            handle.is_dragging = false;
        }
        return;
    }

    let mut dragging_spell: Option<Spell> = None;
    for (_interaction, handle) in &slider_handles {
        if handle.is_dragging {
            dragging_spell = Some(handle.spell);
            break;
        }
    }

    let Some(spell) = dragging_spell else {
        return;
    };

    let insight_balance = get_insight();

    for (_interaction, cursor_pos, track) in &slider_tracks {
        if track.spell != spell {
            continue;
        }

        let Some(pos) = cursor_pos.normalized else {
            continue;
        };

        let cost = spell.research_cost();
        let progress = get_spell_research_progress(spell);
        let remaining = cost.saturating_sub(progress);

        if remaining == 0 {
            continue;
        }

        // Cursor position maps to fraction of total cost bar
        let cursor_frac = (pos.x + 0.5).clamp(0.0, 1.0);
        // Floor: committed progress fraction (can't go below this)
        let progress_frac = if cost > 0 {
            progress as f32 / cost as f32
        } else {
            0.0
        };
        // Allocation = how far above the floor the cursor is, in cost units
        let alloc_frac = (cursor_frac - progress_frac).max(0.0);
        let desired = (alloc_frac * cost as f32).round() as u32;

        let others: u32 = allocation
            .allocations
            .iter()
            .filter(|(s, _)| **s != spell)
            .map(|(_, v)| *v)
            .sum();
        let max_for_spell = insight_balance.saturating_sub(others).min(remaining);
        let clamped = desired.min(max_for_spell);

        let old = allocation.get(&spell);
        if clamped != old {
            allocation.set(spell, clamped);
            slider_adjusted.write(SliderAdjusted);
        }
    }
}

/// Updates slider fill widths and handle positions in the unified progress+allocation bar.
pub(super) fn update_detail_sliders(
    allocation: Res<InsightAllocation>,
    mut alloc_fills: Query<
        (&mut Node, &StudyAllocationFill),
        (Without<StudyAllocationHandle>, Without<StudyProgressFill>),
    >,
    mut slider_handles: Query<
        (&mut Node, &StudyAllocationHandle),
        (Without<StudyAllocationFill>, Without<StudyProgressFill>),
    >,
) {
    if !allocation.is_changed() {
        return;
    }

    for (mut node, fill) in &mut alloc_fills {
        let spell = fill.spell;
        let progress = get_spell_research_progress(spell);
        let alloc = allocation.get(&spell);
        let (progress_frac, alloc_frac, _) =
            compute_slider_fracs(progress, alloc, spell.research_cost());

        node.left = Val::Percent(progress_frac * 100.0);
        node.width = Val::Percent(alloc_frac * 100.0);
    }

    for (mut node, handle) in &mut slider_handles {
        let spell = handle.spell;
        let progress = get_spell_research_progress(spell);
        let alloc = allocation.get(&spell);
        let (_, _, handle_pos) = compute_slider_fracs(progress, alloc, spell.research_cost());

        node.left = Val::Px(handle_pos * SLIDER_TRACK_WIDTH - SLIDER_HANDLE_WIDTH / 2.0);
    }
}

/// Updates "current+pending / total" text for the detail panel allocation.
pub(super) fn update_allocation_text(
    allocation: Res<InsightAllocation>,
    battle_insight: Res<BattleInsightData>,
    mut texts: Query<(&mut Text, &StudyAllocationText)>,
) {
    if !allocation.is_changed() {
        return;
    }

    let affinities = &battle_insight.damage_types_used;

    for (mut text, alloc_text) in &mut texts {
        let spell = alloc_text.spell;
        let cost = spell.research_cost();
        let progress = get_spell_research_progress(spell);
        let alloc = allocation.get(&spell);

        let has_affinity = affinities.contains(&spell.damage_type());
        let effective = if has_affinity { alloc * 2 } else { alloc };

        if alloc > 0 {
            text.0 = format!("{}+{}/{}", progress, effective, cost);
        } else {
            text.0 = format!("{}/{}", progress, cost);
        }
    }
}

/// Updates the "Pending: X" display in the study header.
pub(super) fn update_pending_insight_display(
    allocation: Res<InsightAllocation>,
    mut texts: Query<&mut Text, With<PendingInsightDisplay>>,
) {
    if !allocation.is_changed() {
        return;
    }

    let total = allocation.total_allocated();
    for mut text in &mut texts {
        text.0 = format!("Pending: {}", total);
    }
}

// ===========================================================================
// Top-level cleanup (exiting WizardTower entirely)
// ===========================================================================

/// Cleans up wizard tower-specific resources when leaving AppState::MetaGame.
pub(super) fn cleanup_wizard_tower_resources(mut commands: Commands) {
    commands.remove_resource::<InsightAllocation>();
    commands.remove_resource::<GraphViewState>();
    commands.remove_resource::<GraphDragState>();
    commands.remove_resource::<SelectedStudySpell>();
    commands.remove_resource::<GraphViewAnimation>();
    commands.remove_resource::<GraphBounds>();
    commands.remove_resource::<SelectedTimeTravelLevel>();
}

// ===========================================================================
// Star sky time update
// ===========================================================================

/// Updates the time and parallax uniforms on all StarSkyMaterial instances each frame.
pub(super) fn update_star_sky_time(
    time: Res<Time>,
    view: Option<Res<GraphViewState>>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut materials: ResMut<Assets<StarSkyMaterial>>,
) {
    let elapsed = time.elapsed_secs();

    // Compute normalized pan offset for parallax.
    let (pan_normalized, zoom) = if let Some(view) = &view {
        let container_size = graph_area_query
            .single()
            .map(|c| c.size() * c.inverse_scale_factor())
            .unwrap_or(Vec2::ONE);
        // Normalize offset to roughly 0..1 range relative to container size.
        let pan = view.offset / container_size.max(Vec2::ONE);
        (pan, view.scale)
    } else {
        (Vec2::ZERO, 1.0)
    };

    for (_id, mat) in materials.iter_mut() {
        mat.data.time = elapsed;
        mat.data.pan_offset = pan_normalized;
        mat.data.zoom = zoom;
    }
}

// ===========================================================================
// Talent interaction systems
// ===========================================================================

/// Handles clicks on talent cards to select/deselect talents.
pub(super) fn handle_talent_card_clicks(
    mut selected: ResMut<SelectedStudySpell>,
    interaction_query: Query<(&Interaction, &TalentCard), Changed<Interaction>>,
) {
    use crate::config::save_data::{get_spell_talent_progress, set_spell_talent_selection};
    use crate::game::units::wizard::talents::constants as talent_consts;

    for (interaction, card) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let talent_progress = get_spell_talent_progress(card.spell);
        let thresholds = talent_consts::tier_thresholds(card.spell);

        // Check if tier is unlocked
        if talent_progress < thresholds[card.tier as usize] {
            continue;
        }

        // Toggle selection: if already selected, deselect; otherwise select
        let current = crate::config::save_data::get_spell_talent_selections(card.spell);
        let new_choice = if current[card.tier as usize] == Some(card.choice) {
            None
        } else {
            Some(card.choice)
        };

        set_spell_talent_selection(card.spell, card.tier as usize, new_choice);

        // Force detail panel refresh by re-triggering SelectedStudySpell change
        let spell = selected.0;
        selected.0 = None;
        selected.0 = spell;
    }
}

/// Updates the talent description text when hovering over talent cards.
pub(super) fn update_talent_hover_description(
    interaction_query: Query<(&Interaction, &TalentCard), Changed<Interaction>>,
    mut desc_query: Query<(&mut Text, &mut TextFont, &mut TextColor), With<TalentDescriptionText>>,
) {
    use crate::config::save_data::get_spell_talent_progress;
    use crate::game::units::wizard::talents::{constants as talent_consts, definitions};

    for (interaction, card) in &interaction_query {
        if *interaction != Interaction::Hovered && *interaction != Interaction::Pressed {
            continue;
        }

        let talent_progress = get_spell_talent_progress(card.spell);
        let thresholds = talent_consts::tier_thresholds(card.spell);
        let tier_unlocked = talent_progress >= thresholds[card.tier as usize];
        let defs = definitions::talent_definitions(card.spell);
        let def = &defs[card.tier as usize][card.choice as usize];

        for (mut text, mut font, mut color) in &mut desc_query {
            if tier_unlocked {
                let desc = format!("{}: {}", def.name, def.description);
                let font_size = if desc.len() > DESC_SHRINK_THRESHOLD {
                    TALENT_DESC_FONT_SMALL
                } else {
                    TALENT_DESC_FONT
                };
                *text = Text::new(desc);
                *font = TextFont::from_font_size(font_size);
                *color = TextColor(TEXT_COLOR);
            } else {
                let font_size = if def.locked_text.len() > DESC_SHRINK_THRESHOLD {
                    TALENT_DESC_FONT_SMALL
                } else {
                    TALENT_DESC_FONT
                };
                *text = Text::new(def.locked_text);
                *font = TextFont::from_font_size(font_size);
                *color = TextColor(LOCKED_TEXT_COLOR);
            }
        }
    }
}

/// Clears the talent description text when not hovering any talent card.
pub(super) fn clear_talent_hover_description(
    interaction_query: Query<&Interaction, (With<TalentCard>, Changed<Interaction>)>,
    mut desc_query: Query<&mut Text, With<TalentDescriptionText>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::None {
            // Only clear if no other card is hovered
            let any_hovered = interaction_query
                .iter()
                .any(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);
            if !any_hovered {
                for mut text in &mut desc_query {
                    *text = Text::new("");
                }
            }
        }
    }
}
