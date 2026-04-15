//! Black Hole spell systems.

use super::components::{BlackHole, BlackHoleSfx, BlackHoleTalentParams, UnitInBlackHole};
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::{Acceleration, OnGameplayScreen};
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::{
    Corpse, Health, SlowMovementModifier, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    PendingDefenderHeal, SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist,
    build_wizard_input, clamp_to_spell_range, cleanup_spell_caster, spawn_circle_indicator,
    update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

type DimensionalRiftUnitData = (
    Entity,
    &'static mut Transform,
    &'static mut Health,
    Option<&'static mut TemporaryHitPoints>,
    Has<SpellShield>,
);
type DimensionalRiftUnitFilter = (
    With<Team>,
    Without<Wizard>,
    Without<Corpse>,
    Without<BlackHole>,
);

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned/skipped).
    completed: bool,
    /// Cursor position at time of completion (for network message).
    cursor_pos: Option<Vec3>,
}

/// Compute talent parameters from active talent selections.
fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> BlackHoleTalentParams {
    let mut params = BlackHoleTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::BlackHole, 0);
    let t2 = talents.get_selection(Spell::BlackHole, 1);
    let t3 = talents.get_selection(Spell::BlackHole, 2);

    // Tier 1
    match t1 {
        Some(0) => params.gravity_mult = DENSER_CORE_GRAVITY_MULT,
        Some(1) => {
            params.radius_mult = EXPANSIVE_VOID_RADIUS_MULT;
            params.damage_mult = EXPANSIVE_VOID_DAMAGE_MULT;
        }
        // Some(2) Quick Collapse: handled at cast time, not stored in params
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => params.event_horizon = true,
        Some(1) => params.crushing_pressure = true,
        Some(2) => params.void_siphon = true,
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => params.singularity = true,
        // Some(1) Twin Stars: handled at spawn time
        Some(2) => params.dimensional_rift = true,
        _ => {}
    }

    params
}

/// Spawns a black hole entity (solid black icosphere) with a looping sound.
pub(crate) fn spawn_black_hole(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: BlackHoleTalentParams,
) {
    let max_radius = MAX_RADIUS * empowerment * talent_params.radius_mult;
    let spawn_pos = Vec3::new(position.x, BLACK_HOLE_HEIGHT, position.z);

    let black_hole_entity = commands
        .spawn((
            BlackHole::new(spawn_pos, max_radius, empowerment, talent_params),
            Mesh3d(assets.black_hole_sphere.clone()),
            MeshMaterial3d(assets.black_hole.clone()),
            Transform::from_translation(spawn_pos).with_scale(Vec3::ZERO),
            NetworkedSpellEffect {
                kind: SpellEffectKind::BlackHole,
            },
            OnGameplayScreen,
        ))
        .id();

    // Looping sound effect attenuated by distance from wizard to black hole
    let sfx_entity = audio::play_looping_sfx_at(
        commands,
        &sfx.black_hole_persistent,
        spawn_pos,
        game_config,
        sfx,
    );
    commands
        .entity(sfx_entity)
        .insert(BlackHoleSfx { black_hole_entity });
}

/// Local wizard Black Hole casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_black_hole_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (Entity, &mut CastingState, &mut Mana, &PrimedSpell, &Wizard),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    target_assist: Res<TargetAssistWorldPos>,
) {
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::BlackHole {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let indicator_radius = MAX_RADIUS * primed_spell.empowerment * talent_params.radius_mult;

    // Check Twin Stars talent for mana cost multiplier
    let t3 = active_talents
        .as_deref()
        .and_then(|t| t.get_selection(Spell::BlackHole, 2));
    let mana_mult = if t3 == Some(1) {
        TWIN_STARS_MANA_MULT
    } else {
        1.0
    };

    let total_mana_cost = MANA_COST * mana_mult;

    // Clamp cursor to spell range for indicator positioning
    let clamped_cursor = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, SPELL_ORIGIN, wizard.spell_range));

    // Handle release -- clean up indicator and SpellCaster
    if input.just_released {
        cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
    }

    // Manage indicator based on casting state
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err()
                && mana.can_afford(total_mana_cost)
                && let Some(pos) = clamped_cursor
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.black_hole_indicator.clone(),
                    pos,
                    indicator_radius,
                )
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
            }
        }
        CastingState::Casting { .. } => {
            if let Some(pos) = clamped_cursor {
                update_indicator_position(wizard_entity, pos, &caster_query, &mut indicator_query);
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
        }
    }

    let cast_result = black_hole_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        wizard,
        mana_mult,
    );

    if cast_result.completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            vfx::systems::SpellSchool::Arcane,
            time.elapsed_secs(),
        );
        // Clean up indicator and SpellCaster
        cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);

        if let Some(pos) = cast_result.cursor_pos {
            if t3 == Some(1) {
                // Twin Stars: spawn 2 black holes offset from the target position
                let offset = Vec3::new(TWIN_STARS_OFFSET / 2.0, 0.0, 0.0);
                let emp = primed_spell.empowerment * TWIN_STARS_EFFECTIVENESS;
                spawn_black_hole(
                    &mut commands,
                    &visual_assets,
                    pos - offset,
                    emp,
                    &sfx,
                    &game_config,
                    talent_params,
                );
                spawn_black_hole(
                    &mut commands,
                    &visual_assets,
                    pos + offset,
                    emp,
                    &sfx,
                    &game_config,
                    talent_params,
                );
            } else {
                spawn_black_hole(
                    &mut commands,
                    &visual_assets,
                    pos,
                    primed_spell.empowerment,
                    &sfx,
                    &game_config,
                    talent_params,
                );
            }
        }
        mouse_state.left_consumed = true;
    }
}

