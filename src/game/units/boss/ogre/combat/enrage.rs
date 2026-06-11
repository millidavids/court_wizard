use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{DamageMultiplier, Health, OriginalMaterial};

/// Updates the ogre's enrage state based on HP thresholds.
/// Modifies the sprite material's base_color to match the current enrage phase.
#[allow(clippy::type_complexity)]
pub fn update_enrage_state(
    mut bosses: Query<
        (
            &Health,
            &mut OgreEnrageState,
            &mut DamageMultiplier,
            &MeshMaterial3d<StandardMaterial>,
            Option<&OriginalMaterial>,
        ),
        With<Boss>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (health, mut enrage, mut damage_mult, mesh_material, original_material) in &mut bosses {
        let hp_ratio = health.current / health.max;

        let new_phase = if hp_ratio <= ENRAGE_PHASE_3_THRESHOLD {
            3
        } else if hp_ratio <= ENRAGE_PHASE_2_THRESHOLD {
            2
        } else if hp_ratio <= ENRAGE_PHASE_1_THRESHOLD {
            1
        } else {
            0
        };

        if new_phase != enrage.phase {
            enrage.phase = new_phase;

            // Update bonuses
            match new_phase {
                1 => {
                    enrage.speed_bonus = ENRAGE_1_SPEED_BONUS;
                    enrage.damage_bonus = ENRAGE_1_DAMAGE_BONUS;
                }
                2 => {
                    enrage.speed_bonus = ENRAGE_2_SPEED_BONUS;
                    enrage.damage_bonus = ENRAGE_2_DAMAGE_BONUS;
                }
                3 => {
                    enrage.speed_bonus = ENRAGE_3_SPEED_BONUS;
                    enrage.damage_bonus = ENRAGE_3_DAMAGE_BONUS;
                }
                _ => {
                    enrage.speed_bonus = 0.0;
                    enrage.damage_bonus = 0.0;
                }
            }

            // Update damage multiplier (base + enrage bonus)
            damage_mult.0 = OGRE_DAMAGE_MULTIPLIER + enrage.damage_bonus;

            let phase_tint = enrage_phase_tint(new_phase);

            // Update base_color on the per-entity sprite material.
            // If OriginalMaterial is present (spell effect active), update that
            // so the correct enrage tint restores when the effect ends.
            if let Some(orig) = original_material {
                if let Some(orig_mat) = materials.get_mut(&orig.0) {
                    orig_mat.base_color = phase_tint;
                }
            } else if let Some(mat) = materials.get_mut(&mesh_material.0) {
                mat.base_color = phase_tint;
            }
        }
    }
}

/// Returns the sprite tint color for a given enrage phase.
pub(crate) fn enrage_phase_tint(phase: u8) -> Color {
    match phase {
        1 => OGRE_ENRAGE_1_COLOR,
        2 => OGRE_ENRAGE_2_COLOR,
        3 => OGRE_ENRAGE_3_COLOR,
        _ => OGRE_COLOR,
    }
}
