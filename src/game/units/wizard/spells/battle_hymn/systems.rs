use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::BattleHymnIndicator;
use super::constants;
use crate::config::GameConfig;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{BattleHymnModifier, HasteModifier, Team, TemporaryHitPoints};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    clamp_cursor_to_spell_range, get_cursor_world_position, spawn_circle_indicator,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Local wizard battle hymn casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_battle_hymn_casting(
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
    mut indicator_query: Query<&mut BattleHymnIndicator>,
    mut targets_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut BattleHymnModifier>,
            Option<&mut TemporaryHitPoints>,
            Option<&mut HasteModifier>,
        ),
        Without<Wizard>,
    >,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    active_talents: Option<Res<ActiveTalents>>,
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
    if primed_spell.spell != Spell::BattleHymn {
        return;
    }

    // Calculate talent modifications
    let talents = active_talents.as_deref();
    let t1 = talents.and_then(|t| t.get_selection(Spell::BattleHymn, 0));
    let t3 = talents.and_then(|t| t.get_selection(Spell::BattleHymn, 2));
    let radius_mult = match t1 {
        Some(2) => 1.4, // Wide Anthem: +40% radius
        _ => 1.0,
    };
    let chorus_of_valor = t3 == Some(2);
    let mana_cost = if chorus_of_valor {
        constants::MANA_COST * 2.0
    } else {
        constants::MANA_COST
    };

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
                && mana.can_afford(mana_cost)
                && let Some(pos) = clamped_cursor
            {
                if chorus_of_valor {
                    // No indicator for Chorus of Valor (affects all defenders)
                    commands.entity(wizard_entity).insert(SpellCaster::new());
                } else {
                    let mut indicator = BattleHymnIndicator::new(pos, primed_spell.empowerment);
                    indicator.talent_radius_mult = radius_mult;
                    let circle_entity = spawn_circle_indicator(
                        &mut commands,
                        &visual_assets,
                        visual_assets.battle_hymn_indicator.clone(),
                        pos,
                        constants::CIRCLE_RADIUS * primed_spell.empowerment * radius_mult,
                        constants::CIRCLE_Y_POSITION,
                    )
                    .insert(indicator)
                    .id();
                    commands
                        .entity(wizard_entity)
                        .insert(SpellCaster::with_indicator(circle_entity));
                }
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

    let completed = battle_hymn_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        clamped_cursor,
        mana_cost,
    );

    if completed {
        if chorus_of_valor {
            // Chorus of Valor: no indicator, buff all defenders from wizard position
            apply_battle_hymn_buff(
                &mut commands,
                SPELL_ORIGIN,
                0.0, // radius unused since Chorus of Valor ignores radius
                primed_spell.empowerment,
                &mut targets_query,
                &mut talent_progress,
                talents,
            );
            audio::play_sfx(
                &mut commands,
                &sfx.battle_hymn_cast,
                SPELL_ORIGIN,
                &game_config,
                &sfx,
            );
        } else if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                let radius =
                    constants::CIRCLE_RADIUS * indicator.empowerment * indicator.talent_radius_mult;
                apply_battle_hymn_buff(
                    &mut commands,
                    indicator.position,
                    radius,
                    indicator.empowerment,
                    &mut targets_query,
                    &mut talent_progress,
                    talents,
                );
                audio::play_sfx(
                    &mut commands,
                    &sfx.battle_hymn_cast,
                    indicator.position,
                    &game_config,
                    &sfx,
                );
            }
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Core battle hymn casting logic.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn battle_hymn_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    _clamped_cursor: Option<Vec3>,
    mana_cost: f32,
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
                if mana.consume(mana_cost) {
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

pub fn update_battle_hymn_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut BattleHymnIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();
        let radius =
            constants::CIRCLE_RADIUS * indicator.empowerment * indicator.talent_radius_mult;
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(radius * pulse);
        transform.translation.x = indicator.position.x;
        transform.translation.y = constants::CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

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
        Without<Wizard>,
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
        Some(0) => duration *= 1.5,     // Inspiring Words: +50% duration
        Some(1) => damage_bonus *= 1.5, // War Drums: +50% damage bonus
        // Wide Anthem radius is already applied via indicator.talent_radius_mult
        _ => {}
    }

    // Tier 3: Hymn of Legends doubles both bonuses (applied before Tier 2 adds effects)
    if t3 == Some(0) {
        damage_bonus *= 2.0;
        attack_speed *= 2.0;
    }

    // Tier 2 echo duration
    let echo_duration = if t2 == Some(1) { duration * 0.5 } else { 0.0 }; // Echoing Song

    // Tier 3 damage reduction
    let damage_reduction = if t3 == Some(1) { 0.3 } else { 0.0 }; // Anthem of Resilience

    // Tier 3: Chorus of Valor ignores radius (buff all defenders)
    let ignore_radius = t3 == Some(2);

    let mut buffed_count = 0u32;
    for (entity, transform, team, existing, existing_temp_hp, existing_haste) in targets.iter_mut()
    {
        let in_range = if ignore_radius {
            // Chorus of Valor: only buff defenders, but ignore radius
            *team == Team::Defenders
        } else {
            let distance = transform.translation.distance(circle_pos);
            distance <= radius
        };

        if in_range {
            if let Some(mut buff) = existing {
                buff.damage_bonus = damage_bonus;
                buff.attack_speed = attack_speed;
                buff.echo_duration = echo_duration;
                buff.damage_reduction = damage_reduction;
                buff.refresh(duration);
            } else {
                let mut modifier = BattleHymnModifier::new(damage_bonus, attack_speed, duration);
                modifier.echo_duration = echo_duration;
                modifier.damage_reduction = damage_reduction;
                commands.entity(entity).insert(modifier);
            }

            // Tier 2: Fortifying Hymn grants 20 temporary HP
            if t2 == Some(0) {
                let temp_hp_amount = 20.0 * empowerment;
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

            // Tier 2: Swift March grants 25% movement speed
            if t2 == Some(2) {
                let speed_bonus = 0.25;
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
