//! Per-infusion behavior: what a crystal projects once a spell has charged it.
//!
//! Explicit re-exports rather than globs — `systems.rs` glob-imports this module,
//! and with one file per infusion the burst/tick function names would otherwise
//! collide across two dozen modules.

mod auras;
mod control;
mod driver;
mod exotic;
mod kinds;
mod modifiers;
mod run_conditions;
mod support;
mod sustained;
mod utility;
mod zones;

pub(crate) use auras::{
    tick_battle_hymn_infusion, tick_berserker_rage_infusion, tick_guardian_circle_infusion,
};
pub(crate) use control::{
    tick_entangle_infusion, tick_fog_cloud_infusion, tick_plague_wind_infusion, tick_sleep_infusion,
};
pub(crate) use exotic::{
    tick_banishment_infusion, tick_black_hole_infusion, tick_teleport_infusion,
};
pub(crate) use kinds::{CrystalCharge, CrystalInfusion, InfusionFamily};
pub(crate) use modifiers::{
    CrystalAnchored, CrystalEnraged, CrystalHastened, CrystalWarded, apply_modifier,
    tick_enraged_lifetime,
};
pub(crate) use run_conditions::crystal_infused_with;
pub(crate) use support::{tick_healing_plume_infusion, tick_mark_of_death_infusion};
pub(crate) use sustained::{tick_lightning_rod_infusion, tick_squall_infusion};
pub(crate) use utility::{tick_raise_the_dead_infusion, tick_telekinesis_infusion};
pub(crate) use zones::{tick_grease_infusion, tick_spike_growth_infusion};
