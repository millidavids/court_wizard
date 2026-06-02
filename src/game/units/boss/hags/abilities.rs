//! Hag abilities: justina lightning/fireball, josephina leap/maul/consume, martina teleport/mind-control.

use super::core::{hag_casting_animation, restore_hag_walking_pose, set_hag_attack_pose_frame};
use std::collections::HashSet;

use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::{HagAssets, HagDeathTracker};
use crate::game::components::{OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, StagingAttacker};
use crate::game::units::components::Knockback;
use crate::game::units::components::{
    AnimationOverride, AttackTiming, BanishedModifier, Corpse, FacingDirection, Health, Hitbox,
    KingsGuard, MindControlled, MovementSpeed, RetaliationTarget, TargetingVelocity, Team,
    TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::king::components::King;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

type MindControlTargetData = (
    Entity,
    &'static Transform,
    &'static Team,
    &'static FlowFieldInfluence,
);
type MindControlTargetFilter = (
    Without<Hag>,
    Without<Corpse>,
    Without<MindControlled>,
    Without<Wizard>,
    Without<BanishedModifier>,
);

/// Builds a `WalkingAnimation` configured for the hag sprite sheet (4×4 frames).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn justina_chain_lightning(
    time: Res<Time>,
    mut commands: Commands,
    hag_assets: Res<HagAssets>,
    death_tracker: Res<HagDeathTracker>,
    mut justina_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            &HagIdentity,
            &HagEyeState,
            &mut ChainLightningCooldown,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<MindControlled>,
            Without<Wizard>,
            Without<BanishedModifier>,
        ),
    >,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
    visual_assets: Res<SpellVisualAssets>,
) {
    use crate::game::units::wizard::spells::chain_lightning::{
        components::{ChainLightningBolt, ChainLightningGroup},
        constants as cl_constants, systems as cl_systems,
    };

    let delta = time.delta_secs();
    let enraged = death_tracker.permanent_deaths >= 2;

    for (hag_entity, transform, team, identity, eye_state, mut cooldown) in &mut justina_query {
        if *identity != HagIdentity::Justina || (!eye_state.has_ability_eye && !enraged) {
            continue;
        }

        cooldown.time_remaining -= delta;
        if cooldown.time_remaining > 0.0 {
            continue;
        }
        cooldown.time_remaining = CHAIN_LIGHTNING_COOLDOWN;
        commands
            .entity(hag_entity)
            .insert(hag_casting_animation(&hag_assets));

        let hag_pos = transform.translation;

        // Find nearest enemy within range
        let mut nearest: Option<(Entity, Vec3, f32)> = None;
        for (entity, target_transform, target_team) in &targets {
            if !team.is_enemy(target_team) {
                continue;
            }
            let dx = target_transform.translation.x - hag_pos.x;
            let dz = target_transform.translation.z - hag_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist <= CHAIN_LIGHTNING_RANGE && nearest.as_ref().is_none_or(|n| dist < n.2) {
                nearest = Some((entity, target_transform.translation, dist));
            }
        }

        if let Some((first_target, target_pos, _)) = nearest {
            // Apply initial damage to first target
            if let Ok((mut health, mut temp_hp)) = health_query.get_mut(first_target) {
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), CHAIN_LIGHTNING_DAMAGE);
            }

            // Spawn chain lightning group for hit tracking
            let group_entity = commands
                .spawn((
                    ChainLightningGroup {
                        hit_entities: HashSet::from([first_target]),
                    },
                    OnGameplayScreen,
                ))
                .id();

            // Spawn the bounce bolt (starts from first target, bounces to others)
            commands.spawn((
                ChainLightningBolt {
                    group_entity,
                    current_damage: CHAIN_LIGHTNING_DAMAGE * cl_constants::DAMAGE_FALLOFF,
                    damage_type: crate::game::units::DamageType::Electric,
                    bounces_remaining: cl_constants::MAX_BOUNCES,
                    last_hit_position: target_pos,
                    bounce_delay_timer: cl_constants::BOUNCE_DELAY,
                    empowerment: 1.0,
                    split_depth: 1,
                    split_count: cl_constants::SPLIT_COUNT,
                    damage_falloff: cl_constants::DAMAGE_FALLOFF,
                    static_charge: false,
                    magnetic_pull: false,
                    chain_reaction: false,
                    bounce_range_mult: 1.0,
                },
                OnGameplayScreen,
            ));

            // Spawn the visual arc from Justina to the first target
            cl_systems::spawn_arc(&mut commands, &visual_assets, hag_pos, target_pos, 0, 1.0);
        }
    }
}

