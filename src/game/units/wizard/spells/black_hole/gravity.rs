//! Black hole gravity, damage, and persistent effects.

use super::components::{BlackHole, BlackHoleSfx, UnitInBlackHole};
use super::constants::*;
use crate::game::components::Acceleration;
use crate::game::units::components::{
    Corpse, Health, SlowMovementModifier, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{Spell, Wizard};
use crate::game::units::wizard::spells::utils::{PendingDefenderHeal, local_player_team};
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

type DimensionalRiftUnitData = (
    Entity,
    &'static mut Transform,
    &'static mut Health,
    Option<&'static mut TemporaryHitPoints>,
    Has<SpellShield>,
    &'static Team,
);
type DimensionalRiftUnitFilter = (
    With<Team>,
    Without<Wizard>,
    Without<Corpse>,
    Without<BlackHole>,
);

pub(super) fn apply_gravitational_forces(
    // No `Without<GhostSpellEffect>` filter — the host runs this system, and
    // its `Acceleration` writes only land on host-owned units. When the guest
    // casts a black hole, the host has only a `GhostSpellEffect`-tagged copy
    // of it; excluding ghosts here would mean nobody applies gravity for
    // guest-cast black holes (the guest's own `apply_gravitational_forces`
    // doesn't run — it lives in `MovementCalculationSet` which is host-only).
    mut black_holes: Query<&mut BlackHole>,
    mut units: Query<
        (&Transform, &mut Acceleration),
        (
            With<Team>,
            Without<Wizard>,
            Without<BlackHole>,
            Without<Corpse>,
        ),
    >,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for mut black_hole in black_holes.iter_mut() {
        // Update black hole timers
        black_hole.update_timers(delta);

        let gravity_strength = black_hole.gravitational_strength();
        let bh_pos = black_hole.position;

        for (transform, mut acceleration) in units.iter_mut() {
            let unit_pos = transform.translation;
            let to_black_hole = bh_pos - unit_pos;
            let distance = to_black_hole.length();

            // Only apply forces within gravity range and avoid division by zero
            if distance > 0.01 && distance <= GRAVITY_RANGE {
                // Use inverse square law for realistic gravity that grows stronger with proximity
                let distance_factor = 1.0 / (distance * distance);
                let pull_strength = (gravity_strength * distance_factor).min(MAX_FORCE_CLAMP);
                let direction = to_black_hole.normalize();

                // Apply gravitational force to acceleration
                // This will be integrated into velocity and applied to transform by apply_unit_movement
                let force = direction * pull_strength;
                acceleration.add_force(force);
            }
        }
    }
}

/// Applies gravitational forces to corpses and despawns them if they touch the black hole.
///
/// Corpses are pulled by the same gravitational forces as living units.
/// When a corpse intersects the black hole sphere, it is despawned.
pub(super) fn apply_corpse_gravity_and_despawn(
    mut commands: Commands,
    // See `apply_gravitational_forces`: no `GhostSpellEffect` filter so
    // guest-cast black holes (ghost copies on the host) still apply gravity
    // to host-side corpses.
    mut black_holes: Query<&BlackHole>,
    mut corpses: Query<(Entity, &Transform, &mut Acceleration), With<Corpse>>,
) {
    for black_hole in black_holes.iter_mut() {
        let gravity_strength = black_hole.gravitational_strength();
        let bh_pos = black_hole.position;

        for (entity, transform, mut acceleration) in corpses.iter_mut() {
            let corpse_pos = transform.translation;
            let to_black_hole = bh_pos - corpse_pos;
            let distance = to_black_hole.length();

            // Check if corpse intersects the black hole sphere - if so, despawn it
            if black_hole.contains_point(corpse_pos) {
                commands.entity(entity).try_despawn();
                continue;
            }

            // Apply gravitational forces within range
            if distance > 0.01 && distance <= GRAVITY_RANGE {
                // Use inverse square law for realistic gravity that grows stronger with proximity
                let distance_factor = 1.0 / (distance * distance);
                let pull_strength = (gravity_strength * distance_factor).min(MAX_FORCE_CLAMP);
                let direction = to_black_hole.normalize();

                // Apply gravitational force to acceleration
                let force = direction * pull_strength;
                acceleration.add_force(force);
            }
        }
    }
}

/// Applies damage to units touching the black hole sphere.
///
/// Damage increases over time for units that remain in contact.
/// Supports Event Horizon (double damage in inner zone) and Void Siphon (healing).
pub(super) fn apply_black_hole_damage(
    time: Res<Time>,
    mut commands: Commands,
    // No `Without<GhostSpellEffect>` filter: this system is gated to the
    // host in MP (it lives under `MovementCalculationSet` via the plugin's
    // chain), so processing both host-cast and guest-cast (ghost) black
    // holes from here is safe — there's only one peer running it. Damage
    // lands on host-owned `Health`; the guest sees results via the unit
    // snapshot.
    mut black_holes: Query<&mut BlackHole>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Option<&mut UnitInBlackHole>,
            Has<SpellShield>,
            &Team,
        ),
        Without<Wizard>,
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let mut total_siphon_heal = 0.0;
    let mut siphon_origin = Vec3::ZERO;

    for mut black_hole in black_holes.iter_mut() {
        if !black_hole.should_damage() {
            continue;
        }

        let bh_pos = black_hole.position;

        for (entity, transform, mut health, mut temp_hp, tracking, has_spell_shield, team) in
            units.iter_mut()
        {
            let unit_pos = transform.translation;

            if black_hole.contains_point(unit_pos) {
                // Enemy shielded units (the enemy King) are immune; your own
                // shielded King still takes your own black hole's friendly fire.
                if has_spell_shield && caster_team != *team {
                    continue;
                }

                // Track or update time inside
                let damage_multiplier = if let Some(mut tracker) = tracking {
                    tracker.time_inside += time.delta_secs();
                    tracker.damage_multiplier()
                } else {
                    commands.entity(entity).insert(UnitInBlackHole::new());
                    1.0 // First tick, no multiplier yet
                };

                // Event Horizon: double damage in inner zone
                let event_horizon_mult = if black_hole.talent_params.event_horizon {
                    let dist = bh_pos.distance(unit_pos);
                    let inner_radius = black_hole.current_radius * EVENT_HORIZON_INNER_FRACTION;
                    if dist <= inner_radius {
                        EVENT_HORIZON_DAMAGE_MULT
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

                // Apply scaled damage
                let total_damage =
                    black_hole.damage_per_tick() * damage_multiplier * event_horizon_mult;
                apply_spell_damage_with_team(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    total_damage,
                    DamageType::Force,
                    has_spell_shield,
                    caster_team,
                    *team,
                );

                // Accumulate siphon healing
                if black_hole.talent_params.void_siphon {
                    total_siphon_heal += total_damage * VOID_SIPHON_HEAL_FRACTION;
                    siphon_origin = bh_pos;
                }
            }
        }

        // Track talent progress
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::BlackHole, 1);
        }

        black_hole.reset_damage_timer();
    }

    // Defer siphon healing to a separate system to avoid query conflicts
    if total_siphon_heal > 0.0 {
        commands.insert_resource(PendingDefenderHeal {
            amount: total_siphon_heal,
            origin: siphon_origin,
        });
    }
}

