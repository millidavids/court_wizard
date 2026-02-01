use bevy::prelude::*;

use crate::state::InGameState;

use super::chain_lightning::ChainLightningPlugin;
use super::disintegrate::DisintegratePlugin;
use super::finger_of_death::FingerOfDeathPlugin;
use super::fireball::FireballPlugin;
use super::guardian_circle::GuardianCirclePlugin;
use super::magic_missile::MagicMissilePlugin;
use super::raise_the_dead::RaiseTheDeadPlugin;
use super::systems;
use super::teleport::TeleportPlugin;
use super::wall_of_stone::plugin::WallOfStonePlugin;

/// Plugin that handles wizard spells and projectiles.
///
/// Registers systems for:
/// - Magic missile spell (MagicMissilePlugin)
/// - Disintegrate beam spell (DisintegratePlugin)
/// - Fireball spell (FireballPlugin)
/// - Guardian Circle spell (GuardianCirclePlugin)
/// - Chain Lightning spell (ChainLightningPlugin)
/// - Finger of Death spell (FingerOfDeathPlugin)
/// - Raise The Dead spell (RaiseTheDeadPlugin)
/// - Projectile movement
/// - Projectile collision detection
/// - Spell effect lifetime management
/// - Projectile cleanup
pub struct SpellsPlugin;

impl Plugin for SpellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MagicMissilePlugin,
            DisintegratePlugin,
            FireballPlugin,
            GuardianCirclePlugin,
            ChainLightningPlugin,
            FingerOfDeathPlugin,
            RaiseTheDeadPlugin,
            TeleportPlugin,
            WallOfStonePlugin,
        ))
        .add_systems(
            Update,
            (
                systems::move_projectiles,
                systems::check_projectile_collisions,
                systems::update_spell_effects,
                systems::despawn_distant_projectiles,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