/// Justina's fireball — shoots fireballs at random defenders.
#[allow(clippy::too_many_arguments)]
pub fn justina_fireball(
    time: Res<Time>,
    mut commands: Commands,
    hag_assets: Res<HagAssets>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    death_tracker: Res<HagDeathTracker>,
    mut justina_query: Query<
        (
            Entity,
            &HagIdentity,
            &HagEyeState,
            &Transform,
            &mut FireballCooldown,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    defender_teams: Query<
        (&Transform, &Team),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<MindControlled>,
            Without<Wizard>,
            Without<BanishedModifier>,
        ),
    >,
    visual_assets: Res<SpellVisualAssets>,
) {
    let delta = time.delta_secs();

    let enraged = death_tracker.permanent_deaths >= 2;

    for (hag_entity, identity, eye_state, transform, mut cooldown) in &mut justina_query {
        if *identity != HagIdentity::Justina || (!eye_state.has_ability_eye && !enraged) {
            continue;
        }

        cooldown.time_remaining -= delta;
        if cooldown.time_remaining > 0.0 {
            continue;
        }
        cooldown.time_remaining = FIREBALL_COOLDOWN;
        commands
            .entity(hag_entity)
            .insert(hag_casting_animation(&hag_assets));

        // Collect defender positions
        let defender_positions: Vec<Vec3> = defender_teams
            .iter()
            .filter(|(_, team)| **team == Team::Defenders)
            .map(|(t, _)| t.translation)
            .collect();

        if defender_positions.is_empty() {
            continue;
        }

        let hag_pos = transform.translation;

        for _ in 0..FIREBALL_COUNT {
            let idx = game_rng.0.random_range(0..defender_positions.len());
            let target_pos = defender_positions[idx];

            let direction = (target_pos - hag_pos).normalize_or_zero();
            let velocity = direction * FIREBALL_SPEED;
            // Offset spawn past Justina's hitbox so she doesn't hit herself
            let spawn_pos = hag_pos + direction * (HAG_RADIUS + FIREBALL_COLLISION_RADIUS + 5.0);

            crate::game::units::wizard::spells::fireball::systems::spawn_fireball_entity(
                &mut commands,
                &visual_assets,
                spawn_pos,
                velocity,
                FIREBALL_DAMAGE,
                crate::game::units::DamageType::Fire,
                FIREBALL_EXPLOSION_RADIUS,
                FIREBALL_COLLISION_RADIUS,
                0.0,
                FIREBALL_VISUAL_RADIUS,
            );
        }
    }
}

// ===== Phase 4: Josephina Abilities =====

