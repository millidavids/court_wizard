//! Wall of fire casting and spawn.

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{
    WallOfFireCaster, WallOfFireEffect, WallOfFirePreview, WallOfFireSfx, WallOfFireTalentParams,
};
use super::constants;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, UniqueHitTracker, apply_target_assist, build_wizard_input,
    clamp_to_spell_range,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Computes the axis-aligned bounding box of a rotated wall, expanded by the obstacle buffer.
///
/// The wall is defined by its start/end points and half-width. The AABB covers the
/// rotated rectangle plus a buffer zone so units start rerouting before reaching it.
pub(super) fn wall_obstacle_bounds(start: Vec3, end: Vec3, half_width: f32) -> Rect {
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
    talent_params: WallOfFireTalentParams,
}

/// Computes talent parameters from the player's active talent selections.
fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> WallOfFireTalentParams {
    let mut params = WallOfFireTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::WallOfFire, 0);
    let t2 = talents.get_selection(Spell::WallOfFire, 1);
    let t3 = talents.get_selection(Spell::WallOfFire, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => params.damage_mult = constants::INFERNAL_INTENSITY_DAMAGE_MULT,
        Some(1) => {
            params.width_mult = constants::FIREBREAK_WIDTH_MULT;
            params.duration_mult = constants::FIREBREAK_DURATION_MULT;
        }
        Some(2) => {
            params.max_length_mult = constants::FLASH_FIRE_MAX_LENGTH_MULT;
            params.damage_mult = constants::FLASH_FIRE_DAMAGE_MULT;
            params.duration_mult = constants::FLASH_FIRE_DURATION_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.searing_heat = true,
        Some(1) => params.scorched_earth = true,
        Some(2) => params.spreading_flames = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.firestorm = true,
        Some(1) => params.twin_walls = true,
        Some(2) => params.consuming_inferno = true,
        _ => {}
    }

    params
}

/// Computes the Transform for a wall entity given its start/end points and half-width.
fn wall_transform(start: Vec3, end: Vec3, half_width: f32) -> Transform {
    let wall_dir = (end - start).normalize_or_zero();
    let wall_len = start.distance(end);
    let center = start + wall_dir * (wall_len / 2.0);
    let rotation = Quat::from_rotation_arc(Vec3::X, wall_dir);
    let preview_height = 10.0;
    Transform::from_xyz(center.x, preview_height / 2.0 + 1.0, center.z)
        .with_rotation(rotation)
        .with_scale(Vec3::new(wall_len, preview_height, half_width * 2.0))
}

/// Spawns fire sparks and looping SFX along a wall segment.
fn spawn_wall_vfx(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    start: Vec3,
    end: Vec3,
    wall_entity: Entity,
) {
    let wall_dir = (end - start).normalize_or_zero();
    let wall_len = start.distance(end);
    let spark_points = 4;
    let t_secs = start.x * 0.01;
    for j in 0..spark_points {
        let frac = (j as f32 + 0.5) / spark_points as f32;
        let pos = start + wall_dir * (wall_len * frac);
        vfx::systems::spawn_fire_sparks(
            commands,
            visual_assets,
            pos,
            vfx::constants::SPARK_COUNT / 2,
            t_secs + j as f32,
        );
    }

    let midpoint = (start + end) / 2.0;
    let sfx_entity = audio::play_looping_sfx_at(
        commands,
        &sfx.wall_of_fire_persistent,
        midpoint,
        game_config,
        sfx,
    );
    commands
        .entity(sfx_entity)
        .insert(WallOfFireSfx { wall_entity });
}

