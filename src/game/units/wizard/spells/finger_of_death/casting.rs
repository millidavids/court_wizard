//! Finger of Death casting and beam spawn.

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::*;
use super::constants;
use super::effects::spawn_necrotic_explosion;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::crt_effect::ScreenDesaturateMessage;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Health, Team, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    PendingDefenderHeal, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

/// Computes talent-modified parameters for Finger of Death.
pub(super) fn compute_fod_params(active_talents: Option<&ActiveTalents>) -> FodTalentParams {
    let mut params = FodTalentParams::default();

    let talents = match active_talents {
        Some(t) => t,
        None => return params,
    };

    let t1 = talents.get_selection(Spell::FingerOfDeath, 0);
    let t2 = talents.get_selection(Spell::FingerOfDeath, 1);
    let t3 = talents.get_selection(Spell::FingerOfDeath, 2);

    // Tier 1
    match t1 {
        Some(0) => {
            // Death's Reach: wider beam
            params.beam_width *= constants::DEATHS_REACH_WIDTH_MULT;
            params.beam_width_fired *= constants::DEATHS_REACH_WIDTH_MULT;
        }
        Some(1) => {
            // Soul Harvest: mana refund on kill
            params.soul_harvest_refund = constants::SOUL_HARVEST_MANA_REFUND;
        }
        Some(2) => {
            // Quick Draw: faster cast
            params.cast_time_mult = constants::QUICK_DRAW_CAST_MULT;
        }
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => {
            // Finger of Undeath: raise killed as undead
            params.finger_of_undeath = true;
        }
        Some(1) => {
            // Death Sentence: cheaper, weaker, faster cooldown
            params.mana_threshold = constants::DEATH_SENTENCE_MANA_THRESHOLD;
            params.damage = constants::DEATH_SENTENCE_DAMAGE;
            params.cooldown_mult = constants::DEATH_SENTENCE_COOLDOWN_MULT;
        }
        Some(2) => {
            // Siphon Life: heal nearest defender
            params.siphon_life_percent = constants::SIPHON_LIFE_HEAL_PERCENT;
        }
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => {
            // Reaper's Scythe: sweep arc, reduced damage
            params.reapers_scythe = true;
            params.damage *= constants::REAPERS_SCYTHE_DAMAGE_MULT;
        }
        Some(1) => {
            // Necrotic Explosion: AoE on kill
            params.necrotic_explosion = true;
        }
        Some(2) => {
            // Deathmark: reduced damage + chain on kill
            params.deathmark = true;
            params.chain_damage_mult = constants::DEATHMARK_CHAIN_DAMAGE_PERCENT;
        }
        _ => {}
    }

    params
}

/// Action the shared logic requests the wrapper to perform on beams.
enum BeamAction {
    /// Update existing beam with new data.
    UpdateBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        cast_progress: f32,
        delta_secs: f32,
    },
    /// Spawn a new beam (optionally with initial cast progress).
    SpawnBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        empowerment: f32,
        cast_progress: f32,
        talent_params: FodTalentParams,
    },
    /// Despawn all beams for this wizard.
    DespawnAll,
    /// No beam action needed.
    None,
}

/// Result from the shared casting logic.
struct CastingResult {
    beam_action: BeamAction,
    /// Whether to remove the AwaitingFingerOfDeathRelease component.
    remove_awaiting_release: bool,
}

