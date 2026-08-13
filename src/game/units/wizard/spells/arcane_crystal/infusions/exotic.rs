//! Infusions that move or remove units rather than damaging them.

use bevy::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

use super::super::components::ArcaneCrystal;
use super::super::constants::*;
use super::driver::{InfusedCrystals, begin_infusion_tick};
use super::kinds::CrystalInfusion;
use super::run_conditions::is_infused;
use crate::game::components::Acceleration;
use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::multiplayer::components::GhostEntity;
use crate::game::units::components::{BanishedModifier, Corpse, Team};
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::banishment::constants as banish_constants;
use crate::game::units::wizard::spells::black_hole::constants as bh_constants;
use crate::game::units::wizard::spells::utils::{local_player_team, xz_distance};
use crate::networking::session::MultiplayerSession;

/// Banishes an enemy caught in the crystal's range.
///
/// Banished units return where they left, so unlike the hand-cast version this
/// is pure tempo — it removes a unit from the fight and hands it back later.
#[allow(clippy::type_complexity)]
pub(crate) fn tick_banishment_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut crystals: InfusedCrystals,
    targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<Wizard>,
            Without<BanishedModifier>,
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();

    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::Banishment,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };

        // Banishment is the one infusion that stays team-aware: removing your own
        // infantry from the field for eight seconds is not an interesting trade.
        let mut candidates: Vec<Entity> = targets
            .iter()
            .filter(|(_, _, team)| caster_team.is_enemy(team))
            .filter(|(_, transform, _)| {
                xz_distance(params.position, transform.translation) <= params.range
            })
            .map(|(target, _, _)| target)
            .collect();

        candidates.shuffle(&mut game_rng.0);

        let duration = banish_constants::BANISH_DURATION * INFUSION_DURATION_SCALE;
        for target in candidates
            .into_iter()
            .take(params.pick_count(INFUSION_BURST_COUNT, INFUSION_ONGOING_COUNT))
        {
            commands
                .entity(target)
                .insert(BanishedModifier::new(duration));
        }
    }
}

/// Drags a random enemy to the crystal.
///
/// Turns the crystal into a lure: enemies keep arriving at a spot of your
/// choosing, which is worth far more next to a damage crystal than alone.
#[allow(clippy::type_complexity)]
pub(crate) fn tick_teleport_infusion(
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut crystals: InfusedCrystals,
    mut targets: Query<
        (Entity, &mut Transform, &Team),
        (
            Without<Corpse>,
            Without<Wizard>,
            Without<ArcaneCrystal>,
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();

    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::Teleport,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };

        let wanted = params.pick_count(INFUSION_BURST_COUNT, 1);
        let drop_radius = params.range * TELEPORT_DROP_RING_SCALE;

        // Shuffle the eligible set rather than taking whatever the query happens
        // to yield first. Units land on a ring *outside* the "already close
        // enough" threshold, so a deterministic pick re-grabs the same unit on
        // every tick and no one else is ever lured in.
        let mut eligible: Vec<Entity> = targets
            .iter()
            .filter(|(_, transform, team)| {
                if !caster_team.is_enemy(team) {
                    return false;
                }
                let distance = xz_distance(params.position, transform.translation);
                distance <= params.range && distance > drop_radius
            })
            .map(|(entity, _, _)| entity)
            .collect();
        eligible.shuffle(&mut game_rng.0);

        let half_field = BATTLEFIELD_SIZE / 2.0;
        for target in eligible.into_iter().take(wanted) {
            let Ok((_, mut transform, _)) = targets.get_mut(target) else {
                continue;
            };
            // Land them on a ring just off the crystal so a pulled stack does not
            // pile into a single point. Clamped to the battlefield — a crystal
            // near the edge must not deposit units outside it.
            let angle = game_rng.0.random_range(0.0..std::f32::consts::TAU);
            transform.translation.x =
                (params.position.x + angle.cos() * drop_radius).clamp(-half_field, half_field);
            transform.translation.z =
                (params.position.z + angle.sin() * drop_radius).clamp(-half_field, half_field);
        }
    }
}

/// Exerts a steady inward pull on everything in range.
///
/// The crystal borrows the black hole's gravity without its event horizon — it
/// gathers units rather than eating them, which is what makes it worth pairing
/// with a damage crystal.
#[allow(clippy::type_complexity)]
pub(crate) fn tick_black_hole_infusion(
    crystals: Query<
        &ArcaneCrystal,
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    // `With<Team>` is load-bearing: it restricts this to units. Without it the
    // query matches every entity that has a `Transform` — grid tiles, terrain,
    // props — and the pull drags the whole battlefield toward the crystal.
    mut units: Query<
        (&Transform, &mut Acceleration),
        (
            With<Team>,
            Without<Wizard>,
            Without<Corpse>,
            Without<ArcaneCrystal>,
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    for crystal in &crystals {
        if !is_infused(crystal, CrystalInfusion::BlackHole) {
            continue;
        }
        for (transform, mut acceleration) in &mut units {
            let to_crystal = Vec3::new(
                crystal.position.x - transform.translation.x,
                0.0,
                crystal.position.z - transform.translation.z,
            );
            let distance = to_crystal.length();
            if distance > crystal.range || distance < 1.0 {
                continue;
            }
            // Feed the movement system rather than writing `Transform` directly,
            // exactly as the real black hole does. Moving transforms by hand
            // would shove units through walls and fight the flocking code.
            // Clamp last, so the constant means what it says. Clamping before
            // the scale made the effective ceiling half the documented value.
            let pull = (bh_constants::BASE_GRAVITY_STRENGTH / (distance * distance) * DAMAGE_SCALE)
                .min(CRYSTAL_GRAVITY_MAX_PULL);
            acceleration.add_force(to_crystal.normalize() * pull);
        }
    }
}
