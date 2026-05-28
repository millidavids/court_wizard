//! Disintegrate casting and damage application.

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::beam::{despawn_all_beam_visuals, spawn_beam_with_talents, spawn_searing_finale};
use super::components::{
    BeamEclipse, BeamGlow, BeamOriginFlare, DisintegrateBeam, DisintegrateParticle,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::units::components::{Health, Hitbox, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::audio::{self, ChannelingSfx, SpellSfxAssets};
use crate::game::units::wizard::spells::fireball;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
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
    cursor_resources: (
        Res<CorrectedCursorPosition>,
        Res<LocalSpellOrigin>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
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
    let (corrected_cursor, local_origin, mut pending_cast_events) = cursor_resources;
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
        local_origin.0,
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
            vfx::systems::spawn_school_flare_synced(
                &mut commands,
                &visual_assets,
                &mut pending_cast_events,
                local_origin.0,
                vfx::systems::SpellSchool::Arcane,
                time.elapsed_secs(),
            );
            // Annihilation Beam: shoot from the sky above the clamped target
            let mut annihilation_forward = Vec3::X;
            if talent_cfg.annihilation {
                // Use the already range-clamped target from casting logic
                let ground_target = origin + direction * length;
                let wizard_xz = Vec3::new(local_origin.0.x, 0.0, local_origin.0.z);
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
    local_origin: Vec3,
) -> CastingResult {
    let mut result = CastingResult {
        beam_action: BeamAction::None,
    };

    let wizard_pos = local_origin;

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
#[allow(clippy::too_many_arguments)]
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
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
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

            if !(beam.annihilation && beam.origin.y > 50.0) {
                let tip = beam.origin + beam.direction * beam.current_length();
                terrain_damage.write(TerrainDamageMessage {
                    position: tip,
                    radius: 0.0,
                    damage,
                    damage_type: constants::DAMAGE_TYPE,
                });
            }

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
