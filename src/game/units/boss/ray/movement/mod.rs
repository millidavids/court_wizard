//! Ray spawn, movement, body damage, eye lifecycle.

mod disintegration;
mod eye_movement;
mod fear_movement;
mod lifecycle;
mod particles;
mod petrification;
mod spawn;

pub use disintegration::ray_disintegration_sweep;
pub use eye_movement::update_ray_eye_movement;
pub use fear_movement::{cleanse_fear_with_rage, update_fear_movement};
pub use lifecycle::{
    ray_all_eyes_dead_check, ray_body_damage_to_eyes, ray_death_cleanup, ray_eye_death_check,
    update_ray_dying_eyes,
};
pub use particles::{spawn_ray_stalk_particles, update_ray_stalk_particles};
pub use petrification::{ray_petrification_beam, update_ray_petrification_visuals};
pub use spawn::ray_movement;
pub use spawn::spawn_ray;
pub(crate) use spawn::ray_sfx_volume;
pub use disintegration::update_ray_beam_visuals;
