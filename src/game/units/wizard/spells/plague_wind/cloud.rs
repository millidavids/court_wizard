//! Plague wind cloud: spawn, movement, damage application, cleanup.

use super::components::{
    InsidePlagueCloud, PandemicProcessed, PlagueCarrierDoT, PlagueWindCloud,
    PlagueWindTalentParams, ToxicWeaknessDebuff,
};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, SlowMovementModifier, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::utils::{UniqueHitTracker, local_player_team};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Computes talent parameters from the player's active talent selections.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_plague_cloud(
    commands: &mut Commands,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    pos: Vec3,
    radius: f32,
    damage: f32,
    duration: f32,
    speed: f32,
    direction: Vec3,
    talent_params: PlagueWindTalentParams,
) {
    // Notify pathfinding
    let origin_2d = Vec2::new(pos.x, pos.z);
    let buffered = radius + OBSTACLE_BUFFER;
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
        obstacle_type: ObstacleType::Hazard(10.0),
        shape: Some(ObstacleShape::circle(origin_2d, buffered)),
        rebuild: false,
    });

    commands.spawn((
        Transform::from_translation(Vec3::new(pos.x, 0.0, pos.z)),
        PlagueWindCloud::new(
            pos,
            radius,
            damage,
            constants::TICK_INTERVAL,
            duration,
            speed,
            direction,
            talent_params,
        ),
        UniqueHitTracker::default(),
        NetworkedSpellEffect {
            kind: SpellEffectKind::PlagueWindCloud,
        },
        OnGameplayScreen,
    ));
}

/// Moves the plague wind cloud in its drift direction and updates pathfinding.
pub fn move_plague_wind_cloud(
    time: Res<Time>,
    // Host-only — the guest mirrors cloud position via the snapshot, so the
    // ghost cloud must NOT independently drift (would diverge from host AND
    // double-update the pathfinding grid).
    mut clouds: Query<
        (&mut PlagueWindCloud, &mut Transform),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    for (mut cloud, mut transform) in clouds.iter_mut() {
        // Remove old pathfinding bounds
        let old_origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
        let buffered = cloud.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(old_origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(old_origin_2d, buffered)),
            rebuild: false,
        });

        // Move cloud
        let movement = cloud.direction * cloud.speed * delta;
        cloud.origin += movement;
        transform.translation.x = cloud.origin.x;
        transform.translation.z = cloud.origin.z;

        // Add new pathfinding bounds
        let new_origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(new_origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Hazard(10.0),
            shape: Some(ObstacleShape::circle(new_origin_2d, buffered)),
            rebuild: false,
        });
    }
}

/// Returns horizontal (XZ-plane) distance between two 3D positions.
fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

