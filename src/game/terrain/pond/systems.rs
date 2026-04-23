use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::PondAssets;
use crate::game::battlefield::components::{WaterRipple, WaterRippleAssets};
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{
    Corpse, Health, RoughTerrainModifier, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::archetypes::meteorologist::components::WetModifier;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns a single pond at the given position with the given radius.
pub(in crate::game) fn spawn_single_pond(
    commands: &mut Commands,
    assets: &PondAssets,
    x: f32,
    z: f32,
    radius: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let center = Vec3::new(x, 0.0, z);

    commands.spawn((
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(assets.material.clone()),
        Transform::from_xyz(x, POND_SURFACE_Y, z)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
        Pond {
            center,
            radius,
            ripple_timer: 0.0,
        },
        OnGameplayScreen,
    ));

    let center_xz = Vec2::new(x, z);
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_half_size(center_xz, Vec2::splat(radius)),
        obstacle_type: ObstacleType::SlowTerrain(POND_FLOW_COST),
        shape: Some(ObstacleShape::circle(center_xz, radius)),
        rebuild: false,
    });
}

use crate::game::units::wizard::archetypes::meteorologist::components::WET_DURATION;

/// Helper: checks if a position is inside any non-frozen pond.
fn is_in_any_liquid_pond(unit_xz: Vec2, ponds: &Query<(&Pond, Has<PondFrozen>)>) -> bool {
    ponds.iter().any(|(pond, frozen)| {
        if frozen {
            return false;
        }
        let pond_xz = Vec2::new(pond.center.x, pond.center.z);
        unit_xz.distance_squared(pond_xz) <= pond.radius * pond.radius
    })
}

/// Single-pass system that applies Wet to units in liquid (non-frozen) ponds and refreshes their timer.
/// Units not in a pond are left alone (timer ticks down via `tick_wet_timer`).
pub fn apply_pond_wet(
    mut commands: Commands,
    ponds: Query<(&Pond, Has<PondFrozen>)>,
    mut units: Query<
        (Entity, &Transform, Option<&mut WetModifier>),
        (With<Health>, Without<Corpse>),
    >,
) {
    for (entity, transform, wet) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        if !is_in_any_liquid_pond(unit_xz, &ponds) {
            continue;
        }
        if let Some(mut wet) = wet {
            wet.time_remaining = WET_DURATION;
        } else {
            commands.entity(entity).insert(WetModifier {
                intensity: 1.0,
                time_remaining: WET_DURATION,
            });
        }
    }
}

/// Ticks the wet timer on all wet units and removes expired WetModifier.
pub fn tick_wet_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut wet_units: Query<(Entity, &mut WetModifier)>,
) {
    let delta = time.delta_secs();
    for (entity, mut wet) in &mut wet_units {
        wet.time_remaining -= delta;
        if wet.time_remaining <= 0.0 {
            commands.entity(entity).remove::<WetModifier>();
        }
    }
}

/// Applies a stronger movement slow to units inside frozen ponds.
///
/// Inserts `RoughTerrainModifier(FROZEN_POND_SPEED_MODIFIER)` when a unit overlaps a pond
/// whose `freeze_level` is above the pathfinding threshold. Overrides weaker existing
/// modifiers only (won't override a stronger slow from a different source).
pub fn apply_frozen_pond_slow(
    mut commands: Commands,
    ponds: Query<(&Pond, &PondFrozen)>,
    mut units: Query<
        (Entity, &Transform, Option<&mut RoughTerrainModifier>),
        (With<Health>, Without<Corpse>),
    >,
) {
    for (entity, transform, terrain_mod) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        let on_frozen_pond = ponds.iter().any(|(pond, frozen)| {
            if frozen.freeze_level < POND_FREEZE_PATHFINDING_THRESHOLD {
                return false;
            }
            let pond_xz = Vec2::new(pond.center.x, pond.center.z);
            unit_xz.distance_squared(pond_xz) <= pond.radius * pond.radius
        });
        if !on_frozen_pond {
            continue;
        }
        if let Some(mut tm) = terrain_mod {
            if tm.0 > FROZEN_POND_SPEED_MODIFIER {
                tm.0 = FROZEN_POND_SPEED_MODIFIER;
            }
        } else {
            commands
                .entity(entity)
                .insert(RoughTerrainModifier(FROZEN_POND_SPEED_MODIFIER));
        }
    }
}

