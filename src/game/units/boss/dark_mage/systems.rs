use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::DarkMageAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, Corpse, DamageMultiplier, Effectiveness, FlockingModifier,
    FlockingVelocity, FrozenSolidModifier, Health, Hitbox, MovementSpeed, OriginalMaterial,
    RootedModifier, SickenedModifier, SleepModifier, Sleepwalking, TargetingVelocity, Team,
    Teleportable, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns the Dark Mage at a tunnel spawn point (walks in like other bosses).
pub fn spawn_dark_mage(rng: &mut impl Rng, mut commands: Commands, assets: Res<DarkMageAssets>) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = crate::game::units::random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(DARK_MAGE_RADIUS, DARK_MAGE_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + (DARK_MAGE_ELLIPSE_DEPTH / 2.0) + 1.0;

    // Initial velocity toward castle (approach phase)
    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * DARK_MAGE_APPROACH_SPEED;

    commands
        .spawn((
            // Rendering
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.material_phase0.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            // Physics
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
            },
            Acceleration::new(),
            // Core
            hitbox,
            Health::new(DARK_MAGE_HEALTH),
            MovementSpeed(DARK_MAGE_APPROACH_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Boss,
            DarkMage,
        ))
        .insert((
            DarkMageState::Approaching,
            DarkMageSpellCooldowns::new(
                METEOR_COOLDOWN * 0.3,    // First meteor comes relatively fast
                LIGHTNING_COOLDOWN * 0.1, // Lightning comes first
                PLAGUE_COOLDOWN * 0.5,    // Plague a bit later
            ),
            DarkMageSpellQueue::new(),
            DarkMageTeleportTimer::new(TELEPORT_COOLDOWN),
            DarkMageEnrage::new(),
            DamageMultiplier(DARK_MAGE_DAMAGE_MULTIPLIER),
            crate::game::units::boss::ogre::MeleeDamageReduction {
                multiplier: DARK_MAGE_MELEE_DAMAGE_REDUCTION,
            },
            // Movement systems (used during approach phase)
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            FlockingVelocity::default(),
            FlockingModifier::new(0.0, 0.0, 0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ));
}

/// Dark Mage movement: always follows flow field pathfinding.
/// During Approaching, transitions to Idle once reaching the battlefield.
/// During Telegraphing/Casting, stands still.
pub fn dark_mage_movement(
    time: Res<Time>,
    mut bosses: Query<
        (
            &Transform,
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &mut DarkMageState,
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
) {
    for (
        transform,
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        mut state,
    ) in &mut bosses
    {
        // Freeze movement during telegraphing and casting
        if matches!(
            *state,
            DarkMageState::Telegraphing { .. } | DarkMageState::Casting { .. }
        ) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            acceleration.x = 0.0;
            acceleration.z = 0.0;
            continue;
        }

        // Follow flow field at all times (approach and idle)
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            false,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        // Transition from approaching to idle once on the battlefield
        if matches!(*state, DarkMageState::Approaching)
            && transform.translation.x <= DARK_MAGE_APPROACH_TARGET_X
        {
            *state = DarkMageState::Idle;
        }
    }
}

/// Ticks spell cooldowns and enqueues spells that come off cooldown.
pub fn dark_mage_spell_queue(
    time: Res<Time>,
    mut bosses: Query<
        (
            &mut DarkMageSpellCooldowns,
            &mut DarkMageSpellQueue,
            &DarkMageState,
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
) {
    let delta = time.delta_secs();

    for (mut cooldowns, mut queue, state) in &mut bosses {
        // Don't tick cooldowns while approaching
        if matches!(state, DarkMageState::Approaching) {
            continue;
        }
        cooldowns.tick(delta);

        // Enqueue spells as they come off cooldown (order: lightning, meteor, plague)
        let spell_order = [
            DarkMageSpellType::ShadowLightning,
            DarkMageSpellType::DarkMeteor,
            DarkMageSpellType::PlagueCloud,
        ];

        for spell in &spell_order {
            if cooldowns.is_ready(*spell) && !queue.queue.contains(spell) {
                queue.queue.push_back(*spell);
            }
        }
    }
}

/// Main Dark Mage AI: processes the spell queue, manages telegraph → cast transitions.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn dark_mage_ai(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    assets: Res<DarkMageAssets>,
    spell_assets: Res<SpellVisualAssets>,
    sfx: Res<crate::game::units::wizard::spells::audio::SpellSfxAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bosses: Query<
        (
            &Transform,
            &mut DarkMageState,
            &mut DarkMageSpellCooldowns,
            &mut DarkMageSpellQueue,
            &DarkMageEnrage,
            &Team,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
            ),
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
    potential_targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<DarkMage>,
            Without<Corpse>,
            Without<Boss>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        boss_transform,
        mut state,
        mut cooldowns,
        mut queue,
        enrage,
        boss_team,
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned),
    ) in &mut bosses
    {
        // CC'd bosses can't cast
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
        ) {
            // If telegraphing, cancel and go back to idle
            if let DarkMageState::Telegraphing { indicators, .. } = state.as_ref() {
                despawn_indicators(&mut commands, indicators.all());
            }
            if !matches!(state.as_ref(), &DarkMageState::Idle) {
                *state = DarkMageState::Idle;
            }
            continue;
        }

        match state.as_mut() {
            DarkMageState::Approaching => {
                // Don't cast spells while approaching
                continue;
            }
            DarkMageState::Idle => {
                // Pop next spell from queue
                if let Some(spell_type) = queue.queue.pop_front() {
                    // Find target position
                    let boss_pos = boss_transform.translation;
                    if let Some((target_pos, direction)) =
                        find_spell_target(spell_type, boss_pos, boss_team, &potential_targets)
                    {
                        let duration = telegraph_duration(spell_type);

                        // Spawn telegraph indicators
                        let indicators = spawn_telegraph_indicators(
                            &mut commands,
                            &assets,
                            &mut materials,
                            spell_type,
                            target_pos,
                            direction,
                        );

                        *state = DarkMageState::Telegraphing {
                            spell_type,
                            elapsed: 0.0,
                            duration,
                            target_pos,
                            direction,
                            indicators,
                        };

                        // Reset cooldown now so it starts ticking during telegraph
                        let base_cd = spell_cooldown(spell_type);
                        cooldowns.reset(spell_type, base_cd * enrage.cooldown_mult);
                    } else {
                        // No valid target -- push spell back and wait
                        queue.queue.push_front(spell_type);
                    }
                }
            }

            DarkMageState::Telegraphing {
                spell_type,
                elapsed,
                duration,
                target_pos,
                direction,
                indicators,
            } => {
                *elapsed += delta;
                let progress = (*elapsed / *duration).min(1.0);

                // Animate indicator emissive glow
                if let Some(mat) = materials.get_mut(&indicators.fill_material) {
                    let pulse =
                        (*elapsed * INDICATOR_PULSE_FREQUENCY * std::f32::consts::TAU).sin() * 0.5
                            + 0.5;
                    let intensity = progress * INDICATOR_EMISSIVE_MAX * (0.6 + 0.4 * pulse);
                    mat.emissive = bevy::color::LinearRgba::new(
                        intensity,
                        intensity * 0.08,
                        intensity * 0.02,
                        1.0,
                    );
                    let alpha = 0.1 + progress * 0.4;
                    mat.base_color = Color::srgba(0.8, 0.1, 0.05, alpha);
                }

                if *elapsed >= *duration {
                    let sp = *spell_type;
                    let tp = *target_pos;
                    let dir = *direction;
                    despawn_indicators(&mut commands, indicators.all());
                    *state = DarkMageState::Casting {
                        spell_type: sp,
                        target_pos: tp,
                        direction: dir,
                    };
                }
            }

            DarkMageState::Casting {
                spell_type,
                target_pos,
                direction,
            } => {
                // Fire the spell with sound effects
                let tp = *target_pos;
                match spell_type {
                    DarkMageSpellType::DarkMeteor => {
                        spawn_meteor_explosion(&mut commands, &assets, &spell_assets, tp);
                        crate::game::units::wizard::spells::audio::play_sfx_scaled(
                            &mut commands,
                            &sfx.fireball_impact,
                            tp,
                            &game_config,
                            1.0,
                        );
                    }
                    DarkMageSpellType::ShadowLightning => {
                        if let Some(dir) = direction {
                            spawn_lightning_strike(&mut commands, &assets, &spell_assets, tp, *dir);
                        }
                        crate::game::units::wizard::spells::audio::play_sfx_scaled(
                            &mut commands,
                            &sfx.chain_lightning_cast,
                            tp,
                            &game_config,
                            1.0,
                        );
                    }
                    DarkMageSpellType::PlagueCloud => {
                        spawn_plague_cloud(&mut game_rng.0, &mut commands, &assets, &spell_assets, tp);
                        crate::game::units::wizard::spells::audio::play_sfx_scaled(
                            &mut commands,
                            &sfx.plague_wind_cast,
                            tp,
                            &game_config,
                            1.0,
                        );
                    }
                }
                *state = DarkMageState::Idle;
            }
        }
    }
}

