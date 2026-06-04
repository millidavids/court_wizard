//! Lightning rod casting and rod spawn.

use super::components::{LightningRod, LightningRodTalentParams, LightningStrike};
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::lightning_bolt::{
    LightningBoltConfig, spawn_lightning_bolt,
};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range_ground, cleanup_spell_caster, handle_spell_release,
    spawn_circle_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellEffectKind;
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

/// Compute talent parameters from active talent selections.
fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> LightningRodTalentParams {
    let mut params = LightningRodTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::LightningRod, 0);
    let t2 = talents.get_selection(Spell::LightningRod, 1);
    let t3 = talents.get_selection(Spell::LightningRod, 2);

    // Tier 1
    match t1 {
        Some(0) => params.duration_mult *= TALLER_ROD_DURATION_MULT,
        Some(1) => params.strike_interval_mult = RAPID_STRIKES_INTERVAL_MULT,
        Some(2) => {
            params.arc_radius_mult = WIDER_ARC_RADIUS_MULT;
            params.extra_targets = WIDER_ARC_EXTRA_TARGETS;
        }
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => params.chain_reaction = true,
        Some(1) => params.magnetic_field = true,
        Some(2) => params.overcharge = true,
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => {
            params.storm_spire = true;
            params.damage_mult = STORM_SPIRE_DAMAGE_MULT;
            // No duration multiplier: Storm Spire rods are concentration-held
            // (spawned with f32::MAX duration), so a finite-duration scale is moot.
        }
        Some(1) => params.tesla_coil = true,
        Some(2) => params.lightning_nexus = true,
        _ => {}
    }

    params
}

/// Spawns a descending lightning bolt: a jagged `LightningBolt` parent that
/// the strike system drives downward by mutating its `end.y`. The
/// `LightningStrike` component lives on the same entity so the descent system
/// can find it.
pub(super) fn spawn_descending_strike(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    strike: LightningStrike,
) {
    let bolt_width = STRIKE_BOLT_WIDTH * strike.empowerment;
    let strike_start = Vec3::new(
        strike.target_pos.x,
        STRIKE_SPAWN_HEIGHT,
        strike.target_pos.z,
    );
    // Use the cross-plane cylinder so the vertical bolt reads as a solid tube
    // from any camera angle. A flat `unit_rect` thins out and visibly breaks
    // apart between jagged segments because each segment's roll-axis rotation
    // can flip independently with the jitter direction.
    let bolt_entity = spawn_lightning_bolt(
        commands,
        assets.cross_plane_cylinder.clone(),
        assets.lightning_strike.clone(),
        strike_start,
        // Bolt visible length grows as the head descends; start with a short tail.
        Vec3::new(
            strike.target_pos.x,
            STRIKE_SPAWN_HEIGHT - 1.0,
            strike.target_pos.z,
        ),
        LightningBoltConfig {
            width: bolt_width,
            // Effectively no per-bolt lifetime cap — the descent system flips
            // the bolt into its afterimage phase on impact.
            lifetime: 60.0,
            peak_height: 0.0,
            jitter_amplitude: 18.0,
            segments: 24,
            fork_count: 2,
            fork_segments: 3,
            fork_length: bolt_width * 4.0 + 30.0,
            // Long, slow fade so the bolt visibly hangs at the rod after
            // impact — like the retinal ghost of a real lightning strike.
            afterimage_duration: 0.4,
        },
    );
    commands.entity(bolt_entity).insert(strike);
}

