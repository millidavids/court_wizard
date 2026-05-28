//! Plague wind casting: input handling and indicator setup.

use super::cloud::spawn_plague_cloud;
use super::components::{PlagueWindIndicator, PlagueWindTalentParams};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::ObstacleChanged;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range_ground, spawn_circle_indicator,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

/// Computes talent parameters from the player's active talent selections.
fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> PlagueWindTalentParams {
    let mut params = PlagueWindTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::PlagueWind, 0);
    let t2 = talents.get_selection(Spell::PlagueWind, 1);
    let t3 = talents.get_selection(Spell::PlagueWind, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => params.damage_mult = constants::VIRULENT_STRAIN_DAMAGE_MULT,
        Some(1) => {
            params.radius_mult = constants::MIASMA_RADIUS_MULT;
            params.duration_mult = constants::MIASMA_DURATION_MULT;
        }
        Some(2) => {
            params.duration_mult = constants::LINGERING_FOG_DURATION_MULT;
            params.speed_mult = constants::LINGERING_FOG_SPEED_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.plague_carrier = true,
        Some(1) => params.toxic_weakness = true,
        Some(2) => params.choking_gas = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.pandemic = true,
        Some(1) => params.twin_plumes = true,
        Some(2) => params.necrotic_rot = true,
        _ => {}
    }

    params
}

