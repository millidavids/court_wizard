//! Study tab panel setup, graph rendering, and node systems.

use super::interaction::*;
use bevy::input::mouse::{MouseButton, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

#[cfg(debug_assertions)]
use crate::config::save_data::grant_insight;
use crate::config::save_data::{
    add_spell_research_progress, get_insight, get_spell_research_progress, load_unified_save,
    set_insight_bonus_levels, spend_insight,
};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseClicked;
use crate::game::messages::{InsightBonusUpgradedMessage, SpellResearchedMessage};
use crate::game::resources::BattleInsightData;
use crate::game::units::DamageType;
use crate::game::units::wizard::components::Spell;
use crate::ui::systems::{scale_font_by_text_width, spawn_button};

use super::super::components::*;
use super::super::constants::*;
use crate::game::insight_bonuses::InsightBonusStat;

use super::super::graph::{build_insight_constellation, build_spell_graph};
use super::super::materials::{
    ConcentricRingsData, ConcentricRingsMaterial, RadialProgressData, RadialProgressMaterial,
    StarSkyData, StarSkyMaterial,
};

// ===========================================================================
// Shared helpers
// ===========================================================================

/// Calculates font size for talent card names based on the longest word.
pub(super) fn calculate_talent_font_size(name: &str) -> f32 {
    let max_word_width = name.split_whitespace().map(|w| w.len()).max().unwrap_or(0) as f32;
    scale_font_by_text_width(max_word_width, 7.0, 13.0, 0.65, TALENT_CARD_FONT)
}

/// Returns the number of spells the player has fully researched.
pub(super) fn count_researched_spells() -> u32 {
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
pub(super) fn is_prereq_met(spell: Spell) -> bool {
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
pub(crate) fn is_spell_unlocked(spell: Spell) -> bool {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();
    let name = format!("{:?}", spell);
    unlocked.contains(&name)
}

/// Returns the color associated with a damage type for UI display.
pub(super) fn element_color(damage_type: DamageType) -> Color {
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
pub(super) fn graph_to_screen(
    graph_pos: Vec2,
    view: &GraphViewState,
    container_center: Vec2,
) -> Vec2 {
    graph_pos * view.scale + view.offset + container_center
}

/// Clips a line segment to an axis-aligned rectangle using the Liang-Barsky algorithm.
/// Returns the clipped endpoints, or `None` if the segment is entirely outside.
pub(super) fn clip_line_to_rect(
    a: Vec2,
    b: Vec2,
    rect: &std::ops::RangeInclusive<Vec2>,
) -> Option<(Vec2, Vec2)> {
    let min = *rect.start();
    let max = *rect.end();
    let d = b - a;

    let mut t0: f32 = 0.0;
    let mut t1: f32 = 1.0;

    let edges = [
        (-d.x, a.x - min.x), // left
        (d.x, max.x - a.x),  // right
        (-d.y, a.y - min.y), // top
        (d.y, max.y - a.y),  // bottom
    ];

    for (p, q) in edges {
        if p.abs() < 1e-10 {
            // Parallel to edge — outside if q < 0
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
            if t0 > t1 {
                return None;
            }
        }
    }

    Some((a + d * t0, a + d * t1))
}

/// Computes progress and allocation fractions for the unified slider.
/// Returns `(progress_frac, alloc_frac, handle_pos)` where handle_pos = progress_frac + alloc_frac.
pub(super) fn compute_slider_fracs(progress: u32, alloc: u32, cost: u32) -> (f32, f32, f32) {
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
// Study (MetaGameState::Study) systems
// ===========================================================================

/// Builds the study tab content into the wizard tower's left and right panels.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_study_panels(
    commands: &mut Commands,
    right_panel_entity: Entity,
    left_panel_entity: Entity,
    battle_insight: &BattleInsightData,
    asset_server: &AssetServer,
    progress_materials: &mut Assets<RadialProgressMaterial>,
    ring_materials: &mut Assets<ConcentricRingsMaterial>,
    star_sky_materials: &mut Assets<StarSkyMaterial>,
) {
    commands.insert_resource(InsightAllocation::default());
    // Zoom out and offset to show both spell web and insight constellation.
    commands.insert_resource(GraphViewState {
        offset: default_graph_offset(),
        scale: GRAPH_DEFAULT_SCALE,
    });
    commands.insert_resource(GraphDragState::default());
    commands.insert_resource(SelectedStudySpell::default());
    commands.insert_resource(SelectedInsightBonus::default());

    spawn_study_panels(
        commands,
        right_panel_entity,
        left_panel_entity,
        battle_insight,
        asset_server,
        progress_materials,
        ring_materials,
        star_sky_materials,
    );
}

/// Cleans up study screen-specific resources when exiting the state.
pub(crate) fn cleanup_study_resources(mut commands: Commands) {
    commands.remove_resource::<InsightAllocation>();
    commands.remove_resource::<GraphViewState>();
    commands.remove_resource::<GraphDragState>();
    commands.remove_resource::<SelectedStudySpell>();
    commands.remove_resource::<SelectedInsightBonus>();
    commands.remove_resource::<GraphViewAnimation>();
    commands.remove_resource::<GraphBounds>();
}

/// Handles Commit and Back button actions on the study screen.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_study_button_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&StudyButtonAction>,
    allocation: Option<Res<InsightAllocation>>,
    battle_insight: Res<BattleInsightData>,
    mut spell_researched: MessageWriter<SpellResearchedMessage>,
    mut insight_upgraded: MessageWriter<InsightBonusUpgradedMessage>,
    left_panel: Query<Entity, With<super::super::layout::WizardTowerLeftPanel>>,
    right_panel: Query<Entity, With<super::super::layout::WizardTowerRightPanel>>,
    asset_server: Res<AssetServer>,
    mut selected: Option<ResMut<SelectedStudySpell>>,
    mut progress_materials: ResMut<Assets<RadialProgressMaterial>>,
    mut ring_materials: ResMut<Assets<ConcentricRingsMaterial>>,
    mut star_sky_materials: ResMut<Assets<StarSkyMaterial>>,
) {
    for event in button_clicked.read() {
        let Ok(action) = button_query.get(event.button) else {
            continue;
        };

        match action {
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

                // Apply insight bonus allocations (insight already spent above)
                let cost_per = InsightBonusStat::cost_per_level();
                let max_level = InsightBonusStat::max_level();
                let mut bonus_updates: Vec<(&str, u8)> = Vec::new();
                for (stat, &amount) in &alloc.bonus_allocations {
                    if amount == 0 {
                        continue;
                    }
                    let current = stat.current_level();
                    let levels_earned = (amount / cost_per) as u8;
                    let new_level = (current + levels_earned).min(max_level);
                    if new_level > current {
                        bonus_updates.push((stat.id(), new_level));
                    }
                }
                set_insight_bonus_levels(&bonus_updates);
                if !bonus_updates.is_empty() {
                    insight_upgraded.write(InsightBonusUpgradedMessage);
                }

                for spell in newly_unlocked {
                    spell_researched.write(SpellResearchedMessage { spell });
                }

                rebuild_study_ui(
                    &mut commands,
                    &left_panel,
                    &right_panel,
                    &mut selected,
                    &battle_insight,
                    &asset_server,
                    &mut progress_materials,
                    &mut ring_materials,
                    &mut star_sky_materials,
                    true,
                );
            }
            #[cfg(debug_assertions)]
            StudyButtonAction::DebugGrantInsight => {
                grant_insight(10000);
                rebuild_study_ui(
                    &mut commands,
                    &left_panel,
                    &right_panel,
                    &mut selected,
                    &battle_insight,
                    &asset_server,
                    &mut progress_materials,
                    &mut ring_materials,
                    &mut star_sky_materials,
                    false,
                );
            }
        }
    }
}

/// Tears down and rebuilds the study screen UI. Optionally animates back to default view.
#[allow(clippy::too_many_arguments)]
pub(super) fn rebuild_study_ui(
    commands: &mut Commands,
    left_panel: &Query<Entity, With<super::super::layout::WizardTowerLeftPanel>>,
    right_panel: &Query<Entity, With<super::super::layout::WizardTowerRightPanel>>,
    selected: &mut Option<ResMut<SelectedStudySpell>>,
    battle_insight: &BattleInsightData,
    asset_server: &AssetServer,
    progress_materials: &mut Assets<RadialProgressMaterial>,
    ring_materials: &mut Assets<ConcentricRingsMaterial>,
    star_sky_materials: &mut Assets<StarSkyMaterial>,
    animate_to_default: bool,
) {
    let Ok(left_entity) = left_panel.single() else {
        return;
    };
    let Ok(right_entity) = right_panel.single() else {
        return;
    };
    commands.entity(left_entity).despawn_related::<Children>();
    commands.entity(right_entity).despawn_related::<Children>();
    commands.remove_resource::<InsightAllocation>();
    commands.insert_resource(InsightAllocation::default());
    commands.insert_resource(SelectedInsightBonus::default());
    if let Some(sel) = selected {
        sel.0 = None;
    }
    if animate_to_default {
        animate_to_default_view(commands);
    }
    spawn_study_panels(
        commands,
        right_entity,
        left_entity,
        battle_insight,
        asset_server,
        progress_materials,
        ring_materials,
        star_sky_materials,
    );
}

/// Spawns study tab content into the wizard tower's left and right panels.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_study_panels(
    commands: &mut Commands,
    right_panel_entity: Entity,
    left_panel_entity: Entity,
    battle_insight: &BattleInsightData,
    asset_server: &AssetServer,
    progress_materials: &mut Assets<RadialProgressMaterial>,
    ring_materials: &mut Assets<ConcentricRingsMaterial>,
    star_sky_materials: &mut Assets<StarSkyMaterial>,
) {
    let insight_balance = get_insight();
    let (node_defs, edge_defs) = build_spell_graph();
    let (insight_nodes, insight_edges, insight_anchor_pos) = build_insight_constellation();
    let affinities = &battle_insight.damage_types_used;

    // Compute graph bounds from node positions for pan clamping.
    let mut bounds_min = Vec2::ZERO;
    let mut bounds_max = Vec2::ZERO;
    for node_def in &node_defs {
        bounds_min = bounds_min.min(node_def.position);
        bounds_max = bounds_max.max(node_def.position);
    }
    // Include insight constellation in bounds
    bounds_min = bounds_min.min(insight_anchor_pos);
    bounds_max = bounds_max.max(insight_anchor_pos);
    for inode in &insight_nodes {
        bounds_min = bounds_min.min(inode.position);
        bounds_max = bounds_max.max(inode.position);
    }
    commands.insert_resource(GraphBounds {
        min: bounds_min,
        max: bounds_max,
    });

    // -- Right panel: Graph Area with starry sky background --
    let sky_mat = star_sky_materials.add(StarSkyMaterial {
        data: StarSkyData {
            base_color: GRAPH_AREA_BG.to_linear(),
            time: 0.0,
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
        },
    });
    commands.entity(right_panel_entity).with_children(|right| {
        right
            .spawn((
                MaterialNode(sky_mat),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                Interaction::None,
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
                            border_radius: BorderRadius::all(Val::Percent(50.0)),
                            ..default()
                        },
                        BackgroundColor(GRAPH_NODE_BG),
                        BorderColor::all(GRAPH_NODE_FREE_BORDER),
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
                                border_radius: BorderRadius::all(Val::Percent(50.0)),
                                ..default()
                            },
                            BackgroundColor(bg),
                            BorderColor::all(border),
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
                                    GraphNodeLabel { base_size: 14.0 },
                                ));
                            }
                        });
                }

                // -------------------------------------------------------
                // Insight Bonus Constellation
                // -------------------------------------------------------

                // Constellation edges
                for edge_def in &insight_edges {
                    graph_area.spawn((
                        Node {
                            width: Val::Px(0.0),
                            height: Val::Px(GRAPH_EDGE_THICKNESS),
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            ..default()
                        },
                        BackgroundColor(INSIGHT_EDGE_COLOR),
                        UiTransform::default(),
                        InsightConstellationEdge {
                            start: edge_def.start,
                            end: edge_def.end,
                        },
                    ));
                }

                // Constellation central anchor
                graph_area.spawn((
                    Node {
                        width: Val::Px(INSIGHT_ANCHOR_SIZE),
                        height: Val::Px(INSIGHT_ANCHOR_SIZE),
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(INSIGHT_NODE_BG),
                    BorderColor::all(INSIGHT_ANCHOR_BORDER),
                    ZIndex(1),
                    InsightConstellationAnchor,
                ));

                // Constellation stat nodes
                for inode_def in &insight_nodes {
                    let stat = inode_def.stat;
                    let level = stat.current_level();
                    let maxed = level >= InsightBonusStat::max_level();
                    let border = if maxed {
                        INSIGHT_NODE_MAXED_BORDER
                    } else {
                        INSIGHT_NODE_BORDER
                    };

                    graph_area
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(INSIGHT_NODE_SIZE),
                                height: Val::Px(INSIGHT_NODE_SIZE),
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                overflow: Overflow::clip(),
                                border_radius: BorderRadius::all(Val::Percent(50.0)),
                                ..default()
                            },
                            BackgroundColor(INSIGHT_NODE_BG),
                            BorderColor::all(border),
                            ZIndex(1),
                            InsightBonusNode {
                                stat,
                                graph_position: inode_def.position,
                            },
                        ))
                        .with_children(|node| {
                            // Concentric rings showing level progress
                            let mat = ring_materials.add(ConcentricRingsMaterial {
                                data: ConcentricRingsData {
                                    fill_color: INSIGHT_PROGRESS_FILL.to_linear(),
                                    bg_color: LinearRgba::new(0.2, 0.15, 0.3, 0.5),
                                    pending_color: SLIDER_FILL_COLOR.to_linear(),
                                    filled: level as f32,
                                    pending: 0.0,
                                    total: InsightBonusStat::max_level() as f32,
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
                                InsightBonusRings { stat },
                            ));

                            // Stat label -- constrained to node bounds
                            let label = match stat {
                                InsightBonusStat::SpellDamage => "DMG",
                                InsightBonusStat::SpellRange => "RNG",
                                InsightBonusStat::CastSpeed => "SPD",
                                InsightBonusStat::ManaCost => "MP",
                            };
                            node.spawn((
                                Text::new(label),
                                TextFont::from_font_size(11.0),
                                TextColor(if maxed {
                                    INSIGHT_NODE_MAXED_BORDER
                                } else {
                                    Color::srgb(0.8, 0.75, 0.95)
                                }),
                                GraphNodeLabel { base_size: 11.0 },
                            ));
                        });
                }
            });
    });

    // -- Overlay HUD elements on top of the graph area --
    // These are absolute-positioned inside the SpellGraphArea so they
    // float over the starry sky.  Text is click-through; buttons capture.
    // Find the graph area entity we just spawned (last child of right panel).
    // We build them as additional children of the graph area.
    // (We can't easily get the entity from the closure above, so we add
    //  another with_children call on right_panel_entity — the graph area
    //  is its only child, and these will also be children of right_panel_entity
    //  but positioned absolutely over the graph.)
    commands.entity(right_panel_entity).with_children(|right| {
        // Top-left: Arcane Insight (with shadow)
        right
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(8.0),
                    left: Val::Px(12.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|container| {
                let insight_text = format!("Arcane Insight: {}", insight_balance);
                let offset = INSIGHT_FONT_SIZE / 20.0;
                // Shadow
                container.spawn((
                    Text::new(insight_text.clone()),
                    TextFont::from_font_size(INSIGHT_FONT_SIZE),
                    TextColor(crate::ui::constants::TEXT_SHADOW_COLOR),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(offset),
                        top: Val::Px(offset),
                        ..default()
                    },
                ));
                // Foreground
                container.spawn((
                    Text::new(insight_text),
                    TextFont::from_font_size(INSIGHT_FONT_SIZE),
                    TextColor(INSIGHT_COLOR),
                    StudyInsightDisplay,
                ));
            });

        // Top-right: Pending (with shadow)
        right
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(8.0),
                    right: Val::Px(12.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|container| {
                let offset = INSIGHT_FONT_SIZE / 20.0;
                // Shadow
                container.spawn((
                    Text::new("Pending: 0"),
                    TextFont::from_font_size(INSIGHT_FONT_SIZE),
                    TextColor(crate::ui::constants::TEXT_SHADOW_COLOR),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(offset),
                        top: Val::Px(offset),
                        ..default()
                    },
                ));
                // Foreground
                container.spawn((
                    Text::new("Pending: 0"),
                    TextFont::from_font_size(INSIGHT_FONT_SIZE),
                    TextColor(PENDING_COLOR),
                    PendingInsightDisplay,
                ));
            });

        // Commit button now lives in the left panel under the allocation
        // slider (see `update_study_detail_panel` / `update_insight_detail_panel`).

        // Bottom-right: Debug insight button
        #[cfg(debug_assertions)]
        right
            .spawn(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                right: Val::Px(12.0),
                ..default()
            })
            .with_children(|wrapper| {
                spawn_button(
                    wrapper,
                    "+10000 Insight",
                    (
                        StudyButtonAction::DebugGrantInsight,
                        crate::ui::focus::NoGamepadFocus,
                    ),
                    &DEBUG_BUTTON_STYLE,
                );
            });
    });

    // -- Left panel: Detail panel for selected node --
    commands.entity(left_panel_entity).with_children(|left| {
        // Scrollable detail panel. Right-stick scrolls it while a spell or
        // bonus is selected (via `GamepadScrollTarget`).
        left.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_height: Val::Px(0.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            StudyDetailPanel,
            crate::ui::focus::GamepadScrollTarget,
        ))
        .with_children(|panel| {
            // Placeholder text shown when nothing is selected
            panel.spawn((
                Text::new("Select a spell or bonus to view details"),
                TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
                TextColor(LOCKED_TEXT_COLOR),
            ));
        });
    });
}