/// Removes tracking component when units leave the black hole.
pub(super) fn remove_units_from_black_hole(
    mut commands: Commands,
    black_holes: Query<&BlackHole>,
    units: Query<(Entity, &Transform), With<UnitInBlackHole>>,
) {
    for (entity, transform) in units.iter() {
        let unit_pos = transform.translation;
        let mut is_in_any_black_hole = false;

        for black_hole in black_holes.iter() {
            if black_hole.contains_point(unit_pos) {
                is_in_any_black_hole = true;
                break;
            }
        }

        if !is_in_any_black_hole {
            commands.entity(entity).remove::<UnitInBlackHole>();
        }
    }
}

/// Applies Crushing Pressure slow to units inside the black hole.
pub(super) fn apply_crushing_pressure(
    mut commands: Commands,
    black_holes: Query<&BlackHole>,
    units: Query<(Entity, &Transform), (With<Team>, Without<Wizard>, Without<Corpse>)>,
) {
    // Only run if any black hole has crushing pressure
    let has_crushing = black_holes
        .iter()
        .any(|bh| bh.talent_params.crushing_pressure);
    if !has_crushing {
        return;
    }

    for (entity, transform) in units.iter() {
        let unit_pos = transform.translation;
        let mut inside_crushing = false;

        for black_hole in black_holes.iter() {
            if black_hole.talent_params.crushing_pressure && black_hole.contains_point(unit_pos) {
                inside_crushing = true;
                break;
            }
        }

        if inside_crushing {
            commands.entity(entity).insert(SlowMovementModifier::new(
                CRUSHING_PRESSURE_SLOW,
                CRUSHING_PRESSURE_SLOW_DURATION,
            ));
        }
    }
}

