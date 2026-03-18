use bevy::prelude::*;
use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{
    FirestormMarked, FirestormProcessed, InsideWallOfFire, ScorchedEarthZone, SearingHeatDebuff,
    SpreadingFlamesDoT, WallOfFireCaster, WallOfFireEffect, WallOfFirePreview, WallOfFireSfx,
    WallOfFireTalentParams,
};
use super::constants;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, ResidualFireDamaged, SlowMovementModifier, TemporaryHitPoints,
    apply_spell_damage,
};
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    UniqueHitTracker, clamp_to_spell_range, get_cursor_world_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;

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
    commands.entity(sfx_entity).insert(WallOfFireSfx {
        wall_entity,
    });
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
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut caster_query: Query<&mut WallOfFireCaster>,
    mut preview_query: Query<&mut Transform, (With<WallOfFirePreview>, Without<Wizard>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
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

    let talent_params = compute_talent_params(active_talents.as_deref());

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
        &talent_params,
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
                Transform::from_xyz(pos.x, preview_height / 2.0 + 1.0, pos.z)
                    .with_scale(Vec3::new(0.0, preview_height, WALL_WIDTH * talent_params.width_mult)),
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
            ([
                (info.wall_start + offset, info.wall_end + offset, info.half_width, twin_damage),
                (info.wall_start - offset, info.wall_end - offset, info.half_width, twin_damage),
            ], 2)
        } else {
            ([
                (info.wall_start, info.wall_end, info.half_width, info.damage),
                (Vec3::ZERO, Vec3::ZERO, 0.0, 0.0), // unused
            ], 1)
        };

        for (i, &(start, end, hw, dmg)) in walls[..wall_count].iter().enumerate() {
            let wall_mat = materials.add(StandardMaterial {
                base_color: Color::NONE,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            let effect = WallOfFireEffect::new(
                start, end, hw, dmg, DamageType::Fire,
                TICK_INTERVAL, info.fire_duration, info.talent_params.clone(),
            );
            let transform = wall_transform(start, end, hw);
            let net = NetworkedSpellEffect { kind: SpellEffectKind::WallOfFire };

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
                    shape: Some(ObstacleShape::obb_from_wall(start, end, hw + OBSTACLE_BUFFER)),
                    rebuild: true,
                });
            }

            spawn_wall_vfx(
                &mut commands, &visual_assets, &sfx, &game_config,
                start, end, wall_entity,
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
                let fire_duration = FIRE_DURATION * scale * talent_params.duration_mult;
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
/// Also marks units as InsideWallOfFire for talent tracking and applies Searing Heat.
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
) {
    let delta = time.delta_secs();

    for (mut effect, mut hit_tracker) in &mut effects {
        effect.time_alive += delta;
        effect.time_since_last_tick += delta;

        if effect.time_since_last_tick >= effect.tick_interval {
            effect.time_since_last_tick = 0.0;

            let tick_damage = effect.effective_damage();
            let mut units_hit = 0u32;

            for (entity, transform, mut health, mut temp_hp, has_spell_shield, is_inside, searing) in &mut targets {
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
                        commands.entity(entity).insert(SearingHeatDebuff(
                            constants::SEARING_HEAT_HEALING_REDUCTION,
                        ));
                    }

                    if hit_tracker.track_hit(entity) {
                        units_hit += 1;
                    }
                }
            }

            if units_hit > 0 {
                if let Some(ref mut progress) = talent_progress {
                    progress.increment(Spell::WallOfFire, units_hit);
                }
            }
        }
    }
}


/// Despawns wall of fire effects that have expired.
/// If Scorched Earth talent is active, spawns a slow zone in its place.
pub fn cleanup_wall_of_fire(
    mut commands: Commands,
    effects: Query<(Entity, &WallOfFireEffect)>,
    mut materials: ResMut<Assets<StandardMaterial>>,  // For scorched earth zones
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
    mut marked_units: Query<(
        Entity,
        &Transform,
        Option<&SearingHeatDebuff>,
        Option<&mut Health>,
    ), With<InsideWallOfFire>>,
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
        (With<FirestormMarked>, Without<Corpse>, Without<FirestormProcessed>),
    >,
    assets: Res<SpellVisualAssets>,
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

        commands.spawn((
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.fireball_explosion.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(0.1)),
            explosion,
            OnGameplayScreen,
        ));

        // Sparks + smoke are spawned automatically by update_explosions

        // Heat shimmer
        vfx::systems::spawn_heat_shimmer(&mut commands, &assets, pos, 2, time_secs);
    }
}