/// Emits ripple effects within each liquid (non-frozen) pond, reusing the WaterRipple system.
pub fn emit_pond_ripples(
    time: Res<Time>,
    mut ponds: Query<(&mut Pond, Has<PondFrozen>)>,
    ripple_assets: Option<Res<WaterRippleAssets>>,
    pond_assets: Res<PondAssets>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if ripple_assets.is_none() {
        return;
    };

    for (mut pond, frozen) in &mut ponds {
        if frozen {
            continue;
        }
        pond.ripple_timer += time.delta_secs();
        if pond.ripple_timer < POND_RIPPLE_INTERVAL {
            continue;
        }
        pond.ripple_timer -= POND_RIPPLE_INTERVAL;

        // Pseudo-random position within the pond, constrained so ripples stay inside
        let t = time.elapsed_secs();
        let angle = (t * 2.39 + pond.center.x * 0.1).sin() * std::f32::consts::TAU;
        // Keep ripple spawn within 40% of radius so expanding ring stays inside
        let dist_frac = ((t * 17.3 + pond.center.z * 0.07).sin() * 0.5 + 0.5) * 0.4;
        let spawn_dist = pond.radius * dist_frac;
        let x = pond.center.x + angle.cos() * spawn_dist;
        let z = pond.center.z + angle.sin() * spawn_dist;

        // Scale ripple to fit within pond edge from this spawn point
        let remaining_radius = pond.radius - spawn_dist;
        let scale_variance = (t * 7.1 + pond.center.x * 0.3).sin() * 0.5 + 0.5;
        let max_scale = (remaining_radius * 0.8) * (0.6 + 0.4 * scale_variance);
        let lifetime_variance = (t * 11.3 + pond.center.z * 0.2).sin() * 0.5 + 0.5;
        let lifetime = POND_RIPPLE_LIFETIME * (0.8 + 0.4 * lifetime_variance);

        let mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.9, 0.95, 1.0, POND_RIPPLE_ALPHA),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });

        commands.spawn((
            Mesh3d(pond_assets.ripple_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(x, POND_SURFACE_Y + 0.5, z)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(5.0)),
            WaterRipple {
                lifetime: 0.0,
                max_lifetime: lifetime,
                max_scale,
            },
            OnGameplayScreen,
        ));
    }
}

/// Advances `time_since_contribution` on each evaporating pond. When no fire has hit for
/// `FOG_LINGER_DURATION` seconds, removes `PondEvaporation` and despawns the child fog cloud.
pub fn tick_pond_evaporation(
    mut commands: Commands,
    time: Res<Time>,
    mut ponds: Query<(Entity, &mut PondEvaporation)>,
    fog_clouds: Query<(Entity, &ChildOf), With<PondFogCloud>>,
) {
    let delta = time.delta_secs();
    for (entity, mut evap) in &mut ponds {
        evap.time_since_contribution += delta;
        if evap.time_since_contribution >= FOG_LINGER_DURATION {
            commands.entity(entity).remove::<PondEvaporation>();
            for (cloud_entity, child_of) in &fog_clouds {
                if child_of.parent() == entity {
                    commands.entity(cloud_entity).try_despawn();
                }
            }
        }
    }
}