/// Applies periodic necrotic damage to all units within the cloud.
/// Handles Toxic Weakness (vulnerability), Choking Gas (slow), Necrotic Rot (max HP reduction),
/// and tracks units inside cloud for Plague Carrier.
pub fn apply_plague_wind_damage(
    mut commands: Commands,
    time: Res<Time>,
    // Host-only — ghost cloud on the guest must NOT also apply DPS, or every
    // tick deals damage on both peers and CRDT max-merge masks the doubling
    // but talent status effects (Toxic Weakness, Choking Gas) get applied
    // twice locally.
    mut clouds: Query<
        (&mut PlagueWindCloud, &mut UniqueHitTracker),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut units: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        Has<InsidePlagueCloud>,
        &Team,
    )>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();
    let mut unique_hits: u32 = 0;

    for (mut cloud, mut hit_tracker) in &mut clouds {
        cloud.time_alive += delta;
        cloud.time_since_last_tick += delta;

        let has_plague_carrier = cloud.talent_params.plague_carrier;
        let has_toxic_weakness = cloud.talent_params.toxic_weakness;
        let has_choking_gas = cloud.talent_params.choking_gas;
        let has_necrotic_rot = cloud.talent_params.necrotic_rot;

        let should_tick = cloud.time_since_last_tick >= cloud.tick_interval;
        if should_tick {
            cloud.time_since_last_tick = 0.0;
        }

        // Skip unit iteration if nothing to do this frame
        if !should_tick && !has_plague_carrier {
            continue;
        }

        for (entity, transform, mut health, mut temp_hp, has_spell_shield, already_marked, team) in
            &mut units
        {
            let inside = horizontal_distance(cloud.origin, transform.translation) <= cloud.radius;

            if inside {
                // Mark unit as inside cloud (for Plague Carrier tracking), skip if already marked
                if has_plague_carrier && !already_marked {
                    commands.entity(entity).insert(InsidePlagueCloud);
                }

                if should_tick {
                    // Toxic Weakness: additive vulnerability while inside cloud
                    if has_toxic_weakness {
                        health.spell_vulnerability += constants::TOXIC_WEAKNESS_VULNERABILITY;
                        commands
                            .entity(entity)
                            .insert(ToxicWeaknessDebuff(constants::TOXIC_WEAKNESS_VULNERABILITY));
                    }

                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        cloud.damage_per_tick,
                        DamageType::Poison,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                    if hit_tracker.track_hit(entity) {
                        unique_hits += 1;
                    }

                    // Necrotic Rot: reduce max HP by the damage dealt
                    if has_necrotic_rot {
                        let max_hp_reduction = cloud.damage_per_tick
                            * constants::NECROTIC_ROT_MAX_HP_REDUCTION_FRACTION;
                        health.max = (health.max - max_hp_reduction).max(1.0);
                        health.current = health.current.min(health.max);
                    }

                    // Choking Gas: slow enemies inside
                    if has_choking_gas {
                        commands.entity(entity).insert(SlowMovementModifier::new(
                            constants::CHOKING_GAS_SLOW,
                            constants::CHOKING_GAS_SLOW_DURATION,
                        ));
                    }
                }
            }
        }
    }

    if unique_hits > 0
        && let Some(ref mut progress) = talent_progress
    {
        progress.increment(Spell::PlagueWind, unique_hits);
    }
}

/// Removes Toxic Weakness vulnerability from units no longer in any cloud with the talent.
pub fn cleanup_toxic_weakness(
    mut commands: Commands,
    clouds: Query<&PlagueWindCloud>,
    mut debuffed_units: Query<(Entity, &Transform, &ToxicWeaknessDebuff, &mut Health)>,
) {
    for (entity, transform, debuff, mut health) in &mut debuffed_units {
        let still_inside = clouds.iter().any(|cloud| {
            cloud.talent_params.toxic_weakness
                && horizontal_distance(cloud.origin, transform.translation) <= cloud.radius
        });

        if !still_inside {
            health.spell_vulnerability = (health.spell_vulnerability - debuff.0).max(0.0);
            commands.entity(entity).remove::<ToxicWeaknessDebuff>();
        }
    }
}

/// Removes InsidePlagueCloud marker from units no longer in any cloud,
/// and applies Plague Carrier lingering DoT when they leave.
pub fn track_plague_carrier(
    mut commands: Commands,
    clouds: Query<&PlagueWindCloud>,
    marked_units: Query<(Entity, &Transform), With<InsidePlagueCloud>>,
) {
    for (entity, transform) in &marked_units {
        let mut still_inside = false;
        let mut carrier_damage = 0.0_f32;

        for cloud in &clouds {
            if !cloud.talent_params.plague_carrier {
                continue;
            }

            if horizontal_distance(cloud.origin, transform.translation) <= cloud.radius {
                still_inside = true;
                break;
            }
            // Track highest damage cloud for the lingering DoT
            carrier_damage = carrier_damage
                .max(cloud.damage_per_tick * constants::PLAGUE_CARRIER_DAMAGE_FRACTION);
        }

        if !still_inside {
            commands.entity(entity).remove::<InsidePlagueCloud>();

            if carrier_damage > 0.0 {
                commands.entity(entity).insert(PlagueCarrierDoT::new(
                    carrier_damage,
                    constants::PLAGUE_CARRIER_TICK_INTERVAL,
                    constants::PLAGUE_CARRIER_DURATION,
                ));
            }
        }
    }
}

