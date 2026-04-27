use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::{
    GreaseBubble, GreaseIgnited, GreaseOilSlickDebuff, GreaseRegenerating, GreaseSplatter,
    GreaseTalentParams, GreaseZone, GreaseZonePresenceTracker,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{
    Airborne, Corpse, Health, RootedModifier, SlowMovementModifier, TemporaryHitPoints,
    apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    cleanup_spell_caster, handle_spell_release, spawn_circle_indicator, update_indicator_position,
    xz_distance,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;
use rand::Rng;

/// Helper to write an obstacle event for a grease zone.
fn write_grease_obstacle(
    origin: Vec3,
    radius: f32,
    obstacle_type: ObstacleType,
    events: &mut MessageWriter<ObstacleChanged>,
) {
    let origin_2d = Vec2::new(origin.x, origin.z);
    let buffered_radius = radius + OBSTACLE_BUFFER;
    events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type,
        shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
        rebuild: false,
    });
}

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> GreaseTalentParams {
    let mut params = GreaseTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Grease, 0);
    let t2 = talents.get_selection(Spell::Grease, 1);
    let t3 = talents.get_selection(Spell::Grease, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Extra Slippery
            params.slow_mult = constants::EXTRA_SLIPPERY_SLOW_MULT;
        }
        Some(1) => {
            // Wider Slick
            params.radius_mult = constants::WIDER_SLICK_RADIUS_MULT;
        }
        Some(2) => {
            // Volatile Mixture
            params.burn_damage_mult = constants::VOLATILE_MIXTURE_BURN_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.slip_and_fall = true,
        Some(1) => params.oil_slick = true,
        Some(2) => params.lingering_flames = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.chain_combustion = true,
        Some(1) => params.grease_geyser = true,
        Some(2) => params.endless_oil = true,
        _ => {}
    }

    params
}

/// Local wizard grease casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_grease_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    cursor_resources: (Res<CorrectedCursorPosition>, Res<TargetAssistWorldPos>),
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    talent_resources: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
        Option<Res<ActiveToggles>>,
    ),
) {
    let (active_talents, _talent_progress, active_toggles) = talent_resources;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let (corrected_cursor, target_assist) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Grease {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

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
        &mut meshes,
        &mut materials,
        &mut obstacle_events,
        &sfx,
        &game_config,
        &talent_params,
        scorched_mult,
    );

    if completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            vfx::systems::SpellSchool::Transmutation,
            time.elapsed_secs(),
        );
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
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: &GreaseTalentParams,
    scorched_mult: f32,
) -> bool {
    let mut completed = false;

    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
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
    let circle_radius = constants::CIRCLE_RADIUS * scale * talent_params.radius_mult;
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
                    meshes,
                    assets.grease_indicator.clone(),
                    cursor_world_pos,
                    circle_radius,
                )
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            update_indicator_position(
                wizard_entity,
                cursor_world_pos,
                caster_query,
                indicator_query,
            );
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let radius = constants::CIRCLE_RADIUS
                                * primed_spell.empowerment
                                * talent_params.radius_mult;
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
                                primed_spell.empowerment,
                                obstacle_events,
                                *talent_params,
                                scorched_mult,
                            );
                        }
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    completed = true;
                } else {
                    cleanup_spell_caster(commands, wizard_entity, caster_query);
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(commands, wizard_entity, caster_query);
            casting_state.cancel();
        }
    }

    completed
}

