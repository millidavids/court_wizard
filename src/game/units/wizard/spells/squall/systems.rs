//! Squall spell systems.

use bevy::prelude::*;
use rand::Rng;

use super::components::{
    AbsoluteZeroSlow, FrozenGround, IceExplosion, IceProjectile, PermafrostTracker,
    SnowParticle, SquallStorm, SquallStormRing, SquallTalentParams,
};
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::{Billboard, ConcentrationSpell, OnGameplayScreen};
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::DamageType;
use crate::game::units::components::{
    FogEvasionModifier, FrostEffectMarker, FrozenSolidModifier, Health, SlowMovementModifier,
    Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::utils::{
    RETICLE_Y, SpellCircleIndicator, clamp_to_spell_range_ground, get_cursor_world_position,
    indicator_pulse_scale, make_reticle_mesh, spawn_circle_indicator, xz_distance,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;

/// Applies or inserts a [`SlowMovementModifier`] on an entity.
fn apply_or_insert_slow(
    commands: &mut Commands,
    entity: Entity,
    existing: Option<Mut<SlowMovementModifier>>,
    modifier: f32,
    duration: f32,
) {
    if let Some(mut slow) = existing {
        slow.apply(modifier, duration);
    } else {
        commands
            .entity(entity)
            .insert(SlowMovementModifier::new(modifier, duration));
    }
}

/// Applies or inserts a [`FrostEffectMarker`] on an entity.
fn apply_or_insert_frost(
    commands: &mut Commands,
    entity: Entity,
    existing: Option<Mut<FrostEffectMarker>>,
    duration: f32,
) {
    if let Some(mut frost) = existing {
        frost.apply(duration);
    } else {
        commands
            .entity(entity)
            .insert(FrostEffectMarker::new(duration));
    }
}

/// Despawns all storm ring reticles.
fn despawn_storm_rings(commands: &mut Commands, rings: &Query<Entity, With<SquallStormRing>>) {
    for ring in rings.iter() {
        commands.entity(ring).try_despawn();
    }
}

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> SquallTalentParams {
    let mut params = SquallTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Squall, 0);
    let t2 = talents.get_selection(Spell::Squall, 1);
    let t3 = talents.get_selection(Spell::Squall, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Bitter Cold: +30% damage
            params.damage_mult = BITTER_COLD_DAMAGE_MULT;
        }
        Some(1) => {
            // Howling Winds: +30% radius
            params.radius_mult = HOWLING_WINDS_RADIUS_MULT;
        }
        Some(2) => {
            // Freezing Rain: faster spawn, less damage per shard
            params.spawn_rate_mult = FREEZING_RAIN_SPAWN_MULT;
            params.damage_mult = FREEZING_RAIN_DAMAGE_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.permafrost = true,
        Some(1) => params.hailstones = true,
        Some(2) => params.sleet_storm = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.absolute_zero = true,
        Some(1) => params.blizzard = true,
        Some(2) => params.ice_age = true,
        _ => {}
    }

    params
}

/// Local wizard squall casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_squall_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    existing_storms: Query<Entity, With<SquallStorm>>,
    existing_rings: Query<Entity, With<SquallStormRing>>,
    active_talents: Option<Res<ActiveTalents>>,
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
    if primed_spell.spell != Spell::Squall {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = squall_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &existing_storms,
        &existing_rings,
        &mut commands,
        &mut meshes,
        &visual_assets,
        &talent_params,
    );

    if completed {
        vfx::systems::spawn_school_flare(&mut commands, &visual_assets, vfx::systems::SpellSchool::Force, time.elapsed_secs());
        mouse_state.left_consumed = true;
    }
}

