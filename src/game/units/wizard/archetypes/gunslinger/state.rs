//! Gun state management and aim helpers.

use bevy::prelude::*;

use super::messages::{ReloadMessage, SelectGunMessage};
use super::resources::{FlamethrowerSfx, GunState};
use crate::config::input_bindings::InputBindings;
use crate::game::units::wizard::spells::utils::local_spell_origin_snapshot;

/// Gun spawn position — `local_spell_origin + (0, 30, 0)`. Computed each
/// call from the lock-free `LocalSpellOrigin` snapshot so the multiplayer
/// guest's aim calculations use their own wizard's position.
fn gun_spawn_pos() -> Vec3 {
    let origin = local_spell_origin_snapshot();
    Vec3::new(origin.x, origin.y + 30.0, origin.z)
}

/// Initialize gun state when entering gameplay.
pub fn init_gun_state(mut commands: Commands) {
    commands.init_resource::<GunState>();
    commands.init_resource::<FlamethrowerSfx>();
}

/// Reset gun state when leaving gameplay.
pub fn reset_gun_state(mut commands: Commands) {
    commands.remove_resource::<GunState>();
    commands.remove_resource::<FlamethrowerSfx>();
}

/// Process gun selection messages.
pub fn process_gun_selection(
    mut messages: MessageReader<SelectGunMessage>,
    mut gun_state: ResMut<GunState>,
) {
    for message in messages.read() {
        gun_state.selected_gun = message.gun;
    }
}

/// Handle reload key for manual reload.
pub fn handle_reload_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut reload_message: MessageWriter<ReloadMessage>,
) {
    if let Some(key) = bindings.warglock.reload
        && keyboard.just_pressed(key)
    {
        reload_message.write(ReloadMessage);
    }
}

/// Process manual reload messages.
pub fn process_manual_reload(
    mut messages: MessageReader<ReloadMessage>,
    mut gun_state: ResMut<GunState>,
) {
    for _ in messages.read() {
        let ammo = gun_state.current_ammo_mut();
        if !ammo.reloading && ammo.current < ammo.max {
            ammo.reloading = true;
            ammo.reload_timer = 0.0;
        }
    }
}

/// Tick reload timers and fire cooldowns for all guns.
pub fn tick_gun_timers(time: Res<Time>, mut gun_state: ResMut<GunState>) {
    let dt = time.delta_secs();
    for ammo in &mut gun_state.ammo {
        if ammo.fire_cooldown > 0.0 {
            ammo.fire_cooldown -= dt;
        }
        if ammo.reloading {
            ammo.reload_timer += dt;
            if ammo.reload_timer >= ammo.reload_duration {
                ammo.reloading = false;
                ammo.reload_timer = 0.0;
                ammo.current = ammo.max;
            }
        }
    }
}

/// Auto-reload when ammo hits 0.
pub fn auto_reload(mut gun_state: ResMut<GunState>) {
    for ammo in &mut gun_state.ammo {
        if ammo.current == 0 && !ammo.reloading {
            ammo.reloading = true;
            ammo.reload_timer = 0.0;
        }
    }
}

/// Returns the direction and distance from the gun spawn position toward the cursor world position.
pub(super) fn aim_at_cursor(cursor_pos: Option<Vec3>) -> Option<(Vec3, f32)> {
    cursor_pos.and_then(|target| {
        let diff = target - gun_spawn_pos();
        let distance = diff.length();
        if distance < 1.0 {
            None
        } else {
            Some((diff / distance, distance))
        }
    })
}

/// Apply spread rotation around Y axis to a direction vector.
pub(super) fn apply_spread(dir: Vec3, spread_angle: f32) -> Vec3 {
    Quat::from_rotation_y(spread_angle) * dir
}

/// Apply cone spread — rotate by a random angle around a random axis perpendicular to dir.
pub(super) fn apply_cone_spread(dir: Vec3, max_spread: f32, rng: &mut impl rand::Rng) -> Vec3 {
    let spread_angle = rng.random_range(0.0..max_spread);
    let random_rotation = rng.random_range(0.0..std::f32::consts::TAU);
    // Pick an arbitrary perpendicular axis, then rotate it around dir
    let perp = if dir.y.abs() < 0.9 {
        dir.cross(Vec3::Y).normalize()
    } else {
        dir.cross(Vec3::X).normalize()
    };
    let rotated_perp = Quat::from_axis_angle(dir, random_rotation) * perp;
    Quat::from_axis_angle(rotated_perp, spread_angle) * dir
}