/// Local wizard plague wind casting -- click-drag vector mechanic.
/// Click locks origin, drag defines travel direction, cast completes when timer fills.
#[allow(clippy::too_many_arguments)]
pub fn handle_plague_wind_casting(
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
    cursor_resources: (
        Res<CorrectedCursorPosition>,
        Res<TargetAssistWorldPos>,
        Res<LocalSpellOrigin>,
    ),
    caster_query: Query<&SpellCaster>,
    circle_indicator_query: Query<&SpellCircleIndicator>,
    indicator_query: Query<&PlagueWindIndicator>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    active_talents: Option<Res<ActiveTalents>>,
    mut audio_ctx: (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (ref sfx, ref game_config, ref mut pending_cast_events) = audio_ctx;
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::PlagueWind {
        return;
    }

    let wizard_pos = local_origin.0;
    let scale = primed_spell.empowerment;
    let radius = constants::CLOUD_RADIUS * scale;
    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range_ground(pos, wizard_pos, wizard.spell_range, radius));

    // Spawn indicator on Resting -> Casting transition (origin locked at click position)
    if matches!(*casting_state, CastingState::Resting)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(constants::MANA_COST)
        && let Some(pos) = clamped_pos
    {
        // Spawn directional arrow
        let arrow_entity = commands
            .spawn((
                Mesh3d(visual_assets.arrow_mesh.clone()),
                MeshMaterial3d(visual_assets.plague_wind_arrow.clone()),
                arrow_transform(pos, 0.0),
                OnGameplayScreen,
            ))
            .id();

        let circle_entity = spawn_circle_indicator(
            &mut commands,
            &mut meshes,
            visual_assets.plague_wind_indicator.clone(),
            pos,
            constants::CLOUD_RADIUS * scale,
        )
        .insert(PlagueWindIndicator {
            arrow_entity: Some(arrow_entity),
        })
        .id();
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(circle_entity));
    }

    // Update arrow direction during casting (indicator position stays locked)
    if matches!(*casting_state, CastingState::Casting { .. })
        && let Some(cursor) = clamped_pos
        && let Ok(caster) = caster_query.get(wizard_entity)
        && let Some(indicator_entity) = caster.indicator_entity
        && let Ok(circle_indicator) = circle_indicator_query.get(indicator_entity)
        && let Ok(pw_indicator) = indicator_query.get(indicator_entity)
        && let Some(arrow_entity) = pw_indicator.arrow_entity
    {
        let origin = circle_indicator.position;
        let delta_xz = Vec2::new(cursor.x - origin.x, cursor.z - origin.z);
        if delta_xz.length_squared() > 1.0 {
            let angle = -delta_xz.x.atan2(-delta_xz.y);
            commands
                .entity(arrow_entity)
                .insert(arrow_transform(origin, angle));
        }
    }

    // Get the locked origin from indicator
    let indicator_pos = caster_query
        .get(wizard_entity)
        .ok()
        .and_then(|caster| caster.indicator_entity)
        .and_then(|ie| circle_indicator_query.get(ie).ok())
        .map(|indicator| indicator.position);

    let effective_input = WizardInput {
        cursor_pos: indicator_pos.or(clamped_pos),
        ..input
    };

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = plague_wind_casting_logic(
        &effective_input,
        &time,
        wizard_entity,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &indicator_query,
        &mut commands,
        &mut obstacle_events,
        sfx,
        game_config,
        talent_params,
        clamped_pos,
        local_origin.0,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Nature,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core plague wind casting logic.
/// `cursor_pos` is the current (live) cursor position used to compute travel direction.
#[allow(clippy::too_many_arguments)]
fn plague_wind_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &Query<&PlagueWindIndicator>,
    commands: &mut Commands,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: PlagueWindTalentParams,
    cursor_pos: Option<Vec3>,
    local_origin: Vec3,
) -> bool {
    let wizard_pos = local_origin;
    let scale = primed_spell.empowerment;
    let radius = constants::CLOUD_RADIUS * scale * talent_params.radius_mult;

    // Check for release event — cancels the cast
    if input.just_released {
        cleanup_indicator(commands, caster_query, indicator_query, wizard_entity);
        commands.entity(wizard_entity).remove::<SpellCaster>();
        casting_state.cancel();
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
                && input.cursor_pos.is_some()
            {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    // Origin is the locked indicator position (where the user first clicked)
                    let pos = input.cursor_pos.unwrap_or(wizard_pos);

                    // Direction = from origin toward current cursor position
                    let base_direction = cursor_pos
                        .map(|cursor| {
                            let delta = Vec3::new(cursor.x - pos.x, 0.0, cursor.z - pos.z);
                            if delta.length_squared() > 1.0 {
                                delta.normalize()
                            } else {
                                // If cursor hasn't moved, default forward
                                Vec3::X
                            }
                        })
                        .unwrap_or(Vec3::X);

                    let damage = constants::DAMAGE_PER_TICK * scale * talent_params.damage_mult;
                    let duration = constants::CLOUD_DURATION * scale * talent_params.duration_mult;
                    let speed = constants::CLOUD_SPEED * talent_params.speed_mult;

                    audio::play_sfx(commands, &sfx.plague_wind_cast, pos, game_config, sfx);

                    if talent_params.twin_plumes {
                        // Twin Plumes: spawn 2 clouds at diverging angles
                        let half_angle = constants::TWIN_PLUMES_ANGLE_SPREAD / 2.0;
                        let twin_damage = damage * constants::TWIN_PLUMES_DAMAGE_MULT;

                        for angle_offset in [-half_angle, half_angle] {
                            let cos_a = angle_offset.cos();
                            let sin_a = angle_offset.sin();
                            let dir = Vec3::new(
                                base_direction.x * cos_a - base_direction.z * sin_a,
                                0.0,
                                base_direction.x * sin_a + base_direction.z * cos_a,
                            )
                            .normalize();

                            spawn_plague_cloud(
                                commands,
                                obstacle_events,
                                pos,
                                radius,
                                twin_damage,
                                duration,
                                speed,
                                dir,
                                talent_params,
                            );
                        }
                    } else {
                        spawn_plague_cloud(
                            commands,
                            obstacle_events,
                            pos,
                            radius,
                            damage,
                            duration,
                            speed,
                            base_direction,
                            talent_params,
                        );
                    }

                    completed = true;
                }

                // Clean up indicator and caster regardless of mana success
                cleanup_indicator(commands, caster_query, indicator_query, wizard_entity);
                commands.entity(wizard_entity).remove::<SpellCaster>();
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_indicator(commands, caster_query, indicator_query, wizard_entity);
            commands.entity(wizard_entity).remove::<SpellCaster>();
            casting_state.cancel();
        }
    }

    completed
}

/// Builds a flat arrow Transform at the given position, rotated by `angle` radians in the XZ plane.
fn arrow_transform(origin: Vec3, angle: f32) -> Transform {
    Transform::from_translation(Vec3::new(origin.x, constants::CLOUD_BASE_Y + 0.1, origin.z))
        .with_rotation(
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2) * Quat::from_rotation_z(angle),
        )
        .with_scale(Vec3::new(
            constants::ARROW_WIDTH,
            constants::ARROW_LENGTH,
            1.0,
        ))
}

/// Despawns the indicator circle and its arrow entity.
fn cleanup_indicator(
    commands: &mut Commands,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &Query<&PlagueWindIndicator>,
    wizard_entity: Entity,
) {
    if let Ok(caster) = caster_query.get(wizard_entity)
        && let Some(indicator_entity) = caster.indicator_entity
    {
        // Despawn arrow if it exists
        if let Ok(indicator) = indicator_query.get(indicator_entity)
            && let Some(arrow_entity) = indicator.arrow_entity
        {
            commands.entity(arrow_entity).try_despawn();
        }
        commands.entity(indicator_entity).try_despawn();
    }
}
