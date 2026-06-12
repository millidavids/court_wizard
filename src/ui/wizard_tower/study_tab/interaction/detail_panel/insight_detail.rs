use bevy::prelude::*;

use crate::game::insight_bonuses::InsightBonusStat;
use crate::ui::systems::spawn_button;

use super::super::super::super::components::*;
use super::super::super::super::constants::*;
use super::super::allocation::spawn_slider_row_with_buttons;
use super::super::slider_interaction::spawn_insight_bonus_slider;

/// Placeholder shown in the detail panel when no spell or bonus is selected.
pub(super) const NO_SELECTION_PLACEHOLDER: &str = "Select a spell or bonus to view details";

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
                Text::new(NO_SELECTION_PLACEHOLDER),
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
