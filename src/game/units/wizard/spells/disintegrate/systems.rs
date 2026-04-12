use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{
    BeamEclipse, BeamGlow, BeamOriginFlare, BeamSmoke, DisintegrateBeam, DisintegrateParticle,
    SearingFinaleDetonation,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Health, Hitbox, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::audio::{self, ChannelingSfx, SpellSfxAssets};
use crate::game::units::wizard::spells::fireball;
use crate::game::units::wizard::spells::utils::{UniqueHitTracker, get_cursor_world_position};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

/// Talent configuration computed once from ActiveTalents.
pub(crate) struct TalentConfig {
    pub(crate) width_multiplier: f32,
    pub(crate) damage_multiplier: f32,
    pub(crate) mana_cost_multiplier: f32,
    pub(crate) forked: bool,
    pub(crate) escalating: bool,
    pub(crate) sweeping: bool,
    pub(crate) searing_finale: bool,
    pub(crate) resonance: bool,
    pub(crate) annihilation: bool,
}

impl Default for TalentConfig {
    fn default() -> Self {
        Self {
            width_multiplier: 1.0,
            damage_multiplier: 1.0,
            mana_cost_multiplier: 1.0,
            forked: false,
            escalating: false,
            sweeping: false,
            searing_finale: false,
            resonance: false,
            annihilation: false,
        }
    }
}

pub(crate) fn compute_talent_config(active_talents: Option<&ActiveTalents>) -> TalentConfig {
    let talents = active_talents;
    let t1 = talents.and_then(|t| t.get_selection(Spell::Disintegrate, 0));
    let t2 = talents.and_then(|t| t.get_selection(Spell::Disintegrate, 1));
    let t3 = talents.and_then(|t| t.get_selection(Spell::Disintegrate, 2));

    let mut cfg = TalentConfig::default();

    // Tier 1
    match t1 {
        Some(0) => {
            // Focused Lens
            cfg.width_multiplier *= constants::FOCUSED_LENS_WIDTH_MULT;
            cfg.damage_multiplier *= constants::FOCUSED_LENS_DAMAGE_MULT;
        }
        Some(1) => {
            // Unfocused Beam
            cfg.width_multiplier *= constants::UNFOCUSED_BEAM_WIDTH_MULT;
            cfg.damage_multiplier *= constants::UNFOCUSED_BEAM_DAMAGE_MULT;
        }
        Some(2) => {
            // Efficient Channeling
            cfg.mana_cost_multiplier *= constants::EFFICIENT_CHANNELING_MANA_MULT;
        }
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => {
            // Forked Beam
            cfg.forked = true;
            cfg.damage_multiplier *= constants::FORKED_DAMAGE_MULT;
        }
        Some(1) => {
            // Escalating Intensity
            cfg.escalating = true;
        }
        Some(2) => {
            // Sweeping Destruction (+100% damage since player loses aim control)
            cfg.sweeping = true;
            cfg.damage_multiplier *= constants::SWEEPING_DAMAGE_MULT;
        }
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => {
            // Annihilation Beam
            cfg.width_multiplier *= constants::ANNIHILATION_WIDTH_MULT;
            cfg.damage_multiplier *= constants::ANNIHILATION_DAMAGE_MULT;
            cfg.mana_cost_multiplier *= constants::ANNIHILATION_MANA_MULT;
            cfg.annihilation = true;
        }
        Some(1) => {
            // Searing Finale
            cfg.searing_finale = true;
        }
        Some(2) => {
            // Unstable Resonance
            cfg.resonance = true;
        }
        _ => {}
    }

    cfg
}

/// Action the shared logic requests the wrapper to perform on beams.
enum BeamAction {
    /// Update existing beam with new origin, direction, length.
    UpdateBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
    },
    /// Spawn a new beam.
    SpawnBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        empowerment: f32,
    },
    /// Despawn all beams for this wizard.
    DespawnAll,
    /// No beam action needed.
    None,
}

/// Result from the shared casting logic.
struct CastingResult {
    beam_action: BeamAction,
}

