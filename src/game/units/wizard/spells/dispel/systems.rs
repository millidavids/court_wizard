use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::components::*;
use super::constants;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{ObstacleChanged, ObstacleShape, ObstacleType, OBSTACLE_BUFFER};
use crate::game::constants::SPELL_ORIGIN;
use crate::game::units::components::MindControlled;
use crate::game::units::wizard::components::{LocalWizard, Mana, PrimedSpell, Spell, Wizard};
use crate::game::units::wizard::spells::grease::components::GreaseZone;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::config::GameConfig;

// ===== Wizard Casting =====

/// Instant-cast dispel on click — fires projectile at cursor position.
#[allow(clippy::too_many_arguments)]
pub fn handle_dispel_casting(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
            &mut Mana,
            &PrimedSpell,
            &Wizard,
            Option<&DispelCooldown>,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let Some(target_pos) = cursor_pos else {
        return;
    };

    let Ok((wizard_entity, mut mana, primed_spell, wizard, cooldown)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Dispel {
        return;
    }

    // Check cooldown
    if cooldown.is_some_and(|cd| cd.remaining > 0.0) {
        return;
    }

    let mana_cost = constants::MANA_COST * wizard.mana_cost_multiplier;
    if !mana.consume(mana_cost) {
        return;
    }

    let origin = SPELL_ORIGIN;
    audio::play_sfx(&mut commands, &sfx.dispel_cast, origin, &game_config, &sfx);
    spawn_dispel_projectile(
        &mut commands,
        &mut meshes,
        &mut materials,
        origin,
        target_pos,
        constants::SPAWN_HEIGHT_OFFSET,
    );

    commands.entity(wizard_entity).insert(DispelCooldown {
        remaining: constants::COOLDOWN,
    });
}

/// Ticks down the dispel cooldown timer each frame.
pub fn tick_dispel_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut DispelCooldown)>,
) {
    for (entity, mut cooldown) in &mut cooldowns {
        cooldown.remaining -= time.delta_secs();
        if cooldown.remaining <= 0.0 {
            commands.entity(entity).remove::<DispelCooldown>();
        }
    }
}

// ===== Shared Spawn Helper =====

/// Spawns a dispel projectile from `origin` toward `target_pos`.
///
/// `height_offset` controls how high above the origin the projectile spawns.
/// Wizard uses `SPAWN_HEIGHT_OFFSET` (arcs down to ground), dispellers use `0.0`.
/// The projectile travels in 3D and detonates when it hits the battlefield (y<=0).
pub(crate) fn spawn_dispel_projectile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    target_pos: Vec3,
    height_offset: f32,
) {
    let spawn_pos = origin + Vec3::Y * height_offset;
    // Target is on the ground (y=0)
    let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
    let diff = ground_target - spawn_pos;
    let direction = diff.normalize_or_zero();
    let velocity = direction * constants::PROJECTILE_SPEED;

    commands.spawn((
        Mesh3d(meshes.add(Circle::new(constants::PROJECTILE_RADIUS))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: constants::PROJECTILE_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(spawn_pos),
        DispelProjectile {
            velocity,
            lifetime: constants::PROJECTILE_LIFETIME,
        },
        Billboard,
        OnGameplayScreen,
    ));
}

// ===== Projectile + Impact Systems =====

/// Moves dispel projectiles. Detonates on ground impact (y<=0) or lifetime expiry.
pub fn move_dispel_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut projectiles: Query<(Entity, &mut Transform, &mut DispelProjectile)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut projectile) in &mut projectiles {
        // Move projectile
        transform.translation += projectile.velocity * delta;
        projectile.lifetime -= delta;

        // Detonate when hitting the battlefield (y<=0) or lifetime expired
        let hit_ground = transform.translation.y <= 0.0;
        if hit_ground || projectile.lifetime <= 0.0 {
            // Impact position slightly above ground so cross-plane sphere is visible
            let impact_pos = Vec3::new(
                transform.translation.x,
                5.0,
                transform.translation.z,
            );

            commands.spawn((
                Mesh3d(visual_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: constants::PROJECTILE_COLOR
                        .with_alpha(constants::IMPACT_INITIAL_ALPHA),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_translation(impact_pos).with_scale(Vec3::ZERO),
                DispelImpact {
                    time_alive: 0.0,
                    duration: constants::IMPACT_DURATION,
                },
                OnGameplayScreen,
            ));
            commands.entity(entity).try_despawn();
        }
    }
}

