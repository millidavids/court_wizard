use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{WallOfFireCaster, WallOfFireEffect, WallOfFirePreview, WallOfFireSfx};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, ResidualFireDamaged, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{clamp_to_spell_range, get_cursor_world_position};
use crate::config::GameConfig;

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

/// Data returned by shared logic so the wrapper can decide what to do with preview/mouse.
struct WallOfFireCastResult {
    /// Whether the spell completed successfully (fire wall was placed).
    completed: bool,
    /// Whether the cast was released but failed (too short / can't afford) — preview should be despawned.
    despawn_preview: bool,
    /// If the wall was placed, stores the wall segment info so local wrapper can convert preview.
    wall_placed: Option<WallPlacedInfo>,
}

/// Info about a successfully placed fire wall, used by the local wrapper to convert preview.
struct WallPlacedInfo {
    wall_start: Vec3,
    wall_end: Vec3,
    half_width: f32,
    damage: f32,
    fire_duration: f32,
}

/// Local wizard Wall of Fire casting — reads mouse input, manages preview.
#[allow(clippy::too_many_arguments)]
pub fn handle_wall_of_fire_casting(
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (
            Entity,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut caster_query: Query<&mut WallOfFireCaster>,
    mut preview_query: Query<&mut Transform, (With<WallOfFirePreview>, Without<Wizard>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::WallOfFire {
        return;
    }

    let mut caster = if let Ok(c) = caster_query.get_mut(wizard_entity) {
        c
    } else {
        commands
            .entity(wizard_entity)
            .insert(WallOfFireCaster::new());
        return;
    };

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, SPELL_ORIGIN, wizard.spell_range));

    let cast_result = wall_of_fire_casting_logic(
        &input,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut caster,
        &mut obstacle_events,
    );

    // Handle preview spawning on cast start (anchor set, no preview yet)
    if caster.anchor.is_some() && caster.preview_entity.is_none()
        && let Some(pos) = clamped_pos
    {
        let preview_height = 10.0;
        let preview_entity =
            commands
                .spawn((
                    Mesh3d(visual_assets.unit_cuboid.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: PREVIEW_COLOR,
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from_xyz(pos.x, preview_height / 2.0 + 1.0, pos.z)
                        .with_scale(Vec3::new(0.0, preview_height, WALL_WIDTH)),
                    WallOfFirePreview,
                    OnGameplayScreen,
                ))
                .id();

        caster.preview_entity = Some(preview_entity);
    }

    // Update preview during casting
    if matches!(*casting_state, CastingState::Casting { .. })
        && let Some(anchor) = caster.anchor
        && let Some(preview_entity) = caster.preview_entity
        && let Ok(mut preview_transform) = preview_query.get_mut(preview_entity)
        && let Some(pos) = clamped_pos
    {
        let diff = Vec3::new(pos.x - anchor.x, 0.0, pos.z - anchor.z);
        let length = diff.length().min(MAX_WALL_LENGTH);

        if length > 0.1 {
            let forward = diff.normalize();
            let center = anchor + forward * (length / 2.0);
            let rotation = Quat::from_rotation_arc(Vec3::X, forward);
            let preview_height = 10.0;

            preview_transform.translation = Vec3::new(center.x, preview_height / 2.0 + 1.0, center.z);
            preview_transform.rotation = rotation;
            preview_transform.scale = Vec3::new(length, preview_height, WALL_WIDTH);
        }
    }

    // On successful placement, convert preview entity to active fire wall
    if let Some(ref info) = cast_result.wall_placed {
        if let Some(preview_entity) = caster.preview_entity {
            // Clone wall_of_fire material for per-instance fading animation
            let base = materials
                .get(&visual_assets.wall_of_fire)
                .cloned()
                .unwrap_or_default();
            commands
                .entity(preview_entity)
                .remove::<WallOfFirePreview>()
                .insert((
                    MeshMaterial3d(materials.add(base)),
                    WallOfFireEffect::new(
                        info.wall_start,
                        info.wall_end,
                        info.half_width,
                        info.damage,
                        DamageType::Fire,
                        TICK_INTERVAL,
                        info.fire_duration,
                    ),
                    NetworkedSpellEffect {
                        kind: SpellEffectKind::WallOfFire,
                    },
                ));

            // Spawn sparks along the wall on placement
            let wall_dir = (info.wall_end - info.wall_start).normalize_or_zero();
            let wall_len = info.wall_start.distance(info.wall_end);
            let spark_points = 4;
            let t_secs = info.wall_start.x * 0.01;
            for i in 0..spark_points {
                let frac = (i as f32 + 0.5) / spark_points as f32;
                let pos = info.wall_start + wall_dir * (wall_len * frac);
                vfx::systems::spawn_fire_sparks(
                    &mut commands,
                    &visual_assets,
                    pos,
                    vfx::constants::SPARK_COUNT / 2,
                    t_secs + i as f32,
                );
            }

            // Spawn looping sound at wall midpoint
            let midpoint = (info.wall_start + info.wall_end) / 2.0;
            let sfx_entity = audio::play_looping_sfx_at(
                &mut commands,
                &sfx.wall_of_fire_persistent,
                midpoint,
                &game_config,
                &sfx,
            );
            commands.entity(sfx_entity).insert(WallOfFireSfx {
                wall_entity: preview_entity,
            });
        }
        caster.preview_entity = None;
    }

    // Despawn preview on failure (too short / can't afford)
    if cast_result.despawn_preview {
        if let Some(preview_entity) = caster.preview_entity {
            commands.entity(preview_entity).try_despawn();
        }
        caster.preview_entity = None;
    }

    if cast_result.completed {
        mouse_state.left_consumed = true;
    }
}

/// Core Wall of Fire casting logic.
///
/// Handles state machine transitions, mana consumption, and obstacle events.
/// Does NOT manage preview entities — that is the responsibility of the wrapper.
#[allow(clippy::too_many_arguments)]
fn wall_of_fire_casting_logic(
    input: &WizardInput,
    clamped_pos: Option<Vec3>,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster: &mut WallOfFireCaster,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) -> WallOfFireCastResult {
    let mut result = WallOfFireCastResult {
        completed: false,
        despawn_preview: false,
        wall_placed: None,
    };

    let Some(clamped_pos) = clamped_pos else {
        return result;
    };

    // Handle release — place fire wall or cancel
    if input.just_released {
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

                // Notify pathfinding about hazard
                obstacle_events.write(ObstacleChanged {
                    bounds: wall_obstacle_bounds(wall_start, wall_end, half_width),
                    obstacle_type: ObstacleType::Hazard(4.5),
                    shape: Some(ObstacleShape::obb_from_wall(
                        wall_start,
                        wall_end,
                        half_width + OBSTACLE_BUFFER,
                    )),
                });

                result.wall_placed = Some(WallPlacedInfo {
                    wall_start,
                    wall_end,
                    half_width,
                    damage,
                    fire_duration,
                });
                result.completed = true;
            } else {
                // Too short or can't afford
                result.despawn_preview = true;
            }

            caster.anchor = None;
            casting_state.cancel();
        }
        return result;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(MANA_COST) {
                caster.anchor = Some(clamped_pos);
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Preview update is handled by the local wrapper only
        }
        _ => {}
    }

    result
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
        commands.entity(preview_entity).try_despawn();
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
        Has<SpellShield>,
    )>,
) {
    let delta = time.delta_secs();

    for mut effect in &mut effects {
        effect.time_alive += delta;
        effect.time_since_last_tick += delta;

        if effect.time_since_last_tick >= effect.tick_interval {
            effect.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp, has_spell_shield) in &mut targets {
                let distance = effect.distance_to_point(transform.translation);

                if distance <= effect.half_width {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        effect.damage_per_tick,
                        DamageType::Fire,
                        has_spell_shield,
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
    config: Res<crate::config::GameConfig>,
) {
    let is_excremage = config.wizard_type == crate::config::WizardType::Excremage;
    for (effect, material_handle) in &effects {
        let Some(material) = materials.get_mut(material_handle) else {
            continue;
        };

        // Fade out over the last second
        let remaining = effect.duration - effect.time_alive;
        let fade = if remaining < FADE_DURATION {
            (remaining / FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        let (base_color, emissive) = vfx::systems::effect_color_at(effect.time_alive, fade, is_excremage);
        material.base_color = base_color;
        material.emissive = emissive;
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
                shape: Some(ObstacleShape::obb_from_wall(
                    effect.start,
                    effect.end,
                    effect.half_width + OBSTACLE_BUFFER,
                )),
            });

            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns smoke wisps rising off active fire walls.
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

        // Spawn a wisp at a pseudo-random point along the wall
        let frac = (t * 3.7 + effect.start.x * 0.1).fract();
        let pos = effect.start + wall_dir * (wall_len * frac);

        vfx::systems::spawn_fire_smoke_wisps(
            &mut commands,
            &visual_assets,
            pos,
            vfx::constants::SURFACE_SMOKE_COUNT,
            t,
            vfx::constants::SMOKE_LIFETIME,
            vfx::constants::SURFACE_SMOKE_SIZE,
            vfx::constants::SMOKE_RISE_SPEED,
            vfx::constants::SMOKE_SPREAD_SPEED,
        );

        vfx::systems::spawn_heat_shimmer_sized(
            &mut commands,
            &visual_assets,
            pos,
            vfx::constants::SURFACE_SHIMMER_COUNT,
            t,
            vfx::constants::SURFACE_SHIMMER_SIZE,
        );
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
