//! Hit detection for spells absorbed by crystals.

use super::auto::{spawn_crystal_disintegrate_beam, spawn_crystal_mini_missile};
use super::setup::{
    crystal_beam_geometry, find_random_enemies_in_range, find_random_targets_in_range,
    increment_resonance, scaled_count, spell_echo_multiplier,
};
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::chain_lightning::systems as chain_lightning_systems;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::disintegrate::systems as disintegrate_systems;
use crate::game::units::wizard::spells::finger_of_death::components::FingerOfDeathBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::fireball::systems as fireball_systems;
use crate::game::units::wizard::spells::magic_missile::components::MagicMissile;
use crate::game::units::wizard::spells::meteor_fall::casting::MeteorProjectileTalentFlags;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorProjectile;
use crate::game::units::wizard::spells::meteor_fall::systems as meteor_fall_systems;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::{
    disintegrate_constants, finger_of_death_constants, fireball_constants, magic_missile_constants,
    meteor_fall_constants,
};
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};

pub(super) fn detect_fireball_hits(
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
    explosions: Query<(Entity, &FireballExplosion), Without<CrystalSpawn>>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    for (explosion_entity, explosion) in &explosions {
        for (mut crystal, mut resonance) in &mut crystals {
            if crystal.permanent {
                continue;
            }
            if crystal.explosions_processed.contains(&explosion_entity) {
                continue;
            }

            let distance = xz_distance(crystal.position, explosion.origin);

            if distance <= explosion.max_radius {
                crystal.explosions_processed.push(explosion_entity);
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::Fireball);
                crystal.auto_cast_timer = 0.0;

                // Spell Echo: chance to double the emission
                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);

                // Emit mini fireballs at random targets
                let count = scaled_count(MINI_FB_COUNT, crystal.count_mult) * echo_mult;
                let enemies = find_random_targets_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    count,
                    &targets,
                );

                let mini_radius =
                    fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;
                let damage_scale = DAMAGE_SCALE * crystal.damage_mult;

                for (_, target_pos) in &enemies {
                    let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
                    let direction = (ground_target - crystal.position).normalize();
                    let speed = fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
                    let velocity = direction * speed;

                    let entity = fireball_systems::spawn_fireball_entity(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        velocity,
                        fireball_constants::DAMAGE_PER_TICK * damage_scale,
                        fireball_constants::DAMAGE_TYPE,
                        fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                        fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
                        explosion.empowerment * damage_scale,
                        mini_radius,
                    );
                    commands.entity(entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                        lifetime: None,
                    });
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);
            }
        }
    }
}

// ===== Disintegrate Beam Absorption =====

/// Detects disintegrate and finger of death beams hitting crystals.
///
/// Disintegrate: Maintains persistent beams that update each frame while channeling.
/// Finger of Death: One-shot burst of beams when the damage beam strikes.
/// All crystal beams are now real DisintegrateBeam entities with CrystalSpawn marker.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_beam_hits(
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
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
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
                crystal.fod_beams_processed.push(fod_entity);
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
                let fod_damage_per_tick = finger_of_death_constants::DAMAGE * damage_scale
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

// ===== Meteor Absorption =====

/// Detects meteors hitting crystals and emits mini meteors.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_meteor_hits(
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
    meteors: Query<(Entity, &Transform, &MeteorProjectile)>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;

    for (meteor_entity, meteor_transform, meteor) in &meteors {
        for (mut crystal, mut resonance) in &mut crystals {
            if crystal.permanent {
                continue;
            }
            let distance = xz_distance(crystal.position, meteor_transform.translation);

            // Check if meteor is near the crystal's XZ position and falling through it
            if distance <= crystal.collision_radius
                && meteor_transform.translation.y <= crystal.position.y + CRYSTAL_HEIGHT
                && meteor_transform.translation.y >= 0.0
            {
                // Absorb the meteor
                commands.entity(meteor_entity).try_despawn();
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::Meteor);
                crystal.auto_cast_timer = 0.0;

                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
                let count = scaled_count(2, crystal.count_mult) * echo_mult;
                let damage_scale = DAMAGE_SCALE * crystal.damage_mult;

                // Emit mini meteors at random targets
                let enemies = find_random_targets_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    count,
                    &targets,
                );

                for (_, target_pos) in &enemies {
                    let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);
                    let damage = meteor.damage * damage_scale;
                    let explosion_radius = meteor.explosion_radius * SIZE_SCALE;

                    let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
                        &mut commands,
                        &visual_assets,
                        spawn_pos,
                        Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
                        damage,
                        explosion_radius,
                        meteor.empowerment,
                        mini_radius,
                        MeteorProjectileTalentFlags::default(),
                    );
                    commands.entity(entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                        lifetime: None,
                    });
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);

                break;
            }
        }
    }
}

