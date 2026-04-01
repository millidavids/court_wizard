use bevy::prelude::*;

use super::components::Pond;
use super::constants::*;
use super::resources::PondAssets;
use crate::game::battlefield::components::{WaterRipple, WaterRippleAssets};
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{Corpse, Health};
use crate::game::units::wizard::archetypes::meteorologist::components::WetModifier;

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

/// Helper: checks if a position is inside any pond.
fn is_in_any_pond(unit_xz: Vec2, ponds: &Query<&Pond>) -> bool {
    ponds.iter().any(|pond| {
        let pond_xz = Vec2::new(pond.center.x, pond.center.z);
        unit_xz.distance_squared(pond_xz) <= pond.radius * pond.radius
    })
}

/// Single-pass system that applies Wet to units in ponds and refreshes their timer.
/// Units not in a pond are left alone (timer ticks down via `tick_wet_timer`).
pub fn apply_pond_wet(
    mut commands: Commands,
    ponds: Query<&Pond>,
    mut units: Query<
        (Entity, &Transform, Option<&mut WetModifier>),
        (With<Health>, Without<Corpse>),
    >,
) {
    for (entity, transform, wet) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        if !is_in_any_pond(unit_xz, &ponds) {
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

/// Emits ripple effects within each pond, reusing the WaterRipple system.
pub fn emit_pond_ripples(
    time: Res<Time>,
    mut ponds: Query<&mut Pond>,
    ripple_assets: Option<Res<WaterRippleAssets>>,
    pond_assets: Res<PondAssets>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if ripple_assets.is_none() {
        return;
    };

    for mut pond in &mut ponds {
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
