use bevy::prelude::*;
use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::{GreaseFireOverlay, GreaseIndicator, GreaseZone};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, SlowMovementModifier, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::utils::{
    get_cursor_world_position, spawn_circle_indicator,
};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::networking::snapshot::SpellEffectKind;

/// Local wizard grease casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_grease_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut GreaseIndicator>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let input = WizardInput {
        just_pressed: true, // Run conditions already ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Grease {
        return;
    }

    let completed = grease_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &mut commands,
        &visual_assets,
        &mut materials,
        &mut obstacle_events,
        &sfx,
        &game_config,
    );

    if completed {
        mouse_state.left_consumed = true;
    }
}

/// Core grease casting logic. Returns true if the spell completed.
#[allow(clippy::too_many_arguments)]
fn grease_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut GreaseIndicator>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
) -> bool {
    let mut completed = false;

    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).try_despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return false;
    }

    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = SPELL_ORIGIN;
    let wizard_height = wizard_pos.y;
    let max_ground_radius = if wizard_height < wizard.spell_range {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };
    let scale = primed_spell.empowerment;
    let circle_radius = constants::CIRCLE_RADIUS * scale;
    let max_center_distance = (max_ground_radius - circle_radius).max(0.0);
    let direction = cursor_world_pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        cursor_world_pos = wizard_pos + normalized_direction * max_center_distance;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                let circle_entity = spawn_circle_indicator(
                    commands,
                    assets,
                    assets.grease_indicator.clone(),
                    cursor_world_pos,
                    constants::CIRCLE_RADIUS * primed_spell.empowerment,
                    constants::CIRCLE_Y_POSITION,
                )
                .insert(GreaseIndicator::new(
                    cursor_world_pos,
                    primed_spell.empowerment,
                ))
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = cursor_world_pos;
            }
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
                            audio::play_sfx(
                                commands,
                                &sfx.grease_cast,
                                indicator.position,
                                game_config,
                                sfx,
                            );
                            spawn_grease_zone(
                                commands,
                                assets,
                                materials,
                                indicator.position,
                                radius,
                                indicator.empowerment,
                                obstacle_events,
                            );
                        }
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    completed = true;
                } else {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
        }
    }

    completed
}

pub fn update_grease_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut GreaseIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();
        let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(radius * pulse);
        transform.translation.x = indicator.position.x;
        transform.translation.y = constants::CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

/// Applies slow to units inside non-ignited grease zone.
pub fn apply_grease_slow(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<&mut GreaseZone>,
    mut targets: Query<(Entity, &Transform, Option<&mut SlowMovementModifier>), Without<Corpse>>,
) {
    let delta = time.delta_secs();
    for mut zone in &mut zones {
        if zone.ignited {
            // Still apply slow even when ignited — only skip time_alive tracking
            // (ignited zones track time_alive in apply_grease_burn instead)
        } else {
            zone.time_alive += delta;
        }
        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;
            for (entity, transform, existing_slow) in &mut targets {
                let dist = Vec3::new(
                    zone.origin.x - transform.translation.x,
                    0.0,
                    zone.origin.z - transform.translation.z,
                )
                .length();
                if dist <= zone.radius {
                    if let Some(mut slow) = existing_slow {
                        slow.apply(zone.slow_modifier, zone.slow_duration);
                    } else {
                        let modifier =
                            SlowMovementModifier::new(zone.slow_modifier, zone.slow_duration);
                        commands
                            .entity(entity)
                            .queue_silenced(move |mut e: EntityWorldMut| {
                                e.insert(modifier);
                            });
                    }
                }
            }
        }
    }
}

