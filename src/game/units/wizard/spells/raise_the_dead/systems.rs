use std::cmp::Ordering;

use bevy::prelude::*;
use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, WizardInput,
};
use super::components::*;
use super::constants::{
    CHANNEL_RAMP_TIME, INITIAL_CHANNEL_INTERVAL, MANA_COST_PER_CORPSE, MIN_CHANNEL_INTERVAL,
    RESURRECTION_RADIUS,
};
use crate::config::GameConfig;
use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, PermanentCorpse, Team};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::infantry::styles::UNDEAD_SPRITE_TINT;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::game::crt_effect::CorrectedCursorPosition;

/// Local wizard Raise The Dead casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_raise_the_dead_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<(&mut CastingState, &mut Mana, &PrimedSpell), With<LocalWizard>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    corpse_query: Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    infantry_assets: Res<InfantryAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::RaiseTheDead {
        return;
    }

    raise_the_dead_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &corpse_query,
        &infantry_assets,
        &mut materials,
        &sfx,
        &game_config,
    );
}

/// Core Raise The Dead casting logic.
///
/// Handles the full Resting -> Casting -> Channeling state machine.
/// During channeling, resurrects corpses at increasing frequency.
#[allow(clippy::too_many_arguments)]
fn raise_the_dead_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    corpse_query: &Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
) {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());

            if casting_state.should_channel(
                INITIAL_CHANNEL_INTERVAL,
                MIN_CHANNEL_INTERVAL,
                CHANNEL_RAMP_TIME,
            ) {
                if mana.consume(MANA_COST_PER_CORPSE) {
                    if let Some(cursor_pos) = input.cursor_pos {
                        resurrect_nearest_corpse(
                            commands,
                            cursor_pos,
                            corpse_query,
                            infantry_assets,
                            materials,
                            primed_spell.empowerment,
                        );
                    }
                    casting_state.reset_channel_interval();
                } else {
                    casting_state.cancel();
                }
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(MANA_COST_PER_CORPSE) {
                    if let Some(cursor_pos) = input.cursor_pos {
                        audio::play_sfx(
                            commands,
                            &sfx.raise_the_dead_cast,
                            cursor_pos,
                            game_config,
                            sfx,
                        );
                        resurrect_nearest_corpse(
                            commands,
                            cursor_pos,
                            corpse_query,
                            infantry_assets,
                            materials,
                            primed_spell.empowerment,
                        );
                    }
                    casting_state.start_channeling();
                } else {
                    casting_state.cancel();
                }
            }
        }
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(MANA_COST_PER_CORPSE) {
                casting_state.start_cast();
            }
        }
    }
}

/// Resurrects the nearest corpse to the target position as undead infantry.
///
/// Searches for corpses within RESURRECTION_RADIUS and resurrects the closest one.
/// Uses the shared `resurrect_corpse_as_infantry` helper.
fn resurrect_nearest_corpse(
    commands: &mut Commands,
    target_pos: Vec3,
    corpse_query: &Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    empowerment: f32,
) {
    // Find nearest corpse within radius
    if let Some((corpse_entity, corpse_transform)) = corpse_query
        .iter()
        .filter(|(_, transform)| target_pos.distance(transform.translation) <= RESURRECTION_RADIUS)
        .min_by(|a, b| {
            let dist_a = target_pos.distance(a.1.translation);
            let dist_b = target_pos.distance(b.1.translation);
            dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
        })
    {
        let health = UNIT_HEALTH * empowerment;
        let speed = UNIT_MOVEMENT_SPEED * 0.5 * empowerment;

        crate::game::units::systems::resurrect_corpse_as_infantry(
            commands,
            corpse_entity,
            corpse_transform.translation,
            Team::Undead,
            health,
            speed,
            UNDEAD_SPRITE_TINT,
            infantry_assets,
            materials,
        );

        // Add RaisedUndead marker for tracking
        commands.entity(corpse_entity).insert(RaisedUndead);

        // Apply empowerment bonus if applicable
        if empowerment > 1.0 {
            let mut effectiveness = crate::game::units::components::Effectiveness::new();
            effectiveness.spell_bonus = 0.25;
            commands.entity(corpse_entity).insert(effectiveness);
        }
    }
}
