//! Study tab detail panels, sliders, talent cards, cursor systems.

use super::panels::*;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui::ui_transform::UiGlobalTransform;

use crate::game::input::gamepad::resources::{ActiveInputDevice, GamepadAimSettings};
use crate::game::input::gamepad::systems::read_left_stick_shaped;

use crate::config::save_data::{
    get_insight, get_spell_research_progress, get_spell_talent_progress,
    get_spell_talent_selections,
};
use crate::game::input::messages::MouseClicked;
use crate::game::resources::BattleInsightData;
use crate::game::units::wizard::components::Spell;
use crate::ui::components::ButtonColors;
use crate::ui::main_menu::settings::components::SliderAdjusted;
use crate::ui::systems::spawn_button;

use super::super::components::*;
use super::super::constants::*;
use crate::game::insight_bonuses::InsightBonusStat;

use super::super::materials::{ConcentricRingsMaterial, StarSkyMaterial};

// ===========================================================================
// Shared helpers
// ===========================================================================

/// Calculates font size for talent card names based on the longest word.
pub(crate) fn update_study_detail_panel(
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
            // Unified progress + allocation slider, flanked by +/- buttons.
            let current_alloc = allocation.as_ref().map(|a| a.get(&spell)).unwrap_or(0);
            spawn_slider_row_with_buttons(panel, AllocTarget::Spell(spell), |row| {
                spawn_detail_unified_slider(row, spell, progress, cost, current_alloc);
            });

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

            // Commit button — only appears while the spell still has
            // progress left to earn.
            panel
                .spawn(Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(
                        row,
                        "Commit",
                        StudyButtonAction::Commit,
                        &COMMIT_BUTTON_STYLE,
                    );
                });
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
pub(super) fn spawn_talent_section(parent: &mut ChildSpawnerCommands, spell: Spell) {
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
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(TALENT_BAR_BG),
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
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(TALENT_BAR_FILL),
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

                                let mut talent_btn = card_row.spawn((
                                    Node {
                                        width: Val::Px(TALENT_CARD_WIDTH),
                                        height: Val::Px(TALENT_CARD_HEIGHT),
                                        border: UiRect::all(Val::Px(1.0)),
                                        padding: UiRect::all(Val::Px(3.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    Button,
                                    BackgroundColor(bg_color),
                                    BorderColor::all(border_color),
                                    ButtonColors {
                                        background: bg_color,
                                        border: border_color,
                                    },
                                    TalentCard {
                                        spell,
                                        tier,
                                        choice,
                                    },
                                    crate::ui::focus::Focusable,
                                ));

                                if is_selected {
                                    talent_btn.insert(crate::ui::components::ButtonActive);
                                }

                                talent_btn.with_children(|card| {
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
pub(crate) fn update_insight_detail_panel(
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
            // Allocation slider with +/- adjust buttons.
            spawn_slider_row_with_buttons(panel, AllocTarget::Bonus(stat), |row| {
                spawn_insight_bonus_slider(row, stat, committed_insight, total_cost, current_alloc);
            });

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

            // Commit button — only visible while the bonus can still rank up.
            panel
                .spawn(Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(
                        row,
                        "Commit",
                        StudyButtonAction::Commit,
                        &COMMIT_BUTTON_STYLE,
                    );
                });
        }
    });
}

/// Wraps an allocation slider in a horizontal row flanked by `-` and `+`
/// adjust buttons so gamepad users can step the allocation in discrete
/// increments. The caller supplies the inner slider via `spawn_slider`.
pub(super) fn spawn_slider_row_with_buttons(
    parent: &mut ChildSpawnerCommands,
    target: AllocTarget,
    spawn_slider: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let step = ALLOC_ADJUST_STEP as i32;
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|row| {
            spawn_button(
                row,
                "-",
                StudyAllocAdjustButton {
                    target,
                    delta: -step,
                },
                &ALLOC_ADJUST_BUTTON_STYLE,
            );
            spawn_slider(row);
            spawn_button(
                row,
                "+",
                StudyAllocAdjustButton {
                    target,
                    delta: step,
                },
                &ALLOC_ADJUST_BUTTON_STYLE,
            );
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
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BorderColor::all(INSIGHT_NODE_BORDER),
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
                    border_radius: BorderRadius {
                        top_left: Val::Px(8.0),
                        bottom_left: Val::Px(8.0),
                        top_right: Val::Px(0.0),
                        bottom_right: Val::Px(0.0),
                    },
                    ..default()
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
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
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
pub(super) fn spawn_detail_unified_slider(
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
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BorderColor::all(SLIDER_TRACK_BORDER),
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
                    border_radius: BorderRadius {
                        top_left: Val::Px(8.0),
                        bottom_left: Val::Px(8.0),
                        top_right: Val::Px(0.0),
                        bottom_right: Val::Px(0.0),
                    },
                    ..default()
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
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
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
pub(crate) fn handle_detail_slider_interaction(
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
pub(crate) fn update_detail_sliders(
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
pub(crate) fn update_allocation_text(
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
pub(crate) fn update_pending_insight_display(
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
pub(crate) fn handle_insight_bonus_slider_interaction(
    buttons: Res<ButtonInput<MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut InsightBonusSliderHandle)>,
    slider_tracks: Query<(&Interaction, &RelativeCursorPosition, &InsightBonusSlider)>,
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

        let others: u32 = allocation.allocations.values().sum::<u32>()
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
pub(crate) fn update_insight_bonus_sliders(
    allocation: Res<InsightAllocation>,
    mut alloc_fills: Query<
        (&mut Node, &InsightBonusAllocationFill),
        (
            Without<InsightBonusSliderHandle>,
            Without<InsightBonusProgressFill>,
        ),
    >,
    mut slider_handles: Query<
        (&mut Node, &InsightBonusSliderHandle),
        (
            Without<InsightBonusAllocationFill>,
            Without<InsightBonusProgressFill>,
        ),
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
pub(crate) fn update_insight_bonus_allocation_text(
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
            text.0 = format!(
                "{}+{}/{} (+{}%)",
                committed, alloc, total_cost, pending_levels
            );
        } else {
            text.0 = format!("{}/{}", committed, total_cost);
        }
    }
}

/// Scales insight bonus label font sizes with the graph zoom level.
pub(crate) fn update_graph_node_label_scale(
    view: Res<GraphViewState>,
    mut labels: Query<(&mut TextFont, &GraphNodeLabel)>,
) {
    for (mut font, label) in &mut labels {
        font.font_size = (label.base_size * view.scale).max(1.0);
    }
}

/// Updates concentric ring visuals on insight nodes when allocation changes.
pub(crate) fn update_insight_bonus_rings(
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
pub(crate) fn update_star_sky_time(
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

/// Handles clicks on talent cards to select/deselect talents. Updates
/// `ButtonActive` markers on the affected tier's cards in place rather than
/// rebuilding the detail panel — rebuilding would despawn the focused card
/// and snap focus back to the first card in tier 1.
pub(crate) fn handle_talent_card_clicks(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    card_query: Query<&TalentCard>,
    all_cards_query: Query<(Entity, &TalentCard)>,
) {
    use crate::config::save_data::{get_spell_talent_progress, set_spell_talent_selection};
    use crate::game::units::wizard::talents::constants as talent_consts;

    for event in button_clicked.read() {
        let Ok(card) = card_query.get(event.button) else {
            continue;
        };

        let talent_progress = get_spell_talent_progress(card.spell);
        let thresholds = talent_consts::tier_thresholds(card.spell);

        if talent_progress < thresholds[card.tier as usize] {
            continue;
        }

        let current = crate::config::save_data::get_spell_talent_selections(card.spell);
        let new_choice = if current[card.tier as usize] == Some(card.choice) {
            None
        } else {
            Some(card.choice)
        };

        set_spell_talent_selection(card.spell, card.tier as usize, new_choice);

        // Toggle ButtonActive on each sibling card in the same spell+tier so
        // exactly the selected card is marked active.
        for (entity, other) in &all_cards_query {
            if other.spell != card.spell || other.tier != card.tier {
                continue;
            }
            if new_choice == Some(other.choice) {
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
}

/// Updates the talent description text when hovering over talent cards.
pub(crate) fn update_talent_hover_description(
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
pub(crate) fn clear_talent_hover_description(
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

// ===========================================================================
// Gamepad cursor for the spell web
// ===========================================================================

/// Cursor speed at full right-stick deflection, in logical pixels per second.
const STUDY_CURSOR_SPEED: f32 = 900.0;

/// Reticle side length in logical pixels when idle.
const STUDY_CURSOR_SIZE: f32 = 24.0;

/// Reticle side length in logical pixels when a node is within click range.
const STUDY_CURSOR_HOVER_SIZE: f32 = 36.0;

/// Distance (physical pixels) from cursor center to a node's center within
/// which a press of A activates that node.
const STUDY_CURSOR_HOVER_RADIUS: f32 = 44.0;

const STUDY_CURSOR_IDLE_BORDER: Color = Color::hsla(270.0, 0.70, 0.75, 0.85);
const STUDY_CURSOR_HOVER_BORDER: Color = Color::hsla(48.0, 0.95, 0.65, 1.0);
const STUDY_CURSOR_IDLE_BG: Color = Color::hsla(270.0, 0.40, 0.40, 0.18);
const STUDY_CURSOR_HOVER_BG: Color = Color::hsla(48.0, 0.85, 0.55, 0.28);

/// Cursor state for the Study tab. Position is in logical pixels measured
/// from the `SpellGraphArea` top-left. `hovered` tracks the current graph
/// node within click radius (spell or insight).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct StudyCursorMode {
    pub position: Vec2,
    pub hovered: Option<Entity>,
}

#[derive(Component)]
pub(crate) struct StudyCursorReticle;

/// Spawns the cursor reticle inside the graph area and initialises cursor
/// resources the first frame a `SpellGraphArea` appears. Removed when the
/// graph area is despawned (tab change) via `cleanup_study_cursor`.
pub(crate) fn spawn_study_cursor_on_area_added(
    mut commands: Commands,
    new_area: Query<Entity, Added<SpellGraphArea>>,
    graph_area_node: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    for area in &new_area {
        commands.entity(area).with_children(|a| {
            a.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(STUDY_CURSOR_SIZE),
                    height: Val::Px(STUDY_CURSOR_SIZE),
                    left: Val::Px(-STUDY_CURSOR_SIZE * 0.5),
                    top: Val::Px(-STUDY_CURSOR_SIZE * 0.5),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                BackgroundColor(STUDY_CURSOR_IDLE_BG),
                BorderColor::all(STUDY_CURSOR_IDLE_BORDER),
                ZIndex(1000),
                StudyCursorReticle,
                // Let mouse clicks pass through the reticle to the spell
                // nodes beneath it.
                Pickable::IGNORE,
            ));
        });
        // Initialise cursor at panel center if we have the size; otherwise
        // the first update_study_cursor tick will fix it on clamp.
        let center = graph_area_node
            .get(area)
            .ok()
            .map(|n| n.size() * n.inverse_scale_factor() * 0.5)
            .unwrap_or(Vec2::ZERO);
        commands.insert_resource(StudyCursorMode {
            position: center,
            hovered: None,
        });
        commands.init_resource::<crate::ui::focus::FocusNavInhibit>();
    }
}

/// Reads the right stick and moves `StudyCursorMode.position` within the
/// logical bounds of the graph area. Clamps so the reticle can't leave the
/// panel.
pub(crate) fn update_study_cursor(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    aim: Res<GamepadAimSettings>,
    gamepads: Query<&Gamepad>,
    graph_area: Query<&ComputedNode, With<SpellGraphArea>>,
    mut cursor: ResMut<StudyCursorMode>,
) {
    let Some(gp_entity) = active.gamepad_entity() else {
        return;
    };
    let Ok(gamepad) = gamepads.get(gp_entity) else {
        return;
    };
    let Ok(area_node) = graph_area.single() else {
        return;
    };

    let shaped = read_left_stick_shaped(gamepad, &aim);
    let delta = shaped * STUDY_CURSOR_SPEED * time.delta_secs();
    if delta == Vec2::ZERO && !cursor.is_added() {
        return;
    }
    cursor.position += delta;

    let inv_sf = area_node.inverse_scale_factor();
    let size_logical = area_node.size() * inv_sf;
    cursor.position.x = cursor.position.x.clamp(0.0, size_logical.x);
    cursor.position.y = cursor.position.y.clamp(0.0, size_logical.y);
}

/// Writes `StudyCursorMode.position` into the reticle's `Node.left`/`top`
/// whenever the cursor moves. Skips when nothing changed to avoid dirtying
/// the UI layout every frame while the stick is idle.
pub(crate) fn sync_study_cursor_visual(
    cursor: Res<StudyCursorMode>,
    mut reticle: Query<&mut Node, With<StudyCursorReticle>>,
) {
    if !cursor.is_changed() {
        return;
    }
    let Ok(mut node) = reticle.single_mut() else {
        return;
    };
    let half = match node.width {
        Val::Px(w) => w * 0.5,
        _ => STUDY_CURSOR_SIZE * 0.5,
    };
    node.left = Val::Px(cursor.position.x - half);
    node.top = Val::Px(cursor.position.y - half);
}

/// Finds the nearest spell / insight node within `STUDY_CURSOR_HOVER_RADIUS`
/// of the reticle and caches it in `StudyCursorMode.hovered`.
#[allow(clippy::type_complexity)]
pub(crate) fn detect_study_cursor_hover(
    reticle: Query<&UiGlobalTransform, With<StudyCursorReticle>>,
    nodes: Query<
        (Entity, &UiGlobalTransform, Option<&InheritedVisibility>),
        Or<(With<SpellGraphNode>, With<InsightBonusNode>)>,
    >,
    mut cursor: ResMut<StudyCursorMode>,
) {
    let Ok(reticle_xform) = reticle.single() else {
        return;
    };
    let reticle_pos = reticle_xform.translation;

    let mut best: Option<(Entity, f32)> = None;
    for (entity, xform, vis) in &nodes {
        if !vis.map(|v| v.get()).unwrap_or(true) {
            continue;
        }
        let dist = (xform.translation - reticle_pos).length();
        if dist <= STUDY_CURSOR_HOVER_RADIUS
            && best.as_ref().map(|(_, d)| dist < *d).unwrap_or(true)
        {
            best = Some((entity, dist));
        }
    }
    let new_hover = best.map(|(e, _)| e);
    if cursor.hovered != new_hover {
        cursor.hovered = new_hover;
    }
}

/// Animates the reticle's size and colors between idle and hover states, and
/// hides the reticle whenever cursor mode is off (detail panel open). The
/// "cursor mode active" signal is the presence of `FocusNavInhibit`.
pub(crate) fn update_reticle_appearance(
    cursor: Res<StudyCursorMode>,
    inhibit: Option<Res<crate::ui::focus::FocusNavInhibit>>,
    active_input: Res<ActiveInputDevice>,
    mut reticle: Query<
        (&mut Node, &mut BackgroundColor, &mut BorderColor),
        With<StudyCursorReticle>,
    >,
) {
    let Ok((mut node, mut bg, mut border)) = reticle.single_mut() else {
        return;
    };
    // Reticle is a controller affordance only — when the player is on mouse
    // + keyboard the OS cursor is the input, so the reticle would just be
    // visual clutter.
    if inhibit.is_none() || !active_input.is_gamepad() {
        node.display = Display::None;
        return;
    }
    node.display = Display::Flex;
    let (size, bg_color, border_color) = if cursor.hovered.is_some() {
        (
            STUDY_CURSOR_HOVER_SIZE,
            STUDY_CURSOR_HOVER_BG,
            STUDY_CURSOR_HOVER_BORDER,
        )
    } else {
        (
            STUDY_CURSOR_SIZE,
            STUDY_CURSOR_IDLE_BG,
            STUDY_CURSOR_IDLE_BORDER,
        )
    };
    node.width = Val::Px(size);
    node.height = Val::Px(size);
    bg.set_if_neq(BackgroundColor(bg_color));
    border.set_if_neq(BorderColor::all(border_color));
}

/// Emits a `MouseClicked` on the hovered node when the A / South button is
/// just-pressed. The existing `handle_graph_node_clicks` system consumes it.
pub(crate) fn study_cursor_confirm(
    cursor: Res<StudyCursorMode>,
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut clicks: MessageWriter<MouseClicked>,
) {
    let Some(target) = cursor.hovered else {
        return;
    };
    let Some(gp_entity) = active.gamepad_entity() else {
        return;
    };
    let Ok(gamepad) = gamepads.get(gp_entity) else {
        return;
    };
    if gamepad.just_pressed(GamepadButton::South) {
        clicks.write(MouseClicked { button: target });
    }
}

/// Pans the graph when the cursor is pushed against the edge of the panel.
/// Pan speed scales with how close to the edge the cursor is (0 at
/// `EDGE_SCROLL_THRESHOLD` from the edge, full speed at the edge itself).
#[allow(clippy::too_many_arguments)]
pub(crate) fn study_cursor_edge_scroll(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    aim: Res<GamepadAimSettings>,
    gamepads: Query<&Gamepad>,
    cursor: Res<StudyCursorMode>,
    mut view: ResMut<GraphViewState>,
    bounds: Option<Res<GraphBounds>>,
    graph_area: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    const EDGE_SCROLL_THRESHOLD: f32 = 60.0; // logical px from edge
    const EDGE_SCROLL_SPEED: f32 = 700.0; // logical px/sec at the edge

    let Some(gp_entity) = active.gamepad_entity() else {
        return;
    };
    let Ok(gamepad) = gamepads.get(gp_entity) else {
        return;
    };
    let Ok(area_node) = graph_area.single() else {
        return;
    };

    // Stick input: only scroll while the player is actively pushing toward
    // the edge — letting go of the stick while the cursor rests at the edge
    // should stop the pan.
    let stick = read_left_stick_shaped(gamepad, &aim);
    if stick == Vec2::ZERO {
        return;
    }

    let size = area_node.size() * area_node.inverse_scale_factor();
    let pos = cursor.position;

    // Per-axis edge fraction: 0 when inside, rising to 1 at the edge. Sign
    // matches the pan direction the graph should move (`offset` +x reveals
    // content on the left).
    let edge_frac = |p: f32, max: f32| -> f32 {
        if p < EDGE_SCROLL_THRESHOLD {
            1.0 - (p / EDGE_SCROLL_THRESHOLD).clamp(0.0, 1.0)
        } else if p > max - EDGE_SCROLL_THRESHOLD {
            -(1.0 - ((max - p) / EDGE_SCROLL_THRESHOLD).clamp(0.0, 1.0))
        } else {
            0.0
        }
    };
    // Pan only along axes where both the edge AND the stick are pushing in
    // the same direction. `edge_frac` is positive when near the left/top
    // (pan content right/down) and the stick is negative in those directions
    // (pushing left/up) — so they have OPPOSITE signs when aligned.
    let raw_pan = Vec2::new(edge_frac(pos.x, size.x), edge_frac(pos.y, size.y));
    let pan = Vec2::new(
        if raw_pan.x * stick.x < 0.0 {
            raw_pan.x
        } else {
            0.0
        },
        if raw_pan.y * stick.y < 0.0 {
            raw_pan.y
        } else {
            0.0
        },
    );
    if pan == Vec2::ZERO {
        return;
    }
    view.offset += pan * EDGE_SCROLL_SPEED * time.delta_secs();
    if let Some(b) = bounds.as_ref() {
        clamp_view_offset(&mut view, b);
    }
}

/// Zooms the graph around the cursor when either trigger is held. RT zooms
/// in (scale up), LT zooms out. Reads the triggers as analog axes so partial
/// squeezes scale the zoom rate.
#[allow(clippy::too_many_arguments)]
pub(crate) fn study_cursor_trigger_zoom(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    cursor: Res<StudyCursorMode>,
    mut commands: Commands,
    mut view: ResMut<GraphViewState>,
    bounds: Option<Res<GraphBounds>>,
    graph_area: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    const TRIGGER_DEADZONE: f32 = 0.08;
    const TRIGGER_ZOOM_RATE: f32 = 2.4; // exp rate/sec at full trigger

    let Some(gp_entity) = active.gamepad_entity() else {
        return;
    };
    let Ok(gamepad) = gamepads.get(gp_entity) else {
        return;
    };
    let rt = gamepad
        .get(GamepadButton::RightTrigger2)
        .unwrap_or(0.0)
        .max(0.0);
    let lt = gamepad
        .get(GamepadButton::LeftTrigger2)
        .unwrap_or(0.0)
        .max(0.0);
    let rt = if rt >= TRIGGER_DEADZONE { rt } else { 0.0 };
    let lt = if lt >= TRIGGER_DEADZONE { lt } else { 0.0 };
    let signed = rt - lt; // +1 zoom in, -1 zoom out
    if signed.abs() < f32::EPSILON {
        return;
    }

    let Ok(area_node) = graph_area.single() else {
        return;
    };
    let old_scale = view.scale;
    let factor = (signed * TRIGGER_ZOOM_RATE * time.delta_secs()).exp();
    let new_scale = (old_scale * factor).clamp(GRAPH_ZOOM_MIN, GRAPH_ZOOM_MAX);
    if (new_scale - old_scale).abs() < f32::EPSILON {
        return;
    }

    // Keep the graph point under the cursor fixed during the zoom.
    let area_size = area_node.size() * area_node.inverse_scale_factor();
    let cursor_from_center = cursor.position - area_size * 0.5;
    let graph_point = (cursor_from_center - view.offset) / old_scale;
    view.offset = cursor_from_center - graph_point * new_scale;
    view.scale = new_scale;

    commands.remove_resource::<GraphViewAnimation>();
    if let Some(b) = bounds.as_ref() {
        clamp_view_offset(&mut view, b);
    }
}

/// Removes cursor resources when the `SpellGraphArea` is despawned (tab
/// change) so focus navigation resumes for the next tab.
pub(crate) fn cleanup_study_cursor_on_area_removed(
    mut commands: Commands,
    mut removed: RemovedComponents<SpellGraphArea>,
) {
    if removed.read().next().is_some() {
        commands.remove_resource::<StudyCursorMode>();
        commands.remove_resource::<crate::ui::focus::FocusNavInhibit>();
    }
}

// ===========================================================================
// Allocation +/- buttons + selection lifecycle
// ===========================================================================

/// Handles `StudyAllocAdjustButton` clicks — bumps the allocation for the
/// target spell/bonus by the button's `delta`. Clamped to `[0, remaining]`
/// where remaining is the smaller of "insight still needed to fully unlock"
/// and "insight the player hasn't already allocated elsewhere".
pub(crate) fn handle_alloc_adjust_buttons(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&StudyAllocAdjustButton>,
    mut allocation: ResMut<InsightAllocation>,
    battle_insight: Res<BattleInsightData>,
) {
    for event in button_clicked.read() {
        let Ok(btn) = button_query.get(event.button) else {
            continue;
        };
        let total_available = get_insight();
        match btn.target {
            AllocTarget::Spell(spell) => {
                let current = allocation.get(&spell) as i32;
                let progress = get_spell_research_progress(spell);
                let has_affinity = battle_insight
                    .damage_types_used
                    .contains(&spell.damage_type());
                // How much MORE insight is needed to finish this spell,
                // accounting for 2x affinity.
                let remaining_cost = spell.research_cost().saturating_sub(progress);
                let max_for_this = if has_affinity {
                    remaining_cost.div_ceil(2)
                } else {
                    remaining_cost
                };
                let other_allocated = allocation.total_allocated() - allocation.get(&spell);
                let cap = alloc_cap(total_available, other_allocated, max_for_this);
                let new = (current + btn.delta).clamp(0, cap as i32) as u32;
                allocation.set(spell, new);
            }
            AllocTarget::Bonus(stat) => {
                let current = allocation.get_bonus(&stat) as i32;
                let cost_per = InsightBonusStat::cost_per_level();
                let levels_remaining =
                    InsightBonusStat::max_level().saturating_sub(stat.current_level()) as u32;
                let max_for_this = levels_remaining * cost_per;
                let other_allocated = allocation.total_allocated() - allocation.get_bonus(&stat);
                let cap = alloc_cap(total_available, other_allocated, max_for_this);
                let new = (current + btn.delta).clamp(0, cap as i32) as u32;
                allocation.set_bonus(stat, new);
            }
        }
    }
}

/// Largest amount of insight the player can still commit to a single target,
/// given total available, what's already committed elsewhere, and the
/// target-specific ceiling (remaining unlock cost / levels × cost-per).
pub(super) fn alloc_cap(total_available: u32, other_allocated: u32, max_for_this: u32) -> u32 {
    total_available
        .saturating_sub(other_allocated)
        .min(max_for_this)
}

/// Per-button state for hold-to-repeat on `StudyAllocAdjustButton`.
#[derive(Clone, Copy)]
pub(crate) struct AllocHoldState {
    entity: Entity,
    pressed_at: std::time::Duration,
    last_fired_at: std::time::Duration,
}

/// Fires synthetic `MouseClicked` events on a held allocation `+`/`-` button
/// so the slider ramps up while the user holds it down. The interval starts
/// slow and accelerates to a cap over ~1.5 s of holding.
///
/// The initial single-press click still comes from `button_click_detection`
/// on release — this system only drives auto-repeats past an initial delay.
pub(crate) fn handle_alloc_adjust_buttons_hold(
    time: Res<Time>,
    buttons: Query<(Entity, &Interaction), With<StudyAllocAdjustButton>>,
    mut hold: Local<Option<AllocHoldState>>,
    mut clicks: MessageWriter<MouseClicked>,
) {
    const INITIAL_DELAY: std::time::Duration = std::time::Duration::from_millis(400);
    const START_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);
    const END_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);
    const RAMP_DURATION: std::time::Duration = std::time::Duration::from_millis(1500);

    let pressed = buttons
        .iter()
        .find(|(_, i)| **i == Interaction::Pressed)
        .map(|(e, _)| e);

    let Some(entity) = pressed else {
        *hold = None;
        return;
    };

    let now = time.elapsed();
    match hold.as_mut() {
        None => {
            *hold = Some(AllocHoldState {
                entity,
                pressed_at: now,
                last_fired_at: now,
            });
        }
        Some(state) if state.entity != entity => {
            *state = AllocHoldState {
                entity,
                pressed_at: now,
                last_fired_at: now,
            };
        }
        Some(state) => {
            let held = now.saturating_sub(state.pressed_at);
            if held < INITIAL_DELAY {
                return;
            }
            let ramp_t = ((held - INITIAL_DELAY).as_secs_f32() / RAMP_DURATION.as_secs_f32())
                .clamp(0.0, 1.0);
            let interval_secs =
                START_INTERVAL.as_secs_f32() * (1.0 - ramp_t) + END_INTERVAL.as_secs_f32() * ramp_t;
            let since_last = now.saturating_sub(state.last_fired_at);
            if since_last.as_secs_f32() >= interval_secs {
                state.last_fired_at = now;
                clicks.write(MouseClicked { button: entity });
            }
        }
    }
}

/// Resets the previously-selected spell/bonus's uncommitted allocation to
/// zero whenever the selection changes. Stops insight from "sticking" on a
/// spell the player glanced at and moved away from.
pub(crate) fn reset_previous_alloc_on_selection_change(
    selected: Res<SelectedStudySpell>,
    selected_insight: Res<SelectedInsightBonus>,
    mut allocation: ResMut<InsightAllocation>,
    mut last_spell: Local<Option<Spell>>,
    mut last_stat: Local<Option<InsightBonusStat>>,
) {
    if selected.is_changed() && *last_spell != selected.0 {
        if let Some(prev) = *last_spell
            && Some(prev) != selected.0
        {
            allocation.set(prev, 0);
        }
        *last_spell = selected.0;
    }
    if selected_insight.is_changed() && *last_stat != selected_insight.0 {
        if let Some(prev) = *last_stat
            && Some(prev) != selected_insight.0
        {
            allocation.set_bonus(prev, 0);
        }
        *last_stat = selected_insight.0;
    }
}

/// Hands focus between the spell web's cursor and the detail-panel's
/// focusables based on whether a spell/bonus is currently selected.
///
/// - Selected → remove `FocusNavInhibit` so the left stick / D-pad navigate
///   the detail panel (+/-, commit, talents).
/// - Deselected → re-insert `FocusNavInhibit` so the left stick resumes
///   driving the spell-web cursor.
pub(crate) fn toggle_focus_nav_on_study_selection(
    selected: Res<SelectedStudySpell>,
    selected_insight: Res<SelectedInsightBonus>,
    mut commands: Commands,
) {
    if !selected.is_changed() && !selected_insight.is_changed() {
        return;
    }
    let has_selection = selected.0.is_some() || selected_insight.0.is_some();
    if has_selection {
        commands.remove_resource::<crate::ui::focus::FocusNavInhibit>();
    } else {
        commands.init_resource::<crate::ui::focus::FocusNavInhibit>();
    }
}

/// While on the Study tab, the Back button (B / Escape) first deselects
/// any selected spell/bonus so focus returns to the cursor, and snaps the
/// graph view back to the default zoomed-out, centered position. Only after
/// nothing is selected does `escape_to_main_menu` exit to the main menu.
pub(crate) fn study_back_to_cursor(
    mut back: MessageReader<crate::game::input::gamepad::messages::MenuBackPressed>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selected: ResMut<SelectedStudySpell>,
    mut selected_insight: ResMut<SelectedInsightBonus>,
) {
    let back_fired = back.read().next().is_some() || keys.just_pressed(KeyCode::Escape);
    if !back_fired {
        return;
    }
    let mut changed = false;
    if selected.0.is_some() {
        selected.0 = None;
        changed = true;
    }
    if selected_insight.0.is_some() {
        selected_insight.0 = None;
        changed = true;
    }
    if changed {
        animate_to_default_view(&mut commands);
    }
}

/// Offset for the default (fully zoomed-out) view — horizontally centered
/// between the spell web and the insight constellation.
pub(super) fn default_graph_offset() -> Vec2 {
    Vec2::new(
        -INSIGHT_CONSTELLATION_OFFSET.x * 0.5 * GRAPH_DEFAULT_SCALE,
        0.0,
    )
}