/// Expands impact spheres, checks overlap with spell effects, and despawns expired impacts.
#[allow(clippy::too_many_arguments)]
pub fn update_dispel_impacts(
    mut commands: Commands,
    time: Res<Time>,
    mut impacts: Query<(
        Entity,
        &mut DispelImpact,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect), Without<DispelImpact>>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<&GreaseZone>,
    meteor_fire_query: Query<&MeteorGroundFire>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mind_controlled_query: Query<(Entity, &Transform), (With<MindControlled>, Without<DispelImpact>)>,
) {
    for (entity, mut impact, mut transform, material_handle) in &mut impacts {
        impact.time_alive += time.delta_secs();

        if impact.time_alive >= impact.duration {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = impact.time_alive / impact.duration;

        // Expand at constant speed
        let radius = constants::IMPACT_EXPAND_SPEED * impact.time_alive;
        transform.scale = Vec3::splat(radius);

        // Fade alpha
        let alpha = constants::IMPACT_INITIAL_ALPHA * (1.0 - progress);
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = constants::PROJECTILE_COLOR.with_alpha(alpha);
        }

        let impact_center = transform.translation;

        // Check overlap with dispellable spell effects
        for (spell_entity, spell_tf, nse) in &spell_effects {
            if !is_dispellable(nse.kind) {
                continue;
            }

            let edge_dist = spell_edge_distance(
                impact_center,
                spell_entity,
                spell_tf.translation,
                &wall_of_fire_query,
                &wall_of_stone_query,
                &spike_growth_query,
                &grease_query,
                &meteor_fire_query,
            );

            if edge_dist <= radius {
                despawn_spell_effect(
                    &mut commands,
                    spell_entity,
                    &wall_of_stone_query,
                    &wall_of_fire_query,
                    &spike_growth_query,
                    &grease_query,
                    &meteor_fire_query,
                    &mut obstacle_events,
                );
            }
        }

        // Remove mind control from units in range
        for (mc_entity, mc_transform) in &mind_controlled_query {
            let dx = mc_transform.translation.x - impact_center.x;
            let dz = mc_transform.translation.z - impact_center.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist <= radius {
                commands.entity(mc_entity).remove::<MindControlled>();
            }
        }
    }
}

// ===== Shared Helpers (moved from dispeller) =====

/// Returns true if the spell effect kind is dispellable.
pub(crate) fn is_dispellable(kind: SpellEffectKind) -> bool {
    !matches!(
        kind,
        SpellEffectKind::FireballExplosion
            | SpellEffectKind::MeteorExplosion
            | SpellEffectKind::IceExplosion
            | SpellEffectKind::HealingPlumeZone
    )
}

