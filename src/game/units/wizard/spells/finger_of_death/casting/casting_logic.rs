use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::super::components::*;
use super::super::constants;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input,
};
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

/// Action the shared logic requests the wrapper to perform on beams.
enum BeamAction {
    /// Update existing beam with new data.
    UpdateBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        cast_progress: f32,
        delta_secs: f32,
    },
    /// Spawn a new beam (optionally with initial cast progress).
    SpawnBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        empowerment: f32,
        cast_progress: f32,
        talent_params: FodTalentParams,
    },
    /// Despawn all beams for this wizard.
    DespawnAll,
    /// No beam action needed.
    None,
}

/// Result from the shared casting logic.
struct CastingResult {
    beam_action: BeamAction,
    /// Whether to remove the AwaitingFingerOfDeathRelease component.
    remove_awaiting_release: bool,
}

/// Computes talent-modified parameters for Finger of Death.
pub(crate) fn compute_fod_params(active_talents: Option<&ActiveTalents>) -> FodTalentParams {
    let mut params = FodTalentParams::default();

    let talents = match active_talents {
        Some(t) => t,
        None => return params,
    };

    let t1 = talents.get_selection(Spell::FingerOfDeath, 0);
    let t2 = talents.get_selection(Spell::FingerOfDeath, 1);
    let t3 = talents.get_selection(Spell::FingerOfDeath, 2);

    // Tier 1
    match t1 {
        Some(0) => {
            // Death's Reach: wider beam
            params.beam_width *= constants::DEATHS_REACH_WIDTH_MULT;
            params.beam_width_fired *= constants::DEATHS_REACH_WIDTH_MULT;
        }
        Some(1) => {
            // Soul Harvest: mana refund on kill
            params.soul_harvest_refund = constants::SOUL_HARVEST_MANA_REFUND;
        }
        Some(2) => {
            // Quick Draw: faster cast
            params.cast_time_mult = constants::QUICK_DRAW_CAST_MULT;
        }
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => {
            // Finger of Undeath: raise killed as undead
            params.finger_of_undeath = true;
        }
        Some(1) => {
            // Death Sentence: cheaper, weaker, faster cooldown
            params.mana_threshold = constants::DEATH_SENTENCE_MANA_THRESHOLD;
            params.damage = constants::DEATH_SENTENCE_DAMAGE;
            params.cooldown_mult = constants::DEATH_SENTENCE_COOLDOWN_MULT;
        }
        Some(2) => {
            // Siphon Life: heal nearest defender
            params.siphon_life_percent = constants::SIPHON_LIFE_HEAL_PERCENT;
        }
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => {
            // Reaper's Scythe: sweep arc, reduced damage
            params.reapers_scythe = true;
            params.damage *= constants::REAPERS_SCYTHE_DAMAGE_MULT;
        }
        Some(1) => {
            // Necrotic Explosion: AoE on kill
            params.necrotic_explosion = true;
        }
        Some(2) => {
            // Deathmark: reduced damage + chain on kill
            params.deathmark = true;
            params.chain_damage_mult = constants::DEATHMARK_CHAIN_DAMAGE_PERCENT;
        }
        _ => {}
    }

    params
}

/// Local wizard Finger of Death casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_finger_of_death_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &mut CastingState, &Mana, &PrimedSpell, &Wizard),
        With<LocalWizard>,
    >,
    awaiting_release_query: Query<(), With<AwaitingFingerOfDeathRelease>>,
    cooldown_query: Query<(), (With<FingerOfDeathCooldown>, With<LocalWizard>)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut beams: Query<(Entity, &mut FingerOfDeathBeam)>,
    active_talents: Option<Res<ActiveTalents>>,
    target_assist: Res<TargetAssistWorldPos>,
    local_origin: Res<LocalSpellOrigin>,
) {
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, mut casting_state, mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::FingerOfDeath {
        return;
    }

    let awaiting_release = awaiting_release_query.get(wizard_entity).is_ok();
    let on_cooldown = cooldown_query.get(wizard_entity).is_ok();
    let has_existing_beam = beams.iter().next().is_some();
    let talent_params = compute_fod_params(active_talents.as_deref());

    let result = finger_of_death_casting_logic(
        &input,
        &time,
        &mut casting_state,
        mana,
        primed_spell,
        wizard,
        awaiting_release,
        on_cooldown,
        has_existing_beam,
        &talent_params,
        local_origin.0,
    );

    // Apply component changes
    if result.remove_awaiting_release {
        commands
            .entity(wizard_entity)
            .remove::<AwaitingFingerOfDeathRelease>();
    }

    // Apply beam action
    match result.beam_action {
        BeamAction::UpdateBeam {
            origin,
            direction,
            length,
            cast_progress,
            delta_secs,
        } => {
            if let Some((_, mut beam)) = beams.iter_mut().next() {
                beam.origin = origin;
                beam.direction = direction;
                beam.length = length;
                beam.cast_progress = cast_progress;
                beam.time_alive += delta_secs;
            }
        }
        BeamAction::SpawnBeam {
            origin,
            direction,
            length,
            empowerment,
            cast_progress,
            talent_params,
        } => {
            let mut new_beam = FingerOfDeathBeam::with_talents(
                origin,
                direction,
                length,
                empowerment,
                talent_params,
            );
            new_beam.cast_progress = cast_progress;
            super::damage::spawn_beam(&mut commands, &visual_assets, &mut materials, new_beam);
        }
        BeamAction::DespawnAll => {
            for (beam_entity, _) in beams.iter() {
                commands.entity(beam_entity).try_despawn();
            }
        }
        BeamAction::None => {}
    }
}