/// Applies lingering Plague Carrier DoT damage and cleans up expired DoTs.
pub fn apply_plague_carrier_dot(
    mut commands: Commands,
    time: Res<Time>,
    mut dot_units: Query<(
        Entity,
        &mut PlagueCarrierDoT,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();

    for (entity, mut dot, mut health, mut temp_hp, has_spell_shield, team) in &mut dot_units {
        dot.time_remaining -= delta;
        dot.time_since_last_tick += delta;

        if dot.time_remaining <= 0.0 {
            commands.entity(entity).remove::<PlagueCarrierDoT>();
            continue;
        }

        if dot.time_since_last_tick >= dot.tick_interval {
            dot.time_since_last_tick = 0.0;
            apply_spell_damage_with_team(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                dot.damage_per_tick,
                DamageType::Poison,
                has_spell_shield,
                caster_team,
                *team,
            );
        }
    }
}

/// Pandemic: when an enemy dies inside a cloud, spawn a smaller child cloud at their position.
/// Only triggers once per death (uses PandemicProcessed marker) and only from non-child clouds.
pub fn spawn_pandemic_clouds(
    mut commands: Commands,
    clouds: Query<&PlagueWindCloud>,
    dead_units: Query<(Entity, &Transform, &Health), (Without<Corpse>, Without<PandemicProcessed>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, transform, health) in &dead_units {
        if !health.is_dead() {
            continue;
        }

        let unit_pos = transform.translation;

        for cloud in &clouds {
            if !cloud.talent_params.pandemic {
                continue;
            }

            if horizontal_distance(cloud.origin, unit_pos) <= cloud.radius {
                // Spawn stationary child cloud at death position
                let child_radius = cloud.radius * constants::PANDEMIC_CHILD_RADIUS_MULT;

                // Child inherits parent talents but cannot spawn further children
                let mut child_params = cloud.talent_params;
                child_params.pandemic = false;

                spawn_plague_cloud(
                    &mut commands,
                    &mut obstacle_events,
                    unit_pos,
                    child_radius,
                    cloud.damage_per_tick,
                    constants::PANDEMIC_CHILD_DURATION,
                    0.0, // Stationary
                    Vec3::ZERO,
                    child_params,
                );

                // Mark this death as processed so we don't spawn again next frame
                commands.entity(entity).insert(PandemicProcessed);

                // Only spawn one child per death
                break;
            }
        }
    }
}

/// Continuously spawns plague smoke particles from active clouds.
pub fn emit_plague_cloud_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut clouds: Query<&mut PlagueWindCloud>,
    assets: Res<SpellVisualAssets>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for mut cloud in &mut clouds {
        // Don't emit particles during fade-out
        let remaining = cloud.duration - cloud.time_alive;
        if remaining < constants::FADE_DURATION {
            continue;
        }

        cloud.smoke_spawn_timer += dt;
        if cloud.smoke_spawn_timer >= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL {
            cloud.smoke_spawn_timer -= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL;

            vfx::systems::spawn_plague_smoke_puffs(
                &mut commands,
                &assets,
                cloud.origin,
                cloud.radius,
                vfx::constants::PLAGUE_SMOKE_COUNT_PER_SPAWN,
                t,
            );
        }
    }
}

/// Cleans up expired plague wind clouds and notifies pathfinding.
pub fn cleanup_plague_wind_cloud(
    mut commands: Commands,
    // Ghost clouds are reconciliation-driven and host-authoritative; never run
    // the lifetime cleanup (which fires `ObstacleChanged` into the pathfinding
    // grid) on them. Matches the same exclusion on `move_plague_wind_cloud`.
    clouds: Query<
        (Entity, &PlagueWindCloud),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, cloud) in &clouds {
        if cloud.time_alive >= cloud.duration {
            let origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
            let buffered = cloud.radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(origin_2d, buffered)),
                rebuild: false,
            });
            commands.entity(entity).try_despawn();
        }
    }
}
