//! Wall of fire cancel + damage application.

use super::super::super::components::{CastingState, LocalWizard, Spell};
use super::casting::wall_obstacle_bounds;
use super::components::{
    FirestormMarked, FirestormProcessed, InsideWallOfFire, ScorchedEarthZone, SearingHeatDebuff,
    SpreadingFlamesDoT, WallOfFireCaster, WallOfFireEffect, WallOfFireSfx,
};
use super::constants;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, ResidualFireDamaged, SlowMovementModifier, TemporaryHitPoints,
    apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::utils::UniqueHitTracker;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use bevy::prelude::*;

/// Computes the axis-aligned bounding box of a rotated wall, expanded by the obstacle buffer.
///
/// The wall is defined by its start/end points and half-width. The AABB covers the
/// rotated rectangle plus a buffer zone so units start rerouting before reaching it.
pub fn handle_wall_of_fire_cancel(
    mut mouse_right_pressed: MessageReader<crate::game::input::messages::MouseRightPressed>,
    mut commands: Commands,
    mut wizard_query: Query<&mut CastingState, With<LocalWizard>>,
    mut caster_query: Query<&mut WallOfFireCaster, With<LocalWizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    if mouse_right_pressed.read().next().is_none() {
        return;
    }

    let Ok(mut casting_state) = wizard_query.single_mut() else {
        return;
    };

    let Ok(mut caster) = caster_query.single_mut() else {
        return;
    };

    if let Some(preview_entity) = caster.preview_entity {
        commands.entity(preview_entity).try_despawn();
    }

    caster.anchor = None;
    caster.preview_entity = None;
    casting_state.cancel();
    mouse_state.left_consumed = true;
}

/// Applies periodic fire damage to all units within the wall's rectangular area.
/// Also marks units as InsideWallOfFire for talent tracking and applies Searing Heat.
#[allow(clippy::too_many_arguments)]
pub fn apply_wall_of_fire_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(&mut WallOfFireEffect, &mut UniqueHitTracker)>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        Has<InsideWallOfFire>,
        Option<&SearingHeatDebuff>,
    )>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
) {
    let delta = time.delta_secs();

    for (mut effect, mut hit_tracker) in &mut effects {
        effect.time_alive += delta;
        effect.time_since_last_tick += delta;

        if effect.time_since_last_tick >= effect.tick_interval {
            effect.time_since_last_tick = 0.0;

            let tick_damage = effect.effective_damage();
            let mut units_hit = 0u32;

            // Broadcast terrain damage covering the wall's footprint (bounding circle at midpoint).
            let midpoint = (effect.start + effect.end) * 0.5;
            let half_length = effect.start.distance(effect.end) * 0.5;
            terrain_damage.write(TerrainDamageMessage {
                position: midpoint,
                radius: half_length + effect.half_width,
                damage: tick_damage,
                damage_type: DamageType::Fire,
            });

            for (
                entity,
                transform,
                mut health,
                mut temp_hp,
                has_spell_shield,
                is_inside,
                searing,
            ) in &mut targets
            {
                let distance = effect.distance_to_point(transform.translation);

                if distance <= effect.half_width {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        tick_damage,
                        DamageType::Fire,
                        has_spell_shield,
                    );
                    commands.entity(entity).insert(ResidualFireDamaged);

                    // Mark unit as inside wall for Spreading Flames / Searing Heat tracking
                    if !is_inside {
                        commands.entity(entity).insert(InsideWallOfFire);
                    }

                    // Firestorm: mark unit so it explodes on death (even after leaving)
                    if effect.talent_params.firestorm {
                        commands.entity(entity).insert(FirestormMarked);
                    }

                    // Searing Heat: apply healing reduction debuff
                    if effect.talent_params.searing_heat && searing.is_none() {
                        health.healing_reduction += constants::SEARING_HEAT_HEALING_REDUCTION;
                        commands
                            .entity(entity)
                            .insert(SearingHeatDebuff(constants::SEARING_HEAT_HEALING_REDUCTION));
                    }

                    if hit_tracker.track_hit(entity) {
                        units_hit += 1;
                    }
                }
            }

            if units_hit > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::WallOfFire, units_hit);
            }
        }
    }
}