/// Emits fog smoke puffs above evaporating ponds. Reuses the fog-cloud spell's particle spawner.
/// Fog radius scales with accumulated `fog_intensity`, capped at `FOG_INTENSITY_MAX`.
pub fn emit_pond_fog_particles(
    mut commands: Commands,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    ponds: Query<(&Pond, &PondEvaporation, Has<PondFrozen>)>,
    mut fog_clouds: Query<(&mut PondFogCloud, &ChildOf)>,
) {
    let delta = time.delta_secs();
    let time_secs = time.elapsed_secs();
    for (mut cloud, child_of) in &mut fog_clouds {
        let Ok((pond, evap, frozen)) = ponds.get(child_of.parent()) else {
            continue;
        };
        cloud.smoke_timer += delta;
        if cloud.smoke_timer < POND_FOG_SPAWN_INTERVAL {
            continue;
        }
        cloud.smoke_timer -= POND_FOG_SPAWN_INTERVAL;

        if frozen {
            continue; // ice doesn't evaporate
        }

        let intensity = evap.fog_intensity.clamp(0.0, FOG_INTENSITY_MAX);
        if intensity <= 0.0 {
            continue;
        }
        let cloud_radius = pond.radius * intensity;
        let count = (POND_FOG_COUNT_PER_SPAWN as f32 * intensity.max(1.0)) as usize;

        vfx::systems::spawn_fog_smoke_puffs(
            &mut commands,
            &visual_assets,
            pond.center,
            cloud_radius,
            count,
            time_secs,
        );
    }
}

/// Ticks frozen ponds: after the decay delay, thaws at `POND_FREEZE_DECAY_RATE/sec`.
/// Handles pathfinding-cost transitions and removes `PondFrozen` when fully thawed.
pub fn tick_pond_frozen(
    mut commands: Commands,
    time: Res<Time>,
    mut ponds: Query<(Entity, &Pond, &mut PondFrozen)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();
    for (entity, pond, mut frozen) in &mut ponds {
        if frozen.decay_delay > 0.0 {
            frozen.decay_delay = (frozen.decay_delay - delta).max(0.0);
        } else {
            frozen.freeze_level = (frozen.freeze_level - POND_FREEZE_DECAY_RATE * delta).max(0.0);
        }

        let should_be_frozen = frozen.freeze_level >= POND_FREEZE_PATHFINDING_THRESHOLD;
        if should_be_frozen != frozen.pathfinding_frozen {
            let center_xz = Vec2::new(pond.center.x, pond.center.z);
            let cost = if should_be_frozen {
                POND_FROZEN_FLOW_COST
            } else {
                POND_FLOW_COST
            };
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_half_size(center_xz, Vec2::splat(pond.radius)),
                obstacle_type: ObstacleType::SlowTerrain(cost),
                shape: Some(ObstacleShape::circle(center_xz, pond.radius)),
                rebuild: false,
            });
            frozen.pathfinding_frozen = should_be_frozen;
        }

        if frozen.freeze_level <= 0.0 {
            commands.entity(entity).remove::<PondFrozen>();
        }
    }
}

/// Lerps the pond's material color from liquid blue toward frozen light-blue by `freeze_level`.
/// Clones the shared material on first freeze so per-pond tinting is isolated.
/// Only runs for ponds that currently have `PondFrozen`; restore-on-thaw is handled by
/// `restore_pond_material_on_thaw`.
pub fn update_frozen_pond_tint(
    mut commands: Commands,
    mut ponds: Query<
        (
            Entity,
            &PondFrozen,
            &mut MeshMaterial3d<StandardMaterial>,
            Has<ClonedPondMaterial>,
        ),
        With<Pond>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let base = POND_COLOR.to_srgba();
    let frozen_color = FROZEN_POND_COLOR.to_srgba();

    for (entity, frozen, mut material_handle, already_cloned) in &mut ponds {
        if !already_cloned {
            let Some(shared) = materials.get(&material_handle.0) else {
                continue;
            };
            let cloned = shared.clone();
            material_handle.0 = materials.add(cloned);
            commands.entity(entity).insert(ClonedPondMaterial);
        }

        let Some(mat) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        let t = frozen.freeze_level.clamp(0.0, 1.0);
        let r = base.red + (frozen_color.red - base.red) * t;
        let g = base.green + (frozen_color.green - base.green) * t;
        let b = base.blue + (frozen_color.blue - base.blue) * t;
        let a = base.alpha + (frozen_color.alpha - base.alpha) * t;
        mat.base_color = Color::srgba(r, g, b, a);
    }
}

