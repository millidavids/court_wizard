use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::core::hag_casting_animation;
use super::super::resources::{HagAssets, HagDeathTracker};
use crate::game::pathfinding::{FlowFieldInfluence, StagingAttacker};
use crate::game::units::components::{BanishedModifier, Corpse, KingsGuard, MindControlled, Team};
use crate::game::units::king::components::King;
use crate::game::units::wizard::components::Wizard;

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
