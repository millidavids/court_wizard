use bevy::prelude::*;

use crate::game::run_conditions::{any_exist, is_spell_effects_active, is_warglock};
use crate::state::{InGameState, MultiplayerGameState};

use super::components::*;
use super::fire::FlameGroundFire;
use super::messages::{ReloadMessage, SelectGunMessage};
use super::replication;
use super::systems::*;

/// Plugin for the Warglock (gunslinger) wizard archetype.
pub(in crate::game) struct GunslingerPlugin;

impl Plugin for GunslingerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SelectGunMessage>()
            .add_message::<ReloadMessage>()
            // Initialize gun state on entering gameplay (SP + MP)
            .add_systems(
                OnEnter(InGameState::Running),
                init_gun_state.run_if(is_warglock),
            )
            .add_systems(
                OnEnter(MultiplayerGameState::Running),
                init_gun_state.run_if(is_warglock),
            )
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                reset_gun_state.run_if(is_warglock),
            )
            .add_systems(
                OnEnter(MultiplayerGameState::ScoreScreen),
                reset_gun_state.run_if(is_warglock),
            )
            // Gun selection (from action bar) and reload input
            .add_systems(
                Update,
                (
                    process_gun_selection,
                    handle_reload_key,
                    process_manual_reload,
                )
                    .chain()
                    .run_if(is_spell_effects_active)
                    .run_if(is_warglock),
            )
            // Gun timers tick the LOCAL player's fire cooldowns / reloads.
            // `GunState` is a per-peer local resource, so this must run on both
            // peers — otherwise the guest Warglock's `fire_cooldown` never
            // decrements and each gun jams after one shot. Use
            // `is_spell_effects_active` (NOT `is_local_wizard_active`) to match
            // the firing systems: it covers both MP peers AND single-player's
            // Urgent-mode menus, so the timers don't freeze while the spell
            // book / cauldron is open in Urgent mode.
            .add_systems(
                Update,
                (tick_gun_timers, auto_reload)
                    .chain()
                    .run_if(is_spell_effects_active)
                    .run_if(is_warglock),
            )
            // Firing systems (one per gun, each checks selected gun internally)
            .add_systems(
                Update,
                (
                    fire_machine_gun,
                    fire_magnum,
                    fire_rocket,
                    fire_shotgun,
                    fire_flamethrower,
                )
                    .run_if(is_spell_effects_active)
                    .run_if(is_warglock),
            )
            // Hitscan collision — process rays spawned by firing systems
            .add_systems(
                Update,
                check_hitscan_collisions
                    .run_if(any_exist::<HitscanRay>())
                    .run_if(is_spell_effects_active),
            )
            // Visual tracer movement and cleanup
            .add_systems(
                Update,
                (move_tracers, despawn_distant_tracers)
                    .chain()
                    .run_if(any_exist::<BulletTracer>())
                    .run_if(is_spell_effects_active),
            )
            // Flame particle systems
            .add_systems(
                Update,
                (
                    update_flame_particles,
                    check_flame_collisions,
                    despawn_expired_flames,
                )
                    .chain()
                    .run_if(any_exist::<FlameParticle>())
                    .run_if(is_spell_effects_active),
            )
            // Fire+smoke emission for in-flight flame projectiles
            .add_systems(
                Update,
                emit_flame_particle_vfx
                    .run_if(any_exist::<FlameParticle>())
                    .run_if(is_spell_effects_active),
            )
            // Fire+smoke emission for ground patches left behind by flames
            .add_systems(
                Update,
                emit_flame_ground_fire_vfx
                    .run_if(any_exist::<FlameGroundFire>())
                    .run_if(is_spell_effects_active),
            )
            // VFX
            .add_systems(
                Update,
                update_muzzle_flashes
                    .run_if(any_exist::<MuzzleFlash>())
                    .run_if(is_spell_effects_active),
            )
            // ── Multiplayer replication (full visual fidelity) ───────────
            // Ship locally-spawned gun visuals + shot SFX to the opponent. The
            // emit systems watch `Added<…>` on the local Warglock's own visuals,
            // so they run on whichever peer is the Warglock
            // (`is_warglock` + `is_spell_effects_active`).
            .add_systems(
                Update,
                (
                    replication::replicate_gun_muzzle_flashes.run_if(any_exist::<MuzzleFlash>()),
                    replication::replicate_gun_tracers.run_if(any_exist::<BulletTracer>()),
                    replication::replicate_gun_flames.run_if(any_exist::<FlameParticle>()),
                )
                    .run_if(is_spell_effects_active)
                    .run_if(is_warglock),
            )
            // Tag the persistent flamethrower ground fire for the spell-effect
            // snapshot. Runs on whichever peer is the Warglock (the snapshot
            // collector ships effects from both peers), and only ever matches
            // that peer's own real patches — ghost flames never spawn ground
            // fire, so there is nothing to mis-tag on the opponent.
            .add_systems(
                Update,
                replication::tag_flame_ground_fire_for_replication
                    .run_if(any_exist::<FlameGroundFire>())
                    .run_if(is_spell_effects_active)
                    .run_if(is_warglock),
            );
    }
}