/// Local wizard Lightning Rod casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_lightning_rod_casting(
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
    active_talents: Option<Res<ActiveTalents>>,
    target_assist: Res<TargetAssistWorldPos>,
    local_origin: Res<LocalSpellOrigin>,
    mut audio_ctx: (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (ref sfx, ref game_config, ref mut pending_cast_events) = audio_ctx;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::LightningRod {
        return;
    }

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range_ground(pos, local_origin.0, wizard.spell_range, 0.0));

    // Spawn indicator on Resting -> Casting transition
    if matches!(*casting_state, CastingState::Resting)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(MANA_COST)
        && let Some(pos) = clamped_pos
    {
        let circle_entity = spawn_circle_indicator(
            &mut commands,
            &mut meshes,
            visual_assets.lightning_rod_indicator.clone(),
            pos,
            ARC_RADIUS * primed_spell.empowerment,
        )
        .id();
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(circle_entity));
    }

    // Update indicator position during casting
    if matches!(*casting_state, CastingState::Casting { .. })
        && let Some(pos) = clamped_pos
    {
        update_indicator_position(wizard_entity, pos, &caster_query, &mut indicator_query);
    }

    // Get the final spawn position from indicator if available
    let indicator_pos = caster_query
        .get(wizard_entity)
        .ok()
        .and_then(|caster| caster.indicator_entity)
        .and_then(|ie| indicator_query.get(ie).ok())
        .map(|indicator| indicator.position);

    // Override cursor_pos with indicator position for shared logic
    let effective_input = WizardInput {
        cursor_pos: indicator_pos.or(clamped_pos),
        ..input
    };

    let completed = lightning_rod_casting_logic(
        &effective_input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut commands,
        &visual_assets,
        sfx,
        game_config,
        active_talents.as_deref(),
        local_origin.0,
        pending_cast_events,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Lightning,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core Lightning Rod casting logic.
#[allow(clippy::too_many_arguments)]
fn lightning_rod_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    _wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    active_talents: Option<&ActiveTalents>,
    local_origin: Vec3,
    pending_cast_events: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) -> bool {
    let wizard_pos = local_origin;

    // Check for release event - cancel cast
    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(MANA_COST)
                && input.cursor_pos.is_some()
            {
                // SpellCaster insertion handled by the wrapper
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(MANA_COST) {
                    let spawn_pos = input.cursor_pos.unwrap_or(wizard_pos);
                    let talent_params = compute_talent_params(active_talents);
                    let duration =
                        TOWER_DURATION * primed_spell.empowerment * talent_params.duration_mult;

                    if talent_params.storm_spire {
                        // Storm Spire makes Lightning Rod a CONCENTRATION spell: a
                        // single rod never expires on a timer (duration = MAX) and
                        // persists until the player ends concentration. The
                        // `ConcentrationSpell` spawns the End button and reserves mana.
                        let rod = spawn_lightning_rod(
                            commands,
                            assets,
                            spawn_pos,
                            primed_spell.empowerment,
                            f32::MAX,
                            talent_params,
                        );
                        commands.entity(rod).insert(ConcentrationSpell {
                            spell_name: "Lightning Rod",
                            mana_cost: MANA_COST,
                        });
                    } else {
                        spawn_lightning_rod(
                            commands,
                            assets,
                            spawn_pos,
                            primed_spell.empowerment,
                            duration,
                            talent_params,
                        );
                    }

                    audio::play_sfx_synced(
                        commands,
                        pending_cast_events,
                        SpellSoundId::LightningRodImpact,
                        spawn_pos,
                        game_config,
                        sfx,
                    );
                    completed = true;
                }

                // Clean up indicator and caster
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

/// Spawns the lightning rod tower entity.
pub(crate) fn spawn_lightning_rod(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
    duration: f32,
    talent_params: LightningRodTalentParams,
) -> Entity {
    let tower_height = TOWER_HEIGHT;
    let tower_radius = TOWER_RADIUS;

    // Cylinder sits centered, so position at half height
    let spawn_pos = Vec3::new(position.x, tower_height / 2.0, position.z);

    // cross_plane_cylinder has radius 0.5 and height 1.0, scale to tower dimensions
    // radius scale = tower_radius / 0.5, height scale = tower_height / 1.0
    let radius_scale = tower_radius / 0.5;

    commands
        .spawn((
            LightningRod::new(
                Vec3::new(position.x, 0.0, position.z),
                duration,
                empowerment,
                talent_params,
            ),
            Mesh3d(assets.cross_plane_cylinder.clone()),
            MeshMaterial3d(assets.lightning_rod.clone()),
            Transform::from_translation(spawn_pos).with_scale(Vec3::new(
                radius_scale,
                tower_height,
                radius_scale,
            )),
            NetworkedSpellEffect {
                kind: SpellEffectKind::LightningRod,
            },
            OnGameplayScreen,
        ))
        .id()
}
