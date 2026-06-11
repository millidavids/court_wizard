use std::collections::HashSet;

use crate::game::units::wizard::components::Wizard;
use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::core::hag_casting_animation;
use super::super::resources::{HagAssets, HagDeathTracker};
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{
    BanishedModifier, Corpse, Health, MindControlled, Team, TemporaryHitPoints,
    apply_damage_to_unit,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Justina's chain lightning — bounces between enemies.
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
            Without<crate::game::multiplayer::components::GhostEntity>,
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
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
    mut health_query: Query<
        (&mut Health, Option<&mut TemporaryHitPoints>),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
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
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
            Without<crate::game::multiplayer::components::GhostEntity>,
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
            Without<crate::game::multiplayer::components::GhostEntity>,
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
