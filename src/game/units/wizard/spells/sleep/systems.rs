use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::SleepTalentParams;
use super::constants;
use crate::config::GameConfig;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    Comatose, Corpse, Health, MovementSpeed, NarcolepticWave, NightTerrors, SleepModifier,
    Sleepwalking, TargetingVelocity, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, build_wizard_input, cleanup_spell_caster, handle_spell_release,
    spawn_circle_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> SleepTalentParams {
    let mut params = SleepTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Sleep, 0);
    let t2 = talents.get_selection(Spell::Sleep, 1);
    let t3 = talents.get_selection(Spell::Sleep, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Deep Slumber: +40% duration
            params.duration_mult = constants::DEEP_SLUMBER_DURATION_MULT;
        }
        Some(1) => {
            // Lullaby: +40% radius
            params.radius_mult = constants::LULLABY_RADIUS_MULT;
        }
        Some(2) => {
            // Nightmare Fuel: +50% wake-up bonus damage
            params.bonus_damage_mult = constants::NIGHTMARE_FUEL_BONUS_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.narcoleptic_wave = true,
        Some(1) => params.night_terrors = true,
        Some(2) => {
            // Drowsy: halved cast time, -25% mana
            params.cast_time_mult = constants::DROWSY_CAST_TIME_MULT;
            params.mana_mult = constants::DROWSY_MANA_MULT;
        }
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.comatose = true,
        Some(1) => params.dreamwalker = true,
        Some(2) => params.eternal_slumber = true,
        _ => {}
    }

    params
}

/// Local wizard sleep casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_sleep_casting(
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
    targets_query: Query<(Entity, &Transform, &Health, &Team), Without<Corpse>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    talent_resources: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
    ),
) {
    let (active_talents, mut talent_progress) = talent_resources;
    let input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Sleep {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = sleep_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &targets_query,
        &mut commands,
        &visual_assets,
        &mut meshes,
        &sfx,
        &game_config,
        &talent_params,
        &mut talent_progress,
    );

    if completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            vfx::systems::SpellSchool::Dark,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core sleep casting logic.
#[allow(clippy::too_many_arguments)]
fn sleep_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    targets_query: &Query<(Entity, &Transform, &Health, &Team), Without<Corpse>>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    meshes: &mut Assets<Mesh>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: &SleepTalentParams,
    talent_progress: &mut Option<ResMut<BattleTalentProgress>>,
) -> bool {
    // Check for release event
    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = SPELL_ORIGIN;
    let wizard_height = wizard_pos.y;
    let max_ground_radius = if wizard_height < wizard.spell_range {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };
    let scale = primed_spell.empowerment;
    let effective_radius = constants::CIRCLE_RADIUS * scale * talent_params.radius_mult;
    let effective_mana_cost = constants::MANA_COST * talent_params.mana_mult;
    let effective_cast_time = primed_spell.cast_time * talent_params.cast_time_mult;
    let max_center_distance = (max_ground_radius - effective_radius).max(0.0);
    let direction = cursor_world_pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        cursor_world_pos = wizard_pos + normalized_direction * max_center_distance;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(effective_mana_cost)
            {
                let circle_entity = spawn_circle_indicator(
                    commands,
                    meshes,
                    assets.sleep_indicator.clone(),
                    cursor_world_pos,
                    effective_radius,
                )
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            update_indicator_position(
                wizard_entity,
                cursor_world_pos,
                caster_query,
                indicator_query,
            );
            if casting_state.is_complete(effective_cast_time) {
                if mana.consume(effective_mana_cost) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                        && let Ok(indicator) = indicator_query.get(indicator_entity)
                    {
                        audio::play_sfx(
                            commands,
                            &sfx.sleep_cast,
                            indicator.position,
                            game_config,
                            sfx,
                        );
                        vfx::systems::spawn_aura_bubble(
                            commands,
                            assets,
                            assets.sleep_aura_sphere.clone(),
                            indicator.position,
                            effective_radius,
                            2.5,
                        );
                        let hit_count = apply_sleep(
                            commands,
                            indicator.position,
                            effective_radius,
                            primed_spell.empowerment,
                            targets_query,
                            talent_params,
                        );
                        // Track talent progress
                        if hit_count > 0
                            && let Some(progress) = talent_progress
                        {
                            progress.increment(Spell::Sleep, hit_count);
                        }
                    }
                    completed = true;
                }
                cleanup_spell_caster(commands, wizard_entity, caster_query);
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(commands, wizard_entity, caster_query);
            casting_state.cancel();
        }
    }

    completed
}