/// Josephina's leap — parabolic arc jump to a random defender, knockback on landing.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn josephina_leap(
    time: Res<Time>,
    mut commands: Commands,
    hag_assets: Res<HagAssets>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    death_tracker: Res<HagDeathTracker>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut josephina_query: Query<
        (
            Entity,
            &mut Transform,
            &Team,
            &HagIdentity,
            &HagEyeState,
            &mut LeapState,
            &mut Velocity,
            &FacingDirection,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    defenders: Query<
        (&Transform, &Team),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<MindControlled>,
            Without<Wizard>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        hag_entity,
        mut transform,
        team,
        identity,
        eye_state,
        mut leap,
        mut velocity,
        facing,
        material_handle,
    ) in &mut josephina_query
    {
        let enraged = death_tracker.permanent_deaths >= 2;
        if *identity != HagIdentity::Josephina || (!eye_state.has_ability_eye && !enraged) {
            // If eye transferred away mid-leap, cancel and land — also reset Y
            // back to the pre-leap height so successive leaps don't stack
            // elevation each time the eye is yanked mid-air.
            if let LeapState::InAir {
                target, start_pos, ..
            } = &*leap
            {
                transform.translation.x = target.x;
                transform.translation.y = start_pos.y;
                transform.translation.z = target.z;
                *leap = LeapState::Idle {
                    cooldown: LEAP_COOLDOWN,
                };
                restore_hag_walking_pose(&mut materials, material_handle, &hag_assets, *facing);
                commands.entity(hag_entity).remove::<AnimationOverride>();
            }
            continue;
        }

        match &mut *leap {
            LeapState::Idle { cooldown } => {
                *cooldown -= delta;
                if *cooldown > 0.0 {
                    continue;
                }

                // Pick a random defender target within leap range
                let josephina_pos = transform.translation;
                let defender_positions: Vec<Vec3> = defenders
                    .iter()
                    .filter(|(_, t)| team.is_enemy(t))
                    .map(|(t, _)| t.translation)
                    .filter(|pos| {
                        let dx = pos.x - josephina_pos.x;
                        let dz = pos.z - josephina_pos.z;
                        (dx * dx + dz * dz).sqrt() <= LEAP_MAX_RANGE
                    })
                    .collect();

                if defender_positions.is_empty() {
                    continue;
                }

                let idx = game_rng.0.random_range(0..defender_positions.len());
                let target = defender_positions[idx];

                *leap = LeapState::InAir {
                    target,
                    start_pos: transform.translation,
                    progress: 0.0,
                };
                // Pin to the second frame (index 1) of the attack sheet for the
                // duration of the leap. AnimationOverride keeps update_walking_animation
                // from clobbering the pose.
                commands.entity(hag_entity).insert(AnimationOverride);
                set_hag_attack_pose_frame(&mut materials, material_handle, &hag_assets, *facing, 1);

                // Zero velocity during leap
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
            LeapState::InAir {
                target,
                start_pos,
                progress,
            } => {
                *progress += delta / LEAP_FLIGHT_DURATION;

                if *progress >= 1.0 {
                    // Land at target, restore Y to pre-leap height
                    transform.translation.x = target.x;
                    transform.translation.y = start_pos.y;
                    transform.translation.z = target.z;
                    // Knockback is applied by josephina_leap_knockback system
                    *leap = LeapState::Landing {
                        timer: 0.3,
                        knockback_applied: false,
                    };
                    // Hold the third frame (index 2) of the attack sheet on landing.
                    set_hag_attack_pose_frame(
                        &mut materials,
                        material_handle,
                        &hag_assets,
                        *facing,
                        2,
                    );
                } else {
                    // Parabolic arc interpolation
                    let t = *progress;
                    let x = start_pos.x + (target.x - start_pos.x) * t;
                    let z = start_pos.z + (target.z - start_pos.z) * t;
                    // Parabolic height: peaks at t=0.5
                    let height_offset = LEAP_MAX_HEIGHT * 4.0 * t * (1.0 - t);
                    transform.translation.x = x;
                    transform.translation.y = start_pos.y + height_offset;
                    transform.translation.z = z;
                }
            }
            LeapState::Landing { timer, .. } => {
                *timer -= delta;
                if *timer <= 0.0 {
                    // After landing, transition to mauling (handled by mauling system)
                    *leap = LeapState::Idle {
                        cooldown: LEAP_COOLDOWN,
                    };
                    restore_hag_walking_pose(&mut materials, material_handle, &hag_assets, *facing);
                    commands.entity(hag_entity).remove::<AnimationOverride>();
                }
            }
        }
    }
}

/// Apply knockback on Josephina's leap landing.
pub fn josephina_leap_knockback(
    mut commands: Commands,
    mut josephina_query: Query<
        (&Transform, &Team, &HagIdentity, &mut LeapState),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<Wizard>,
            Without<BanishedModifier>,
        ),
    >,
) {
    for (transform, team, identity, mut leap) in &mut josephina_query {
        if *identity != HagIdentity::Josephina {
            continue;
        }

        // Only apply knockback once at landing moment
        if let LeapState::Landing {
            knockback_applied, ..
        } = leap.as_mut()
        {
            if *knockback_applied {
                continue;
            }
            *knockback_applied = true;

            let land_pos = transform.translation;
            for (entity, target_transform, target_team) in &targets {
                if !team.is_enemy(target_team) {
                    continue;
                }

                let dx = target_transform.translation.x - land_pos.x;
                let dz = target_transform.translation.z - land_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= LEAP_KNOCKBACK_RADIUS {
                    let direction = if dist > 0.1 {
                        Vec3::new(dx, 0.0, dz)
                    } else {
                        Vec3::X
                    };
                    commands.entity(entity).insert(Knockback::new(
                        direction,
                        LEAP_KNOCKBACK_SPEED,
                        LEAP_KNOCKBACK_DURATION,
                    ));
                }
            }
        }
    }
}

