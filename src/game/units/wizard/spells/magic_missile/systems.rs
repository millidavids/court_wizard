use std::cmp::Ordering;

use bevy::prelude::*;
use rand::Rng;

use super::super::super::components::{LocalWizard, Mana, PrimedSpell, Spell, Wizard};
use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::constants::SPELL_ORIGIN;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::crdt::PeerId;

/// Talent-modified missile parameters.
struct MissileParams {
    missile_count: u32,
    damage_mult: f32,
    mana_cost: f32,
    cooldown_mult: f32,
    piercing: bool,
    heavy: bool,
    storm_wobble: bool,
    detonation: bool,
    seeker_swarm: bool,
    guided: bool,
}

/// Computes talent-modified parameters for magic missile casting.
fn compute_missile_params(talents: Option<&ActiveTalents>, wizard: &Wizard) -> MissileParams {
    let t1 = talents.and_then(|t| t.get_selection(Spell::MagicMissile, 0));
    let t2 = talents.and_then(|t| t.get_selection(Spell::MagicMissile, 1));
    let t3 = talents.and_then(|t| t.get_selection(Spell::MagicMissile, 2));

    // Tier 1: all roughly equivalent at ~4.0 effective damage (base 3.0)
    let (missile_count, damage_mult, mana_mult, cooldown_mult) = match t1 {
        Some(0) => (5, 0.8, 1.0, 1.0), // Volley: 5 missiles at 80% = 4.0 effective
        Some(1) => (1, 4.0, 1.0, 1.0), // Heavy Ordnance: 1 missile at 4x = 4.0 effective
        Some(2) => (3, 1.0, 1.5, 0.75), // Swift Salvo: 75% cooldown, +50% mana
        _ => (constants::MISSILES_PER_CAST, 1.0, 1.0, 1.0),
    };

    // Tier 3 talent effects on missile count and damage
    let (missile_count, damage_mult) = match t3 {
        Some(0) => (missile_count * 4, 0.25), // Missile Storm: 4x missiles at 25% damage
        Some(2) => (missile_count, damage_mult * 1.5), // Guided Devastation: 1.5x damage
        _ => (missile_count, damage_mult),
    };

    MissileParams {
        missile_count,
        damage_mult,
        mana_cost: constants::MANA_COST * wizard.mana_cost_multiplier * mana_mult,
        cooldown_mult,
        piercing: t2 == Some(2),
        heavy: t1 == Some(1),
        storm_wobble: t3 == Some(0),
        detonation: t3 == Some(1),
        seeker_swarm: t2 == Some(0),
        guided: t3 == Some(2),
    }
}