// ===========================================================================
// Graph node selection
// ===========================================================================

/// Detects clicks on spell graph nodes and insight bonus nodes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_graph_node_clicks(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    node_query: Query<&SpellGraphNode>,
    insight_node_query: Query<&InsightBonusNode>,
    mut selected: ResMut<SelectedStudySpell>,
    mut selected_insight: ResMut<SelectedInsightBonus>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    for event in button_clicked.read() {
        // Check spell nodes first
        if let Ok(node) = node_query.get(event.button) {
            selected_insight.0 = None; // Clear insight selection
            if selected.0 == Some(node.spell) {
                selected.0 = None;
                animate_to_default_view(&mut commands);
            } else {
                selected.0 = Some(node.spell);
                animate_to_node(&mut commands, &graph_area_query, node.graph_position);
            }
            continue;
        }

        // Check insight bonus nodes
        if let Ok(inode) = insight_node_query.get(event.button) {
            selected.0 = None; // Clear spell selection
            if selected_insight.0 == Some(inode.stat) {
                selected_insight.0 = None;
                animate_to_default_view(&mut commands);
            } else {
                selected_insight.0 = Some(inode.stat);
                animate_to_node(&mut commands, &graph_area_query, inode.graph_position);
            }
        }
    }
}

/// Inserts the `GraphViewAnimation` that returns the graph to the "both
/// clusters visible" default — used by deselect paths (B button, re-clicking
/// the selected node, Commit rebuild).
pub(super) fn animate_to_default_view(commands: &mut Commands) {
    commands.insert_resource(GraphViewAnimation {
        target_offset: default_graph_offset(),
        target_scale: GRAPH_DEFAULT_SCALE,
        speed: GRAPH_ANIMATION_SPEED,
    });
}

