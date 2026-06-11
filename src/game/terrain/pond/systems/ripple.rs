use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::PondAssets;
use crate::game::battlefield::components::{WaterRipple, WaterRippleAssets};
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

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
