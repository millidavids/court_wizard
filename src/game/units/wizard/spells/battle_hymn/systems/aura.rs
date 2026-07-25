use super::super::constants;
use crate::game::units::components::{
    AnthemResilience, BattleHymnModifier, EchoingSong, HasteModifier, Team, TemporaryHitPoints,
};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

#[allow(clippy::type_complexity)]
pub(crate) fn apply_battle_hymn_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    empowerment: f32,
    targets: &mut Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut BattleHymnModifier>,
            Option<&mut TemporaryHitPoints>,
            Option<&mut HasteModifier>,
        ),
        (
            Without<crate::game::units::wizard::components::Wizard>,
            Without<crate::game::multiplayer::components::GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    talent_progress: &mut Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    active_talents: Option<&ActiveTalents>,
) {
    let t1 = active_talents.and_then(|t| t.get_selection(Spell::BattleHymn, 0));
    let t2 = active_talents.and_then(|t| t.get_selection(Spell::BattleHymn, 1));
    let t3 = active_talents.and_then(|t| t.get_selection(Spell::BattleHymn, 2));

    // Base values
    let mut damage_bonus = constants::DAMAGE_BONUS * empowerment;
    let mut attack_speed = constants::ATTACK_SPEED_BONUS * empowerment;
    let mut duration = constants::BUFF_DURATION * empowerment;

    // Tier 1 modifications
    match t1 {
        Some(0) => duration *= constants::INSPIRING_WORDS_DURATION_MULT, // Inspiring Words: +50% duration
        Some(1) => damage_bonus *= constants::WAR_DRUMS_DAMAGE_MULT, // War Drums: +50% damage bonus
        // Wide Anthem radius is already applied via indicator.talent_radius_mult
        _ => {}
    }

    // Tier 3: Hymn of Legends doubles both bonuses (applied before Tier 2 adds effects)
    if t3 == Some(0) {
        damage_bonus *= constants::HYMN_OF_LEGENDS_MULT;
        attack_speed *= constants::HYMN_OF_LEGENDS_MULT;
    }

    // Tier 2 echo duration
    let has_echoing_song = t2 == Some(1);
    let echo_duration = if has_echoing_song {
        duration * 0.5
    } else {
        0.0
    };

    // Tier 3 damage reduction
    let has_anthem_resilience = t3 == Some(1);
    let anthem_reduction = if has_anthem_resilience {
        constants::ANTHEM_RESILIENCE_REDUCTION
    } else {
        0.0
    };

    // Tier 3: Chorus of Valor ignores radius (buff all defenders)
    let ignore_radius = t3 == Some(2);

    let mut buffed_count = 0u32;
    for (entity, transform, team, existing, existing_temp_hp, existing_haste) in targets.iter_mut()
    {
        let in_range = if ignore_radius {
            // Chorus of Valor: only buff defenders, but ignore radius
            *team == Team::Defenders
        } else {
            let distance = crate::game::units::wizard::spells::utils::xz_distance(
                transform.translation,
                circle_pos,
            );
            distance <= radius
        };

        if in_range {
            if let Some(mut buff) = existing {
                buff.damage_bonus = damage_bonus;
                buff.attack_speed = attack_speed;
                buff.refresh(duration);
            } else {
                commands.entity(entity).insert(BattleHymnModifier::new(
                    damage_bonus,
                    attack_speed,
                    duration,
                ));
            }

            // Insert/update talent sub-components
            if has_echoing_song {
                commands
                    .entity(entity)
                    .insert(EchoingSong::new(echo_duration));
            }
            if has_anthem_resilience {
                commands
                    .entity(entity)
                    .insert(AnthemResilience::new(anthem_reduction));
            }

            // Tier 2: Fortifying Hymn grants temporary HP
            if t2 == Some(0) {
                let temp_hp_amount = constants::FORTIFYING_HYMN_TEMP_HP * empowerment;
                if let Some(mut temp_hp) = existing_temp_hp {
                    if temp_hp.amount < temp_hp_amount {
                        temp_hp.amount = temp_hp_amount;
                        temp_hp.time_remaining = duration;
                    }
                } else {
                    commands
                        .entity(entity)
                        .insert(TemporaryHitPoints::new(temp_hp_amount, duration));
                }
            }

            // Tier 2: Swift March grants movement speed
            if t2 == Some(2) {
                let speed_bonus = constants::SWIFT_MARCH_SPEED_BONUS;
                if let Some(mut haste) = existing_haste {
                    haste.modifier = haste.modifier.max(speed_bonus);
                    haste.time_remaining = haste.time_remaining.max(duration);
                } else {
                    commands
                        .entity(entity)
                        .insert(HasteModifier::new(speed_bonus, duration));
                }
            }

            buffed_count += 1;
        }
    }

    if buffed_count > 0
        && let Some(progress) = talent_progress.as_deref_mut()
    {
        progress.increment(
            crate::game::units::wizard::components::Spell::BattleHymn,
            buffed_count,
        );
    }
}