/// Checks if any fire source overlaps the grease zone to trigger ignition.
/// Fire sources include: units with FireDoT, FireballExplosion, WallOfFireEffect,
/// MeteorGroundFire, and DisintegrateBeam.
#[allow(clippy::too_many_arguments)]
pub fn check_grease_ignition(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut zones: Query<(Entity, &mut GreaseZone)>,
    fire_units: Query<
        &Transform,
        (
            With<crate::game::units::components::FireDoT>,
            Without<Corpse>,
        ),
    >,
    fireball_explosions: Query<&FireballExplosion>,
    wall_of_fires: Query<&WallOfFireEffect>,
    meteor_ground_fires: Query<&MeteorGroundFire>,
    disintegrate_beams: Query<&DisintegrateBeam>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<Corpse>,
    >,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    // Collect ignited zone positions for chain-ignition checks
    let ignited_zones: Vec<(Vec3, f32)> = zones
        .iter()
        .filter(|(_, z)| z.ignited)
        .map(|(_, z)| (z.origin, z.radius))
        .collect();

    for (zone_entity, mut zone) in &mut zones {
        if zone.ignited {
            continue;
        }

        // Track ignition source point
        let mut ignition_pos: Option<Vec3> = None;

        // Check if any already-ignited grease zone overlaps this one
        for &(ignited_origin, ignited_radius) in &ignited_zones {
            let to_this = Vec2::new(
                zone.origin.x - ignited_origin.x,
                zone.origin.z - ignited_origin.z,
            );
            let dist = to_this.length();
            if dist <= zone.radius + ignited_radius {
                // Ignition point: nearest edge of ignited zone toward this zone
                let dir = if dist > 0.001 {
                    to_this / dist
                } else {
                    Vec2::X
                };
                let edge = Vec2::new(ignited_origin.x, ignited_origin.z) + dir * ignited_radius;
                ignition_pos = Some(Vec3::new(edge.x, 0.0, edge.y));
                break;
            }
        }

        // Check if any unit with FireDoT is inside the grease zone
        if ignition_pos.is_none() {
            for fire_transform in &fire_units {
                let dist = Vec3::new(
                    zone.origin.x - fire_transform.translation.x,
                    0.0,
                    zone.origin.z - fire_transform.translation.z,
                )
                .length();
                if dist <= zone.radius {
                    ignition_pos = Some(Vec3::new(
                        fire_transform.translation.x,
                        0.0,
                        fire_transform.translation.z,
                    ));
                    break;
                }
            }
        }

        // Check if any fireball explosion overlaps the grease zone
        if ignition_pos.is_none() {
            for explosion in &fireball_explosions {
                let dist = Vec2::new(
                    zone.origin.x - explosion.origin.x,
                    zone.origin.z - explosion.origin.z,
                )
                .length();
                if dist <= zone.radius + explosion.max_radius {
                    ignition_pos = Some(Vec3::new(explosion.origin.x, 0.0, explosion.origin.z));
                    break;
                }
            }
        }

        // Check if any wall of fire overlaps the grease zone
        if ignition_pos.is_none() {
            for wall in &wall_of_fires {
                let dist = wall.distance_to_point(zone.origin);
                if dist <= zone.radius + wall.half_width {
                    // Closest point on wall line to zone center
                    let p = Vec2::new(zone.origin.x, zone.origin.z);
                    let a = Vec2::new(wall.start.x, wall.start.z);
                    let b = Vec2::new(wall.end.x, wall.end.z);
                    let ab = b - a;
                    let ap = p - a;
                    let ab_len_sq = ab.length_squared();
                    let t = if ab_len_sq > 0.0001 {
                        (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let closest = a + ab * t;
                    ignition_pos = Some(Vec3::new(closest.x, 0.0, closest.y));
                    break;
                }
            }
        }

        // Check if any meteor ground fire overlaps the grease zone
        if ignition_pos.is_none() {
            for fire in &meteor_ground_fires {
                let dist = Vec2::new(zone.origin.x - fire.origin.x, zone.origin.z - fire.origin.z)
                    .length();
                if dist <= zone.radius + fire.radius {
                    ignition_pos = Some(Vec3::new(fire.origin.x, 0.0, fire.origin.z));
                    break;
                }
            }
        }

        // Check if any disintegrate beam passes through the grease zone at ground level
        if ignition_pos.is_none() {
            for beam in &disintegrate_beams {
                let to_zone = zone.origin - beam.origin;
                let projection = to_zone.dot(beam.direction);
                let clamped_proj = projection.clamp(0.0, beam.current_length());
                let closest = beam.origin + beam.direction * clamped_proj;
                if closest.y > constants::IGNITION_HEIGHT_THRESHOLD {
                    continue;
                }
                let dist = Vec2::new(zone.origin.x - closest.x, zone.origin.z - closest.z).length();
                if dist <= zone.radius + beam.beam_width() {
                    ignition_pos = Some(Vec3::new(closest.x, 0.0, closest.z));
                    break;
                }
            }
        }

        if let Some(ign_point) = ignition_pos {
            zone.ignited = true;
            zone.ignition_point = Some(ign_point);
            zone.fire_spread_time = 0.0;

            // Spawn fire overlay mesh at the ignition point
            let base_mat = materials
                .get(&visual_assets.grease_fire)
                .cloned()
                .unwrap_or_default();
            let overlay_material = materials.add(base_mat);
            commands.spawn((
                Mesh3d(visual_assets.unit_circle.clone()),
                MeshMaterial3d(overlay_material),
                Transform::from_translation(Vec3::new(
                    ign_point.x,
                    constants::FIRE_OVERLAY_Y_POSITION,
                    ign_point.z,
                ))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(0.01 * zone.radius)),
                GreaseFireOverlay { zone_entity },
                NetworkedSpellEffect {
                    kind: SpellEffectKind::GreaseFire,
                },
                OnGameplayScreen,
            ));

            // Apply one-time burst fire damage only near the ignition point
            if zone.ignite_damage > 0.0 {
                let burst_radius = zone.radius * constants::IGNITION_BURST_RADIUS_FRACTION;
                for (entity, transform, mut health, mut temp_hp, has_spell_shield) in &mut targets {
                    let dist = Vec2::new(
                        ign_point.x - transform.translation.x,
                        ign_point.z - transform.translation.z,
                    )
                    .length();
                    if dist <= burst_radius {
                        apply_spell_damage(
                            &mut commands,
                            entity,
                            &mut health,
                            temp_hp.as_deref_mut(),
                            zone.ignite_damage * zone.empowerment,
                            DamageType::Fire,
                            has_spell_shield,
                        );
                    }
                }
            }

            // Upgrade pathfinding to hazard
            let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
            let buffered_radius = zone.radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                obstacle_type: ObstacleType::Hazard(5.0),
                shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
            });
        }
    }
}

