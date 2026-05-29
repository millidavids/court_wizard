//! Magic missile casting and projectile spawn.

use std::cmp::Ordering;

use bevy::prelude::*;
use rand::Rng;

use super::super::super::components::{LocalWizard, Mana, PrimedSpell, Spell, Wizard};
use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseLeftHeld;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::networking::snapshot::SpellSoundId;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
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
    time: Res<Time>,
    mut mouse_left_held: MessageReader<MouseLeftHeld>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
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
    local_origin: Res<LocalSpellOrigin>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: Query<(Entity, &Transform, &ArcaneCrystal)>,
    peer_id: Option<Res<PeerId>>,
    (sfx, config, active_talents, mut pending_cast_events): (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        Option<Res<ActiveTalents>>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
    existing_barrage: Query<Entity, With<ArcaneBarrage>>,
    // Hard guard against same-window double-cast. Tracks the `Time::elapsed_secs`
    // value at the most recent successful cast; re-entry within half the
    // cooldown window is refused regardless of whether the
    // `MagicMissileCooldown` component has been applied by deferred commands.
    mut last_cast_elapsed: Local<f32>,
) {
    // Fire while the button is held so holding RT/mouse auto-cycles at the
    // cooldown rate. `MouseLeftHeld` fires every frame the button is down
    // (including the press frame). The hard-guard timestamp gate below
    // prevents spam.
    if mouse_left_held.read().next().is_none() {
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

    // Host/SP wizard targets attackers; guest wizard targets defenders
    let target_teams = if peer_id.is_some_and(|p| p.0 == PeerId::GUEST) {
        TargetTeams::DefendersAndUndead
    } else {
        TargetTeams::AttackersAndUndead
    };

    let talents = active_talents.as_deref();
    let params = compute_missile_params(talents, wizard);

    // Authoritative cooldown gate. The `MagicMissileCooldown` component is the
    // in-world signal but its insert is deferred via `Commands`, so on the
    // frame after a cast the component is invisible to this query. The
    // `Local<f32>` timestamp is the source of truth — refuse re-entry until
    // the talent-scaled cooldown has actually elapsed since the last cast.
    let actual_cooldown = constants::COOLDOWN * params.cooldown_mult;
    let now = time.elapsed_secs();
    if *last_cast_elapsed > 0.0 && now - *last_cast_elapsed < actual_cooldown {
        return;
    }
    // Backup gate against an out-of-band cooldown component from another
    // codepath (e.g. spawn-time pre-cooldowns).
    if cooldown.is_some_and(|cd| cd.remaining > 0.0) {
        return;
    }

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
                spell_name: constants::ARCANE_BARRAGE_NAME,
                mana_cost: constants::MANA_COST,
            },
            OnGameplayScreen,
        ));
        *last_cast_elapsed = now;
        return;
    }

    if !mana.consume(params.mana_cost) {
        return;
    }

    vfx::systems::spawn_school_flare_synced(
        &mut commands,
        &visual_assets,
        &mut pending_cast_events,
        local_origin.0,
        vfx::systems::SpellSchool::Arcane,
        time.elapsed_secs(),
    );

    let spawn_origin = local_origin.0;

    // Spawn missiles with modified parameters
    for _ in 0..params.missile_count {
        spawn_magic_missile_with_talents(
            &mut game_rng.0,
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

    audio::play_sfx_synced(
        &mut commands,
        &mut pending_cast_events,
        SpellSoundId::MagicMissileCast,
        spawn_origin,
        &config,
        &sfx,
    );

    // Set cooldown (modified by talents)
    commands.entity(wizard_entity).insert(MagicMissileCooldown {
        remaining: constants::COOLDOWN * params.cooldown_mult,
    });
    *last_cast_elapsed = now;
}

/// Ticks the Arcane Barrage concentration entity, periodically firing missile volleys.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
pub fn update_arcane_barrage(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut barrage_query: Query<&mut ArcaneBarrage>,
    camera_query_3d: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    local_origin: Res<LocalSpellOrigin>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: Query<(Entity, &Transform, &ArcaneCrystal)>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
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
    let spawn_origin = local_origin.0;

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
            &mut game_rng.0,
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

    audio::play_sfx_synced(
        &mut commands,
        &mut pending_cast_events,
        SpellSoundId::MagicMissileCast,
        spawn_origin,
        &config,
        &sfx,
    );
}

/// Spawns a magic missile with talent modifications applied.
#[allow(clippy::too_many_arguments)]
fn spawn_magic_missile_with_talents(
    rng: &mut impl Rng,
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
                    let mut random_value = rng.random_range(0.0..total_weight);
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
                    let index = rng.random_range(0..enemies_in_range.len());
                    Some(enemies_in_range[index])
                }
            } else {
                let index = rng.random_range(0..enemies_in_range.len());
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
    let horizontal_x =
        rng.random_range(constants::HORIZONTAL_VEL_MIN..constants::HORIZONTAL_VEL_MAX);
    let horizontal_z =
        rng.random_range(constants::HORIZONTAL_VEL_MIN..constants::HORIZONTAL_VEL_MAX);
    let vertical = rng.random_range(constants::VERTICAL_VEL_MIN..constants::VERTICAL_VEL_MAX);
    let mut initial_velocity = Vec3::new(horizontal_x, vertical, horizontal_z);

    // Storm wobble: increase initial velocity randomness
    if params.storm_wobble {
        initial_velocity *= 1.5;
    }

    if let Ok(camera_transform) = camera_query.single() {
        let camera_pos = camera_transform.translation();
        let to_camera = (camera_pos - spawn_pos).normalize_or_zero();
        let camera_arc_speed =
            rng.random_range(constants::CAMERA_ARC_SPEED_MIN..constants::CAMERA_ARC_SPEED_MAX);
        let camera_arc = to_camera * camera_arc_speed;
        initial_velocity += camera_arc;
    }

    let wobble_offset = rng.random_range(0.0..std::f32::consts::TAU);

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
