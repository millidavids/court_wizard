use rand::Rng;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard,
};
use super::components::{
    SpikeGrowthLingeringPoison, SpikeGrowthTalentParams, SpikeGrowthZone, SpikeStormProjectile,
    ZonePresenceTracker,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, RootedModifier, SlowMovementModifier, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    self, SpellCircleIndicator, TargetAssistWorldPos, UniqueHitTracker, apply_target_assist,
    build_wizard_input, clamp_cursor_to_spell_range, cleanup_spell_caster, handle_spell_release,
    try_start_cast_with_indicator, update_indicator_position, xz_distance,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> SpikeGrowthTalentParams {
    let mut params = SpikeGrowthTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::SpikeGrowth, 0);
    let t2 = talents.get_selection(Spell::SpikeGrowth, 1);
    let t3 = talents.get_selection(Spell::SpikeGrowth, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Wider Zone
            params.radius_mult = constants::WIDER_ZONE_RADIUS_MULT;
        }
        Some(1) => {
            // Sharper Spikes
            params.damage_mult = constants::SHARPER_SPIKES_DAMAGE_MULT;
        }
        Some(2) => {
            // Quick Bloom
            params.cast_time_mult = constants::QUICK_BLOOM_CAST_TIME_MULT;
            params.mana_mult = constants::QUICK_BLOOM_MANA_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.thorn_maze = true,
        Some(1) => params.poisoned_spikes = true,
        Some(2) => params.quicksand = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.death_garden = true,
        Some(1) => params.minefield = true,
        Some(2) => params.spike_storm = true,
        _ => {}
    }

    params
}

/// Local wizard spike growth casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_spike_growth_casting(
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
    cursor_resources: (
        Res<CorrectedCursorPosition>,
        Res<TargetAssistWorldPos>,
        Res<LocalSpellOrigin>,
    ),
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
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let (active_talents, _talent_progress, active_toggles) = talent_resources;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::SpikeGrowth {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let effective_radius =
        constants::CIRCLE_RADIUS * primed_spell.empowerment * talent_params.radius_mult;
    let effective_mana_cost = constants::MANA_COST * talent_params.mana_mult;
    let effective_cast_time = primed_spell.cast_time * talent_params.cast_time_mult;

    // Clamp cursor to spell range
    let clamped_cursor =
        clamp_cursor_to_spell_range(input.cursor_pos, wizard.spell_range, effective_radius);

    // Handle release — clean up indicator
    if handle_spell_release(
        &input,
        &mut commands,
        wizard_entity,
        &mut casting_state,
        &caster_query,
    ) {
        return;
    }

    let Some(clamped_cursor) = clamped_cursor else {
        return;
    };

    match *casting_state {
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                try_start_cast_with_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.spike_growth_indicator.clone(),
                    wizard_entity,
                    &mut casting_state,
                    &mana,
                    effective_mana_cost,
                    clamped_cursor,
                    effective_radius,
                    &caster_query,
                );
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            // Update indicator position
            update_indicator_position(
                wizard_entity,
                clamped_cursor,
                &caster_query,
                &mut indicator_query,
            );

            if casting_state.is_complete(effective_cast_time) {
                if mana.consume(effective_mana_cost) {
                    vfx::systems::spawn_school_flare(
                        &mut commands,
                        &visual_assets,
                        local_origin.0,
                        vfx::systems::SpellSchool::Nature,
                        time.elapsed_secs(),
                    );
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            audio::play_sfx(
                                &mut commands,
                                &sfx.spike_growth_cast,
                                indicator.position,
                                &game_config,
                                &sfx,
                            );

                            if talent_params.minefield {
                                spawn_minefield_zones(
                                    &mut commands,
                                    &visual_assets,
                                    indicator.position,
                                    effective_radius,
                                    primed_spell.empowerment,
                                    &mut obstacle_events,
                                    &talent_params,
                                    scorched_mult,
                                );
                            } else {
                                spawn_spike_growth_zone(
                                    &mut commands,
                                    &visual_assets,
                                    indicator.position,
                                    effective_radius,
                                    primed_spell.empowerment,
                                    &mut obstacle_events,
                                    &talent_params,
                                    scorched_mult,
                                );
                            }
                        }
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    mouse_state.left_consumed = true;
                } else {
                    cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
            casting_state.cancel();
        }
    }
}

