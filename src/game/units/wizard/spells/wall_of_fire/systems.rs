use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, LocalWizard, Wizard};
use super::components::{WallOfFireCaster, WallOfFireEffect, WallOfFirePreview};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, ResidualFireDamaged, TemporaryHitPoints, apply_spell_damage,
};

/// Computes the axis-aligned bounding box of a rotated wall, expanded by the obstacle buffer.
///
/// The wall is defined by its start/end points and half-width. The AABB covers the
/// rotated rectangle plus a buffer zone so units start rerouting before reaching it.
fn wall_obstacle_bounds(start: Vec3, end: Vec3, half_width: f32) -> Rect {
    let a = Vec2::new(start.x, start.z);
    let b = Vec2::new(end.x, end.z);
    let dir = b - a;
    let perp = Vec2::new(-dir.y, dir.x).normalize_or_zero() * half_width;

    // Four corners of the rotated rectangle
    let c0 = a + perp;
    let c1 = a - perp;
    let c2 = b + perp;
    let c3 = b - perp;

    let min_x = c0.x.min(c1.x).min(c2.x).min(c3.x);
    let max_x = c0.x.max(c1.x).max(c2.x).max(c3.x);
    let min_y = c0.y.min(c1.y).min(c2.y).min(c3.y);
    let max_y = c0.y.max(c1.y).max(c2.y).max(c3.y);

    // Expand by obstacle buffer so units start rerouting before reaching the wall
    Rect::new(
        min_x - OBSTACLE_BUFFER,
        min_y - OBSTACLE_BUFFER,
        max_x + OBSTACLE_BUFFER,
        max_y + OBSTACLE_BUFFER,
    )
}

/// Handles Wall of Fire casting — click to anchor, drag to extend, release to place fire line.
#[allow(clippy::too_many_arguments)]
pub fn handle_wall_of_fire_casting(
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
            &Transform,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut caster_query: Query<&mut WallOfFireCaster, With<LocalWizard>>,
    mut preview_query: Query<&mut Transform, (With<WallOfFirePreview>, Without<Wizard>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };

    let mut caster = if let Ok(c) = caster_query.single_mut() {
        c
    } else {
        commands
            .entity(wizard_entity)
            .insert(WallOfFireCaster::new());
        return;
    };

    let mouse_released = mouse_left_released.read().next().is_some();

    let Some(cursor_pos) = get_cursor_world_position(&camera_query, &window_query) else {
        return;
    };
    let clamped_pos =
        clamp_to_spell_range(cursor_pos, wizard_transform.translation, wizard.spell_range);

    // Handle release — place fire wall or cancel
    if mouse_released {
        if let Some(anchor) = caster.anchor {
            let diff = Vec3::new(clamped_pos.x - anchor.x, 0.0, clamped_pos.z - anchor.z);
            let length = diff.length();

            if length >= MIN_WALL_LENGTH && mana.can_afford(MANA_COST) {
                let clamped_length = length.min(MAX_WALL_LENGTH);
                let forward = diff.normalize();

                mana.consume(MANA_COST);

                let scale = primed_spell.empowerment;
                let fire_duration = FIRE_DURATION * scale;
                let damage = DAMAGE_PER_TICK * scale;
                let half_width = WALL_WIDTH / 2.0 * scale;

                let wall_start = anchor;
                let wall_end = anchor + forward * clamped_length;

                // Notify pathfinding about hazard (AABB of rotated wall + buffer)
                obstacle_events.write(ObstacleChanged {
                    bounds: wall_obstacle_bounds(wall_start, wall_end, half_width),
                    obstacle_type: ObstacleType::Hazard(3.0),
                });

                // Convert the preview entity into the active fire wall
                if let Some(preview_entity) = caster.preview_entity {
                    commands
                        .entity(preview_entity)
                        .remove::<WallOfFirePreview>()
                        .insert((
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgba(1.0, 0.5, 0.0, 0.4),
                                unlit: true,
                                alpha_mode: AlphaMode::Blend,
                                cull_mode: None,
                                ..default()
                            })),
                            WallOfFireEffect::new(
                                wall_start,
                                wall_end,
                                half_width,
                                damage,
                                DamageType::Fire,
                                TICK_INTERVAL,
                                fire_duration,
                            ),
                            NetworkedSpellEffect { kind: SpellEffectKind::WallOfFire },
                        ));
                }
            } else {
                // Too short or can't afford — despawn preview
                if let Some(preview_entity) = caster.preview_entity {
                    commands.entity(preview_entity).despawn();
                }
            }

            caster.anchor = None;
            caster.preview_entity = None;
            casting_state.cancel();
            mouse_state.left_consumed = true;
        }
        return;
    }

    match *casting_state {
        CastingState::Resting => {
            if !mana.can_afford(MANA_COST) {
                return;
            }

            // Set anchor and spawn preview
            caster.anchor = Some(clamped_pos);

            let preview_height = 10.0;
            let preview_entity = commands
                .spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.0, preview_height, WALL_WIDTH))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: PREVIEW_COLOR,
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from_xyz(clamped_pos.x, preview_height / 2.0, clamped_pos.z)
                        .with_scale(Vec3::new(0.0, 1.0, 1.0)),
                    WallOfFirePreview,
                    OnGameplayScreen,
                ))
                .id();

            caster.preview_entity = Some(preview_entity);
            casting_state.start_cast();
        }
        CastingState::Casting { .. } => {
            // Update preview to stretch from anchor to cursor
            if let Some(anchor) = caster.anchor
                && let Some(preview_entity) = caster.preview_entity
                && let Ok(mut preview_transform) = preview_query.get_mut(preview_entity)
            {
                let diff = Vec3::new(clamped_pos.x - anchor.x, 0.0, clamped_pos.z - anchor.z);
                let length = diff.length().min(MAX_WALL_LENGTH);

                if length > 0.1 {
                    let forward = diff.normalize();
                    let center = anchor + forward * (length / 2.0);
                    let rotation = Quat::from_rotation_arc(Vec3::X, forward);
                    let preview_height = 10.0;

                    preview_transform.translation =
                        Vec3::new(center.x, preview_height / 2.0, center.z);
                    preview_transform.rotation = rotation;
                    preview_transform.scale = Vec3::new(length, 1.0, 1.0);
                }
            }
        }
        _ => {}
    }
}

