use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::super::super::vfx;
use super::super::components::BanishmentTalentParams;
use super::super::constants;
use super::cast_logic::{cast_mass_banishment, cast_single_banishment};
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{BanishedModifier, Corpse, Health, Team, WasBanished};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_cursor_to_spell_range_with_origin, ground_projected_range, local_player_team,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::session::MultiplayerSession;
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> BanishmentTalentParams {
    let mut params = BanishmentTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    // Tier 1
    match talents.get_selection(Spell::Banishment, 0) {
        Some(0) => {
            params.duration = constants::EXTENDED_EXILE_DURATION;
        }
        Some(1) => {
            params.cast_time_mult = constants::QUICK_DISMISSAL_CAST_TIME_MULT;
        }
        Some(2) => {
            params.mana_mult = constants::CHEAP_TICKET_MANA_MULT;
        }
        _ => {}
    }

    // Tier 2
    match talents.get_selection(Spell::Banishment, 1) {
        Some(0) => params.painful_return = true,
        Some(1) => params.displacement = true,
        Some(2) => params.dual_banishment = true,
        _ => {}
    }

    // Tier 3
    match talents.get_selection(Spell::Banishment, 2) {
        Some(0) => params.dimensional_shunt = true,
        Some(1) => params.mass_banishment = true,
        Some(2) => params.one_way_trip = true,
        _ => {}
    }

    params
}

/// Local wizard banishment casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_banishment_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    enemies_query: Query<
        (Entity, &Transform, &Team, &Health),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    mut progress: ResMut<BattleTalentProgress>,
    visual_assets: Res<SpellVisualAssets>,
    // `session` is bundled with `target_assist` to stay under Bevy's 16-param
    // system limit. `local_player_team` reads it so a versus guest (Team::Attackers)
    // banishes the enemy wave, not their own army.
    assist_ctx: (Res<TargetAssistWorldPos>, Option<Res<MultiplayerSession>>),
    local_origin: Res<LocalSpellOrigin>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let (target_assist, session) = &assist_ctx;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, target_assist);

    let Ok((_wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Banishment {
        return;
    }

    let spell_range = ground_projected_range(wizard.spell_range, local_origin.0.y);
    input.cursor_pos = clamp_cursor_to_spell_range_with_origin(
        input.cursor_pos,
        local_origin.0,
        wizard.spell_range,
        0.0,
    );

    let talent_params = compute_talent_params(active_talents.as_deref());
    let caster_team = local_player_team(session.as_deref());

    let banished_count = banishment_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &enemies_query,
        &talent_params,
        spell_range,
        &visual_assets,
        time.elapsed_secs(),
        local_origin.0,
        &mut pending_cast_events,
        caster_team,
    );

    if banished_count > 0 {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Force,
            time.elapsed_secs(),
        );
        progress.increment(Spell::Banishment, banished_count);
        audio::play_sfx_synced(
            &mut commands,
            &mut pending_cast_events,
            SpellSoundId::BanishmentCast,
            local_origin.0,
            &game_config,
            &sfx,
        );
        mouse_state.left_consumed = true;
    }
}

/// Core banishment casting logic. Returns the number of units banished.
#[allow(clippy::too_many_arguments)]
fn banishment_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    enemies_query: &Query<
        (Entity, &Transform, &Team, &Health),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
    params: &BanishmentTalentParams,
    spell_range: f32,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
    local_origin: Vec3,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
    caster_team: Team,
) -> u32 {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return 0;
    }

    let mana_cost = if params.mass_banishment {
        constants::MASS_BANISHMENT_MANA_COST * params.mana_mult
    } else {
        constants::MANA_COST * params.mana_mult
    };
    let cast_time = primed_spell.cast_time * params.cast_time_mult;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(mana_cost) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if casting_state.is_complete(cast_time) {
                if let Some(cursor_pos) = input.cursor_pos
                    && mana.consume(mana_cost)
                {
                    let banished = if params.mass_banishment {
                        cast_mass_banishment(
                            commands,
                            enemies_query,
                            cursor_pos,
                            primed_spell.empowerment,
                            params,
                            spell_range,
                            visual_assets,
                            time_secs,
                            local_origin,
                            pending,
                            caster_team,
                        )
                    } else {
                        cast_single_banishment(
                            commands,
                            enemies_query,
                            cursor_pos,
                            primed_spell.empowerment,
                            mana,
                            params,
                            spell_range,
                            visual_assets,
                            time_secs,
                            local_origin,
                            pending,
                            caster_team,
                        )
                    };
                    casting_state.cancel();
                    return banished;
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    0
}