/// Applies periodic damage and slow to ALL units within the spike growth zone.
/// Handles Thorn Maze (enhanced slow), Quicksand (root after threshold),
/// and Poisoned Spikes (zone presence tracking + lingering on exit).
/// Also tracks Death Garden kill extensions.
pub fn apply_spike_growth_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<(Entity, &mut SpikeGrowthZone, &mut UniqueHitTracker)>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Option<&mut SlowMovementModifier>,
        Has<SpellShield>,
        Option<&mut ZonePresenceTracker>,
    )>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let delta = time.delta_secs();

    for (zone_entity, mut zone, mut hit_tracker) in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;

        if zone.time_since_last_tick < zone.tick_interval {
            continue;
        }
        zone.time_since_last_tick = 0.0;

        let effective_radius = zone.effective_radius();
        let needs_tracking = zone.talent_params.quicksand || zone.talent_params.poisoned_spikes;

        let slow_mod = if zone.talent_params.thorn_maze {
            constants::THORN_MAZE_SLOW_MODIFIER
        } else {
            zone.slow_modifier
        };

        let mut units_hit: u32 = 0;

        for (
            entity,
            transform,
            mut health,
            mut temp_hp,
            existing_slow,
            has_spell_shield,
            zone_tracker,
        ) in &mut targets
        {
            let distance = xz_distance(zone.origin, transform.translation);

            if distance <= effective_radius {
                let was_alive = !health.is_dead();

                apply_spell_damage(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    zone.damage_per_tick,
                    DamageType::Poison,
                    has_spell_shield,
                );

                if has_spell_shield {
                    continue;
                }

                // Track unique units hit for talent progress
                if hit_tracker.track_hit(entity) {
                    units_hit += 1;
                }

                // Death Garden: extend duration on kills
                if zone.talent_params.death_garden
                    && was_alive
                    && health.is_dead()
                    && zone.death_garden_extension < constants::DEATH_GARDEN_MAX_EXTENSION
                {
                    zone.death_garden_extension = (zone.death_garden_extension
                        + constants::DEATH_GARDEN_KILL_EXTENSION)
                        .min(constants::DEATH_GARDEN_MAX_EXTENSION);
                }

                // Apply or refresh slow
                if let Some(mut slow) = existing_slow {
                    slow.apply(slow_mod, zone.slow_duration);
                } else {
                    commands
                        .entity(entity)
                        .insert(SlowMovementModifier::new(slow_mod, zone.slow_duration));
                }

                // Zone presence tracking (for Quicksand and/or Poisoned Spikes)
                if needs_tracking {
                    if let Some(mut tracker) = zone_tracker {
                        if tracker.zone_entity == zone_entity {
                            tracker.time_in_zone += zone.tick_interval;
                            // Quicksand: root after threshold
                            if zone.talent_params.quicksand
                                && !tracker.rooted
                                && tracker.time_in_zone >= constants::QUICKSAND_TIME_THRESHOLD
                            {
                                tracker.rooted = true;
                                commands.entity(entity).insert(RootedModifier::new(
                                    constants::QUICKSAND_ROOT_DURATION,
                                ));
                            }
                        }
                    } else {
                        commands.entity(entity).insert(ZonePresenceTracker {
                            zone_entity,
                            time_in_zone: 0.0,
                            rooted: false,
                        });
                    }
                }
            } else if needs_tracking
                && let Some(ref tracker) = zone_tracker
                && tracker.zone_entity == zone_entity
            {
                // Unit is outside the zone — handle exit
                commands.entity(entity).remove::<ZonePresenceTracker>();
                // Poisoned Spikes: apply lingering poison on exit
                if zone.talent_params.poisoned_spikes {
                    commands
                        .entity(entity)
                        .insert(SpikeGrowthLingeringPoison::new());
                }
            }
        }

        // Track talent progress
        if units_hit > 0
            && let Some(ref mut progress) = talent_progress
        {
            progress.increment(Spell::SpikeGrowth, units_hit);
        }
    }
}

