//! Hit detection for disintegrate and finger-of-death beams absorbed by crystals.

use super::super::auto::spawn_crystal_disintegrate_beam;
use super::super::setup::{
    crystal_beam_geometry, find_random_targets_in_range, increment_resonance, scaled_count,
    spell_echo_multiplier,
};
use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::components::{Corpse, Health};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::disintegrate::systems as disintegrate_systems;
use crate::game::units::wizard::spells::disintegrate_constants;
use crate::game::units::wizard::spells::finger_of_death::components::FingerOfDeathBeam;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};

/// Detects disintegrate and finger of death beams hitting crystals.
///
/// Disintegrate: Maintains persistent beams that update each frame while channeling.
/// Finger of Death: One-shot burst of beams when the damage beam strikes.
/// All crystal beams are now real DisintegrateBeam entities with CrystalSpawn marker.
#[allow(clippy::too_many_arguments)]
pub(crate) fn detect_beam_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    // Each peer drives its own real crystal only — the ghost copy of the
    // remote peer's crystal is excluded so the same absorption never fires
    // twice across the network.
    mut crystals: Query<
        (&mut ArcaneCrystal, Option<&mut ResonanceCascade>),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    disintegrate_beams: Query<&DisintegrateBeam, Without<CrystalSpawn>>,
    fod_beams: Query<(Entity, &FingerOfDeathBeam)>,
    mut crystal_beams: Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    active_talents: Option<Res<ActiveTalents>>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    let talent_cfg = disintegrate_systems::compute_talent_config(active_talents.as_deref());
    for (mut crystal, mut resonance) in &mut crystals {
        if crystal.permanent {
            continue;
        }
        // === Disintegrate: persistent beams while channeling ===
        let mut hit_by_disintegrate = false;
        for beam in &disintegrate_beams {
            if beam.contains_point(crystal.position) {
                hit_by_disintegrate = true;
                break;
            }
        }

        if hit_by_disintegrate {
            crystal.hit_by_disintegrate = true;
            crystal.mark_absorption();
            crystal.remembered_spell = Some(RememberedSpell::Disintegrate);
            crystal.auto_cast_timer = 0.0;
            // Intentionally does NOT call increment_resonance() or progress.increment() here:
            // disintegrate hits run every frame while the beam is channeled, so tracking
            // them per-frame would over-count. Progress is tracked via the beam's own systems.

            // Clean up beam groups whose entities were despawned externally
            crystal.active_beams.retain(|(beam_entities, _)| {
                beam_entities.iter().any(|e| crystal_beams.get(*e).is_ok())
            });

            // Check each existing beam group's target — replace if dead or out of range
            let mut used_targets: Vec<Entity> = Vec::new();
            let mut groups_needing_new_target: Vec<usize> = Vec::new();

            for (i, (beam_entities, target_entity)) in crystal.active_beams.iter().enumerate() {
                if let Ok((_, target_transform)) = targets.get(*target_entity) {
                    let dist = xz_distance(crystal.position, target_transform.translation);
                    if dist <= crystal.range {
                        // Target still valid — update all beams in group to track it
                        let (base_direction, length) = crystal_beam_geometry(
                            crystal.position,
                            target_transform.translation,
                            crystal.range,
                        );
                        for beam_entity in beam_entities {
                            if let Ok(mut beam) =
                                crystal_beams.get_mut(*beam_entity).map(|(_, beam)| beam)
                            {
                                beam.origin = crystal.position;
                                beam.direction =
                                    Quat::from_axis_angle(Vec3::Y, beam.fan_offset_angle)
                                        * base_direction;
                                beam.length = length;
                            }
                        }
                        used_targets.push(*target_entity);
                        continue;
                    }
                }
                // Target dead or out of range
                groups_needing_new_target.push(i);
            }

            // Find replacement targets for beam groups that lost theirs
            if !groups_needing_new_target.is_empty() {
                let mut candidates: Vec<(Entity, Vec3)> = targets
                    .iter()
                    .filter(|(e, _)| !used_targets.contains(e))
                    .filter(|(_, transform)| {
                        xz_distance(crystal.position, transform.translation) <= crystal.range
                    })
                    .map(|(entity, transform)| (entity, transform.translation))
                    .collect();

                let rng = &mut game_rng.0;
                let len = candidates.len();
                for i in (1..len).rev() {
                    let j = rng.random_range(0..=i);
                    candidates.swap(i, j);
                }

                for (idx, group_idx) in groups_needing_new_target.iter().enumerate() {
                    if let Some((new_target, new_pos)) = candidates.get(idx) {
                        // Despawn old group, spawn new group targeting new enemy
                        let (old_beams, _) = &crystal.active_beams[*group_idx];
                        for beam_entity in old_beams {
                            commands.entity(*beam_entity).try_despawn();
                        }
                        let new_beams = spawn_crystal_disintegrate_beam(
                            &mut commands,
                            &visual_assets,
                            crystal.position,
                            *new_pos,
                            crystal.range,
                            crystal.empowerment,
                            Some(&talent_cfg),
                        );
                        crystal.active_beams[*group_idx] = (new_beams, *new_target);
                        used_targets.push(*new_target);
                    } else {
                        // No replacement available — despawn the group
                        let (old_beams, _) = &crystal.active_beams[*group_idx];
                        for beam_entity in old_beams {
                            commands.entity(*beam_entity).try_despawn();
                        }
                    }
                }

                // Remove groups that had no replacement (iterate in reverse to keep indices valid)
                let candidate_count = candidates.len();
                for (idx, group_idx) in groups_needing_new_target.iter().enumerate().rev() {
                    if idx >= candidate_count {
                        crystal.active_beams.remove(*group_idx);
                    }
                }
            }

            // Spawn new beam groups if we have fewer than the scaled beam count
            let beam_target_count = scaled_count(BEAM_COUNT, crystal.count_mult);
            if crystal.active_beams.len() < beam_target_count {
                let needed = beam_target_count - crystal.active_beams.len();
                let mut candidates: Vec<(Entity, Vec3)> = targets
                    .iter()
                    .filter(|(e, _)| !used_targets.contains(e))
                    .filter(|(_, transform)| {
                        xz_distance(crystal.position, transform.translation) <= crystal.range
                    })
                    .map(|(entity, transform)| (entity, transform.translation))
                    .collect();

                let rng = &mut game_rng.0;
                let len = candidates.len();
                for i in (1..len).rev() {
                    let j = rng.random_range(0..=i);
                    candidates.swap(i, j);
                }
                candidates.truncate(needed);

                for (target_entity, target_pos) in &candidates {
                    let beam_entities = spawn_crystal_disintegrate_beam(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        *target_pos,
                        crystal.range,
                        crystal.empowerment,
                        Some(&talent_cfg),
                    );
                    crystal.active_beams.push((beam_entities, *target_entity));
                }
            }
        } else if crystal.hit_by_disintegrate {
            // Disintegrate just stopped — despawn persistent beams
            crystal.hit_by_disintegrate = false;
            for (beam_entities, _) in crystal.active_beams.drain(..) {
                for beam_entity in beam_entities {
                    commands.entity(beam_entity).try_despawn();
                }
            }
        }

        // === Finger of Death: one-shot burst of beams ===
        for (fod_entity, fod_beam) in &fod_beams {
            if !fod_beam.has_fired || crystal.fod_beams_processed.contains(&fod_entity) {
                continue;
            }

            if fod_beam.contains_point(crystal.position, fod_beam.beam_width_fired()) {
                crystal.fod_beams_processed.insert(fod_entity);
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::FingerOfDeath);
                crystal.auto_cast_timer = 0.0;

                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
                let fod_beam_count = scaled_count(BEAM_COUNT, crystal.count_mult) * echo_mult;
                let enemies = find_random_targets_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    fod_beam_count,
                    &targets,
                );
                let damage_scale = BEAM_DAMAGE_SCALE * crystal.damage_mult;
                let fod_damage_per_tick = FOD_ECHO_BASE_DAMAGE * damage_scale
                    / (BEAM_DURATION / disintegrate_constants::DAMAGE_INTERVAL);
                let forked = talent_cfg.forked;
                let offsets: &[f32] = if forked {
                    &[-FORKED_FAN_HALF_ANGLE, 0.0, FORKED_FAN_HALF_ANGLE]
                } else {
                    &[0.0]
                };
                for (_, target_pos) in &enemies {
                    let (base_direction, length) =
                        crystal_beam_geometry(crystal.position, *target_pos, crystal.range);
                    for &offset in offsets {
                        let direction = if offset.abs() > 0.001 {
                            Quat::from_axis_angle(Vec3::Y, offset) * base_direction
                        } else {
                            base_direction
                        };
                        let beam_entity = disintegrate_systems::spawn_beam_with_damage(
                            &mut commands,
                            &visual_assets,
                            crystal.position,
                            direction,
                            length,
                            crystal.empowerment,
                            fod_damage_per_tick,
                            Some(&talent_cfg),
                            damage_scale,
                            offset,
                        );
                        commands.entity(beam_entity).insert(CrystalSpawn {
                            origin: crystal.position,
                            max_range: crystal.range,
                            lifetime: Some(BEAM_DURATION),
                        });
                    }
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, fod_beam_count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);
            }
        }
    }
}