/// Teleport system: teleports the Dark Mage away when enemy units get into melee range.
/// Has a cooldown to prevent constant teleporting.
#[allow(clippy::type_complexity)]
pub fn dark_mage_teleport(
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut bosses: Query<
        (
            &mut Transform,
            &mut DarkMageTeleportTimer,
            &DarkMageState,
            &Hitbox,
            &Team,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
            ),
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
    nearby_units: Query<
        (&Transform, &Hitbox, &Team),
        (
            Without<DarkMage>,
            Without<Corpse>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        mut transform,
        mut teleport_timer,
        state,
        hitbox,
        boss_team,
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned),
    ) in &mut bosses
    {
        // Don't teleport while approaching
        if matches!(state, DarkMageState::Approaching) {
            continue;
        }
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
        ) {
            continue;
        }

        teleport_timer.tick(delta);

        if !teleport_timer.is_ready() {
            continue;
        }

        // Check if any enemy is in melee range
        let boss_pos = transform.translation;
        let melee_range = (hitbox.radius * ATTACK_RANGE_MULTIPLIER) * 1.5;
        let mut enemy_nearby = false;

        for (unit_transform, unit_hitbox, unit_team) in &nearby_units {
            if !boss_team.is_enemy(unit_team) {
                continue;
            }
            let dx = unit_transform.translation.x - boss_pos.x;
            let dz = unit_transform.translation.z - boss_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist <= melee_range + unit_hitbox.radius {
                enemy_nearby = true;
                break;
            }
        }

        if !enemy_nearby {
            continue;
        }

        teleport_timer.reset(TELEPORT_COOLDOWN);

        // Pick a random valid destination within the visible area and wizard's spell range
        let castle_xz = Vec2::new(CASTLE_POSITION.x, CASTLE_POSITION.z);
        let wizard_xz = Vec2::new(WIZARD_POSITION.x, WIZARD_POSITION.z);
        let wizard_ground_range = crate::game::units::wizard::spells::utils::ground_projected_range(
            crate::game::units::wizard::constants::DEFAULT_SPELL_RANGE,
            WIZARD_POSITION.y,
        );

        for _ in 0..20 {
            let x = VISIBLE_MIN_X + game_rng.0.r#gen::<f32>() * (VISIBLE_MAX_X - VISIBLE_MIN_X);
            let z = VISIBLE_MIN_Z + game_rng.0.r#gen::<f32>() * (VISIBLE_MAX_Z - VISIBLE_MIN_Z);
            let candidate = Vec2::new(x, z);

            let dist_from_current = ((x - boss_pos.x).powi(2) + (z - boss_pos.z).powi(2)).sqrt();
            let dist_from_castle = candidate.distance(castle_xz);
            let dist_from_wizard = candidate.distance(wizard_xz);

            if dist_from_current >= TELEPORT_MIN_DISTANCE
                && dist_from_castle >= TELEPORT_MIN_CASTLE_DISTANCE
                && dist_from_wizard <= wizard_ground_range
            {
                let y = hitbox.height / 2.0 + (DARK_MAGE_ELLIPSE_DEPTH / 2.0) + 1.0;
                transform.translation = Vec3::new(x, y, z);
                break;
            }
        }
    }
}