/// Josephina's frenzy — 5x attack speed after leap landing for MAULING_DURATION.
pub fn josephina_vicious_mauling(
    time: Res<Time>,
    death_tracker: Res<HagDeathTracker>,
    mut josephina_query: Query<
        (&HagIdentity, &HagEyeState, &LeapState, &mut MaulingState),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();
    let enraged = death_tracker.permanent_deaths >= 2;

    for (identity, eye_state, leap, mut mauling) in &mut josephina_query {
        if *identity != HagIdentity::Josephina || (!eye_state.has_ability_eye && !enraged) {
            mauling.frenzy_timer = 0.0;
            continue;
        }

        // Activate frenzy on leap landing
        if matches!(leap, LeapState::Landing { .. }) && !mauling.is_frenzied() {
            mauling.frenzy_timer = MAULING_DURATION;
        }

        // Tick down frenzy timer
        if mauling.is_frenzied() {
            mauling.frenzy_timer = (mauling.frenzy_timer - delta).max(0.0);
        }
    }
}

/// Josephina's corpse consume — stationary for 3s near a corpse, heals, despawns corpse.
#[allow(clippy::type_complexity)]
pub fn josephina_corpse_consume(
    time: Res<Time>,
    mut commands: Commands,
    death_tracker: Res<HagDeathTracker>,
    mut josephina_query: Query<
        (
            Entity,
            &Transform,
            &HagIdentity,
            &HagEyeState,
            &mut Health,
            Option<&mut CorpseConsumeState>,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    corpses: Query<(Entity, &Transform), With<Corpse>>,
) {
    let delta = time.delta_secs();
    let enraged = death_tracker.permanent_deaths >= 2;

    for (entity, transform, identity, eye_state, mut health, consume_state) in &mut josephina_query
    {
        if *identity != HagIdentity::Josephina || (!eye_state.has_ability_eye && !enraged) {
            // Cancel consume if eye lost
            if consume_state.is_some() {
                commands.entity(entity).remove::<CorpseConsumeState>();
            }
            continue;
        }

        if let Some(mut state) = consume_state {
            // Currently consuming a corpse
            state.timer -= delta;
            if state.timer <= 0.0 {
                // Heal and despawn corpse
                let heal = health.max * CORPSE_CONSUME_HEAL_PERCENT;
                health.current = (health.current + heal).min(health.max);
                commands.entity(state.corpse_entity).try_despawn();
                commands.entity(entity).remove::<CorpseConsumeState>();
            }
        } else {
            // Check if there's a nearby corpse to consume (only if health not full)
            if health.current >= health.max * CORPSE_CONSUME_HEALTH_THRESHOLD {
                continue;
            }

            let hag_pos = transform.translation;
            let mut nearest_corpse: Option<(Entity, f32)> = None;

            for (corpse_entity, corpse_transform) in &corpses {
                let dx = corpse_transform.translation.x - hag_pos.x;
                let dz = corpse_transform.translation.z - hag_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist < CORPSE_CONSUME_RANGE {
                    if let Some((_, best_dist)) = nearest_corpse {
                        if dist < best_dist {
                            nearest_corpse = Some((corpse_entity, dist));
                        }
                    } else {
                        nearest_corpse = Some((corpse_entity, dist));
                    }
                }
            }

            if let Some((corpse_entity, _)) = nearest_corpse {
                commands.entity(entity).insert(CorpseConsumeState {
                    timer: CORPSE_CONSUME_DURATION,
                    corpse_entity,
                });
            }
        }
    }
}

// ===== Phase 5: Martina Abilities =====

/// Martina's teleport pull — teleports random defenders to her position.
/// King and guards move as a group.
#[allow(clippy::type_complexity)]
pub fn martina_teleport_pull(
    time: Res<Time>,
    mut commands: Commands,
    hag_assets: Res<HagAssets>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    death_tracker: Res<HagDeathTracker>,
    mut martina_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            &HagIdentity,
            &HagEyeState,
            &mut TeleportPullCooldown,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    mut defenders: Query<
        (
            Entity,
            &mut Transform,
            &Team,
            Option<&King>,
            Option<&KingsGuard>,
        ),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<MindControlled>,
            Without<Wizard>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let delta = time.delta_secs();

    let enraged = death_tracker.permanent_deaths >= 2;

    // Single-pass: find the king and categorize defenders simultaneously.
    let mut king_pos: Option<Vec3> = None;
    let mut king_entity: Option<Entity> = None;
    let mut regular_defenders: Vec<Entity> = Vec::new();
    let mut guard_entities: Vec<Entity> = Vec::new();
    for (entity, def_transform, team, king, guard) in &defenders {
        if *team != Team::Defenders {
            continue;
        }
        if king.is_some() {
            king_entity = Some(entity);
            king_pos = Some(def_transform.translation);
        } else if guard.is_some() {
            guard_entities.push(entity);
        } else {
            regular_defenders.push(entity);
        }
    }

    for (hag_entity, transform, _team, identity, eye_state, mut cooldown) in &mut martina_query {
        if *identity != HagIdentity::Martina || (!eye_state.has_ability_eye && !enraged) {
            continue;
        }

        cooldown.time_remaining -= delta;
        if cooldown.time_remaining > 0.0 {
            continue;
        }

        // Only cast when Martina is within `TELEPORT_PULL_KING_RANGE` of the king.
        let Some(king_pos) = king_pos else {
            continue;
        };
        let dx = transform.translation.x - king_pos.x;
        let dz = transform.translation.z - king_pos.z;
        let king_range_sq = TELEPORT_PULL_KING_RANGE * TELEPORT_PULL_KING_RANGE;
        if dx * dx + dz * dz > king_range_sq {
            continue;
        }

        cooldown.time_remaining = TELEPORT_PULL_COOLDOWN;
        commands
            .entity(hag_entity)
            .insert(hag_casting_animation(&hag_assets));

        let pull_pos = transform.translation;
        // Local mutable copy so multiple Martina instances each consume targets.
        let mut regular_defenders = regular_defenders.clone();

        let mut pulled = 0u32;

        // Pull random regular defenders first
        while pulled < TELEPORT_PULL_COUNT && !regular_defenders.is_empty() {
            let idx = game_rng.0.random_range(0..regular_defenders.len());
            let entity = regular_defenders.swap_remove(idx);

            if let Ok((_, mut def_transform, _, _, _)) = defenders.get_mut(entity) {
                def_transform.translation.x = pull_pos.x + game_rng.0.random_range(-20.0..20.0);
                def_transform.translation.z = pull_pos.z + game_rng.0.random_range(-20.0..20.0);
            }
            pulled += 1;
        }

        // If we haven't pulled enough and king is available, pull king + all guards as a group
        if pulled < TELEPORT_PULL_COUNT
            && let Some(king_e) = king_entity
            && let Ok((_, mut king_transform, _, _, _)) = defenders.get_mut(king_e)
        {
            king_transform.translation.x = pull_pos.x + game_rng.0.random_range(-20.0..20.0);
            king_transform.translation.z = pull_pos.z + game_rng.0.random_range(-20.0..20.0);
            // Guards will snap to king via their existing system
        }
    }
}

/// Martina's mind control aura — instantly mind controls any defender inside the radius.
pub fn martina_mind_control(
    mut commands: Commands,
    martina_query: Query<
        (&Transform, &HagIdentity),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    defenders: Query<MindControlTargetData, MindControlTargetFilter>,
    existing_controlled: Query<&MindControlled, Without<Corpse>>,
) {
    let controlled_count = existing_controlled.iter().count() as u32;
    if controlled_count >= MIND_CONTROL_MAX_CONTROLLED {
        return;
    }

    for (transform, identity) in &martina_query {
        if *identity != HagIdentity::Martina {
            continue;
        }

        let hag_pos = transform.translation;

        for (entity, def_transform, def_team, flow_influence) in &defenders {
            if *def_team != Team::Defenders {
                continue;
            }

            let dx = def_transform.translation.x - hag_pos.x;
            let dz = def_transform.translation.z - hag_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist <= MIND_CONTROL_AURA_RADIUS {
                let original_spawn_pos = match flow_influence {
                    FlowFieldInfluence::Defender { spawn_pos } => Some(*spawn_pos),
                    _ => None,
                };

                commands.entity(entity).insert((
                    MindControlled {
                        time_elapsed: 0.0,
                        wear_off_duration: 300.0, // 5 minutes
                        original_spawn_pos,
                        damage_multiplier: 1.0,
                    },
                    FlowFieldInfluence::Attacker,
                ));
            }
        }
    }
}

/// Updates mind-controlled units — they target non-MC same-team allies.
pub fn update_mind_controlled_targeting(
    mut controlled: Query<(
        Entity,
        &Transform,
        &Team,
        &mut TargetingVelocity,
        &MindControlled,
    )>,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<MindControlled>,
            Without<BanishedModifier>,
        ),
    >,
) {
    for (entity, transform, team, mut targeting, _mc) in &mut controlled {
        // Find nearest ALLY to attack (reversed targeting)
        let pos = transform.translation;
        let mut nearest: Option<(f32, Vec3)> = None;

        for (other_entity, other_transform, other_team) in &all_units {
            if other_entity == entity {
                continue;
            }
            // Target same team (allies become enemies)
            if other_team != team {
                continue;
            }

            let dx = other_transform.translation.x - pos.x;
            let dz = other_transform.translation.z - pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if let Some((best_dist, _)) = nearest {
                if dist < best_dist {
                    nearest = Some((dist, other_transform.translation));
                }
            } else {
                nearest = Some((dist, other_transform.translation));
            }
        }

        if let Some((_, target_pos)) = nearest {
            let dir =
                Vec3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z).normalize_or_zero();
            targeting.velocity = dir;
        } else {
            targeting.velocity = Vec3::ZERO;
        }
    }
}

