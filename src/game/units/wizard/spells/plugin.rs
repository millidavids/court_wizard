use bevy::prelude::*;

use crate::state::InGameState;

use super::disintegrate::DisintegratePlugin;
use super::fireball::FireballPlugin;
use super::guardian_circle::GuardianCirclePlugin;
use super::magic_missile::MagicMissilePlugin;
use super::systems;

/// Plugin that handles wizard spells and projectiles.
///
/// Registers systems for:
/// - Magic missile spell (MagicMissilePlugin)
/// - Disintegrate beam spell (DisintegratePlugin)
/// - Fireball spell (FireballPlugin)
/// - Guardian Circle spell (GuardianCirclePlugin)
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
