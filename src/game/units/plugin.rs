use bevy::prelude::*;

use super::components::FrostAccumulation;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};

use super::aerialist::AerialistPlugin;
use super::archer::ArcherPlugin;
use super::assassin::AssassinPlugin;
use super::boss::BossPlugin;
use super::brute::BrutePlugin;
use super::commander::CommanderPlugin;
use super::components::{
    Airborne, BerserkerRageModifier, CombatAnimation, DeathAnimationFinished, DyingAnimation,
    FacingDirection, FearModifier, FireDoT, FogEvasionModifier, FrozenSolidModifier, HasteModifier,
    Knockback, MarkedForDeathModifier, Petrified, PoisonedModifier, PulsingAnimation,
    RemoteFireEffect, RisingAnimation, RootedModifier, SickenedModifier, SlowMovementModifier,
    SmellyModifier, Stunned, TemporaryHitPoints, WalkingAnimation,
};
use super::dispeller::DispellerPlugin;
use super::elite::ElitePlugin;
use super::healer::HealerPlugin;
use super::infantry::InfantryPlugin;
use super::king::KingPlugin;
use super::movement;
use super::ranged_bolt::{self, MagicBolt};
use super::shielder::ShielderPlugin;
use super::systems;
use super::teleporter::TeleporterPlugin;
use super::wizard::WizardPlugin;
use super::wizard::spells::vfx::channel::{self, ChannelParticle, ChannelingCast};
use super::{ApplyTransformsSet, MovementCalculationSet};
use crate::game::terrain::TerrainPlugin;

