use bevy::prelude::*;
use super::components::{
    InsidePlagueCloud, PandemicProcessed, PlagueCarrierDoT, PlagueWindCloud,
    PlagueWindIndicator, PlagueWindTalentParams, ToxicWeaknessDebuff,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::input::MouseButtonState;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{
    Corpse, Health, SlowMovementModifier, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, UniqueHitTracker, clamp_to_spell_range_ground,
    get_cursor_world_position, spawn_circle_indicator,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::game::units::DamageType;
use crate::networking::snapshot::SpellEffectKind;

/// Computes talent parameters from the player's active talent selections.
fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> PlagueWindTalentParams {
    let mut params = PlagueWindTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::PlagueWind, 0);
    let t2 = talents.get_selection(Spell::PlagueWind, 1);
    let t3 = talents.get_selection(Spell::PlagueWind, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => params.damage_mult = constants::VIRULENT_STRAIN_DAMAGE_MULT,
        Some(1) => {
            params.radius_mult = constants::MIASMA_RADIUS_MULT;
            params.duration_mult = constants::MIASMA_DURATION_MULT;
        }
        Some(2) => {
            params.duration_mult = constants::LINGERING_FOG_DURATION_MULT;
            params.speed_mult = constants::LINGERING_FOG_SPEED_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.plague_carrier = true,
        Some(1) => params.toxic_weakness = true,
        Some(2) => params.choking_gas = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.pandemic = true,
        Some(1) => params.twin_plumes = true,
        Some(2) => params.necrotic_rot = true,
        _ => {}
    }

    params
}

/// Local wizard plague wind casting -- click-drag vector mechanic.
/// Click locks origin, drag defines travel direction, cast completes when timer fills.
#[allow(clippy::too_many_arguments)]
pub fn handle_plague_wind_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    circle_indicator_query: Query<&SpellCircleIndicator>,
    indicator_query: Query<&PlagueWindIndicator>,
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
    if primed_spell.spell != Spell::PlagueWind {
        return;
    }

    let wizard_pos = SPELL_ORIGIN;
    let scale = primed_spell.empowerment;
    let radius = constants::CLOUD_RADIUS * scale;
    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range_ground(pos, wizard_pos, wizard.spell_range, radius));

    // Spawn indicator on Resting -> Casting transition (origin locked at click position)
    if matches!(*casting_state, CastingState::Resting)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(constants::MANA_COST)
        && let Some(pos) = clamped_pos
    {
        // Spawn directional arrow
        let arrow_entity = commands
            .spawn((
                Mesh3d(visual_assets.arrow_mesh.clone()),
                MeshMaterial3d(visual_assets.plague_wind_arrow.clone()),
                arrow_transform(pos, 0.0),
                OnGameplayScreen,
            ))
            .id();

        let circle_entity = spawn_circle_indicator(
            &mut commands,
            &mut meshes,
            visual_assets.plague_wind_indicator.clone(),
            pos,
            constants::CLOUD_RADIUS * scale,
        )
        .insert(PlagueWindIndicator {
            arrow_entity: Some(arrow_entity),
        })
        .id();
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(circle_entity));
    }

    // Update arrow direction during casting (indicator position stays locked)
    if matches!(*casting_state, CastingState::Casting { .. })
        && let Some(cursor) = clamped_pos
        && let Ok(caster) = caster_query.get(wizard_entity)
        && let Some(indicator_entity) = caster.indicator_entity
        && let Ok(circle_indicator) = circle_indicator_query.get(indicator_entity)
        && let Ok(pw_indicator) = indicator_query.get(indicator_entity)
        && let Some(arrow_entity) = pw_indicator.arrow_entity
    {
        let origin = circle_indicator.position;
        let delta_xz = Vec2::new(cursor.x - origin.x, cursor.z - origin.z);
        if delta_xz.length_squared() > 1.0 {
            let angle = -delta_xz.x.atan2(-delta_xz.y);
            commands
                .entity(arrow_entity)
                .insert(arrow_transform(origin, angle));
        }
    }

    // Get the locked origin from indicator
    let indicator_pos = caster_query
        .get(wizard_entity)
        .ok()
        .and_then(|caster| caster.indicator_entity)
        .and_then(|ie| circle_indicator_query.get(ie).ok())
        .map(|indicator| indicator.position);

    let effective_input = WizardInput {
        cursor_pos: indicator_pos.or(clamped_pos),
        ..input
    };

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = plague_wind_casting_logic(
        &effective_input,
        &time,
        wizard_entity,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &indicator_query,
        &mut commands,
        &mut obstacle_events,
        &sfx,
        &game_config,
        talent_params,
        clamped_pos,
    );

    if completed {
        mouse_state.left_consumed = true;
    }
}