/// When a pond's `PondFrozen` is removed (fully thawed), restore the shared pond material
/// handle so the blue color comes back. Cheap: iterates only the removed entities.
pub fn restore_pond_material_on_thaw(
    mut commands: Commands,
    mut removed: RemovedComponents<PondFrozen>,
    mut ponds: Query<
        (
            &mut MeshMaterial3d<StandardMaterial>,
            Has<ClonedPondMaterial>,
        ),
        With<Pond>,
    >,
    pond_assets: Res<PondAssets>,
) {
    for entity in removed.read() {
        if let Ok((mut material_handle, already_cloned)) = ponds.get_mut(entity)
            && already_cloned
        {
            material_handle.0 = pond_assets.material.clone();
            commands.entity(entity).remove::<ClonedPondMaterial>();
        }
    }
}

/// Ticks shocked ponds. Each arc cooldown, arcs lightning from the pond center to up to
/// `POND_SHOCK_MAX_TARGETS` nearby non-corpse units within `POND_SHOCK_ARC_RADIUS`.
/// Damage bypasses `PendingDamageEffect`, so arcs don't propagate the shock condition.
pub fn tick_pond_shocked(
    mut commands: Commands,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut ponds: Query<(Entity, &Pond, &mut PondShocked)>,
    target_query: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<
        (
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<WetModifier>,
        ),
        Without<Corpse>,
    >,
) {
    let delta = time.delta_secs();

    for (entity, pond, mut shock) in &mut ponds {
        shock.time_remaining -= delta;
        shock.arc_cooldown = (shock.arc_cooldown - delta).max(0.0);

        if shock.time_remaining <= 0.0 {
            commands.entity(entity).remove::<PondShocked>();
            continue;
        }

        if shock.arc_cooldown > 0.0 {
            continue;
        }

        // Find nearby targets within the arc radius
        let radius_sq = POND_SHOCK_ARC_RADIUS * POND_SHOCK_ARC_RADIUS;
        let mut targets: Vec<(Entity, Vec3, f32)> = target_query
            .iter()
            .filter_map(|(target_entity, target_transform)| {
                let dx = pond.center.x - target_transform.translation.x;
                let dz = pond.center.z - target_transform.translation.z;
                let dist_sq = dx * dx + dz * dz;
                if dist_sq <= radius_sq {
                    Some((target_entity, target_transform.translation, dist_sq))
                } else {
                    None
                }
            })
            .collect();

        if targets.is_empty() {
            continue;
        }

        targets.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        targets.truncate(POND_SHOCK_MAX_TARGETS);

        shock.arc_cooldown = POND_SHOCK_ARC_COOLDOWN;

        let source_pos = Vec3::new(pond.center.x, POND_SURFACE_Y + 2.0, pond.center.z);

        for (target_entity, target_pos, _) in &targets {
            if let Ok((mut health, mut temp_hp, is_wet)) = health_query.get_mut(*target_entity) {
                let damage = if is_wet {
                    POND_SHOCK_ARC_DAMAGE * WET_ELECTRIC_DAMAGE_MULTIPLIER
                } else {
                    POND_SHOCK_ARC_DAMAGE
                };
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
            }

            crate::game::units::wizard::spells::chain_lightning::systems::spawn_arc(
                &mut commands,
                &visual_assets,
                source_pos,
                *target_pos,
                1,
                1.0,
            );
        }
    }
}
