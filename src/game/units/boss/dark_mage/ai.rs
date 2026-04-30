//! Dark mage spawn, movement, and AI.

use super::spells::{
    find_spell_target, spawn_lightning_strike, spawn_meteor_explosion, spawn_plague_cloud,
    spawn_telegraph_indicators, spell_cooldown, telegraph_duration,
};
use crate::game::units::boss::utils::{animate_telegraph_material, despawn_indicators};
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::DarkMageAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, Corpse, DamageMultiplier, Effectiveness, FlockingModifier,
    FlockingVelocity, FrozenSolidModifier, Health, Hitbox, MovementSpeed, RootedModifier,
    SickenedModifier, SleepModifier, Sleepwalking, TargetingVelocity, Team, Teleportable,
};
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
                Option<&crate::game::units::components::Petrified>,
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
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned, petrified),
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
            petrified,
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
                    animate_telegraph_material(mat, *elapsed, progress, 0.8);
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
                        spawn_plague_cloud(
                            &mut game_rng.0,
                            &mut commands,
                            &assets,
                            &spell_assets,
                            tp,
                        );
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
                Option<&crate::game::units::components::Petrified>,
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
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned, petrified),
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
            petrified,
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
            let x = VISIBLE_MIN_X + game_rng.0.random::<f32>() * (VISIBLE_MAX_X - VISIBLE_MIN_X);
            let z = VISIBLE_MIN_Z + game_rng.0.random::<f32>() * (VISIBLE_MAX_Z - VISIBLE_MIN_Z);
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