/// Core plague wind casting logic.
/// `cursor_pos` is the current (live) cursor position used to compute travel direction.
#[allow(clippy::too_many_arguments)]
fn plague_wind_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &Query<&PlagueWindIndicator>,
    commands: &mut Commands,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: PlagueWindTalentParams,
    cursor_pos: Option<Vec3>,
) -> bool {
    let wizard_pos = SPELL_ORIGIN;
    let scale = primed_spell.empowerment;
    let radius = constants::CLOUD_RADIUS * scale * talent_params.radius_mult;

    // Check for release event — cancels the cast
    if input.just_released {
        cleanup_indicator(commands, caster_query, indicator_query, wizard_entity);
        commands.entity(wizard_entity).remove::<SpellCaster>();
        casting_state.cancel();
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
                && input.cursor_pos.is_some()
            {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    // Origin is the locked indicator position (where the user first clicked)
                    let pos = input.cursor_pos.unwrap_or(wizard_pos);

                    // Direction = from origin toward current cursor position
                    let base_direction = cursor_pos
                        .map(|cursor| {
                            let delta = Vec3::new(cursor.x - pos.x, 0.0, cursor.z - pos.z);
                            if delta.length_squared() > 1.0 {
                                delta.normalize()
                            } else {
                                // If cursor hasn't moved, default forward
                                Vec3::X
                            }
                        })
                        .unwrap_or(Vec3::X);

                    let damage = constants::DAMAGE_PER_TICK * scale * talent_params.damage_mult;
                    let duration = constants::CLOUD_DURATION * scale * talent_params.duration_mult;
                    let speed = constants::CLOUD_SPEED * talent_params.speed_mult;

                    audio::play_sfx(commands, &sfx.plague_wind_cast, pos, game_config, sfx);

                    if talent_params.twin_plumes {
                        // Twin Plumes: spawn 2 clouds at diverging angles
                        let half_angle = constants::TWIN_PLUMES_ANGLE_SPREAD / 2.0;
                        let twin_damage = damage * constants::TWIN_PLUMES_DAMAGE_MULT;

                        for angle_offset in [-half_angle, half_angle] {
                            let cos_a = angle_offset.cos();
                            let sin_a = angle_offset.sin();
                            let dir = Vec3::new(
                                base_direction.x * cos_a - base_direction.z * sin_a,
                                0.0,
                                base_direction.x * sin_a + base_direction.z * cos_a,
                            )
                            .normalize();

                            spawn_plague_cloud(
                                commands,
                                obstacle_events,
                                pos,
                                radius,
                                twin_damage,
                                duration,
                                speed,
                                dir,
                                talent_params,
                            );
                        }
                    } else {
                        spawn_plague_cloud(
                            commands,
                            obstacle_events,
                            pos,
                            radius,
                            damage,
                            duration,
                            speed,
                            base_direction,
                            talent_params,
                        );
                    }

                    completed = true;
                }

                // Clean up indicator and caster regardless of mana success
                cleanup_indicator(commands, caster_query, indicator_query, wizard_entity);
                commands.entity(wizard_entity).remove::<SpellCaster>();
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_indicator(commands, caster_query, indicator_query, wizard_entity);
            commands.entity(wizard_entity).remove::<SpellCaster>();
            casting_state.cancel();
        }
    }

    completed
}

