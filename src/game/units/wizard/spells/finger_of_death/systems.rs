use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::crt_effect::ScreenDesaturateMessage;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    Corpse, Health, PermanentCorpse, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::constants::{EXCREMAGE_BROWN, EXCREMAGE_BROWN_DARK};
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

/// Processes pending undead raises from Finger of Undeath.
/// Runs the frame after kills, when corpses have been created by the death system.
pub fn process_pending_undead_raises(
    mut commands: Commands,
    pending: Option<Res<PendingUndeadRaise>>,
    corpse_query: Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: Option<Res<crate::game::units::undead::resources::UndeadAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(pending) = pending else {
        return;
    };
    let Some(undead_assets) = undead_assets else {
        commands.remove_resource::<PendingUndeadRaise>();
        return;
    };

    use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
    use crate::game::units::infantry::constants::UNDEAD_SPRITE_TINT;

    let mut raised_entities = Vec::new();
    for kill_pos in &pending.kill_positions {
        let mut best: Option<(Entity, f32)> = None;
        for (entity, transform) in corpse_query.iter() {
            if raised_entities.contains(&entity) {
                continue;
            }
            let dist = transform.translation.distance(*kill_pos);
            if dist < 50.0
                && (best.is_none() || dist < best.as_ref().map(|(_, d)| *d).unwrap_or(f32::MAX))
            {
                best = Some((entity, dist));
            }
        }
        if let Some((corpse_entity, _)) = best {
            raised_entities.push(corpse_entity);
            crate::game::units::systems::resurrect_corpse_as_infantry(
                &mut commands,
                corpse_entity,
                *kill_pos,
                Team::Undead,
                UNIT_HEALTH,
                UNIT_MOVEMENT_SPEED * 0.5,
                UNDEAD_SPRITE_TINT,
                undead_assets.sprite_texture.clone(),
                undead_assets.sprite_mesh.clone(),
                &mut materials,
            );
        }
    }

    commands.remove_resource::<PendingUndeadRaise>();
}

/// Removes AwaitingFingerOfDeathRelease when the mouse is no longer held.
/// This runs independently of the casting system's run conditions.
pub fn clear_awaiting_fod_release(
    mut commands: Commands,
    mouse_held: Res<crate::game::input::components::MouseLeftHeldThisFrame>,
    query: Query<Entity, With<AwaitingFingerOfDeathRelease>>,
) {
    if mouse_held.held {
        return;
    }
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<AwaitingFingerOfDeathRelease>();
    }
}

/// Spawns a necrotic explosion AoE visual and applies damage at a position.
fn spawn_necrotic_explosion(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    visual_assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    let pulse_material = materials
        .get(&visual_assets.necrotic_pulse)
        .cloned()
        .unwrap_or_default();
    let instance = materials.add(pulse_material);

    commands.spawn((
        NecroticExplosionBurst {
            time_alive: 0.0,
            lifetime: constants::NECROTIC_EXPLOSION_PULSE_LIFETIME,
            max_radius: constants::NECROTIC_EXPLOSION_RADIUS,
            damage,
            damage_applied: false,
        },
        Mesh3d(visual_assets.unit_circle.clone()),
        MeshMaterial3d(instance),
        Transform::from_translation(Vec3::new(
            position.x,
            constants::PULSE_Y_POSITION,
            position.z,
        ))
        .with_rotation(UPWARD_ROTATION)
        .with_scale(Vec3::splat(1.0)),
        OnGameplayScreen,
    ));
}

/// Applies necrotic explosion AoE damage to enemies near explosion positions (one-shot).
pub fn apply_necrotic_explosion_damage(
    mut commands: Commands,
    mut explosions: Query<(&Transform, &mut NecroticExplosionBurst)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<Wizard>,
    >,
) {
    // Collect positions and damage of explosions that haven't applied damage yet
    let mut pending: Vec<(Vec3, f32)> = Vec::new();
    for (transform, mut burst) in explosions.iter_mut() {
        if !burst.damage_applied {
            burst.damage_applied = true;
            pending.push((transform.translation, burst.damage));
        }
    }

    for (explosion_pos, explosion_damage) in &pending {
        for (entity, transform, mut health, mut temp_hp, has_spell_shield) in targets.iter_mut() {
            if has_spell_shield {
                continue;
            }
            let dist = transform.translation.distance(*explosion_pos);
            if dist <= constants::NECROTIC_EXPLOSION_RADIUS {
                apply_spell_damage(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    *explosion_damage,
                    DamageType::Necrotic,
                    false,
                );
            }
        }
    }
}