/// Computes the XZ distance from a point to the nearest edge of a spell effect's volume.
///
/// For volumetric effects (wall of fire, wall of stone, circular zones), returns the
/// distance to the closest edge of the area rather than the center. Returns 0 if
/// the point is inside the volume. Falls back to center-point distance for unknown types.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spell_edge_distance(
    point: Vec3,
    spell_entity: Entity,
    spell_center: Vec3,
    wall_of_fire_query: &Query<&WallOfFireEffect>,
    wall_of_stone_query: &Query<&WallOfStone>,
    spike_growth_query: &Query<&SpikeGrowthZone>,
    grease_query: &Query<&GreaseZone>,
    meteor_fire_query: &Query<&MeteorGroundFire>,
) -> f32 {
    // Wall of Fire: line segment with half_width
    if let Ok(wall) = wall_of_fire_query.get(spell_entity) {
        let dist_to_line = wall.distance_to_point(point);
        return (dist_to_line - wall.half_width).max(0.0);
    }

    // Wall of Stone: oriented bounding box
    if let Ok(wall) = wall_of_stone_query.get(spell_entity) {
        if wall.contains_point_xz(point) {
            return 0.0;
        }
        let diff = Vec3::new(point.x - wall.center.x, 0.0, point.z - wall.center.z);
        let forward_proj = diff.dot(wall.forward).clamp(-wall.half_length, wall.half_length);
        let right_proj = diff.dot(wall.right).clamp(-wall.half_width, wall.half_width);
        let closest = wall.center + wall.forward * forward_proj + wall.right * right_proj;
        return ((point.x - closest.x).powi(2) + (point.z - closest.z).powi(2)).sqrt();
    }

    // Spike Growth: circular zone
    if let Ok(zone) = spike_growth_query.get(spell_entity) {
        let dist_to_center =
            ((point.x - zone.origin.x).powi(2) + (point.z - zone.origin.z).powi(2)).sqrt();
        return (dist_to_center - zone.radius).max(0.0);
    }

    // Grease: circular zone
    if let Ok(zone) = grease_query.get(spell_entity) {
        let dist_to_center =
            ((point.x - zone.origin.x).powi(2) + (point.z - zone.origin.z).powi(2)).sqrt();
        return (dist_to_center - zone.radius).max(0.0);
    }

    // Meteor Ground Fire: circular zone
    if let Ok(fire) = meteor_fire_query.get(spell_entity) {
        let dist_to_center =
            ((point.x - fire.origin.x).powi(2) + (point.z - fire.origin.z).powi(2)).sqrt();
        return (dist_to_center - fire.radius).max(0.0);
    }

    // Fallback: center-point distance
    ((point.x - spell_center.x).powi(2) + (point.z - spell_center.z).powi(2)).sqrt()
}

/// Despawns a spell effect entity and cleans up its pathfinding obstacle if applicable.
#[allow(clippy::too_many_arguments)]
pub(crate) fn despawn_spell_effect(
    commands: &mut Commands,
    spell_entity: Entity,
    wall_of_stone_query: &Query<&WallOfStone>,
    wall_of_fire_query: &Query<&WallOfFireEffect>,
    spike_growth_query: &Query<&SpikeGrowthZone>,
    grease_query: &Query<&GreaseZone>,
    meteor_fire_query: &Query<&MeteorGroundFire>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    // Wall of Stone -- blocked obstacle
    if let Ok(wall) = wall_of_stone_query.get(spell_entity) {
        let obs_bounds = wall.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::obb_from_center(
                wall.center,
                wall.forward,
                wall.half_length,
                wall.half_width,
            )),
        });
    }

    // Wall of Fire -- hazard obstacle
    if let Ok(effect) = wall_of_fire_query.get(spell_entity) {
        let a = Vec2::new(effect.start.x, effect.start.z);
        let b = Vec2::new(effect.end.x, effect.end.z);
        let dir = b - a;
        let perp = Vec2::new(-dir.y, dir.x).normalize_or_zero() * effect.half_width;
        let c0 = a + perp;
        let c1 = a - perp;
        let c2 = b + perp;
        let c3 = b - perp;
        let min_x = c0.x.min(c1.x).min(c2.x).min(c3.x) - OBSTACLE_BUFFER;
        let max_x = c0.x.max(c1.x).max(c2.x).max(c3.x) + OBSTACLE_BUFFER;
        let min_y = c0.y.min(c1.y).min(c2.y).min(c3.y) - OBSTACLE_BUFFER;
        let max_y = c0.y.max(c1.y).max(c2.y).max(c3.y) + OBSTACLE_BUFFER;

        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(min_x, min_y, max_x, max_y),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::obb_from_wall(
                effect.start,
                effect.end,
                effect.half_width + OBSTACLE_BUFFER,
            )),
        });
    }

    // Spike Growth -- hazard obstacle (circular zone)
    if let Ok(zone) = spike_growth_query.get(spell_entity) {
        let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
        let buffered_radius = zone.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
        });
    }

    // Grease -- hazard obstacle when ignited
    if let Ok(zone) = grease_query.get(spell_entity)
        && zone.ignited
    {
        let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
        let buffered_radius = zone.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
        });
    }

    // Meteor Ground Fire -- hazard obstacle
    if let Ok(fire) = meteor_fire_query.get(spell_entity) {
        let origin_2d = Vec2::new(fire.origin.x, fire.origin.z);
        let buffered = fire.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(origin_2d, buffered)),
        });
    }

    commands.entity(spell_entity).try_despawn();
}
