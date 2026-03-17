use bevy::prelude::*;
use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::{
    BerserkerRageTalentParams, Bloodlust, ContagiousRage, FinalStand, FinalStandExplosionVfx,
    Frenzy, FrenzyActive, UndyingFury, UndyingFuryActive,
};
use super::constants;
use super::messages::ContagiousRageKillMessage;
use crate::config::GameConfig;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    BerserkerRageModifier, Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, clamp_cursor_to_spell_range, get_cursor_world_position,
    spawn_circle_indicator,
};
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::vfx;
use crate::game::components::OnGameplayScreen;

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

/// Local wizard berserker rage casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_berserker_rage_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    mut targets_query: Query<
        (Entity, &Transform, &Team, Option<&mut BerserkerRageModifier>),
        (Without<Wizard>, Without<Corpse>),
    >,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
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
    if primed_spell.spell != Spell::BerserkerRage {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let base_radius = constants::CIRCLE_RADIUS * talent_params.radius_mult;

    let clamped_cursor = clamp_cursor_to_spell_range(
        input.cursor_pos,
        wizard.spell_range,
        base_radius * primed_spell.empowerment,
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
                    &mut meshes,
                    visual_assets.berserker_rage_indicator.clone(),
                    pos,
                    base_radius * primed_spell.empowerment,
                )
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

    let completed = berserker_rage_casting_logic(
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
                let radius = base_radius * primed_spell.empowerment;
                let buffed_count = apply_berserker_rage_buff(
                    &mut commands,
                    indicator.position,
                    radius,
                    primed_spell.empowerment,
                    &talent_params,
                    &mut targets_query,
                );
                audio::play_sfx(
                    &mut commands,
                    &sfx.berserker_rage_cast,
                    indicator.position,
                    &game_config,
                    &sfx,
                );
                // Track talent progress
                if buffed_count > 0 {
                    if let Some(ref mut progress) = talent_progress {
                        progress.increment(Spell::BerserkerRage, buffed_count);
                    }
                }
            }
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Core berserker rage casting logic.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn berserker_rage_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    _clamped_cursor: Option<Vec3>,
) -> bool {
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

/// Applies the berserker rage buff to all units within the circle.
/// Returns the number of units buffed.
pub(crate) fn apply_berserker_rage_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    empowerment: f32,
    talent_params: &BerserkerRageTalentParams,
    targets: &mut Query<
        (Entity, &Transform, &Team, Option<&mut BerserkerRageModifier>),
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
        let distance = transform.translation.distance(circle_pos);
        if distance <= radius {
            if let Some(mut buff) = existing {
                buff.damage_bonus = damage_bonus;
                buff.damage_vulnerability = vulnerability;
                buff.refresh(duration);
            } else {
                commands
                    .entity(entity)
                    .insert(BerserkerRageModifier::new(damage_bonus, vulnerability, duration));
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

/// Undying Fury: prevent death for enraged units.
/// Runs after combat but before corpse conversion.
/// If a unit with UndyingFury has <= 0 HP, restores them to 1 HP
/// and starts the active protection timer.
pub fn undying_fury_trigger(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health), (With<UndyingFury>, Without<Corpse>)>,
) {
    for (entity, mut health) in &mut query {
        if health.is_dead() {
            health.current = 1.0;
            commands.entity(entity).remove::<UndyingFury>();
            commands.entity(entity).insert(UndyingFuryActive {
                time_remaining: constants::UNDYING_FURY_DURATION,
            });
        }
    }
}

/// Tick Undying Fury Active timer and enforce minimum 1 HP while active.
/// When timer expires, the unit can die normally.
pub fn tick_undying_fury_active(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut UndyingFuryActive, &mut Health), Without<Corpse>>,
) {
    let delta = time.delta_secs();
    for (entity, mut active, mut health) in &mut query {
        active.time_remaining -= delta;
        if active.time_remaining <= 0.0 {
            commands.entity(entity).remove::<UndyingFuryActive>();
        } else {
            // Enforce minimum 1 HP while active
            if health.current < 1.0 {
                health.current = 1.0;
            }
        }
    }
}

/// Frenzy: toggle FrenzyActive based on HP threshold.
/// Runs each frame for units with the Frenzy component.
pub fn frenzy_check_system(
    mut commands: Commands,
    query: Query<
        (Entity, &Health, &Frenzy, Option<&FrenzyActive>),
        (With<BerserkerRageModifier>, Without<Corpse>),
    >,
) {
    for (entity, health, frenzy, active) in &query {
        let below_threshold =
            health.max > 0.0 && health.current / health.max <= frenzy.hp_threshold;
        if below_threshold && active.is_none() {
            commands.entity(entity).insert(FrenzyActive);
        } else if !below_threshold && active.is_some() {
            commands.entity(entity).remove::<FrenzyActive>();
        }
    }
}

/// Contagious Rage: when an enraged unit kills an enemy, spread rage to the nearest calm ally.
pub fn contagious_rage_spread(
    mut commands: Commands,
    mut kill_events: MessageReader<ContagiousRageKillMessage>,
    killer_query: Query<(&Transform, &Team, &ContagiousRage), Without<Corpse>>,
    candidates: Query<
        (Entity, &Transform, &Team),
        (Without<BerserkerRageModifier>, Without<Corpse>, Without<Wizard>),
    >,
) {
    for event in kill_events.read() {
        let Ok((killer_pos, killer_team, rage_params)) = killer_query.get(event.killer) else {
            continue;
        };

        // Find nearest same-team ally without berserker rage
        let nearest = candidates
            .iter()
            .filter(|(_, _, team)| **team == *killer_team)
            .min_by(|(_, a_pos, _), (_, b_pos, _)| {
                let da = a_pos.translation.distance_squared(killer_pos.translation);
                let db = b_pos.translation.distance_squared(killer_pos.translation);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((target_entity, _, _)) = nearest {
            // Apply rage with reduced effectiveness
            let effectiveness =
                1.0 - constants::CONTAGIOUS_RAGE_EFFECTIVENESS_LOSS;
            commands.entity(target_entity).insert(BerserkerRageModifier::new(
                rage_params.damage_bonus * effectiveness,
                rage_params.vulnerability * effectiveness,
                rage_params.duration * effectiveness,
            ));
            // Spread the ContagiousRage component so kills by the new unit also spread
            commands.entity(target_entity).insert(ContagiousRage {
                damage_bonus: rage_params.damage_bonus * effectiveness,
                vulnerability: rage_params.vulnerability * effectiveness,
                duration: rage_params.duration * effectiveness,
            });
        }
    }
}

/// Final Stand: when an enraged unit dies, explode for AoE damage.
/// Queries corpses with FinalStand and applies damage to nearby enemies.
/// Spawns a fireball explosion visual at the death location.
pub fn final_stand_explosion(
    mut commands: Commands,
    dead_query: Query<(Entity, &FinalStand, &Transform, &Team, &Health), With<Corpse>>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
) {
    for (corpse_entity, final_stand, transform, team, health) in &dead_query {
        // Damage = fraction of the dead unit's max HP
        let explosion_damage = health.max * final_stand.damage_fraction;
        let position = transform.translation;

        for (target_entity, target_pos, target_team, mut target_health, temp_hp) in
            targets.iter_mut()
        {
            if *target_team == *team {
                continue;
            }
            if target_pos.translation.distance(position) <= final_stand.radius {
                apply_spell_damage(
                    &mut commands,
                    target_entity,
                    &mut target_health,
                    temp_hp.map(|t| t.into_inner()),
                    explosion_damage,
                    DamageType::Force,
                    false,
                );
            }
        }

        // Spawn fireball explosion visual
        commands.spawn((
            FinalStandExplosionVfx {
                time_alive: 0.0,
                max_radius: final_stand.radius,
                lifetime: constants::FINAL_STAND_VFX_LIFETIME,
            },
            Mesh3d(visual_assets.cross_plane_sphere.clone()),
            MeshMaterial3d(visual_assets.fireball_explosion.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
            OnGameplayScreen,
        ));

        // Fire sparks + smoke burst
        let time_secs = time.elapsed_secs();
        vfx::systems::spawn_fire_sparks(
            &mut commands,
            &visual_assets,
            position,
            constants::FINAL_STAND_SPARK_COUNT,
            time_secs,
        );
        vfx::systems::spawn_explosion_smoke(&mut commands, &visual_assets, position, time_secs);

        // One-shot: remove marker so explosion doesn't fire again
        commands.entity(corpse_entity).remove::<FinalStand>();
    }
}

/// Updates Final Stand explosion visuals: expand then despawn.
pub fn update_final_stand_vfx(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FinalStandExplosionVfx, &mut Transform)>,
) {
    for (entity, mut vfx, mut transform) in &mut query {
        vfx.time_alive += time.delta_secs();
        if vfx.time_alive >= vfx.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }
        let progress = (vfx.time_alive / vfx.lifetime).min(1.0);
        let current_radius = vfx.max_radius * progress;
        transform.scale = Vec3::splat(current_radius.max(0.1));
    }
}

/// Clean up berserker rage talent components when the base modifier is removed.
/// This handles the case where the buff expires naturally.
pub fn cleanup_berserker_rage_talents(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            Without<BerserkerRageModifier>,
            Or<(
                With<Bloodlust>,
                With<Frenzy>,
                With<FrenzyActive>,
                With<UndyingFury>,
                With<UndyingFuryActive>,
                With<ContagiousRage>,
                With<FinalStand>,
            )>,
            Without<Corpse>,
        ),
    >,
) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<Bloodlust>()
            .remove::<Frenzy>()
            .remove::<FrenzyActive>()
            .remove::<UndyingFury>()
            .remove::<UndyingFuryActive>()
            .remove::<ContagiousRage>()
            .remove::<FinalStand>();
    }
}