/// Core Finger of Death casting logic -- called by the local system.
#[allow(clippy::too_many_arguments)]
fn finger_of_death_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    awaiting_release: bool,
    on_cooldown: bool,
    has_existing_beam: bool,
    talent_params: &FodTalentParams,
    local_origin: Vec3,
) -> CastingResult {
    let mut result = CastingResult {
        beam_action: BeamAction::None,
        remove_awaiting_release: false,
    };

    let wizard_pos = local_origin;

    // Check for release event
    if input.just_released {
        result.remove_awaiting_release = true;
        casting_state.cancel();
        result.beam_action = BeamAction::DespawnAll;
        return result;
    }

    // Talent-modified cast time
    let cast_time = primed_spell.cast_time * talent_params.cast_time_mult;

    // Mouse is held - handle casting based on state
    match *casting_state {
        CastingState::Channeling { .. } => {
            // Finger of Death doesn't channel - just cancel
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Calculate beam target
            if let Some(cursor_pos) = input.cursor_pos {
                let (beam_origin, direction, beam_length) =
                    compute_beam_geometry(wizard_pos, cursor_pos, wizard.spell_range);

                // Calculate cast progress using talent-modified cast time
                let cast_progress = (casting_state.progress(cast_time)).min(1.0);

                // Update existing beam or spawn new one
                if has_existing_beam {
                    result.beam_action = BeamAction::UpdateBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        cast_progress,
                        delta_secs: time.delta_secs(),
                    };
                } else {
                    result.beam_action = BeamAction::SpawnBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        empowerment: primed_spell.empowerment,
                        cast_progress,
                        talent_params: talent_params.clone(),
                    };
                }
            }
        }
        CastingState::Resting => {
            // Not casting - check if we're waiting for mouse release or on cooldown
            if awaiting_release || on_cooldown {
                return result;
            }

            // Check for active input with talent-modified mana threshold
            if (input.just_pressed || input.pressed)
                && mana.percentage() >= talent_params.mana_threshold
            {
                casting_state.start_cast();

                // Spawn initial beam
                if let Some(cursor_pos) = input.cursor_pos {
                    let (beam_origin, direction, beam_length) =
                        compute_beam_geometry(wizard_pos, cursor_pos, wizard.spell_range);

                    result.beam_action = BeamAction::SpawnBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        empowerment: primed_spell.empowerment,
                        cast_progress: 0.0,
                        talent_params: talent_params.clone(),
                    };
                }
            }
        }
    }

    result
}

/// Computes beam origin, range-clamped direction, and beam length from wizard position
/// and cursor world position.
fn compute_beam_geometry(
    wizard_pos: Vec3,
    cursor_pos: Vec3,
    spell_range: f32,
) -> (Vec3, Vec3, f32) {
    let beam_origin = wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);
    let to_target = cursor_pos - beam_origin;
    let distance = to_target.length();
    let clamped_target = if distance > spell_range {
        beam_origin + to_target.normalize() * spell_range
    } else {
        cursor_pos
    };
    let direction = (clamped_target - beam_origin).normalize();
    let beam_length = (clamped_target - beam_origin)
        .length()
        .min(constants::BEAM_LENGTH);
    (beam_origin, direction, beam_length)
}
