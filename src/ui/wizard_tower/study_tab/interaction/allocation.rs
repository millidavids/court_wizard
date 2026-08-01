use bevy::prelude::*;

use crate::config::save_data::{get_insight, get_spell_research_progress};
use crate::game::input::messages::MouseClicked;
use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::units::wizard::components::Spell;
use crate::ui::systems::spawn_button;

use super::super::super::components::*;
use super::super::super::constants::*;
use super::super::panels::*;

/// Wraps an allocation slider in a horizontal row flanked by `-` and `+`
/// adjust buttons so gamepad users can step the allocation in discrete
/// increments. The caller supplies the inner slider via `spawn_slider`.
pub(crate) fn spawn_slider_row_with_buttons(
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

/// Largest amount of insight the player can still commit to a single target,
/// given total available, what's already committed elsewhere, and the
/// target-specific ceiling (remaining unlock cost / levels × cost-per).
pub(crate) fn alloc_cap(total_available: u32, other_allocated: u32, max_for_this: u32) -> u32 {
    total_available
        .saturating_sub(other_allocated)
        .min(max_for_this)
}

/// Per-button state for hold-to-repeat on `StudyAllocAdjustButton`.
#[derive(Clone, Copy)]
pub(crate) struct AllocHoldState {
    pub entity: Entity,
    pub pressed_at: std::time::Duration,
    pub last_fired_at: std::time::Duration,
}

/// Handles `StudyAllocAdjustButton` clicks — bumps the allocation for the
/// target spell/bonus by the button's `delta`. Clamped to `[0, remaining]`
/// where remaining is the smaller of "insight still needed to fully unlock"
/// and "insight the player hasn't already allocated elsewhere".
pub(crate) fn handle_alloc_adjust_buttons(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&StudyAllocAdjustButton>,
    mut allocation: ResMut<InsightAllocation>,
    battle_insight: Res<crate::game::resources::BattleInsightData>,
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
                // Cap on banked progress, not whole levels — partial progress
                // counts, so the node can absorb any amount up to its total.
                let max_for_this = InsightBonusStat::remaining_capacity(stat.current_progress());
                let other_allocated = allocation.total_allocated() - allocation.get_bonus(&stat);
                let cap = alloc_cap(total_available, other_allocated, max_for_this);
                let new = (current + btn.delta).clamp(0, cap as i32) as u32;
                allocation.set_bonus(stat, new);
            }
        }
    }
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
pub(crate) fn default_graph_offset() -> Vec2 {
    Vec2::new(
        -INSIGHT_CONSTELLATION_OFFSET.x * 0.5 * GRAPH_DEFAULT_SCALE,
        0.0,
    )
}