/// Core Black Hole casting logic -- called by the local system.
///
/// Handles CastingState transitions, mana consumption, and cursor clamping.
/// Does NOT spawn the black hole or manage mouse_state -- those are the wrapper's job.
fn black_hole_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    mana_mult: f32,
) -> CastResult {
    let mut result = CastResult {
        completed: false,
        cursor_pos: None,
    };

    let total_mana_cost = MANA_COST * mana_mult;

    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return result;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(total_mana_cost) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if let Some(cursor_pos) = input.cursor_pos {
                    let clamped_pos =
                        clamp_to_spell_range(cursor_pos, SPELL_ORIGIN, wizard.spell_range);

                    if mana.consume(total_mana_cost) {
                        result.completed = true;
                        result.cursor_pos = Some(clamped_pos);
                    }
                }

                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    result
}

/// Applies gravitational forces to all living units near black holes.
///
/// Gravitational pull increases over time (0-5 seconds) AND with proximity.
/// Uses inverse square law so units cannot escape when close enough.
/// Forces are applied to acceleration, which can override normal movement limits.
/// This system runs in MovementCalculationSet, after unit-specific movement calculations.
pub(super) fn apply_gravitational_forces(
    mut black_holes: Query<&mut BlackHole>,
    mut units: Query<
        (&Transform, &mut Acceleration),
        (
            With<Team>,
            Without<Wizard>,
            Without<BlackHole>,
            Without<Corpse>,
        ),
    >,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for mut black_hole in black_holes.iter_mut() {
        // Update black hole timers
        black_hole.update_timers(delta);

        let gravity_strength = black_hole.gravitational_strength();
        let bh_pos = black_hole.position;

        for (transform, mut acceleration) in units.iter_mut() {
            let unit_pos = transform.translation;
            let to_black_hole = bh_pos - unit_pos;
            let distance = to_black_hole.length();

            // Only apply forces within gravity range and avoid division by zero
            if distance > 0.01 && distance <= GRAVITY_RANGE {
                // Use inverse square law for realistic gravity that grows stronger with proximity
                let distance_factor = 1.0 / (distance * distance);
                let pull_strength = (gravity_strength * distance_factor).min(MAX_FORCE_CLAMP);
                let direction = to_black_hole.normalize();

                // Apply gravitational force to acceleration
                // This will be integrated into velocity and applied to transform by apply_unit_movement
                let force = direction * pull_strength;
                acceleration.add_force(force);
            }
        }
    }
}

