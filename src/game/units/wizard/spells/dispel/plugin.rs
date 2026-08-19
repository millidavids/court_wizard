use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{DispelCooldown, DispelImpact, DispelProjectile, NullZone};
use super::systems;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};
use crate::game::units::wizard::spells::utils::tick_spell_cooldown;

pub struct DispelPlugin;

impl Plugin for DispelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tick_spell_cooldown::<DispelCooldown>
                    .run_if(any_exist::<DispelCooldown>())
                    .run_if(is_spell_effects_active),
                systems::handle_dispel_casting
                    .run_if(spell_is_primed(Spell::Dispel))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(is_spell_effects_active),
                systems::move_dispel_projectiles
                    .run_if(any_exist::<DispelProjectile>())
                    .run_if(is_spell_effects_active),
                systems::update_dispel_impacts
                    .run_if(any_exist::<DispelImpact>())
                    .run_if(is_spell_effects_active),
                systems::announce_dispel_wave
                    .run_if(any_exist::<DispelImpact>())
                    .run_if(is_spell_effects_active),
                // Intentionally uses `is_gameplay_running` instead of
                // `is_spell_effects_active`: the null zone must continue to
                // despawn spell effects and decrement its timer while the game
                // is running, even if the spell-effects layer is otherwise
                // paused (e.g. victory screen still ticking).
                systems::update_null_zones
                    .run_if(any_exist::<NullZone>())
                    .run_if(is_gameplay_running),
                // Both peers: ship DispelSpellEffect / DispelShield messages for
                // each impact's overlap with the *other* peer's spell-effect
                // ghosts. The owning peer then performs the authoritative
                // despawn via `receive_dispel_messages`. Runs on the host too —
                // a host-cast dispel has no other way to reach a guest-owned
                // effect, since `update_dispel_impacts` only touches effects the
                // local peer owns. Also gated `is_spell_effects_active` so the
                // forwarder stops firing when the game is paused — otherwise
                // stale dispel messages would pile up while the receiver is also
                // paused, and process all-at-once on resume.
                systems::forward_dispel_impacts_to_owner
                    .run_if(any_exist::<DispelImpact>())
                    .run_if(is_spell_effects_active)
                    .run_if(
                        crate::networking::session::is_multiplayer_guest
                            .or_else(crate::networking::session::is_multiplayer_host),
                    ),
                // Both peers: visual-tick the GHOST DispelImpact entities so the
                // expanding sphere animates for whoever did not cast it (the
                // main update_dispel_impacts is filtered out for ghosts to avoid
                // double-despawning the other peer's spell effects).
                systems::tick_ghost_dispel_impacts
                    .run_if(any_exist::<DispelImpact>())
                    .run_if(is_spell_effects_active)
                    .run_if(
                        crate::networking::session::is_multiplayer_guest
                            .or_else(crate::networking::session::is_multiplayer_host),
                    ),
            ),
        );
    }
}
