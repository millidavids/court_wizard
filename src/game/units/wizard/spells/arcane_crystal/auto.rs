//! Auto-cast and talent burst handlers.

use super::setup::{
    crystal_beam_geometry, crystal_target_teams, find_random_enemies_in_range,
    find_random_targets_in_range, increment_resonance, scaled_count,
};
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::chain_lightning::constants as cl_constants;
use crate::game::units::wizard::spells::chain_lightning::systems as chain_lightning_systems;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::disintegrate::systems as disintegrate_systems;
use crate::game::units::wizard::spells::fireball::systems as fireball_systems;
use crate::game::units::wizard::spells::magic_missile::components::TargetTeams;
use crate::game::units::wizard::spells::meteor_fall::casting::MeteorProjectileTalentFlags;
use crate::game::units::wizard::spells::meteor_fall::systems as meteor_fall_systems;
use crate::game::units::wizard::spells::utils::{local_player_team, xz_distance};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::{
    disintegrate_constants, finger_of_death_constants, fireball_constants, magic_missile_constants,
    meteor_fall_constants,
};
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::session::MultiplayerSession;

#[allow(clippy::too_many_arguments)]
pub(super) fn auto_cast_remembered_spell(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    // Each peer drives its own real crystal only — see hits.rs for rationale.
    mut crystals: Query<
        (Entity, &mut ArcaneCrystal),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut crystal_beams: Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    active_talents: Option<Res<ActiveTalents>>,
    peer_id: Option<Res<crate::networking::crdt::PeerId>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let target_teams = crystal_target_teams(peer_id.as_deref());
    let talent_cfg = disintegrate_systems::compute_talent_config(active_talents.as_deref());
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();

    // Collect crystal data to avoid borrow conflicts (skip permanent turrets).
    // Store Entity IDs for O(1) lookup via get_mut() instead of O(n) iter_mut().nth().
    let crystal_data: Vec<_> = crystals
        .iter()
        .filter(|(_, c)| !c.permanent)
        .map(|(e, c)| {
            (
                e,
                c.position,
                c.range,
                c.empowerment,
                c.remembered_spell,
                c.auto_cast_timer,
                c.auto_disintegrate_beam.clone(),
                c.damage_mult,
                c.count_mult,
            )
        })
        .collect();

    for (
        entity,
        position,
        range,
        empowerment,
        remembered,
        timer,
        auto_beam,
        damage_mult,
        count_mult,
    ) in crystal_data.into_iter()
    {
        let Some(remembered) = remembered else {
            // No remembered spell — clean up any lingering auto-disintegrate beam
            if let Some((beam_entities, _)) = &auto_beam {
                for beam_entity in beam_entities {
                    commands.entity(*beam_entity).try_despawn();
                }
                if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                    crystal.auto_disintegrate_beam = None;
                }
            }
            continue;
        };

        // === Special case: Disintegrate = constant single beam ===
        if remembered == RememberedSpell::Disintegrate {
            handle_auto_disintegrate(
                &mut game_rng.0,
                entity,
                position,
                range,
                empowerment,
                auto_beam,
                &mut commands,
                &visual_assets,
                &mut crystal_beams,
                &targets,
                &mut crystals,
                &talent_cfg,
            );
            continue;
        }

        // === Timer-based auto-cast for all other spells ===

        // Clean up any lingering auto-disintegrate beam if spell changed
        if let Some((beam_entities, _)) = &auto_beam {
            for beam_entity in beam_entities {
                commands.entity(*beam_entity).try_despawn();
            }
            if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                crystal.auto_disintegrate_beam = None;
            }
        }

        let new_timer = timer + delta;
        let interval = remembered.auto_cast_interval();

        if new_timer >= interval {
            // Reset timer
            if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                crystal.auto_cast_timer = 0.0;
                crystal.trigger_pulse();
            }

            let autocast = CrystalAutocastParams {
                position,
                range,
                empowerment,
                damage_mult,
                count_mult,
            };

            match remembered {
                RememberedSpell::MagicMissile => {
                    auto_cast_magic_missiles(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &enemies,
                        target_teams,
                    );
                }
                RememberedSpell::Fireball => {
                    auto_cast_fireballs(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                    );
                }
                RememberedSpell::ChainLightning => {
                    auto_cast_chain_lightning(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                        &mut health_query,
                        caster_team,
                    );
                }
                RememberedSpell::Meteor => {
                    auto_cast_meteors(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                    );
                }
                RememberedSpell::FingerOfDeath => {
                    auto_cast_fod_beams(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                        &talent_cfg,
                    );
                }
                RememberedSpell::Disintegrate => unreachable!(),
            }
        } else {
            // Just advance timer
            if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                crystal.auto_cast_timer = new_timer;
            }
        }
    }
}

