//! Study tab systems for the wizard tower.

use bevy::input::mouse::{MouseButton, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

#[cfg(debug_assertions)]
use crate::config::save_data::grant_insight;
use crate::config::save_data::{
    add_spell_research_progress, get_insight, get_spell_research_progress,
    get_spell_talent_progress, get_spell_talent_selections, load_unified_save,
    set_insight_bonus_levels, spend_insight,
};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseClicked;
use crate::game::messages::{InsightBonusUpgradedMessage, SpellResearchedMessage};
use crate::game::resources::BattleInsightData;
use crate::game::units::DamageType;
use crate::game::units::wizard::components::Spell;
use crate::ui::components::ButtonColors;
use crate::ui::main_menu::settings::components::SliderAdjusted;
use crate::ui::systems::{scale_font_by_text_width, spawn_button};

use super::components::*;
use super::constants::*;
use crate::game::insight_bonuses::InsightBonusStat;

use super::graph::{build_insight_constellation, build_spell_graph};
use super::materials::{
    ConcentricRingsData, ConcentricRingsMaterial, RadialProgressData, RadialProgressMaterial,
    StarSkyData, StarSkyMaterial,
};

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

/// Clips a line segment to an axis-aligned rectangle using the Liang-Barsky algorithm.
/// Returns the clipped endpoints, or `None` if the segment is entirely outside.
fn clip_line_to_rect(
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
        (-d.x, a.x - min.x),  // left
        (d.x, max.x - a.x),   // right
        (-d.y, a.y - min.y),  // top
        (d.y, max.y - a.y),   // bottom
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
// Study (MetaGameState::Study) systems
// ===========================================================================

/// Builds the study tab content into the wizard tower's left and right panels.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_study_panels(
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
    // Zoom out and offset to show both spell web and insight constellation
    commands.insert_resource(GraphViewState {
        offset: Vec2::new(-INSIGHT_CONSTELLATION_OFFSET.x * 0.5 * 0.55, 0.0),
        scale: 0.55,
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
pub(super) fn cleanup_study_resources(mut commands: Commands) {
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
pub(super) fn handle_study_button_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&StudyButtonAction>,
    allocation: Option<Res<InsightAllocation>>,
    battle_insight: Res<BattleInsightData>,
    mut spell_researched: MessageWriter<SpellResearchedMessage>,
    mut insight_upgraded: MessageWriter<InsightBonusUpgradedMessage>,
    left_panel: Query<Entity, With<super::layout::WizardTowerLeftPanel>>,
    right_panel: Query<Entity, With<super::layout::WizardTowerRightPanel>>,
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
fn rebuild_study_ui(
    commands: &mut Commands,
    left_panel: &Query<Entity, With<super::layout::WizardTowerLeftPanel>>,
    right_panel: &Query<Entity, With<super::layout::WizardTowerRightPanel>>,
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
        commands.insert_resource(GraphViewAnimation {
            target_offset: Vec2::ZERO,
            target_scale: 1.0,
            speed: GRAPH_ANIMATION_SPEED,
        });
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
fn spawn_study_panels(
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
    commands
        .entity(right_panel_entity)
        .with_children(|right| {
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
                            ..default()
                        },
                        BackgroundColor(INSIGHT_NODE_BG),
                        BorderColor::all(INSIGHT_ANCHOR_BORDER),
                        BorderRadius::all(Val::Percent(50.0)),
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
                                    ..default()
                                },
                                BackgroundColor(INSIGHT_NODE_BG),
                                BorderColor::all(border),
                                BorderRadius::all(Val::Percent(50.0)),
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
    commands
        .entity(right_panel_entity)
        .with_children(|right| {
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

            // Bottom-left: Commit button
            right
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(8.0),
                    left: Val::Px(12.0),
                    ..default()
                })
                .with_children(|wrapper| {
                    spawn_button(
                        wrapper,
                        "Commit",
                        StudyButtonAction::Commit,
                        &COMMIT_BUTTON_STYLE,
                    );
                });

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
                        StudyButtonAction::DebugGrantInsight,
                        &DEBUG_BUTTON_STYLE,
                    );
                });
        });

    // -- Left panel: Detail panel for selected node --
    commands
        .entity(left_panel_entity)
        .with_children(|left| {
            // Detail panel (always visible, shows placeholder when nothing selected)
            left.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    min_height: Val::Px(0.0),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                StudyDetailPanel,
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
pub(super) fn handle_graph_node_clicks(
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
                commands.remove_resource::<GraphViewAnimation>();
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
                commands.remove_resource::<GraphViewAnimation>();
            } else {
                selected_insight.0 = Some(inode.stat);
                animate_to_node(&mut commands, &graph_area_query, inode.graph_position);
            }
        }
    }
}

/// Animates the graph view to center a node in the right 2/3 of the screen.
fn animate_to_node(
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
pub(super) fn handle_graph_zoom(
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
    let container_abs_center = Vec2::new(ui_transform.translation.x, ui_transform.translation.y) * isf;

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
    mut segments: Query<(&mut Node, &mut UiTransform, &mut Visibility, &SpellGraphEdge)>,
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
pub(super) fn update_insight_node_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut inode_query: Query<
        (&mut Node, &InsightBonusNode),
        (Without<InsightConstellationAnchor>, Without<SpellGraphNode>, Without<FreeNode>),
    >,
    mut anchor_query: Query<
        &mut Node,
        (With<InsightConstellationAnchor>, Without<InsightBonusNode>, Without<SpellGraphNode>, Without<FreeNode>),
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
pub(super) fn update_insight_edge_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut segments: Query<
        (&mut Node, &mut UiTransform, &mut Visibility, &InsightConstellationEdge),
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
pub(super) fn update_insight_node_borders(
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

// ===========================================================================
// Detail panel
// ===========================================================================

/// Updates the detail panel based on the currently selected spell.
pub(super) fn update_study_detail_panel(
    mut commands: Commands,
    selected: Res<SelectedStudySpell>,
    selected_insight: Res<SelectedInsightBonus>,
    battle_insight: Res<BattleInsightData>,
    allocation: Option<Res<InsightAllocation>>,
    mut panel_query: Query<(Entity, &mut Node), With<StudyDetailPanel>>,
    asset_server: Res<AssetServer>,
) {
    if !selected.is_changed() {
        return;
    }

    // Don't touch the panel at all if insight bonus owns it
    if selected_insight.0.is_some() {
        return;
    }

    let Ok((panel_entity, _panel_node)) = panel_query.single_mut() else {
        return;
    };

    // Clear existing children
    commands.entity(panel_entity).despawn_related::<Children>();

    let Some(spell) = selected.0 else {
        // Show placeholder text when nothing is selected
        commands.entity(panel_entity).with_children(|panel| {
            panel.spawn((
                Text::new("Select a spell or bonus to view details"),
                TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
                TextColor(LOCKED_TEXT_COLOR),
            ));
        });
        return;
    };

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
            // Locked -- show requirements
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
                row_gap: Val::Px(10.0),
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

                    // Three cards in a row — padding accommodates 3D button edges
                    tiers_col
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(TALENT_CARD_GAP),
                            padding: UiRect::new(
                                Val::Px(2.0),
                                Val::Px(6.0),
                                Val::Px(0.0),
                                Val::Px(6.0),
                            ),
                            ..default()
                        })
                        .with_children(|card_row| {
                            for choice in 0..3u8 {
                                let def = &tier_defs[choice as usize];
                                let is_selected = current_selection == Some(choice);

                                let (bg_color, border_color) = if is_selected {
                                    (TALENT_ACTIVE_BG, TALENT_ACTIVE_BORDER)
                                } else if tier_unlocked {
                                    (TALENT_UNLOCKED_BG, TALENT_UNLOCKED_BORDER)
                                } else {
                                    (TALENT_LOCKED_BG, TALENT_LOCKED_BORDER)
                                };

                                let display_name = if tier_unlocked { def.name } else { "???" };

                                let mut talent_btn = card_row
                                    .spawn((
                                        Node {
                                            width: Val::Px(TALENT_CARD_WIDTH),
                                            height: Val::Px(TALENT_CARD_HEIGHT),
                                            border: UiRect::all(Val::Px(1.0)),
                                            padding: UiRect::all(Val::Px(3.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        Button,
                                        BackgroundColor(bg_color),
                                        BorderColor::all(border_color),
                                        BorderRadius::all(Val::Px(4.0)),
                                        ButtonColors {
                                            background: bg_color,
                                            border: border_color,
                                        },
                                        TalentCard {
                                            spell,
                                            tier,
                                            choice,
                                        },
                                    ));

                                if is_selected {
                                    talent_btn.insert(crate::ui::components::ButtonActive);
                                }

                                talent_btn
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

// ===========================================================================
// Insight bonus detail panel
// ===========================================================================

/// Updates the detail panel when an insight bonus is selected.
pub(super) fn update_insight_detail_panel(
    mut commands: Commands,
    selected: Res<SelectedStudySpell>,
    selected_insight: Res<SelectedInsightBonus>,
    allocation: Option<Res<InsightAllocation>>,
    mut panel_query: Query<(Entity, &mut Node), With<StudyDetailPanel>>,
) {
    if !selected_insight.is_changed() {
        return;
    }

    // Don't touch the panel if spell selection owns it
    if selected.0.is_some() {
        return;
    }

    let Ok((panel_entity, _panel_node)) = panel_query.single_mut() else {
        return;
    };

    commands.entity(panel_entity).despawn_related::<Children>();

    let Some(stat) = selected_insight.0 else {
        // Show placeholder text when nothing is selected
        commands.entity(panel_entity).with_children(|panel| {
            panel.spawn((
                Text::new("Select a spell or bonus to view details"),
                TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
                TextColor(LOCKED_TEXT_COLOR),
            ));
        });
        return;
    };

    let level = stat.current_level();
    let max = InsightBonusStat::max_level();
    let maxed = level >= max;
    let bonus_pct = level as f32 * InsightBonusStat::bonus_per_level() * 100.0;
    let cost_per = InsightBonusStat::cost_per_level();
    let total_cost = InsightBonusStat::total_cost();
    let committed_insight = level as u32 * cost_per;
    let current_alloc = allocation.as_ref().map(|a| a.get_bonus(&stat)).unwrap_or(0);

    commands.entity(panel_entity).with_children(|panel| {
        // Title
        panel.spawn((
            Text::new(stat.display_name()),
            TextFont::from_font_size(DETAIL_TITLE_FONT_SIZE),
            TextColor(INSIGHT_NODE_BORDER),
        ));

        // Description
        panel.spawn((
            Text::new(stat.description()),
            TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
            TextColor(TEXT_COLOR),
            Node {
                max_width: Val::Px(DETAIL_PANEL_WIDTH - DETAIL_PANEL_PADDING * 2.0),
                ..default()
            },
        ));

        // Current bonus
        let bonus_text = if maxed {
            format!("+{:.0}% (MAX)", bonus_pct)
        } else {
            format!("+{:.0}%", bonus_pct)
        };
        panel.spawn((
            Text::new(bonus_text),
            TextFont::from_font_size(DETAIL_TEXT_FONT_SIZE),
            TextColor(if maxed {
                INSIGHT_NODE_MAXED_BORDER
            } else {
                INSIGHT_PROGRESS_FILL
            }),
        ));

        // Level display
        panel.spawn((
            Text::new(format!("Level {} / {}", level, max)),
            TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
            TextColor(TEXT_COLOR),
        ));

        // Cost per level
        panel.spawn((
            Text::new(format!("{} Insight per level", cost_per)),
            TextFont::from_font_size(DETAIL_SMALL_FONT_SIZE),
            TextColor(LOCKED_TEXT_COLOR),
        ));

        // Separator
        panel.spawn(Node {
            height: Val::Px(6.0),
            ..default()
        });

        if maxed {
            panel.spawn((
                Text::new("MAXED"),
                TextFont::from_font_size(DETAIL_TITLE_FONT_SIZE),
                TextColor(INSIGHT_NODE_MAXED_BORDER),
            ));
        } else {
            // Allocation slider (same pattern as spell research)
            spawn_insight_bonus_slider(
                panel,
                stat,
                committed_insight,
                total_cost,
                current_alloc,
            );

            // Allocation text
            let pending_levels = current_alloc / cost_per;
            let alloc_text = if current_alloc > 0 {
                format!(
                    "{}+{}/{} (+{}%)",
                    committed_insight, current_alloc, total_cost, pending_levels
                )
            } else {
                format!("{}/{}", committed_insight, total_cost)
            };
            panel.spawn((
                Text::new(alloc_text),
                TextFont::from_font_size(DETAIL_TEXT_FONT_SIZE),
                TextColor(TEXT_COLOR),
                InsightBonusAllocationText { stat },
            ));
        }
    });
}

/// Spawns the allocation slider for an insight bonus in the detail panel.
fn spawn_insight_bonus_slider(
    parent: &mut ChildSpawnerCommands,
    stat: InsightBonusStat,
    committed: u32,
    total_cost: u32,
    current_alloc: u32,
) {
    let (progress_frac, alloc_frac, handle_pos) =
        compute_slider_fracs(committed, current_alloc, total_cost);

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
            BorderColor::all(INSIGHT_NODE_BORDER),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(SLIDER_TRACK_BG),
            Interaction::default(),
            RelativeCursorPosition::default(),
            InsightBonusSlider { stat },
        ))
        .with_children(|track| {
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
                BackgroundColor(INSIGHT_PROGRESS_FILL),
                InsightBonusProgressFill,
            ));

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
                InsightBonusAllocationFill { stat },
            ));

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
                InsightBonusSliderHandle {
                    stat,
                    is_dragging: false,
                },
            ));
        });
}

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
// Insight bonus slider interaction
// ===========================================================================

/// Handles click and drag on insight bonus allocation sliders.
/// Mirrors `handle_detail_slider_interaction` for spell sliders.
pub(super) fn handle_insight_bonus_slider_interaction(
    buttons: Res<ButtonInput<MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut InsightBonusSliderHandle)>,
    slider_tracks: Query<(
        &Interaction,
        &RelativeCursorPosition,
        &InsightBonusSlider,
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
                if handle.stat == track.stat {
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

    let mut dragging_stat: Option<InsightBonusStat> = None;
    for (_interaction, handle) in &slider_handles {
        if handle.is_dragging {
            dragging_stat = Some(handle.stat);
            break;
        }
    }

    let Some(stat) = dragging_stat else {
        return;
    };

    let insight_balance = get_insight();
    let cost_per = InsightBonusStat::cost_per_level();
    let total_cost = InsightBonusStat::total_cost();
    let level = stat.current_level();
    let committed = level as u32 * cost_per;
    let remaining = total_cost.saturating_sub(committed);

    if remaining == 0 {
        return;
    }

    for (_interaction, cursor_pos, track) in &slider_tracks {
        if track.stat != stat {
            continue;
        }

        let Some(pos) = cursor_pos.normalized else {
            continue;
        };

        let cursor_frac = (pos.x + 0.5).clamp(0.0, 1.0);
        let progress_frac = committed as f32 / total_cost as f32;
        let alloc_frac = (cursor_frac - progress_frac).max(0.0);
        let desired = (alloc_frac * total_cost as f32).round() as u32;

        let others: u32 = allocation
            .allocations
            .values()
            .sum::<u32>()
            + allocation
                .bonus_allocations
                .iter()
                .filter(|(s, _)| **s != stat)
                .map(|(_, v)| *v)
                .sum::<u32>();
        let max_for_stat = insight_balance.saturating_sub(others).min(remaining);
        let clamped = desired.min(max_for_stat);

        let old = allocation.get_bonus(&stat);
        if clamped != old {
            allocation.set_bonus(stat, clamped);
            slider_adjusted.write(SliderAdjusted);
        }
    }
}

/// Updates insight bonus slider visuals when allocation changes.
pub(super) fn update_insight_bonus_sliders(
    allocation: Res<InsightAllocation>,
    mut alloc_fills: Query<
        (&mut Node, &InsightBonusAllocationFill),
        (Without<InsightBonusSliderHandle>, Without<InsightBonusProgressFill>),
    >,
    mut slider_handles: Query<
        (&mut Node, &InsightBonusSliderHandle),
        (Without<InsightBonusAllocationFill>, Without<InsightBonusProgressFill>),
    >,
) {
    if !allocation.is_changed() {
        return;
    }

    let cost_per = InsightBonusStat::cost_per_level();
    let total_cost = InsightBonusStat::total_cost();

    for (mut node, fill) in &mut alloc_fills {
        let committed = fill.stat.current_level() as u32 * cost_per;
        let alloc = allocation.get_bonus(&fill.stat);
        let (progress_frac, alloc_frac, _) = compute_slider_fracs(committed, alloc, total_cost);

        node.left = Val::Percent(progress_frac * 100.0);
        node.width = Val::Percent(alloc_frac * 100.0);
    }

    for (mut node, handle) in &mut slider_handles {
        let committed = handle.stat.current_level() as u32 * cost_per;
        let alloc = allocation.get_bonus(&handle.stat);
        let (_, _, handle_pos) = compute_slider_fracs(committed, alloc, total_cost);

        node.left = Val::Px(handle_pos * SLIDER_TRACK_WIDTH - SLIDER_HANDLE_WIDTH / 2.0);
    }
}

/// Updates insight bonus allocation text.
pub(super) fn update_insight_bonus_allocation_text(
    allocation: Res<InsightAllocation>,
    mut texts: Query<(&mut Text, &InsightBonusAllocationText)>,
) {
    if !allocation.is_changed() {
        return;
    }

    let cost_per = InsightBonusStat::cost_per_level();
    let total_cost = InsightBonusStat::total_cost();

    for (mut text, alloc_text) in &mut texts {
        let stat = alloc_text.stat;
        let committed = stat.current_level() as u32 * cost_per;
        let alloc = allocation.get_bonus(&stat);
        let pending_levels = alloc / cost_per;

        if alloc > 0 {
            text.0 = format!("{}+{}/{} (+{}%)", committed, alloc, total_cost, pending_levels);
        } else {
            text.0 = format!("{}/{}", committed, total_cost);
        }
    }
}

/// Scales insight bonus label font sizes with the graph zoom level.
pub(super) fn update_graph_node_label_scale(
    view: Res<GraphViewState>,
    mut labels: Query<(&mut TextFont, &GraphNodeLabel)>,
) {
    for (mut font, label) in &mut labels {
        font.font_size = (label.base_size * view.scale).max(1.0);
    }
}

/// Updates concentric ring visuals on insight nodes when allocation changes.
pub(super) fn update_insight_bonus_rings(
    allocation: Res<InsightAllocation>,
    rings_query: Query<(&InsightBonusRings, &MaterialNode<ConcentricRingsMaterial>)>,
    mut ring_materials: ResMut<Assets<ConcentricRingsMaterial>>,
) {
    if !allocation.is_changed() {
        return;
    }

    let cost_per = InsightBonusStat::cost_per_level() as f32;

    for (rings, mat_handle) in &rings_query {
        let alloc = allocation.get_bonus(&rings.stat) as f32;
        // Fractional levels: e.g. 750 insight / 500 per level = 1.5 rings
        let pending_fractional = alloc / cost_per;
        if let Some(mat) = ring_materials.get_mut(mat_handle) {
            mat.data.pending = pending_fractional;
        }
    }
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
    mut button_clicked: MessageReader<MouseClicked>,
    card_query: Query<&TalentCard>,
) {
    use crate::config::save_data::{get_spell_talent_progress, set_spell_talent_selection};
    use crate::game::units::wizard::talents::constants as talent_consts;

    for event in button_clicked.read() {
        let Ok(card) = card_query.get(event.button) else {
            continue;
        };

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