/// Mind-controlled units charge their nearest former ally so they close to melee
/// range (where `mind_controlled_combat` lands hits) instead of drifting with the
/// herd toward the enemy base. Runs AFTER `MovementCalculationSet` to override the
/// blended steering — the weighted blend keys off flow-field distance, so without
/// this override an MC'd attacker just keeps marching at the castle and never
/// engages its former allies. Mirrors the Pig Form / Dire Sheep velocity overrides.
pub fn mind_controlled_pursue_allies(
    controlled: Query<(Entity, &Transform, &Team, &MovementSpeed), With<MindControlled>>,
    allies: Query<(Entity, &Transform, &Team), (Without<Corpse>, Without<MindControlled>)>,
    mut velocity_query: Query<&mut Velocity>,
) {
    for (entity, transform, team, speed) in &controlled {
        let pos = transform.translation;

        // Nearest same-team unit = the former ally we now turn on.
        let mut nearest: Option<(f32, Vec3)> = None;
        for (other, other_transform, other_team) in &allies {
            if other == entity || other_team != team {
                continue;
            }
            let dist = pos.distance(other_transform.translation);
            if nearest.as_ref().is_none_or(|(best, _)| dist < *best) {
                nearest = Some((dist, other_transform.translation));
            }
        }

        if let Some((_, target_pos)) = nearest
            && let Ok(mut velocity) = velocity_query.get_mut(entity)
        {
            let dir = (target_pos - pos).normalize_or_zero();
            velocity.x = dir.x * speed.0;
            velocity.z = dir.z * speed.0;
        }
    }
}