/// Ticks lingering poison effect and applies damage.
pub fn tick_lingering_poison(
    mut commands: Commands,
    time: Res<Time>,
    mut targets: Query<(
        Entity,
        &mut SpikeGrowthLingeringPoison,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) {
    let delta = time.delta_secs();

    for (entity, mut poison, mut health, mut temp_hp, has_spell_shield) in &mut targets {
        poison.time_remaining -= delta;
        poison.time_since_last_tick += delta;

        if poison.time_remaining <= 0.0 {
            commands
                .entity(entity)
                .remove::<SpikeGrowthLingeringPoison>();
            continue;
        }

        if poison.time_since_last_tick >= poison.tick_interval {
            poison.time_since_last_tick = 0.0;
            apply_spell_damage(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                poison.damage_per_tick,
                DamageType::Poison,
                has_spell_shield,
            );
        }
    }
}

/// Death Garden: grows zone radius over time and updates pathfinding obstacles.
pub fn update_death_garden(
    time: Res<Time>,
    mut zones: Query<&mut SpikeGrowthZone>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    for mut zone in &mut zones {
        if !zone.talent_params.death_garden {
            continue;
        }

        let new_radius = zone.effective_radius();

        // Throttle obstacle updates to every 1 second
        zone.death_garden_obstacle_timer += delta;
        if zone.death_garden_obstacle_timer >= 1.0 {
            zone.death_garden_obstacle_timer = 0.0;
            let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
            let buffered_radius = new_radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                obstacle_type: ObstacleType::Hazard(15.0),
                shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
                rebuild: false,
            });
        }
    }
}