/// Updates necrotic explosion burst visuals — expand and fade.
pub fn update_necrotic_explosion_bursts(
    mut commands: Commands,
    time: Res<Time>,
    mut bursts: Query<(
        Entity,
        &mut NecroticExplosionBurst,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut burst, mut transform, material_handle) in bursts.iter_mut() {
        burst.time_alive += dt;

        if burst.time_alive >= burst.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = burst.time_alive / burst.lifetime;
        let radius = burst.max_radius * progress;
        transform.scale = Vec3::splat(radius);

        if let Some(material) = materials.get_mut(&material_handle.0) {
            let color = constants::NECROTIC_EXPLOSION_COLOR.to_srgba();
            let alpha = 0.6 * (1.0 - progress);
            material.base_color = Color::srgba(color.red, color.green, color.blue, alpha);
        }
    }
}

/// Ticks deathmark debuffs and fires chain beams when marked targets die.
#[allow(clippy::too_many_arguments)]
pub fn update_deathmark_debuffs(
    mut commands: Commands,
    time: Res<Time>,
    mut marked_targets: Query<(Entity, &Transform, &Health, &mut DeathmarkDebuff)>,
    all_enemies: Query<(Entity, &Transform, &Health), Without<Wizard>>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, transform, health, mut debuff) in marked_targets.iter_mut() {
        debuff.time_remaining -= dt;

        // Target died while debuffed — fire chain beam at nearest enemy
        if health.current <= 0.0 {
            let target_pos = transform.translation;
            // Find nearest living enemy
            let mut best: Option<(Vec3, f32)> = None;
            for (other_entity, other_transform, other_health) in all_enemies.iter() {
                if other_entity == entity || other_health.current <= 0.0 {
                    continue;
                }
                let dist = other_transform.translation.distance(target_pos);
                match best {
                    None => best = Some((other_transform.translation, dist)),
                    Some((_, best_dist)) if dist < best_dist => {
                        best = Some((other_transform.translation, dist));
                    }
                    _ => {}
                }
            }

            if let Some((enemy_pos, _)) = best {
                let direction = (enemy_pos - debuff.beam_origin).normalize();
                let length = enemy_pos.distance(debuff.beam_origin) + 50.0; // extend slightly past target

                let mut chain_params = debuff.talent_params.clone();
                chain_params.is_chain_beam = true;
                // Flat 10% of base damage every hop (no falloff)
                chain_params.chain_damage_mult = constants::DEATHMARK_CHAIN_DAMAGE_PERCENT;

                let mut chain_beam = FingerOfDeathBeam::with_talents(
                    debuff.beam_origin,
                    direction,
                    length,
                    debuff.empowerment,
                    chain_params,
                );
                chain_beam.cast_progress = 1.0; // fires immediately
                spawn_beam(&mut commands, &visual_assets, &mut materials, chain_beam);
            }

            commands.entity(entity).remove::<DeathmarkDebuff>();
            continue;
        }

        // Debuff expired
        if debuff.time_remaining <= 0.0 {
            commands.entity(entity).remove::<DeathmarkDebuff>();
        }
    }
}