/// Updates meteor explosion visuals and applies one-time damage.
#[allow(clippy::type_complexity)]
pub fn update_meteor_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut explosions: Query<
        (Entity, &mut DarkMageMeteorExplosion, &mut Transform),
        Without<DarkMage>,
    >,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<DarkMageMeteorExplosion>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (entity, mut explosion, mut transform) in &mut explosions {
        explosion.lifetime -= delta;

        // Visual: grow then shrink
        let progress = 1.0 - (explosion.lifetime / explosion.max_lifetime);
        let scale = if progress < 0.4 {
            // Grow phase
            let t = progress / 0.4;
            explosion.radius * t
        } else if progress < 0.6 {
            // Hold
            explosion.radius
        } else {
            // Shrink
            let t = (progress - 0.6) / 0.4;
            explosion.radius * (1.0 - t)
        };
        transform.scale = Vec3::splat(scale.max(0.1));

        // Apply one-time damage
        if !explosion.damage_applied {
            explosion.damage_applied = true;
            let center = transform.translation;

            for (
                target_entity,
                target_transform,
                target_hitbox,
                team,
                mut health,
                temp_hp,
                has_shield,
            ) in &mut targets
            {
                if *team != Team::Defenders {
                    continue;
                }

                let dx = target_transform.translation.x - center.x;
                let dz = target_transform.translation.z - center.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= explosion.radius + target_hitbox.radius {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        explosion.damage,
                        DamageType::Fire,
                        has_shield,
                    );
                }
            }
        }

        if explosion.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Updates lightning strike visuals and applies one-time corridor damage.