// ===== Magic Missile Absorption =====

/// Detects magic missiles hitting crystals and emits mini homing missiles.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_magic_missile_hits(
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
    missiles: Query<(Entity, &Transform, &MagicMissile), Without<CrystalSpawn>>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    for (missile_entity, missile_transform, _missile) in &missiles {
        for (mut crystal, mut resonance) in &mut crystals {
            if crystal.permanent {
                continue;
            }
            let distance = missile_transform.translation.distance(crystal.position);

            if distance <= crystal.collision_radius {
                // Absorb the missile
                commands.entity(missile_entity).try_despawn();
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::MagicMissile);
                crystal.auto_cast_timer = 0.0;

                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
                let count = scaled_count(MINI_MISSILE_COUNT, crystal.count_mult) * echo_mult;

                // Emit mini missiles at random enemy targets (not defenders)
                let targets = find_random_enemies_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    count,
                    &enemies,
                );

                let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

                for (target_entity, target_pos) in &targets {
                    let direction = (*target_pos - crystal.position).normalize();
                    let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
                    let initial_velocity = direction * speed;

                    let rng = &mut game_rng.0;
                    let wobble_offset = rng.random_range(0.0..std::f32::consts::TAU);

                    spawn_crystal_mini_missile(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        crystal.range,
                        initial_velocity,
                        wobble_offset,
                        Some(*target_entity),
                        mini_radius,
                        crystal.damage_mult,
                    );
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);

                break;
            }
        }
    }
}

// ===== Chain Lightning Absorption =====

/// Detects chain lightning hitting crystals and emits lightning arcs.
/// This is called when chain lightning bounces to a crystal (crystal is added
/// as a valid bounce target in the chain lightning systems).
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_chain_lightning_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (Entity, &mut ArcaneCrystal, Option<&mut ResonanceCascade>),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    bolts: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningBolt,
    >,
    groups: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningGroup,
    >,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    // Check if any bolt's last_hit_position matches a crystal position
    // (chain lightning system sets crystal as a bounce target, so the bolt
    // will have the crystal's position as last_hit_position after bouncing to it)
    for (crystal_entity, mut crystal, mut resonance) in &mut crystals {
        if crystal.permanent {
            continue;
        }
        for bolt in &bolts {
            // Check if this bolt just bounced to this crystal
            let dist = bolt.last_hit_position.distance(crystal.position);
            if dist > crystal.collision_radius {
                continue;
            }

            // Check if we're in the group's hit list (meaning we were targeted)
            let Ok(group) = groups.get(bolt.group_entity) else {
                continue;
            };
            if !group.hit_entities.contains(&crystal_entity) {
                continue;
            }

            crystal.mark_absorption();
            crystal.remembered_spell = Some(RememberedSpell::ChainLightning);
            crystal.auto_cast_timer = 0.0;

            let rng = &mut game_rng.0;
            let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
            let count = scaled_count(LIGHTNING_ARC_COUNT, crystal.count_mult) * echo_mult;

            // Emit arcs to random targets
            let enemies =
                find_random_targets_in_range(rng, crystal.position, crystal.range, count, &targets);

            let damage = bolt.current_damage * DAMAGE_SCALE * crystal.damage_mult;

            for (target_entity, target_pos) in &enemies {
                // Apply damage
                if let Ok((mut health, mut temp_hp, has_spell_shield)) =
                    health_query.get_mut(*target_entity)
                {
                    apply_spell_damage(
                        &mut commands,
                        *target_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        DamageType::Electric,
                        has_spell_shield,
                    );
                }

                // Spawn arc visual using shared chain lightning helper
                chain_lightning_systems::spawn_arc(
                    &mut commands,
                    &visual_assets,
                    crystal.position,
                    *target_pos,
                    0,
                    crystal.empowerment,
                );
            }

            // Track progress
            progress.increment(Spell::ArcaneCrystal, count as u32);

            // Resonance cascade
            increment_resonance(&mut resonance);

            break; // Only process once per crystal per frame
        }
    }
}