/// Handles right-click cancellation of wall of fire placement.
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
        commands.entity(preview_entity).despawn();
    }

    caster.anchor = None;
    caster.preview_entity = None;
    casting_state.cancel();
    mouse_state.left_consumed = true;
}

/// Applies periodic fire damage to all units within the wall's rectangular area.
pub fn apply_wall_of_fire_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<&mut WallOfFireEffect>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
    )>,
) {
    let delta = time.delta_secs();

    for mut effect in &mut effects {
        effect.time_alive += delta;
        effect.time_since_last_tick += delta;

        if effect.time_since_last_tick >= effect.tick_interval {
            effect.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp) in &mut targets {
                let distance = effect.distance_to_point(transform.translation);

                if distance <= effect.half_width {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        effect.damage_per_tick,
                        DamageType::Fire,
                    );
                    commands.entity(entity).insert(ResidualFireDamaged);
                }
            }
        }
    }
}

/// Applies flickering fire visual and fades out wall of fire over the last second.
pub fn fade_wall_of_fire(
    effects: Query<(&WallOfFireEffect, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (effect, material_handle) in &effects {
        let Some(material) = materials.get_mut(material_handle) else {
            continue;
        };

        // Flicker using layered sine waves for organic fire look
        let t = effect.time_alive;
        let flicker =
            0.7 + 0.15 * (t * 8.3).sin() + 0.10 * (t * 13.7).sin() + 0.05 * (t * 23.1).sin();

        // Fade out over the last second
        let remaining = effect.duration - t;
        let fade = if remaining < FADE_DURATION {
            (remaining / FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        let base_alpha = 0.4 * fade * flicker;
        let green = 0.5 + 0.15 * (t * 11.0).sin();
        material.base_color = Color::srgba(1.0, green, 0.0, base_alpha);
    }
}

/// Despawns wall of fire effects that have expired.
pub fn cleanup_wall_of_fire(
    mut commands: Commands,
    effects: Query<(Entity, &WallOfFireEffect)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, effect) in &effects {
        if effect.time_alive >= effect.duration {
            // Clear hazard from pathfinding (same bounds as when spawned)
            obstacle_events.write(ObstacleChanged {
                bounds: wall_obstacle_bounds(effect.start, effect.end, effect.half_width),
                obstacle_type: ObstacleType::Removed,
            });

            commands.entity(entity).despawn();
        }
    }
}

fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;
    let t = -ray.origin.y / ray.direction.y;
    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

fn clamp_to_spell_range(target: Vec3, wizard_pos: Vec3, spell_range: f32) -> Vec3 {
    let diff = target - wizard_pos;
    let distance = diff.length();
    if distance > spell_range {
        wizard_pos + diff.normalize() * spell_range
    } else {
        target
    }
}