#[allow(clippy::type_complexity)]
pub fn update_lightning_strikes(
    time: Res<Time>,
    mut commands: Commands,
    mut strikes: Query<(Entity, &Transform, &mut DarkMageLightningStrike), Without<DarkMage>>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<DarkMageLightningStrike>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (entity, strike_transform, mut strike) in &mut strikes {
        strike.lifetime -= delta;

        // Apply one-time damage in the corridor
        if !strike.damage_applied {
            strike.damage_applied = true;
            let center = strike_transform.translation;
            let dir = strike.direction;
            let perp = Vec3::new(-dir.z, 0.0, dir.x);

            for (
                target_entity,
                target_transform,
                target_hitbox,
                team,
                mut health,
                temp_hp,
                has_shield,
            ) in &mut targets
            {
                if *team != Team::Defenders {
                    continue;
                }

                let to_target = target_transform.translation - center;
                let along = to_target.dot(dir);
                let across = to_target.dot(perp).abs();

                if along.abs() <= strike.half_length + target_hitbox.radius
                    && across <= strike.half_width + target_hitbox.radius
                {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        strike.damage,
                        DamageType::Electric,
                        has_shield,
                    );
                }
            }
        }

        if strike.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Updates persistent plague clouds: ticks damage and despawns when expired.
#[allow(clippy::type_complexity)]
pub fn update_plague_clouds(
    time: Res<Time>,
    mut commands: Commands,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut clouds: Query<(Entity, &Transform, &mut DarkMagePlagueCloud), Without<DarkMage>>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<DarkMagePlagueCloud>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (entity, cloud_transform, mut cloud) in &mut clouds {
        cloud.lifetime -= delta;
        cloud.tick_timer -= delta;

        if cloud.tick_timer <= 0.0 {
            cloud.tick_timer += PLAGUE_TICK_INTERVAL;

            let center = cloud_transform.translation;

            for (
                target_entity,
                target_transform,
                target_hitbox,
                team,
                mut health,
                temp_hp,
                has_shield,
            ) in &mut targets
            {
                if *team != Team::Defenders {
                    continue;
                }

                let dx = target_transform.translation.x - center.x;
                let dz = target_transform.translation.z - center.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= cloud.radius + target_hitbox.radius {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        cloud.damage,
                        DamageType::Poison,
                        has_shield,
                    );
                }
            }
        }

        if cloud.lifetime <= 0.0 {
            // Remove hazard from flow field
            let center_xz = Vec2::new(cloud_transform.translation.x, cloud_transform.translation.z);
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(center_xz, Vec2::splat(cloud.radius * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(center_xz, cloud.radius)),
                rebuild: false,
            });
            commands.entity(entity).try_despawn();
        }
    }
}

/// Monitors health thresholds and updates enrage state.
#[allow(clippy::type_complexity)]
pub fn dark_mage_enrage(
    mut bosses: Query<
        (
            &Health,
            &mut DarkMageEnrage,
            &mut MeshMaterial3d<StandardMaterial>,
            Option<&mut OriginalMaterial>,
        ),
        With<DarkMage>,
    >,
    assets: Res<DarkMageAssets>,
) {
    for (health, mut enrage, mut mesh_material, original_material) in &mut bosses {
        let hp_ratio = health.current / health.max;

        let new_phase = if hp_ratio <= ENRAGE_PHASE_3_THRESHOLD {
            3
        } else if hp_ratio <= ENRAGE_PHASE_2_THRESHOLD {
            2
        } else if hp_ratio <= ENRAGE_PHASE_1_THRESHOLD {
            1
        } else {
            0
        };

        if new_phase != enrage.phase {
            enrage.phase = new_phase;

            enrage.cooldown_mult = match new_phase {
                1 => ENRAGE_1_COOLDOWN_MULT,
                2 => ENRAGE_2_COOLDOWN_MULT,
                3 => ENRAGE_3_COOLDOWN_MULT,
                _ => 1.0,
            };

            let phase_material = match new_phase {
                1 => assets.material_phase1.clone(),
                2 => assets.material_phase2.clone(),
                3 => assets.material_phase3.clone(),
                _ => assets.material_phase0.clone(),
            };

            if let Some(mut orig) = original_material {
                orig.0 = phase_material;
            } else {
                mesh_material.0 = phase_material;
            }
        }
    }
}