/// Local wizard magic missile casting — instant cast with cooldown.
///
/// On click, spawns a volley of missiles immediately.
/// Only fires on initial click, not while held. A cooldown prevents spam.
/// Talent effects modify missile count, damage, cooldown, and mana cost.
/// With Arcane Barrage talent, spawns a concentration entity instead.
#[allow(clippy::too_many_arguments)]
pub fn handle_magic_missile_casting(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (
            Entity,
            &mut Mana,
            &PrimedSpell,
            &Wizard,
            Option<&MagicMissileCooldown>,
        ),
        With<LocalWizard>,
    >,
    camera_query_3d: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: Query<(Entity, &Transform, &ArcaneCrystal)>,
    peer_id: Option<Res<PeerId>>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    existing_barrage: Query<Entity, With<ArcaneBarrage>>,
) {
    // Only fire on initial click, not while mouse is held
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let cursor_pos = get_cursor_world_position(&camera_query_3d, &corrected_cursor);

    let Ok((wizard_entity, mut mana, primed_spell, wizard, cooldown)) = wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::MagicMissile {
        return;
    }

    // Check cooldown
    if cooldown.is_some_and(|cd| cd.remaining > 0.0) {
        return;
    }

    // Host/SP wizard targets attackers; guest wizard targets defenders
    let target_teams = if peer_id.is_some_and(|p| p.0 == PeerId::GUEST) {
        TargetTeams::DefendersAndUndead
    } else {
        TargetTeams::AttackersAndUndead
    };

    let talents = active_talents.as_deref();
    let params = compute_missile_params(talents, wizard);

    // Arcane Barrage (tier 2, choice 1): spawn concentration entity instead of normal cast
    let t2 = talents.and_then(|t| t.get_selection(Spell::MagicMissile, 1));
    if t2 == Some(1) {
        // Requires initial mana cost to activate
        if !mana.consume(params.mana_cost) {
            return;
        }

        // Despawn any existing barrage
        for entity in existing_barrage.iter() {
            commands.entity(entity).try_despawn();
        }

        // 5s base interval; Swift Salvo reduces it
        let interval = 5.0 * params.cooldown_mult;

        commands.spawn((
            ArcaneBarrage {
                timer: 0.0, // Fire immediately on first tick
                interval,
                missile_count: params.missile_count,
                damage_mult: params.damage_mult,
                piercing: params.piercing,
                heavy: params.heavy,
                storm_wobble: params.storm_wobble,
                detonation: params.detonation,
                seeker_swarm: params.seeker_swarm,
                guided: params.guided,
                target_teams,
                spell_range: wizard.spell_range,
                empowerment: primed_spell.empowerment,
            },
            ConcentrationSpell {
                spell_name: "Arcane Barrage",
            },
            OnGameplayScreen,
        ));
        return;
    }

    if !mana.consume(params.mana_cost) {
        return;
    }

    let spawn_origin = SPELL_ORIGIN;

    // Spawn missiles with modified parameters
    for _ in 0..params.missile_count {
        spawn_magic_missile_with_talents(
            &mut commands,
            &visual_assets,
            &camera_query,
            &targets,
            &crystals,
            wizard.spell_range,
            primed_spell.empowerment,
            cursor_pos,
            spawn_origin,
            target_teams,
            &params,
        );
    }

    audio::play_sfx(
        &mut commands,
        &sfx.magic_missile_cast,
        spawn_origin,
        &config,
        &sfx,
    );

    // Set cooldown (modified by talents)
    commands.entity(wizard_entity).insert(MagicMissileCooldown {
        remaining: constants::COOLDOWN * params.cooldown_mult,
    });
}

/// Ticks the Arcane Barrage concentration entity, periodically firing missile volleys.
#[allow(clippy::too_many_arguments)]
pub fn update_arcane_barrage(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut barrage_query: Query<&mut ArcaneBarrage>,
    camera_query_3d: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: Query<(Entity, &Transform, &ArcaneCrystal)>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
) {
    let Ok(mut barrage) = barrage_query.single_mut() else {
        return;
    };

    barrage.timer += time.delta_secs();
    if barrage.timer < barrage.interval {
        return;
    }
    barrage.timer -= barrage.interval;

    let cursor_pos = get_cursor_world_position(&camera_query_3d, &corrected_cursor);
    let spawn_origin = SPELL_ORIGIN;

    let params = MissileParams {
        missile_count: barrage.missile_count,
        damage_mult: barrage.damage_mult,
        mana_cost: 0.0,
        cooldown_mult: 1.0,
        piercing: barrage.piercing,
        heavy: barrage.heavy,
        storm_wobble: barrage.storm_wobble,
        detonation: barrage.detonation,
        seeker_swarm: barrage.seeker_swarm,
        guided: barrage.guided,
    };

    for _ in 0..params.missile_count {
        spawn_magic_missile_with_talents(
            &mut commands,
            &visual_assets,
            &camera_query,
            &targets,
            &crystals,
            barrage.spell_range,
            barrage.empowerment,
            cursor_pos,
            spawn_origin,
            barrage.target_teams,
            &params,
        );
    }

    audio::play_sfx(
        &mut commands,
        &sfx.magic_missile_cast,
        spawn_origin,
        &config,
        &sfx,
    );
}

