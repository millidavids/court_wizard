use super::super::components::{BlackHole, BlackHoleSfx};
use super::super::constants::*;
use crate::game::units::components::{
    Health, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

/// Updates black hole visual scale to match growth animation, adds vibration effect,
/// billboards the circle to face the wizard, and applies pulsing.
///
/// Also ticks `time_alive` and recomputes `current_radius` here so that
/// growth animates on BOTH peers — `apply_gravitational_forces` (which
/// previously owned the timer tick) only runs on the host, leaving the
/// guest's local black hole and ghost black holes frozen at `Vec3::ZERO`
/// scale and invisible.
pub(crate) fn update_black_hole_visuals(
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
#[allow(clippy::type_complexity)]
pub(crate) fn despawn_expired_black_holes(
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
        (
            Without<Wizard>,
            Without<BlackHole>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
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
                            black_hole.damage_type,
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
pub(crate) fn cleanup_black_hole_sfx(
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