/// Updates fire spread animation — ticks spread timer and scales the overlay mesh.
pub fn update_grease_fire_spread(
    time: Res<Time>,
    mut zones: Query<(Entity, &mut GreaseZone)>,
    mut overlays: Query<(&GreaseFireOverlay, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (zone_entity, mut zone) in &mut zones {
        if !zone.ignited {
            continue;
        }
        zone.fire_spread_time += delta;

        let Some(ign_point) = zone.ignition_point else {
            continue;
        };

        let progress = (zone.fire_spread_time / constants::FIRE_SPREAD_DURATION).min(1.0);

        // Update the overlay transform
        for (overlay, mut transform) in &mut overlays {
            if overlay.zone_entity != zone_entity {
                continue;
            }
            // Scale from 0 to zone.radius (unit circle scaled by radius)
            transform.scale = Vec3::splat(progress * zone.radius);

            // Shift center from ignition point toward zone center as it grows
            // so the expanding circle stays within the zone bounds
            let center_x = ign_point.x + (zone.origin.x - ign_point.x) * progress;
            let center_z = ign_point.z + (zone.origin.z - ign_point.z) * progress;
            transform.translation.x = center_x;
            transform.translation.z = center_z;
        }
    }
}

/// Spawns smoke wisps and heat shimmer rising off burning grease zones.
pub fn spawn_grease_fire_smoke(
    mut commands: Commands,
    zones: Query<&GreaseZone>,
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

    for zone in zones.iter() {
        if !zone.ignited {
            continue;
        }

        // Don't emit smoke during the fade-out period
        let remaining = zone.duration - zone.time_alive;
        if remaining < constants::FADE_DURATION {
            continue;
        }

        // Pick a pseudo-random position within the fire's current radius
        let seed = t * 3.7 + zone.origin.x * 0.1 + zone.origin.z * 0.07;
        let angle = seed * 2.39 + (seed * 13.7).sin();
        let frac = (seed * 7.3).fract();
        let offset_r = zone.radius * frac * 0.8;
        let pos = Vec3::new(
            zone.origin.x + angle.cos() * offset_r,
            constants::FIRE_OVERLAY_Y_POSITION,
            zone.origin.z + angle.sin() * offset_r,
        );

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

/// Applies burn damage from ignited grease zones.
/// During fire spread, only damages units within the current fire radius.
pub fn apply_grease_burn(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<&mut GreaseZone>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<Corpse>,
    >,
) {
    let delta = time.delta_secs();
    for mut zone in &mut zones {
        if !zone.ignited {
            continue;
        }
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.ignite_burn_tick {
            zone.time_since_last_tick = 0.0;

            // During spread phase, scope damage to current fire radius from ignition point
            let fire_radius = zone.current_fire_radius(constants::FIRE_SPREAD_DURATION);
            let spreading = zone.fire_spread_time < constants::FIRE_SPREAD_DURATION;

            for (entity, transform, mut health, mut temp_hp, has_spell_shield) in &mut targets {
                let in_burn_area = if spreading {
                    if let Some(ign_point) = zone.ignition_point {
                        // Check distance from ignition point during spread
                        let dist = Vec2::new(
                            ign_point.x - transform.translation.x,
                            ign_point.z - transform.translation.z,
                        )
                        .length();
                        dist <= fire_radius
                    } else {
                        false
                    }
                } else {
                    // Fire fully spread — use full zone radius from center
                    let dist = Vec2::new(
                        zone.origin.x - transform.translation.x,
                        zone.origin.z - transform.translation.z,
                    )
                    .length();
                    dist <= zone.radius
                };

                if in_burn_area {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        zone.ignite_burn_damage * zone.empowerment,
                        DamageType::Fire,
                        has_spell_shield,
                    );
                }
            }
        }
    }
}

pub fn fade_grease_zone(
    zones: Query<(Entity, &GreaseZone, &MeshMaterial3d<StandardMaterial>)>,
    mut overlays: Query<
        (&GreaseFireOverlay, &MeshMaterial3d<StandardMaterial>),
        Without<GreaseZone>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<crate::config::GameConfig>,
) {
    let is_excremage = config.wizard_type == crate::config::WizardType::Excremage;
    for (zone_entity, zone, material_handle) in &zones {
        let remaining = zone.duration - zone.time_alive;
        let fade = if remaining < constants::FADE_DURATION {
            (remaining / constants::FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        // Fade the grease base mesh
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = Color::srgba(0.45, 0.4, 0.05, 0.4 * fade);
        }

        // Fade the fire overlay mesh if this zone is ignited
        if zone.ignited {
            let (fire_base, fire_emissive) =
                vfx::systems::effect_color_at(zone.time_alive, fade, is_excremage);
            for (overlay, overlay_handle) in &mut overlays {
                if overlay.zone_entity == zone_entity
                    && let Some(overlay_mat) = materials.get_mut(overlay_handle)
                {
                    overlay_mat.base_color = fire_base;
                    overlay_mat.emissive = fire_emissive;
                }
            }
        }
    }
}

pub fn cleanup_grease_zone(
    mut commands: Commands,
    zones: Query<(Entity, &GreaseZone)>,
    overlays: Query<(Entity, &GreaseFireOverlay)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.duration {
            if zone.ignited {
                let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
                let buffered_radius = zone.radius + OBSTACLE_BUFFER;
                obstacle_events.write(ObstacleChanged {
                    bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                    obstacle_type: ObstacleType::Removed,
                    shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
                });
                // Despawn fire overlay
                for (overlay_entity, overlay) in &overlays {
                    if overlay.zone_entity == entity {
                        commands.entity(overlay_entity).try_despawn();
                    }
                }
            }
            commands.entity(entity).try_despawn();
        }
    }
}

pub(crate) fn spawn_grease_zone(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let duration = constants::ZONE_DURATION * empowerment;
    let slow_mod = constants::SLOW_MODIFIER;
    let slow_dur = constants::SLOW_DURATION * empowerment;

    // Notify pathfinding about slow terrain
    let origin_2d = Vec2::new(position.x, position.z);
    let buffered_radius = radius + OBSTACLE_BUFFER;
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type: ObstacleType::SlowTerrain(3.0),
        shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
    });

    let base_mat = materials
        .get(&assets.grease_zone)
        .cloned()
        .unwrap_or_default();
    let instance_material = materials.add(base_mat);

    commands.spawn((
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(instance_material),
        Transform::from_translation(Vec3::new(
            position.x,
            constants::CIRCLE_Y_POSITION,
            position.z,
        ))
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::splat(radius)),
        GreaseZone::new(
            Vec3::new(position.x, 0.0, position.z),
            radius,
            slow_mod,
            slow_dur,
            constants::TICK_INTERVAL,
            duration,
            constants::IGNITE_DAMAGE,
            constants::IGNITE_BURN_DAMAGE,
            constants::IGNITE_BURN_TICK,
            empowerment,
        ),
        NetworkedSpellEffect {
            kind: SpellEffectKind::GreaseZone,
        },
        OnGameplayScreen,
    ));
}
