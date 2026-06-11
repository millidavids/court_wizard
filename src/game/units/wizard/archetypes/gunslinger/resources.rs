use bevy::prelude::*;

use super::components::GunType;

/// Per-gun ammo and reload state.
#[derive(Debug, Clone)]
pub struct AmmoState {
    pub current: u32,
    pub max: u32,
    pub reloading: bool,
    pub reload_timer: f32,
    pub reload_duration: f32,
    pub fire_cooldown: f32,
}

impl AmmoState {
    pub fn new(gun: GunType) -> Self {
        Self {
            current: gun.max_ammo(),
            max: gun.max_ammo(),
            reloading: false,
            reload_timer: 0.0,
            reload_duration: gun.reload_time(),
            fire_cooldown: 0.0,
        }
    }

    /// Returns reload progress as 0.0 to 1.0.
    pub fn reload_progress(&self) -> f32 {
        if !self.reloading || self.reload_duration <= 0.0 {
            return 0.0;
        }
        (self.reload_timer / self.reload_duration).min(1.0)
    }
}

/// Resource tracking all gun states for the gunslinger.
#[derive(Resource, Debug, Clone)]
pub struct GunState {
    pub selected_gun: GunType,
    pub ammo: [AmmoState; 5],
}

impl Default for GunState {
    fn default() -> Self {
        let guns = GunType::all();
        Self {
            selected_gun: GunType::MachineGun,
            ammo: [
                AmmoState::new(guns[0]),
                AmmoState::new(guns[1]),
                AmmoState::new(guns[2]),
                AmmoState::new(guns[3]),
                AmmoState::new(guns[4]),
            ],
        }
    }
}

impl GunState {
    /// Returns a reference to the ammo state for the currently selected gun.
    pub fn current_ammo(&self) -> &AmmoState {
        let idx = self.gun_index(self.selected_gun);
        &self.ammo[idx]
    }

    /// Returns a mutable reference to the ammo state for the currently selected gun.
    pub fn current_ammo_mut(&mut self) -> &mut AmmoState {
        let idx = self.gun_index(self.selected_gun);
        &mut self.ammo[idx]
    }

    /// Returns the index of a gun type in the ammo array.
    fn gun_index(&self, gun: GunType) -> usize {
        match gun {
            GunType::MachineGun => 0,
            GunType::Magnum => 1,
            GunType::RocketLauncher => 2,
            GunType::Shotgun => 3,
            GunType::Flamethrower => 4,
        }
    }
}

/// Tracks the looping flamethrower sound effect entity so it can be stopped.
#[derive(Resource, Default)]
pub struct FlamethrowerSfx {
    pub entity: Option<Entity>,
}