/// Applies slow to units inside grease zones, ticks time_alive for non-ignited zones,
/// and handles Slip and Fall / Oil Slick talent effects.
#[allow(clippy::too_many_arguments)]
pub fn apply_grease_slow(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut zones: Query<(
        Entity,
        &mut GreaseZone,
        Has<GreaseIgnited>,
        Has<GreaseRegenerating>,
    )>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            Option<&mut SlowMovementModifier>,
            Option<&mut GreaseZonePresenceTracker>,
            Option<&GreaseOilSlickDebuff>,
            Option<&mut Health>,
        ),
        Without<Corpse>,
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let delta = time.delta_secs();
    let rng = &mut game_rng.0;

    for (zone_entity, mut zone, is_ignited, is_regenerating) in &mut zones {
        // Only track time_alive for non-ignited zones
        // (ignited zones track time_alive in apply_grease_burn instead)
        if !is_ignited {
            zone.time_alive += delta;
        }

        // Don't apply slow while regenerating (not yet slippery)
        if is_regenerating {
            continue;
        }

        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;

            let mut units_slowed: u32 = 0;
            let needs_tracking = zone.talent_params.slip_and_fall || zone.talent_params.oil_slick;

            for (entity, transform, existing_slow, existing_tracker, has_oil_debuff, mut health) in
                &mut targets
            {
                let dist = xz_distance(zone.origin, transform.translation);

                if dist <= zone.radius {
                    // Apply slow
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
                    units_slowed += 1;

                    if needs_tracking {
                        let is_new = existing_tracker
                            .as_ref()
                            .is_none_or(|t| t.zone_entity != zone_entity);

                        if is_new {
                            commands
                                .entity(entity)
                                .insert(GreaseZonePresenceTracker { zone_entity });

                            // Talent: Slip and Fall — stun on zone entry
                            if zone.talent_params.slip_and_fall {
                                let roll: f32 = rng.random_range(0.0..1.0);
                                if roll < constants::SLIP_AND_FALL_CHANCE {
                                    commands.entity(entity).insert(RootedModifier::new(
                                        constants::SLIP_AND_FALL_STUN_DURATION,
                                    ));
                                }
                            }

                            // Talent: Oil Slick — apply vulnerability debuff (once per unit)
                            if zone.talent_params.oil_slick
                                && has_oil_debuff.is_none()
                                && let Some(ref mut health) = health
                            {
                                health.spell_vulnerability += constants::OIL_SLICK_VULNERABILITY;
                                commands.entity(entity).insert(GreaseOilSlickDebuff::new());
                            }
                        }
                    }
                } else if needs_tracking {
                    // Unit is outside the zone — clean up tracker and debuffs
                    if let Some(ref tracker) = existing_tracker
                        && tracker.zone_entity == zone_entity
                    {
                        commands
                            .entity(entity)
                            .remove::<GreaseZonePresenceTracker>();

                        // Remove Oil Slick vulnerability when leaving
                        if let Some(debuff) = has_oil_debuff {
                            if let Some(ref mut health) = health {
                                health.spell_vulnerability =
                                    (health.spell_vulnerability - debuff.vulnerability).max(0.0);
                            }
                            commands.entity(entity).remove::<GreaseOilSlickDebuff>();
                        }
                    }
                }
            }

            // Track talent progress
            if units_slowed > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::Grease, units_slowed);
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
    mut zones: Query<
        (Entity, &mut GreaseZone),
        (Without<GreaseIgnited>, Without<GreaseRegenerating>),
    >,
    ignited_zone_query: Query<&GreaseZone, With<GreaseIgnited>>,
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
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    // Collect ignited zone positions for chain-ignition checks
    let ignited_zones: Vec<(Vec3, f32)> = ignited_zone_query
        .iter()
        .map(|z| (z.origin, z.radius))
        .collect();

    for (zone_entity, mut zone) in &mut zones {
        // Track ignition source point
        let mut ignition_pos: Option<Vec3> = None;

        // Check if any already-ignited grease zone overlaps this one
        // Chain Combustion talent extends the range
        let chain_range_mult = if zone.talent_params.chain_combustion {
            constants::CHAIN_COMBUSTION_RANGE_MULT
        } else {
            1.0
        };
        for &(ignited_origin, ignited_radius) in &ignited_zones {
            let dist = xz_distance(zone.origin, ignited_origin);
            let to_this = Vec2::new(
                zone.origin.x - ignited_origin.x,
                zone.origin.z - ignited_origin.z,
            );
            if dist <= (zone.radius + ignited_radius) * chain_range_mult {
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
                let dist = xz_distance(zone.origin, fire_transform.translation);
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
                let dist = xz_distance(zone.origin, explosion.origin);
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
                let dist = xz_distance(zone.origin, fire.origin);
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
                let dist = xz_distance(zone.origin, closest);
                if dist <= zone.radius + beam.beam_width() {
                    ignition_pos = Some(Vec3::new(closest.x, 0.0, closest.z));
                    break;
                }
            }
        }

        if let Some(ign_point) = ignition_pos {
            commands
                .entity(zone_entity)
                .insert(GreaseIgnited::new(ign_point));

            // Talent: Lingering Flames — reset time_alive so fire burns for the full zone duration
            if zone.talent_params.lingering_flames {
                zone.time_alive = 0.0;
            }

            // Apply one-time burst fire damage only near the ignition point
            if zone.ignite_damage > 0.0 {
                let burst_radius = zone.radius * constants::IGNITION_BURST_RADIUS_FRACTION;
                for (entity, transform, mut health, mut temp_hp, has_spell_shield) in &mut targets {
                    let dist = xz_distance(ign_point, transform.translation);
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

            // Talent: Grease Geyser — launch enemies upward at ignition
            if zone.talent_params.grease_geyser {
                let mut units_launched: u32 = 0;
                for (entity, transform, _health, _temp_hp, _has_spell_shield) in &mut targets {
                    let dist = xz_distance(zone.origin, transform.translation);
                    if dist <= zone.radius {
                        commands.entity(entity).insert((
                            Airborne::new(
                                constants::GEYSER_LAUNCH_VELOCITY,
                                constants::GEYSER_GRAVITY,
                                transform.translation.y,
                                DamageType::Fire,
                            ),
                            RootedModifier::new(constants::GEYSER_ROOT_DURATION),
                        ));
                        units_launched += 1;
                    }
                }
                if units_launched > 0
                    && let Some(ref mut progress) = talent_progress
                {
                    progress.increment(Spell::Grease, units_launched);
                }
            }

            // Upgrade pathfinding to hazard
            write_grease_obstacle(
                zone.origin,
                zone.radius,
                ObstacleType::Hazard(5.0),
                &mut obstacle_events,
            );
        }
    }
}

/// Updates fire spread timer for burning grease zones (controls smoke VFX and burn damage radius).
pub fn update_grease_fire_spread(
    time: Res<Time>,
    mut zones: Query<(&GreaseZone, &mut GreaseIgnited)>,
) {
    let delta = time.delta_secs();
    for (_zone, mut ignited) in &mut zones {
        ignited.fire_spread_time += delta;
    }
}

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

/// Applies burn damage from ignited grease zones.
/// During fire spread, only damages units within the current fire radius.
pub fn apply_grease_burn(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<(&mut GreaseZone, &GreaseIgnited)>,
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
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let delta = time.delta_secs();
    for (mut zone, ignited) in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.ignite_burn_tick {
            zone.time_since_last_tick = 0.0;

            // During spread phase, scope damage to current fire radius from ignition point
            let fire_radius =
                ignited.current_fire_radius(zone.radius, constants::FIRE_SPREAD_DURATION);
            let spreading = ignited.fire_spread_time < constants::FIRE_SPREAD_DURATION;

            let burn_damage =
                zone.ignite_burn_damage * zone.empowerment * zone.talent_params.burn_damage_mult;
            let mut units_burned: u32 = 0;

            for (entity, transform, mut health, mut temp_hp, has_spell_shield) in &mut targets {
                let in_burn_area = if spreading {
                    // Check distance from ignition point during spread
                    xz_distance(ignited.ignition_point, transform.translation) <= fire_radius
                } else {
                    // Fire fully spread — use full zone radius from center
                    xz_distance(zone.origin, transform.translation) <= zone.radius
                };

                if in_burn_area {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        burn_damage,
                        DamageType::Fire,
                        has_spell_shield,
                    );
                    units_burned += 1;
                }
            }

            // Track talent progress for burns
            if units_burned > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::Grease, units_burned);
            }
        }
    }
}