/// Local wizard disintegrate casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_disintegrate_casting(
    time: Res<Time>,
    mut left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (&mut CastingState, &mut Mana, &PrimedSpell, &Wizard),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut beams: Query<(Entity, &mut DisintegrateBeam), Without<CrystalSpawn>>,
    visual_assets: Res<SpellVisualAssets>,
    glow_query: Query<Entity, With<BeamGlow>>,
    flare_query: Query<Entity, With<BeamOriginFlare>>,
    particle_query: Query<Entity, With<DisintegrateParticle>>,
    eclipse_query: Query<Entity, With<BeamEclipse>>,
    channeling_sfx_query: Query<Entity, With<ChannelingSfx>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
) {
    let released = left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((mut casting_state, mut mana, primed_spell, wizard)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::Disintegrate {
        return;
    }

    let talent_cfg = compute_talent_config(active_talents.as_deref());
    let has_existing_beam = beams.iter().next().is_some();

    let result = disintegrate_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        wizard,
        has_existing_beam,
        talent_cfg.mana_cost_multiplier,
    );

    match result.beam_action {
        BeamAction::UpdateBeam {
            origin,
            direction,
            length,
        } => {
            // Update ALL existing beams (supports forked multi-beam)
            for (_, mut beam) in beams.iter_mut() {
                // Annihilation beams are position-locked — skip origin/direction/length updates
                if beam.annihilation {
                    continue;
                }
                beam.origin = origin;
                beam.length = length;
                // For sweeping beams, don't update direction directly — the sweep system handles it.
                // For non-sweeping beams, update direction (applying fan offset if forked).
                if beam.sweeping {
                    beam.sweep_center_direction = direction;
                } else if beam.fan_offset_angle.abs() > 0.001 {
                    // Forked: apply fan offset from base direction
                    let up = Vec3::Y;
                    let rotated = Quat::from_axis_angle(up, beam.fan_offset_angle) * direction;
                    beam.direction = rotated;
                } else {
                    beam.direction = direction;
                }
            }
        }
        BeamAction::SpawnBeam {
            mut origin,
            mut direction,
            mut length,
            empowerment,
        } => {
            vfx::systems::spawn_school_flare(
                &mut commands,
                &visual_assets,
                vfx::systems::SpellSchool::Arcane,
                time.elapsed_secs(),
            );
            // Annihilation Beam: shoot from the sky above the clamped target
            let mut annihilation_forward = Vec3::X;
            if talent_cfg.annihilation {
                // Use the already range-clamped target from casting logic
                let ground_target = origin + direction * length;
                let wizard_xz = Vec3::new(SPELL_ORIGIN.x, 0.0, SPELL_ORIGIN.z);
                let target_xz = Vec3::new(ground_target.x, 0.0, ground_target.z);
                annihilation_forward = (target_xz - wizard_xz).normalize_or(Vec3::X);

                origin = Vec3::new(
                    ground_target.x,
                    constants::ANNIHILATION_SKY_HEIGHT,
                    ground_target.z,
                );
                direction = Vec3::NEG_Y;
                length = constants::ANNIHILATION_SKY_HEIGHT;
            }

            if talent_cfg.forked {
                // Spawn 3 beams in a fan pattern
                let offsets = [
                    -constants::FORKED_FAN_HALF_ANGLE,
                    0.0,
                    constants::FORKED_FAN_HALF_ANGLE,
                ];
                for &offset in &offsets {
                    let (beam_origin, beam_dir, beam_len) = if talent_cfg.annihilation {
                        // Shared origin, angled directions to offset ground targets
                        let perp = Vec3::new(-annihilation_forward.z, 0.0, annihilation_forward.x);
                        let lateral = offset / constants::FORKED_FAN_HALF_ANGLE;
                        let offset_xz = perp * lateral * constants::ANNIHILATION_FORKED_SPREAD;
                        let ground_target =
                            Vec3::new(origin.x + offset_xz.x, 0.0, origin.z + offset_xz.z);
                        let to_target = ground_target - origin;
                        (origin, to_target.normalize(), to_target.length())
                    } else {
                        (
                            origin,
                            Quat::from_axis_angle(Vec3::Y, offset) * direction,
                            length,
                        )
                    };
                    // Shared cast_pos for all annihilation beams so they sweep together
                    let cast_pos = Vec3::new(origin.x, 0.0, origin.z);
                    spawn_beam_with_talents(
                        &mut commands,
                        &visual_assets,
                        beam_origin,
                        beam_dir,
                        beam_len,
                        empowerment,
                        &talent_cfg,
                        offset,
                        cast_pos,
                        annihilation_forward,
                    );
                }
            } else {
                let cast_pos = Vec3::new(origin.x, 0.0, origin.z);
                spawn_beam_with_talents(
                    &mut commands,
                    &visual_assets,
                    origin,
                    direction,
                    length,
                    empowerment,
                    &talent_cfg,
                    0.0,
                    cast_pos,
                    annihilation_forward,
                );
            }
            audio::play_looping_sfx(&mut commands, &sfx.disintegrate_channel, &game_config, &sfx);
        }
        BeamAction::DespawnAll => {
            // Spawn searing finale detonations before despawning
            if talent_cfg.searing_finale {
                for (_, beam) in beams.iter() {
                    spawn_searing_finale(&mut commands, &visual_assets, beam);
                    // Play fireball impact sound at beam tip
                    let tip = beam.origin + beam.direction * beam.current_length();
                    audio::play_impact_sfx(
                        &mut commands,
                        &sfx.fireball_impact,
                        tip,
                        &game_config,
                        &sfx,
                    );
                }
            }
            despawn_all_beam_visuals(
                &mut commands,
                &beams,
                &glow_query,
                &flare_query,
                &particle_query,
                &eclipse_query,
            );
            for entity in channeling_sfx_query.iter() {
                commands.entity(entity).try_despawn();
            }
        }
        BeamAction::None => {}
    }
}

