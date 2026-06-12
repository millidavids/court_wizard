//! Entangle spell application and vine VFX: roots targets, spawns the ground-effect entity,
//! notifies pathfinding, and manages vine torus spawning/animation/cleanup.

use super::super::super::components::Wizard;
use super::casting::apply_entangle_to_unit;
use super::components::{EntangleGroundEffect, EntangleTalentParams, EntangleVine};
use super::constants;
use crate::game::achievements::messages::EntangleHitDefenderMessage;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::spells::utils::{self};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_entangle(
    rng: &mut impl rand::Rng,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    circle_pos: Vec3,
    radius: f32,
    root_duration: f32,
    targets: &Query<(Entity, &Transform, &Team), (Without<Wizard>, Without<Corpse>)>,
    defender_hit_msg: &mut MessageWriter<EntangleHitDefenderMessage>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    talent_params: &EntangleTalentParams,
) -> u32 {
    let mut hit_count = 0u32;

    for (entity, transform, team) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);
        if distance <= radius
            && apply_entangle_to_unit(
                commands,
                entity,
                team,
                root_duration,
                talent_params,
                defender_hit_msg,
            )
        {
            hit_count += 1;
        }
    }

    // Spawn invisible ground effect entity (tracks zone for overgrowth/fade)
    commands.spawn((
        Transform::from_translation(Vec3::new(
            circle_pos.x,
            constants::CIRCLE_Y_POSITION,
            circle_pos.z,
        )),
        Visibility::Hidden,
        EntangleGroundEffect::new(root_duration, circle_pos, radius, *talent_params),
        NetworkedSpellEffect {
            kind: SpellEffectKind::EntangleGround,
        },
        OnGameplayScreen,
    ));

    // Notify pathfinding that this zone is a hazard
    let buffered_radius = radius + OBSTACLE_BUFFER;
    let origin_2d = Vec2::new(circle_pos.x, circle_pos.z);
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type: ObstacleType::Hazard(15.0),
        shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
        rebuild: false,
    });

    // Spawn vine toruses rising from the ground
    spawn_vine_toruses(
        rng,
        commands,
        assets,
        materials,
        circle_pos,
        radius,
        root_duration,
        OnGameplayScreen,
    );

    hit_count
}

/// Spawns random flat vine rings within the entangle circle. Generic over the
/// cleanup marker so the multiplayer ghost can reuse it with
/// `OnMultiplayerGameScreen`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_vine_toruses<M: Component + Clone>(
    rng: &mut impl rand::Rng,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    center: Vec3,
    radius: f32,
    duration: f32,
    screen_marker: M,
) {
    for _ in 0..constants::VINE_COUNT {
        // Random position within circle (uniform distribution via rejection-free polar)
        let angle = rng.random::<f32>() * std::f32::consts::TAU;
        let dist = radius * rng.random::<f32>().sqrt() * 0.85; // 0.85 keeps vines slightly inward
        let x = center.x + angle.cos() * dist;
        let z = center.z + angle.sin() * dist;

        // Random scale
        let scale = constants::VINE_MIN_SCALE
            + rng.random::<f32>() * (constants::VINE_MAX_SCALE - constants::VINE_MIN_SCALE);

        // Random orientation — tilt the ring so it looks like a vine arching out of the ground
        // Annulus lies in XZ plane by default, so we tilt it partially upright
        let yaw = rng.random::<f32>() * std::f32::consts::TAU;
        let tilt = 0.4 + rng.random::<f32>() * 0.8; // 0.4..1.2 radians tilt from horizontal
        let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(tilt);

        // Position so at most 75% of the ring is above ground (y=0).
        // The ring's visible height above ground depends on tilt and scale.
        // We set final_y so the center is near or below ground level,
        // leaving only the top arc poking through.
        let max_above = constants::VINE_MAX_ABOVE_GROUND * (0.3 + rng.random::<f32>() * 0.7);
        // Center of ring sits below ground so only the top arch is visible
        let final_y = max_above - scale * tilt.sin() * 0.5;

        // Each vine gets its own material instance for independent alpha fading
        let base_mat = materials
            .get(&assets.entangle_vine)
            .cloned()
            .unwrap_or_default();
        let vine_material = materials.add(base_mat);

        commands.spawn((
            Mesh3d(assets.entangle_vine_ring.clone()),
            MeshMaterial3d(vine_material),
            Transform::from_translation(Vec3::new(x, constants::VINE_START_OFFSET, z))
                .with_rotation(rotation)
                .with_scale(Vec3::splat(scale)),
            EntangleVine {
                final_y,
                rise_elapsed: 0.0,
                rise_duration: constants::VINE_RISE_DURATION * (0.7 + rng.random::<f32>() * 0.6), // Stagger rise timing
                duration,
                time_remaining: duration,
            },
            screen_marker.clone(),
        ));
    }
}

/// Animates vine toruses: rise from ground, then fade out at end of life.
pub fn animate_entangle_vines(
    time: Res<Time>,
    mut commands: Commands,
    mut vines: Query<(
        Entity,
        &mut EntangleVine,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();
    for (entity, mut vine, mut transform, material_handle) in &mut vines {
        vine.time_remaining -= delta;

        // Rise animation
        if vine.rise_elapsed < vine.rise_duration {
            vine.rise_elapsed += delta;
            let progress = (vine.rise_elapsed / vine.rise_duration).clamp(0.0, 1.0);
            // Ease-out: fast start, slow finish
            let eased = 1.0 - (1.0 - progress) * (1.0 - progress);
            transform.translation.y = constants::VINE_START_OFFSET
                + (vine.final_y - constants::VINE_START_OFFSET) * eased;
        }

        // Fade out in the last 25% of lifetime
        let life_fraction = (vine.time_remaining / vine.duration).clamp(0.0, 1.0);
        if life_fraction < 0.25 {
            let fade = life_fraction / 0.25; // 1.0 → 0.0
            if let Some(material) = materials.get_mut(material_handle) {
                material.base_color = Color::srgba(0.05, 0.3, 0.05, 0.75 * fade);
            }
        }

        // Despawn when expired
        if vine.time_remaining <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns animated vine ring particles from active entangle zones.
pub fn emit_animated_vine_rings(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut effects: Query<&mut EntangleGroundEffect>,
) {
    let delta = time.delta_secs();
    for mut effect in &mut effects {
        if effect.time_remaining <= 0.0 {
            continue;
        }

        effect.animated_vine_timer += delta;
        if effect.animated_vine_timer < utils::RING_SPAWN_INTERVAL {
            continue;
        }
        effect.animated_vine_timer -= utils::RING_SPAWN_INTERVAL;

        utils::spawn_ring_particle(
            &mut game_rng.0,
            &mut commands,
            visual_assets.entangle_vine_ring.clone(),
            visual_assets.entangle_vine.clone(),
            effect.center,
            effect.current_radius,
        );
    }
}