// ===== Helper Functions =====

/// Returns the telegraph duration for a given spell type.
fn telegraph_duration(spell: DarkMageSpellType) -> f32 {
    match spell {
        DarkMageSpellType::DarkMeteor => METEOR_TELEGRAPH_DURATION,
        DarkMageSpellType::ShadowLightning => LIGHTNING_TELEGRAPH_DURATION,
        DarkMageSpellType::PlagueCloud => PLAGUE_TELEGRAPH_DURATION,
    }
}

/// Returns the base cooldown for a given spell type.
fn spell_cooldown(spell: DarkMageSpellType) -> f32 {
    match spell {
        DarkMageSpellType::DarkMeteor => METEOR_COOLDOWN,
        DarkMageSpellType::ShadowLightning => LIGHTNING_COOLDOWN,
        DarkMageSpellType::PlagueCloud => PLAGUE_COOLDOWN,
    }
}

/// Finds the best target position for a spell, avoiding the boss's own position.
fn find_spell_target(
    spell: DarkMageSpellType,
    boss_pos: Vec3,
    boss_team: &Team,
    targets: &Query<
        (Entity, &Transform, &Team),
        (
            Without<DarkMage>,
            Without<Corpse>,
            Without<Boss>,
            Without<BanishedModifier>,
        ),
    >,
) -> Option<(Vec3, Option<Vec3>)> {
    // Collect enemy positions within spell range and visible area
    let enemies: Vec<Vec3> = targets
        .iter()
        .filter(|(_, _, team)| boss_team.is_enemy(team))
        .map(|(_, transform, _)| transform.translation)
        .filter(|pos| {
            let dist = ((*pos - boss_pos) * Vec3::new(1.0, 0.0, 1.0)).length();
            dist <= MAX_SPELL_RANGE
                && pos.x >= VISIBLE_MIN_X
                && pos.x <= VISIBLE_MAX_X
                && pos.z >= VISIBLE_MIN_Z
                && pos.z <= VISIBLE_MAX_Z
        })
        .collect();

    if enemies.is_empty() {
        return None;
    }

    match spell {
        DarkMageSpellType::DarkMeteor | DarkMageSpellType::PlagueCloud => {
            // Find the densest cluster of enemies (most enemies within spell radius)
            let radius = if spell == DarkMageSpellType::DarkMeteor {
                METEOR_RADIUS
            } else {
                PLAGUE_RADIUS
            };

            let mut best_pos = None;
            let mut best_count = 0;

            for &pos in &enemies {
                // Skip if too close to boss
                let dx = pos.x - boss_pos.x;
                let dz = pos.z - boss_pos.z;
                if (dx * dx + dz * dz).sqrt() < MIN_TARGET_DISTANCE_FROM_SELF {
                    continue;
                }

                let count = enemies
                    .iter()
                    .filter(|&other| {
                        let d = (*other - pos).xz().length();
                        d <= radius
                    })
                    .count();

                if count > best_count {
                    best_count = count;
                    best_pos = Some(pos);
                }
            }

            best_pos.map(|p| (Vec3::new(p.x, INDICATOR_Y, p.z), None))
        }

        DarkMageSpellType::ShadowLightning => {
            // Find a direction that hits the most enemies in a corridor
            let mut best_pos = None;
            let mut best_dir = None;
            let mut best_count = 0;

            for &origin in &enemies {
                let dx = origin.x - boss_pos.x;
                let dz = origin.z - boss_pos.z;
                if (dx * dx + dz * dz).sqrt() < MIN_TARGET_DISTANCE_FROM_SELF {
                    continue;
                }

                // Try corridor from boss toward this enemy
                let dir = Vec3::new(dx, 0.0, dz).normalize_or_zero();
                if dir.length_squared() < 0.5 {
                    continue;
                }
                let perp = Vec3::new(-dir.z, 0.0, dir.x);
                let corridor_center = boss_pos + dir * (LIGHTNING_CORRIDOR_LENGTH / 2.0);

                let count = enemies
                    .iter()
                    .filter(|&other| {
                        let to = *other - corridor_center;
                        let along = to.dot(dir).abs();
                        let across = to.dot(perp).abs();
                        along <= LIGHTNING_CORRIDOR_LENGTH / 2.0
                            && across <= LIGHTNING_CORRIDOR_WIDTH / 2.0
                    })
                    .count();

                if count > best_count {
                    best_count = count;
                    best_pos = Some(corridor_center);
                    best_dir = Some(dir);
                }
            }

            if let (Some(pos), Some(dir)) = (best_pos, best_dir) {
                Some((Vec3::new(pos.x, INDICATOR_Y, pos.z), Some(dir)))
            } else {
                // Fallback: aim at nearest enemy
                let nearest = enemies
                    .iter()
                    .filter(|p| {
                        let d = (**p - boss_pos).xz().length();
                        d >= MIN_TARGET_DISTANCE_FROM_SELF
                    })
                    .min_by(|a, b| {
                        let da = (**a - boss_pos).length_squared();
                        let db = (**b - boss_pos).length_squared();
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    })?;
                let dir = (*nearest - boss_pos).normalize_or_zero();
                let center = boss_pos + dir * (LIGHTNING_CORRIDOR_LENGTH / 2.0);
                Some((
                    Vec3::new(center.x, INDICATOR_Y, center.z),
                    Some(Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero()),
                ))
            }
        }
    }
}