/// Applies gravitational forces to corpses and despawns them if they touch the black hole.
///
/// Corpses are pulled by the same gravitational forces as living units.
/// When a corpse intersects the black hole sphere, it is despawned.
pub(super) fn apply_corpse_gravity_and_despawn(
    mut commands: Commands,
    mut black_holes: Query<&BlackHole>,
    mut corpses: Query<(Entity, &Transform, &mut Acceleration), With<Corpse>>,
) {
    for black_hole in black_holes.iter_mut() {
        let gravity_strength = black_hole.gravitational_strength();
        let bh_pos = black_hole.position;

        for (entity, transform, mut acceleration) in corpses.iter_mut() {
            let corpse_pos = transform.translation;
            let to_black_hole = bh_pos - corpse_pos;
            let distance = to_black_hole.length();

            // Check if corpse intersects the black hole sphere - if so, despawn it
            if black_hole.contains_point(corpse_pos) {
                commands.entity(entity).try_despawn();
                continue;
            }

            // Apply gravitational forces within range
            if distance > 0.01 && distance <= GRAVITY_RANGE {
                // Use inverse square law for realistic gravity that grows stronger with proximity
                let distance_factor = 1.0 / (distance * distance);
                let pull_strength = (gravity_strength * distance_factor).min(MAX_FORCE_CLAMP);
                let direction = to_black_hole.normalize();

                // Apply gravitational force to acceleration
                let force = direction * pull_strength;
                acceleration.add_force(force);
            }
        }
    }
}

/// Applies damage to units touching the black hole sphere.
///
/// Damage increases over time for units that remain in contact.
/// Supports Event Horizon (double damage in inner zone) and Void Siphon (healing).
pub(super) fn apply_black_hole_damage(
    time: Res<Time>,
    mut commands: Commands,
    mut black_holes: Query<&mut BlackHole>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Option<&mut UnitInBlackHole>,
            Has<SpellShield>,
        ),
        Without<Wizard>,
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let mut total_siphon_heal = 0.0;
    let mut siphon_origin = Vec3::ZERO;

    for mut black_hole in black_holes.iter_mut() {
        if !black_hole.should_damage() {
            continue;
        }

        let bh_pos = black_hole.position;

        for (entity, transform, mut health, mut temp_hp, tracking, has_spell_shield) in
            units.iter_mut()
        {
            let unit_pos = transform.translation;

            if black_hole.contains_point(unit_pos) {
                if has_spell_shield {
                    continue;
                }

                // Track or update time inside
                let damage_multiplier = if let Some(mut tracker) = tracking {
                    tracker.time_inside += time.delta_secs();
                    tracker.damage_multiplier()
                } else {
                    commands.entity(entity).insert(UnitInBlackHole::new());
                    1.0 // First tick, no multiplier yet
                };

                // Event Horizon: double damage in inner zone
                let event_horizon_mult = if black_hole.talent_params.event_horizon {
                    let dist = bh_pos.distance(unit_pos);
                    let inner_radius = black_hole.current_radius * EVENT_HORIZON_INNER_FRACTION;
                    if dist <= inner_radius {
                        EVENT_HORIZON_DAMAGE_MULT
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

                // Apply scaled damage
                let total_damage =
                    black_hole.damage_per_tick() * damage_multiplier * event_horizon_mult;
                apply_spell_damage(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    total_damage,
                    DamageType::Force,
                    false,
                );

                // Accumulate siphon healing
                if black_hole.talent_params.void_siphon {
                    total_siphon_heal += total_damage * VOID_SIPHON_HEAL_FRACTION;
                    siphon_origin = bh_pos;
                }
            }
        }

        // Track talent progress
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::BlackHole, 1);
        }

        black_hole.reset_damage_timer();
    }

    // Defer siphon healing to a separate system to avoid query conflicts
    if total_siphon_heal > 0.0 {
        commands.insert_resource(PendingDefenderHeal {
            amount: total_siphon_heal,
            origin: siphon_origin,
        });
    }
}

/// Removes tracking component when units leave the black hole.
pub(super) fn remove_units_from_black_hole(
    mut commands: Commands,
    black_holes: Query<&BlackHole>,
    units: Query<(Entity, &Transform), With<UnitInBlackHole>>,
) {
    for (entity, transform) in units.iter() {
        let unit_pos = transform.translation;
        let mut is_in_any_black_hole = false;

        for black_hole in black_holes.iter() {
            if black_hole.contains_point(unit_pos) {
                is_in_any_black_hole = true;
                break;
            }
        }

        if !is_in_any_black_hole {
            commands.entity(entity).remove::<UnitInBlackHole>();
        }
    }
}

/// Applies Crushing Pressure slow to units inside the black hole.
pub(super) fn apply_crushing_pressure(
    mut commands: Commands,
    black_holes: Query<&BlackHole>,
    units: Query<(Entity, &Transform), (With<Team>, Without<Wizard>, Without<Corpse>)>,
) {
    // Only run if any black hole has crushing pressure
    let has_crushing = black_holes
        .iter()
        .any(|bh| bh.talent_params.crushing_pressure);
    if !has_crushing {
        return;
    }

    for (entity, transform) in units.iter() {
        let unit_pos = transform.translation;
        let mut inside_crushing = false;

        for black_hole in black_holes.iter() {
            if black_hole.talent_params.crushing_pressure && black_hole.contains_point(unit_pos) {
                inside_crushing = true;
                break;
            }
        }

        if inside_crushing {
            commands.entity(entity).insert(SlowMovementModifier::new(
                CRUSHING_PRESSURE_SLOW,
                CRUSHING_PRESSURE_SLOW_DURATION,
            ));
        }
    }
}

