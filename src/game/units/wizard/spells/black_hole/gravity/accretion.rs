use super::super::components::{BlackHole, UnitInBlackHole};
use super::super::constants::*;
use crate::game::units::components::{
    Corpse, Health, SlowMovementModifier, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
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

/// Applies damage to units touching the black hole sphere.
///
/// Damage increases over time for units that remain in contact.
/// Supports Event Horizon (double damage in inner zone) and Void Siphon (healing).
pub(crate) fn apply_black_hole_damage(
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
                    black_hole.damage_type,
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
pub(crate) fn remove_units_from_black_hole(
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
pub(crate) fn apply_crushing_pressure(
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
pub(crate) fn apply_dimensional_rift(
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
                    black_hole.damage_type,
                    has_spell_shield,
                    caster_team,
                    *team,
                );
            }
        }
    }
}