/// Apply sleep to all targets in radius, returning the number of enemies hit.
pub(crate) fn apply_sleep(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    empowerment: f32,
    targets: &Query<(Entity, &Transform, &Health, &Team), Without<Corpse>>,
    talent_params: &SleepTalentParams,
) -> u32 {
    let duration = constants::SLEEP_DURATION * empowerment * talent_params.duration_mult;
    let bonus = constants::BONUS_DAMAGE_MULTIPLIER * talent_params.bonus_damage_mult;
    let mut hit_count = 0u32;

    for (entity, transform, health, team) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);
        if distance > radius {
            continue;
        }

        // Eternal Slumber: enemies below 25% HP are killed instantly
        if talent_params.eternal_slumber
            && *team != Team::Defenders
            && health.current <= health.max * constants::ETERNAL_SLUMBER_HP_THRESHOLD
        {
            // Set health to 0 to kill (death system will handle corpse conversion)
            commands.entity(entity).insert(Health {
                current: 0.0,
                max: health.max,
                spell_vulnerability: health.spell_vulnerability,
                healing_reduction: health.healing_reduction,
            });
            hit_count += 1;
            continue;
        }

        let mut modifier = if talent_params.dreamwalker {
            SleepModifier::new(constants::DREAMWALKER_DURATION, bonus)
        } else {
            SleepModifier::new(duration, bonus)
        };

        let mut entity_commands = commands.entity(entity);

        // Night Terrors: minor DPS while sleeping
        if talent_params.night_terrors {
            entity_commands.insert(NightTerrors::new(constants::NIGHT_TERRORS_DPS));
        }

        // Comatose: require 30% max HP damage to wake
        if talent_params.comatose {
            entity_commands.insert(Comatose::new(constants::COMATOSE_WAKE_THRESHOLD));
        }

        // Narcoleptic Wave: spreading sleep after delay
        if talent_params.narcoleptic_wave {
            entity_commands.insert(NarcolepticWave::new(
                constants::NARCOLEPTIC_SPREAD_DELAY,
                constants::NARCOLEPTIC_SPREAD_RADIUS,
            ));
        }

        // Dreamwalker: enemies sleepwalk back toward spawn
        if talent_params.dreamwalker {
            entity_commands.insert(Sleepwalking::new(constants::DREAMWALKER_SPEED_MULT));
            modifier.full_duration = constants::DREAMWALKER_DURATION;
        }

        entity_commands.insert(modifier);
        hit_count += 1;
    }

    hit_count
}

/// Tick sleep timer and remove when expired (along with all sub-components).
pub fn update_sleep_modifiers(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SleepModifier)>,
) {
    let delta = time.delta_secs();
    for (entity, mut sleep) in query.iter_mut() {
        if sleep.update(delta) {
            commands.entity(entity).remove::<(
                SleepModifier,
                NightTerrors,
                Comatose,
                NarcolepticWave,
                Sleepwalking,
            )>();
        }
    }
}