/// Builds a flat arrow Transform at the given position, rotated by `angle` radians in the XZ plane.
fn arrow_transform(origin: Vec3, angle: f32) -> Transform {
    Transform::from_translation(Vec3::new(origin.x, constants::CLOUD_BASE_Y + 0.1, origin.z))
        .with_rotation(
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2) * Quat::from_rotation_z(angle),
        )
        .with_scale(Vec3::new(
            constants::ARROW_WIDTH,
            constants::ARROW_LENGTH,
            1.0,
        ))
}

/// Despawns the indicator circle and its arrow entity.
fn cleanup_indicator(
    commands: &mut Commands,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &Query<&PlagueWindIndicator>,
    wizard_entity: Entity,
) {
    if let Ok(caster) = caster_query.get(wizard_entity)
        && let Some(indicator_entity) = caster.indicator_entity
    {
        // Despawn arrow if it exists
        if let Ok(indicator) = indicator_query.get(indicator_entity)
            && let Some(arrow_entity) = indicator.arrow_entity
        {
            commands.entity(arrow_entity).try_despawn();
        }
        commands.entity(indicator_entity).try_despawn();
    }
}

/// Spawns a plague wind cloud entity (game logic only — particles are spawned continuously).
#[allow(clippy::too_many_arguments)]
fn spawn_plague_cloud(
    commands: &mut Commands,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    pos: Vec3,
    radius: f32,
    damage: f32,
    duration: f32,
    speed: f32,
    direction: Vec3,
    talent_params: PlagueWindTalentParams,
) {
    // Notify pathfinding
    let origin_2d = Vec2::new(pos.x, pos.z);
    let buffered = radius + OBSTACLE_BUFFER;
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
        obstacle_type: ObstacleType::Hazard(10.0),
        shape: Some(ObstacleShape::circle(origin_2d, buffered)),
        rebuild: false,
    });

    commands.spawn((
        Transform::from_translation(Vec3::new(pos.x, 0.0, pos.z)),
        PlagueWindCloud::new(
            pos, radius, damage, constants::TICK_INTERVAL, duration, speed, direction,
            talent_params,
        ),
        UniqueHitTracker::default(),
        NetworkedSpellEffect {
            kind: SpellEffectKind::PlagueWindCloud,
        },
        OnGameplayScreen,
    ));
}

/// Moves the plague wind cloud in its drift direction and updates pathfinding.
pub fn move_plague_wind_cloud(
    time: Res<Time>,
    mut clouds: Query<(&mut PlagueWindCloud, &mut Transform)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    for (mut cloud, mut transform) in clouds.iter_mut() {
        // Remove old pathfinding bounds
        let old_origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
        let buffered = cloud.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(old_origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(old_origin_2d, buffered)),
            rebuild: false,
        });

        // Move cloud
        let movement = cloud.direction * cloud.speed * delta;
        cloud.origin += movement;
        transform.translation.x = cloud.origin.x;
        transform.translation.z = cloud.origin.z;

        // Add new pathfinding bounds
        let new_origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(new_origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Hazard(10.0),
            shape: Some(ObstacleShape::circle(new_origin_2d, buffered)),
            rebuild: false,
        });
    }
}

/// Returns horizontal (XZ-plane) distance between two 3D positions.
fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

