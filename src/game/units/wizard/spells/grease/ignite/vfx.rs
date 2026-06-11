use super::super::components::{
    GreaseBubble, GreaseIgnited, GreaseRegenerating, GreaseSplatter, GreaseZone,
};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Spawns wall-of-fire-style orange and black smoke puffs over burning grease zones.
pub fn spawn_grease_fire_smoke(
    mut commands: Commands,
    zones: Query<(&GreaseZone, &GreaseIgnited)>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < constants::FIRE_SMOKE_INTERVAL {
        return;
    }
    *timer -= constants::FIRE_SMOKE_INTERVAL;

    let t = time.elapsed_secs();

    for (zone, ignited) in zones.iter() {
        // Don't emit smoke during the fade-out period
        let remaining = zone.duration - zone.time_alive;
        if remaining < constants::FADE_DURATION {
            continue;
        }

        // Use the current fire radius (accounts for fire spread animation)
        let fire_radius = ignited.current_fire_radius(zone.radius, constants::FIRE_SPREAD_DURATION);

        // Spawn orange fire smoke puffs scattered across the burning area
        let fire_pos = Vec3::new(zone.origin.x, 0.0, zone.origin.z);
        vfx::systems::spawn_fire_orange_smoke(
            &mut commands,
            &visual_assets,
            fire_pos,
            fire_radius,
            9,
            t,
        );
        vfx::systems::spawn_heat_shimmer(&mut commands, &visual_assets, fire_pos, 2, t);
    }
}

/// Spawns fume wisps, bubbles, and splatters for non-ignited grease zones.
pub fn spawn_grease_zone_vfx(
    mut commands: Commands,
    zones: Query<&GreaseZone, (Without<GreaseIgnited>, Without<GreaseRegenerating>)>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut fume_timer: Local<f32>,
    mut bubble_timer: Local<f32>,
    mut splatter_timer: Local<f32>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    *fume_timer += dt;
    *bubble_timer += dt;
    *splatter_timer += dt;

    for zone in zones.iter() {
        // Don't emit VFX during fade-out
        let remaining = zone.duration - zone.time_alive;
        if remaining < constants::FADE_DURATION {
            continue;
        }

        // 1. Fume wisps — yellowish-brown vapor rising off the surface
        if *fume_timer >= constants::FUME_SPAWN_INTERVAL {
            let seed = t * 3.7 + zone.origin.x * 0.1 + zone.origin.z * 0.07;
            for i in 0..constants::FUME_COUNT_PER_SPAWN {
                let s = seed + i as f32 * 1.618_034;
                let angle = s * 2.39 + (s * 13.7).sin();
                let frac = (s * 7.3).fract();
                let offset_r = zone.radius * frac * 0.7;
                let pos = Vec3::new(
                    zone.origin.x + angle.cos() * offset_r,
                    constants::CIRCLE_Y_POSITION + 1.0,
                    zone.origin.z + angle.sin() * offset_r,
                );

                let spread_var = 0.6 + 0.4 * ((s * 17.3).sin() * 0.5 + 0.5);
                let rise_var = 0.7 + 0.3 * ((s * 23.1).cos() * 0.5 + 0.5);
                let velocity = Vec3::new(
                    angle.cos() * constants::FUME_SPREAD_SPEED * spread_var,
                    constants::FUME_RISE_SPEED * rise_var,
                    angle.sin() * constants::FUME_SPREAD_SPEED * spread_var,
                );

                commands.spawn((
                    vfx::components::FireSmoke {
                        velocity,
                        time_alive: 0.0,
                        lifetime: constants::FUME_LIFETIME,
                        base_size: constants::FUME_SIZE,
                    },
                    Mesh3d(visual_assets.particle_quad.clone()),
                    MeshMaterial3d(visual_assets.grease_fume.clone()),
                    Transform::from_translation(pos)
                        .with_rotation(UPWARD_ROTATION)
                        .with_scale(Vec3::splat(constants::FUME_SIZE)),
                    OnGameplayScreen,
                ));
            }
        }

        // 2. Bubbles — translucent spheres that rise and pop
        if *bubble_timer >= constants::BUBBLE_SPAWN_INTERVAL {
            let seed = t * 5.3 + zone.origin.x * 0.13 + zone.origin.z * 0.09;
            let angle = seed * 2.39 + (seed * 11.3).sin();
            let frac = (seed * 9.7).fract();
            let offset_r = zone.radius * frac * 0.8;
            let pos = Vec3::new(
                zone.origin.x + angle.cos() * offset_r,
                constants::CIRCLE_Y_POSITION,
                zone.origin.z + angle.sin() * offset_r,
            );

            let size_frac = (seed * 17.1).fract();
            let max_size = constants::BUBBLE_SIZE_MIN
                + size_frac * (constants::BUBBLE_SIZE_MAX - constants::BUBBLE_SIZE_MIN);
            let lifetime_frac = (seed * 23.7).fract();
            let lifetime = constants::BUBBLE_LIFETIME_MIN
                + lifetime_frac * (constants::BUBBLE_LIFETIME_MAX - constants::BUBBLE_LIFETIME_MIN);

            commands.spawn((
                GreaseBubble {
                    time_alive: 0.0,
                    lifetime,
                    max_size,
                    rise_speed: constants::BUBBLE_RISE_SPEED,
                },
                Mesh3d(visual_assets.particle_quad.clone()),
                MeshMaterial3d(visual_assets.grease_bubble.clone()),
                Transform::from_translation(pos)
                    .with_rotation(UPWARD_ROTATION)
                    .with_scale(Vec3::splat(0.5)),
                OnGameplayScreen,
            ));
        }

        // 3. Splatters — dark drops at zone edges that fade out
        if *splatter_timer >= constants::SPLATTER_SPAWN_INTERVAL {
            let seed = t * 7.1 + zone.origin.x * 0.11 + zone.origin.z * 0.13;
            let angle = seed * 2.39 + (seed * 19.3).sin();
            // Position near the edge (80-100% of radius)
            let edge_frac = 0.8 + 0.2 * (seed * 31.7).fract();
            let offset_r = zone.radius * edge_frac;
            let pos = Vec3::new(
                zone.origin.x + angle.cos() * offset_r,
                constants::CIRCLE_Y_POSITION + 0.5,
                zone.origin.z + angle.sin() * offset_r,
            );

            commands.spawn((
                GreaseSplatter {
                    time_alive: 0.0,
                    lifetime: constants::SPLATTER_LIFETIME,
                },
                Mesh3d(visual_assets.particle_quad.clone()),
                MeshMaterial3d(visual_assets.grease_splatter.clone()),
                Transform::from_translation(pos)
                    .with_rotation(UPWARD_ROTATION)
                    .with_scale(Vec3::splat(constants::SPLATTER_SIZE)),
                OnGameplayScreen,
            ));
        }
    }

    // Reset timers (outside zone loop so timing is shared)
    if *fume_timer >= constants::FUME_SPAWN_INTERVAL {
        *fume_timer -= constants::FUME_SPAWN_INTERVAL;
    }
    if *bubble_timer >= constants::BUBBLE_SPAWN_INTERVAL {
        *bubble_timer -= constants::BUBBLE_SPAWN_INTERVAL;
    }
    if *splatter_timer >= constants::SPLATTER_SPAWN_INTERVAL {
        *splatter_timer -= constants::SPLATTER_SPAWN_INTERVAL;
    }
}

