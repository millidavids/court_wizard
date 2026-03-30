use bevy::prelude::*;

use super::dark_mage::DarkMagePlugin;
use super::hags::HagsPlugin;
use super::lich::LichPlugin;
use super::ogre::OgrePlugin;

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((OgrePlugin, HagsPlugin, LichPlugin, DarkMagePlugin));
    }
}
