use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::GuardianCircleIndicator;
use super::constants;
use crate::config::GameConfig;
use crate::game::achievements::messages::GuardianCircleHitAttackerMessage;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Team, TemporaryHitPoints};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    clamp_cursor_to_spell_range, get_cursor_world_position, spawn_circle_indicator,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Local wizard Guardian Circle casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_guardian_circle_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut GuardianCircleIndicator>,
    mut targets_query: Query<(Entity, &Transform, &Team), Without<Wizard>>,
    mut attacker_hit_msg: MessageWriter<GuardianCircleHitAttackerMessage>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::GuardianCircle {
        return;
    }

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_spell_range(
        input.cursor_pos,
        wizard.spell_range,
        constants::CIRCLE_RADIUS * primed_spell.empowerment,
    );

    // Handle release -- clean up indicator and SpellCaster
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).try_despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    // Manage indicator based on casting state
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
                && let Some(pos) = clamped_cursor
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &visual_assets,
                    visual_assets.guardian_circle_indicator.clone(),
                    pos,
                    constants::CIRCLE_RADIUS * primed_spell.empowerment,
                    constants::CIRCLE_Y_POSITION,
                )
                .insert(GuardianCircleIndicator::new(pos, primed_spell.empowerment))
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
            }
        }
        CastingState::Casting { .. } => {
            if let Some(pos) = clamped_cursor
                && let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = pos;
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
        }
    }

    let completed = guardian_circle_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        clamped_cursor,
    );

    if completed {
        // Apply buff using indicator position
        if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                let scale = indicator.empowerment;
                let radius = constants::CIRCLE_RADIUS * scale;

                audio::play_sfx(
                    &mut commands,
                    &sfx.guardian_circle_cast,
                    indicator.position,
                    &game_config,
                    &sfx,
                );
                apply_guardian_circle_buff(
                    &mut commands,
                    indicator.position,
                    radius,
                    constants::TEMP_HP_AMOUNT,
                    constants::TEMP_HP_DURATION,
                    indicator.empowerment,
                    &mut targets_query,
                    &mut attacker_hit_msg,
                );
            }
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Core Guardian Circle casting logic.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn guardian_circle_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    _clamped_cursor: Option<Vec3>,
) -> bool {
    // Release is handled by the wrappers before calling this function
    if input.just_released {
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                casting_state.start_cast();
            }
        }
    }

    completed
}

/// Helper function to apply Guardian Circle buff to all units in radius.
///
/// Grants temporary HP to units. If a unit already has temp HP, takes the maximum.
/// Scales temp HP amount and duration by 1.25x when empowered.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_guardian_circle_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    temp_hp_amount: f32,
    duration: f32,
    empowerment: f32,
    targets: &mut Query<(Entity, &Transform, &Team), Without<Wizard>>,
    attacker_hit_msg: &mut MessageWriter<GuardianCircleHitAttackerMessage>,
) {
    // Scale values by empowerment
    let scale = empowerment;
    let scaled_temp_hp = temp_hp_amount * scale;
    let scaled_duration = duration * scale;

    for (entity, transform, team) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);

        if distance <= radius {
            // Unit is in range - add or update TemporaryHitPoints
            commands
                .entity(entity)
                .insert(TemporaryHitPoints::new(scaled_temp_hp, scaled_duration));

            // Protective Instincts: Guardian Circle hit an attacker or undead
            if *team == Team::Attackers || *team == Team::Undead {
                attacker_hit_msg.write(GuardianCircleHitAttackerMessage);
            }
        }
    }
}
