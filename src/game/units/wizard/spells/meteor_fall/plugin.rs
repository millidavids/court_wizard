//! Meteor Fall spell plugin.

use bevy::prelude::*;

use super::components::{MeteorExplosion, MeteorFallStorm, MeteorGroundFire, MeteorProjectile};
use super::systems::*;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::*;

/// Plugin for the Meteor Fall spell.
///
/// Handles spell casting, meteor projectile spawning, physics, explosions,
/// and persistent ground fire hazards.
pub struct MeteorFallPlugin;

impl Plugin for MeteorFallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                handle_meteor_fall_casting
                    .run_if(spell_is_primed(Spell::MeteorFall))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Storm → projectile pipeline (chained for correct ordering)
                (
                    spawn_meteor_projectiles.run_if(any_exist::<MeteorFallStorm>()),
                    process_extinction_event.run_if(any_exist::<MeteorFallStorm>()),
                    update_meteor_projectiles.run_if(any_exist::<MeteorProjectile>()),
                    spawn_meteor_smoke_trail.run_if(any_exist::<MeteorProjectile>()),
                    check_meteor_collisions.run_if(any_exist::<MeteorProjectile>()),
                )
                    .chain(),
                // Explosion: animate (runs on ghost copies too) then host-only
                // damage + despawn.
                (animate_meteor_explosions, apply_meteor_explosion_damage)
                    .chain()
                    .run_if(any_exist::<MeteorExplosion>()),
                // Ground fire systems (chained for correct ordering)
                (
                    spawn_ground_fire_particles,
                    apply_ground_fire_damage,
                    fade_ground_fire,
                    cleanup_ground_fire,
                )
                    .chain()
                    .run_if(any_exist::<MeteorGroundFire>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