/// Plugin that coordinates all unit-related sub-plugins.
///
/// Registers sub-plugins for:
/// - Wizard entity (WizardPlugin)
/// - Infantry units on both teams (InfantryPlugin)
/// - Archer units on both teams (ArcherPlugin)
/// - King unit (defender only) (KingPlugin)
///
/// Also registers global unit systems for:
/// - Timed modifier expiration
/// - Movement application (transforms)
pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, super::undead::resources::preload_undead_assets)
            .add_plugins((
                CommanderPlugin,
                ElitePlugin,
                WizardPlugin,
                InfantryPlugin,
                ArcherPlugin,
                AerialistPlugin,
                AssassinPlugin,
                DispellerPlugin,
                HealerPlugin,
                ShielderPlugin,
                BrutePlugin,
                TeleporterPlugin,
                BossPlugin,
                KingPlugin,
            ))
            .add_plugins(TerrainPlugin)
            .configure_sets(
                Update,
                (MovementCalculationSet, ApplyTransformsSet)
                    .chain()
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    systems::update_timed_modifier::<TemporaryHitPoints>,
                    systems::update_timed_modifier::<SlowMovementModifier>,
                    systems::update_frost_accumulation
                        .run_if(any_with_component::<FrostAccumulation>),
                    systems::update_timed_modifier::<RootedModifier>,
                    systems::update_timed_modifier::<HasteModifier>,
                    systems::update_timed_modifier::<Stunned>.run_if(any_with_component::<Stunned>),
                    movement::zero_velocity_for::<Stunned>
                        .after(MovementCalculationSet)
                        .before(ApplyTransformsSet)
                        .run_if(any_with_component::<Stunned>),
                    movement::zero_velocity_for::<Petrified>
                        .after(MovementCalculationSet)
                        .before(ApplyTransformsSet)
                        .run_if(any_with_component::<Petrified>),
                    movement::zero_velocity_for::<ChannelingCast>
                        .after(MovementCalculationSet)
                        .before(ApplyTransformsSet)
                        .run_if(any_with_component::<ChannelingCast>),
                    movement::apply_unit_movement.in_set(ApplyTransformsSet),
                    movement::clear_corpse_velocity.after(movement::apply_unit_movement),
                    channel::update_channel_particles.run_if(any_with_component::<ChannelParticle>),
                    (
                        ranged_bolt::move_magic_bolts,
                        ranged_bolt::check_magic_bolt_collisions,
                    )
                        .chain()
                        .run_if(any_with_component::<MagicBolt>),
                )
                    .run_if(is_gameplay_running),
            )
            // Animation systems run for the guest too so MP ghost units
            // animate from their synthesised Velocity (derived in
            // `apply_state_snapshot` from snapshot-to-snapshot position
            // deltas) — gated on `is_spell_effects_active` rather than
            // `is_gameplay_running` so both peers tick.
            //
            // Ordering:
            // - `.after(ApplyTransformsSet)`: read SP host velocity after
            //   `apply_unit_movement` has integrated it for the frame.
            // - `.after(GuestSnapshotSet)`: read MP guest ghost velocity
            //   after `apply_state_snapshot` has synthesised it from the
            //   latest snapshot. Both `.after()`s resolve vacuously on
            //   peers where the dependency set doesn't run.
            // - `update_combat_animation.after(update_facing_direction)`:
            //   so the combat sprite picks the just-updated directional row.
            .add_systems(
                Update,
                (
                    systems::update_walking_animation
                        .after(ApplyTransformsSet)
                        .after(crate::game::units::GuestSnapshotSet)
                        .run_if(any_with_component::<WalkingAnimation>),
                    systems::update_pulsing_animation
                        .after(ApplyTransformsSet)
                        .run_if(any_with_component::<PulsingAnimation>),
                    systems::update_facing_direction
                        .after(ApplyTransformsSet)
                        .after(crate::game::units::GuestSnapshotSet)
                        .run_if(any_with_component::<FacingDirection>),
                    systems::update_combat_animation
                        .after(ApplyTransformsSet)
                        .after(systems::update_facing_direction)
                        .run_if(any_with_component::<CombatAnimation>),
                    systems::update_dying_animation
                        .after(ApplyTransformsSet)
                        .run_if(any_with_component::<DyingAnimation>),
                    systems::finalize_dying_to_corpse
                        .after(systems::update_dying_animation)
                        .run_if(any_with_component::<DeathAnimationFinished>),
                    systems::update_rising_animation
                        .after(ApplyTransformsSet)
                        .run_if(any_with_component::<RisingAnimation>),
                )
                    .run_if(crate::game::run_conditions::is_spell_effects_active),
            )
            // Spell damage effects must run on both host AND guest so that
            // guest spell DoTs tick and feed damage into the CRDT.
            .add_systems(
                Update,
                (
                    systems::process_pending_damage_effects,
                    systems::update_fire_dot,
                    systems::update_electric_charge,
                    systems::update_electric_arc_visuals,
                    systems::update_poisoned.run_if(any_with_component::<PoisonedModifier>),
                    systems::update_sickened.run_if(any_with_component::<SickenedModifier>),
                    systems::update_timed_modifier::<SmellyModifier>
                        .run_if(any_with_component::<SmellyModifier>),
                    systems::update_persistent_effect_visuals,
                    // Fire on the guest's ghost units is tagged via
                    // `RemoteFireEffect` rather than the real `FireDoT`.
                    // Run the VFX emitter when either marker is present.
                    systems::emit_burning_unit_vfx.run_if(
                        any_with_component::<FireDoT>
                            .or(any_with_component::<RemoteFireEffect>),
                    ),
                )
                    .run_if(is_spell_effects_active),
            )
            .add_systems(
                Update,
                (
                    systems::apply_knockback_effects.run_if(any_with_component::<Knockback>),
                    systems::update_airborne_units.run_if(any_with_component::<Airborne>),
                )
                    .after(ApplyTransformsSet)
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    systems::update_timed_modifier::<MarkedForDeathModifier>
                        .run_if(any_with_component::<MarkedForDeathModifier>),
                    // SleepModifier timer is handled by SleepPlugin::update_sleep_modifiers
                    // BattleHymnModifier timer is handled by BattleHymnPlugin (handles EchoingSong)
                    systems::update_timed_modifier::<BerserkerRageModifier>
                        .run_if(any_with_component::<BerserkerRageModifier>),
                    systems::update_timed_modifier::<FogEvasionModifier>
                        .run_if(any_with_component::<FogEvasionModifier>),
                    systems::update_timed_modifier::<FrozenSolidModifier>
                        .run_if(any_with_component::<FrozenSolidModifier>),
                    systems::update_timed_modifier::<Petrified>
                        .run_if(any_with_component::<Petrified>),
                    systems::update_timed_modifier::<FearModifier>
                        .run_if(any_with_component::<FearModifier>),
                )
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    super::hit_flash::update_hit_flashes
                        .run_if(any_with_component::<super::hit_flash::HitFlash>),
                    super::hit_flash::update_hit_flash_vfx
                        .run_if(any_with_component::<super::hit_flash::HitFlashVfx>),
                )
                    .run_if(is_spell_effects_active),
            );
    }
}
