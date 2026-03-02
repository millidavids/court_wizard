use bevy::input::mouse::{MouseButton, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::ActiveSave;
use crate::config::save_data::{
    add_spell_research_progress, get_insight, get_spell_research_progress, load_unified_save,
    spend_insight,
};
#[cfg(debug_assertions)]
use crate::config::save_data::grant_insight;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::game::messages::SpellResearchedMessage;
use crate::game::resources::{BattleInsightData, CurrentLevel, KillStats};
use crate::game::units::DamageType;
use crate::game::units::wizard::components::Spell;
use crate::state::{AppState, MetaGameState};
use crate::ui::main_menu::settings::components::SliderAdjusted;
use crate::ui::systems::spawn_button;

use super::components::*;
use super::constants::*;
use super::graph::build_spell_graph;

// ===========================================================================
// Shared helpers
// ===========================================================================

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
    }
}

// ===========================================================================
// Hub (MetaGameState::WizardTower) systems
// ===========================================================================

/// Sets up the wizard tower main hub screen.
pub(super) fn setup_wizard_tower_main(mut commands: Commands, current_level: Res<CurrentLevel>) {
    let insight_balance = get_insight();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
            OnMainScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Wizard's Tower"),
                TextFont::from_font_size(TITLE_FONT_SIZE),
                TextColor(TITLE_COLOR),
            ));

            // Level display
            #[cfg(debug_assertions)]
            parent.spawn((
                Text::new(format!("Level {}", current_level.0)),
                TextFont::from_font_size(LEVEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
                LevelDisplay,
            ));
            #[cfg(not(debug_assertions))]
            parent.spawn((
                Text::new(format!("Level {}", current_level.0)),
                TextFont::from_font_size(LEVEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
            ));

            // Insight balance
            parent.spawn((
                Text::new(format!("Arcane Insight: {}", insight_balance)),
                TextFont::from_font_size(INSIGHT_FONT_SIZE),
                TextColor(INSIGHT_COLOR),
                InsightDisplay,
            ));

            // Buttons
            parent
                .spawn(Node {
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
}

/// Cleans up main hub screen entities.
pub(super) fn cleanup_main_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnMainScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handles button actions on the hub screen.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_main_button_actions(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardTowerButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_wt_state: ResMut<NextState<MetaGameState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    #[cfg(debug_assertions)] mut current_level: ResMut<CurrentLevel>,
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

// ===========================================================================
// Study (MetaGameState::Study) systems
// ===========================================================================

/// Sets up the study screen with the spell graph.
pub(super) fn setup_study_screen(
    mut commands: Commands,
    battle_insight: Res<BattleInsightData>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(InsightAllocation::default());
    commands.insert_resource(GraphViewState::default());
    commands.insert_resource(GraphDragState::default());
    commands.insert_resource(SelectedStudySpell::default());

    spawn_study_screen(&mut commands, &battle_insight, &asset_server);
}

/// Cleans up study screen entities and resources.
pub(super) fn cleanup_study_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnStudyScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<InsightAllocation>();
    commands.remove_resource::<GraphViewState>();
    commands.remove_resource::<GraphDragState>();
    commands.remove_resource::<SelectedStudySpell>();
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

                // Despawn and rebuild
                for entity in &screen_query {
                    commands.entity(entity).despawn();
                }
                commands.remove_resource::<InsightAllocation>();
                commands.insert_resource(InsightAllocation::default());
                if let Some(ref mut sel) = selected {
                    sel.0 = None;
                }

                // Reset view state
                commands.insert_resource(GraphViewState::default());

                spawn_study_screen(&mut commands, &battle_insight, &asset_server);
            }
            #[cfg(debug_assertions)]
            StudyButtonAction::DebugGrantInsight => {
                grant_insight(10000);

                for entity in &screen_query {
                    commands.entity(entity).despawn();
                }
                commands.remove_resource::<InsightAllocation>();
                commands.insert_resource(InsightAllocation::default());
                if let Some(ref mut sel) = selected {
                    sel.0 = None;
                }

                spawn_study_screen(&mut commands, &battle_insight, &asset_server);
            }
        }
    }
}

