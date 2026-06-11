use super::beam_actions::{BeamAction, disintegrate_casting_logic};
use super::talent_config::compute_talent_config;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::audio::{self, ChannelingSfx, SpellSfxAssets};
use crate::game::units::wizard::spells::disintegrate::beam::{
    despawn_all_beam_visuals, spawn_beam_with_talents, spawn_searing_finale,
};
use crate::game::units::wizard::spells::disintegrate::components::{
    BeamEclipse, BeamGlow, BeamOriginFlare, DisintegrateBeam, DisintegrateParticle,
};
use crate::game::units::wizard::spells::disintegrate::constants;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

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
            // The channel loop is local-only; emit a synced one-shot so the
            // opponent hears the beam fire (it was silent on the other client).
            // Project to ground level — with the Annihilation talent `origin`
            // sits at sky height (y≈2000), which would attenuate the remote
            // sound to a near-silent whisper against the listener's wizard.
            audio::emit_sfx_event(
                &mut pending_cast_events,
                crate::networking::snapshot::SpellSoundId::DisintegrateChannel,
                Vec3::new(origin.x, 0.0, origin.z),
            );
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