/// Night Terrors talent: apply DPS to sleeping units.
pub fn update_night_terrors(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut NightTerrors,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
    )>,
) {
    let delta = time.delta_secs();
    for (entity, mut terrors, mut health, mut temp_hp) in query.iter_mut() {
        terrors.tick_accumulator += delta;
        if terrors.tick_accumulator >= constants::NIGHT_TERRORS_TICK_INTERVAL {
            let damage = terrors.dps * terrors.tick_accumulator;
            apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
            terrors.tick_accumulator = 0.0;
            if health.current <= 0.0 {
                commands.entity(entity).insert(Corpse);
            }
        }
    }
}

/// Narcoleptic Wave: after the delay, spread sleep to nearby awake enemies.
pub fn update_narcoleptic_wave(
    time: Res<Time>,
    mut commands: Commands,
    mut wave_query: Query<(
        Entity,
        &mut NarcolepticWave,
        &SleepModifier,
        &Transform,
        &Team,
        Has<NightTerrors>,
        Has<Comatose>,
        Option<&Sleepwalking>,
    )>,
    awake_targets: Query<(Entity, &Transform, &Team), (Without<SleepModifier>, Without<Corpse>)>,
) {
    let delta = time.delta_secs();

    struct SpreadEvent {
        position: Vec3,
        radius: f32,
        duration: f32,
        bonus_damage: f32,
        has_night_terrors: bool,
        has_comatose: bool,
        sleepwalking: Option<f32>,
    }

    let mut spread_events: Vec<SpreadEvent> = Vec::new();

    for (entity, mut wave, sleep, transform, team, has_night_terrors, has_comatose, sleepwalking) in
        wave_query.iter_mut()
    {
        wave.timer -= delta;
        if wave.timer <= 0.0 {
            // Remove the component now that it's spread — avoids iterating every frame
            commands.entity(entity).remove::<NarcolepticWave>();
            // Only spread from non-defender sleepers
            if *team != Team::Defenders {
                spread_events.push(SpreadEvent {
                    position: transform.translation,
                    radius: wave.radius,
                    duration: sleep.full_duration * 0.5,
                    bonus_damage: sleep.bonus_damage_multiplier,
                    has_night_terrors,
                    has_comatose,
                    sleepwalking: sleepwalking.map(|s| s.speed_mult),
                });
            }
        }
    }

    // Apply spreads
    for event in spread_events {
        for (entity, transform, team) in awake_targets.iter() {
            if *team == Team::Defenders {
                continue;
            }
            let dist = transform.translation.distance(event.position);
            if dist <= event.radius {
                let modifier = SleepModifier::new(event.duration, event.bonus_damage);
                let mut entity_commands = commands.entity(entity);
                entity_commands.insert(modifier);
                if event.has_night_terrors {
                    entity_commands.insert(NightTerrors::new(constants::NIGHT_TERRORS_DPS));
                }
                if event.has_comatose {
                    entity_commands.insert(Comatose::new(constants::COMATOSE_WAKE_THRESHOLD));
                }
                if let Some(speed_mult) = event.sleepwalking {
                    entity_commands.insert(Sleepwalking::new(speed_mult));
                }
            }
        }
    }
}

/// Dreamwalker: sleepwalking units walk away from the castle (back toward spawn).
/// Overrides their targeting velocity so they move in the opposite direction of SPELL_ORIGIN.
pub fn update_sleepwalkers(
    mut query: Query<(
        &Transform,
        &Sleepwalking,
        &mut TargetingVelocity,
        &MovementSpeed,
    )>,
) {
    for (transform, sleepwalking, mut targeting, movement_speed) in query.iter_mut() {
        // Walk away from the castle (SPELL_ORIGIN)
        let away_dir = transform.translation - SPELL_ORIGIN;
        let horizontal = Vec3::new(away_dir.x, 0.0, away_dir.z);
        let length = horizontal.length();

        if length > 0.001 {
            let normalized = horizontal / length;
            targeting.velocity = normalized * movement_speed.0 * sleepwalking.speed_mult;
            // Set a large distance so flocking/flow fields don't override
            targeting.distance_to_target = 1000.0;
        }
    }
}
