use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{ArcaneBarrage, MagicMissile, MagicMissileCooldown};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

/// Plugin that handles magic missile spell casting and behavior.
pub struct MagicMissilePlugin;

impl Plugin for MagicMissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::tick_magic_missile_cooldown
                    .run_if(any_exist::<MagicMissileCooldown>())
                    .run_if(is_spell_effects_active),
                systems::handle_magic_missile_casting
                    .run_if(spell_is_primed(Spell::MagicMissile))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(is_spell_effects_active),
                systems::update_arcane_barrage
                    .run_if(any_exist::<ArcaneBarrage>())
                    .run_if(is_spell_effects_active),
                (
                    systems::move_magic_missiles,
                    systems::check_magic_missile_collisions,
                    systems::despawn_distant_magic_missiles,
                )
                    .chain()
                    .run_if(any_exist::<MagicMissile>())
                    .run_if(is_spell_effects_active),
            ),
        );
    }
}