/// Local wizard Finger of Death casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_finger_of_death_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &mut CastingState, &Mana, &PrimedSpell, &Wizard),
        With<LocalWizard>,
    >,
    awaiting_release_query: Query<(), With<AwaitingFingerOfDeathRelease>>,
    cooldown_query: Query<(), (With<FingerOfDeathCooldown>, With<LocalWizard>)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut beams: Query<(Entity, &mut FingerOfDeathBeam)>,
    active_talents: Option<Res<ActiveTalents>>,
    target_assist: Res<TargetAssistWorldPos>,
) {
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, mut casting_state, mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::FingerOfDeath {
        return;
    }

    let awaiting_release = awaiting_release_query.get(wizard_entity).is_ok();
    let on_cooldown = cooldown_query.get(wizard_entity).is_ok();
    let has_existing_beam = beams.iter().next().is_some();
    let talent_params = compute_fod_params(active_talents.as_deref());

    let result = finger_of_death_casting_logic(
        &input,
        &time,
        &mut casting_state,
        mana,
        primed_spell,
        wizard,
        awaiting_release,
        on_cooldown,
        has_existing_beam,
        &talent_params,
    );

    // Apply component changes
    if result.remove_awaiting_release {
        commands
            .entity(wizard_entity)
            .remove::<AwaitingFingerOfDeathRelease>();
    }

    // Apply beam action
    match result.beam_action {
        BeamAction::UpdateBeam {
            origin,
            direction,
            length,
            cast_progress,
            delta_secs,
        } => {
            if let Some((_, mut beam)) = beams.iter_mut().next() {
                beam.origin = origin;
                beam.direction = direction;
                beam.length = length;
                beam.cast_progress = cast_progress;
                beam.time_alive += delta_secs;
            }
        }
        BeamAction::SpawnBeam {
            origin,
            direction,
            length,
            empowerment,
            cast_progress,
            talent_params,
        } => {
            let mut new_beam = FingerOfDeathBeam::with_talents(
                origin,
                direction,
                length,
                empowerment,
                talent_params,
            );
            new_beam.cast_progress = cast_progress;
            spawn_beam(&mut commands, &visual_assets, &mut materials, new_beam);
        }
        BeamAction::DespawnAll => {
            for (beam_entity, _) in beams.iter() {
                commands.entity(beam_entity).try_despawn();
            }
        }
        BeamAction::None => {}
    }
}

/// Core Finger of Death casting logic -- called by the local system.
#[allow(clippy::too_many_arguments)]
fn finger_of_death_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    awaiting_release: bool,
    on_cooldown: bool,
    has_existing_beam: bool,
    talent_params: &FodTalentParams,
) -> CastingResult {
    let mut result = CastingResult {
        beam_action: BeamAction::None,
        remove_awaiting_release: false,
    };

    let wizard_pos = SPELL_ORIGIN;

    // Check for release event
    if input.just_released {
        result.remove_awaiting_release = true;
        casting_state.cancel();
        result.beam_action = BeamAction::DespawnAll;
        return result;
    }

    // Talent-modified cast time
    let cast_time = primed_spell.cast_time * talent_params.cast_time_mult;

    // Mouse is held - handle casting based on state
    match *casting_state {
        CastingState::Channeling { .. } => {
            // Finger of Death doesn't channel - just cancel
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Calculate beam target
            if let Some(cursor_pos) = input.cursor_pos {
                let beam_origin =
                    wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                // Clamp target position to spell range
                let to_target = cursor_pos - beam_origin;
                let distance = to_target.length();
                let clamped_target = if distance > wizard.spell_range {
                    beam_origin + to_target.normalize() * wizard.spell_range
                } else {
                    cursor_pos
                };

                let direction = (clamped_target - beam_origin).normalize();
                let beam_length = (clamped_target - beam_origin)
                    .length()
                    .min(constants::BEAM_LENGTH);

                // Calculate cast progress using talent-modified cast time
                let cast_progress = (casting_state.progress(cast_time)).min(1.0);

                // Update existing beam or spawn new one
                if has_existing_beam {
                    result.beam_action = BeamAction::UpdateBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        cast_progress,
                        delta_secs: time.delta_secs(),
                    };
                } else {
                    result.beam_action = BeamAction::SpawnBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        empowerment: primed_spell.empowerment,
                        cast_progress,
                        talent_params: talent_params.clone(),
                    };
                }
            }
        }
        CastingState::Resting => {
            // Not casting - check if we're waiting for mouse release or on cooldown
            if awaiting_release || on_cooldown {
                return result;
            }

            // Check for active input with talent-modified mana threshold
            if (input.just_pressed || input.pressed)
                && mana.percentage() >= talent_params.mana_threshold
            {
                casting_state.start_cast();

                // Spawn initial beam
                if let Some(cursor_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    // Clamp target position to spell range
                    let to_target = cursor_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        cursor_pos
                    };

                    let direction = (clamped_target - beam_origin).normalize();
                    let beam_length = (clamped_target - beam_origin)
                        .length()
                        .min(constants::BEAM_LENGTH);

                    result.beam_action = BeamAction::SpawnBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        empowerment: primed_spell.empowerment,
                        cast_progress: 0.0,
                        talent_params: talent_params.clone(),
                    };
                }
            }
        }
    }

    result
}

