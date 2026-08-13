//! Healing and damage-amplification infusions.

use bevy::prelude::*;

use super::super::components::ArcaneCrystal;
use super::super::constants::*;
use super::super::setup::register_infusion_spawn;
use super::driver::{InfusedCrystals, begin_infusion_tick, needs_sustained_source};
use super::kinds::CrystalInfusion;
use super::run_conditions::is_infused;
use crate::game::multiplayer::components::GhostEntity;
use crate::game::units::components::{Corpse, Health, MarkedForDeathModifier, Team};
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::healing_plume::aura::spawn_healing_plume_zone;
use crate::game::units::wizard::spells::healing_plume::components::HealingPlumeTalentParams;
use crate::game::units::wizard::spells::mark_of_death::components::{
    ActiveMarkOfDeath, MarkTalentFlags,
};
use crate::game::units::wizard::spells::mark_of_death::constants as mod_constants;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;

/// Keeps a healing zone alive over the crystal's range.
///
/// Continuous with an idempotent respawn, like the sustained infusions: the zone
/// has its own lifetime, and the crystal simply replaces it when it lapses.
pub(crate) fn tick_healing_plume_infusion(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (Entity, &mut ArcaneCrystal),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    live_spawns: Query<(), With<super::super::components::CrystalOwned>>,
) {
    for (entity, mut crystal) in &mut crystals {
        if !is_infused(&crystal, CrystalInfusion::HealingPlume) {
            continue;
        }
        if !needs_sustained_source(&mut crystal, &live_spawns) {
            continue;
        }

        let zone = spawn_healing_plume_zone(
            &mut commands,
            &visual_assets,
            Vec3::new(crystal.position.x, 0.0, crystal.position.z),
            crystal.range,
            crystal.empowerment * INFUSION_DURATION_SCALE,
            &HealingPlumeTalentParams::default(),
            1.0,
        );
        register_infusion_spawn(&mut commands, &mut crystal, entity, zone);
    }
}

/// Keeps the sturdiest enemy in range marked for death.
///
/// This infusion deals no damage itself — it makes everything *else* hit harder,
/// which is a role no other crystal state fills.
#[allow(clippy::type_complexity)]
pub(crate) fn tick_mark_of_death_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut crystals: InfusedCrystals,
    candidates: Query<
        (Entity, &Transform, &Health, &Team),
        (
            Without<Corpse>,
            Without<Wizard>,
            Without<ActiveMarkOfDeath>,
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    // `With<MarkedForDeathModifier>` matters on the guest: ghost units carry a
    // bare `ActiveMarkOfDeath` mirrored from the host, and counting those would
    // make the crystal believe its quota was already met and never mark anything.
    marked: Query<(Entity, &Transform), (With<ActiveMarkOfDeath>, With<MarkedForDeathModifier>)>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();
    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::MarkOfDeath,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };

        // Hold at most one mark from the ongoing tick; the burst marks a spread.
        let already_marked = marked
            .iter()
            .filter(|(_, transform)| {
                xz_distance(params.position, transform.translation) <= params.range
            })
            .count();
        let wanted = params.pick_count(INFUSION_BURST_COUNT, 1);
        if already_marked >= wanted {
            continue;
        }

        let mut in_range: Vec<(Entity, f32)> = candidates
            .iter()
            // Enemies only. This is a damage-*taken* amplifier and it picks the
            // highest-health unit in range — without the team check it locks
            // onto the King, whose 200 HP outranks every ordinary unit and whose
            // death ends the run instantly.
            .filter(|(_, _, _, team)| caster_team.is_enemy(team))
            .filter(|(_, transform, _, _)| {
                xz_distance(params.position, transform.translation) <= params.range
            })
            .map(|(target, _, health, _)| (target, health.current))
            .collect();
        // Sturdiest first — marking the unit that will take the most hits is
        // worth more than marking whatever happens to be nearest.
        in_range.sort_by(|a, b| b.1.total_cmp(&a.1));

        let amplification =
            mod_constants::DAMAGE_AMPLIFICATION * INFUSION_DURATION_SCALE * params.empowerment;
        let duration = mod_constants::MARK_DURATION * INFUSION_DURATION_SCALE;

        for (target, _) in in_range.into_iter().take(wanted - already_marked) {
            commands.entity(target).insert((
                MarkedForDeathModifier::new(amplification, duration),
                ActiveMarkOfDeath,
                MarkTalentFlags {
                    amplification,
                    ..Default::default()
                },
            ));
        }
    }
}
