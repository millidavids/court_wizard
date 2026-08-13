//! Infusions where the crystal *becomes* another spell's source object.
//!
//! Lightning Rod and Squall both work by keeping one long-lived entity alive at
//! the crystal and letting that spell's own systems drive it — no new tick logic,
//! and the behaviour stays in step with the original spell if it is ever retuned.
//! Both are [`InfusionFamily::Continuous`] and idempotent: they re-spawn their
//! source only once it has gone, which also covers a rod reaching its own expiry.

use bevy::prelude::*;

use super::super::components::{ArcaneCrystal, CrystalOwned};
use super::super::constants::*;
use super::super::setup::register_infusion_spawn;
use super::driver::needs_sustained_source;
use super::kinds::CrystalInfusion;
use super::run_conditions::is_infused;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::wizard::spells::lightning_rod::casting::spawn_lightning_rod;
use crate::game::units::wizard::spells::lightning_rod::components::LightningRodTalentParams;
use crate::game::units::wizard::spells::squall::components::{SquallStorm, SquallTalentParams};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;

/// Keeps a lightning rod standing at the crystal for as long as the infusion holds.
///
/// The rod's own strike systems do all the work; the crystal only guarantees one
/// exists. Its lifetime is the crystal's remaining lifetime, so the two expire
/// together rather than the rod outliving the crystal that summoned it.
pub(crate) fn tick_lightning_rod_infusion(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (Entity, &mut ArcaneCrystal),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    live_spawns: Query<(), With<CrystalOwned>>,
) {
    for (entity, mut crystal) in &mut crystals {
        if !is_infused(&crystal, CrystalInfusion::LightningRod) {
            continue;
        }
        if !needs_sustained_source(&mut crystal, &live_spawns) {
            continue;
        }

        let remaining = (crystal.duration - crystal.time_alive).max(0.0);
        let rod = spawn_lightning_rod(
            &mut commands,
            &visual_assets,
            Vec3::new(crystal.position.x, 0.0, crystal.position.z),
            crystal.empowerment,
            remaining,
            LightningRodTalentParams::default(),
        );
        register_infusion_spawn(&mut commands, &mut crystal, entity, rod);
    }
}

/// Sustains an ice storm centred on the crystal.
///
/// Squall is a concentration spell, so this is the payoff: feed a storm to a
/// crystal and it keeps raining after the wizard stops concentrating, freeing the
/// reserved mana. The storm is scaled to the crystal's range.
pub(crate) fn tick_squall_infusion(
    mut commands: Commands,
    // No `Has<CrystalHastened>`: the storm runs on its own cadence, so Haste has
    // nothing here to speed up.
    mut crystals: Query<
        (Entity, &mut ArcaneCrystal),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    live_spawns: Query<(), With<CrystalOwned>>,
) {
    for (entity, mut crystal) in &mut crystals {
        if !is_infused(&crystal, CrystalInfusion::Squall) {
            continue;
        }
        if !needs_sustained_source(&mut crystal, &live_spawns) {
            continue;
        }

        let radius = crystal.range * SQUALL_INFUSION_RADIUS_SCALE;
        let storm = commands
            .spawn((
                SquallStorm::new(
                    Vec3::new(crystal.position.x, 0.0, crystal.position.z),
                    radius,
                    crystal.empowerment * DAMAGE_SCALE,
                    SquallTalentParams::default(),
                ),
                // Matches the hand-cast storm: without this the remote peer sees
                // ice landing out of a clear sky.
                NetworkedSpellEffect {
                    kind: SpellEffectKind::SquallStorm,
                },
                crate::game::components::OnGameplayScreen,
            ))
            .id();
        register_infusion_spawn(&mut commands, &mut crystal, entity, storm);
    }
}