/// Spike Storm: launches projectiles at nearby enemies periodically.
pub fn spike_storm_volley(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<&mut SpikeGrowthZone>,
    targets: Query<(Entity, &Transform, &Health), Without<SpikeGrowthZone>>,
    visual_assets: Res<SpellVisualAssets>,
) {
    let delta = time.delta_secs();

    for mut zone in &mut zones {
        if !zone.talent_params.spike_storm {
            continue;
        }

        zone.spike_storm_timer += delta;
        if zone.spike_storm_timer < constants::SPIKE_STORM_INTERVAL {
            continue;
        }
        zone.spike_storm_timer = 0.0;

        let targeting_range = zone.effective_radius() * constants::SPIKE_STORM_RANGE_MULT;

        // Find nearest enemies
        let mut candidates: Vec<(Vec3, f32)> = targets
            .iter()
            .filter(|(_, _, health)| !health.is_dead())
            .filter_map(|(_, transform, _)| {
                let dist = xz_distance(zone.origin, transform.translation);
                if dist <= targeting_range {
                    Some((transform.translation, dist))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(constants::SPIKE_STORM_MAX_TARGETS);

        let Some(ref material) = zone.spike_storm_material else {
            continue;
        };

        for (target_pos, _) in &candidates {
            let spawn_pos = Vec3::new(zone.origin.x, 3.0, zone.origin.z);
            let direction = Vec3::new(target_pos.x - spawn_pos.x, 0.0, target_pos.z - spawn_pos.z);
            let direction = if direction.length_squared() > 0.001 {
                direction.normalize()
            } else {
                Vec3::X
            };

            let max_lifetime = targeting_range / constants::SPIKE_STORM_PROJECTILE_SPEED + 0.5;

            commands.spawn((
                Mesh3d(visual_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(spawn_pos)
                    .with_scale(Vec3::splat(constants::SPIKE_STORM_PROJECTILE_SCALE)),
                SpikeStormProjectile {
                    direction,
                    speed: constants::SPIKE_STORM_PROJECTILE_SPEED,
                    damage: constants::SPIKE_STORM_DAMAGE,
                    radius: constants::SPIKE_STORM_PROJECTILE_RADIUS,
                    time_alive: 0.0,
                    max_lifetime,
                },
                OnGameplayScreen,
            ));
        }
    }
}

/// Updates spike storm projectile positions and checks for collisions.
pub fn update_spike_storm_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Transform, &mut SpikeStormProjectile)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<SpikeStormProjectile>,
    >,
) {
    let delta = time.delta_secs();

    for (proj_entity, mut proj_transform, mut projectile) in &mut projectiles {
        projectile.time_alive += delta;

        if projectile.time_alive >= projectile.max_lifetime {
            commands.entity(proj_entity).try_despawn();
            continue;
        }

        proj_transform.translation += projectile.direction * projectile.speed * delta;

        let mut hit = false;
        for (target_entity, target_transform, mut health, mut temp_hp, has_spell_shield) in
            &mut targets
        {
            if health.is_dead() {
                continue;
            }
            let dist = xz_distance(proj_transform.translation, target_transform.translation);

            if dist <= projectile.radius + constants::UNIT_COLLISION_RADIUS {
                apply_spell_damage(
                    &mut commands,
                    target_entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    projectile.damage,
                    DamageType::Poison,
                    has_spell_shield,
                );
                hit = true;
                break;
            }
        }

        if hit {
            commands.entity(proj_entity).try_despawn();
        }
    }
}

/// Spawns animated green vine rings and red spike rings from active spike growth zones.
pub fn emit_spike_growth_rings(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut zones: Query<&mut SpikeGrowthZone>,
) {
    let delta = time.delta_secs();
    for mut zone in &mut zones {
        if zone.time_alive >= zone.effective_duration() {
            continue;
        }

        zone.ring_timer += delta;
        if zone.ring_timer < utils::RING_SPAWN_INTERVAL {
            continue;
        }
        zone.ring_timer -= utils::RING_SPAWN_INTERVAL;

        // Alternate between green vine rings and red spike rings
        let material = if game_rng.0.random::<f32>() < 0.35 {
            visual_assets.spike_growth_spike.clone()
        } else {
            visual_assets.spike_growth_vine.clone()
        };

        utils::spawn_ring_particle(
            &mut game_rng.0,
            &mut commands,
            visual_assets.entangle_vine_ring.clone(),
            material,
            zone.origin,
            zone.effective_radius(),
        );
    }
}

/// Despawns expired spike growth zones.
pub fn cleanup_spike_growth_zone(
    mut commands: Commands,
    zones: Query<(Entity, &SpikeGrowthZone)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.effective_duration() {
            let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
            let buffered_radius = zone.effective_radius() + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
                rebuild: false,
            });
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns a single spike growth zone with talent parameters.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_spike_growth_zone(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    talent_params: &SpikeGrowthTalentParams,
    scorched_mult: f32,
) {
    let duration = constants::ZONE_DURATION * empowerment * scorched_mult;
    let damage = constants::DAMAGE_PER_TICK * empowerment * talent_params.damage_mult;
    let slow_mod = constants::SLOW_MODIFIER * empowerment;
    let slow_dur = constants::SLOW_DURATION * empowerment;

    // Notify pathfinding about hazard zone
    let origin_2d = Vec2::new(position.x, position.z);
    let buffered_radius = radius + OBSTACLE_BUFFER;
    let hazard_cost = if talent_params.thorn_maze {
        15.0 * constants::THORN_MAZE_HAZARD_MULT
    } else {
        15.0
    };
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type: ObstacleType::Hazard(hazard_cost),
        shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
        rebuild: false,
    });

    // Use pre-loaded spike storm material from visual assets
    let spike_storm_material = if talent_params.spike_storm {
        Some(assets.spike_storm_projectile.clone())
    } else {
        None
    };

    let mut zone = SpikeGrowthZone::new(
        Vec3::new(position.x, 0.0, position.z),
        radius,
        damage,
        constants::TICK_INTERVAL,
        slow_mod,
        slow_dur,
        duration,
        *talent_params,
    );
    zone.spike_storm_material = spike_storm_material;

    commands.spawn((
        Transform::from_translation(Vec3::new(
            position.x,
            constants::CIRCLE_Y_POSITION,
            position.z,
        )),
        zone,
        UniqueHitTracker::default(),
        NetworkedSpellEffect {
            kind: SpellEffectKind::SpikeGrowthZone,
        },
        OnGameplayScreen,
    ));
}

/// Nature's Minefield: spawns 3 smaller zones in a triangle pattern.
#[allow(clippy::too_many_arguments)]
fn spawn_minefield_zones(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    base_radius: f32,
    empowerment: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    talent_params: &SpikeGrowthTalentParams,
    scorched_mult: f32,
) {
    let sub_radius = base_radius * constants::MINEFIELD_RADIUS_MULT;
    let spread = base_radius * constants::MINEFIELD_SPREAD_FRACTION;

    // Triangle offsets: 120° apart
    let offsets = [
        Vec3::new(0.0, 0.0, -spread),
        Vec3::new(spread * 0.866, 0.0, spread * 0.5),
        Vec3::new(-spread * 0.866, 0.0, spread * 0.5),
    ];

    for offset in &offsets {
        let sub_position = position + *offset;
        spawn_spike_growth_zone(
            commands,
            assets,
            sub_position,
            sub_radius,
            empowerment,
            obstacle_events,
            talent_params,
            scorched_mult,
        );
    }
}
