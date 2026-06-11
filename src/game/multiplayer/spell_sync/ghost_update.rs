use bevy::prelude::*;

use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::{
    AuraBubbleVariant, CastEventKind, MoteMaterial, PoofVariant, SparkMaterial, SpellSchoolWire,
    SpellSoundId,
};

use super::ghost_spawn::aura_material_handle;
use super::snapshot_send::LatestSpellSnapshot;

/// Dispatches the remote peer's one-shot cast VFX events into local spawn
/// calls. Runs after `apply_remote_spell_snapshot` and reads the same
/// `LatestSpellSnapshot` resource.
///
/// Each event maps to one of the existing `vfx::systems::spawn_*` helpers;
/// the spawned entities tag `OnGameplayScreen` so MP cleanup
/// (`cleanup_game` on `OnExit(AppState::MultiplayerGame)`) catches them.
#[allow(clippy::too_many_arguments)]
pub fn apply_remote_cast_events(
    mut commands: Commands,
    latest: Res<LatestSpellSnapshot>,
    assets: Option<Res<SpellVisualAssets>>,
    mut sphere_materials: ResMut<
        Assets<crate::game::units::wizard::spells::visual_assets::FireExplosionSphereMaterial>,
    >,
    time: Res<Time>,
    game_config: Res<crate::config::GameConfig>,
    sfx_assets: Option<Res<crate::game::units::wizard::spells::audio::SpellSfxAssets>>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(snapshot) = &latest.0 else { return };
    let Some(assets) = assets else { return };
    if snapshot.cast_events.is_empty() {
        return;
    }

    // The opponent's spells should sound like their archetype on this screen too.
    let remote_is_excremage = session
        .map(|s| s.remote_wizard() == crate::config::WizardType::Excremage)
        .unwrap_or(false);

    use crate::game::units::wizard::archetypes::gunslinger::replication as gunslinger_replication;
    use crate::game::units::wizard::spells::banishment::components::BanishmentVfx;
    use crate::game::units::wizard::spells::vfx::systems as vfx;

    let now = time.elapsed_secs();

    for event in &snapshot.cast_events {
        let pos = Vec3::new(event.x, event.y, event.z);
        let Ok(kind) = CastEventKind::try_from(event.kind) else {
            continue;
        };
        match kind {
            CastEventKind::SchoolFlare => {
                let Ok(school_wire) = SpellSchoolWire::try_from(event.subkind) else {
                    continue;
                };
                let school = match school_wire {
                    SpellSchoolWire::Fire => vfx::SpellSchool::Fire,
                    SpellSchoolWire::Lightning => vfx::SpellSchool::Lightning,
                    SpellSchoolWire::Arcane => vfx::SpellSchool::Arcane,
                    SpellSchoolWire::Nature => vfx::SpellSchool::Nature,
                    SpellSchoolWire::Holy => vfx::SpellSchool::Holy,
                    SpellSchoolWire::Dark => vfx::SpellSchool::Dark,
                    SpellSchoolWire::Force => vfx::SpellSchool::Force,
                    SpellSchoolWire::Transmutation => vfx::SpellSchool::Transmutation,
                };
                vfx::spawn_school_flare(&mut commands, &assets, pos, school, now);
            }
            CastEventKind::AuraBubble | CastEventKind::AuraBubbleContract => {
                let Ok(variant) = AuraBubbleVariant::try_from(event.subkind) else {
                    continue;
                };
                let material = aura_material_handle(&assets, variant);
                let radius = event.extra[0].max(0.01);
                let duration = event.extra[1].max(0.01);
                if kind == CastEventKind::AuraBubbleContract {
                    vfx::spawn_aura_bubble_contracting(
                        &mut commands,
                        &assets,
                        material,
                        pos,
                        radius,
                        duration,
                    );
                } else {
                    vfx::spawn_aura_bubble(&mut commands, &assets, material, pos, radius, duration);
                }
            }
            CastEventKind::SmokePoof => {
                let Ok(variant) = PoofVariant::try_from(event.subkind) else {
                    continue;
                };
                let material = match variant {
                    PoofVariant::Banishment => assets.banishment_poof.clone(),
                    PoofVariant::Polymorph => assets.polymorph_poof.clone(),
                };
                // `extra[0]` carries the caller-supplied count so host /
                // guest spawn the same number of puffs. Clamp defensively
                // against corrupt packets.
                let count = (event.extra[0] as usize).clamp(1, 64);
                vfx::spawn_smoke_poof(&mut commands, &assets, &material, pos, count, now);
            }
            CastEventKind::FloatingMotes => {
                let Ok(material_kind) = MoteMaterial::try_from(event.subkind) else {
                    continue;
                };
                let material = match material_kind {
                    MoteMaterial::Healing => assets.healing_mote.clone(),
                    MoteMaterial::Nature => assets.nature_mote.clone(),
                    MoteMaterial::Sleep => assets.sleep_mote.clone(),
                };
                let radius = event.extra[0].max(0.01);
                let count = (event.extra[1] as usize).clamp(1, 200);
                vfx::spawn_floating_motes(
                    &mut commands,
                    &assets,
                    &material,
                    pos,
                    radius,
                    count,
                    now,
                );
            }
            CastEventKind::Sparks => {
                let Ok(material_kind) = SparkMaterial::try_from(event.subkind) else {
                    continue;
                };
                let material = match material_kind {
                    SparkMaterial::Banishment => assets.banishment_spark.clone(),
                    SparkMaterial::Dispel => assets.dispel_spark.clone(),
                };
                // `extra[0]` carries the caller-supplied count.
                let count = (event.extra[0] as usize).clamp(1, 64);
                vfx::spawn_sparks_with_material(&mut commands, &assets, pos, count, now, material);
            }
            CastEventKind::DustSmoke => {
                // half_width / count default to the SP wall-of-stone collapse
                // (8.0 / 14). Distinguish "not provided" (sentinel < 0) from
                // a legitimately small/zero value the caller passed.
                let half_width = if event.extra[0] >= 0.0 {
                    event.extra[0]
                } else {
                    8.0
                };
                // Clamp count against malformed packets — FloatingMotes
                // uses the same defensive bound.
                let count = if event.extra[1] > 0.0 {
                    (event.extra[1] as usize).clamp(1, 200)
                } else {
                    14
                };
                vfx::spawn_dust_smoke(&mut commands, &assets, pos, half_width, count, now);
            }
            CastEventKind::FinalStandExplosion => {
                // Berserker Rage's Final Stand detonation — spawn the same
                // FinalStandExplosionVfx component the host spawns so the
                // guest's `update_final_stand_vfx` (gated
                // `is_spell_effects_active`) animates it identically. The
                // material is cloned per-instance so the time uniform
                // animates independently from other concurrent explosions.
                use crate::game::units::wizard::spells::berserker_rage::components::FinalStandExplosionVfx;
                use crate::game::units::wizard::spells::spell_materials::clone_sphere_material;
                let max_radius = event.extra[0].max(0.01);
                let lifetime = event.extra[1].max(0.05);
                let mat_handle =
                    clone_sphere_material(&mut sphere_materials, &assets.fireball_explosion_sphere);
                commands.spawn((
                    FinalStandExplosionVfx {
                        time_alive: 0.0,
                        max_radius,
                        lifetime,
                    },
                    Mesh3d(assets.explosion_sphere.clone()),
                    MeshMaterial3d(mat_handle),
                    Transform::from_translation(pos).with_scale(Vec3::splat(0.1)),
                    crate::game::components::OnGameplayScreen,
                ));
            }
            CastEventKind::BanishmentLens => {
                // Spawn the BanishmentVfx component so `update_banishment_vfx`
                // (gated `is_spell_effects_active` after Phase 1) animates
                // the lensing-sphere collapse on the guest. Uses the shared
                // `cross_plane_sphere` asset to match the SP path's mesh
                // shape exactly — `banishment_lens` material is designed
                // for the cross-plane fans, not a tessellated sphere.
                let radius = event.extra[0].max(0.01);
                let duration = event.extra[1].max(0.05);
                commands.spawn((
                    BanishmentVfx {
                        time_alive: 0.0,
                        lifetime: duration,
                        start_radius: radius,
                    },
                    Mesh3d(assets.cross_plane_sphere.clone()),
                    MeshMaterial3d(assets.banishment_lens.clone()),
                    Transform::from_translation(pos).with_scale(Vec3::splat(radius)),
                    crate::game::components::OnGameplayScreen,
                ));
            }
            CastEventKind::SfxOneShot => {
                let Some(ref sfx) = sfx_assets else { continue };
                let Ok(sound_id) = SpellSoundId::try_from(event.subkind) else {
                    continue;
                };
                let volume_scale = if event.extra[0] > 0.0 {
                    event.extra[0]
                } else {
                    1.0
                };
                crate::game::units::wizard::spells::audio::play_remote_sfx(
                    &mut commands,
                    sound_id,
                    pos,
                    volume_scale,
                    &game_config,
                    sfx.as_ref(),
                    remote_is_excremage,
                );
            }
            // Warglock gun visuals (opponent renders the local Warglock's shots).
            CastEventKind::GunMuzzleFlash => {
                gunslinger_replication::spawn_ghost_muzzle_flash(&mut commands, &assets, pos);
            }
            CastEventKind::GunBulletTracer => {
                let velocity = Vec3::new(event.extra[0], event.extra[1], event.extra[2]);
                gunslinger_replication::spawn_ghost_tracer(
                    &mut commands,
                    &assets,
                    pos,
                    velocity,
                    event.extra[3],
                );
            }
            CastEventKind::GunFlameParticle => {
                let velocity = Vec3::new(event.extra[0], event.extra[1], event.extra[2]);
                gunslinger_replication::spawn_ghost_flame(
                    &mut commands,
                    pos,
                    velocity,
                    event.extra[3],
                );
            }
            // Swordcerer sword arc — render the opponent's swing (visual only).
            CastEventKind::SwordArc => {
                let dir = Vec2::new(event.extra[0], event.extra[1]);
                crate::game::units::wizard::archetypes::swordcerer::spawn_sword_arc(
                    &mut commands,
                    &mut meshes,
                    &mut standard_materials,
                    pos,
                    dir,
                    true,
                );
            }
        }
    }
}

