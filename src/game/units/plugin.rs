use bevy::prelude::*;

use super::components::FrostAccumulation;
use crate::game::run_conditions::{
    any_exist, is_gameplay_running, is_not_mp_setup_phase, is_spell_effects_active,
};

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
    RemoteFireEffect, RemoteTempHpEffect, RisingAnimation, RootedModifier, Shocked,
    SickenedModifier, SlowMovementModifier, SmellyModifier, Stunned, TemporaryHitPoints,
    WalkingAnimation,
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
            .init_resource::<super::hit_feedback::HitSfxBudget>()
            .configure_sets(
                Update,
                (MovementCalculationSet, ApplyTransformsSet)
                    .chain()
                    .run_if(is_gameplay_running)
                    // Freeze all unit movement during the multiplayer setup stage.
                    .run_if(is_not_mp_setup_phase),
            )
            .add_systems(
                Update,
                (
                    systems::update_timed_modifier::<TemporaryHitPoints>
                        .run_if(any_with_component::<TemporaryHitPoints>),
                    systems::update_timed_modifier::<SlowMovementModifier>
                        .run_if(any_with_component::<SlowMovementModifier>),
                    systems::update_frost_accumulation
                        .run_if(any_with_component::<FrostAccumulation>),
                    systems::update_timed_modifier::<RootedModifier>
                        .run_if(any_with_component::<RootedModifier>),
                    systems::update_timed_modifier::<HasteModifier>
                        .run_if(any_with_component::<HasteModifier>),
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
            // Status-effect bookkeeping runs on BOTH peers — but the
            // affected queries skip ghost units (see `Without<GhostEntity>`
            // inside each system). That gives us:
            //   • Local wizards on each peer process their own DoTs locally
            //     (e.g. guest takes friendly-fire fireball → guest's wizard
            //     burns) — without this the wizard's PendingDamageEffect would
            //     accumulate forever with no FireDoT ever applied.
            //   • Host's authoritative units run the full SP pipeline as in
            //     SP — process → FireDoT → tick damage → CRDT.
            //   • Guest's GHOST units are NOT processed locally. Instead,
            //     `forward_spell_hits_to_host` ships a `SpellHitUnit` message
            //     to the host, which applies the effect on its authoritative
            //     copy; the resulting status bit comes back via the snapshot
            //     and renders as `RemoteFireEffect` on the ghost.
            .add_systems(
                Update,
                (
                    systems::process_pending_damage_effects,
                    systems::update_fire_dot.run_if(any_with_component::<FireDoT>),
                    systems::update_shocked.run_if(any_with_component::<Shocked>),
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
                            .or_else(any_with_component::<RemoteFireEffect>),
                    ),
                    // Temp-HP shield feet ring — real component on this peer's
                    // units, snapshot-mirrored marker on guest ghosts. The
                    // ring-indicator condition keeps update running after the
                    // last shield expires so orphaned rings despawn.
                    (systems::spawn_temp_hp_rings, systems::update_temp_hp_rings)
                        .chain()
                        .run_if(
                            any_with_component::<TemporaryHitPoints>
                                .or_else(any_with_component::<RemoteTempHpEffect>)
                                .or_else(any_with_component::<systems::TempHpRingIndicator>),
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
                    .run_if(is_gameplay_running)
                    // Freeze spell-driven displacement (knockback/airborne launches)
                    // during the multiplayer setup stage, matching the movement sets.
                    // This also stops update_airborne_units from stacking a landing
                    // PendingDamageEffect on the immune, frozen armies.
                    .run_if(is_not_mp_setup_phase),
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
                    // Deliberately unordered. `drive_spell_hit_feedback` is the
                    // sole remover of `PendingSpellHit`, so it observes every
                    // marker exactly once whenever the deferred insert lands —
                    // no ordering edge, and therefore no injected
                    // `ApplyDeferred` barrier in Update.
                    super::hit_feedback::drive_spell_hit_feedback
                        .run_if(any_with_component::<super::components::PendingSpellHit>),
                    // Unlike every other `update_timed_modifier` (which run
                    // under `is_gameplay_running`), this one sits under
                    // `is_spell_effects_active` so the multiplayer guest
                    // flashes too.
                    systems::update_timed_modifier::<super::hit_feedback::SpellHitCooldown>
                        .run_if(any_with_component::<super::hit_feedback::SpellHitCooldown>),
                    super::hit_feedback::update_hit_flashes
                        .run_if(any_with_component::<super::hit_feedback::HitFlash>),
                    super::hit_feedback::update_hit_flash_vfx
                        .run_if(any_with_component::<super::hit_feedback::HitFlashVfx>),
                )
                    .run_if(is_spell_effects_active),
            )
            // Staging shields are SP-only (StagingAttacker never exists in
            // multiplayer), so gate on AppState::InGame. Note this is the
            // ONLY gate on purpose: pause/menu states must not suspend the
            // clear system, or RemovedComponents events read while paused
            // could be missed and leak glows.
            .add_systems(
                Update,
                (
                    super::staging_shield::apply_staging_shield_glow
                        .run_if(any_exist::<crate::game::pathfinding::StagingAttacker>()),
                    // Gate the clear on the glow marker, not StagingAttacker:
                    // the frame the last wave activates has zero stagers
                    // left, which would skip this system and leak that
                    // frame's RemovedComponents events. The activating units
                    // still carry the marker, so this gate is self-consistent.
                    super::staging_shield::clear_staging_shield_glow
                        .run_if(any_with_component::<super::staging_shield::StagingShieldGlow>),
                    #[cfg(debug_assertions)]
                    super::staging_shield::warn_on_staging_spell_effects
                        .run_if(any_exist::<crate::game::pathfinding::StagingAttacker>()),
                )
                    .run_if(in_state(crate::state::AppState::InGame)),
            );
    }
}
