use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::save_data::{get_insight, get_spell_research_progress};
use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::units::wizard::components::Spell;
use crate::ui::main_menu::settings::components::SliderAdjusted;

use super::super::super::super::components::*;
use super::super::super::super::constants::*;
use super::super::super::super::materials::{ConcentricRingsMaterial, StarSkyMaterial};
use super::super::super::panels::*;

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

        // Sum everything else allocated (spell slots AND bonus-stat slots), so a
        // drag can't exceed the true insight balance. Mirrors the +/- buttons.
        let others: u32 = allocation.total_allocated() - allocation.get(&spell);
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
    battle_insight: Res<crate::game::resources::BattleInsightData>,
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