/// Applies periodic necrotic damage to all units within the cloud.
/// Handles Toxic Weakness (vulnerability), Choking Gas (slow), Necrotic Rot (max HP reduction),
/// and tracks units inside cloud for Plague Carrier.
pub fn apply_plague_wind_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut clouds: Query<(&mut PlagueWindCloud, &mut UniqueHitTracker)>,
    mut units: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        Has<InsidePlagueCloud>,
    )>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let delta = time.delta_secs();
    let mut unique_hits: u32 = 0;

    for (mut cloud, mut hit_tracker) in &mut clouds {
        cloud.time_alive += delta;
        cloud.time_since_last_tick += delta;

        let has_plague_carrier = cloud.talent_params.plague_carrier;
        let has_toxic_weakness = cloud.talent_params.toxic_weakness;
        let has_choking_gas = cloud.talent_params.choking_gas;
        let has_necrotic_rot = cloud.talent_params.necrotic_rot;

        let should_tick = cloud.time_since_last_tick >= cloud.tick_interval;
        if should_tick {
            cloud.time_since_last_tick = 0.0;
        }

        // Skip unit iteration if nothing to do this frame
        if !should_tick && !has_plague_carrier {
            continue;
        }

        for (entity, transform, mut health, mut temp_hp, has_spell_shield, already_marked) in
            &mut units
        {
            let inside = horizontal_distance(cloud.origin, transform.translation) <= cloud.radius;

            if inside {
                // Mark unit as inside cloud (for Plague Carrier tracking), skip if already marked
                if has_plague_carrier && !already_marked {
                    commands.entity(entity).insert(InsidePlagueCloud);
                }

                if should_tick {
                    // Toxic Weakness: additive vulnerability while inside cloud
                    if has_toxic_weakness {
                        health.spell_vulnerability += constants::TOXIC_WEAKNESS_VULNERABILITY;
                        commands
                            .entity(entity)
                            .insert(ToxicWeaknessDebuff(constants::TOXIC_WEAKNESS_VULNERABILITY));
                    }

                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        cloud.damage_per_tick,
                        DamageType::Poison,
                        has_spell_shield,
                    );
                    if hit_tracker.track_hit(entity) {
                        unique_hits += 1;
                    }

                    // Necrotic Rot: reduce max HP by the damage dealt
                    if has_necrotic_rot {
                        let max_hp_reduction = cloud.damage_per_tick
                            * constants::NECROTIC_ROT_MAX_HP_REDUCTION_FRACTION;
                        health.max = (health.max - max_hp_reduction).max(1.0);
                        health.current = health.current.min(health.max);
                    }

                    // Choking Gas: slow enemies inside
                    if has_choking_gas {
                        commands.entity(entity).insert(SlowMovementModifier::new(
                            constants::CHOKING_GAS_SLOW,
                            constants::CHOKING_GAS_SLOW_DURATION,
                        ));
                    }
                }
            }
        }
    }

    if unique_hits > 0 {
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::PlagueWind, unique_hits);
        }
    }
}

/// Removes Toxic Weakness vulnerability from units no longer in any cloud with the talent.
pub fn cleanup_toxic_weakness(
    mut commands: Commands,
    clouds: Query<&PlagueWindCloud>,
    mut debuffed_units: Query<(Entity, &Transform, &ToxicWeaknessDebuff, &mut Health)>,
) {
    for (entity, transform, debuff, mut health) in &mut debuffed_units {
        let still_inside = clouds.iter().any(|cloud| {
            cloud.talent_params.toxic_weakness
                && horizontal_distance(cloud.origin, transform.translation) <= cloud.radius
        });

        if !still_inside {
            health.spell_vulnerability = (health.spell_vulnerability - debuff.0).max(0.0);
            commands.entity(entity).remove::<ToxicWeaknessDebuff>();
        }
    }
}

/// Removes InsidePlagueCloud marker from units no longer in any cloud,
/// and applies Plague Carrier lingering DoT when they leave.
pub fn track_plague_carrier(
    mut commands: Commands,
    clouds: Query<&PlagueWindCloud>,
    marked_units: Query<(Entity, &Transform), With<InsidePlagueCloud>>,
) {
    for (entity, transform) in &marked_units {
        let mut still_inside = false;
        let mut carrier_damage = 0.0_f32;

        for cloud in &clouds {
            if !cloud.talent_params.plague_carrier {
                continue;
            }

            if horizontal_distance(cloud.origin, transform.translation) <= cloud.radius {
                still_inside = true;
                break;
            }
            // Track highest damage cloud for the lingering DoT
            carrier_damage = carrier_damage
                .max(cloud.damage_per_tick * constants::PLAGUE_CARRIER_DAMAGE_FRACTION);
        }

        if !still_inside {
            commands.entity(entity).remove::<InsidePlagueCloud>();

            if carrier_damage > 0.0 {
                commands.entity(entity).insert(PlagueCarrierDoT::new(
                    carrier_damage,
                    constants::PLAGUE_CARRIER_TICK_INTERVAL,
                    constants::PLAGUE_CARRIER_DURATION,
                ));
            }
        }
    }
}