/// Spawns the study screen graph UI.
fn spawn_study_screen(
    commands: &mut Commands,
    battle_insight: &BattleInsightData,
    asset_server: &AssetServer,
) {
    let insight_balance = get_insight();
    let (node_defs, edge_defs) = build_spell_graph();
    let affinities = &battle_insight.damage_types_used;

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
            // -- Graph Area (full size, clipped) --
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(GRAPH_AREA_BG),
                SpellGraphArea,
            ))
            .with_children(|graph_area| {
                // Edges as L-shaped connectors (H + V segments per edge)
                for edge_def in &edge_defs {
                    let to_unlocked = is_spell_unlocked(edge_def.to_spell);
                    let to_prereq_met = is_prereq_met(edge_def.to_spell);
                    let edge_color = if to_unlocked || to_prereq_met {
                        GRAPH_EDGE_COLOR
                    } else {
                        GRAPH_EDGE_LOCKED_COLOR
                    };

                    // Horizontal segment
                    graph_area.spawn((
                        Node {
                            height: Val::Px(GRAPH_EDGE_THICKNESS),
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            ..default()
                        },
                        BackgroundColor(edge_color),
                        SpellGraphEdge {
                            from_spell: edge_def.from_spell,
                            to_spell: edge_def.to_spell,
                        },
                        EdgeSegmentH,
                    ));

                    // Vertical segment
                    graph_area.spawn((
                        Node {
                            width: Val::Px(GRAPH_EDGE_THICKNESS),
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            ..default()
                        },
                        BackgroundColor(edge_color),
                        SpellGraphEdge {
                            from_spell: edge_def.from_spell,
                            to_spell: edge_def.to_spell,
                        },
                        EdgeSegmentV,
                    ));
                }

                // Central "Free" anchor node
                graph_area.spawn((
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
                            BorderRadius::all(Val::Px(GRAPH_NODE_SIZE / 2.0)),
                            ZIndex(1),
                            SpellGraphNode {
                                spell,
                                graph_position: node_def.position,
                            },
                        ))
                        .with_children(|node| {
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

                            // Progress indicator (small bar at bottom of node)
                            if cost > 0 && !unlocked {
                                let fill_pct = if cost > 0 {
                                    (progress as f32 / cost as f32 * 100.0).min(100.0)
                                } else {
                                    0.0
                                };
                                node.spawn((
                                    Node {
                                        width: Val::Percent(80.0),
                                        height: Val::Px(3.0),
                                        position_type: PositionType::Absolute,
                                        bottom: Val::Px(3.0),
                                        left: Val::Percent(10.0),
                                        ..default()
                                    },
                                    BackgroundColor(PROGRESS_BAR_BACKGROUND),
                                    BorderRadius::all(Val::Px(1.5)),
                                ))
                                .with_children(|bar| {
                                    bar.spawn((
                                        Node {
                                            width: Val::Percent(fill_pct),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(if fill_pct >= 100.0 {
                                            PROGRESS_BAR_FULL
                                        } else {
                                            PROGRESS_BAR_FILL
                                        }),
                                        BorderRadius::all(Val::Px(1.5)),
                                    ));
                                });
                            }
                        });
                }
            });

            // -- Header Overlay (top, absolute) --
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
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

            // -- Footer Overlay (bottom, absolute) --
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
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

            // -- Detail Panel (left side, absolute, hidden by default) --
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(60.0),
                    left: Val::Px(12.0),
                    width: Val::Px(DETAIL_PANEL_WIDTH),
                    max_height: Val::Percent(80.0),
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
                StudyDetailPanel,
            ));
        });
}

// ===========================================================================
// Graph node selection
// ===========================================================================

/// Detects clicks on spell graph nodes and updates the selected spell.
pub(super) fn handle_graph_node_clicks(
    mut button_clicked: MessageReader<MouseClicked>,
    node_query: Query<&SpellGraphNode>,
    mut selected: ResMut<SelectedStudySpell>,
) {
    for event in button_clicked.read() {
        if let Ok(node) = node_query.get(event.button) {
            if selected.0 == Some(node.spell) {
                selected.0 = None;
            } else {
                selected.0 = Some(node.spell);
            }
        }
    }
}

// ===========================================================================
// Pan & Zoom
// ===========================================================================

/// Handles panning the graph via left-click drag on the background.
pub(super) fn handle_graph_pan(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut view: ResMut<GraphViewState>,
    mut drag: ResMut<GraphDragState>,
    node_interactions: Query<&Interaction, With<SpellGraphNode>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        if drag.dragging {
            drag.dragging = false;
        }
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        // Don't start dragging if a node is being pressed
        let any_node_pressed = node_interactions
            .iter()
            .any(|i| *i == Interaction::Pressed);
        if !any_node_pressed {
            drag.dragging = true;
            drag.last_cursor = cursor_pos;
        }
    }

    if !buttons.pressed(MouseButton::Left) {
        drag.dragging = false;
        return;
    }

    if drag.dragging {
        let delta = cursor_pos - drag.last_cursor;
        view.offset += delta;
        drag.last_cursor = cursor_pos;
    }
}