/// Spawns a magic missile with talent modifications applied.
#[allow(clippy::too_many_arguments)]
fn spawn_magic_missile_with_talents(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    camera_query: &Query<&GlobalTransform, With<Camera>>,
    targets: &Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: &Query<(Entity, &Transform, &ArcaneCrystal)>,
    spell_range: f32,
    empowerment: f32,
    cursor_world_pos: Option<Vec3>,
    spawn_origin: Vec3,
    target_teams: TargetTeams,
    params: &MissileParams,
) {
    let spawn_pos = spawn_origin + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);

    let crystal_target: Option<Entity> = cursor_world_pos.and_then(|cursor_pos| {
        let mut closest: Option<(Entity, f32)> = None;
        for (entity, transform, crystal) in crystals.iter() {
            let dist = Vec3::new(
                cursor_pos.x - transform.translation.x,
                0.0,
                cursor_pos.z - transform.translation.z,
            )
            .length();
            if dist <= crystal.collision_radius * 5.0 {
                match closest {
                    None => closest = Some((entity, dist)),
                    Some((_, prev_dist)) if dist < prev_dist => closest = Some((entity, dist)),
                    _ => {}
                }
            }
        }
        closest.map(|(e, _)| e)
    });

    let mut rng = rand::thread_rng();

    // Guided missiles don't need a target — they steer toward the cursor
    let target = if params.guided {
        None
    } else if crystal_target.is_some() {
        crystal_target
    } else {
        let enemies_in_range: Vec<Entity> = targets
            .iter()
            .filter(|(_, _, team)| target_teams.matches(team))
            .filter(|(_, transform, _)| {
                let distance = spawn_pos.distance(transform.translation);
                distance <= spell_range
            })
            .map(|(entity, _, _)| entity)
            .collect();

        if !enemies_in_range.is_empty() {
            if let Some(cursor_pos) = cursor_world_pos {
                let mut total_weight = 0.0;
                let weighted_targets: Vec<(Entity, f32)> = enemies_in_range
                    .iter()
                    .filter_map(|&entity| {
                        targets.get(entity).ok().map(|(_, transform, _)| {
                            let distance = cursor_pos.distance(transform.translation);
                            let weight = 1.0
                                / (distance.powi(constants::CURSOR_TARGETING_WEIGHT_POWER) + 1.0);
                            total_weight += weight;
                            (entity, weight)
                        })
                    })
                    .collect();

                if total_weight > 0.0 {
                    let mut random_value = rng.gen_range(0.0..total_weight);
                    let mut selected_target = None;
                    for (entity, weight) in weighted_targets {
                        random_value -= weight;
                        if random_value <= 0.0 {
                            selected_target = Some(entity);
                            break;
                        }
                    }
                    selected_target.or_else(|| enemies_in_range.first().copied())
                } else {
                    let index = rng.gen_range(0..enemies_in_range.len());
                    Some(enemies_in_range[index])
                }
            } else {
                let index = rng.gen_range(0..enemies_in_range.len());
                Some(enemies_in_range[index])
            }
        } else {
            targets
                .iter()
                .filter(|(_, _, team)| target_teams.matches(team))
                .min_by(|a, b| {
                    let dist_a = spawn_pos.distance(a.1.translation);
                    let dist_b = spawn_pos.distance(b.1.translation);
                    dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
                })
                .map(|(entity, _, _)| entity)
        }
    };

    // Random initial velocity
    let horizontal_x = rng.gen_range(constants::HORIZONTAL_VEL_MIN..constants::HORIZONTAL_VEL_MAX);
    let horizontal_z = rng.gen_range(constants::HORIZONTAL_VEL_MIN..constants::HORIZONTAL_VEL_MAX);
    let vertical = rng.gen_range(constants::VERTICAL_VEL_MIN..constants::VERTICAL_VEL_MAX);
    let mut initial_velocity = Vec3::new(horizontal_x, vertical, horizontal_z);

    // Storm wobble: increase initial velocity randomness
    if params.storm_wobble {
        initial_velocity *= 1.5;
    }

    if let Ok(camera_transform) = camera_query.single() {
        let camera_pos = camera_transform.translation();
        let to_camera = (camera_pos - spawn_pos).normalize_or_zero();
        let camera_arc_speed =
            rng.gen_range(constants::CAMERA_ARC_SPEED_MIN..constants::CAMERA_ARC_SPEED_MAX);
        let camera_arc = to_camera * camera_arc_speed;
        initial_velocity += camera_arc;
    }

    let wobble_offset = rng.gen_range(0.0..std::f32::consts::TAU);

    // Construct missile with modified damage and radius
    let scale = empowerment;
    let radius_mult = if params.heavy { 2.0 } else { 1.0 };

    let mut missile = MagicMissile::new(
        initial_velocity,
        wobble_offset,
        target,
        empowerment,
        target_teams,
        spell_range,
        spawn_pos,
    );
    missile.damage = constants::DAMAGE * scale * params.damage_mult;
    missile.radius = constants::COLLISION_RADIUS * scale * radius_mult;
    missile.piercing = params.piercing;
    missile.detonation = params.detonation;
    missile.seeker_swarm = params.seeker_swarm;
    missile.guided = params.guided;

    let entity = commands
        .spawn((
            Mesh3d(assets.magic_missile_mesh.clone()),
            MeshMaterial3d(assets.magic_missile.clone()),
            Transform::from_translation(spawn_pos),
            missile,
            OnGameplayScreen,
        ))
        .id();

    vfx::systems::spawn_missile_glow(
        commands,
        assets,
        entity,
        spawn_pos,
        constants::COLLISION_RADIUS * radius_mult,
    );
}