/// Spawns telegraph indicator entities for a spell.
fn spawn_telegraph_indicators(
    commands: &mut Commands,
    assets: &DarkMageAssets,
    materials: &mut Assets<StandardMaterial>,
    spell_type: DarkMageSpellType,
    target_pos: Vec3,
    direction: Option<Vec3>,
) -> DarkMageIndicators {
    let fill_material = materials.add(StandardMaterial {
        base_color: INDICATOR_BASE_COLOR,
        emissive: bevy::color::LinearRgba::new(0.0, 0.0, 0.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    });

    let mut entities = Vec::new();

    match spell_type {
        DarkMageSpellType::DarkMeteor => {
            // Circle indicator
            let entity = commands
                .spawn((
                    Mesh3d(assets.circle_mesh.clone()),
                    MeshMaterial3d(fill_material.clone()),
                    Transform::from_translation(target_pos.with_y(INDICATOR_Y))
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                        .with_scale(Vec3::splat(METEOR_RADIUS)),
                    DarkMageIndicator,
                    OnGameplayScreen,
                ))
                .id();
            entities.push(entity);
        }

        DarkMageSpellType::ShadowLightning => {
            if let Some(dir) = direction {
                // Rectangle corridor indicator
                let rotation = indicator_rotation(dir);
                let entity = commands
                    .spawn((
                        Mesh3d(assets.rect_mesh.clone()),
                        MeshMaterial3d(fill_material.clone()),
                        Transform::from_translation(target_pos.with_y(INDICATOR_Y))
                            .with_rotation(rotation)
                            .with_scale(Vec3::new(
                                LIGHTNING_CORRIDOR_WIDTH,
                                LIGHTNING_CORRIDOR_LENGTH,
                                1.0,
                            )),
                        DarkMageIndicator,
                        OnGameplayScreen,
                    ))
                    .id();
                entities.push(entity);
            }
        }

        DarkMageSpellType::PlagueCloud => {
            // Circle indicator (like meteor but different radius)
            let entity = commands
                .spawn((
                    Mesh3d(assets.circle_mesh.clone()),
                    MeshMaterial3d(fill_material.clone()),
                    Transform::from_translation(target_pos.with_y(INDICATOR_Y))
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                        .with_scale(Vec3::splat(PLAGUE_RADIUS)),
                    DarkMageIndicator,
                    OnGameplayScreen,
                ))
                .id();
            entities.push(entity);
        }
    }

    DarkMageIndicators {
        entities,
        fill_material,
    }
}

/// Spawns the meteor explosion effect entity.
fn spawn_meteor_explosion(
    commands: &mut Commands,
    assets: &DarkMageAssets,
    spell_assets: &SpellVisualAssets,
    target_pos: Vec3,
) {
    // Ground explosion effect
    commands.spawn((
        Mesh3d(assets.explosion_mesh.clone()),
        MeshMaterial3d(assets.meteor_explosion_material.clone()),
        Transform::from_translation(target_pos.with_y(INDICATOR_Y + 1.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(0.1)),
        DarkMageMeteorExplosion {
            lifetime: METEOR_EXPLOSION_DURATION,
            max_lifetime: METEOR_EXPLOSION_DURATION,
            radius: METEOR_RADIUS,
            damage_applied: false,
            damage: METEOR_DAMAGE,
        },
        OnGameplayScreen,
    ));

    // Falling meteor projectile from the sky
    commands.spawn((
        Mesh3d(spell_assets.cross_plane_sphere.clone()),
        MeshMaterial3d(spell_assets.meteor_projectile.clone()),
        Transform::from_translation(Vec3::new(target_pos.x, METEOR_FALL_HEIGHT, target_pos.z))
            .with_scale(Vec3::splat(METEOR_PROJECTILE_RADIUS)),
        DarkMageMeteorProjectile {
            target_pos,
            velocity: METEOR_FALL_HEIGHT / 0.3, // Reaches ground in ~0.3s
        },
        OnGameplayScreen,
    ));
}

/// Spawns the lightning strike effect entity.
fn spawn_lightning_strike(
    commands: &mut Commands,
    assets: &DarkMageAssets,
    spell_assets: &SpellVisualAssets,
    target_pos: Vec3,
    direction: Vec3,
) {
    let rotation = indicator_rotation(direction);

    // Ground corridor damage zone
    commands.spawn((
        Mesh3d(assets.rect_mesh.clone()),
        MeshMaterial3d(assets.lightning_strike_material.clone()),
        Transform::from_translation(target_pos.with_y(INDICATOR_Y + 1.0))
            .with_rotation(rotation)
            .with_scale(Vec3::new(
                LIGHTNING_CORRIDOR_WIDTH,
                LIGHTNING_CORRIDOR_LENGTH,
                1.0,
            )),
        DarkMageLightningStrike {
            lifetime: LIGHTNING_STRIKE_DURATION,
            half_width: LIGHTNING_CORRIDOR_WIDTH / 2.0,
            half_length: LIGHTNING_CORRIDOR_LENGTH / 2.0,
            direction,
            damage_applied: false,
            damage: LIGHTNING_DAMAGE,
        },
        OnGameplayScreen,
    ));

    // Vertical lightning bolt striking down from sky along the corridor
    // Spawn several bolts along the corridor length for visual impact
    let bolt_count = 3;
    let step = LIGHTNING_CORRIDOR_LENGTH / (bolt_count as f32 + 1.0);
    let corridor_start = target_pos - direction * (LIGHTNING_CORRIDOR_LENGTH / 2.0);

    for i in 1..=bolt_count {
        let pos = corridor_start + direction * (step * i as f32);
        commands.spawn((
            Mesh3d(spell_assets.cross_plane_cylinder.clone()),
            MeshMaterial3d(assets.lightning_strike_material.clone()),
            Transform::from_translation(Vec3::new(pos.x, LIGHTNING_BOLT_HEIGHT / 2.0, pos.z))
                .with_scale(Vec3::new(8.0, LIGHTNING_BOLT_HEIGHT, 8.0)),
            DarkMageLightningBolt {
                lifetime: LIGHTNING_STRIKE_DURATION,
            },
            OnGameplayScreen,
        ));
    }
}

/// Spawns the persistent plague cloud entity and broadcasts hazard to flow field.
fn spawn_plague_cloud(
    rng: &mut impl Rng,
    commands: &mut Commands,
    assets: &DarkMageAssets,
    spell_assets: &SpellVisualAssets,
    target_pos: Vec3,
) {
    // Ground zone circle
    commands.spawn((
        Mesh3d(assets.circle_mesh.clone()),
        MeshMaterial3d(assets.plague_zone_material.clone()),
        Transform::from_translation(target_pos.with_y(INDICATOR_Y + 0.5))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(PLAGUE_RADIUS)),
        DarkMagePlagueCloud {
            lifetime: PLAGUE_DURATION,
            tick_timer: 0.0,
            particle_timer: 0.0,
            radius: PLAGUE_RADIUS,
            damage: PLAGUE_DAMAGE_PER_TICK,
        },
        OnGameplayScreen,
    ));

    // Spawn initial cloud puffs (cross-plane spheres floating above the zone)
    for i in 0..5 {
        let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
        let offset_r = PLAGUE_RADIUS * 0.5 * rng.r#gen::<f32>();
        let px = target_pos.x + angle.cos() * offset_r;
        let pz = target_pos.z + angle.sin() * offset_r;
        let py = 20.0 + rng.r#gen::<f32>() * 40.0;

        commands.spawn((
            Mesh3d(spell_assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.plague_zone_material.clone()),
            Transform::from_translation(Vec3::new(px, py, pz))
                .with_scale(Vec3::splat(40.0 + rng.r#gen::<f32>() * 30.0)),
            DarkMageVisualEffect {
                lifetime: PLAGUE_DURATION,
            },
            OnGameplayScreen,
        ));
    }
}

/// System that broadcasts plague cloud hazards to the flow field when they spawn.
/// Runs once per cloud entity that doesn't yet have the marker.
pub fn broadcast_plague_hazards(
    mut commands: Commands,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    clouds: Query<(Entity, &Transform, &DarkMagePlagueCloud), Without<PlagueHazardBroadcast>>,
) {
    for (entity, transform, cloud) in &clouds {
        let center_xz = Vec2::new(transform.translation.x, transform.translation.z);
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(center_xz, Vec2::splat(cloud.radius * 2.0)),
            obstacle_type: ObstacleType::Hazard(PLAGUE_HAZARD_COST),
            shape: Some(ObstacleShape::circle(center_xz, cloud.radius)),
            rebuild: false,
        });
        commands.entity(entity).insert(PlagueHazardBroadcast);
    }
}

/// Marker component to track that a plague cloud has already broadcast its hazard.
#[derive(Component)]
pub struct PlagueHazardBroadcast;

/// Updates falling meteor projectiles -- moves them downward and despawns on impact.
pub fn update_meteor_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &DarkMageMeteorProjectile)>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, projectile) in &mut projectiles {
        transform.translation.y -= projectile.velocity * delta;

        // Despawn when it reaches the ground
        if transform.translation.y <= projectile.target_pos.y + 5.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Updates lightning bolt visuals -- despawns after lifetime expires.
pub fn update_lightning_bolts(
    time: Res<Time>,
    mut commands: Commands,
    mut bolts: Query<(Entity, &mut DarkMageLightningBolt)>,
) {
    let delta = time.delta_secs();

    for (entity, mut bolt) in &mut bolts {
        bolt.lifetime -= delta;
        if bolt.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Updates temporary visual effects (plague puffs, etc.) -- despawns after lifetime expires.
pub fn update_visual_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Query<(Entity, &mut DarkMageVisualEffect)>,
) {
    let delta = time.delta_secs();

    for (entity, mut effect) in &mut effects {
        effect.lifetime -= delta;
        if effect.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns floating cloud puffs periodically for active plague clouds.
pub fn update_plague_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    assets: Res<DarkMageAssets>,
    spell_assets: Res<SpellVisualAssets>,
    mut clouds: Query<(&Transform, &mut DarkMagePlagueCloud)>,
) {
    let delta = time.delta_secs();

    for (transform, mut cloud) in &mut clouds {
        cloud.particle_timer -= delta;
        if cloud.particle_timer <= 0.0 {
            cloud.particle_timer += 0.4;

            let center = transform.translation;
            let angle = game_rng.0.r#gen::<f32>() * std::f32::consts::TAU;
            let offset_r = cloud.radius * 0.6 * game_rng.0.r#gen::<f32>();
            let px = center.x + angle.cos() * offset_r;
            let pz = center.z + angle.sin() * offset_r;
            let py = 15.0 + game_rng.0.r#gen::<f32>() * 30.0;

            commands.spawn((
                Mesh3d(spell_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(assets.plague_zone_material.clone()),
                Transform::from_translation(Vec3::new(px, py, pz))
                    .with_scale(Vec3::splat(30.0 + game_rng.0.r#gen::<f32>() * 25.0)),
                DarkMageVisualEffect {
                    lifetime: 2.0 + game_rng.0.r#gen::<f32>() * 1.5,
                },
                OnGameplayScreen,
            ));
        }
    }
}

use crate::game::units::boss::utils::{despawn_indicators, indicator_rotation};
