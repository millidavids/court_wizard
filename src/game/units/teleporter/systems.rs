use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker, WaveGroup};
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::brute::components::Brute;
use crate::game::units::commander::components::Commander;
use crate::game::units::components::{
    BanishedModifier, CombatAnimation, CommanderAuraSpeedModifier, Corpse, Effectiveness,
    EliteSpeedBonus, FacingDirection, FlockingModifier, FlockingVelocity, FrozenSolidModifier,
    HasteModifier, Health, Hitbox, InMelee, MovementSpeed, PolymorphedModifier, RootedModifier,
    RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking, SlowMovementModifier,
    Stunned, TargetingVelocity, Team, Teleportable, TemporaryHitPoints, UnitTypeGlow,
    WalkingAnimation,
};
use crate::game::units::ranged_bolt::RangedAttackTimer;
use crate::game::units::infantry::Infantry;
use crate::game::units::king::components::King;
use crate::game::units::random_position_in_cell;
use crate::game::units::wizard::spells::teleport::vfx_components::TeleportWarpEffect;
use crate::game::units::wizard::spells::teleport::vfx_systems::spawn_teleport_vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use super::resources::TeleporterAssets;

/// Spawns a single teleporter attacker.
pub(in crate::game) fn spawn_single_teleporter(
    rng: &mut impl Rng,
    commands: &mut Commands,
    teleporter_assets: &TeleporterAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(TELEPORTER_RADIUS, TELEPORTER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * TELEPORTER_MOVEMENT_SPEED;

    let anim = WalkingAnimation::new_staggered(rng);
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        teleporter_assets.sprite_texture.clone(),
        TELEPORTER_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(teleporter_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, spawn_y, final_z)
                .with_scale(Vec3::splat(TELEPORTER_SCALE)),
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
            },
            Acceleration::new(),
            hitbox,
            Health::new(TELEPORTER_HEALTH),
            MovementSpeed(TELEPORTER_MOVEMENT_SPEED),
            Effectiveness::new(),
            Team::Attackers,
            Teleporter,
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
        ))
        .insert((
            TeleporterState::default(),
            RangedAttackTimer::new(),
            anim,
            FacingDirection::default(),
            FlockingVelocity::default(),
            FlockingModifier::new(1.0, 1.0, 1.0),
            CommanderAuraSpeedModifier(0.0),
            RoughTerrainModifier(0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
            UnitTypeGlow {
                color: TELEPORTER_GLOW_COLOR,
            },
        ));
}

/// Targeting: beeline toward the king. Never engages — the teleporter has no
/// `AttackTiming`, so the combat system ignores her.
pub(super) fn update_teleporter_targeting(
    mut commands: Commands,
    mut teleporters: Query<
        (Entity, &Transform, &TeleporterState, &mut TargetingVelocity),
        (With<Teleporter>, Without<Corpse>),
    >,
    king_query: Query<(&Transform, &Team), (With<King>, Without<Corpse>)>,
) {
    let Some((king_transform, _)) = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
    else {
        return;
    };
    let king_pos = king_transform.translation;

    for (entity, transform, state, mut targeting) in &mut teleporters {
        if matches!(state, TeleporterState::Channeling { .. }) {
            targeting.velocity = Vec3::ZERO;
            targeting.distance_to_target = 0.0;
            commands.entity(entity).remove::<InMelee>();
            continue;
        }

        let to_king = Vec3::new(
            king_pos.x - transform.translation.x,
            0.0,
            king_pos.z - transform.translation.z,
        );
        targeting.velocity = to_king.normalize_or_zero();
        targeting.distance_to_target = to_king.length();
        commands.entity(entity).remove::<InMelee>();
    }
}

