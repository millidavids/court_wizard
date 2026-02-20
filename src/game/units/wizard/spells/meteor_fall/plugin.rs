//! Meteor Fall spell plugin.

use bevy::prelude::*;

use super::components::{
    MeteorExplosion, MeteorFallCircleIndicator, MeteorFallStorm, MeteorGroundFire, MeteorProjectile,
};
use super::systems::*;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::*;
use crate::game::run_conditions::is_gameplay_running;

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
                // Casting system (only when MeteorFall is primed and mouse conditions met)
                handle_meteor_fall_casting
                    .run_if(spell_is_primed(Spell::MeteorFall))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Circle indicator updates
                update_circle_indicator.run_if(any_exist::<MeteorFallCircleIndicator>()),
                // Storm systems (spawn projectiles, update physics, check collisions)
                spawn_meteor_projectiles.run_if(any_exist::<MeteorFallStorm>()),
                update_meteor_projectiles.run_if(any_exist::<MeteorProjectile>()),
                check_meteor_collisions.run_if(any_exist::<MeteorProjectile>()),
                // Explosion updates
                update_meteor_explosions.run_if(any_exist::<MeteorExplosion>()),
                // Ground fire systems (chained for correct ordering)
                (
                    apply_ground_fire_damage,
                    fade_ground_fire,
                    cleanup_ground_fire,
                )
                    .chain()
                    .run_if(any_exist::<MeteorGroundFire>()),
            )
                .run_if(is_gameplay_running),
        );
    }
}
