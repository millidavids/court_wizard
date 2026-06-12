use bevy::prelude::*;

use crate::config::save_data::load_unified_save;
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

/// Returns the number of spells the player has fully researched.
pub(crate) fn count_researched_spells() -> u32 {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();

    Spell::researchable()
        .iter()
        .filter(|spell| {
            let name = spell.save_key().to_string();
            unlocked.contains(&name)
        })
        .count() as u32
}

/// Returns true if a spell's prerequisite is met.
pub(crate) fn is_prereq_met(spell: Spell) -> bool {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();

    if let Some(prereq) = spell.prerequisite() {
        let prereq_name = prereq.save_key().to_string();
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
    let name = spell.save_key().to_string();
    unlocked.contains(&name)
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