/// Updates mind control wear-off timer — removes when duration expires.
/// Also cleans up RetaliationTarget components that point at freed entities.
/// Handles talent on-expiry effects: Amnesia (confused state) and Sleeper Agent (delayed betrayal).
pub fn update_mind_control_wear_off(
    time: Res<Time>,
    mut commands: Commands,
    mut controlled: Query<(
        Entity,
        &mut MindControlled,
        Has<crate::game::units::wizard::spells::mind_control::components::AmnesiaOnExpiry>,
        Has<crate::game::units::wizard::spells::mind_control::components::SleeperAgentPending>,
    )>,
    retaliators: Query<(Entity, &RetaliationTarget)>,
) {
    use crate::game::units::wizard::spells::mind_control::components::{
        AmnesiaEffect, SleeperAgentActive,
    };
    use crate::game::units::wizard::spells::mind_control::constants;

    let delta = time.delta_secs();

    for (entity, mut mc, has_amnesia, has_sleeper) in &mut controlled {
        mc.time_elapsed += delta;

        if mc.time_elapsed >= mc.wear_off_duration {
            // Restore original flow field influence before removing mind control
            if let Some(spawn_pos) = mc.original_spawn_pos {
                commands
                    .entity(entity)
                    .insert(FlowFieldInfluence::Defender { spawn_pos });
            }

            commands.entity(entity).remove::<MindControlled>();

            // Clean up talent marker components
            crate::game::units::wizard::spells::mind_control::systems::strip_mind_control_talent_components(
                &mut commands, entity,
            );

            // Amnesia: apply confused state on expiry
            if has_amnesia {
                commands.entity(entity).insert(AmnesiaEffect {
                    time_remaining: constants::AMNESIA_DURATION,
                });
            }

            // Sleeper Agent: start delayed betrayal timer
            if has_sleeper {
                commands.entity(entity).insert(SleeperAgentActive {
                    delay_remaining: constants::SLEEPER_AGENT_DELAY,
                    damage_multiplier: constants::SLEEPER_AGENT_DAMAGE_MULT,
                });
            }

            // Remove RetaliationTarget from any units retaliating against this entity
            for (retaliator_entity, retaliation) in &retaliators {
                if retaliation.0 == entity {
                    commands
                        .entity(retaliator_entity)
                        .remove::<RetaliationTarget>();
                }
            }
        }
    }
}