/// Updates grease bubbles: grow, rise, then pop (rapid scale-down + despawn).
pub fn update_grease_bubbles(
    mut commands: Commands,
    mut bubbles: Query<(Entity, &mut GreaseBubble, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut bubble, mut transform) in &mut bubbles {
        bubble.time_alive += dt;

        if bubble.time_alive >= bubble.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Rise upward
        transform.translation.y += bubble.rise_speed * dt;

        let progress = bubble.time_alive / bubble.lifetime;
        // Grow to max_size over first 70%, then rapidly shrink (pop) in last 30%
        let size = if progress < 0.7 {
            let grow = progress / 0.7;
            // Ease-out growth
            bubble.max_size * (1.0 - (1.0 - grow) * (1.0 - grow))
        } else {
            // Pop: rapid shrink
            let pop = (1.0 - progress) / 0.3;
            bubble.max_size * pop * pop
        };
        transform.scale = Vec3::splat(size);
    }
}

/// Updates grease splatters: fade out over lifetime and despawn.
pub fn update_grease_splatters(
    mut commands: Commands,
    mut splatters: Query<(Entity, &mut GreaseSplatter, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut splatter, mut transform) in &mut splatters {
        splatter.time_alive += dt;

        if splatter.time_alive >= splatter.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Shrink linearly over lifetime
        let remaining = 1.0 - (splatter.time_alive / splatter.lifetime);
        transform.scale = Vec3::splat(constants::SPLATTER_SIZE * remaining);
    }
}