/// Despawns wall of fire effects that have expired.
/// If Scorched Earth talent is active, spawns a slow zone in its place.
pub fn cleanup_wall_of_fire(
    mut commands: Commands,
    effects: Query<(Entity, &WallOfFireEffect)>,
    mut materials: ResMut<Assets<StandardMaterial>>, // For scorched earth zones
    visual_assets: Res<SpellVisualAssets>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, effect) in &effects {
        if effect.time_alive >= effect.duration {
            // Scorched Earth: leave behind a slow zone
            if effect.talent_params.scorched_earth {
                let wall_dir = (effect.end - effect.start).normalize_or_zero();
                let wall_len = effect.start.distance(effect.end);
                let center = effect.start + wall_dir * (wall_len / 2.0);
                let rotation = Quat::from_rotation_arc(Vec3::X, wall_dir);

                commands.spawn((
                    Mesh3d(visual_assets.unit_cuboid.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.15, 0.08, 0.02, 0.4),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from_xyz(center.x, 0.5, center.z)
                        .with_rotation(rotation)
                        .with_scale(Vec3::new(wall_len, 1.0, effect.half_width * 2.0)),
                    ScorchedEarthZone {
                        start: effect.start,
                        end: effect.end,
                        half_width: effect.half_width,
                        duration: constants::SCORCHED_EARTH_DURATION,
                        time_alive: 0.0,
                        tick_timer: 0.0,
                    },
                    OnGameplayScreen,
                ));
            }

            // Clear hazard from pathfinding (same bounds as when spawned)
            obstacle_events.write(ObstacleChanged {
                bounds: wall_obstacle_bounds(effect.start, effect.end, effect.half_width),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::obb_from_wall(
                    effect.start,
                    effect.end,
                    effect.half_width + OBSTACLE_BUFFER,
                )),
                rebuild: true,
            });

            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns thick orange fire smoke along the wall, plus black smoke and heat shimmer above.
pub fn spawn_wall_of_fire_smoke(
    mut commands: Commands,
    effects: Query<&WallOfFireEffect>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < WALL_SMOKE_INTERVAL {
        return;
    }
    *timer -= WALL_SMOKE_INTERVAL;

    let t = time.elapsed_secs();

    for effect in effects.iter() {
        // Don't emit smoke during the fade-out period
        let remaining = effect.duration - effect.time_alive;
        if remaining < FADE_DURATION {
            continue;
        }

        let wall_dir = (effect.end - effect.start).normalize_or_zero();
        let wall_len = effect.start.distance(effect.end);

        // Spawn orange fire smoke puffs at multiple points along the wall.
        // Each puff will automatically emit a black smoke puff at its apex.
        let num_points = ((wall_len / 40.0) as usize).max(3);
        for j in 0..num_points {
            let frac = (j as f32 + (t * 2.3 + j as f32 * 1.7).fract()) / num_points as f32;
            let pos = effect.start + wall_dir * (wall_len * frac.clamp(0.0, 1.0));

            vfx::systems::spawn_fire_orange_smoke(
                &mut commands,
                &visual_assets,
                pos,
                effect.half_width,
                3,
                t + j as f32,
            );
        }
    }
}

/// Despawns orphaned wall of fire sound effects whose parent no longer exists.
pub(super) fn cleanup_wall_of_fire_sfx(
    mut commands: Commands,
    sfx_entities: Query<(Entity, &WallOfFireSfx)>,
    walls: Query<&WallOfFireEffect>,
) {
    for (entity, sfx) in sfx_entities.iter() {
        if walls.get(sfx.wall_entity).is_err() {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Interval between smoke wisp spawns for wall of fire.
const WALL_SMOKE_INTERVAL: f32 = 0.25;

// === Talent Systems ===

/// Handles units exiting wall of fire zones:
/// - Removes InsideWallOfFire marker
/// - Restores healing_reduction from Searing Heat debuff
/// - Applies Spreading Flames lingering DoT
pub fn track_wall_of_fire_exit(
    mut commands: Commands,
    walls: Query<&WallOfFireEffect>,
    mut marked_units: Query<
        (
            Entity,
            &Transform,
            Option<&SearingHeatDebuff>,
            Option<&mut Health>,
        ),
        With<InsideWallOfFire>,
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
pub fn apply_spreading_flames_dot(
    mut commands: Commands,
    time: Res<Time>,
    mut dots: Query<(
        Entity,
        &mut SpreadingFlamesDoT,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) {
    let delta = time.delta_secs();

    for (entity, mut dot, mut health, mut temp_hp, has_spell_shield) in &mut dots {
        dot.time_remaining -= delta;
        if dot.time_remaining <= 0.0 {
            commands.entity(entity).remove::<SpreadingFlamesDoT>();
            continue;
        }

        dot.tick_timer += delta;
        if dot.tick_timer >= dot.tick_interval {
            dot.tick_timer = 0.0;
            apply_spell_damage(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                dot.damage_per_tick,
                DamageType::Fire,
                has_spell_shield,
            );
            commands.entity(entity).insert(ResidualFireDamaged);
        }
    }
}

/// Applies Scorched Earth slow to units inside burnt zones.
pub fn apply_scorched_earth_slow(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<(Entity, &mut ScorchedEarthZone)>,
    targets: Query<(Entity, &Transform), Without<Corpse>>,
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
pub fn firestorm_death_explosion(
    mut commands: Commands,
    dead_units: Query<
        (Entity, &Transform, &Health),
        (
            With<FirestormMarked>,
            Without<Corpse>,
            Without<FirestormProcessed>,
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