/// Spawns a Finger of Death beam entity with triangle mesh (like disintegrate),
/// plus glow aura and origin flare siblings.
pub(crate) fn spawn_beam(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    beam: FingerOfDeathBeam,
) {
    let origin = beam.origin;

    // Clone the base material so each beam can animate independently
    let material = materials
        .get(&assets.finger_of_death_beam)
        .cloned()
        .unwrap_or_default();
    let instance_material = materials.add(material);

    let beam_entity = commands
        .spawn((
            beam,
            Mesh3d(assets.cross_plane_triangle.clone()),
            MeshMaterial3d(instance_material),
            Transform::from_translation(origin),
            OnGameplayScreen,
        ))
        .id();

    // Glow triangle sibling (wider, semi-transparent)
    commands.spawn((
        FingerOfDeathGlow { beam_entity },
        Mesh3d(assets.cross_plane_triangle.clone()),
        MeshMaterial3d(assets.finger_of_death_glow.clone()),
        Transform::from_translation(origin),
        OnGameplayScreen,
    ));

    // Origin flare (bright spot at beam origin)
    commands.spawn((
        FingerOfDeathFlare { beam_entity },
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.finger_of_death_flare.clone()),
        Transform::from_translation(origin),
        OnGameplayScreen,
    ));
}

