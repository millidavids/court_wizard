use bevy::prelude::*;

use crate::config::save_data::{
    get_spell_research_progress, get_spell_talent_progress, get_spell_talent_selections,
};
use crate::game::resources::BattleInsightData;
use crate::game::units::wizard::components::Spell;
use crate::ui::systems::spawn_button;

use super::super::super::super::components::*;
use super::super::super::super::constants::*;
use super::super::super::panels::*;
use super::super::allocation::spawn_slider_row_with_buttons;
use super::super::slider_interaction::spawn_detail_unified_slider;

/// Updates the detail panel when a spell is selected.
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
pub(crate) fn spawn_talent_section(parent: &mut ChildSpawnerCommands, spell: Spell) {
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
                                    crate::ui::components::ButtonColors {
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
