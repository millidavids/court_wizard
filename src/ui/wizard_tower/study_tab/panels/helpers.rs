use bevy::prelude::*;

use crate::config::save_data::load_unified_save;
use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::units::DamageType;
use crate::game::units::wizard::components::Spell;
use crate::ui::systems::scale_font_by_text_width;

use super::super::super::components::*;
use super::super::super::constants::*;

/// Calculates font size for talent card names based on the longest word.
pub(crate) fn calculate_talent_font_size(name: &str) -> f32 {
    let max_word_width = name.split_whitespace().map(|w| w.len()).max().unwrap_or(0) as f32;
    scale_font_by_text_width(max_word_width, 7.0, 13.0, 0.65, TALENT_CARD_FONT)
}

/// Loads the player's unlocked-spell name list from save (empty if no save).
pub(crate) fn load_unlocked_spells() -> Vec<String> {
    load_unified_save()
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default()
}

/// Returns true if `spell`'s save key is present in a pre-loaded unlocked list.
pub(crate) fn is_spell_unlocked_in(spell: Spell, unlocked: &[String]) -> bool {
    unlocked.iter().any(|n| n.as_str() == spell.save_key())
}

/// Counts fully-researched spells against a pre-loaded unlocked list.
pub(crate) fn count_researched_spells_in(unlocked: &[String]) -> u32 {
    Spell::researchable()
        .iter()
        .filter(|spell| is_spell_unlocked_in(**spell, unlocked))
        .count() as u32
}

/// Returns true if `spell`'s prerequisite is met against a pre-loaded unlocked list.
pub(crate) fn is_prereq_met_in(spell: Spell, unlocked: &[String]) -> bool {
    if let Some(prereq) = spell.prerequisite()
        && !is_spell_unlocked_in(prereq, unlocked)
    {
        return false;
    }
    let required = spell.required_total_spells();
    required == 0 || count_researched_spells_in(unlocked) >= required
}

/// Returns the number of spells the player has fully researched.
pub(crate) fn count_researched_spells() -> u32 {
    count_researched_spells_in(&load_unlocked_spells())
}

/// Returns true if a spell's prerequisite is met.
pub(crate) fn is_prereq_met(spell: Spell) -> bool {
    is_prereq_met_in(spell, &load_unlocked_spells())
}

/// Returns true if this spell is fully researched (unlocked).
pub(crate) fn is_spell_unlocked(spell: Spell) -> bool {
    is_spell_unlocked_in(spell, &load_unlocked_spells())
}

/// Returns the color associated with a damage type for UI display.
pub(crate) fn element_color(damage_type: DamageType) -> Color {
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
pub(crate) fn graph_to_screen(
    graph_pos: Vec2,
    view: &GraphViewState,
    container_center: Vec2,
) -> Vec2 {
    graph_pos * view.scale + view.offset + container_center
}

/// Clips a line segment to an axis-aligned rectangle using the Liang-Barsky algorithm.
/// Returns the clipped endpoints, or `None` if the segment is entirely outside.
pub(crate) fn clip_line_to_rect(
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

/// Whole bonus levels a pending allocation would complete, accounting for the
/// partial progress already banked below the next threshold. `alloc / cost` is
/// wrong here: 400 banked plus 200 allocated completes a level, but 200/500 = 0.
fn pending_bonus_levels(progress: u32, alloc: u32) -> u32 {
    let after = InsightBonusStat::level_for_progress(progress.saturating_add(alloc));
    after.saturating_sub(InsightBonusStat::level_for_progress(progress)) as u32
}

/// Progress readout under a bonus node's slider, e.g. `550+300/2500 (+1%)`.
pub(crate) fn format_bonus_alloc_text(committed: u32, alloc: u32) -> String {
    let total = InsightBonusStat::total_cost();
    if alloc > 0 {
        format!(
            "{}+{}/{} (+{}%)",
            committed,
            alloc,
            total,
            pending_bonus_levels(committed, alloc)
        )
    } else {
        format!("{}/{}", committed, total)
    }
}

/// Computes progress and allocation fractions for the unified slider.
/// Returns `(progress_frac, alloc_frac, handle_pos)` where handle_pos = progress_frac + alloc_frac.
pub(crate) fn compute_slider_fracs(progress: u32, alloc: u32, cost: u32) -> (f32, f32, f32) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_levels_account_for_banked_remainder() {
        assert_eq!(pending_bonus_levels(0, 200), 0);
        // 400 banked + 200 allocated completes a level; `alloc / cost` said 0.
        assert_eq!(pending_bonus_levels(400, 200), 1);
        assert_eq!(pending_bonus_levels(0, 1200), 2);
    }

    #[test]
    fn pending_levels_cap_at_max() {
        assert_eq!(pending_bonus_levels(2400, 9999), 1);
        assert_eq!(pending_bonus_levels(2500, 500), 0);
    }

    #[test]
    fn alloc_text_shows_banked_progress_and_pending_levels() {
        assert_eq!(format_bonus_alloc_text(250, 0), "250/2500");
        assert_eq!(format_bonus_alloc_text(400, 200), "400+200/2500 (+1%)");
    }
}