/// Applies Finger of Death damage when cast completes.
///
/// Handles all talent effects: Soul Harvest, Finger of Undeath, Siphon Life,
/// Necrotic Explosion, Deathmark, and Reaper's Scythe initiation.
#[allow(clippy::too_many_arguments)]
pub fn apply_finger_of_death_damage(
    time: Res<Time>,
    mut commands: Commands,
    mut mouse_state: ResMut<MouseButtonState>,
    mut beams: Query<&mut FingerOfDeathBeam>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        Without<Wizard>,
    >,
    mut wizard_query: Query<(Entity, &mut Mana, &mut CastingState), With<Wizard>>,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    rocks: Query<&crate::game::terrain::boulder::components::Boulder>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut desaturate: MessageWriter<ScreenDesaturateMessage>,
    mut vignette_pulse: MessageWriter<crate::game::crt_effect::VignettePulseMessage>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let mut any_fired = false;

    let mut hit_positions: Vec<Vec3> = Vec::new();
    let mut kill_positions: Vec<Vec3> = Vec::new();
    let mut kill_count: u32 = 0;
    let mut total_damage_dealt: f32 = 0.0;
    let mut beam_origin = Vec3::ZERO;
    let mut beam_damage: f32 = 0.0;
    let mut beam_talent_params = FodTalentParams::default();

    for mut beam in beams.iter_mut() {
        // Only apply damage if cast is complete and hasn't fired yet
        if beam.has_fired || beam.cast_progress < 1.0 {
            continue;
        }

        // Reaper's Scythe starts a sweep instead of instant fire
        if beam.talent_params.reapers_scythe {
            beam.has_fired = true;
            any_fired = true;
            beam_origin = beam.origin;
            beam_talent_params = beam.talent_params.clone();

            // The sweep system handles damage — spawn sweep entity
            commands.spawn((
                ReapersScytheSweep {
                    center_direction: beam.direction,
                    time_elapsed: 0.0,
                    duration: constants::REAPERS_SCYTHE_SWEEP_DURATION
                        * beam.talent_params.cast_time_mult,
                    origin: beam.origin,
                    length: beam.length,
                    empowerment: beam.empowerment,
                    talent_params: beam.talent_params.clone(),
                    hit_entities: std::collections::HashSet::new(),
                },
                OnGameplayScreen,
            ));
            continue;
        }

        // Mark as fired
        beam.has_fired = true;
        any_fired = true;
        beam_origin = beam.origin;
        beam_damage = beam.damage();
        beam_talent_params = beam.talent_params.clone();

        // Find nearest wall/rock intersection to limit beam reach
        let beam_end = beam.origin + beam.direction * beam.length;
        let mut max_t = 1.0_f32;
        for wall in &walls {
            if let Some(t) = wall.line_segment_intersects(beam.origin, beam_end) {
                max_t = max_t.min(t);
            }
        }
        for rock in &rocks {
            if !rock.sinking
                && let Some(t) = rock.line_segment_intersects(beam.origin, beam_end)
            {
                max_t = max_t.min(t);
            }
        }
        let effective_length = beam.length * max_t;

        // Apply damage to all units along beam (before wall)
        let beam_width = beam.beam_width();
        let damage = beam.damage();

        // Normal damage application
        for (entity, transform, mut health, mut temp_hp, has_spell_shield, _team) in
            targets.iter_mut()
        {
            if has_spell_shield {
                continue;
            }
            if beam.contains_point(transform.translation, beam_width) {
                let proj = (transform.translation - beam.origin).dot(beam.direction);
                if proj <= effective_length {
                    let was_alive = health.current > 0.0;
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        DamageType::Necrotic,
                        false,
                    );
                    total_damage_dealt += damage;
                    hit_positions.push(transform.translation);

                    if was_alive && health.current <= 0.0 {
                        kill_count += 1;
                        kill_positions.push(transform.translation);
                    }

                    // Deathmark: also apply debuff to survivors (chain beam fires if they die later)
                    if beam.talent_params.deathmark && health.current > 0.0 {
                        commands.entity(entity).insert(DeathmarkDebuff {
                            time_remaining: constants::DEATHMARK_DURATION,
                            beam_origin: beam.origin,
                            empowerment: beam.empowerment,
                            talent_params: beam.talent_params.clone(),
                        });
                    }
                }
            }
        }
    }

    // Play sound effect and drain mana
    if any_fired {
        vignette_pulse.write(crate::game::crt_effect::VignettePulseMessage {
            duration: 0.4,
            intensity: 0.15,
        });

        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            vfx::systems::SpellSchool::Dark,
            time.elapsed_secs(),
        );
        audio::play_sfx(
            &mut commands,
            &sfx.finger_of_death_cast,
            SPELL_ORIGIN,
            &game_config,
            &sfx,
        );

        // Chain beams (from deathmark) skip wizard state changes entirely
        if !beam_talent_params.is_chain_beam {
            for (wizard_entity, mut mana, mut casting_state) in wizard_query.iter_mut() {
                if !matches!(*casting_state, CastingState::Resting) {
                    mana.current -= mana.max * beam_talent_params.mana_threshold;
                    mana.current = mana.current.max(0.0);
                    casting_state.cancel();

                    // Add awaiting release marker to prevent immediate recast
                    commands
                        .entity(wizard_entity)
                        .insert(AwaitingFingerOfDeathRelease);

                    // Death Sentence: apply cooldown
                    let cooldown = constants::COOLDOWN * beam_talent_params.cooldown_mult;
                    if cooldown > 0.0 {
                        commands
                            .entity(wizard_entity)
                            .insert(FingerOfDeathCooldown {
                                remaining: cooldown,
                            });
                    }

                    // Soul Harvest: refund mana on kill
                    if beam_talent_params.soul_harvest_refund > 0.0 && kill_count > 0 {
                        let refund =
                            mana.max * beam_talent_params.soul_harvest_refund * kill_count as f32;
                        mana.current = (mana.current + refund).min(mana.max);
                    }

                    // Siphon Life: queue heal for nearest defender (resolved next frame)
                    if beam_talent_params.siphon_life_percent > 0.0 && total_damage_dealt > 0.0 {
                        let heal_amount =
                            total_damage_dealt * beam_talent_params.siphon_life_percent;
                        commands.insert_resource(PendingDefenderHeal {
                            amount: heal_amount,
                            origin: beam_origin,
                        });
                    }
                }
            }

            // Mark mouse hold as consumed to prevent immediate recast
            mouse_state.left_consumed = true;

            // Trigger screen desaturation
            desaturate.write(ScreenDesaturateMessage);
        }

        // Finger of Undeath: queue raises for next frame (corpses don't exist yet this frame)
        if beam_talent_params.finger_of_undeath && !kill_positions.is_empty() {
            commands.insert_resource(PendingUndeadRaise {
                kill_positions: kill_positions.clone(),
            });
        }

        // Necrotic Explosion: AoE at kill positions (20% of beam damage)
        if beam_talent_params.necrotic_explosion {
            let explosion_damage = beam_damage * constants::NECROTIC_EXPLOSION_DAMAGE_PERCENT;
            for kill_pos in &kill_positions {
                spawn_necrotic_explosion(
                    &mut commands,
                    *kill_pos,
                    explosion_damage,
                    &visual_assets,
                    &mut materials,
                );
            }
        }

        // Track talent progress (kills, not just hits)
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::FingerOfDeath, kill_count);
        }

        // Spawn necrotic vein particles from each hit unit
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        for hit_pos in &hit_positions {
            let ground_pos = Vec3::new(hit_pos.x, constants::VEIN_Y_POSITION, hit_pos.z);
            for i in 0..constants::VEIN_COUNT {
                let angle = i as f32 * golden_angle;
                let dir = Vec3::new(angle.cos(), 0.0, angle.sin());

                let vein_material = materials
                    .get(&visual_assets.necrotic_vein)
                    .cloned()
                    .unwrap_or_default();
                let instance = materials.add(vein_material);

                commands.spawn((
                    NecroticVein {
                        velocity: dir * constants::VEIN_SPEED,
                        time_alive: 0.0,
                        lifetime: constants::VEIN_LIFETIME,
                        base_size: constants::VEIN_SIZE,
                        wander_phase: i as f32 * 1.7,
                    },
                    Mesh3d(visual_assets.particle_quad.clone()),
                    MeshMaterial3d(instance),
                    Transform::from_translation(ground_pos)
                        .with_rotation(UPWARD_ROTATION)
                        .with_scale(Vec3::splat(constants::VEIN_SIZE)),
                    OnGameplayScreen,
                ));
            }
        }

        // Spawn necrotic pulse ring at beam origin
        let pulse_material = materials
            .get(&visual_assets.necrotic_pulse)
            .cloned()
            .unwrap_or_default();
        let pulse_instance = materials.add(pulse_material);

        commands.spawn((
            NecroticPulse {
                time_alive: 0.0,
                lifetime: constants::PULSE_LIFETIME,
                max_radius: constants::PULSE_MAX_RADIUS,
            },
            Mesh3d(visual_assets.unit_circle.clone()),
            MeshMaterial3d(pulse_instance),
            Transform::from_translation(Vec3::new(
                beam_origin.x,
                constants::PULSE_Y_POSITION,
                beam_origin.z,
            ))
            .with_rotation(UPWARD_ROTATION)
            .with_scale(Vec3::splat(1.0)),
            OnGameplayScreen,
        ));
    }
}
