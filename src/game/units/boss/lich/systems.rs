use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::LichAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker, WaveGroup};
use crate::game::resources::KillStats;
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::ogre::MeleeDamageReduction;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, Corpse, DamageMultiplier, Effectiveness,
    FlockingModifier, FlockingVelocity, Health, Hitbox, MovementSpeed,
    RoughTerrainModifier, SleepModifier, TargetingVelocity, Team, Teleportable,
    TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::infantry::components::Infantry;
use crate::game::units::infantry::styles::UNDEAD_SPRITE_TINT;
use crate::game::units::random_position_in_cell;
use crate::game::units::systems::create_default_sprite_material;
use crate::game::units::undead::resources::UndeadAssets;

/// Checks if it's time to spawn the Lich mid-game.
/// The Lich spawns as an extra wave after all normal waves have been dispatched
/// and every attacker (including staging) is dead.
pub fn check_lich_spawn(
    mut commands: Commands,
    lich_assets: Res<LichAssets>,
    pending: Option<Res<LichSpawnPending>>,
    wave_state: Option<Res<crate::game::resources::WaveState>>,
    existing: Query<(), With<Lich>>,
    all_attackers: Query<&Team, Without<Corpse>>,
) {
    let Some(_pending) = pending else { return };
    if !existing.is_empty() { return };
    let Some(wave_state) = wave_state else { return };

    // Wait for all normal waves to finish spawning
    if !wave_state.waves_complete {
        return;
    }

    // Wait for every attacker to die (staging or activated)
    let has_living_attackers = all_attackers.iter().any(|t| *t == Team::Attackers);
    if has_living_attackers {
        return;
    }

    spawn_lich(&mut commands, &lich_assets, wave_state.current_wave);
    commands.remove_resource::<LichSpawnPending>();
}