/// Mind-controlled units attack their own team (gated by global attack cycle).
/// They skip other mind-controlled units and only attack non-MC allies.
pub fn mind_controlled_combat(
    attack_cycle: Res<crate::game::plugin::GlobalAttackCycle>,
    mut commands: Commands,
    mut controlled: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut AttackTiming,
            &MindControlled,
        ),
        Without<Corpse>,
    >,
    mut potential_targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<Corpse>,
            Without<MindControlled>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let current_time = attack_cycle.current_time;
    // `previous_time` is the cycle position before the most recent `tick()` —
    // see `GlobalAttackCycle` doc for why this is preferred over the
    // `current_time - APPROX_FRAME_TIME` approximation.
    let last_time = attack_cycle.previous_time;

    for (mc_entity, mc_transform, mc_hitbox, mc_team, mut timing, mc) in &mut controlled {
        if !timing.can_attack(current_time, last_time) {
            continue;
        }

        let mc_pos = mc_transform.translation;

        // Find nearest same-team unit to attack (mind controlled = attacks allies)
        for (entity, target_transform, target_hitbox, target_team, mut health, mut temp_hp) in
            &mut potential_targets
        {
            if entity == mc_entity {
                continue;
            }
            // Attack same team (reversed)
            if target_team != mc_team {
                continue;
            }

            let dx = target_transform.translation.x - mc_pos.x;
            let dz = target_transform.translation.z - mc_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            let attack_range = (mc_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;

            if dist <= attack_range {
                let damage = MIND_CONTROL_COMBAT_DAMAGE * mc.damage_multiplier;
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
                timing.last_attack_time = Some(current_time);

                // Victim retaliates — consider the MC attacker a valid target
                commands.entity(entity).insert(RetaliationTarget(mc_entity));

                break; // One attack per cycle
            }
        }
    }
}

/// Cleans up RetaliationTarget when the target entity is dead or no longer mind-controlled.
pub fn cleanup_retaliation_targets(
    mut commands: Commands,
    retaliators: Query<(Entity, &RetaliationTarget)>,
    mc_units: Query<Entity, With<MindControlled>>,
) {
    for (entity, retaliation) in &retaliators {
        // Remove if the retaliation target is no longer mind-controlled (or despawned)
        if mc_units.get(retaliation.0).is_err() {
            commands.entity(entity).remove::<RetaliationTarget>();
        }
    }
}