/// Core disintegrate casting logic.
///
/// Takes extracted data from queries and returns actions for the wrapper to apply.
#[allow(clippy::too_many_arguments)]
fn disintegrate_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    has_existing_beam: bool,
    mana_cost_multiplier: f32,
) -> CastingResult {
    let mut result = CastingResult {
        beam_action: BeamAction::None,
    };

    let wizard_pos = SPELL_ORIGIN;

    // Check for release
    if input.just_released {
        casting_state.cancel();
        result.beam_action = BeamAction::DespawnAll;
        return result;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());

            let mana_cost =
                constants::MANA_COST_PER_SECOND * mana_cost_multiplier * time.delta_secs();

            if mana.consume(mana_cost) {
                if let Some(target_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    let to_target = target_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        target_pos
                    };

                    let direction = (clamped_target - beam_origin).normalize();
                    let beam_length = (clamped_target - beam_origin)
                        .length()
                        .min(constants::BEAM_LENGTH);

                    if has_existing_beam {
                        result.beam_action = BeamAction::UpdateBeam {
                            origin: beam_origin,
                            direction,
                            length: beam_length,
                        };
                    } else {
                        result.beam_action = BeamAction::SpawnBeam {
                            origin: beam_origin,
                            direction,
                            length: beam_length,
                            empowerment: primed_spell.empowerment,
                        };
                    }
                }
            } else {
                casting_state.cancel();
                result.beam_action = BeamAction::DespawnAll;
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                casting_state.start_channeling();

                if let Some(target_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    let to_target = target_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        target_pos
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
                    };
                }
            }
        }
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && mana.can_afford(constants::MANA_COST_PER_SECOND * mana_cost_multiplier * 0.1)
            {
                casting_state.start_cast();
            }
        }
    }

    result
}

/// System that applies damage to all units hit by disintegrate beams.
///
/// This is a high-risk spell that damages both attackers and defenders,
/// but not the wizard.
pub fn apply_disintegrate_damage(
    mut commands: Commands,
    mut beam_query: Query<(&mut DisintegrateBeam, &mut UniqueHitTracker)>,
    mut target_query: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<Wizard>,
    >,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
) {
    for (mut beam, mut hit_tracker) in beam_query.iter_mut() {
        beam.update_damage_timer(time.delta_secs());
        beam.update_time_alive(time.delta_secs());

        // Advance resonance timer and spawn mini-fireballs along beam
        if beam.resonance {
            beam.resonance_timer += time.delta_secs();
            if beam.resonance_timer >= constants::MINI_FIREBALL_INTERVAL {
                beam.resonance_timer -= constants::MINI_FIREBALL_INTERVAL;
                let current_len = beam.current_length();
                if current_len > 1.0 {
                    let scale = beam.mini_spell_scale;
                    let mini_damage = fireball::constants::TOTAL_DAMAGE
                        * constants::MINI_FIREBALL_DAMAGE_FRACTION
                        * scale;
                    let explosion_radius = fireball::constants::EXPLOSION_RADIUS
                        * constants::MINI_FIREBALL_DAMAGE_FRACTION
                        * scale;
                    let visual_radius = 8.0 * scale;
                    // Annihilation beams: spawn fireball at impact point (ground level),
                    // flying outward in XZ. Normal beams: spawn at origin, fly along beam.
                    let (spawn_pos, velocity) = if beam.annihilation {
                        let tip = beam.origin + beam.direction * current_len;
                        let ground_pos = Vec3::new(tip.x, 0.0, tip.z);
                        // Random outward XZ direction
                        let angle = beam.resonance_timer * 137.5; // pseudo-random angle
                        let xz_dir = Vec3::new(angle.cos(), 0.0, angle.sin()).normalize();
                        (
                            ground_pos,
                            xz_dir * fireball::constants::PROJECTILE_SPEED * 0.5,
                        )
                    } else {
                        (
                            beam.origin,
                            beam.direction * fireball::constants::PROJECTILE_SPEED,
                        )
                    };
                    fireball::systems::spawn_fireball_entity(
                        &mut commands,
                        &visual_assets,
                        spawn_pos,
                        velocity,
                        mini_damage,
                        constants::DAMAGE_TYPE,
                        explosion_radius,
                        fireball::constants::PROJECTILE_COLLISION_RADIUS * scale,
                        beam.empowerment,
                        visual_radius,
                    );
                }
            }
        }

        if beam.should_damage() {
            let mut hit_count = 0_u32;
            let damage = beam.damage_per_tick();

            for (entity, transform, hitbox, mut health, mut temp_hp, has_spell_shield) in
                target_query.iter_mut()
            {
                if beam.intersects_hitbox_cylinder(
                    transform.translation,
                    hitbox.radius,
                    hitbox.height,
                ) {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        constants::DAMAGE_TYPE,
                        has_spell_shield,
                    );
                    if hit_tracker.track_hit(entity) {
                        hit_count += 1;
                    }
                }
            }

            if hit_count > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::Disintegrate, hit_count);
            }

            beam.reset_damage_timer();
        }
    }
}

