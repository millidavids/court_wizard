use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::components::{Corpse, Health, Team};
use crate::game::units::wizard::spells::utils::{local_player_team, xz_distance};
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::session::MultiplayerSession;

// ===== Talent Param Computation =====

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> ArcaneCrystalTalentParams {
    let mut params = ArcaneCrystalTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    // Tier 1
    match talents.get_selection(
        crate::game::units::wizard::components::Spell::ArcaneCrystal,
        0,
    ) {
        Some(0) => params.damage_mult = REFINED_FACETS_DAMAGE_MULT,
        Some(1) => params.range_mult = WIDER_PRISM_RANGE_MULT,
        Some(2) => params.duration_mult = ENDURING_CRYSTAL_DURATION_MULT,
        _ => {}
    }

    // Tier 2
    match talents.get_selection(
        crate::game::units::wizard::components::Spell::ArcaneCrystal,
        1,
    ) {
        Some(0) => params.count_mult = OVERCHARGED_MATRIX_COUNT_MULT,
        Some(1) => params.resonance_cascade = true,
        Some(2) => params.spell_echo = true,
        _ => {}
    }

    // Tier 3
    match talents.get_selection(
        crate::game::units::wizard::components::Spell::ArcaneCrystal,
        2,
    ) {
        Some(0) => params.crystal_network = true,
        Some(1) => params.prismatic_explosion = true,
        Some(2) => params.auto_crystal = true,
        _ => {}
    }

    params
}

/// Applies the count multiplier to a base count, rounding up.
pub(crate) fn scaled_count(base: usize, count_mult: f32) -> usize {
    (base as f32 * count_mult).ceil() as usize
}

/// Returns 2 if Spell Echo triggers (30% chance), 1 otherwise.
pub(crate) fn spell_echo_multiplier(rng: &mut impl Rng, spell_echo: bool) -> usize {
    if spell_echo && rng.random::<f32>() < SPELL_ECHO_CHANCE {
        return 2;
    }
    1
}

/// Increments resonance cascade counter if the component is present.
pub(crate) fn increment_resonance(resonance: &mut Option<Mut<ResonanceCascade>>) {
    if let Some(res) = resonance {
        res.absorptions += 1;
    }
}

// ===== Frame Reset =====

/// Clears per-frame absorption flags on all crystals.
/// Guarded to avoid triggering Bevy change detection when already false.
pub(crate) fn clear_absorption_flags(mut crystals: Query<&mut ArcaneCrystal>) {
    for mut crystal in &mut crystals {
        if crystal.just_absorbed {
            crystal.just_absorbed = false;
        }
    }
}

// ===== Helper Functions =====

/// Computes beam direction and length for a crystal beam.
/// The beam slopes from crystal height (origin.y) to Y=0 at max_range,
/// with its XZ direction aimed toward the target.
pub(crate) fn crystal_beam_geometry(origin: Vec3, target: Vec3, max_range: f32) -> (Vec3, f32) {
    let origin_xz = Vec3::new(origin.x, 0.0, origin.z);
    let target_xz = Vec3::new(target.x, 0.0, target.z);
    let xz_dir = (target_xz - origin_xz).normalize_or(Vec3::X);
    let end_point = origin_xz + xz_dir * max_range; // Y=0 at range edge
    let direction = (end_point - origin).normalize();
    let length = origin.distance(end_point);
    (direction, length)
}

/// Finds random targets within range of a position.
/// Returns up to `count` random targets from any team (spells are indiscriminate).
pub(crate) fn find_random_targets_in_range(
    rng: &mut impl Rng,
    crystal_pos: Vec3,
    range: f32,
    count: usize,
    units: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
) -> Vec<(Entity, Vec3)> {
    let mut candidates: Vec<(Entity, Vec3)> = units
        .iter()
        .filter(|(_, transform)| xz_distance(crystal_pos, transform.translation) <= range)
        .map(|(entity, transform)| (entity, transform.translation))
        .collect();

    // Shuffle and take up to count
    let len = candidates.len();
    for i in (1..len).rev() {
        let j = rng.random_range(0..=i);
        candidates.swap(i, j);
    }
    candidates.truncate(count);
    candidates
}

/// Returns the hostile team set for whichever peer is running this code, based on
/// the CASTER'S team (not just "am I the guest"). The versus guest commands
/// `Attackers`, so its enemies are `Defenders`; SP, the host, and the **co-op
/// guest** all command `Defenders`, so their enemies are `Attackers` — that's what
/// keeps a co-op guest's crystal hitting the enemy wave instead of its own army.
pub(crate) fn crystal_target_teams(
    session: Option<&MultiplayerSession>,
) -> crate::game::units::wizard::spells::magic_missile::components::TargetTeams {
    use crate::game::units::components::Team;
    use crate::game::units::wizard::spells::magic_missile::components::TargetTeams;
    if local_player_team(session) == Team::Attackers {
        TargetTeams::DefendersAndUndead
    } else {
        TargetTeams::AttackersAndUndead
    }
}

/// Finds random enemy targets within range, restricted to the teams the
/// caster considers hostile. In MP the guest's enemies are `Defenders` and
/// the host's enemies are `Attackers`, so the team filter must be supplied
/// by the calling system after reading `PeerId`.
pub(crate) fn find_random_enemies_in_range(
    rng: &mut impl Rng,
    crystal_pos: Vec3,
    range: f32,
    count: usize,
    units: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
    target_teams: crate::game::units::wizard::spells::magic_missile::components::TargetTeams,
) -> Vec<(Entity, Vec3)> {
    let mut candidates: Vec<(Entity, Vec3)> = units
        .iter()
        .filter(|(_, _, team)| target_teams.matches(team))
        .filter(|(_, transform, _)| xz_distance(crystal_pos, transform.translation) <= range)
        .map(|(entity, transform, _)| (entity, transform.translation))
        .collect();

    let len = candidates.len();
    for i in (1..len).rev() {
        let j = rng.random_range(0..=i);
        candidates.swap(i, j);
    }
    candidates.truncate(count);
    candidates
}