/// Updates Reaper's Scythe sweep — rotates beam through arc and damages targets.
#[allow(clippy::too_many_arguments)]
pub fn update_reapers_scythe(
    mut commands: Commands,
    time: Res<Time>,
    mut sweeps: Query<(Entity, &mut ReapersScytheSweep)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<Wizard>,
    >,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    rocks_query: Query<&crate::game::terrain::boulder::components::Boulder>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let dt = time.delta_secs();

    for (sweep_entity, mut sweep) in sweeps.iter_mut() {
        sweep.time_elapsed += dt;

        if sweep.time_elapsed >= sweep.duration {
            commands.entity(sweep_entity).try_despawn();
            continue;
        }

        // Calculate current sweep angle
        let arc_radians = constants::REAPERS_SCYTHE_ARC_DEGREES.to_radians();
        let half_arc = arc_radians / 2.0;
        let progress = sweep.time_elapsed / sweep.duration;
        let current_angle = -half_arc + arc_radians * progress;

        // Rotate center direction by current_angle around Y axis
        let cos_a = current_angle.cos();
        let sin_a = current_angle.sin();
        let dir = sweep.center_direction;
        let rotated_dir = Vec3::new(
            dir.x * cos_a + dir.z * sin_a,
            dir.y,
            -dir.x * sin_a + dir.z * cos_a,
        )
        .normalize();

        // Create a temporary beam for hit detection
        let beam = FingerOfDeathBeam::with_talents(
            sweep.origin,
            rotated_dir,
            sweep.length,
            sweep.empowerment,
            sweep.talent_params.clone(),
        );

        // Find wall/rock intersection
        let beam_end = sweep.origin + rotated_dir * sweep.length;
        let mut max_t = 1.0_f32;
        for wall in &walls {
            if let Some(t) = wall.line_segment_intersects(sweep.origin, beam_end) {
                max_t = max_t.min(t);
            }
        }
        for rock in &rocks_query {
            if !rock.sinking
                && let Some(t) = rock.line_segment_intersects(sweep.origin, beam_end)
            {
                max_t = max_t.min(t);
            }
        }
        let effective_length = sweep.length * max_t;

        let beam_width = beam.beam_width();
        let damage = beam.damage();
        let mut kill_count = 0u32;

        // Damage targets in current sweep position (skip already-hit)
        for (entity, transform, mut health, mut temp_hp, has_spell_shield) in targets.iter_mut() {
            if has_spell_shield || sweep.hit_entities.contains(&entity) {
                continue;
            }
            if beam.contains_point(transform.translation, beam_width) {
                let proj = (transform.translation - sweep.origin).dot(rotated_dir);
                if proj <= effective_length {
                    sweep.hit_entities.insert(entity);
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

                    if was_alive && health.current <= 0.0 {
                        kill_count += 1;

                        // Necrotic explosion on kills during sweep
                        if sweep.talent_params.necrotic_explosion {
                            let explosion_damage =
                                damage * constants::NECROTIC_EXPLOSION_DAMAGE_PERCENT;
                            spawn_necrotic_explosion(
                                &mut commands,
                                transform.translation,
                                explosion_damage,
                                &visual_assets,
                                &mut materials,
                            );
                        }
                    }
                }
            }
        }

        if kill_count > 0
            && let Some(ref mut progress) = talent_progress
        {
            progress.increment(Spell::FingerOfDeath, kill_count);
        }

        // Spawn visual beam trail at intervals (every ~0.05s) instead of every frame
        let spawn_interval = 0.05;
        let prev_step = ((sweep.time_elapsed - dt) / spawn_interval) as u32;
        let curr_step = (sweep.time_elapsed / spawn_interval) as u32;
        if curr_step > prev_step || sweep.time_elapsed < dt {
            let material = materials
                .get(&visual_assets.finger_of_death_beam)
                .cloned()
                .unwrap_or_default();
            let instance = materials.add(material);

            let mut sweep_beam = FingerOfDeathBeam::with_talents(
                sweep.origin,
                rotated_dir,
                sweep.length,
                sweep.empowerment,
                sweep.talent_params.clone(),
            );
            sweep_beam.has_fired = true;
            sweep_beam.cast_progress = 1.0;
            sweep_beam.time_since_fired = 0.0;

            commands.spawn((
                sweep_beam,
                Mesh3d(visual_assets.cross_plane_triangle.clone()),
                MeshMaterial3d(instance),
                Transform::from_translation(sweep.origin),
                OnGameplayScreen,
            ));
        }
    }
}

/// Ticks down the Finger of Death cooldown timer.
pub fn tick_finger_of_death_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut FingerOfDeathCooldown)>,
) {
    for (entity, mut cooldown) in &mut cooldowns {
        cooldown.remaining -= time.delta_secs();
        if cooldown.remaining <= 0.0 {
            commands.entity(entity).remove::<FingerOfDeathCooldown>();
        }
    }
}

