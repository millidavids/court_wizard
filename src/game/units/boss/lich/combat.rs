//! Lich combat: targeting, movement, beam casting.

use super::spawn::resolve_raise_dead;
use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::LichAssets;
use crate::game::components::{Acceleration, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::resources::KillStats;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    ANIMATION_MOVE_THRESHOLD_SQ, BanishedModifier, Corpse, FacingDirection, Health, Hitbox,
    MovementSpeed, SleepModifier, TargetingVelocity, Team, TemporaryHitPoints, WalkingAnimation,
    apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::undead::resources::UndeadAssets;

type LichBeamTargetData = (
    Entity,
    &'static Transform,
    &'static Team,
    &'static mut Health,
    &'static Hitbox,
    Option<&'static mut TemporaryHitPoints>,
    Has<crate::game::units::king::components::King>,
);
type LichBeamTargetFilter = (Without<Corpse>, Without<Lich>, Without<Boss>);

/// Checks if it's time to spawn the Lich mid-game.
/// The Lich spawns as an extra wave after all normal waves have been dispatched
/// and every attacker (including staging) is dead.
#[allow(clippy::too_many_arguments)]
pub(super) fn track_soul_power(
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
pub(super) fn lich_phase_transition(
    mut commands: Commands,
    mut query: Query<(Entity, &SoulPower, &mut LichPhase), (With<Lich>, Without<Corpse>)>,
) {
    for (entity, soul_power, mut phase) in &mut query {
        if *phase != LichPhase::Summoning || !soul_power.is_full() {
            continue;
        }

        *phase = LichPhase::Combat;

        commands
            .entity(entity)
            .remove::<LichSummonTimer>()
            .insert(LichFingerOfDeath::new());
    }
}

/// Phase 2 targeting: Updates the Lich's movement targeting toward nearest enemy.
/// Only runs in Combat phase.
pub(super) fn update_lich_targeting(
    mut commands: Commands,
    mut lich_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            &LichPhase,
        ),
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
pub(super) fn lich_movement(
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
    for (transform, mut velocity, mut acceleration, targeting, flow_field, speed, phase) in
        &mut query
    {
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
                let staging = STAGING_POINTS[CENTER_STAGING_INDEX];
                let to_staging = Vec3::new(staging.0 - pos.x, 0.0, staging.1 - pos.z);

                if to_staging.length_squared() > 1.0 {
                    let target_vel = to_staging.normalize() * max_speed;
                    let steer = STEERING_FORCE * time.delta_secs();
                    acceleration.x = (target_vel.x - velocity.x).clamp(-steer, steer)
                        / time.delta_secs().max(0.001);
                    acceleration.z = (target_vel.z - velocity.z).clamp(-steer, steer)
                        / time.delta_secs().max(0.001);
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
                    acceleration.x = (target_vel.x - velocity.x).clamp(-steer, steer)
                        / time.delta_secs().max(0.001);
                    acceleration.z = (target_vel.z - velocity.z).clamp(-steer, steer)
                        / time.delta_secs().max(0.001);
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
/// The King is excluded until he is the last living defender — guards and any
/// other defender are valid targets in the meantime.
pub(super) fn lich_combat_targeting(
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
    king_query: Query<
        Entity,
        (
            With<crate::game::units::king::components::King>,
            Without<Corpse>,
        ),
    >,
) {
    let king_entity = king_query.iter().next();

    // Single pass over defenders: collect non-king entities, and detect whether
    // the King is also alive. The King only becomes a valid FoD target once
    // every other defender (including guards) is dead.
    let mut non_king: Vec<Entity> = Vec::new();
    let mut king_alive = false;
    for (entity, team) in &defenders {
        if *team != Team::Defenders {
            continue;
        }
        if Some(entity) == king_entity {
            king_alive = true;
        } else {
            non_king.push(entity);
        }
    }

    let eligible: Vec<Entity> = if non_king.is_empty() && king_alive {
        king_entity.into_iter().collect()
    } else {
        non_king
    };

    for (mut fod, phase) in &mut lich_query {
        if *phase != LichPhase::Combat {
            continue;
        }
        if fod.target.is_some() || !fod.is_ready() {
            continue;
        }
        if eligible.is_empty() {
            continue;
        }
        let index = (kill_stats.elapsed_time * 1000.0) as usize % eligible.len();
        fod.target = Some(eligible[index]);
    }
}

/// Phase 2: Ticks the Finger of Death cooldown and starts a beam cast wind-up
/// when ready. The actual beam fire is deferred to `tick_lich_casting`. The
/// Lich will not fire until he has closed within `FOD_KING_RANGE` of the King.
pub(super) fn lich_fire_beam(
    time: Res<Time>,
    mut commands: Commands,
    mut lich_query: Query<
        (Entity, &Transform, &mut LichFingerOfDeath, &LichPhase),
        (With<Lich>, Without<Corpse>, Without<LichCasting>),
    >,
    king_query: Query<
        &Transform,
        (
            With<crate::game::units::king::components::King>,
            Without<Lich>,
            Without<Corpse>,
        ),
    >,
) {
    let king_pos = king_query.iter().next().map(|t| t.translation);

    for (entity, lich_transform, mut fod, phase) in &mut lich_query {
        if *phase != LichPhase::Combat {
            continue;
        }

        fod.tick(time.delta_secs());

        if !fod.is_ready() {
            continue;
        }

        // Hold fire until the Lich is within range of the King.
        let Some(king_pos) = king_pos else { continue };
        if lich_transform.translation.distance(king_pos) > FOD_KING_RANGE {
            continue;
        }

        let Some(target_entity) = fod.target else {
            continue;
        };

        commands.entity(entity).insert(LichCasting {
            remaining: LICH_FINGER_OF_DEATH_CAST_DURATION,
            kind: LichCastKind::FingerOfDeath {
                target: target_entity,
            },
        });
        fod.reset(BEAM_COOLDOWN);
    }
}

/// Spawns the Finger of Death beam, applies damage, and triggers the screen
/// darkening effect. Called from `tick_lich_casting` when a FoD cast resolves.
#[allow(clippy::too_many_arguments)]
fn resolve_finger_of_death(
    commands: &mut Commands,
    lich_transform: &Transform,
    lich_hitbox: &Hitbox,
    target: Entity,
    spell_assets: &crate::game::units::wizard::spells::visual_assets::SpellVisualAssets,
    target_query: &Query<&Transform, Without<Lich>>,
    materials: &mut Assets<StandardMaterial>,
    desaturate: &mut MessageWriter<crate::game::crt_effect::ScreenDesaturateMessage>,
    target_health: &mut Query<LichBeamTargetData, LichBeamTargetFilter>,
    king_immune_to_fod: bool,
) {
    use crate::game::units::wizard::spells::finger_of_death::components::{
        FingerOfDeathBeam, FodTalentParams, PendingUndeadRaise,
    };

    let Ok(target_transform) = target_query.get(target) else {
        return;
    };

    let origin = Vec3::new(
        lich_transform.translation.x,
        lich_transform.translation.y + lich_hitbox.height * 0.5,
        lich_transform.translation.z,
    );

    let to_target = target_transform.translation - origin;
    let direction = to_target.normalize_or_zero();
    if direction.length_squared() < 0.5 {
        return;
    }

    let beam_length = BEAM_LENGTH.min(to_target.length() + 200.0);

    let talent_params = FodTalentParams {
        damage: BEAM_DAMAGE,
        beam_width: BEAM_WIDTH,
        beam_width_fired: BEAM_WIDTH,
        ..Default::default()
    };
    let mut beam =
        FingerOfDeathBeam::with_talents(origin, direction, beam_length, 1.0, talent_params);
    beam.has_fired = true;
    beam.cast_progress = 1.0;

    crate::game::units::wizard::spells::finger_of_death::systems::spawn_beam(
        commands,
        spell_assets,
        materials,
        beam,
    );

    desaturate.write(crate::game::crt_effect::ScreenDesaturateMessage);

    let mut kill_positions: Vec<Vec3> = Vec::new();
    for (entity, t_transform, team, mut health, hitbox, temp_hp, is_king) in
        target_health.iter_mut()
    {
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
            // King is fully immune to FoD until enough defenders have fallen.
            if is_king && king_immune_to_fod {
                continue;
            }

            let damage = if is_king {
                BEAM_DAMAGE * KING_FOD_DAMAGE_MULTIPLIER
            } else {
                BEAM_DAMAGE
            };

            let hp_before = health.current;
            apply_spell_damage(
                commands,
                entity,
                &mut health,
                temp_hp.map(|t| t.into_inner()),
                damage,
                DamageType::Necrotic,
                false,
            );

            if hp_before > 0.0 && health.is_dead() {
                kill_positions.push(t_transform.translation);
            }
        }
    }

    if !kill_positions.is_empty() {
        commands.insert_resource(PendingUndeadRaise { kill_positions });
    }
}

/// Decrements the active cast's wind-up timer. When it expires, dispatches to
/// the appropriate spell-resolution helper and removes `LichCasting` so the
/// trigger systems can start the next cast on subsequent frames.
#[allow(clippy::too_many_arguments)]
pub(super) fn tick_lich_casting(
    time: Res<Time>,
    kill_stats: Res<KillStats>,
    mut commands: Commands,
    mut lich_query: Query<
        (Entity, &Transform, &Hitbox, &mut LichCasting),
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
    spell_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    target_query: Query<&Transform, Without<Lich>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut desaturate: MessageWriter<crate::game::crt_effect::ScreenDesaturateMessage>,
    mut target_health: Query<LichBeamTargetData, LichBeamTargetFilter>,
) {
    let delta = time.delta_secs();

    for (entity, transform, hitbox, mut casting) in &mut lich_query {
        casting.remaining -= delta;
        if casting.remaining > 0.0 {
            continue;
        }

        match casting.kind {
            LichCastKind::RaiseDead => {
                resolve_raise_dead(
                    &mut commands,
                    transform.translation,
                    &corpse_query,
                    &undead_assets,
                    &mut materials,
                );
            }
            LichCastKind::FingerOfDeath { target } => {
                let defender_death_ratio = if INITIAL_DEFENDER_COUNT > 0 {
                    kill_stats.defenders_killed as f32 / INITIAL_DEFENDER_COUNT as f32
                } else {
                    0.0
                };
                let king_immune_to_fod = defender_death_ratio < KING_FOD_IMMUNITY_THRESHOLD;

                resolve_finger_of_death(
                    &mut commands,
                    transform,
                    hitbox,
                    target,
                    &spell_assets,
                    &target_query,
                    &mut materials,
                    &mut desaturate,
                    &mut target_health,
                    king_immune_to_fod,
                );
            }
        }

        commands.entity(entity).remove::<LichCasting>();
    }
}

/// Swaps the Lich's bound material to the casting sheet on the frame
/// `LichCasting` is inserted.
pub(super) fn on_lich_cast_started(
    lich_assets: Res<LichAssets>,
    mut added: Query<&mut MeshMaterial3d<StandardMaterial>, (With<Lich>, Added<LichCasting>)>,
) {
    for mut mat in &mut added {
        mat.0 = lich_assets.casting_material.clone();
    }
}

/// Swaps the Lich's bound material back to the floating sheet on the frame
/// `LichCasting` is removed. Split from `on_lich_cast_started` so each system
/// has a single non-conflicting `&mut MeshMaterial3d` query.
pub(super) fn on_lich_cast_ended(
    lich_assets: Res<LichAssets>,
    mut removed: RemovedComponents<LichCasting>,
    mut lich_query: Query<&mut MeshMaterial3d<StandardMaterial>, With<Lich>>,
) {
    for entity in removed.read() {
        if let Ok(mut mat) = lich_query.get_mut(entity) {
            mat.0 = lich_assets.floating_material.clone();
        }
    }
}

/// Custom 2-direction facing for the Lich. Runs after the standard
/// `update_facing_direction` and overrides its result. The Lich shows the
/// rear-facing row only when moving in a 120° arc directly away from the
/// camera; lateral movement collapses to the camera-facing row.
pub(super) fn update_lich_facing(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut lich_query: Query<
        (
            &Velocity,
            &mut FacingDirection,
            &WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (With<Lich>, Without<Corpse>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(cam) = camera_query.single() else {
        return;
    };
    let cam_forward = cam.forward();
    let cam_forward_xz = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();

    for (velocity, mut facing, anim, material_handle) in &mut lich_query {
        let v = Vec3::new(velocity.x, 0.0, velocity.z);
        let speed_sq = v.length_squared();
        if speed_sq < ANIMATION_MOVE_THRESHOLD_SQ {
            continue;
        }

        let speed = speed_sq.sqrt();
        let forward_dot = v.dot(cam_forward_xz);
        // Show the back-of-lich row only when the lich is moving in a 120° arc
        // away from the camera. In Court Wizard, +cam_forward points into the
        // screen, so positive forward_dot means moving away from the viewer.
        let new_facing = if forward_dot > LICH_BACK_FACING_THRESHOLD * speed {
            FacingDirection::Back
        } else {
            FacingDirection::Forward
        };

        if *facing != new_facing {
            *facing = new_facing;
            if let Some(mat) = materials.get_mut(material_handle) {
                mat.uv_transform = anim.uv_transform(new_facing);
            }
        }
    }
}

/// Hovers the Lich above the ground with a subtle sinusoidal bob. Writes
/// `transform.translation.y` directly each frame, layering the bob on top of
/// the spawn-time base Y. Should run after movement is applied.
pub(super) fn update_lich_float(
    time: Res<Time>,
    mut lich_query: Query<(&LichFloatBase, &mut Transform), (With<Lich>, Without<Corpse>)>,
) {
    let bob = LICH_FLOAT_AMPLITUDE
        * (time.elapsed_secs() * LICH_FLOAT_FREQUENCY_HZ * std::f32::consts::TAU).sin();
    for (base, mut transform) in &mut lich_query {
        transform.translation.y = base.base_y + LICH_FLOAT_BASE_OFFSET + bob;
    }
}