/// Spawns the Lich at one of the tunnel spawn points.
fn spawn_lich(commands: &mut Commands, lich_assets: &LichAssets, current_wave: u32) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

    let hitbox = Hitbox::new(LICH_RADIUS, LICH_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + (LICH_ELLIPSE_DEPTH / 2.0) + 1.0;

    commands
        .spawn((
            Mesh3d(lich_assets.mesh.clone()),
            MeshMaterial3d(lich_assets.material_summoning.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(LICH_HEALTH),
            MovementSpeed(LICH_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Undead,
            Boss,
            Lich,
        ))
        .insert((
            LichPhase::Approaching,
            SoulPower::new(SOUL_POWER_MAX),
            LichSummonTimer::new(SUMMON_INTERVAL),
            MeleeDamageReduction {
                multiplier: LICH_MELEE_DAMAGE_REDUCTION,
            },
            StagingAttacker,
            WaveGroup(current_wave),
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            DamageMultiplier(LICH_DAMAGE_MULTIPLIER),
        ))
        .insert((
            FlockingVelocity::default(),
            FlockingModifier::new(0.0, 0.0, 0.0),
            RoughTerrainModifier(0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ));
}

/// Detects when the normal staging system has activated the Lich
/// (removed StagingAttacker) and transitions to summoning phase.
pub fn lich_approach_system(
    mut query: Query<
        (&mut LichPhase, &mut Velocity, Has<StagingAttacker>),
        (With<Lich>, Without<Corpse>),
    >,
) {
    for (mut phase, mut velocity, has_staging) in &mut query {
        if *phase != LichPhase::Approaching {
            continue;
        }

        // StagingAttacker was removed by the normal staging system —
        // that means the Lich reached the staging zone, defenders are
        // activated, and the battle timer has started.
        if !has_staging {
            *phase = LichPhase::Summoning;
            velocity.x = 0.0;
            velocity.z = 0.0;
        }
    }
}

/// Phase 1b: Raises corpses as undead infantry (unlimited range).
/// If not enough corpses exist, spawns fresh undead to fill the wave.
pub fn lich_summoning_system(
    time: Res<Time>,
    mut commands: Commands,
    mut lich_query: Query<
        (&Transform, &mut LichSummonTimer, &LichPhase),
        (With<Lich>, Without<Corpse>),
    >,
    corpse_query: Query<
        (Entity, &Transform),
        (
            With<Corpse>,
            Without<crate::game::units::components::PermanentCorpse>,
            Without<Lich>,
        ),
    >,
    undead_assets: Res<UndeadAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (transform, mut timer, phase) in &mut lich_query {
        if *phase != LichPhase::Summoning {
            continue;
        }

        timer.tick(time.delta_secs());
        if !timer.is_ready() {
            continue;
        }

        timer.reset(SUMMON_INTERVAL);

        let lich_pos = transform.translation;
        let target = SUMMON_WAVE_SIZE as usize;
        let mut raised = 0usize;

        // Priority 1: Raise corpses from anywhere on the battlefield
        let corpses: Vec<(Entity, Vec3)> = corpse_query
            .iter()
            .map(|(e, t)| (e, t.translation))
            .take(target)
            .collect();

        for (corpse_entity, position) in corpses {
            crate::game::units::systems::resurrect_corpse_as_infantry(
                &mut commands,
                corpse_entity,
                position,
                Team::Undead,
                SUMMONED_UNDEAD_HEALTH,
                SUMMONED_UNDEAD_SPEED,
                UNDEAD_SPRITE_TINT,
                undead_assets.sprite_texture.clone(),
                undead_assets.sprite_mesh.clone(),
                &mut materials,
            );
            raised += 1;
        }

        // Priority 2: Spawn fresh undead around the Lich for the remainder
        let remaining = target.saturating_sub(raised);
        for i in 0..remaining {
            let angle = (i as f32 / remaining as f32) * std::f32::consts::TAU;
            let spawn_x = lich_pos.x + SUMMON_SPAWN_RADIUS * angle.cos();
            let spawn_z = lich_pos.z + SUMMON_SPAWN_RADIUS * angle.sin();

            spawn_fresh_undead(
                &mut commands,
                &undead_assets,
                &mut materials,
                spawn_x,
                spawn_z,
            );
        }
    }
}

/// Spawns a single fresh undead infantry unit at the given position.
fn spawn_fresh_undead(
    commands: &mut Commands,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
    x: f32,
    z: f32,
) {
    use crate::game::units::infantry::styles::UNIT_RADIUS;

    let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let material = create_default_sprite_material(
        materials,
        undead_assets.sprite_texture.clone(),
        UNDEAD_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(undead_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(x, spawn_y, z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(SUMMONED_UNDEAD_HEALTH),
            MovementSpeed(SUMMONED_UNDEAD_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Undead,
            Infantry,
        ))
        .insert((
            crate::game::units::components::WalkingAnimation::default(),
            crate::game::units::components::FacingDirection::default(),
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ));
}

/// Tracks undead kills and adds soul power to the Lich.
pub fn track_soul_power(
    kill_stats: Res<KillStats>,
    mut query: Query<(&mut SoulPower, &LichPhase), (With<Lich>, Without<Corpse>)>,
) {
    for (mut soul_power, phase) in &mut query {
        // Only accumulate during summoning phase
        if *phase != LichPhase::Summoning {
            continue;
        }

        let current_undead_killed = kill_stats.undead_killed;
        if current_undead_killed > soul_power.last_known_undead_killed {
            let new_kills = current_undead_killed - soul_power.last_known_undead_killed;
            soul_power.current =
                (soul_power.current + new_kills as f32 * SOUL_POWER_PER_KILL).min(soul_power.max);
            soul_power.last_known_undead_killed = current_undead_killed;
        }
    }
}

/// Checks if soul power is full and transitions to Phase 2.
pub fn lich_phase_transition(
    mut commands: Commands,
    lich_assets: Res<LichAssets>,
    mut query: Query<
        (Entity, &SoulPower, &mut LichPhase),
        (With<Lich>, Without<Corpse>),
    >,
) {
    for (entity, soul_power, mut phase) in &mut query {
        if *phase != LichPhase::Summoning || !soul_power.is_full() {
            continue;
        }

        // Transition to combat phase
        *phase = LichPhase::Combat;

        commands.entity(entity)
            .remove::<LichSummonTimer>()
            .insert(LichFingerOfDeath::new())
            .insert(MeshMaterial3d(lich_assets.material_combat.clone()));
    }
}

/// Phase 2 targeting: Updates the Lich's movement targeting toward nearest enemy.
/// Only runs in Combat phase.
pub fn update_lich_targeting(
    mut commands: Commands,
    mut lich_query: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity, &LichPhase),
        (With<Lich>, Without<Corpse>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (Without<Lich>, Without<Corpse>, Without<BanishedModifier>),
    >,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting, phase) in &mut lich_query {
        if *phase != LichPhase::Combat {
            targeting.velocity = Vec3::ZERO;
            continue;
        }

        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            transform,
            *team,
            &mut targeting,
            &mut commands,
            None,
        );
    }
}

/// Lich movement for all phases.
/// - Approaching: follows flow field toward staging point
/// - Summoning: stationary
/// - Combat: targeting + flow field toward defenders
pub fn lich_movement(
    time: Res<Time>,
    mut query: Query<
        (
            &Transform,
            &mut Velocity,
            &mut Acceleration,
            &TargetingVelocity,
            &FlowFieldVelocity,
            &MovementSpeed,
            &LichPhase,
        ),
        (With<Lich>, Without<Corpse>),
    >,
) {
    for (transform, mut velocity, mut acceleration, targeting, flow_field, speed, phase) in &mut query {
        match phase {
            LichPhase::Summoning => {
                // Stationary — zero everything
                velocity.x = 0.0;
                velocity.z = 0.0;
                acceleration.reset();
            }
            LichPhase::Approaching => {
                // Steer directly toward the staging point (can't use the staging
                // flow field because is_staging_attacker is Team::Attackers only).
                let max_speed = speed.0 * GLOBAL_SPEED_MULTIPLIER;
                let pos = transform.translation;
                let to_staging = Vec3::new(
                    STAGING_POINT.0 - pos.x,
                    0.0,
                    STAGING_POINT.1 - pos.z,
                );

                if to_staging.length_squared() > 1.0 {
                    let target_vel = to_staging.normalize() * max_speed;
                    let steer = STEERING_FORCE * time.delta_secs();
                    acceleration.x = (target_vel.x - velocity.x).clamp(-steer, steer) / time.delta_secs().max(0.001);
                    acceleration.z = (target_vel.z - velocity.z).clamp(-steer, steer) / time.delta_secs().max(0.001);
                }

                velocity.max_speed = max_speed;
                let damping = VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
                velocity.x *= damping;
                velocity.z *= damping;
            }
            LichPhase::Combat => {
                // Combine targeting and flow field
                let max_speed = speed.0 * GLOBAL_SPEED_MULTIPLIER;
                let combined = Vec3::new(
                    targeting.velocity.x * 0.7 + flow_field.velocity.x * 0.3,
                    0.0,
                    targeting.velocity.z * 0.7 + flow_field.velocity.z * 0.3,
                );

                if combined.length_squared() > 0.001 {
                    let target_vel = combined.normalize() * max_speed;
                    let steer = STEERING_FORCE * time.delta_secs();
                    acceleration.x = (target_vel.x - velocity.x).clamp(-steer, steer) / time.delta_secs().max(0.001);
                    acceleration.z = (target_vel.z - velocity.z).clamp(-steer, steer) / time.delta_secs().max(0.001);
                }

                velocity.max_speed = max_speed;
                let damping = VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
                velocity.x *= damping;
                velocity.z *= damping;
            }
        }
    }
}

/// Phase 2: Selects a random defender as the beam target.
/// Cannot target the King or King's Guard until 50% of defenders have died.
pub fn lich_combat_targeting(
    kill_stats: Res<KillStats>,
    mut lich_query: Query<(&mut LichFingerOfDeath, &LichPhase), (With<Lich>, Without<Corpse>)>,
    defenders: Query<
        (Entity, &Team),
        (
            Without<Corpse>,
            Without<Lich>,
            Without<Boss>,
            Without<SleepModifier>,
            Without<BanishedModifier>,
        ),
    >,
    king_query: Query<Entity, (With<crate::game::units::king::components::King>, Without<Corpse>)>,
    guard_query: Query<Entity, (With<crate::game::units::components::KingsGuard>, Without<Corpse>)>,
) {
    for (mut fod, phase) in &mut lich_query {
        if *phase != LichPhase::Combat {
            continue;
        }

        // Only pick a new target when cooldown is ready and no current target
        if fod.target.is_some() || !fod.is_ready() {
            continue;
        }

        // Determine if King + King's Guard are targetable
        let defender_death_ratio = if INITIAL_DEFENDER_COUNT > 0 {
            kill_stats.defenders_killed as f32 / INITIAL_DEFENDER_COUNT as f32
        } else {
            0.0
        };
        let can_target_royalty = defender_death_ratio >= KING_TARGET_THRESHOLD;

        let king_entity = king_query.iter().next();
        let guard_entities: Vec<Entity> = guard_query.iter().collect();

        // Collect eligible defender targets
        let eligible: Vec<Entity> = defenders
            .iter()
            .filter(|(_, team)| **team == Team::Defenders)
            .filter(|(entity, _)| {
                if !can_target_royalty {
                    // Skip king
                    if let Some(king_e) = king_entity {
                        if *entity == king_e {
                            return false;
                        }
                    }
                    // Skip king's guard
                    if guard_entities.contains(entity) {
                        return false;
                    }
                }
                true
            })
            .map(|(entity, _)| entity)
            .collect();

        if eligible.is_empty() {
            continue;
        }

        // Pick a random target using a simple hash-based selection
        let index = (kill_stats.elapsed_time * 1000.0) as usize % eligible.len();
        fod.target = Some(eligible[index]);
    }
}

/// Phase 2: Fires the death beam at the current target using the wizard's
/// Finger of Death visual system (same purple beam, screen darkening, casting effect).
/// Beam originates from the top of the Lich's sprite and shoots toward the target.
pub fn lich_fire_beam(
    time: Res<Time>,
    mut commands: Commands,
    spell_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    mut lich_query: Query<
        (&Transform, &Hitbox, &mut LichFingerOfDeath, &LichPhase),
        (With<Lich>, Without<Corpse>),
    >,
    target_query: Query<&Transform, Without<Lich>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut desaturate: MessageWriter<crate::game::crt_effect::ScreenDesaturateMessage>,
    mut target_health: Query<
        (Entity, &Transform, &Team, &mut Health, &Hitbox, Option<&mut TemporaryHitPoints>,
         Has<crate::game::units::king::components::King>),
        (Without<Corpse>, Without<Lich>, Without<Boss>),
    >,
) {
    use crate::game::units::wizard::spells::finger_of_death::components::{
        FingerOfDeathBeam, FodTalentParams, PendingUndeadRaise,
    };

    for (lich_transform, lich_hitbox, mut fod, phase) in &mut lich_query {
        if *phase != LichPhase::Combat {
            continue;
        }

        fod.tick(time.delta_secs());

        if !fod.is_ready() {
            continue;
        }

        let Some(target_entity) = fod.target else {
            continue;
        };

        let Ok(target_transform) = target_query.get(target_entity) else {
            fod.target = None;
            continue;
        };

        // Beam originates from the top of the Lich's sprite
        let origin = Vec3::new(
            lich_transform.translation.x,
            lich_transform.translation.y + lich_hitbox.height * 0.5,
            lich_transform.translation.z,
        );

        // Direction from origin toward the target
        let to_target = target_transform.translation - origin;
        let direction = to_target.normalize_or_zero();

        if direction.length_squared() < 0.5 {
            fod.target = None;
            continue;
        }

        let beam_length = BEAM_LENGTH.min(to_target.length() + 200.0);

        // Create a FoD beam that's already fired (instant cast, no charge-up)
        let talent_params = FodTalentParams {
            damage: BEAM_DAMAGE,
            beam_width: BEAM_WIDTH,
            beam_width_fired: BEAM_WIDTH,
            ..Default::default()
        };
        let mut beam = FingerOfDeathBeam::with_talents(
            origin,
            direction,
            beam_length,
            1.0,
            talent_params,
        );
        beam.has_fired = true;
        beam.cast_progress = 1.0;

        // Spawn using the wizard's visual system (beam + glow + flare)
        crate::game::units::wizard::spells::finger_of_death::systems::spawn_beam(
            &mut commands,
            &spell_assets,
            &mut materials,
            beam,
        );

        // Screen darkening effect
        desaturate.write(crate::game::crt_effect::ScreenDesaturateMessage);

        // Apply damage to defenders in the beam path
        let mut kill_positions: Vec<Vec3> = Vec::new();

        for (entity, t_transform, team, mut health, hitbox, temp_hp, is_king) in &mut target_health {
            if *team != Team::Defenders {
                continue;
            }

            let to_point = t_transform.translation - origin;
            let proj = to_point.dot(direction);
            if proj < 0.0 || proj > beam_length {
                continue;
            }
            let closest = origin + direction * proj;
            let dist = t_transform.translation.distance(closest);
            if dist <= BEAM_WIDTH + hitbox.radius {
                // King has 70% damage resistance to Finger of Death
                let damage = if is_king {
                    BEAM_DAMAGE * KING_FOD_DAMAGE_MULTIPLIER
                } else {
                    BEAM_DAMAGE
                };

                let hp_before = health.current;
                apply_spell_damage(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.map(|t| t.into_inner()),
                    damage,
                    DamageType::Necrotic,
                    false,
                );

                // Track kills for Finger of Undeath raising
                if hp_before > 0.0 && health.is_dead() {
                    kill_positions.push(t_transform.translation);
                }
            }
        }

        // Queue undead raises for killed defenders (processed next frame)
        if !kill_positions.is_empty() {
            commands.insert_resource(PendingUndeadRaise { kill_positions });
        }

        fod.reset(BEAM_COOLDOWN);
    }
}
