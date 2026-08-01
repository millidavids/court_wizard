use bevy::prelude::*;

use crate::config::save_data::{
    get_all_insight_bonus_progress, get_insight, get_spell_research_progress,
};
use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::resources::BattleInsightData;
#[cfg(debug_assertions)]
use crate::ui::systems::spawn_button;

use super::super::super::components::*;
use super::super::super::constants::*;
use super::super::super::graph::{build_insight_constellation, build_spell_graph};
use super::super::super::materials::{
    ConcentricRingsData, ConcentricRingsMaterial, RadialProgressData, RadialProgressMaterial,
    StarSkyData, StarSkyMaterial,
};
use super::helpers::{is_prereq_met, is_spell_unlocked};
// Debug-only marker: defined and used only under `debug_assertions`, so the
// import must be gated too or release builds fail with an unresolved import.
#[cfg(debug_assertions)]
use super::spawn::DebugInsightButton;

/// Spawns study tab content into the wizard tower's left and right panels.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_study_panels(
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
                    let is_free = cost == 0;

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

                // Constellation stat nodes. One batched save read for all of
                // them — each per-stat read deep-clones the whole save file.
                let bonus_progress = get_all_insight_bonus_progress();
                for inode_def in &insight_nodes {
                    let stat = inode_def.stat;
                    let progress = bonus_progress.get(stat.id()).copied().unwrap_or(0);
                    let level = InsightBonusStat::level_for_progress(progress);
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
                                    // Fractional: banked progress below the
                                    // next threshold draws as a partial arc.
                                    filled: progress as f32
                                        / InsightBonusStat::cost_per_level() as f32,
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

        // Bottom-right: Debug insight button. Hidden by default — toggled
        // by the global F2 debug-UI flag (see `crate::game::debug_ui`).
        #[cfg(debug_assertions)]
        right
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(8.0),
                    right: Val::Px(12.0),
                    ..default()
                },
                Visibility::Hidden,
                DebugInsightButton,
            ))
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
                Text::new(NO_SELECTION_PLACEHOLDER),
                TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
                TextColor(LOCKED_TEXT_COLOR),
            ));
        });
    });
}