/// Manages a persistent auto-disintegrate beam group.
///
/// Instead of despawning/respawning beams every frame (which resets time_alive
/// and breaks the growth animation + damage), we update beam fields in-place.
/// New beams are only spawned when the old target dies/leaves range.
#[allow(clippy::too_many_arguments)]
fn handle_auto_disintegrate(
    rng: &mut impl Rng,
    crystal_entity: Entity,
    position: Vec3,
    range: f32,
    empowerment: f32,
    auto_beam: Option<(Vec<Entity>, Entity)>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    crystal_beams: &mut Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    crystals: &mut Query<
        (Entity, &mut ArcaneCrystal),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    talent_cfg: &disintegrate_systems::TalentConfig,
) {
    if let Some((beam_entities, target_entity)) = auto_beam {
        // Check if any beam entity still exists
        let any_alive = beam_entities.iter().any(|e| crystal_beams.get(*e).is_ok());
        if !any_alive {
            if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
                crystal.auto_disintegrate_beam = None;
            }
            // Fall through to spawn a new beam below
        } else if let Ok((_, target_transform)) = targets.get(target_entity) {
            // Target alive — check range
            let dist = xz_distance(position, target_transform.translation);
            if dist <= range {
                // Target still valid — update all beams to track it in-place
                let (base_direction, length) =
                    crystal_beam_geometry(position, target_transform.translation, range);
                for beam_entity in &beam_entities {
                    if let Ok((_, mut beam)) = crystal_beams.get_mut(*beam_entity) {
                        beam.origin = position;
                        beam.direction =
                            Quat::from_axis_angle(Vec3::Y, beam.fan_offset_angle) * base_direction;
                        beam.length = length;
                    }
                }
                return;
            }
            // Target out of range — fall through to replace beam
        } else {
            // Target dead — fall through to replace beam
        }
        // Despawn old beams and find new target
        despawn_beam_group(commands, &beam_entities);
        let new_targets = find_random_targets_in_range(rng, position, range, 1, targets);
        if let Some((new_target, new_pos)) = new_targets.first() {
            let new_beams = spawn_crystal_disintegrate_beam(
                commands,
                assets,
                position,
                *new_pos,
                range,
                empowerment,
                Some(talent_cfg),
            );
            if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
                crystal.auto_disintegrate_beam = Some((new_beams, *new_target));
            }
        } else if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
            crystal.auto_disintegrate_beam = None;
        }
        return;
    }

    // No beam exists — try to spawn one
    let new_targets = find_random_targets_in_range(rng, position, range, 1, targets);
    if let Some((target_entity, target_pos)) = new_targets.first() {
        let beam_entities = spawn_crystal_disintegrate_beam(
            commands,
            assets,
            position,
            *target_pos,
            range,
            empowerment,
            Some(talent_cfg),
        );
        if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
            crystal.auto_disintegrate_beam = Some((beam_entities, *target_entity));
        }
    }
}

/// Helper to despawn all beams in a group.
fn despawn_beam_group(commands: &mut Commands, beam_entities: &[Entity]) {
    for beam_entity in beam_entities {
        commands.entity(*beam_entity).try_despawn();
    }
}

