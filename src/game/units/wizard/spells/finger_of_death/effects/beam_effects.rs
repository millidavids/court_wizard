//! Beam-related effects: deathmark chain, reaper's scythe sweep, beam cleanup.

use super::super::casting::spawn_beam;
use super::super::components::*;
use super::super::constants;
use super::necrotic_explosion::spawn_necrotic_explosion;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{
    Health, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{CastingState, LocalWizard, Spell, Wizard};
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

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
            &Team,
        ),
        Without<Wizard>,
    >,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    rocks_query: Query<&crate::game::terrain::boulder::components::Boulder>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let dt = time.delta_secs();
    let caster_team = local_player_team(session.as_deref());

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
        for (entity, transform, mut health, mut temp_hp, has_spell_shield, team) in
            targets.iter_mut()
        {
            // Enemy shielded King is immune; your own King takes the sweep (friendly fire).
            if (has_spell_shield && caster_team != *team) || sweep.hit_entities.contains(&entity) {
                continue;
            }
            if beam.contains_point(transform.translation, beam_width) {
                let proj = (transform.translation - sweep.origin).dot(rotated_dir);
                if proj <= effective_length {
                    sweep.hit_entities.insert(entity);
                    let was_alive = health.current > 0.0;
                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        DamageType::Necrotic,
                        has_spell_shield,
                        caster_team,
                        *team,
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