/// Applies lingering Plague Carrier DoT damage and cleans up expired DoTs.
pub fn apply_plague_carrier_dot(
    mut commands: Commands,
    time: Res<Time>,
    mut dot_units: Query<(
        Entity,
        &mut PlagueCarrierDoT,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) {
    let delta = time.delta_secs();

    for (entity, mut dot, mut health, mut temp_hp, has_spell_shield) in &mut dot_units {
        dot.time_remaining -= delta;
        dot.time_since_last_tick += delta;

        if dot.time_remaining <= 0.0 {
            commands.entity(entity).remove::<PlagueCarrierDoT>();
            continue;
        }

        if dot.time_since_last_tick >= dot.tick_interval {
            dot.time_since_last_tick = 0.0;
            apply_spell_damage(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                dot.damage_per_tick,
                DamageType::Poison,
                has_spell_shield,
            );
        }
    }
}

/// Pandemic: when an enemy dies inside a cloud, spawn a smaller child cloud at their position.
/// Only triggers once per death (uses PandemicProcessed marker) and only from non-child clouds.
pub fn spawn_pandemic_clouds(
    mut commands: Commands,
    clouds: Query<&PlagueWindCloud>,
    dead_units: Query<
        (Entity, &Transform, &Health),
        (Without<Corpse>, Without<PandemicProcessed>),
    >,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, transform, health) in &dead_units {
        if !health.is_dead() {
            continue;
        }

        let unit_pos = transform.translation;

        for cloud in &clouds {
            if !cloud.talent_params.pandemic {
                continue;
            }

            if horizontal_distance(cloud.origin, unit_pos) <= cloud.radius {
                // Spawn stationary child cloud at death position
                let child_radius = cloud.radius * constants::PANDEMIC_CHILD_RADIUS_MULT;

                // Child inherits parent talents but cannot spawn further children
                let mut child_params = cloud.talent_params;
                child_params.pandemic = false;

                spawn_plague_cloud(
                    &mut commands,
                    &mut obstacle_events,
                    unit_pos,
                    child_radius,
                    cloud.damage_per_tick,
                    constants::PANDEMIC_CHILD_DURATION,
                    0.0, // Stationary
                    Vec3::ZERO,
                    child_params,
                );

                // Mark this death as processed so we don't spawn again next frame
                commands.entity(entity).insert(PandemicProcessed);

                // Only spawn one child per death
                break;
            }
        }
    }
}

/// Continuously spawns plague smoke particles from active clouds.
pub fn emit_plague_cloud_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut clouds: Query<&mut PlagueWindCloud>,
    assets: Res<SpellVisualAssets>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for mut cloud in &mut clouds {
        // Don't emit particles during fade-out
        let remaining = cloud.duration - cloud.time_alive;
        if remaining < constants::FADE_DURATION {
            continue;
        }

        cloud.smoke_spawn_timer += dt;
        if cloud.smoke_spawn_timer >= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL {
            cloud.smoke_spawn_timer -= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL;

            vfx::systems::spawn_plague_smoke_puffs(
                &mut commands,
                &assets,
                cloud.origin,
                cloud.radius,
                vfx::constants::PLAGUE_SMOKE_COUNT_PER_SPAWN,
                t,
            );
        }
    }
}

/// Cleans up expired plague wind clouds and notifies pathfinding.
pub fn cleanup_plague_wind_cloud(
    mut commands: Commands,
    clouds: Query<(Entity, &PlagueWindCloud)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, cloud) in &clouds {
        if cloud.time_alive >= cloud.duration {
            let origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
            let buffered = cloud.radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(origin_2d, buffered)),
                rebuild: false,
            });
            commands.entity(entity).try_despawn();
        }
    }
}
