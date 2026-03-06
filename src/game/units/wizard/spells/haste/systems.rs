use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::HasteIndicator;
use super::constants;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::HasteModifier;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{clamp_cursor_to_spell_range, get_cursor_world_position, spawn_circle_indicator};
use crate::config::GameConfig;

/// Local wizard haste casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_haste_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (
            Entity,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut HasteIndicator>,
    mut targets_query: Query<(Entity, &Transform, Option<&mut HasteModifier>), Without<Wizard>>,
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
    if primed_spell.spell != Spell::Haste {
        return;
    }

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_spell_range(input.cursor_pos, wizard.spell_range, constants::CIRCLE_RADIUS * primed_spell.empowerment);

    // Handle release -- clean up indicator
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

    let Some(clamped_cursor) = clamped_cursor else {
        return;
    };

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &visual_assets,
                    visual_assets.haste_indicator.clone(),
                    clamped_cursor,
                    constants::CIRCLE_RADIUS * primed_spell.empowerment,
                    constants::CIRCLE_Y_POSITION,
                )
                .insert(HasteIndicator::new(clamped_cursor, primed_spell.empowerment))
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            // Update indicator position
            if let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = clamped_cursor;
            }

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
                            audio::play_sfx(&mut commands, &sfx.haste_cast, indicator.position, &game_config, &sfx);
                            apply_haste_buff(
                                &mut commands,
                                indicator.position,
                                radius,
                                indicator.empowerment,
                                &mut targets_query,
                            );
                        }
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    completed = true;
                } else {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
        }
    }

    if completed {
        mouse_state.left_consumed = true;
    }
}


/// Applies haste buff to ALL units in radius (magic is indiscriminate).
pub(crate) fn apply_haste_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    empowerment: f32,
    targets: &mut Query<(Entity, &Transform, Option<&mut HasteModifier>), Without<Wizard>>,
) {
    let modifier = constants::HASTE_MODIFIER * empowerment;
    let duration = constants::HASTE_DURATION * empowerment;

    for (entity, transform, existing_haste) in targets.iter_mut() {
        let distance = transform.translation.distance(circle_pos);
        if distance <= radius {
            if let Some(mut haste) = existing_haste {
                // Refresh duration if already hasted
                haste.refresh(duration);
            } else {
                commands
                    .entity(entity)
                    .insert(HasteModifier::new(modifier, duration));
            }
        }
    }
}
