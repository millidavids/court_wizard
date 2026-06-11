use std::cmp::Ordering;

use bevy::prelude::*;

use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, WizardInput,
};
use super::super::components::{ActiveMarkOfDeath, ExecutionerTriggered, MarkTalentFlags};
use super::super::constants;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, MarkedForDeathModifier, Team};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellSoundId;

/// Local wizard mark of death casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_mark_of_death_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (Entity, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    enemies_query: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    // Only the caster's OWN marks carry `MarkedForDeathModifier`. In multiplayer
    // the guest's ghost units also carry a BARE `ActiveMarkOfDeath` mirrored from
    // the host's marks — filtering on the modifier keeps a guest recast from
    // stripping (and flickering) the host's synced marks.
    existing_marks: Query<Entity, (With<ActiveMarkOfDeath>, With<MarkedForDeathModifier>)>,
    audio_ctx: (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        Option<Res<crate::networking::session::MultiplayerSession>>,
    ),
    active_talents: Option<Res<ActiveTalents>>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    target_assist: Res<TargetAssistWorldPos>,
    local_origin: Res<LocalSpellOrigin>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let (sfx, game_config, session) = &audio_ctx;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);
    let cursor_pos = input.cursor_pos;

    let Ok((_wizard_entity, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::MarkOfDeath {
        return;
    }

    let caster_team =
        crate::game::units::wizard::spells::utils::local_player_team(session.as_deref());
    let completed = mark_of_death_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &enemies_query,
        &existing_marks,
        active_talents.as_deref(),
        &mut talent_progress,
        caster_team,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Dark,
            time.elapsed_secs(),
        );
        if let Some(pos) = cursor_pos {
            audio::play_sfx_synced(
                &mut commands,
                &mut pending_cast_events,
                SpellSoundId::MarkOfDeathCast,
                pos,
                game_config,
                sfx,
            );
        }
        mouse_state.left_consumed = true;
    }
}

/// Core mark of death casting logic.
#[allow(clippy::too_many_arguments)]
fn mark_of_death_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    enemies_query: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
    existing_marks: &Query<Entity, (With<ActiveMarkOfDeath>, With<MarkedForDeathModifier>)>,
    active_talents: Option<&ActiveTalents>,
    talent_progress: &mut Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    caster_team: Team,
) -> bool {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(constants::MANA_COST) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Some(cursor_pos) = input.cursor_pos {
                        // Read talent selections
                        let t1 =
                            active_talents.and_then(|t| t.get_selection(Spell::MarkOfDeath, 0));
                        let t2 =
                            active_talents.and_then(|t| t.get_selection(Spell::MarkOfDeath, 1));
                        let t3 =
                            active_talents.and_then(|t| t.get_selection(Spell::MarkOfDeath, 2));

                        // Tier 1: compute amplification and duration
                        let mut amplification = constants::DAMAGE_AMPLIFICATION;
                        let mut duration = constants::MARK_DURATION;

                        match t1 {
                            Some(0) => amplification = constants::DEEP_MARK_AMPLIFICATION,
                            Some(1) => duration = constants::LINGERING_CURSE_DURATION,
                            _ => {}
                        }

                        // Tier 3-0: Mass Marking overrides amplification
                        let mass_marking = t3 == Some(0);
                        if mass_marking {
                            amplification = constants::MASS_MARKING_AMPLIFICATION;
                        }

                        // Apply empowerment
                        amplification *= primed_spell.empowerment;
                        duration *= primed_spell.empowerment;

                        // Build talent flags
                        let flags = MarkTalentFlags {
                            amplification,
                            swift_hex_refund: if t1 == Some(2) {
                                constants::MANA_COST * constants::SWIFT_HEX_REFUND_PERCENT
                            } else {
                                0.0
                            },
                            spreading_blight: t2 == Some(0),
                            executioner_brand: t2 == Some(1),
                            focal_point: t2 == Some(2),
                            deaths_ledger: t3 == Some(1),
                            doom: t3 == Some(2),
                        };

                        // Remove any existing marks (unless Doom marks which can't be removed)
                        for old_mark_entity in existing_marks.iter() {
                            commands
                                .entity(old_mark_entity)
                                .remove::<MarkedForDeathModifier>()
                                .remove::<ActiveMarkOfDeath>()
                                .remove::<MarkTalentFlags>()
                                .remove::<ExecutionerTriggered>();
                        }

                        let mut marked_count = 0u32;

                        if mass_marking {
                            // Mass Marking: mark all enemies in radius
                            for (entity, transform, team) in enemies_query.iter() {
                                if !caster_team.is_enemy(team) {
                                    continue;
                                }
                                let dist = crate::game::units::wizard::spells::utils::xz_distance(
                                    transform.translation,
                                    cursor_pos,
                                );
                                if dist <= constants::MASS_MARKING_RADIUS {
                                    commands.entity(entity).insert((
                                        MarkedForDeathModifier::new(amplification, duration),
                                        ActiveMarkOfDeath,
                                        flags.clone(),
                                    ));
                                    marked_count += 1;
                                }
                            }
                        } else {
                            // Single target: find nearest enemy to cursor
                            if let Some((target_entity, _)) = enemies_query
                                .iter()
                                .filter(|(_, _, team)| caster_team.is_enemy(team))
                                .filter_map(|(entity, transform, _)| {
                                    let dist =
                                        crate::game::units::wizard::spells::utils::xz_distance(
                                            transform.translation,
                                            cursor_pos,
                                        );
                                    if dist <= constants::TARGET_SEARCH_RADIUS {
                                        Some((entity, dist))
                                    } else {
                                        None
                                    }
                                })
                                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
                            {
                                commands.entity(target_entity).insert((
                                    MarkedForDeathModifier::new(amplification, duration),
                                    ActiveMarkOfDeath,
                                    flags,
                                ));
                                marked_count = 1;
                            }
                        }

                        // Track talent progress
                        if marked_count > 0
                            && let Some(progress) = talent_progress
                        {
                            progress.increment(Spell::MarkOfDeath, marked_count);
                        }
                    }
                    completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    completed
}
