use bevy::prelude::*;

use crate::game::run_conditions::{any_exist, is_gameplay_running, is_warglock, is_spell_effects_active};
use crate::state::InGameState;

use super::components::*;
use super::messages::{ReloadMessage, SelectGunMessage};
use super::systems::*;

/// Plugin for the Warglock (gunslinger) wizard archetype.
pub(in crate::game) struct GunslingerPlugin;

impl Plugin for GunslingerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SelectGunMessage>()
            .add_message::<ReloadMessage>()
            // Initialize gun state on entering gameplay
            .add_systems(
                OnEnter(InGameState::Running),
                init_gun_state.run_if(is_warglock),
            )
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
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
            // Gun timers (always tick when gameplay is running)
            .add_systems(
                Update,
                (tick_gun_timers, auto_reload)
                    .chain()
                    .run_if(is_gameplay_running)
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
            // Bullet hit flash
            .add_systems(
                Update,
                (update_bullet_hit_flashes, update_bullet_hit_flash_vfx)
                    .run_if(
                        any_exist::<BulletHitFlash>()
                            .or(any_exist::<BulletHitFlashVfx>()),
                    )
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
            // VFX
            .add_systems(
                Update,
                update_muzzle_flashes
                    .run_if(any_exist::<MuzzleFlash>())
                    .run_if(is_spell_effects_active),
            );
    }
}