/// Core squall casting logic.
#[allow(clippy::too_many_arguments)]
fn squall_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    existing_storms: &Query<Entity, With<SquallStorm>>,
    existing_rings: &Query<Entity, With<SquallStormRing>>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &SpellVisualAssets,
    talent_params: &SquallTalentParams,
) -> bool {
    let mut completed = false;

    // Check for release event - cancel cast
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

    // Get cursor world position and clamp to wizard's spell range
    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = SPELL_ORIGIN;
    let scale = primed_spell.empowerment;
    let storm_radius = STORM_RADIUS * scale * talent_params.radius_mult;

    cursor_world_pos = clamp_to_spell_range_ground(
        cursor_world_pos,
        wizard_pos,
        wizard.spell_range,
        storm_radius,
    );

    // If Absolute Zero storm is already active, don't start a new cast
    if talent_params.absolute_zero && !existing_storms.is_empty() {
        return false;
    }

    // Handle casting based on state
    match *casting_state {
        CastingState::Resting => {
            // Absolute Zero: no initial mana cost
            let can_afford = talent_params.absolute_zero || mana.can_afford(MANA_COST);
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && can_afford
            {
                let circle_entity = spawn_circle_indicator(
                    commands,
                    meshes,
                    assets.squall_indicator.clone(),
                    cursor_world_pos,
                    storm_radius,
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

            if let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = cursor_world_pos;
            }

            if casting_state.is_complete(primed_spell.cast_time) {
                // Absolute Zero: no mana cost on cast completion
                let mana_ok = talent_params.absolute_zero || mana.consume(MANA_COST);
                if mana_ok {
                    // Despawn any existing storms and their rings (only one storm at a time)
                    for existing_storm in existing_storms.iter() {
                        commands.entity(existing_storm).try_despawn();
                    }
                    despawn_storm_rings(commands, existing_rings);

                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let mut storm_entity = commands.spawn((
                                SquallStorm::new(
                                    indicator.position,
                                    storm_radius,
                                    primed_spell.empowerment,
                                    *talent_params,
                                ),
                                OnGameplayScreen,
                            ));

                            // Absolute Zero: channeled, no concentration marker
                            if !talent_params.absolute_zero {
                                storm_entity.insert(ConcentrationSpell {
                                    spell_name: "Squall",
                                });
                            }

                            // Add permafrost tracker if talent is active
                            if talent_params.permafrost {
                                storm_entity.insert(PermafrostTracker::default());
                            }

                            // Spawn persistent annulus ring reticle for the storm
                            let ring_mesh = meshes.add(make_reticle_mesh(storm_radius));
                            commands.spawn((
                                Mesh3d(ring_mesh),
                                MeshMaterial3d(assets.squall_indicator.clone()),
                                Transform::from_translation(Vec3::new(
                                    indicator.position.x,
                                    RETICLE_Y,
                                    indicator.position.z,
                                ))
                                .with_rotation(Quat::from_rotation_x(
                                    -std::f32::consts::FRAC_PI_2,
                                ))
                                .with_scale(Vec3::splat(storm_radius)),
                                SquallStormRing { time_alive: 0.0 },
                                OnGameplayScreen,
                            ));
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

/// Spawns ice projectiles periodically from active storms.
///
/// Projectiles spawn at random positions within the storm radius, high above the battlefield.
pub(super) fn spawn_ice_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut storms: Query<&mut SquallStorm>,
) {
    let mut rng = rand::thread_rng();

    for mut storm in storms.iter_mut() {
        storm.update_timers(time.delta_secs());

        // Apply spawn rate talent modifier
        let spawn_interval = ICE_SPAWN_INTERVAL * storm.talent_params.spawn_rate_mult;

        // Check if it's time to spawn another projectile
        if storm.time_since_spawn >= spawn_interval {
            storm.reset_spawn_timer();

            // Random position within storm circle (on XZ plane)
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..storm.radius);
            let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let spawn_pos = Vec3::new(
                storm.position.x + offset.x,
                ICE_SPAWN_HEIGHT,
                storm.position.z + offset.z,
            );

            // Determine if this is a hailstone (Tier 2 talent)
            let is_hailstone =
                storm.talent_params.hailstones && rng.gen_range(0.0..1.0) < HAILSTONE_CHANCE;

            // Calculate damage with talent modifiers
            let base_damage = FROST_DAMAGE * storm.empowerment * storm.talent_params.damage_mult;
            let damage = if is_hailstone {
                base_damage * HAILSTONE_DAMAGE_MULT
            } else {
                base_damage
            };
            let explosion_radius = EXPLOSION_RADIUS * storm.empowerment;
            let mesh_scale = if is_hailstone {
                ICE_PROJECTILE_MESH_RADIUS * HAILSTONE_MESH_SCALE
            } else {
                ICE_PROJECTILE_MESH_RADIUS
            };

            commands.spawn((
                IceProjectile::new(
                    Vec3::new(0.0, ICE_INITIAL_VELOCITY, 0.0),
                    damage,
                    explosion_radius,
                    ICE_PROJECTILE_RADIUS,
                    storm.empowerment,
                    is_hailstone,
                    storm.talent_params.ice_age,
                ),
                Mesh3d(visual_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(visual_assets.ice_projectile.clone()),
                Transform::from_translation(spawn_pos).with_scale(Vec3::splat(mesh_scale)),
                OnGameplayScreen,
            ));
        }
    }
}

/// Updates ice projectile physics - applies gravity and moves projectiles.
pub(super) fn update_ice_projectiles(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &mut IceProjectile)>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut projectile) in projectiles.iter_mut() {
        // Apply gravity
        projectile.velocity.y += ICE_GRAVITY * delta;

        // Move projectile
        transform.translation += projectile.velocity * delta;
    }
}

/// Checks for ice projectile collisions with ground or walls, spawns explosions.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_ice_projectile_collisions(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    projectiles: Query<(Entity, &Transform, &IceProjectile)>,
    walls: Query<&WallOfStone>,
    rocks: Query<&crate::game::units::thrown_rock::components::ThrownRock>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    for (entity, transform, projectile) in projectiles.iter() {
        let projectile_pos = transform.translation;

        // Check wall collision
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(projectile_pos) && projectile_pos.y <= wall.height {
                // Hit wall - spawn explosion at wall surface
                let explosion_pos = Vec3::new(projectile_pos.x, wall.height, projectile_pos.z);
                spawn_ice_explosion(
                    &mut commands,
                    &visual_assets,
                    explosion_pos,
                    projectile.explosion_radius,
                    projectile.damage,
                    projectile.empowerment,
                );
                let sfx_scale = if projectile.is_hailstone { 0.5 } else { 0.3 };
                audio::play_impact_sfx_scaled(
                    &mut commands,
                    &sfx.squall_impact,
                    explosion_pos,
                    &game_config,
                    &sfx,
                    sfx_scale,
                );
                commands.entity(entity).try_despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Check collision with rocks
        let mut hit_rock = false;
        for rock in &rocks {
            if rock.blocks_projectile(projectile_pos) {
                let explosion_pos = Vec3::new(projectile_pos.x, rock.height, projectile_pos.z);
                spawn_ice_explosion(
                    &mut commands,
                    &visual_assets,
                    explosion_pos,
                    projectile.explosion_radius,
                    projectile.damage,
                    projectile.empowerment,
                );
                let sfx_scale = if projectile.is_hailstone { 0.5 } else { 0.3 };
                audio::play_impact_sfx_scaled(
                    &mut commands,
                    &sfx.squall_impact,
                    explosion_pos,
                    &game_config,
                    &sfx,
                    sfx_scale,
                );
                commands.entity(entity).try_despawn();
                hit_rock = true;
                break;
            }
        }
        if hit_rock {
            continue;
        }

        // Check ground collision (Y <= 0)
        if projectile_pos.y <= 0.0 {
            // Hit ground - spawn explosion at ground level
            let explosion_pos = Vec3::new(projectile_pos.x, 0.0, projectile_pos.z);
            spawn_ice_explosion(
                &mut commands,
                &visual_assets,
                explosion_pos,
                projectile.explosion_radius,
                projectile.damage,
                projectile.empowerment,
            );
            // Ice Age: spawn frozen ground at impact point
            if projectile.ice_age {
                spawn_frozen_ground_patch(
                    &mut commands,
                    &visual_assets,
                    explosion_pos,
                    projectile.empowerment,
                );
            }
            let sfx_scale = if projectile.is_hailstone { 0.5 } else { 0.3 };
            audio::play_impact_sfx_scaled(
                &mut commands,
                &sfx.squall_impact,
                explosion_pos,
                &game_config,
                &sfx,
                sfx_scale,
            );
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns an ice explosion at the given position.
fn spawn_ice_explosion(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    max_radius: f32,
    damage: f32,
    empowerment: f32,
) {
    // Position slightly above battlefield (y=1) to avoid z-fighting
    let explosion_pos = Vec3::new(position.x, 1.0, position.z);

    commands.spawn((
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.ice_explosion.clone()),
        Transform::from_translation(explosion_pos)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(0.1)),
        IceExplosion::new(position, max_radius, damage, empowerment),
        NetworkedSpellEffect {
            kind: SpellEffectKind::IceExplosion,
        },
        OnGameplayScreen,
    ));
}

/// Updates explosion visuals, applies damage, and tracks Permafrost hits.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_ice_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut IceExplosion, &mut Transform)>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Option<&mut SlowMovementModifier>,
            Option<&mut FrostEffectMarker>,
        ),
        Without<IceExplosion>,
    >,
    mut permafrost_trackers: Query<&mut PermafrostTracker>,
    storms: Query<&SquallStorm>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    // Get the active storm's talent params for permafrost tracking
    let storm_has_permafrost = storms
        .iter()
        .next()
        .map(|s| s.talent_params.permafrost)
        .unwrap_or(false);

    for (explosion_entity, mut explosion, mut transform) in explosions.iter_mut() {
        explosion.time_alive += time.delta_secs();

        // Update visual scale (growth animation)
        let current_radius = explosion.current_radius(EXPLOSION_GROWTH_TIME);
        transform.scale = Vec3::splat(current_radius);

        // Apply damage once when explosion spawns
        if !explosion.damage_applied {
            explosion.damage_applied = true;
            let mut units_hit: u32 = 0;

            for (
                unit_entity,
                unit_transform,
                team,
                mut health,
                mut temp_hp,
                has_spell_shield,
                slow_mod,
                frost_marker,
            ) in units.iter_mut()
            {
                if *team == Team::Defenders {
                    continue;
                }

                let distance = unit_transform.translation.distance(explosion.origin);

                if distance <= explosion.max_radius {
                    apply_spell_damage(
                        &mut commands,
                        unit_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage,
                        DamageType::Frost,
                        has_spell_shield,
                    );
                    units_hit += 1;

                    // Apply frost slow + visual
                    apply_or_insert_slow(&mut commands, unit_entity, slow_mod, FROST_SLOW_MODIFIER, FROST_SLOW_DURATION);
                    apply_or_insert_frost(&mut commands, unit_entity, frost_marker, FROST_SLOW_DURATION);

                    // Track permafrost hits
                    if storm_has_permafrost {
                        for mut tracker in permafrost_trackers.iter_mut() {
                            let count = tracker.hit_counts.entry(unit_entity).or_insert(0);
                            *count += 1;
                            if *count >= PERMAFROST_HIT_THRESHOLD {
                                // Freeze solid!
                                commands.entity(unit_entity).insert((
                                    FrozenSolidModifier::new(PERMAFROST_FREEZE_DURATION),
                                    FrostEffectMarker::new(PERMAFROST_FREEZE_DURATION),
                                ));
                                // Reset counter so they can be frozen again
                                *count = 0;
                            }
                        }
                    }
                }
            }

            // Track talent progress
            if units_hit > 0 {
                if let Some(ref mut progress) = talent_progress {
                    progress.increment(Spell::Squall, units_hit);
                }
            }
        }

        // Despawn explosion after lifetime
        if explosion.time_alive >= EXPLOSION_LIFETIME {
            commands.entity(explosion_entity).try_despawn();
        }
    }
}

/// Applies Sleet Storm evasion debuff to enemies inside the storm radius.
pub(super) fn apply_sleet_storm_evasion(
    storms: Query<&SquallStorm>,
    mut units: Query<(Entity, &Transform, &Team, Option<&mut FogEvasionModifier>), With<Health>>,
    mut commands: Commands,
) {
    for storm in storms.iter() {
        if !storm.talent_params.sleet_storm {
            continue;
        }

        for (entity, unit_transform, team, fog_evasion) in units.iter_mut() {
            if *team == Team::Defenders {
                continue;
            }

            let distance = xz_distance(unit_transform.translation, storm.position);

            if distance <= storm.radius {
                if let Some(mut evasion) = fog_evasion {
                    evasion.refresh(SLEET_STORM_EVASION_DURATION);
                } else {
                    commands.entity(entity).insert(FogEvasionModifier::new(
                        SLEET_STORM_EVASION_CHANCE,
                        SLEET_STORM_EVASION_DURATION,
                    ));
                }
            }
        }
    }
}

/// Handles Absolute Zero: continuously drains mana, applies stacking slow + damage to units in storm.
pub(super) fn update_absolute_zero(
    time: Res<Time>,
    storms: Query<(Entity, &SquallStorm)>,
    rings: Query<Entity, With<SquallStormRing>>,
    mut wizard_query: Query<&mut Mana, With<LocalWizard>>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut AbsoluteZeroSlow>,
            Option<&mut SlowMovementModifier>,
            Option<&mut FrostEffectMarker>,
        ),
    >,
    mut commands: Commands,
) {
    let delta = time.delta_secs();

    for (storm_entity, storm) in storms.iter() {
        if !storm.talent_params.absolute_zero {
            continue;
        }

        // Drain mana continuously
        let Ok(mut mana) = wizard_query.single_mut() else {
            continue;
        };
        let mana_cost = ABSOLUTE_ZERO_MANA_PER_SEC * delta;
        if !mana.consume(mana_cost) {
            // Out of mana — end the channeled storm and its ring
            commands.entity(storm_entity).try_despawn();
            despawn_storm_rings(&mut commands, &rings);
            continue;
        }

        let damage_this_frame = ABSOLUTE_ZERO_DPS * delta;

        for (entity, unit_transform, team, mut health, az_slow, slow_mod, frost_marker) in
            units.iter_mut()
        {
            if *team == Team::Defenders {
                continue;
            }

            let distance = xz_distance(unit_transform.translation, storm.position);

            if distance <= storm.radius {
                // Apply damage
                health.take_damage(damage_this_frame);

                // Stack slow
                if let Some(mut az) = az_slow {
                    az.accumulated_slow =
                        (az.accumulated_slow - ABSOLUTE_ZERO_SLOW_PER_FRAME).max(-ABSOLUTE_ZERO_MAX_SLOW);
                    az.decay_timer = ABSOLUTE_ZERO_SLOW_DECAY_TIME;

                    // Update the movement modifier to match
                    apply_or_insert_slow(&mut commands, entity, slow_mod, az.accumulated_slow, ABSOLUTE_ZERO_SLOW_DECAY_TIME);
                } else {
                    // First frame in zone — create tracker
                    commands.entity(entity).insert(AbsoluteZeroSlow {
                        accumulated_slow: -ABSOLUTE_ZERO_SLOW_PER_FRAME,
                        decay_timer: ABSOLUTE_ZERO_SLOW_DECAY_TIME,
                    });
                    apply_or_insert_slow(&mut commands, entity, slow_mod, -ABSOLUTE_ZERO_SLOW_PER_FRAME, ABSOLUTE_ZERO_SLOW_DECAY_TIME);
                }

                // Apply frost visual
                apply_or_insert_frost(&mut commands, entity, frost_marker, ABSOLUTE_ZERO_SLOW_DECAY_TIME);
            }
        }
    }
}

/// Decays and cleans up Absolute Zero slow when units leave the zone or channeling stops.
pub(super) fn decay_absolute_zero_slow(
    time: Res<Time>,
    storms: Query<&SquallStorm>,
    mut units: Query<(Entity, &Transform, &mut AbsoluteZeroSlow)>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();

    // Check if any active storm has absolute zero
    let has_active_az = storms
        .iter()
        .any(|s| s.talent_params.absolute_zero);

    for (entity, unit_transform, mut az) in units.iter_mut() {
        // Check if unit is currently inside an active AZ storm
        let mut in_zone = false;
        if has_active_az {
            for storm in storms.iter() {
                if !storm.talent_params.absolute_zero {
                    continue;
                }
                let distance = xz_distance(unit_transform.translation, storm.position);
                if distance <= storm.radius {
                    in_zone = true;
                    break;
                }
            }
        }

        // Only decay if NOT in the zone (or no active AZ storm exists)
        if !in_zone {
            az.decay_timer -= delta;
            if az.decay_timer <= 0.0 {
                commands.entity(entity).remove::<AbsoluteZeroSlow>();
            }
        }
    }
}

/// Handles Blizzard talent: storm follows cursor slowly.
pub(super) fn update_blizzard_position(
    time: Res<Time>,
    mut storms: Query<&mut SquallStorm>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    wizard_query: Query<&Wizard, With<LocalWizard>>,
) {
    let Some(cursor_pos) = get_cursor_world_position(&camera_query, &corrected_cursor) else {
        return;
    };

    let Ok(wizard) = wizard_query.single() else {
        return;
    };

    for mut storm in storms.iter_mut() {
        // Both Blizzard and Absolute Zero make the storm follow the cursor
        if !storm.talent_params.blizzard && !storm.talent_params.absolute_zero {
            continue;
        }

        // Clamp target to spell range
        let target = clamp_to_spell_range_ground(
            cursor_pos,
            SPELL_ORIGIN,
            wizard.spell_range,
            storm.radius,
        );

        // Lerp storm position toward cursor
        let direction = Vec3::new(target.x - storm.position.x, 0.0, target.z - storm.position.z);
        let distance = direction.length();

        if distance > 1.0 {
            let move_amount = BLIZZARD_FOLLOW_SPEED * time.delta_secs();
            let move_vec = direction.normalize() * move_amount.min(distance);
            storm.position += move_vec;
        }
    }
}

/// Ends the Absolute Zero channeled storm when the mouse is released.
pub(super) fn end_absolute_zero_on_release(
    mut mouse_released: MessageReader<MouseLeftReleased>,
    storms: Query<(Entity, &SquallStorm)>,
    rings: Query<Entity, With<SquallStormRing>>,
    mut commands: Commands,
) {
    if mouse_released.read().next().is_none() {
        return;
    }

    for (entity, storm) in storms.iter() {
        if storm.talent_params.absolute_zero {
            commands.entity(entity).try_despawn();
            despawn_storm_rings(&mut commands, &rings);
        }
    }
}

/// Spawns a frozen ground patch at an impact point (Ice Age talent).
fn spawn_frozen_ground_patch(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
) {
    let patch_pos = Vec3::new(position.x, 0.05, position.z);
    let patch_radius = ICE_AGE_PATCH_RADIUS * empowerment;

    commands.spawn((
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.ice_explosion.clone()),
        Transform::from_translation(patch_pos)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(patch_radius)),
        FrozenGround::new(patch_pos, patch_radius, ICE_AGE_GROUND_DURATION),
        OnGameplayScreen,
    ));
}

