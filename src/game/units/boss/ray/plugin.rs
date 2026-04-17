use bevy::prelude::*;

use super::components::{Ray, RayBeamVisual, RayEyeDying, RayMindControlBeam, RayDisintegrateBeam, RayEye, RayFearBeam, RayPetrificationBeam, RayStalkParticle, RayTeleportBubble};
use crate::game::units::components::Petrified;
use super::resources;
use super::systems::*;
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;
use crate::game::units::components::FearModifier;

pub struct RayPlugin;

impl Plugin for RayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_ray_assets)
            .add_systems(
                Update,
                (
                    ray_movement.in_set(MovementCalculationSet),
                    ray_disintegration_sweep,
                    ray_petrification_beam,
                    update_ray_disintegrate_visuals
                        .run_if(any_with_component::<RayDisintegrateBeam>),
                    update_ray_petrification_visuals
                        .run_if(any_with_component::<RayPetrificationBeam>),
                    update_ray_eye_movement.run_if(any_with_component::<RayEye>),
                    spawn_ray_stalk_particles,
                    update_ray_stalk_particles
                        .run_if(any_with_component::<RayStalkParticle>),
                    update_ray_beam_visuals.run_if(any_with_component::<RayBeamVisual>),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Ray>),
            )
            .add_systems(
                Update,
                (
                    ray_fear_beam,
                    update_ray_fear_visuals.run_if(any_with_component::<RayFearBeam>),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Ray>),
            )
            .add_systems(
                Update,
                (
                    ray_teleport_eye,
                    update_ray_teleport_bubbles
                        .run_if(any_with_component::<RayTeleportBubble>),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Ray>),
            )
            .add_systems(
                Update,
                (
                    ray_mind_control_beam,
                    update_ray_mind_control_visuals.run_if(any_with_component::<RayMindControlBeam>),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Ray>),
            )
            .add_systems(
                Update,
                update_petrified_damage
                    .run_if(any_with_component::<Petrified>)
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    ray_body_damage_to_eyes,
                    ray_eye_death_check.after(ray_body_damage_to_eyes),
                    ray_all_eyes_dead_check.after(ray_eye_death_check),
                    ray_death_cleanup.after(ray_all_eyes_dead_check),
                )
                    .after(PostCombatSet)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Ray>),
            )
            .add_systems(
                Update,
                update_ray_dying_eyes
                    .run_if(any_with_component::<RayEyeDying>)
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    cleanse_fear_with_rage,
                    update_fear_movement.after(cleanse_fear_with_rage),
                )
                    .after(MovementCalculationSet)
                    .before(crate::game::units::ApplyTransformsSet)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<FearModifier>),
            );
    }
}