/// Spawns crystal disintegrate beam(s) with damage scaling and CrystalSpawn marker.
/// When forked talent is active, spawns 3 beams in a fan pattern.
/// Returns all spawned beam entities.
pub(super) fn spawn_crystal_disintegrate_beam(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    target: Vec3,
    max_range: f32,
    empowerment: f32,
    talent_cfg: Option<&disintegrate_systems::TalentConfig>,
) -> Vec<Entity> {
    let (base_direction, length) = crystal_beam_geometry(origin, target, max_range);
    let forked = talent_cfg.is_some_and(|cfg| cfg.forked);

    let offsets: &[f32] = if forked {
        &[-FORKED_FAN_HALF_ANGLE, 0.0, FORKED_FAN_HALF_ANGLE]
    } else {
        &[0.0]
    };

    let mut entities = Vec::with_capacity(offsets.len());
    for &offset in offsets {
        let direction = if offset.abs() > 0.001 {
            Quat::from_axis_angle(Vec3::Y, offset) * base_direction
        } else {
            base_direction
        };
        let beam_entity = disintegrate_systems::spawn_beam_with_damage(
            commands,
            assets,
            origin,
            direction,
            length,
            empowerment,
            disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
            talent_cfg,
            BEAM_DAMAGE_SCALE,
            offset,
        );
        commands.entity(beam_entity).insert(CrystalSpawn {
            origin,
            max_range,
            lifetime: None,
        });
        entities.push(beam_entity);
    }
    entities
}

/// Shared parameters for crystal auto-cast helper functions.
struct CrystalAutocastParams {
    position: Vec3,
    range: f32,
    empowerment: f32,
    damage_mult: f32,
    count_mult: f32,
}

/// Auto-casts mini magic missiles at random enemies (the caster's hostile
/// team set, passed in by the system that knows the local `PeerId`).
fn auto_cast_magic_missiles(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    enemies: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
    target_teams: TargetTeams,
) {
    let targets = find_random_enemies_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(MINI_MISSILE_COUNT, params.count_mult),
        enemies,
        target_teams,
    );
    let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

    for (target_entity, target_pos) in &targets {
        let direction = (*target_pos - params.position).normalize();
        let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
        let initial_velocity = direction * speed;

        let wobble_offset = rng.random_range(0.0..std::f32::consts::TAU);

        spawn_crystal_mini_missile(
            commands,
            assets,
            params.position,
            params.range,
            initial_velocity,
            wobble_offset,
            Some(*target_entity),
            mini_radius,
            params.damage_mult,
            target_teams,
        );
    }
}

/// Auto-casts mini fireballs at random enemies.
fn auto_cast_fireballs(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(MINI_FB_COUNT, params.count_mult),
        targets,
    );
    let mini_radius = fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;

    for (_, target_pos) in &enemies {
        let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
        let direction = (ground_target - params.position).normalize();
        let speed = fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
        let velocity = direction * speed;

        let entity = fireball_systems::spawn_fireball_entity(
            commands,
            assets,
            params.position,
            velocity,
            fireball_constants::DAMAGE_PER_TICK * DAMAGE_SCALE * params.damage_mult,
            fireball_constants::DAMAGE_TYPE,
            fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
            fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
            params.empowerment * DAMAGE_SCALE * params.damage_mult,
            mini_radius,
        );
        commands.entity(entity).insert(CrystalSpawn {
            origin: params.position,
            max_range: params.range,
            lifetime: None,
        });
    }
}

/// Auto-casts chain lightning arcs at random enemies.
fn auto_cast_chain_lightning(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    health_query: &mut Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    caster_team: Team,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(LIGHTNING_ARC_COUNT, params.count_mult),
        targets,
    );
    let damage =
        cl_constants::INITIAL_DAMAGE * DAMAGE_SCALE * params.empowerment * params.damage_mult;

    for (target_entity, target_pos) in &enemies {
        if let Ok((mut health, mut temp_hp, has_spell_shield, team)) =
            health_query.get_mut(*target_entity)
        {
            apply_spell_damage_with_team(
                commands,
                *target_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
                DamageType::Electric,
                has_spell_shield,
                caster_team,
                *team,
            );
        }

        chain_lightning_systems::spawn_arc(
            commands,
            assets,
            params.position,
            *target_pos,
            0,
            params.empowerment,
        );
    }
}

/// Auto-casts mini meteors at random enemies.
fn auto_cast_meteors(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(2, params.count_mult),
        targets,
    );
    let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;

    for (_, target_pos) in &enemies {
        let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);
        let damage = meteor_fall_constants::METEOR_DAMAGE * DAMAGE_SCALE * params.damage_mult;
        let explosion_radius = meteor_fall_constants::EXPLOSION_RADIUS * SIZE_SCALE;

        let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
            commands,
            assets,
            spawn_pos,
            Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
            damage,
            explosion_radius,
            params.empowerment,
            mini_radius,
            MeteorProjectileTalentFlags::default(),
        );
        commands.entity(entity).insert(CrystalSpawn {
            origin: params.position,
            max_range: params.range,
            lifetime: None,
        });
    }
}