/// Updates the storm ring reticle: syncs position with the storm, pulse animation,
/// and despawns the ring when the storm is gone (concentration ended or AZ released).
pub(super) fn update_storm_ring(
    time: Res<Time>,
    storms: Query<&SquallStorm>,
    mut rings: Query<(Entity, &mut SquallStormRing, &mut Transform)>,
    mut commands: Commands,
) {
    let storm = storms.iter().next();

    for (entity, mut ring, mut transform) in rings.iter_mut() {
        let Some(storm) = storm else {
            // No storm — despawn orphaned ring
            commands.entity(entity).try_despawn();
            continue;
        };

        ring.time_alive += time.delta_secs();
        let pulse = indicator_pulse_scale(ring.time_alive);
        transform.translation.x = storm.position.x;
        transform.translation.z = storm.position.z;
        transform.scale = Vec3::splat(storm.radius * pulse);
    }
}

/// Updates frozen ground patches: applies slow to enemies walking over them.
pub(super) fn update_frozen_ground(
    time: Res<Time>,
    mut commands: Commands,
    mut patches: Query<(Entity, &mut FrozenGround)>,
    mut units: Query<(Entity, &Transform, &Team, Option<&mut SlowMovementModifier>), With<Health>>,
) {
    for (patch_entity, mut patch) in patches.iter_mut() {
        patch.time_remaining -= time.delta_secs();

        if patch.time_remaining <= 0.0 {
            commands.entity(patch_entity).try_despawn();
            continue;
        }

        // Apply slow to enemies inside the patch
        for (unit_entity, unit_transform, team, slow_mod) in units.iter_mut() {
            if *team == Team::Defenders {
                continue;
            }

            let distance = xz_distance(unit_transform.translation, patch.position);

            if distance <= patch.radius {
                apply_or_insert_slow(&mut commands, unit_entity, slow_mod, ICE_AGE_SLOW_MODIFIER, ICE_AGE_SLOW_DURATION);
            }
        }
    }
}