/// Movement: reuses shared weighted movement. Stops when channeling.
#[allow(clippy::type_complexity)]
pub(super) fn teleporter_movement(
    time: Res<Time>,
    mut teleporters: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &TeleporterState,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            Option<&SlowMovementModifier>,
            (
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&Stunned>,
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        (With<Teleporter>, Without<Corpse>),
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting,
        flocking,
        flow_field,
        state,
        aura_mod,
        terrain_mod,
        slow_mod,
        (rooted, haste, elite_speed, sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut teleporters
    {
        if matches!(state, TeleporterState::Channeling { .. }) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        if polymorphed.is_some() {
            continue;
        }

        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            targeting,
            flocking,
            flow_field,
            false,
            aura_mod.map(|m| m.0),
            terrain_mod.map(|m| m.0),
            slow_mod.map(|m| m.modifier),
            None,
            haste.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );
    }
}

/// Transitions teleporters from Approaching → Channeling when in range of the king,
/// ticks the channel, and fires the teleport on completion.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(super) fn update_channel_state(
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

                let push_sorted = |mut candidates: Vec<(Entity, f32, f32)>,
                                   picks: &mut Vec<(Entity, f32)>| {
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
                    let angle = game_rng.0.gen_range(0.0..std::f32::consts::TAU);
                    let r = game_rng.0.gen_range(0.0..DROP_JITTER_RADIUS);
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
pub(super) fn refresh_teleporter_casting_animation(
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
pub(super) fn spawn_channel_particles(
    mut commands: Commands,
    teleporters: Query<(&Transform, &TeleporterState, &Hitbox), (With<Teleporter>, Without<Corpse>)>,
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
pub(super) fn cleanup_dead_teleporter_channels(
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

#[allow(clippy::type_complexity)]
pub(super) fn teleporter_ranged_combat(
    mut commands: Commands,
    time: Res<Time>,
    teleporter_assets: Res<TeleporterAssets>,
    mut teleporters: Query<
        (
            Entity,
            &Transform,
            &Team,
            &TeleporterState,
            &mut RangedAttackTimer,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Teleporter>, Without<Corpse>),
    >,
    targets: Query<(Entity, &Transform, &Team), (Without<Corpse>, Without<BanishedModifier>)>,
) {
    let delta = time.delta_secs();

    for (
        teleporter_entity,
        teleporter_transform,
        teleporter_team,
        state,
        mut attack_timer,
        sleeping,
        banished,
        has_staging,
        has_wave_group,
    ) in &mut teleporters
    {
        if crate::game::units::systems::is_staging_attacker(
            teleporter_team,
            has_staging,
            has_wave_group,
        ) {
            continue;
        }
        attack_timer.time_since_last_attack += delta;

        if matches!(state, TeleporterState::Channeling { .. })
            || sleeping.is_some()
            || banished.is_some()
        {
            continue;
        }

        if attack_timer.time_since_last_attack < TELEPORTER_ATTACK_COOLDOWN {
            continue;
        }

        let nearest_enemy = targets
            .iter()
            .filter(|(entity, _, team)| {
                *entity != teleporter_entity && teleporter_team.is_enemy(team)
            })
            .filter(|(_, transform, _)| {
                teleporter_transform
                    .translation
                    .distance(transform.translation)
                    <= TELEPORTER_ATTACK_RANGE
            })
            .min_by(|a, b| {
                let dist_a = teleporter_transform.translation.distance(a.1.translation);
                let dist_b = teleporter_transform.translation.distance(b.1.translation);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((_, target_transform, _)) = nearest_enemy {
            let origin = teleporter_transform.translation + Vec3::Y * 10.0;
            let diff = target_transform.translation - origin;
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();

            crate::game::units::ranged_bolt::spawn_magic_bolt(
                &mut commands,
                &teleporter_assets.bolt_mesh,
                &teleporter_assets.bolt_material,
                origin,
                direction,
                TELEPORTER_BOLT_SPEED,
                TELEPORTER_BOLT_DAMAGE,
                TELEPORTER_BOLT_RADIUS,
                TELEPORTER_BOLT_LIFETIME,
                *teleporter_team,
            );

            commands
                .entity(teleporter_entity)
                .insert(CombatAnimation::new_casting(
                    teleporter_assets.casting_texture.clone(),
                    teleporter_assets.sprite_texture.clone(),
                ));

            attack_timer.time_since_last_attack = 0.0;
        }
    }
}