/// Auto-casts Finger of Death beams at random enemies.
fn auto_cast_fod_beams(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    talent_cfg: &disintegrate_systems::TalentConfig,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(BEAM_COUNT, params.count_mult),
        targets,
    );
    let fod_damage_per_tick =
        finger_of_death_constants::DAMAGE * BEAM_DAMAGE_SCALE * params.damage_mult
            / (BEAM_DURATION / disintegrate_constants::DAMAGE_INTERVAL);
    let forked = talent_cfg.forked;
    let offsets: &[f32] = if forked {
        &[-FORKED_FAN_HALF_ANGLE, 0.0, FORKED_FAN_HALF_ANGLE]
    } else {
        &[0.0]
    };
    for (_, target_pos) in &enemies {
        let (base_direction, length) =
            crystal_beam_geometry(params.position, *target_pos, params.range);
        for &offset in offsets {
            let direction = if offset.abs() > 0.001 {
                Quat::from_axis_angle(Vec3::Y, offset) * base_direction
            } else {
                base_direction
            };
            let beam_entity = disintegrate_systems::spawn_beam_with_damage(
                commands,
                assets,
                params.position,
                direction,
                length,
                params.empowerment,
                fod_damage_per_tick,
                Some(talent_cfg),
                BEAM_DAMAGE_SCALE,
                offset,
            );
            commands.entity(beam_entity).insert(CrystalSpawn {
                origin: params.position,
                max_range: params.range,
                lifetime: Some(BEAM_DURATION),
            });
        }
    }
}

/// Spawns a crystal mini magic missile with pre-advanced homing.
///
/// Shared helper for both absorption and auto-cast. `target_teams` is the
/// caster's hostile team set — supplied by the calling system after reading
/// `PeerId` (guest targets Defenders, host targets Attackers).
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_crystal_mini_missile(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    crystal_position: Vec3,
    crystal_range: f32,
    initial_velocity: Vec3,
    wobble_offset: f32,
    target: Option<Entity>,
    visual_radius: f32,
    damage_mult: f32,
    target_teams: TargetTeams,
) {
    let mut mini_missile =
        crate::game::units::wizard::spells::magic_missile::components::MagicMissile::new(
            initial_velocity,
            wobble_offset,
            target,
            DAMAGE_SCALE * damage_mult,
            target_teams,
            crystal_range,
            crystal_position,
        );
    mini_missile.time_alive = MINI_MISSILE_HOMING_ADVANCE;

    commands.spawn((
        mini_missile,
        CrystalSpawn {
            origin: crystal_position,
            max_range: crystal_range,
            lifetime: None,
        },
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.crystal_mini_missile.clone()),
        Transform::from_translation(crystal_position).with_scale(Vec3::splat(visual_radius)),
        OnGameplayScreen,
    ));
}

// ===== Resonance Cascade Burst =====

/// When a crystal's resonance counter reaches the threshold, emit a powerful
/// damage burst to all enemies in range and reset the counter.
#[allow(clippy::too_many_arguments)]
pub(super) fn resonance_cascade_burst(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (&mut ArcaneCrystal, &mut ResonanceCascade),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    mut progress: ResMut<BattleTalentProgress>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    for (mut crystal, mut resonance) in &mut crystals {
        if resonance.absorptions < RESONANCE_CASCADE_THRESHOLD {
            continue;
        }

        resonance.absorptions = 0;
        crystal.trigger_pulse();

        let hit_count = crystal_aoe_burst(
            &mut commands,
            &visual_assets,
            crystal.position,
            crystal.range,
            RESONANCE_CASCADE_DAMAGE * crystal.empowerment,
            RESONANCE_CASCADE_RADIUS,
            2.0,
            0.3,
            &targets,
            &mut health_query,
            caster_team,
        );

        if hit_count > 0 {
            progress.increment(Spell::ArcaneCrystal, hit_count);
        }
    }
}

