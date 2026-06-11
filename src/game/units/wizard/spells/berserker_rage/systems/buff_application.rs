use super::super::components::{
    BerserkerRageTalentParams, Bloodlust, ContagiousRage, FinalStand, Frenzy, UndyingFury,
};
use super::super::constants;
use crate::game::units::components::{BerserkerRageModifier, Corpse, Team};
use crate::game::units::wizard::components::{Spell, Wizard};
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> BerserkerRageTalentParams {
    let mut params = BerserkerRageTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    // Tier 1
    match talents.get_selection(Spell::BerserkerRage, 0) {
        Some(0) => {
            params.damage_bonus = constants::BLOOD_FURY_DAMAGE_BONUS;
            params.vulnerability = constants::BLOOD_FURY_VULNERABILITY;
        }
        Some(1) => {
            params.damage_bonus = constants::CONTROLLED_RAGE_DAMAGE_BONUS;
            params.vulnerability = constants::CONTROLLED_RAGE_VULNERABILITY;
        }
        Some(2) => {
            params.radius_mult = constants::PRIMAL_ROAR_RADIUS_MULT;
        }
        _ => {}
    }

    // Tier 2
    match talents.get_selection(Spell::BerserkerRage, 1) {
        Some(0) => params.bloodlust = true,
        Some(1) => params.undying_fury = true,
        Some(2) => params.frenzy = true,
        _ => {}
    }

    // Tier 3
    match talents.get_selection(Spell::BerserkerRage, 2) {
        Some(0) => params.wrath_incarnate = true,
        Some(1) => params.contagious_rage = true,
        Some(2) => params.final_stand = true,
        _ => {}
    }

    params
}

/// Applies the berserker rage buff to all units within the circle.
/// Returns the number of units buffed.
pub(crate) fn apply_berserker_rage_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    empowerment: f32,
    talent_params: &BerserkerRageTalentParams,
    targets: &mut Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut BerserkerRageModifier>,
        ),
        (Without<Wizard>, Without<Corpse>),
    >,
) -> u32 {
    // Apply Wrath Incarnate override if active
    let damage_bonus = if talent_params.wrath_incarnate {
        constants::WRATH_INCARNATE_DAMAGE_BONUS
    } else {
        talent_params.damage_bonus
    } * empowerment;

    let vulnerability = if talent_params.wrath_incarnate {
        constants::WRATH_INCARNATE_VULNERABILITY
    } else {
        talent_params.vulnerability
    } * empowerment;

    let duration = constants::BUFF_DURATION * empowerment;
    let mut buffed_count = 0u32;

    for (entity, transform, _team, existing) in targets.iter_mut() {
        let distance = crate::game::units::wizard::spells::utils::xz_distance(
            transform.translation,
            circle_pos,
        );
        if distance <= radius {
            if let Some(mut buff) = existing {
                buff.damage_bonus = damage_bonus;
                buff.damage_vulnerability = vulnerability;
                buff.refresh(duration);
            } else {
                commands.entity(entity).insert(BerserkerRageModifier::new(
                    damage_bonus,
                    vulnerability,
                    duration,
                ));
            }

            // Tier 2: behavioral components
            if talent_params.bloodlust {
                commands.entity(entity).insert(Bloodlust {
                    heal_fraction: constants::BLOODLUST_HEAL_FRACTION,
                });
            }
            if talent_params.undying_fury {
                commands.entity(entity).insert(UndyingFury);
            }
            if talent_params.frenzy {
                commands.entity(entity).insert(Frenzy {
                    attack_speed_bonus: constants::FRENZY_ATTACK_SPEED_BONUS,
                    hp_threshold: constants::FRENZY_HP_THRESHOLD,
                });
            }

            // Tier 3: behavioral components
            if talent_params.contagious_rage {
                commands.entity(entity).insert(ContagiousRage {
                    damage_bonus,
                    vulnerability,
                    duration,
                });
            }
            if talent_params.final_stand {
                // Damage = 50% of the unit's max HP (applied later when we know max HP)
                commands.entity(entity).insert(FinalStand {
                    damage_fraction: constants::FINAL_STAND_DAMAGE_FRACTION,
                    radius: constants::FINAL_STAND_RADIUS,
                });
            }

            buffed_count += 1;
        }
    }

    buffed_count
}