pub fn fade_grease_zone(
    time: Res<Time>,
    mut zones: Query<(
        &GreaseZone,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
        Has<GreaseIgnited>,
        Has<GreaseRegenerating>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();

    for (zone, mut transform, material_handle, is_ignited, is_regenerating) in &mut zones {
        // Grow animation: scale from 0 to full radius over GROW_DURATION.
        // Skip if ignited or regenerating (time_alive may have been reset by talents).
        if !is_ignited && !is_regenerating && zone.time_alive < constants::GROW_DURATION {
            let grow_progress = (zone.time_alive / constants::GROW_DURATION).min(1.0);
            let grow_scale = 1.0 - (1.0 - grow_progress) * (1.0 - grow_progress);
            transform.scale = Vec3::splat(zone.radius * grow_scale);
        } else if transform.scale.x != zone.radius {
            transform.scale = Vec3::splat(zone.radius);
        }

        let remaining = zone.duration - zone.time_alive;
        let fade = if remaining < constants::FADE_DURATION {
            (remaining / constants::FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        // Fade the grease base mesh with iridescent oil-sheen emissive.
        // With Mask mode, alpha < 0.01 makes pixels disappear entirely.
        if let Some(material) = materials.get_mut(material_handle) {
            let (r, g, b, a) = constants::GREASE_COLOR;
            material.base_color = Color::srgba(r, g, b, a * fade);

            // Iridescent sheen: slow cycling through oil-slick rainbow tones
            // Only when not ignited and not regenerating (those have their own visuals)
            if !is_ignited && !is_regenerating {
                let phase = zone.origin.x * 0.01 + zone.origin.z * 0.013;
                let sheen_r = 0.3 + 0.2 * (t * 1.3 + phase).sin();
                let sheen_g = 0.2 + 0.15 * (t * 1.7 + phase * 1.4).sin();
                let sheen_b = 0.25 + 0.2 * (t * 2.1 + phase * 0.7).cos();
                material.emissive = bevy::color::LinearRgba::new(
                    sheen_r * fade,
                    sheen_g * fade,
                    sheen_b * fade,
                    0.0,
                );
            } else if !is_ignited {
                material.emissive = bevy::color::LinearRgba::NONE;
            }
        }
    }
}

/// Cleans up expired grease zones. For Endless Oil, triggers regeneration instead of despawn.
pub fn cleanup_grease_zone(
    mut commands: Commands,
    zones: Query<(Entity, &GreaseZone, Has<GreaseIgnited>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, zone, is_ignited) in &zones {
        if zone.time_alive >= zone.duration {
            if is_ignited {
                // Talent: Endless Oil — regenerate instead of despawning
                if zone.talent_params.endless_oil {
                    // Remove ignited state and start regeneration
                    commands.entity(entity).remove::<GreaseIgnited>();
                    commands.entity(entity).insert(GreaseRegenerating::new());

                    // Downgrade pathfinding from hazard back to slow terrain
                    write_grease_obstacle(
                        zone.origin,
                        zone.radius,
                        ObstacleType::SlowTerrain(3.0),
                        &mut obstacle_events,
                    );
                    continue;
                }

                write_grease_obstacle(
                    zone.origin,
                    zone.radius,
                    ObstacleType::Removed,
                    &mut obstacle_events,
                );
            }
            commands.entity(entity).try_despawn();
        }
    }
}

/// Handles Endless Oil regeneration: ticks the regen timer and restores the zone to slippery state.
pub fn update_grease_regeneration(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<(Entity, &mut GreaseZone, &mut GreaseRegenerating)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    zone_materials: Query<&MeshMaterial3d<StandardMaterial>, With<GreaseZone>>,
) {
    let delta = time.delta_secs();
    for (entity, mut zone, mut regen) in &mut zones {
        regen.time_regenerating += delta;

        // Fade the grease mesh back in as it regenerates.
        let regen_progress =
            (regen.time_regenerating / constants::ENDLESS_OIL_REGEN_DURATION).min(1.0);
        if let Ok(mat_handle) = zone_materials.get(entity)
            && let Some(material) = materials.get_mut(mat_handle)
        {
            let (r, g, b, a) = constants::GREASE_COLOR;
            material.base_color = Color::srgba(r, g, b, a * regen_progress);
        }

        if regen.time_regenerating >= constants::ENDLESS_OIL_REGEN_DURATION {
            // Regeneration complete — restore to slippery state
            zone.time_alive = (zone.duration - constants::ENDLESS_OIL_EXTRA_DURATION).max(0.0);
            zone.time_since_last_tick = 0.0;
            commands.entity(entity).remove::<GreaseRegenerating>();
        }
    }
}

/// Cleans up Oil Slick debuffs and presence trackers when grease zones are despawned.
pub fn cleanup_grease_debuffs(
    mut commands: Commands,
    zones: Query<Entity, With<GreaseZone>>,
    mut tracked: Query<(
        Entity,
        &GreaseZonePresenceTracker,
        Option<&GreaseOilSlickDebuff>,
        Option<&mut Health>,
    )>,
) {
    for (entity, tracker, oil_slick, mut health) in &mut tracked {
        // If the zone this tracker references no longer exists, clean up
        if zones.get(tracker.zone_entity).is_err() {
            commands
                .entity(entity)
                .remove::<GreaseZonePresenceTracker>();
            if let Some(debuff) = oil_slick {
                if let Some(ref mut health) = health {
                    health.spell_vulnerability =
                        (health.spell_vulnerability - debuff.vulnerability).max(0.0);
                }
                commands.entity(entity).remove::<GreaseOilSlickDebuff>();
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_grease_zone(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    talent_params: GreaseTalentParams,
    scorched_mult: f32,
) {
    let duration = constants::ZONE_DURATION * empowerment * scorched_mult;
    let slow_mod = constants::SLOW_MODIFIER * talent_params.slow_mult;
    let slow_dur = constants::SLOW_DURATION * empowerment;

    // Notify pathfinding about slow terrain
    write_grease_obstacle(
        Vec3::new(position.x, 0.0, position.z),
        radius,
        ObstacleType::SlowTerrain(3.0),
        obstacle_events,
    );

    let mut base_mat = materials
        .get(&assets.grease_zone)
        .cloned()
        .unwrap_or_default();
    // Use Mask so the grease renders in the opaque phase (before transparent unit sprites).
    // This writes to the depth buffer at Y=2, ensuring all units above it render on top.
    base_mat.alpha_mode = bevy::render::alpha::AlphaMode::Mask(0.01);
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
        .with_scale(Vec3::splat(0.01)),
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
            talent_params,
        ),
        NetworkedSpellEffect {
            kind: SpellEffectKind::GreaseZone,
        },
        OnGameplayScreen,
    ));
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
