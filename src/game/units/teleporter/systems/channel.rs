use bevy::prelude::*;
use rand::Rng;

use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::brute::components::Brute;
use crate::game::units::commander::components::Commander;
use crate::game::units::components::{
    CombatAnimation, Corpse, Health, Hitbox, Team, TemporaryHitPoints,
};
use crate::game::units::infantry::Infantry;
use crate::game::units::king::components::King;
use crate::game::units::teleporter::components::{Teleporter, TeleporterState};
use crate::game::units::teleporter::constants::*;
use crate::game::units::teleporter::resources::TeleporterAssets;
use crate::game::units::wizard::spells::teleport::vfx_components::TeleportWarpEffect;
use crate::game::units::wizard::spells::teleport::vfx_systems::spawn_teleport_vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Transitions teleporters from Approaching → Channeling when in range of the king,
/// ticks the channel, and fires the teleport on completion.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(crate) fn update_channel_state(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<GameRng>,
    teleporter_assets: Res<TeleporterAssets>,
    mut teleporters: Query<
        (
            Entity,
            &Transform,
            &mut TeleporterState,
            &Team,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Teleporter>, Without<Corpse>),
    >,
    king_query: Query<(&Transform, &Team), (With<King>, Without<Corpse>)>,
    mut infantry_allies: Query<
        (Entity, &mut Transform, &Team, &Health),
        (
            With<Infantry>,
            Without<Teleporter>,
            Without<King>,
            Without<Corpse>,
            Without<StagingAttacker>,
        ),
    >,
    mut brute_allies: Query<
        (Entity, &mut Transform, &Team, &Health),
        (
            With<Brute>,
            Without<Teleporter>,
            Without<King>,
            Without<Corpse>,
            Without<Infantry>,
            Without<StagingAttacker>,
        ),
    >,
    mut commander_allies: Query<
        (Entity, &mut Transform, &Team, &Health),
        (
            With<Commander>,
            Without<Teleporter>,
            Without<King>,
            Without<Corpse>,
            Without<Infantry>,
            Without<Brute>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let Some((king_transform, _)) = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
    else {
        return;
    };
    let king_pos = king_transform.translation;

    let dt = time.delta_secs();

    for (entity, transform, mut state, team, has_staging, has_wave_group) in &mut teleporters {
        let pos = transform.translation;
        let dx = pos.x - king_pos.x;
        let dz = pos.z - king_pos.z;
        let dist = (dx * dx + dz * dz).sqrt();

        let is_staging =
            crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group);

        match &mut *state {
            TeleporterState::Approaching => {
                if is_staging {
                    continue;
                }
                if dist <= CHANNEL_RANGE {
                    let indicator = commands
                        .spawn((
                            TeleportWarpEffect {
                                position: pos,
                                radius: TELEPORT_VFX_RADIUS,
                                time_alive: 0.0,
                                duration: CHANNEL_DURATION,
                                intensity: 0.8,
                                rift_entity: None,
                            },
                            OnGameplayScreen,
                        ))
                        .id();
                    commands.entity(entity).insert(CombatAnimation::new_casting(
                        teleporter_assets.casting_texture.clone(),
                        teleporter_assets.sprite_texture.clone(),
                    ));
                    *state = TeleporterState::Channeling {
                        elapsed: 0.0,
                        indicator,
                    };
                }
            }
            TeleporterState::Cooldown { remaining } => {
                *remaining -= dt;
                if *remaining <= 0.0 {
                    *state = TeleporterState::Approaching;
                }
            }
            TeleporterState::Channeling { elapsed, indicator } => {
                *elapsed += dt;
                if *elapsed < CHANNEL_DURATION {
                    continue;
                }
                let indicator_entity = *indicator;

                // Collect candidates in priority order: infantry → brute → commander.
                // Each candidate is (entity, distance_sq_to_teleporter, max_hp).
                let mut picks: Vec<(Entity, f32)> = Vec::with_capacity(TELEPORT_GRAB_COUNT);

                let push_sorted =
                    |mut candidates: Vec<(Entity, f32, f32)>, picks: &mut Vec<(Entity, f32)>| {
                        if picks.len() >= TELEPORT_GRAB_COUNT {
                            return;
                        }
                        candidates.sort_by(|a, b| {
                            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        for (e, _, max_hp) in candidates {
                            if picks.len() >= TELEPORT_GRAB_COUNT {
                                break;
                            }
                            picks.push((e, max_hp));
                        }
                    };

                let infantry_candidates: Vec<(Entity, f32, f32)> = infantry_allies
                    .iter()
                    .filter(|(_, _, team, _)| **team == Team::Attackers)
                    .map(|(e, t, _, h)| {
                        let ddx = t.translation.x - pos.x;
                        let ddz = t.translation.z - pos.z;
                        (e, ddx * ddx + ddz * ddz, h.max)
                    })
                    .collect();
                push_sorted(infantry_candidates, &mut picks);

                let brute_candidates: Vec<(Entity, f32, f32)> = brute_allies
                    .iter()
                    .filter(|(_, _, team, _)| **team == Team::Attackers)
                    .map(|(e, t, _, h)| {
                        let ddx = t.translation.x - pos.x;
                        let ddz = t.translation.z - pos.z;
                        (e, ddx * ddx + ddz * ddz, h.max)
                    })
                    .collect();
                push_sorted(brute_candidates, &mut picks);

                let commander_candidates: Vec<(Entity, f32, f32)> = commander_allies
                    .iter()
                    .filter(|(_, _, team, _)| **team == Team::Attackers)
                    .map(|(e, t, _, h)| {
                        let ddx = t.translation.x - pos.x;
                        let ddz = t.translation.z - pos.z;
                        (e, ddx * ddx + ddz * ddz, h.max)
                    })
                    .collect();
                push_sorted(commander_candidates, &mut picks);

                // Move each picked entity onto the king (preserving y+scale) and grant temp HP.
                for (ally, max_hp) in &picks {
                    let angle = game_rng.0.random_range(0.0..std::f32::consts::TAU);
                    let r = game_rng.0.random_range(0.0..DROP_JITTER_RADIUS);
                    let new_x = king_pos.x + angle.cos() * r;
                    let new_z = king_pos.z + angle.sin() * r;

                    if let Ok((_, mut t, _, _)) = infantry_allies.get_mut(*ally) {
                        t.translation.x = new_x;
                        t.translation.z = new_z;
                    } else if let Ok((_, mut t, _, _)) = brute_allies.get_mut(*ally) {
                        t.translation.x = new_x;
                        t.translation.z = new_z;
                    } else if let Ok((_, mut t, _, _)) = commander_allies.get_mut(*ally) {
                        t.translation.x = new_x;
                        t.translation.z = new_z;
                    }

                    commands.entity(*ally).insert(TemporaryHitPoints::new(
                        max_hp * TELEPORT_TEMP_HP_RATIO,
                        TELEPORT_TEMP_HP_DURATION,
                    ));
                }

                // VFX at both endpoints.
                spawn_teleport_vfx(&mut commands, pos, king_pos, TELEPORT_VFX_RADIUS);

                // Despawn indicator and enter cooldown. Let the current casting
                // animation cycle finish naturally so `update_combat_animation`
                // restores the walking texture; `refresh_teleporter_casting_animation`
                // won't re-insert it once state is Cooldown.
                commands.entity(indicator_entity).try_despawn();
                *state = TeleporterState::Cooldown {
                    remaining: CHANNEL_COOLDOWN,
                };
            }
        }
    }
}

/// Re-inserts `CombatAnimation::new_casting` on channeling teleporters that
/// have finished (and thus had their animation removed) so the cast animation
/// loops for the full channel duration.
pub(crate) fn refresh_teleporter_casting_animation(
    mut commands: Commands,
    teleporters: Query<
        (Entity, &TeleporterState),
        (With<Teleporter>, Without<CombatAnimation>, Without<Corpse>),
    >,
    teleporter_assets: Res<TeleporterAssets>,
) {
    for (entity, state) in &teleporters {
        if matches!(state, TeleporterState::Channeling { .. }) {
            commands.entity(entity).insert(CombatAnimation::new_casting(
                teleporter_assets.casting_texture.clone(),
                teleporter_assets.sprite_texture.clone(),
            ));
        }
    }
}

/// Spawns inward-imploding particles on the surface of a growing sphere around
/// each channeling teleporter.
pub(crate) fn spawn_channel_particles(
    mut commands: Commands,
    teleporters: Query<
        (&Transform, &TeleporterState, &Hitbox),
        (With<Teleporter>, Without<Corpse>),
    >,
    teleporter_assets: Res<TeleporterAssets>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut game_rng: ResMut<GameRng>,
) {
    *timer += time.delta_secs();
    if *timer < CHANNEL_PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *timer -= CHANNEL_PARTICLE_SPAWN_INTERVAL;

    let spec = crate::game::units::wizard::spells::vfx::channel::ChannelParticleSpec {
        start_radius: CHANNEL_PARTICLE_START_RADIUS,
        max_radius: CHANNEL_PARTICLE_MAX_RADIUS,
        size: CHANNEL_PARTICLE_SIZE,
        lifetime: CHANNEL_PARTICLE_LIFETIME,
        count_per_spawn: CHANNEL_PARTICLE_COUNT_PER_SPAWN,
    };

    for (transform, state, hitbox) in &teleporters {
        let TeleporterState::Channeling { elapsed, .. } = state else {
            continue;
        };
        let progress = *elapsed / CHANNEL_DURATION;
        let center = transform.translation + Vec3::Y * (hitbox.height * 0.5);
        crate::game::units::wizard::spells::vfx::channel::spawn_channel_particle_batch(
            &mut commands,
            center,
            progress,
            &visual_assets.particle_quad,
            &teleporter_assets.channel_particle_material,
            &spec,
            &mut game_rng.0,
        );
    }
}

/// Cleans up channel indicators when a teleporter dies mid-channel.
pub(crate) fn cleanup_dead_teleporter_channels(
    mut commands: Commands,
    dead: Query<(Entity, &TeleporterState), (With<Teleporter>, With<Corpse>)>,
) {
    for (entity, state) in &dead {
        if let TeleporterState::Channeling { indicator, .. } = state {
            commands.entity(*indicator).try_despawn();
        }
        commands.entity(entity).remove::<TeleporterState>();
    }
}