/// Animates the graph view to center a node in the right 2/3 of the screen.
pub(super) fn animate_to_node(
    commands: &mut Commands,
    graph_area_query: &Query<&ComputedNode, With<SpellGraphArea>>,
    graph_position: Vec2,
) {
    if let Ok(computed) = graph_area_query.single() {
        let size = computed.size() * computed.inverse_scale_factor();
        let container_center = size / 2.0;
        // Center the node in the graph area (detail panel is in a separate left panel now)
        let target = container_center;
        let target_scale = GRAPH_ZOOM_MAX;
        let target_offset = target - container_center - graph_position * target_scale;
        commands.insert_resource(GraphViewAnimation {
            target_offset,
            target_scale,
            speed: GRAPH_ANIMATION_SPEED,
        });
    }
}

// ===========================================================================
// Pan & Zoom
// ===========================================================================

/// Clamps the view offset so the outermost graph nodes can reach roughly
/// the screen center but no further.
pub(super) fn clamp_view_offset(view: &mut GraphViewState, bounds: &GraphBounds) {
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
pub(crate) fn handle_graph_pan(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    ui_scale: Res<bevy::ui::UiScale>,
    mut view: ResMut<GraphViewState>,
    mut drag: ResMut<GraphDragState>,
    bounds: Option<Res<GraphBounds>>,
    mut selected: ResMut<SelectedStudySpell>,
    mut selected_insight: ResMut<SelectedInsightBonus>,
    node_interactions: Query<&Interaction, Or<(With<SpellGraphNode>, With<InsightBonusNode>)>>,
    graph_area_interaction: Query<&Interaction, With<SpellGraphArea>>,
    slider_interactions: Query<
        &Interaction,
        Or<(
            With<StudyAllocationSlider>,
            With<StudyAllocationHandle>,
            With<InsightBonusSlider>,
            With<InsightBonusSliderHandle>,
        )>,
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

    // Only interact with pan/deselect when cursor is over the graph area.
    // Clicks on the left panel (detail panel, talents, etc.) must not affect
    // the right panel's graph state.
    let cursor_over_graph = graph_area_interaction
        .iter()
        .any(|interaction| *interaction != Interaction::None);

    if buttons.just_pressed(MouseButton::Left) {
        // Don't start dragging if a node or slider is being pressed,
        // or if the cursor isn't over the graph area
        let any_node_pressed = node_interactions.iter().any(|i| *i == Interaction::Pressed);
        let slider_pressed = slider_interactions.iter().any(|i| *i != Interaction::None);
        if !any_node_pressed && cursor_over_graph && !slider_pressed {
            drag.dragging = true;
            drag.last_cursor = cursor_ui;
            drag.start_cursor = cursor_ui;
            commands.remove_resource::<GraphViewAnimation>();
        }
    }

    if buttons.just_released(MouseButton::Left) && drag.dragging {
        let total_moved = (cursor_ui - drag.start_cursor).length();
        // Deselect on a click (not a drag) on empty space within the graph area
        if total_moved < 4.0 {
            selected.0 = None;
            selected_insight.0 = None;
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_graph_zoom(
    mut commands: Commands,
    mut mouse_wheel: MessageReader<MouseWheel>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    ui_scale: Res<bevy::ui::UiScale>,
    mut view: ResMut<GraphViewState>,
    bounds: Option<Res<GraphBounds>>,
    graph_area_query: Query<
        (&ComputedNode, &bevy::ui::ui_transform::UiGlobalTransform),
        With<SpellGraphArea>,
    >,
) {
    let Some(cursor_pos) = corrected_cursor.0 else {
        return;
    };
    let Ok((computed, ui_transform)) = graph_area_query.single() else {
        return;
    };
    let cursor_ui = cursor_pos / ui_scale.0;
    let isf = computed.inverse_scale_factor();
    // Absolute center of the graph area in UI space
    let container_abs_center =
        Vec2::new(ui_transform.translation.x, ui_transform.translation.y) * isf;

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
            let cursor_from_center = cursor_ui - container_abs_center;
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
pub(crate) fn animate_graph_view(
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
pub(crate) fn update_graph_node_positions(
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
pub(crate) fn update_graph_node_borders(
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
pub(crate) fn update_graph_edge_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut segments: Query<(
        &mut Node,
        &mut UiTransform,
        &mut Visibility,
        &SpellGraphEdge,
    )>,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_size = computed.size() * computed.inverse_scale_factor();
    let container_center = container_size / 2.0;
    let thickness = (GRAPH_EDGE_THICKNESS * view.scale).max(1.0);

    for (mut node, mut ui_transform, mut vis, edge) in &mut segments {
        let screen_a = graph_to_screen(edge.start, &view, container_center);
        let screen_b = graph_to_screen(edge.end, &view, container_center);

        // Clip the line segment to the graph area bounds so the rotated
        // rectangle stays within the container. UiTransform rotation
        // bypasses Overflow::clip, so we must clip geometry manually.
        let clip = Vec2::ZERO..=container_size;
        if let Some((ca, cb)) = clip_line_to_rect(screen_a, screen_b, &clip) {
            *vis = Visibility::Inherited;
            let delta = cb - ca;
            let length = delta.length();
            let angle = delta.y.atan2(delta.x);
            let midpoint = (ca + cb) / 2.0;
            node.left = Val::Px(midpoint.x - length / 2.0);
            node.top = Val::Px(midpoint.y - thickness / 2.0);
            node.width = Val::Px(length);
            node.height = Val::Px(thickness);
            ui_transform.rotation = Rot2::radians(angle);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

// ===========================================================================
// Insight constellation position & border updates
// ===========================================================================

/// Updates the screen position of insight bonus nodes and the constellation anchor.
#[allow(clippy::type_complexity)]
pub(crate) fn update_insight_node_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut inode_query: Query<
        (&mut Node, &InsightBonusNode),
        (
            Without<InsightConstellationAnchor>,
            Without<SpellGraphNode>,
            Without<FreeNode>,
        ),
    >,
    mut anchor_query: Query<
        &mut Node,
        (
            With<InsightConstellationAnchor>,
            Without<InsightBonusNode>,
            Without<SpellGraphNode>,
            Without<FreeNode>,
        ),
    >,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_center = computed.size() * computed.inverse_scale_factor() / 2.0;

    for (mut node, inode) in &mut inode_query {
        let screen_pos = graph_to_screen(inode.graph_position, &view, container_center);
        let scaled_size = INSIGHT_NODE_SIZE * view.scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }

    for mut node in &mut anchor_query {
        let screen_pos = graph_to_screen(INSIGHT_CONSTELLATION_OFFSET, &view, container_center);
        let scaled_size = INSIGHT_ANCHOR_SIZE * view.scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }
}

/// Updates insight constellation edge positions based on pan/zoom state.
pub(crate) fn update_insight_edge_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut segments: Query<
        (
            &mut Node,
            &mut UiTransform,
            &mut Visibility,
            &InsightConstellationEdge,
        ),
        Without<SpellGraphEdge>,
    >,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_size = computed.size() * computed.inverse_scale_factor();
    let container_center = container_size / 2.0;
    let thickness = (GRAPH_EDGE_THICKNESS * view.scale).max(1.0);

    for (mut node, mut ui_transform, mut vis, edge) in &mut segments {
        let screen_a = graph_to_screen(edge.start, &view, container_center);
        let screen_b = graph_to_screen(edge.end, &view, container_center);

        let clip = Vec2::ZERO..=container_size;
        if let Some((ca, cb)) = clip_line_to_rect(screen_a, screen_b, &clip) {
            *vis = Visibility::Inherited;
            let delta = cb - ca;
            let length = delta.length();
            let angle = delta.y.atan2(delta.x);
            let midpoint = (ca + cb) / 2.0;
            node.left = Val::Px(midpoint.x - length / 2.0);
            node.top = Val::Px(midpoint.y - thickness / 2.0);
            node.width = Val::Px(length);
            node.height = Val::Px(thickness);
            ui_transform.rotation = Rot2::radians(angle);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

/// Updates border colors on insight bonus nodes based on selection state.
pub(crate) fn update_insight_node_borders(
    selected: Res<SelectedInsightBonus>,
    mut border_query: Query<(&mut BorderColor, &InsightBonusNode)>,
) {
    let bonuses = crate::config::save_data::get_all_insight_bonuses();
    let max = InsightBonusStat::max_level();
    for (mut border_color, inode) in &mut border_query {
        if selected.0 == Some(inode.stat) {
            *border_color = BorderColor::all(GRAPH_NODE_SELECTED_BORDER);
        } else {
            let level = bonuses.get(inode.stat.id()).copied().unwrap_or(0).min(max);
            let border = if level >= max {
                INSIGHT_NODE_MAXED_BORDER
            } else {
                INSIGHT_NODE_BORDER
            };
            *border_color = BorderColor::all(border);
        }
    }
}