/// Dimensional Rift: periodically teleports all enemies inside to the center and deals burst damage.
pub(super) fn apply_dimensional_rift(
    mut commands: Commands,
    mut black_holes: Query<&mut BlackHole>,
    mut units: Query<DimensionalRiftUnitData, DimensionalRiftUnitFilter>,
) {
    for mut black_hole in black_holes.iter_mut() {
        if !black_hole.talent_params.dimensional_rift {
            continue;
        }

        if black_hole.time_since_rift_pulse < DIMENSIONAL_RIFT_INTERVAL {
            continue;
        }

        black_hole.time_since_rift_pulse = 0.0;
        let bh_pos = black_hole.position;

        for (entity, mut transform, mut health, mut temp_hp, has_spell_shield) in units.iter_mut() {
            if has_spell_shield {
                continue;
            }

            if black_hole.contains_point(transform.translation) {
                // Teleport to center (keep Y)
                transform.translation.x = bh_pos.x;
                transform.translation.z = bh_pos.z;

                // Deal burst damage
                apply_spell_damage(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    DIMENSIONAL_RIFT_DAMAGE * black_hole.empowerment,
                    DamageType::Force,
                    false,
                );
            }
        }
    }
}

/// Updates black hole visual scale to match growth animation, adds vibration effect,
/// billboards the circle to face the wizard, and applies pulsing.
pub(super) fn update_black_hole_visuals(
    time: Res<Time>,
    mut black_holes: Query<(&BlackHole, &mut Transform)>,
) {
    for (black_hole, mut transform) in black_holes.iter_mut() {
        let growth_factor = (black_hole.time_alive / GROWTH_TIME).min(1.0);

        // Add vibration using sine waves on different axes
        let t = black_hole.time_alive * VIBRATION_FREQUENCY;
        let vibration = Vec3::new(
            (t * 1.0).sin() * VIBRATION_AMPLITUDE,
            (t * 1.7).sin() * VIBRATION_AMPLITUDE,
            (t * 2.3).sin() * VIBRATION_AMPLITUDE,
        );

        // Pulsing scale
        let pulse = 1.0
            + (time.elapsed_secs() * RING_PULSE_FREQUENCY * std::f32::consts::TAU).sin()
                * RING_PULSE_AMPLITUDE;

        let position = black_hole.position + vibration;
        transform.scale = Vec3::splat(black_hole.max_radius * growth_factor * pulse);
        transform.translation = position;
    }
}

/// Despawns black holes when they expire after LIFETIME seconds.
/// Handles Singularity talent: deals collapse damage to all units inside on expiration.
pub(super) fn despawn_expired_black_holes(
    mut commands: Commands,
    black_holes: Query<(Entity, &BlackHole)>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Wizard>, Without<BlackHole>),
    >,
) {
    for (entity, black_hole) in black_holes.iter() {
        if black_hole.is_expired() {
            // Singularity: collapse damage to all units inside
            if black_hole.talent_params.singularity {
                for (unit_entity, transform, mut health, mut temp_hp, has_spell_shield) in
                    units.iter_mut()
                {
                    if has_spell_shield {
                        continue;
                    }
                    if black_hole.contains_point(transform.translation) {
                        apply_spell_damage(
                            &mut commands,
                            unit_entity,
                            &mut health,
                            temp_hp.as_deref_mut(),
                            SINGULARITY_DAMAGE * black_hole.empowerment,
                            DamageType::Force,
                            false,
                        );
                    }
                }
            }

            commands.entity(entity).try_despawn();
        }
    }
}

/// Despawns orphaned black hole sound effects whose parent no longer exists.
pub(super) fn cleanup_black_hole_sfx(
    mut commands: Commands,
    sfx_entities: Query<(Entity, &BlackHoleSfx)>,
    black_holes: Query<&BlackHole>,
) {
    for (entity, sfx) in sfx_entities.iter() {
        if black_holes.get(sfx.black_hole_entity).is_err() {
            commands.entity(entity).try_despawn();
        }
    }
}
