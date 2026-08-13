//! Shared cadence and parameters for infusion tick systems.
//!
//! Every infusion has two faces — the burst it fires the moment a spell lands on
//! the crystal, and the weaker effect it repeats for the rest of its life. Rather
//! than write two functions per infusion, each infusion gets one system that asks
//! [`infusion_should_activate`] whether to run and whether this run is the burst.

use bevy::prelude::*;

use super::super::components::ArcaneCrystal;
use super::super::constants::*;
use super::kinds::{CrystalInfusion, InfusionFamily};
use super::modifiers::{CrystalEnraged, CrystalHastened};
use super::run_conditions::is_infused;

/// The crystal query every infusion tick system shares.
///
/// Written once here rather than spelled out in each system: the tuple is wide
/// enough to trip `clippy::type_complexity`, and a system that got the filter
/// wrong would silently drive the remote peer's ghost crystal.
pub(crate) type InfusedCrystals<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut ArcaneCrystal,
        Has<CrystalHastened>,
        Has<CrystalEnraged>,
    ),
    Without<crate::game::multiplayer::components::GhostSpellEffect>,
>;

/// The opening move of every infusion tick: skip crystals holding a different
/// infusion, advance this one's timer, and snapshot the parameters if it fires.
///
/// Crystal Network allows several crystals at once, so the plugin's run condition
/// is only a gate — the per-crystal `is_infused` check here is what stops a
/// Grease crystal running the Sleep tick.
pub(crate) fn begin_infusion_tick(
    crystal: &mut ArcaneCrystal,
    infusion: CrystalInfusion,
    hastened: bool,
    enraged: bool,
    delta: f32,
) -> Option<InfusionParams> {
    if !is_infused(crystal, infusion) {
        return None;
    }
    let activation = infusion_should_activate(crystal, infusion, hastened, delta)?;
    Some(InfusionParams::new(crystal, enraged, activation))
}

/// Whether an infusion should do work this frame, and at what strength.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Activation {
    /// The one-shot targeted effect, fired the frame the spell landed.
    Burst,
    /// The repeating effect.
    Ongoing,
}

/// Advances a crystal's infusion timer and reports whether it fires this frame.
///
/// A pending burst always fires immediately and outranks the timer. Continuous
/// infusions fire every frame; ticked ones fire on their interval; the Hastened
/// modifier shortens that interval without changing what is emitted.
pub(crate) fn infusion_should_activate(
    crystal: &mut ArcaneCrystal,
    infusion: CrystalInfusion,
    hastened: bool,
    delta: f32,
) -> Option<Activation> {
    if crystal.infusion_burst_pending {
        crystal.infusion_burst_pending = false;
        crystal.auto_cast_timer = 0.0;
        return Some(Activation::Burst);
    }

    if infusion.family() == InfusionFamily::Continuous {
        return Some(Activation::Ongoing);
    }

    let interval = if hastened {
        infusion.interval() / HASTENED_INTERVAL_DIVISOR
    } else {
        infusion.interval()
    };

    crystal.auto_cast_timer += delta;
    if crystal.auto_cast_timer >= interval {
        crystal.auto_cast_timer = 0.0;
        crystal.trigger_pulse();
        return Some(Activation::Ongoing);
    }
    None
}

/// For infusions that keep one long-lived source entity alive rather than
/// re-emitting: drops dead entries and reports whether a replacement is needed.
///
/// Returns `false` while the source is still standing, so the caller does nothing.
pub(crate) fn needs_sustained_source(
    crystal: &mut ArcaneCrystal,
    live_spawns: &Query<(), With<super::super::components::CrystalOwned>>,
) -> bool {
    crystal
        .infusion_spawns
        .retain(|spawned| live_spawns.get(*spawned).is_ok());
    if !crystal.infusion_spawns.is_empty() {
        return false;
    }
    // The burst and the ongoing effect are the same thing here — one source
    // object — so consume the pending burst rather than firing twice.
    crystal.infusion_burst_pending = false;
    crystal.trigger_pulse();
    true
}

/// The crystal state an infusion system needs, snapshotted so the crystal query
/// can be released before spawning or damaging.
#[derive(Clone, Copy)]
pub(crate) struct InfusionParams {
    pub position: Vec3,
    pub range: f32,
    pub empowerment: f32,
    pub damage_mult: f32,
    pub count_mult: f32,
    pub activation: Activation,
}

impl InfusionParams {
    pub(crate) fn new(crystal: &ArcaneCrystal, enraged: bool, activation: Activation) -> Self {
        Self {
            position: crystal.position,
            range: crystal.range,
            empowerment: crystal.empowerment,
            damage_mult: if enraged {
                crystal.damage_mult * ENRAGED_DAMAGE_MULT
            } else {
                crystal.damage_mult
            },
            count_mult: crystal.count_mult,
            activation,
        }
    }

    /// Picks the burst or ongoing value for this activation.
    pub(crate) fn pick<T>(&self, burst: T, ongoing: T) -> T {
        match self.activation {
            Activation::Burst => burst,
            Activation::Ongoing => ongoing,
        }
    }

    /// Picks a count for this activation and applies the crystal's count
    /// multiplier, so Overcharged Matrix widens infusions as well as emitters.
    pub(crate) fn pick_count(&self, burst: usize, ongoing: usize) -> usize {
        let base = self.pick(burst, ongoing);
        ((base as f32 * self.count_mult).ceil() as usize).max(1)
    }

    /// True when this activation is the one-shot targeted effect.
    pub(crate) fn is_burst(&self) -> bool {
        self.activation == Activation::Burst
    }
}