/// Dimensional Rift: periodically teleports all enemies inside to the center and deals burst damage.
pub(super) fn apply_dimensional_rift(
    mut commands: Commands,
    mut black_holes: Query<&mut BlackHole>,
    mut units: Query<DimensionalRiftUnitData, DimensionalRiftUnitFilter>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    for mut black_hole in black_holes.iter_mut() {
        if !black_hole.talent_params.dimensional_rift {
            continue;
        }

        if black_hole.time_since_rift_pulse < DIMENSIONAL_RIFT_INTERVAL {
            continue;
        }

        black_hole.time_since_rift_pulse = 0.0;
        let bh_pos = black_hole.position;

        for (entity, mut transform, mut health, mut temp_hp, has_spell_shield, team) in
            units.iter_mut()
        {
            if black_hole.contains_point(transform.translation) {
                // The shielded King is never yanked to the singularity (preserve
                // its anti-stall teleport immunity). Your own rift still damages
                // your own King; the enemy's rift is blocked by the shield.
                if !has_spell_shield {
                    transform.translation.x = bh_pos.x;
                    transform.translation.z = bh_pos.z;
                }

                apply_spell_damage_with_team(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    DIMENSIONAL_RIFT_DAMAGE * black_hole.empowerment,
                    DamageType::Force,
                    has_spell_shield,
                    caster_team,
                    *team,
                );
            }
        }
    }
}

/// Updates black hole visual scale to match growth animation, adds vibration effect,
/// billboards the circle to face the wizard, and applies pulsing.
///
/// Also ticks `time_alive` and recomputes `current_radius` here so that
/// growth animates on BOTH peers — `apply_gravitational_forces` (which
/// previously owned the timer tick) only runs on the host, leaving the
/// guest's local black hole and ghost black holes frozen at `Vec3::ZERO`
/// scale and invisible.
pub(super) fn update_black_hole_visuals(
    time: Res<Time>,
    mut black_holes: Query<(&mut BlackHole, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (mut black_hole, mut transform) in black_holes.iter_mut() {
        black_hole.time_alive += delta;
        black_hole.calculate_current_radius();
        let growth_factor = (black_hole.time_alive / GROWTH_TIME).min(1.0);

        // Add vibration using sine waves on different axes
        let t = black_hole.time_alive * VIBRATION_FREQUENCY;
        let vibration = Vec3::new(
            (t * 1.0).sin() * VIBRATION_AMPLITUDE,
            (t * 1.7).sin() * VIBRATION_AMPLITUDE,
            (t * 2.3).sin() * VIBRATION_AMPLITUDE,
        );

        // Pulsing scale
        let pulse = 1.0
            + (time.elapsed_secs() * RING_PULSE_FREQUENCY * std::f32::consts::TAU).sin()
                * RING_PULSE_AMPLITUDE;

        let position = black_hole.position + vibration;
        transform.scale = Vec3::splat(black_hole.max_radius * growth_factor * pulse);
        transform.translation = position;
    }
}

/// Despawns black holes when they expire after LIFETIME seconds.
/// Handles Singularity talent: deals collapse damage to all units inside on expiration.
pub(super) fn despawn_expired_black_holes(
    mut commands: Commands,
    // Host-authoritative only — guest's ghost copy is despawned by the
    // snapshot's stale-id cleanup when the host's entity disappears.
    black_holes: Query<
        (Entity, &BlackHole),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (Without<Wizard>, Without<BlackHole>),
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    for (entity, black_hole) in black_holes.iter() {
        if black_hole.is_expired() {
            // Singularity: collapse damage to all units inside
            if black_hole.talent_params.singularity {
                for (unit_entity, transform, mut health, mut temp_hp, has_spell_shield, team) in
                    units.iter_mut()
                {
                    // Enemy shielded King immune; your own King takes friendly fire.
                    if has_spell_shield && caster_team != *team {
                        continue;
                    }
                    if black_hole.contains_point(transform.translation) {
                        apply_spell_damage_with_team(
                            &mut commands,
                            unit_entity,
                            &mut health,
                            temp_hp.as_deref_mut(),
                            SINGULARITY_DAMAGE * black_hole.empowerment,
                            DamageType::Force,
                            has_spell_shield,
                            caster_team,
                            *team,
                        );
                    }
                }
            }

            commands.entity(entity).try_despawn();
        }
    }
}

/// Despawns orphaned black hole sound effects whose parent no longer exists.
pub(super) fn cleanup_black_hole_sfx(
    mut commands: Commands,
    sfx_entities: Query<(Entity, &BlackHoleSfx)>,
    black_holes: Query<&BlackHole>,
) {
    for (entity, sfx) in sfx_entities.iter() {
        if black_holes.get(sfx.black_hole_entity).is_err() {
            commands.entity(entity).try_despawn();
        }
    }
}