/// System that despawns beams when wizard is not actively casting/channeling disintegrate.
///
/// Checks CastingState directly to avoid deferred command timing issues.
/// Excludes crystal-spawned beams (those with CrystalSpawn) — they're managed by the crystal.
#[allow(clippy::too_many_arguments)]
pub fn cleanup_beams_on_cancel(
    mut commands: Commands,
    wizard_query: Query<&CastingState, With<LocalWizard>>,
    beam_query: Query<Entity, (With<DisintegrateBeam>, Without<CrystalSpawn>)>,
    glow_query: Query<(Entity, &BeamGlow)>,
    flare_query: Query<(Entity, &BeamOriginFlare)>,
    particle_query: Query<Entity, With<DisintegrateParticle>>,
    eclipse_query: Query<(Entity, &BeamEclipse)>,
    channeling_sfx_query: Query<Entity, With<ChannelingSfx>>,
) {
    if let Ok(casting_state) = wizard_query.single()
        && matches!(casting_state, CastingState::Resting)
    {
        // Collect wizard beam entities for filtering visuals
        let wizard_beams: Vec<Entity> = beam_query.iter().collect();
        for entity in &wizard_beams {
            commands.entity(*entity).try_despawn();
        }
        // Only despawn visuals that belong to wizard beams
        for (entity, glow) in &glow_query {
            if wizard_beams.contains(&glow.beam_entity) {
                commands.entity(entity).try_despawn();
            }
        }
        for (entity, flare) in &flare_query {
            if wizard_beams.contains(&flare.beam_entity) {
                commands.entity(entity).try_despawn();
            }
        }
        for entity in particle_query.iter() {
            commands.entity(entity).try_despawn();
        }
        for (entity, eclipse) in &eclipse_query {
            if wizard_beams.contains(&eclipse.beam_entity) {
                commands.entity(entity).try_despawn();
            }
        }
        for entity in channeling_sfx_query.iter() {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns a beam entity with custom damage per tick (for crystal use),
/// plus glow and flare sibling entities.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_beam_with_damage(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
    damage_per_tick: f32,
    talent_cfg: Option<&TalentConfig>,
    mini_spell_scale: f32,
    fan_offset_angle: f32,
) -> Entity {
    let mut beam = DisintegrateBeam::new(origin, direction, length, empowerment);
    beam.damage_per_tick_override = Some(damage_per_tick);
    if let Some(cfg) = talent_cfg {
        beam.width_multiplier = cfg.width_multiplier;
        beam.damage_multiplier = cfg.damage_multiplier;
        beam.escalating = cfg.escalating;
        beam.resonance = cfg.resonance;
    }
    beam.mini_spell_scale = mini_spell_scale;
    beam.fan_offset_angle = fan_offset_angle;
    beam.ground_collision = true;
    // Crystal beams only get the core beam mesh — no glow, flare, or eclipse.
    spawn_beam_core(commands, assets, beam)
}

/// Spawns a beam with talent configuration applied.
#[allow(clippy::too_many_arguments)]
fn spawn_beam_with_talents(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
    cfg: &TalentConfig,
    fan_offset_angle: f32,
    annihilation_cast_pos: Vec3,
    annihilation_sweep_dir: Vec3,
) {
    let mut beam = DisintegrateBeam::new(origin, direction, length, empowerment);
    beam.width_multiplier = cfg.width_multiplier;
    beam.damage_multiplier = cfg.damage_multiplier;
    beam.fan_offset_angle = fan_offset_angle;
    beam.escalating = cfg.escalating;
    beam.sweeping = cfg.sweeping;
    beam.searing_finale = cfg.searing_finale;
    beam.resonance = cfg.resonance;
    beam.annihilation = cfg.annihilation;
    beam.annihilation_cast_pos = annihilation_cast_pos;
    if cfg.sweeping {
        if cfg.annihilation {
            // For sky beams, sweep_center_direction stores the XZ forward reference
            beam.sweep_center_direction = annihilation_sweep_dir;
        } else {
            beam.sweep_center_direction = direction;
        }
        beam.sweep_direction = 1.0;
    }
    spawn_beam_visuals(commands, assets, beam);
}

/// Spawns only the core beam entity (mesh + component). Used by crystal beams
/// which don't need glow, flare, or eclipse siblings.
fn spawn_beam_core(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: DisintegrateBeam,
) -> Entity {
    let midpoint = beam.origin + beam.direction * (beam.length / 2.0);

    commands
        .spawn((
            beam,
            UniqueHitTracker::default(),
            Mesh3d(assets.cross_plane_triangle.clone()),
            MeshMaterial3d(assets.disintegrate_beam.clone()),
            Transform::from_translation(midpoint),
            OnGameplayScreen,
        ))
        .id()
}

/// Spawns the core beam entity plus glow, flare, and eclipse siblings.
/// Used by wizard-cast disintegrate beams.
fn spawn_beam_visuals(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: DisintegrateBeam,
) -> Entity {
    let midpoint = beam.origin + beam.direction * (beam.length / 2.0);
    let beam_entity = spawn_beam_core(commands, assets, beam);

    // Glow triangle sibling (wider, semi-transparent)
    commands.spawn((
        BeamGlow { beam_entity },
        Mesh3d(assets.cross_plane_triangle.clone()),
        MeshMaterial3d(assets.disintegrate_glow.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    // Origin flare circle sibling (uses cross-plane sphere for visibility from all angles)
    commands.spawn((
        BeamOriginFlare { beam_entity },
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.disintegrate_flare.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    // Ground eclipse at beam impact point
    commands.spawn((
        BeamEclipse { beam_entity },
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.disintegrate_eclipse.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.05, 0.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        OnGameplayScreen,
    ));

    beam_entity
}

/// Helper to despawn all beam-related visual entities.
fn despawn_all_beam_visuals(
    commands: &mut Commands,
    beams: &Query<(Entity, &mut DisintegrateBeam), Without<CrystalSpawn>>,
    glow_query: &Query<Entity, With<BeamGlow>>,
    flare_query: &Query<Entity, With<BeamOriginFlare>>,
    particle_query: &Query<Entity, With<DisintegrateParticle>>,
    eclipse_query: &Query<Entity, With<BeamEclipse>>,
) {
    for (entity, _) in beams.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in glow_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in flare_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in particle_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in eclipse_query.iter() {
        commands.entity(entity).try_despawn();
    }
}

/// System that updates beam cylinder transform to match beam data,
/// with pulsing width and color cycling.
pub fn update_beam_visuals(
    mut beam_query: Query<(
        &DisintegrateBeam,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (beam, mut transform, material_handle) in beam_query.iter_mut() {
        let current_len = beam.current_length();
        // Crystal beams (ground_collision) shouldn't overshoot past their range.
        let overshoot = if beam.ground_collision {
            0.0
        } else {
            constants::BEAM_VISUAL_OVERSHOOT
        };
        let visual_len = current_len + overshoot;

        transform.translation = beam.origin;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

        // Pulsing width — use beam.beam_width() which includes talent multipliers
        let pulse = 1.0
            + constants::BEAM_PULSE_AMPLITUDE
                * (t * constants::BEAM_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let beam_width = beam.beam_width() * pulse;
        transform.scale = Vec3::new(beam_width, visual_len, beam_width);

        // Color cycling: orange -> yellow -> white -> yellow -> orange
        if let Some(mat) = materials.get_mut(material_handle) {
            let cycle = (t * constants::COLOR_CYCLE_SPEED).sin() * 0.5 + 0.5; // 0..1
            // Interpolate emissive: orange(3,1.5,0.2) -> white(5,4.5,4)
            let r = 3.0 + cycle * 2.0;
            let g = 1.5 + cycle * 3.0;
            let b = 0.2 + cycle * 3.8;
            mat.emissive = bevy::color::LinearRgba::new(r, g, b, 1.0);

            // Also shift base color slightly
            let base_r = 1.0;
            let base_g = 0.6 + cycle * 0.35;
            let base_b = 0.1 + cycle * 0.6;
            mat.base_color = Color::srgb(base_r, base_g, base_b);
        }
    }
}

/// System that positions and animates the outer glow cylinder to follow its beam.
pub fn update_beam_glow(
    mut glow_query: Query<(&BeamGlow, &mut Transform)>,
    beam_query: Query<&DisintegrateBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (glow, mut transform) in glow_query.iter_mut() {
        let Ok(beam) = beam_query.get(glow.beam_entity) else {
            continue;
        };

        let current_len = beam.current_length();
        let visual_len = current_len + constants::BEAM_VISUAL_OVERSHOOT;

        transform.translation = beam.origin;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

        // Glow pulse + shimmer jitter from incommensurate frequencies
        let pulse = 1.0
            + constants::GLOW_PULSE_AMPLITUDE
                * (t * constants::GLOW_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let shimmer = constants::SHIMMER_AMPLITUDE
            * ((t * constants::SHIMMER_FREQ_A).sin() + (t * constants::SHIMMER_FREQ_B).cos());
        let glow_width = beam.beam_width() * constants::GLOW_WIDTH_MULTIPLIER * (pulse + shimmer);
        transform.scale = Vec3::new(glow_width, visual_len, glow_width);
    }
}

/// System that positions and animates the origin flare sphere.
pub fn update_beam_origin_flare(
    mut flare_query: Query<(&BeamOriginFlare, &mut Transform)>,
    beam_query: Query<&DisintegrateBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (flare, mut transform) in flare_query.iter_mut() {
        let Ok(beam) = beam_query.get(flare.beam_entity) else {
            continue;
        };

        // Annihilation beams originate from Y=2000 — hide the flare.
        if beam.annihilation {
            transform.scale = Vec3::ZERO;
            continue;
        }

        transform.translation = beam.origin;

        // Pulsing scale
        let pulse = 1.0
            + constants::FLARE_PULSE_AMPLITUDE
                * (t * constants::FLARE_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let radius = constants::FLARE_RADIUS * pulse;
        transform.scale = Vec3::splat(radius);
    }
}

/// System that positions and scales the ground eclipse at the beam's impact point.
///
/// The eclipse is an ellipse matching the shadow a sphere would cast onto the
/// ground plane. Its major axis stretches along the beam's ground projection by
/// `1 / sin(elevation)`. Pulses in sync with the beam.
pub fn update_beam_eclipse(
    mut eclipse_query: Query<(&BeamEclipse, &mut Transform)>,
    beam_query: Query<&DisintegrateBeam>,
    wizard_query: Query<&Wizard>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    let spell_range = wizard_query
        .iter()
        .next()
        .map(|w| w.spell_range)
        .unwrap_or(500.0);

    for (eclipse, mut transform) in eclipse_query.iter_mut() {
        let Ok(beam) = beam_query.get(eclipse.beam_entity) else {
            continue;
        };

        // Hide eclipse when beam angle is too steep (nearly horizontal)
        if beam.direction.y.abs() < 0.15 {
            transform.scale = Vec3::ZERO;
            continue;
        }

        let Some((eclipse_center, major_axis, _minor_axis, clipped_major, minor_radius)) =
            beam.eclipse_geometry(spell_range)
        else {
            transform.scale = Vec3::ZERO;
            continue;
        };

        transform.translation = Vec3::new(eclipse_center.x, 2.0, eclipse_center.z);

        // Pulse in sync with the beam core
        let pulse = 1.0
            + constants::BEAM_PULSE_AMPLITUDE
                * (t * constants::BEAM_PULSE_FREQUENCY * std::f32::consts::TAU).sin();

        let major_pulsed = clipped_major * pulse;
        let minor_pulsed = minor_radius * pulse;

        // Orient the ellipse so major axis aligns with beam's ground projection
        let theta = major_axis.z.atan2(major_axis.x);

        // Lay circle flat (XY → XZ), then rotate around Y to align stretch
        transform.rotation =
            Quat::from_rotation_y(theta) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        transform.scale = Vec3::new(major_pulsed, minor_pulsed, 1.0);
    }
}

/// System that spawns impact particles at the beam tip.
pub fn spawn_impact_particles(
    mut commands: Commands,
    beam_query: Query<&DisintegrateBeam>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < constants::PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *timer -= constants::PARTICLE_SPAWN_INTERVAL;

    for beam in beam_query.iter() {
        let current_len = beam.current_length();
        if current_len < 1.0 {
            continue;
        }

        let tip = beam.origin + beam.direction * current_len;

        // Annihilation beams: skip particles when tip is still high in the sky (growth phase).
        if beam.annihilation && tip.y > 50.0 {
            continue;
        }

        // Build a perpendicular basis for random spread
        let up = if beam.direction.y.abs() > 0.9 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let right = beam.direction.cross(up).normalize();
        let forward = right.cross(beam.direction).normalize();

        for i in 0..constants::PARTICLE_COUNT_PER_SPAWN {
            // Spread particles in a circle perpendicular to the beam
            let angle = (i as f32 / constants::PARTICLE_COUNT_PER_SPAWN as f32)
                * std::f32::consts::TAU
                + time.elapsed_secs() * 17.3; // rotating offset
            let spread = right * angle.cos() + forward * angle.sin();
            let velocity = spread * constants::PARTICLE_SPEED;

            commands.spawn((
                DisintegrateParticle {
                    velocity,
                    time_alive: 0.0,
                },
                Mesh3d(visual_assets.particle_quad.clone()),
                MeshMaterial3d(visual_assets.disintegrate_particle.clone()),
                Transform::from_translation(tip)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(constants::PARTICLE_SIZE)),
                OnGameplayScreen,
            ));
        }
    }
}

/// System that moves, shrinks, and despawns impact particles.
pub fn update_impact_particles(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut DisintegrateParticle, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform) in particle_query.iter_mut() {
        particle.time_alive += dt;

        if particle.time_alive >= constants::PARTICLE_LIFETIME {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Move by velocity
        transform.translation += particle.velocity * dt;

        // Scale down linearly over lifetime
        let remaining = 1.0 - (particle.time_alive / constants::PARTICLE_LIFETIME);
        let size = constants::PARTICLE_SIZE * remaining;
        transform.scale = Vec3::splat(size);
    }
}

/// System that spawns dark smoke wisps along the beam.
/// Smoke is independent of the beam and self-dissipates after its lifetime.
pub fn spawn_beam_smoke(
    mut commands: Commands,
    beam_query: Query<&DisintegrateBeam>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < constants::SMOKE_SPAWN_INTERVAL {
        return;
    }
    *timer -= constants::SMOKE_SPAWN_INTERVAL;

    let t = time.elapsed_secs();

    for beam in beam_query.iter() {
        let current_len = beam.current_length();
        if current_len < 1.0 {
            continue;
        }

        // Build perpendicular basis
        let up = if beam.direction.y.abs() > 0.9 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let right = beam.direction.cross(up).normalize();
        let forward = right.cross(beam.direction).normalize();

        for i in 0..constants::SMOKE_COUNT_PER_SPAWN {
            // Spawn at a random-ish point along the beam length
            let frac = ((i as f32 + 0.5) / constants::SMOKE_COUNT_PER_SPAWN as f32
                + (t * 13.7).fract())
                % 1.0;

            let pos = beam.origin + beam.direction * (current_len * frac);

            // Annihilation beams span from Y=2000 — only spawn smoke near ground level.
            if beam.annihilation && pos.y > 50.0 {
                continue;
            }

            // Mostly upward drift with slight lateral spread
            let angle = (i as f32 * 2.39 + t * 7.1).sin(); // pseudo-random lateral
            let lateral =
                (right * angle.cos() + forward * angle.sin()) * constants::SMOKE_SPREAD_SPEED;
            let velocity = Vec3::Y * constants::SMOKE_RISE_SPEED + lateral;

            commands.spawn((
                BeamSmoke {
                    velocity,
                    time_alive: 0.0,
                },
                Mesh3d(visual_assets.particle_quad.clone()),
                MeshMaterial3d(visual_assets.disintegrate_smoke.clone()),
                Transform::from_translation(pos)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(constants::SMOKE_SIZE)),
                OnGameplayScreen,
            ));
        }

        // Heat shimmer along the beam
        let shimmer_frac = (t * 5.3 + 0.37).fract();
        let shimmer_pos = beam.origin + beam.direction * (current_len * shimmer_frac);

        // Annihilation beams: skip shimmer high in the sky
        if beam.annihilation && shimmer_pos.y > 50.0 {
            continue;
        }
        vfx::systems::spawn_heat_shimmer(&mut commands, &visual_assets, shimmer_pos, 1, t);
    }
}

/// System that drifts, scale-fades, and despawns smoke wisps.
/// Runs independently of beam existence so smoke lingers after casting stops.
pub fn update_beam_smoke(
    mut commands: Commands,
    mut smoke_query: Query<(Entity, &mut BeamSmoke, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut smoke, mut transform) in smoke_query.iter_mut() {
        smoke.time_alive += dt;

        if smoke.time_alive >= constants::SMOKE_LIFETIME {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Drift by velocity
        transform.translation += smoke.velocity * dt;

        // Grow then shrink: peak at 60% of lifetime, shrink to zero by end
        let progress = smoke.time_alive / constants::SMOKE_LIFETIME;
        let size = if progress < 0.6 {
            constants::SMOKE_SIZE * (1.0 + progress * 0.83)
        } else {
            let shrink = 1.0 - (progress - 0.6) / 0.4;
            constants::SMOKE_SIZE * 1.5 * shrink
        };
        transform.scale = Vec3::splat(size);
    }
}

// ── Talent-specific systems ──────────────────────────────────────────

/// System that auto-sweeps beams with the Sweeping Destruction talent.
/// Oscillates the beam direction around the sweep_center_direction.
pub fn update_sweep_beams(mut beam_query: Query<&mut DisintegrateBeam>, time: Res<Time>) {
    let dt = time.delta_secs();

    for mut beam in beam_query.iter_mut() {
        if !beam.sweeping {
            continue;
        }

        // Advance sweep angle
        beam.sweep_angle += constants::SWEEP_SPEED * beam.sweep_direction * dt;

        // Reverse direction at arc limits
        if beam.sweep_angle.abs() > constants::SWEEP_HALF_ARC {
            beam.sweep_angle = beam
                .sweep_angle
                .clamp(-constants::SWEEP_HALF_ARC, constants::SWEEP_HALF_ARC);
            beam.sweep_direction *= -1.0;
        }

        if beam.annihilation {
            // Sky beam: sweep origin position in XZ instead of rotating direction
            let forward = beam.sweep_center_direction;
            let perp = Vec3::new(-forward.z, 0.0, forward.x);
            let offset = perp * beam.sweep_angle * constants::ANNIHILATION_SWEEP_RADIUS;
            beam.origin = Vec3::new(
                beam.annihilation_cast_pos.x + offset.x,
                constants::ANNIHILATION_SKY_HEIGHT,
                beam.annihilation_cast_pos.z + offset.z,
            );
        } else {
            // Normal beam: apply sweep rotation to center direction
            let total_angle = beam.sweep_angle + beam.fan_offset_angle;
            let rotated = Quat::from_axis_angle(Vec3::Y, total_angle) * beam.sweep_center_direction;
            beam.direction = rotated;
        }
    }
}

/// System that processes searing finale detonations.
/// Applies burst damage once along the detonation line, then fades and despawns.
pub fn update_searing_finale_detonations(
    mut commands: Commands,
    mut detonation_query: Query<(Entity, &mut SearingFinaleDetonation, &mut Transform)>,
    mut target_query: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Wizard>, Without<SearingFinaleDetonation>),
    >,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (det_entity, mut detonation, mut transform) in detonation_query.iter_mut() {
        detonation.time_alive += dt;

        if detonation.time_alive >= constants::SEARING_FINALE_DURATION {
            commands.entity(det_entity).try_despawn();
            continue;
        }

        // Apply damage once
        if !detonation.damage_applied {
            detonation.damage_applied = true;

            for (entity, target_transform, hitbox, mut health, mut temp_hp, has_spell_shield) in
                target_query.iter_mut()
            {
                let pos = target_transform.translation;
                let to_point = pos - detonation.origin;
                let proj = to_point.dot(detonation.direction);

                if proj < -hitbox.radius || proj > detonation.length + hitbox.radius {
                    continue;
                }

                let closest =
                    detonation.origin + detonation.direction * proj.clamp(0.0, detonation.length);
                let dist = pos.distance(closest);

                if dist <= detonation.half_width + hitbox.radius {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        detonation.damage,
                        constants::DAMAGE_TYPE,
                        has_spell_shield,
                    );
                }
            }
        }

        // Visual: expand width over duration
        let progress = detonation.time_alive / constants::SEARING_FINALE_DURATION;
        let visual_width = detonation.half_width * 2.0 * (1.0 + progress * 0.5);
        let alpha = 1.0 - progress;
        transform.scale = Vec3::new(
            visual_width * alpha,
            detonation.length,
            visual_width * alpha,
        );
    }
}

/// Spawns a searing finale detonation entity along a beam's path.
fn spawn_searing_finale(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: &DisintegrateBeam,
) {
    let current_len = beam.current_length();
    if current_len < 1.0 {
        return;
    }

    let midpoint = beam.origin + beam.direction * (current_len / 2.0);
    let half_width = beam.beam_width() * constants::SEARING_FINALE_WIDTH_MULT;
    let damage =
        beam.damage_per_tick() / constants::DAMAGE_INTERVAL * constants::SEARING_FINALE_DAMAGE_MULT;

    let rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

    commands.spawn((
        SearingFinaleDetonation {
            origin: beam.origin,
            direction: beam.direction,
            length: current_len,
            half_width,
            damage,
            time_alive: 0.0,
            damage_applied: false,
        },
        Mesh3d(assets.cross_plane_cylinder.clone()),
        MeshMaterial3d(assets.searing_finale.clone()),
        Transform::from_translation(midpoint)
            .with_rotation(rotation)
            .with_scale(Vec3::new(half_width * 2.0, current_len, half_width * 2.0)),
        OnGameplayScreen,
    ));
}