/// Ticks down the magic missile cooldown timer each frame.
pub fn tick_magic_missile_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut MagicMissileCooldown)>,
) {
    for (entity, mut cooldown) in &mut cooldowns {
        cooldown.remaining -= time.delta_secs();
        if cooldown.remaining <= 0.0 {
            commands.entity(entity).remove::<MagicMissileCooldown>();
        }
    }
}

/// Updates magic missile movement with homing and wobble.
///
/// Missiles lock onto their initial target and only retarget if it despawns.
#[allow(clippy::too_many_arguments)]
pub fn move_magic_missiles(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut missiles: Query<(&mut Transform, &mut MagicMissile)>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystal_transforms: Query<&Transform, (With<ArcaneCrystal>, Without<MagicMissile>)>,
    camera_query_3d: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut sparkle_timer: Local<f32>,
) {
    // Track sparkle spawn timing
    *sparkle_timer += time.delta_secs();
    let should_spawn_sparkles = *sparkle_timer >= vfx::constants::SPARKLE_SPAWN_INTERVAL;
    if should_spawn_sparkles {
        *sparkle_timer -= vfx::constants::SPARKLE_SPAWN_INTERVAL;
    }

    // Get cursor world position once for guided missiles
    let cursor_world_pos = get_cursor_world_position(&camera_query_3d, &corrected_cursor);

    for (mut missile_transform, mut missile) in &mut missiles {
        missile.time_alive += time.delta_secs();

        // Guided missiles: steer toward cursor, no target homing
        if missile.guided {
            let base_max_speed = missile.current_max_speed();

            if let Some(cursor_pos) = cursor_world_pos {
                let to_cursor = cursor_pos - missile_transform.translation;
                let distance = to_cursor.length();

                let proximity_speed_multiplier = if distance < constants::SLOWDOWN_DISTANCE {
                    let t = (distance / constants::SLOWDOWN_DISTANCE).clamp(0.0, 1.0);
                    let min_multiplier = constants::MIN_PROXIMITY_SPEED / base_max_speed;
                    min_multiplier + (1.0 - min_multiplier) * t
                } else {
                    1.0
                };

                let max_speed = base_max_speed * proximity_speed_multiplier;
                let direction = to_cursor.normalize_or_zero();

                // Smooth steering: blend current velocity toward cursor direction
                let steer_strength = 8.0;
                let desired_velocity = direction * max_speed;
                let current_velocity = missile.velocity;
                missile.velocity += (desired_velocity - current_velocity)
                    * (steer_strength * time.delta_secs()).min(1.0);

                let current_speed = missile.velocity.length();
                if current_speed > max_speed {
                    missile.velocity = missile.velocity.normalize() * max_speed;
                }
            }
            // If no cursor position, continue with current velocity

            missile_transform.translation += missile.velocity * time.delta_secs();

            // Spawn sparkle trail
            if should_spawn_sparkles {
                vfx::systems::spawn_missile_sparkles(
                    &mut commands,
                    &visual_assets,
                    missile_transform.translation,
                    missile.velocity,
                    time.elapsed_secs(),
                );
            }
            continue;
        }

        // Normal homing logic for non-guided missiles
        // Check if current target still exists (could be a unit or a crystal)
        let target_exists = missile.target.is_some_and(|target_entity| {
            targets.get(target_entity).is_ok() || crystal_transforms.get(target_entity).is_ok()
        });

        // Retarget if current target despawned
        if !target_exists {
            let mut rng = rand::thread_rng();

            let enemies_in_range: Vec<Entity> = targets
                .iter()
                .filter(|(_, _, team)| missile.target_teams.matches(team))
                .filter(|(_, transform, _)| {
                    let distance = missile_transform
                        .translation
                        .distance(transform.translation);
                    distance <= missile.spell_range
                })
                .map(|(entity, _, _)| entity)
                .collect();

            missile.target = if !enemies_in_range.is_empty() {
                let index = rng.gen_range(0..enemies_in_range.len());
                Some(enemies_in_range[index])
            } else {
                targets
                    .iter()
                    .filter(|(_, _, team)| missile.target_teams.matches(team))
                    .min_by(|a, b| {
                        let dist_a = missile_transform.translation.distance(a.1.translation);
                        let dist_b = missile_transform.translation.distance(b.1.translation);
                        dist_a
                            .partial_cmp(&dist_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(entity, _, _)| entity)
            };
        }

        // Get current target's transform (check units first, then crystals)
        let target_transform = missile.target.and_then(|target_entity| {
            targets
                .get(target_entity)
                .ok()
                .map(|(_, transform, _)| transform)
                .or_else(|| crystal_transforms.get(target_entity).ok())
        });

        if let Some(target_transform) = target_transform {
            let to_target = target_transform.translation - missile_transform.translation;
            let distance_to_target = to_target.length();
            let current_homing_strength = missile.current_homing_strength();

            let base_max_speed = missile.current_max_speed();

            let proximity_speed_multiplier = if distance_to_target < constants::SLOWDOWN_DISTANCE {
                let t = (distance_to_target / constants::SLOWDOWN_DISTANCE).clamp(0.0, 1.0);
                let min_multiplier = constants::MIN_PROXIMITY_SPEED / base_max_speed;
                min_multiplier + (1.0 - min_multiplier) * t
            } else {
                1.0
            };

            let max_speed = base_max_speed * proximity_speed_multiplier;

            let homing_force = if current_homing_strength.is_infinite() {
                to_target.normalize_or_zero()
            } else {
                to_target.normalize_or_zero() * current_homing_strength
            };

            let wobble = if missile.time_alive < constants::PERFECT_TRACKING_TIME {
                let t = missile.time_alive * constants::WOBBLE_FREQUENCY + missile.wobble_offset;

                Vec3::new(
                    t.sin() * constants::WOBBLE_AMPLITUDE,
                    (t * constants::WOBBLE_Y_FREQ_MULTIPLIER).cos()
                        * constants::WOBBLE_AMPLITUDE
                        * constants::WOBBLE_Y_AMPLITUDE_MULTIPLIER,
                    (t * constants::WOBBLE_Z_FREQ_MULTIPLIER).sin() * constants::WOBBLE_AMPLITUDE,
                )
            } else {
                Vec3::ZERO
            };

            if current_homing_strength.is_infinite() {
                missile.velocity = homing_force * max_speed;
            } else {
                missile.velocity += (homing_force + wobble) * time.delta_secs();

                let current_speed = missile.velocity.length();
                if current_speed > max_speed {
                    missile.velocity = missile.velocity.normalize() * max_speed;
                }
            }

            missile_transform.translation += missile.velocity * time.delta_secs();
        } else {
            missile_transform.translation += missile.velocity * time.delta_secs();
        }

        // Spawn sparkle trail particles
        if should_spawn_sparkles {
            vfx::systems::spawn_missile_sparkles(
                &mut commands,
                &visual_assets,
                missile_transform.translation,
                missile.velocity,
                time.elapsed_secs(),
            );
        }
    }
}

/// Checks for magic missile collisions with enemies (Attackers and Undead).
///
/// When a missile hits an enemy, it deals 50 damage and despawns.
pub fn check_magic_missile_collisions(
    mut commands: Commands,
    mut missiles: Query<(Entity, &Transform, &mut MagicMissile)>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &Team,
            Has<SpellShield>,
        ),
        (Without<MagicMissile>, Without<Corpse>),
    >,
    walls: Query<&WallOfStone>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    visual_assets: Res<SpellVisualAssets>,
) {
    // Collect split spawns to avoid borrow conflicts
    let mut splits: Vec<(Vec3, MagicMissile)> = Vec::new();

    for (missile_entity, missile_transform, mut missile) in &mut missiles {
        // Wall collision
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(missile_transform.translation)
                && missile_transform.translation.y <= wall.height
            {
                commands.entity(missile_entity).try_despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        let mut should_despawn = false;
        let mut target_killed = false;
        for (enemy_entity, enemy_transform, mut health, mut temp_hp, team, has_spell_shield) in
            &mut enemies
        {
            if !missile.target_teams.matches(team) {
                continue;
            }

            let distance = missile_transform
                .translation
                .distance(enemy_transform.translation);

            // Check collision
            if distance < missile.radius {
                if has_spell_shield {
                    continue;
                }
                apply_spell_damage(
                    &mut commands,
                    enemy_entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    missile.damage,
                    DamageType::Force,
                    false,
                );
                target_killed = health.current <= 0.0;
                if let Some(ref mut progress) = talent_progress {
                    progress.increment(Spell::MagicMissile, 1);
                }

                // Arcane Detonation: spawn small AoE on impact
                if missile.detonation {
                    spawn_missile_detonation(
                        &mut commands,
                        &visual_assets,
                        missile_transform.translation,
                        missile.damage * 0.2,
                    );
                }

                // Piercing: pass through first target
                if missile.piercing && missile.pierced_count < 1 {
                    missile.pierced_count += 1;
                    // Retarget to a different enemy
                    missile.target = None;
                    continue;
                }

                should_despawn = true;
                break;
            }
        }

        // Seeker Swarm: split into 2 half-damage missiles on kill (max 2 generations)
        if should_despawn && target_killed && missile.seeker_swarm && missile.split_generation < 2 {
            let mut rng = rand::thread_rng();
            for _ in 0..2 {
                let mut split = MagicMissile::new(
                    Vec3::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-500.0..500.0),
                        rng.gen_range(-1000.0..1000.0),
                    ),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    None, // Will retarget automatically
                    missile.empowerment,
                    missile.target_teams,
                    missile.spell_range,
                    missile_transform.translation,
                );
                split.damage = missile.damage * 0.2;
                split.radius = missile.radius;
                split.piercing = missile.piercing;
                split.detonation = missile.detonation;
                split.seeker_swarm = true;
                split.split_generation = missile.split_generation + 1;
                split.guided = missile.guided;
                splits.push((missile_transform.translation, split));
            }
        }

        if should_despawn {
            commands.entity(missile_entity).try_despawn();
        }
    }

    // Spawn split missiles outside the query loop
    for (pos, split_missile) in splits {
        let radius_mult =
            split_missile.radius / (constants::COLLISION_RADIUS * split_missile.empowerment);
        let entity = commands
            .spawn((
                Mesh3d(visual_assets.magic_missile_mesh.clone()),
                MeshMaterial3d(visual_assets.magic_missile.clone()),
                Transform::from_translation(pos),
                split_missile,
                OnGameplayScreen,
            ))
            .id();

        vfx::systems::spawn_missile_glow(
            &mut commands,
            &visual_assets,
            entity,
            pos,
            constants::COLLISION_RADIUS * radius_mult,
        );
    }
}

/// Spawns a small AoE detonation effect for the Arcane Detonation talent.
/// Uses pink magic missile material instead of fire material, deals Force damage (no burning).
fn spawn_missile_detonation(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    damage: f32,
) {
    use crate::game::units::DamageType;
    use crate::game::units::wizard::spells::fireball::components::FireballExplosion;

    let mut explosion = FireballExplosion::new(
        position,
        40.0, // small radius
        damage,
        DamageType::Force,
        1.0,
    );
    explosion.source_spell = Spell::MagicMissile;

    commands.spawn((
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.magic_missile.clone()),
        Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
        explosion,
        OnGameplayScreen,
    ));
}

/// Despawns magic missiles that exit their spell range or have been alive too long.
pub fn despawn_distant_magic_missiles(
    mut commands: Commands,
    missiles: Query<(Entity, &Transform, &MagicMissile)>,
) {
    for (entity, transform, missile) in &missiles {
        let distance_from_origin = transform.translation.distance(missile.origin_pos);
        if distance_from_origin > missile.spell_range || missile.time_alive > 5.0 {
            commands.entity(entity).try_despawn();
        }
    }
}