/// Shared AoE burst: damages all enemies in radius and spawns a visual ring.
/// Returns the number of enemies hit.
#[allow(clippy::too_many_arguments)]
pub(super) fn crystal_aoe_burst(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    position: Vec3,
    crystal_range: f32,
    damage: f32,
    radius: f32,
    visual_height: f32,
    visual_lifetime: f32,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    health_query: &mut Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    caster_team: Team,
) -> u32 {
    let mut hit_count: u32 = 0;

    for (target_entity, target_transform) in targets {
        let dist = xz_distance(position, target_transform.translation);
        if dist > radius {
            continue;
        }

        if let Ok((mut health, mut temp_hp, has_spell_shield, team)) =
            health_query.get_mut(target_entity)
        {
            apply_spell_damage_with_team(
                commands,
                target_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
                DamageType::Force,
                has_spell_shield,
                caster_team,
                *team,
            );
            hit_count += 1;
        }
    }

    // Spawn burst visual — expanding ring
    let burst_pos = Vec3::new(position.x, visual_height, position.z);
    commands.spawn((
        Mesh3d(visual_assets.unit_circle.clone()),
        MeshMaterial3d(visual_assets.arcane_crystal_indicator.clone()),
        Transform::from_translation(burst_pos)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
        CrystalSpawn {
            origin: position,
            max_range: crystal_range,
            lifetime: Some(visual_lifetime),
        },
        OnGameplayScreen,
    ));

    hit_count
}

// ===== Auto-Crystal Firing =====

/// Fires homing magic missiles at nearby enemies on a timer.
#[allow(clippy::too_many_arguments)]
pub(super) fn auto_crystal_fire(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (
            &ArcaneCrystal,
            &mut AutoCrystalTimer,
            Option<&mut ResonanceCascade>,
        ),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut progress: ResMut<BattleTalentProgress>,
    peer_id: Option<Res<crate::networking::crdt::PeerId>>,
) {
    let delta = time.delta_secs();
    let target_teams = crystal_target_teams(peer_id.as_deref());

    for (crystal, mut timer, mut resonance) in &mut crystals {
        timer.timer += delta;

        // Overcharged Matrix speeds up the fire rate
        let interval = AUTO_CRYSTAL_INTERVAL / crystal.count_mult;
        if timer.timer < interval {
            continue;
        }
        timer.timer -= interval;

        // Find a random enemy in range
        let targets = find_random_enemies_in_range(
            &mut game_rng.0,
            crystal.position,
            crystal.range,
            1,
            &enemies,
            target_teams,
        );

        let Some((target_entity, target_pos)) = targets.first() else {
            continue;
        };

        let direction = (*target_pos - crystal.position).normalize();
        let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
        let initial_velocity = direction * speed;
        let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

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
            target_teams,
        );

        // Increment resonance cascade counter on each turret shot
        increment_resonance(&mut resonance);

        progress.increment(Spell::ArcaneCrystal, 1);
    }
}

// ===== Crystal Network Chaining =====