/// Regenerates disintegrate beam-tip impact particles + smoke for the opposing
/// client's ghost beam, locally from the latest beam snapshot.
///
/// Runs on BOTH peers (spell sync is symmetric — each renders the other's
/// spells). The ghost beam spawned in `apply_remote_spell_snapshot` carries no
/// `DisintegrateBeam` component, so the single-player spawners
/// (`spawn_impact_particles` / `spawn_beam_smoke`, which query `DisintegrateBeam`)
/// never run for it and the beam-tip VFX was missing on the opposing client. We
/// reproduce it from the snapshot geometry; the spawned `DisintegrateParticle` /
/// `BeamSmoke` entities are then animated and despawned by the disintegrate
/// plugin's existing `update_impact_particles` / `update_beam_smoke` (which
/// already run on both peers via `is_spell_effects_active`). Throttled with
/// `Local` timers to match the SP cadence.
///
/// `annihilation = true` is passed unconditionally so the helpers' `tip.y > 50`
/// guard suppresses VFX during an annihilation beam's brief sky-descent growth.
/// That guard is a no-op for normal/crystal beams (their tip sits at ground
/// level), and the annihilation flag is not carried in `BeamSnapshot`.
pub fn spawn_ghost_beam_impact_vfx(
    mut commands: Commands,
    latest: Res<LatestSpellSnapshot>,
    assets: Option<Res<SpellVisualAssets>>,
    time: Res<Time>,
    mut particle_timer: Local<f32>,
    mut smoke_timer: Local<f32>,
) {
    use crate::game::units::wizard::spells::disintegrate::beam::{
        emit_beam_smoke, emit_impact_particles,
    };
    use crate::game::units::wizard::spells::disintegrate::constants as disint;

    let Some(snapshot) = &latest.0 else {
        return;
    };
    let Some(assets) = assets else {
        return;
    };

    // Bail before ticking the timers when the remote peer has no active beam,
    // so they freeze between beams — matching the SP spawners, which only run
    // (and tick) while a `DisintegrateBeam` exists.
    if snapshot.beams.is_empty() {
        return;
    }

    // Throttle to the SP cadence with accumulator timers (wrap on fire).
    *particle_timer += time.delta_secs();
    *smoke_timer += time.delta_secs();
    let emit_particles = *particle_timer >= disint::PARTICLE_SPAWN_INTERVAL;
    let emit_smoke = *smoke_timer >= disint::SMOKE_SPAWN_INTERVAL;
    if emit_particles {
        *particle_timer -= disint::PARTICLE_SPAWN_INTERVAL;
    }
    if emit_smoke {
        *smoke_timer -= disint::SMOKE_SPAWN_INTERVAL;
    }
    if !emit_particles && !emit_smoke {
        return;
    }

    let elapsed = time.elapsed_secs();
    for beam in &snapshot.beams {
        let origin = Vec3::new(beam.ox, beam.oy, beam.oz);
        let direction = Vec3::new(beam.dx, beam.dy, beam.dz);
        let length = beam.length;

        // `true` forces the helpers' `tip.y > 50` sky-growth guard (see fn doc).
        if emit_particles {
            emit_impact_particles(
                &mut commands,
                &assets,
                elapsed,
                origin,
                direction,
                length,
                true,
            );
        }
        if emit_smoke {
            emit_beam_smoke(
                &mut commands,
                &assets,
                elapsed,
                origin,
                direction,
                length,
                true,
            );
        }
    }
}
