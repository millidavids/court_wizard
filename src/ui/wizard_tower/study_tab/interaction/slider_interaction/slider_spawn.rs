use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::units::wizard::components::Spell;

use super::super::super::super::components::*;
use super::super::super::super::constants::*;
use super::super::super::panels::*;

/// Spawns a unified progress + allocation slider in the detail panel.
/// Committed progress is shown as a non-reducible filled region on the left.
/// The slider handle controls the pending allocation region that starts after the
/// committed progress.
pub(crate) fn spawn_detail_unified_slider(
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

/// Spawns the allocation slider for an insight bonus in the detail panel.
pub(crate) fn spawn_insight_bonus_slider(
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