/// Local wizard Wall of Fire casting — reads mouse input, manages preview.
#[allow(clippy::too_many_arguments)]
pub fn handle_wall_of_fire_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    cursor_resources: (
        Res<CorrectedCursorPosition>,
        Res<TargetAssistWorldPos>,
        Res<LocalSpellOrigin>,
    ),
    mut caster_query: Query<&mut WallOfFireCaster>,
    mut preview_query: Query<&mut Transform, (With<WallOfFirePreview>, Without<Wizard>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    active_talents: Option<Res<ActiveTalents>>,
    active_toggles: Option<Res<ActiveToggles>>,
    mut audio_ctx: (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (ref sfx, ref game_config, ref mut pending_cast_events) = audio_ctx;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

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

    let talent_params = compute_talent_params(active_talents.as_deref());

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, local_origin.0, wizard.spell_range));

    let cast_result = wall_of_fire_casting_logic(
        &input,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut caster,
        &mut obstacle_events,
        &talent_params,
        scorched_mult,
    );

    // Handle preview spawning on cast start (anchor set, no preview yet)
    if caster.anchor.is_some()
        && caster.preview_entity.is_none()
        && let Some(pos) = clamped_pos
    {
        let preview_height = 10.0;
        let preview_entity = commands
            .spawn((
                Mesh3d(visual_assets.unit_cuboid.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: PREVIEW_COLOR,
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_xyz(pos.x, preview_height / 2.0 + 1.0, pos.z).with_scale(
                    Vec3::new(0.0, preview_height, WALL_WIDTH * talent_params.width_mult),
                ),
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
        let max_len = MAX_WALL_LENGTH * talent_params.max_length_mult;
        let length = diff.length().min(max_len);

        if length > 0.1 {
            let forward = diff.normalize();
            let center = anchor + forward * (length / 2.0);
            let rotation = Quat::from_rotation_arc(Vec3::X, forward);
            let preview_height = 10.0;
            let preview_width = WALL_WIDTH * talent_params.width_mult;

            preview_transform.translation =
                Vec3::new(center.x, preview_height / 2.0 + 1.0, center.z);
            preview_transform.rotation = rotation;
            preview_transform.scale = Vec3::new(length, preview_height, preview_width);
        }
    }

    // On successful placement, convert preview entity to active fire wall
    if let Some(ref info) = cast_result.wall_placed {
        // Build the list of walls to spawn (1 normally, 2 for Twin Walls)
        let (walls, wall_count) = if info.talent_params.twin_walls {
            let wall_dir = (info.wall_end - info.wall_start).normalize_or_zero();
            let perp = Vec3::new(-wall_dir.z, 0.0, wall_dir.x);
            let offset = perp * info.half_width;
            let twin_damage = info.damage * constants::TWIN_WALLS_DAMAGE_MULT;
            (
                [
                    (
                        info.wall_start + offset,
                        info.wall_end + offset,
                        info.half_width,
                        twin_damage,
                    ),
                    (
                        info.wall_start - offset,
                        info.wall_end - offset,
                        info.half_width,
                        twin_damage,
                    ),
                ],
                2,
            )
        } else {
            (
                [
                    (info.wall_start, info.wall_end, info.half_width, info.damage),
                    (Vec3::ZERO, Vec3::ZERO, 0.0, 0.0), // unused
                ],
                1,
            )
        };

        for (i, &(start, end, hw, dmg)) in walls[..wall_count].iter().enumerate() {
            let wall_mat = materials.add(StandardMaterial {
                base_color: Color::NONE,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            let effect = WallOfFireEffect::new(
                start,
                end,
                hw,
                dmg,
                DamageType::Fire,
                TICK_INTERVAL,
                info.fire_duration,
                info.talent_params.clone(),
            );
            let transform = wall_transform(start, end, hw);
            let net = NetworkedSpellEffect {
                kind: SpellEffectKind::WallOfFire,
            };

            let wall_entity = if i == 0 {
                if let Some(preview_entity) = caster.preview_entity {
                    commands
                        .entity(preview_entity)
                        .remove::<WallOfFirePreview>()
                        .insert((
                            MeshMaterial3d(wall_mat),
                            transform,
                            effect,
                            UniqueHitTracker::default(),
                            net,
                        ));
                    preview_entity
                } else {
                    continue;
                }
            } else {
                commands
                    .spawn((
                        Mesh3d(visual_assets.unit_cuboid.clone()),
                        MeshMaterial3d(wall_mat),
                        transform,
                        effect,
                        UniqueHitTracker::default(),
                        net,
                        OnGameplayScreen,
                    ))
                    .id()
            };

            // Twin Walls repositions the first wall, so re-notify pathfinding for all walls
            if info.talent_params.twin_walls || i > 0 {
                obstacle_events.write(ObstacleChanged {
                    bounds: wall_obstacle_bounds(start, end, hw),
                    obstacle_type: ObstacleType::Hazard(4.5),
                    shape: Some(ObstacleShape::obb_from_wall(
                        start,
                        end,
                        hw + OBSTACLE_BUFFER,
                    )),
                    rebuild: true,
                });
            }

            spawn_wall_vfx(
                &mut commands,
                &visual_assets,
                sfx,
                game_config,
                start,
                end,
                wall_entity,
            );
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
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Fire,
            time.elapsed_secs(),
        );
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
    talent_params: &WallOfFireTalentParams,
    scorched_mult: f32,
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
            let max_len = MAX_WALL_LENGTH * talent_params.max_length_mult;

            if length >= MIN_WALL_LENGTH && mana.can_afford(MANA_COST) {
                let clamped_length = length.min(max_len);
                let forward = diff.normalize();

                mana.consume(MANA_COST);

                let scale = primed_spell.empowerment;
                let fire_duration =
                    FIRE_DURATION * scale * talent_params.duration_mult * scorched_mult;
                let damage = DAMAGE_PER_TICK * scale * talent_params.damage_mult;
                let half_width = WALL_WIDTH / 2.0 * scale * talent_params.width_mult;

                let wall_start = anchor;
                let wall_end = anchor + forward * clamped_length;

                // Notify pathfinding about hazard (for non-twin-walls; twin walls re-notifies)
                if !talent_params.twin_walls {
                    obstacle_events.write(ObstacleChanged {
                        bounds: wall_obstacle_bounds(wall_start, wall_end, half_width),
                        obstacle_type: ObstacleType::Hazard(4.5),
                        shape: Some(ObstacleShape::obb_from_wall(
                            wall_start,
                            wall_end,
                            half_width + OBSTACLE_BUFFER,
                        )),
                        rebuild: true,
                    });
                }

                result.wall_placed = Some(WallPlacedInfo {
                    wall_start,
                    wall_end,
                    half_width,
                    damage,
                    fire_duration,
                    talent_params: talent_params.clone(),
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