/// Spawns swirling snow particles within active storm areas.
pub(super) fn spawn_snow_particles(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    storms: Query<&SquallStorm>,
) {
    let mut rng = rand::thread_rng();
    let time_secs = time.elapsed_secs();

    for storm in storms.iter() {
        // Check spawn interval using elapsed time
        let interval = SNOW_SPAWN_INTERVAL;
        let spawn_check = (time_secs / interval) as u32;
        let prev_check = ((time_secs - time.delta_secs()) / interval) as u32;
        if spawn_check == prev_check {
            continue;
        }

        for i in 0..SNOW_BATCH_SIZE {
            let seed = time_secs * 7.1 + i as f32 * 1.618_034;

            // Random position within storm radius
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..storm.radius);
            let height = rng.gen_range(SNOW_MIN_HEIGHT..SNOW_MAX_HEIGHT);

            let spawn_pos = Vec3::new(
                storm.position.x + angle.cos() * distance,
                height,
                storm.position.z + angle.sin() * distance,
            );

            // Tangential velocity for swirling motion
            let swirl_angle = angle + std::f32::consts::FRAC_PI_2;
            let velocity = Vec3::new(
                swirl_angle.cos() * SNOW_SWIRL_SPEED,
                -SNOW_DRIFT_SPEED,
                swirl_angle.sin() * SNOW_SWIRL_SPEED,
            );

            let phase = seed * std::f32::consts::PI + (seed * 41.7).sin();
            let lifetime = SNOW_LIFETIME * rng.gen_range(0.7..1.3);
            let base_size = SNOW_BASE_SIZE * rng.gen_range(0.5..1.5);

            commands.spawn((
                SnowParticle {
                    velocity,
                    time_alive: 0.0,
                    lifetime,
                    base_size,
                    phase,
                },
                Mesh3d(visual_assets.particle_quad.clone()),
                MeshMaterial3d(visual_assets.snow_particle.clone()),
                Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.1)),
                Billboard,
                OnGameplayScreen,
            ));
        }
    }
}

/// Updates snow particles: swirling motion, sway, and fade in/out via scale.
pub(super) fn update_snow_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut SnowParticle, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut snow, mut transform) in particles.iter_mut() {
        snow.time_alive += dt;

        if snow.time_alive >= snow.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Move with velocity (swirl + drift down)
        transform.translation += snow.velocity * dt;

        // Lateral sway
        let t = snow.time_alive;
        let sway = (t * SNOW_SWAY_FREQUENCY * std::f32::consts::TAU + snow.phase).sin()
            * SNOW_SWAY_AMPLITUDE
            * dt;
        transform.translation.x += sway;

        // Fade in/out via scale
        let life_frac = snow.time_alive / snow.lifetime;
        let alpha = if life_frac < 0.15 {
            // Fade in
            life_frac / 0.15
        } else if life_frac > 0.75 {
            // Fade out
            (1.0 - life_frac) / 0.25
        } else {
            1.0
        };
        transform.scale = Vec3::splat(snow.base_size * alpha);
    }
}
