use super::super::components::{
    FirestormMarked, FirestormProcessed, InsideWallOfFire, ScorchedEarthZone, SearingHeatDebuff,
    SpreadingFlamesDoT, WallOfFireEffect,
};
use super::super::constants;
use super::super::constants::TICK_INTERVAL;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::{GhostEntity, GhostSpellEffect};
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, ResidualFireDamaged, SlowMovementModifier, Team, TemporaryHitPoints,
    apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

/// Handles units exiting wall of fire zones:
/// - Removes InsideWallOfFire marker
/// - Restores healing_reduction from Searing Heat debuff
/// - Applies Spreading Flames lingering DoT
#[allow(clippy::type_complexity)]
pub fn track_wall_of_fire_exit(
    mut commands: Commands,
    walls: Query<&WallOfFireEffect, Without<GhostSpellEffect>>,
    mut marked_units: Query<
        (
            Entity,
            &Transform,
            Option<&SearingHeatDebuff>,
            Option<&mut Health>,
        ),
        (With<InsideWallOfFire>, Without<GhostEntity>),
    >,
) {
    for (entity, transform, searing, health) in &mut marked_units {
        let mut still_inside = false;
        let mut spreading_damage = 0.0_f32;

        for wall in &walls {
            let distance = wall.distance_to_point(transform.translation);
            if distance <= wall.half_width {
                still_inside = true;
                break;
            }
            // Track the highest damage wall for spreading flames
            if wall.talent_params.spreading_flames {
                spreading_damage = spreading_damage
                    .max(wall.effective_damage() * constants::SPREADING_FLAMES_DAMAGE_FRACTION);
            }
        }

        if !still_inside {
            // Restore healing_reduction from Searing Heat before removing debuff
            if let Some(debuff) = searing {
                if let Some(mut hp) = health {
                    hp.healing_reduction = (hp.healing_reduction - debuff.0).max(0.0);
                }
                commands.entity(entity).remove::<SearingHeatDebuff>();
            }

            commands.entity(entity).remove::<InsideWallOfFire>();

            // Apply Spreading Flames DoT on exit
            if spreading_damage > 0.0 {
                commands.entity(entity).insert(SpreadingFlamesDoT {
                    damage_per_tick: spreading_damage,
                    tick_interval: TICK_INTERVAL,
                    time_remaining: constants::SPREADING_FLAMES_DURATION,
                    tick_timer: 0.0,
                });
            }
        }
    }
}

/// Applies lingering fire DoT from the Spreading Flames talent.
#[allow(clippy::type_complexity)]
pub fn apply_spreading_flames_dot(
    mut commands: Commands,
    time: Res<Time>,
    mut dots: Query<
        (
            Entity,
            &mut SpreadingFlamesDoT,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (Without<GhostEntity>, Without<StagingAttacker>),
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    let delta = time.delta_secs();
    let caster_team = local_player_team(session.as_deref());

    for (entity, mut dot, mut health, mut temp_hp, has_spell_shield, team) in &mut dots {
        dot.time_remaining -= delta;
        if dot.time_remaining <= 0.0 {
            commands.entity(entity).remove::<SpreadingFlamesDoT>();
            continue;
        }

        dot.tick_timer += delta;
        if dot.tick_timer >= dot.tick_interval {
            dot.tick_timer = 0.0;
            apply_spell_damage_with_team(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                dot.damage_per_tick,
                DamageType::Fire,
                has_spell_shield,
                caster_team,
                *team,
            );
            commands.entity(entity).insert(ResidualFireDamaged);
        }
    }
}

/// Applies Scorched Earth slow to units inside burnt zones.
#[allow(clippy::type_complexity)]
pub fn apply_scorched_earth_slow(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<(Entity, &mut ScorchedEarthZone), Without<GhostSpellEffect>>,
    targets: Query<
        (Entity, &Transform),
        (
            Without<Corpse>,
            Without<GhostEntity>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (zone_entity, mut zone) in &mut zones {
        zone.time_alive += delta;
        if zone.time_alive >= zone.duration {
            commands.entity(zone_entity).try_despawn();
            continue;
        }

        zone.tick_timer += delta;
        if zone.tick_timer >= constants::SCORCHED_EARTH_TICK_INTERVAL {
            zone.tick_timer = 0.0;

            for (entity, transform) in &targets {
                let distance = zone.distance_to_point(transform.translation);
                if distance <= zone.half_width {
                    commands.entity(entity).insert(SlowMovementModifier::new(
                        constants::SCORCHED_EARTH_SLOW,
                        constants::SCORCHED_EARTH_SLOW_DURATION,
                    ));
                }
            }
        }
    }
}

/// Firestorm: when a FirestormMarked unit dies, spawns a fireball-like explosion at its position.
#[allow(clippy::type_complexity)]
pub fn firestorm_death_explosion(
    mut commands: Commands,
    dead_units: Query<
        (Entity, &Transform, &Health),
        (
            With<FirestormMarked>,
            Without<Corpse>,
            Without<FirestormProcessed>,
            Without<GhostEntity>,
            Without<StagingAttacker>,
        ),
    >,
    assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    time: Res<Time>,
) {
    for (entity, transform, health) in &dead_units {
        if !health.is_dead() {
            continue;
        }

        commands.entity(entity).insert(FirestormProcessed);

        let pos = transform.translation;
        let time_secs = time.elapsed_secs();

        // Spawn a FireballExplosion (reuses fireball's damage/growth/visual systems)
        let damage_per_tick = constants::FIRESTORM_EXPLOSION_DAMAGE
            / (constants::FIRESTORM_EXPLOSION_DURATION
                / crate::game::units::wizard::spells::fireball::constants::DAMAGE_TICK_INTERVAL);
        let mut explosion = FireballExplosion::new(
            pos,
            constants::FIRESTORM_EXPLOSION_RADIUS,
            damage_per_tick,
            DamageType::Fire,
            1.0,
        );
        explosion.duration = constants::FIRESTORM_EXPLOSION_DURATION;
        explosion.source_spell = Spell::WallOfFire;

        let mat_handle =
            clone_sphere_material(&mut sphere_materials, &assets.fireball_explosion_sphere);

        commands.spawn((
            Mesh3d(assets.explosion_sphere.clone()),
            MeshMaterial3d(mat_handle),
            Transform::from_translation(pos).with_scale(Vec3::splat(0.1)),
            explosion,
            OnGameplayScreen,
        ));

        // Sparks + smoke are spawned automatically by update_explosions

        // Heat shimmer
        vfx::systems::spawn_heat_shimmer(&mut commands, &assets, pos, 2, time_secs);
    }
}