/// When a crystal absorbs a spell, chain the absorption to nearby crystals
/// with CrystalNetwork marker. Each chain re-triggers the crystal's emission
/// at reduced damage.
#[allow(clippy::too_many_arguments)]
pub(super) fn crystal_network_chain(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (Entity, &mut ArcaneCrystal),
        (
            With<CrystalNetwork>,
            Without<crate::game::multiplayer::components::GhostSpellEffect>,
        ),
    >,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut progress: ResMut<BattleTalentProgress>,
    peer_id: Option<Res<crate::networking::crdt::PeerId>>,
) {
    // Early return if no crystal absorbed this frame
    if !crystals.iter().any(|(_, c)| c.just_absorbed) {
        return;
    }
    let target_teams = crystal_target_teams(peer_id.as_deref());

    // Collect crystal positions and remembered spells for chaining
    let crystal_data: Vec<(Entity, Vec3, f32, f32, Option<RememberedSpell>, bool)> = crystals
        .iter()
        .map(|(e, c)| {
            (
                e,
                c.position,
                c.range,
                c.empowerment,
                c.remembered_spell,
                c.just_absorbed,
            )
        })
        .collect();

    // For each crystal that just pulsed, check if nearby crystals should chain
    for (source_entity, source_pos, _source_range, _source_emp, source_spell, source_pulsed) in
        &crystal_data
    {
        if !source_pulsed {
            continue;
        }
        let Some(remembered) = source_spell else {
            continue;
        };

        for (target_entity, target_pos, _target_range, _target_emp, _target_spell, target_pulsed) in
            &crystal_data
        {
            if source_entity == target_entity || *target_pulsed {
                continue;
            }

            let dist = xz_distance(*source_pos, *target_pos);

            if dist > CRYSTAL_NETWORK_CHAIN_RANGE {
                continue;
            }

            // Chain: trigger the target crystal to emit based on the source's remembered spell
            if let Ok((_, mut target_crystal)) = crystals.get_mut(*target_entity) {
                target_crystal.trigger_pulse();

                // Emit a reduced emission from the chained crystal
                let chain_damage_scale = DAMAGE_SCALE * target_crystal.damage_mult * 0.5;
                match remembered {
                    RememberedSpell::MagicMissile => {
                        let count =
                            scaled_count(MINI_MISSILE_COUNT / 2 + 1, target_crystal.count_mult);
                        let mini_targets = find_random_enemies_in_range(
                            &mut game_rng.0,
                            target_crystal.position,
                            target_crystal.range,
                            count,
                            &enemies,
                            target_teams,
                        );
                        let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;
                        for (te, tp) in &mini_targets {
                            let direction = (*tp - target_crystal.position).normalize();
                            let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
                            let rng = &mut game_rng.0;
                            let wobble = rng.random_range(0.0..std::f32::consts::TAU);
                            spawn_crystal_mini_missile(
                                &mut commands,
                                &visual_assets,
                                target_crystal.position,
                                target_crystal.range,
                                direction * speed,
                                wobble,
                                Some(*te),
                                mini_radius,
                                target_crystal.damage_mult,
                                target_teams,
                            );
                        }
                        progress.increment(Spell::ArcaneCrystal, count as u32);
                    }
                    RememberedSpell::Fireball => {
                        let count = scaled_count(MINI_FB_COUNT / 2 + 1, target_crystal.count_mult);
                        let fire_targets = find_random_targets_in_range(
                            &mut game_rng.0,
                            target_crystal.position,
                            target_crystal.range,
                            count,
                            &targets,
                        );
                        let mini_radius =
                            fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;
                        for (_, tp) in &fire_targets {
                            let ground = Vec3::new(tp.x, 0.0, tp.z);
                            let direction = (ground - target_crystal.position).normalize();
                            let velocity =
                                direction * fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
                            let entity = fireball_systems::spawn_fireball_entity(
                                &mut commands,
                                &visual_assets,
                                target_crystal.position,
                                velocity,
                                fireball_constants::DAMAGE_PER_TICK * chain_damage_scale,
                                fireball_constants::DAMAGE_TYPE,
                                fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                                fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
                                target_crystal.empowerment * chain_damage_scale,
                                mini_radius,
                            );
                            commands.entity(entity).insert(CrystalSpawn {
                                origin: target_crystal.position,
                                max_range: target_crystal.range,
                                lifetime: None,
                            });
                        }
                        progress.increment(Spell::ArcaneCrystal, count as u32);
                    }
                    RememberedSpell::Meteor => {
                        let count = scaled_count(1, target_crystal.count_mult);
                        let meteor_targets = find_random_targets_in_range(
                            &mut game_rng.0,
                            target_crystal.position,
                            target_crystal.range,
                            count,
                            &targets,
                        );
                        let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;
                        for (_, tp) in &meteor_targets {
                            let spawn_pos = Vec3::new(tp.x, MINI_METEOR_SPAWN_HEIGHT, tp.z);
                            let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
                                &mut commands,
                                &visual_assets,
                                spawn_pos,
                                Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
                                meteor_fall_constants::METEOR_DAMAGE * chain_damage_scale,
                                meteor_fall_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                                target_crystal.empowerment,
                                mini_radius,
                                MeteorProjectileTalentFlags::default(),
                            );
                            commands.entity(entity).insert(CrystalSpawn {
                                origin: target_crystal.position,
                                max_range: target_crystal.range,
                                lifetime: None,
                            });
                        }
                        progress.increment(Spell::ArcaneCrystal, count as u32);
                    }
                    _ => {
                        // Chain lightning, disintegrate, FoD — just pulse without extra emission
                        // (beams are too complex to duplicate in chain)
                    }
                }

                // Spawn visual arc between crystals
                chain_lightning_systems::spawn_arc(
                    &mut commands,
                    &visual_assets,
                    *source_pos,
                    target_crystal.position,
                    0,
                    target_crystal.empowerment,
                );
            }
        }
    }
}