/// Updates Finger of Death beam visuals based on cast progress and fire state.
pub fn update_finger_of_death_beam_visuals(
    time: Res<Time>,
    mut beam_query: Query<(
        &mut FingerOfDeathBeam,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<crate::config::GameConfig>,
) {
    let t = time.elapsed_secs();
    let is_excremage = config.wizard_type == crate::config::WizardType::Excremage;

    for (mut beam, mut transform, material_handle) in beam_query.iter_mut() {
        if beam.has_fired {
            beam.time_since_fired += time.delta_secs();
        }

        let visual_len = beam.length + constants::BEAM_VISUAL_OVERSHOOT;

        // Position at origin, rotate to point along beam direction
        transform.translation = beam.origin;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

        // Pulsing width
        let pulse = 1.0
            + constants::BEAM_PULSE_AMPLITUDE
                * (t * constants::BEAM_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let base_width = if beam.has_fired {
            beam.beam_width_fired()
        } else {
            beam.beam_width()
        };
        let beam_width = base_width * pulse;

        // During cast: scale up with cast_progress; after fire: full size
        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        transform.scale = Vec3::new(
            beam_width * progress_scale,
            visual_len * progress_scale,
            beam_width * progress_scale,
        );

        // Color cycling and fade
        if let Some(mat) = materials.get_mut(&material_handle.0) {
            if beam.has_fired {
                let fade = (1.0 - beam.time_since_fired / constants::POST_FIRE_DURATION).max(0.0);

                if is_excremage {
                    let srgba = EXCREMAGE_BROWN.to_srgba();
                    mat.base_color = Color::srgb(srgba.red, srgba.green, srgba.blue);
                    mat.emissive = LinearRgba::new(
                        srgba.red * 2.0 * fade,
                        srgba.green * 0.5 * fade,
                        srgba.blue * 0.2 * fade,
                        1.0,
                    );
                } else {
                    // Purple color cycling: dark purple -> bright violet -> pale purple
                    let cycle = (t * constants::COLOR_CYCLE_SPEED).sin() * 0.5 + 0.5;
                    let r = (0.5 + cycle * 0.5) * fade;
                    let g = (0.0 + cycle * 0.3) * fade;
                    let b = (0.6 + cycle * 0.4) * fade;
                    mat.base_color = Color::srgb(r, g, b);
                    mat.emissive = LinearRgba::new(
                        (1.5 + cycle * 1.5) * fade,
                        (0.0 + cycle * 0.8) * fade,
                        (2.5 + cycle * 2.0) * fade,
                        1.0,
                    );
                }
            } else {
                // During cast: growing intensity with cast_progress
                let intensity = beam.cast_progress;

                if is_excremage {
                    let srgba = EXCREMAGE_BROWN_DARK.to_srgba();
                    mat.base_color = Color::srgb(srgba.red, srgba.green, srgba.blue);
                    mat.emissive = LinearRgba::new(
                        srgba.red * intensity,
                        srgba.green * 0.3 * intensity,
                        srgba.blue * 0.1 * intensity,
                        1.0,
                    );
                } else {
                    mat.base_color = Color::srgb(0.6 * intensity, 0.0, 0.8 * intensity);
                    mat.emissive = LinearRgba::new(1.5 * intensity, 0.0, 2.5 * intensity, 1.0);
                }
            }
        }
    }
}

/// Updates necrotic vein particles: meander, fade color, shrink, despawn when expired.
pub fn update_necrotic_veins(
    mut commands: Commands,
    time: Res<Time>,
    mut veins: Query<(
        Entity,
        &mut NecroticVein,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut vein, mut transform, material_handle) in veins.iter_mut() {
        vein.time_alive += dt;

        if vein.time_alive >= vein.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = vein.time_alive / vein.lifetime;

        // Meander: apply lateral sine-wave offset to velocity direction
        let wander_offset = (t * constants::VEIN_WANDER_FREQUENCY + vein.wander_phase).sin()
            * constants::VEIN_WANDER_AMPLITUDE
            * dt;
        let lateral = Vec3::new(-vein.velocity.z, 0.0, vein.velocity.x).normalize_or_zero();
        transform.translation += vein.velocity * dt + lateral * wander_offset;

        // Clamp Y to ground level
        transform.translation.y = constants::VEIN_Y_POSITION;

        // Shrink over lifetime
        let scale = vein.base_size * (1.0 - progress);
        transform.scale = Vec3::splat(scale);

        // Animate material: purple → dark purple → black, alpha fading out
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let r = 0.6 * (1.0 - progress);
            let g = 0.0;
            let b = 0.8 * (1.0 - progress * 0.5);
            let alpha = 0.8 * (1.0 - progress);
            material.base_color = Color::srgba(r, g, b, alpha);
            let em_scale = 1.0 - progress;
            material.emissive = LinearRgba::new(1.5 * em_scale, 0.0, 2.0 * em_scale, 1.0);
        }
    }
}

/// Updates the glow aura to follow its beam with shimmer and pulsing.
pub fn update_finger_of_death_glow(
    mut glow_query: Query<(&FingerOfDeathGlow, &mut Transform), Without<FingerOfDeathBeam>>,
    beam_query: Query<&FingerOfDeathBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (glow, mut transform) in glow_query.iter_mut() {
        let Ok(beam) = beam_query.get(glow.beam_entity) else {
            continue;
        };

        let visual_len = beam.length + constants::BEAM_VISUAL_OVERSHOOT;

        transform.translation = beam.origin;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

        // Glow pulse + shimmer jitter
        let pulse = 1.0
            + constants::GLOW_PULSE_AMPLITUDE
                * (t * constants::GLOW_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let shimmer = constants::SHIMMER_AMPLITUDE
            * ((t * constants::SHIMMER_FREQ_A).sin() + (t * constants::SHIMMER_FREQ_B).cos());
        let base_width = if beam.has_fired {
            beam.beam_width_fired()
        } else {
            beam.beam_width()
        };
        let glow_width = base_width * constants::GLOW_WIDTH_MULTIPLIER * (pulse + shimmer);

        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        transform.scale = Vec3::new(
            glow_width * progress_scale,
            visual_len * progress_scale,
            glow_width * progress_scale,
        );
    }
}

/// Despawns glow entities when their beam no longer exists.
pub fn cleanup_finger_of_death_glow(
    mut commands: Commands,
    glow_query: Query<(Entity, &FingerOfDeathGlow)>,
    beam_query: Query<Entity, With<FingerOfDeathBeam>>,
) {
    for (glow_entity, glow) in glow_query.iter() {
        if beam_query.get(glow.beam_entity).is_err() {
            commands.entity(glow_entity).try_despawn();
        }
    }
}

/// Updates the origin flare to pulse at the beam origin.
pub fn update_finger_of_death_flare(
    mut flare_query: Query<(&FingerOfDeathFlare, &mut Transform)>,
    beam_query: Query<&FingerOfDeathBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (flare, mut transform) in flare_query.iter_mut() {
        let Ok(beam) = beam_query.get(flare.beam_entity) else {
            continue;
        };

        transform.translation = beam.origin;

        let pulse = 1.0
            + constants::FLARE_PULSE_AMPLITUDE
                * (t * constants::FLARE_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let radius = constants::FLARE_RADIUS * pulse;

        // Scale with cast progress so flare grows during cast
        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        transform.scale = Vec3::splat(radius * progress_scale);
    }
}

/// Despawns flare entities when their beam no longer exists.
pub fn cleanup_finger_of_death_flare(
    mut commands: Commands,
    flare_query: Query<(Entity, &FingerOfDeathFlare)>,
    beam_query: Query<Entity, With<FingerOfDeathBeam>>,
) {
    for (flare_entity, flare) in flare_query.iter() {
        if beam_query.get(flare.beam_entity).is_err() {
            commands.entity(flare_entity).try_despawn();
        }
    }
}

/// Updates necrotic pulse ring: expand scale, fade alpha, despawn when expired.
pub fn update_necrotic_pulse(
    mut commands: Commands,
    time: Res<Time>,
    mut pulses: Query<(
        Entity,
        &mut NecroticPulse,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut pulse, mut transform, material_handle) in pulses.iter_mut() {
        pulse.time_alive += dt;

        if pulse.time_alive >= pulse.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = pulse.time_alive / pulse.lifetime;

        // Expand from small to max_radius
        let radius = pulse.max_radius * progress;
        transform.scale = Vec3::splat(radius);

        // Fade alpha from 0.5 to 0
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = 0.5 * (1.0 - progress);
            material.base_color = Color::srgba(0.4, 0.0, 0.6, alpha);
        }
    }
}

/// Cleans up Finger of Death beams after firing or cancellation.
pub fn cleanup_finger_of_death_beams(
    mut commands: Commands,
    beams: Query<(Entity, &FingerOfDeathBeam)>,
    wizard_query: Query<&CastingState, With<LocalWizard>>,
) {
    let resting = wizard_query
        .single()
        .map(|state| matches!(state, CastingState::Resting))
        .unwrap_or(true);

    for (entity, beam) in beams.iter() {
        let should_despawn = if beam.has_fired {
            beam.time_since_fired >= constants::POST_FIRE_DURATION
        } else {
            resting
        };

        if should_despawn {
            commands.entity(entity).try_despawn();
        }
    }
}