/// Handles zooming the graph via mouse scroll wheel.
pub(super) fn handle_graph_zoom(
    mut mouse_wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    mut view: ResMut<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let container_center = if let Ok(computed) = graph_area_query.single() {
        computed.size() / 2.0
    } else {
        Vec2::new(window.width() / 2.0, window.height() / 2.0)
    };

    for event in mouse_wheel.read() {
        let scroll_delta = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 100.0,
        };

        let old_scale = view.scale;
        let new_scale =
            (old_scale * (1.0 + scroll_delta * GRAPH_ZOOM_SPEED)).clamp(GRAPH_ZOOM_MIN, GRAPH_ZOOM_MAX);

        if (new_scale - old_scale).abs() > f32::EPSILON {
            // Adjust offset to keep point under cursor stationary
            let cursor_from_center = cursor_pos - container_center;
            let graph_point = (cursor_from_center - view.offset) / old_scale;
            view.offset = cursor_from_center - graph_point * new_scale;
            view.scale = new_scale;
        }
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
    selected: Res<SelectedStudySpell>,
    mut border_query: Query<
        (&mut BorderColor, &SpellGraphNode),
        Without<FreeNode>,
    >,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_center = computed.size() / 2.0;
    let scale = view.scale;

    // Update spell nodes
    for (mut node, graph_node) in &mut node_query {
        let screen_pos =
            graph_node.graph_position * scale + view.offset + container_center;
        let scaled_size = GRAPH_NODE_SIZE * scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }

    // Update free node
    for mut node in &mut free_node_query {
        let screen_pos = view.offset + container_center;
        let scaled_size = GRAPH_FREE_NODE_SIZE * scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }

    // Update selected border highlight
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

/// Updates graph edge L-shaped connector positions based on pan/zoom state.
/// Each edge has a horizontal segment and a vertical segment forming an L-path.
pub(super) fn update_graph_edge_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    node_query: Query<&SpellGraphNode>,
    mut h_segments: Query<
        (&mut Node, &SpellGraphEdge),
        (With<EdgeSegmentH>, Without<EdgeSegmentV>),
    >,
    mut v_segments: Query<
        (&mut Node, &SpellGraphEdge),
        (With<EdgeSegmentV>, Without<EdgeSegmentH>),
    >,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_center = computed.size() / 2.0;
    let scale = view.scale;
    let thickness = (GRAPH_EDGE_THICKNESS * scale).max(1.0);

    // Build a map of spell -> graph_position
    let mut spell_positions: std::collections::HashMap<Option<Spell>, Vec2> =
        std::collections::HashMap::new();
    spell_positions.insert(None, Vec2::ZERO);
    for graph_node in &node_query {
        spell_positions.insert(Some(graph_node.spell), graph_node.graph_position);
    }

    // Horizontal segments: from source X to target X, at source Y
    for (mut node, edge) in &mut h_segments {
        let Some(&from_graph) = spell_positions.get(&edge.from_spell) else {
            continue;
        };
        let Some(&to_graph) = spell_positions.get(&Some(edge.to_spell)) else {
            continue;
        };

        let from_screen = from_graph * scale + view.offset + container_center;
        let to_screen = to_graph * scale + view.offset + container_center;

        let min_x = from_screen.x.min(to_screen.x);
        let max_x = from_screen.x.max(to_screen.x);
        let y = from_screen.y;

        node.left = Val::Px(min_x);
        node.top = Val::Px(y - thickness / 2.0);
        node.width = Val::Px((max_x - min_x).max(thickness));
        node.height = Val::Px(thickness);
    }

    // Vertical segments: from source Y to target Y, at target X
    for (mut node, edge) in &mut v_segments {
        let Some(&from_graph) = spell_positions.get(&edge.from_spell) else {
            continue;
        };
        let Some(&to_graph) = spell_positions.get(&Some(edge.to_spell)) else {
            continue;
        };

        let from_screen = from_graph * scale + view.offset + container_center;
        let to_screen = to_graph * scale + view.offset + container_center;

        let x = to_screen.x;
        let min_y = from_screen.y.min(to_screen.y);
        let max_y = from_screen.y.max(to_screen.y);

        node.left = Val::Px(x - thickness / 2.0);
        node.top = Val::Px(min_y);
        node.width = Val::Px(thickness);
        node.height = Val::Px((max_y - min_y).max(thickness));
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
            panel.spawn((
                Text::new(spell.description()),
                TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    max_width: Val::Px(DETAIL_PANEL_WIDTH - DETAIL_PANEL_PADDING * 2.0),
                    ..default()
                },
            ));
        } else {
            panel.spawn((
                Text::new(spell.locked_description()),
                TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
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
        } else if unlocked {
            panel.spawn((
                Text::new("Researched"),
                TextFont::from_font_size(DETAIL_TEXT_FONT_SIZE),
                TextColor(COMPLETED_COLOR),
            ));
        } else if prereq_met {
            // Progress bar
            let fill_pct = if cost > 0 {
                (progress as f32 / cost as f32 * 100.0).min(100.0)
            } else {
                100.0
            };
            spawn_detail_progress_bar(panel, fill_pct);

            // Allocation slider
            let current_alloc = allocation
                .as_ref()
                .map(|a| a.get(&spell))
                .unwrap_or(0);
            let remaining = cost.saturating_sub(progress);
            spawn_detail_allocation_slider(panel, spell, current_alloc, remaining);

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
                height: Val::Px(4.0),
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

/// Spawns a progress bar in the detail panel.
fn spawn_detail_progress_bar(parent: &mut ChildSpawnerCommands, fill_pct: f32) {
    let fill_color = if fill_pct >= 100.0 {
        PROGRESS_BAR_FULL
    } else {
        PROGRESS_BAR_FILL
    };

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(PROGRESS_BAR_HEIGHT),
                ..default()
            },
            BackgroundColor(PROGRESS_BAR_BACKGROUND),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|bg| {
            bg.spawn((
                Node {
                    width: Val::Percent(fill_pct),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(fill_color),
                BorderRadius::all(Val::Px(4.0)),
            ));
        });
}

/// Spawns an allocation slider in the detail panel.
fn spawn_detail_allocation_slider(
    parent: &mut ChildSpawnerCommands,
    spell: Spell,
    current_alloc: u32,
    max_alloc: u32,
) {
    let normalized = if max_alloc > 0 {
        current_alloc as f32 / max_alloc as f32
    } else {
        0.0
    };

    parent
        .spawn((
            Node {
                width: Val::Px(SLIDER_TRACK_WIDTH),
                height: Val::Px(SLIDER_TRACK_HEIGHT),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                position_type: PositionType::Relative,
                ..default()
            },
            BorderColor::all(SLIDER_TRACK_BORDER),
            BorderRadius::all(Val::Px(6.0)),
            BackgroundColor(SLIDER_TRACK_BG),
            Interaction::default(),
            RelativeCursorPosition::default(),
            StudyAllocationSlider { spell },
        ))
        .with_children(|track| {
            // Fill
            track.spawn((
                Node {
                    width: Val::Percent(normalized * 100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BorderRadius {
                    top_left: Val::Px(6.0),
                    bottom_left: Val::Px(6.0),
                    top_right: Val::Px(0.0),
                    bottom_right: Val::Px(0.0),
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
                    left: Val::Px(normalized * SLIDER_TRACK_WIDTH - SLIDER_HANDLE_WIDTH / 2.0),
                    top: Val::Px(-(SLIDER_HANDLE_HEIGHT - SLIDER_TRACK_HEIGHT) / 2.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(3.0)),
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
    let insight_balance = get_insight();

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

        let normalized = (pos.x + 0.5).clamp(0.0, 1.0);
        let desired = (normalized * remaining as f32).round() as u32;

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

/// Updates slider fill widths and handle positions in the detail panel.
pub(super) fn update_detail_sliders(
    allocation: Res<InsightAllocation>,
    mut slider_fills: Query<(&mut Node, &StudyAllocationFill), Without<StudyAllocationHandle>>,
    mut slider_handles: Query<(&mut Node, &StudyAllocationHandle), Without<StudyAllocationFill>>,
) {
    if !allocation.is_changed() {
        return;
    }

    for (mut node, fill) in &mut slider_fills {
        let spell = fill.spell;
        let cost = spell.research_cost();
        let progress = get_spell_research_progress(spell);
        let remaining = cost.saturating_sub(progress);
        let alloc = allocation.get(&spell);

        let normalized = if remaining > 0 {
            alloc as f32 / remaining as f32
        } else {
            0.0
        };

        node.width = Val::Percent(normalized * 100.0);
    }

    for (mut node, handle) in &mut slider_handles {
        let spell = handle.spell;
        let cost = spell.research_cost();
        let progress = get_spell_research_progress(spell);
        let remaining = cost.saturating_sub(progress);
        let alloc = allocation.get(&spell);

        let normalized = if remaining > 0 {
            alloc as f32 / remaining as f32
        } else {
            0.0
        };

        node.left = Val::Px(normalized * SLIDER_TRACK_WIDTH - SLIDER_HANDLE_WIDTH / 2.0);
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

/// Cleans up all wizard tower screen entities when leaving AppState::MetaGame.
pub(super) fn cleanup_wizard_tower_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnWizardTowerScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<InsightAllocation>();
    commands.remove_resource::<GraphViewState>();
    commands.remove_resource::<GraphDragState>();
    commands.remove_resource::<SelectedStudySpell>();
}
